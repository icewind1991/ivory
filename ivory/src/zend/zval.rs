use std::fmt;
use std::fmt::Display;
use std::intrinsics::transmute;
use std::mem::{forget, size_of};
use std::str;

use ivory_sys::*;

use crate::{ArrayKey, PhpVal};

#[repr(transparent)]
pub struct ExecuteData(zend_execute_data);

impl ExecuteData {
    pub fn num_args(&self) -> u32 {
        unsafe { self.0.This.u2.num_args }
    }

    fn get_arg_base(&self) -> *const ZVal {
        let offset = (size_of::<zend_execute_data>() + size_of::<zval>() - 1) / size_of::<zval>();
        let self_ptr: *const zend_execute_data = &self.0;
        unsafe { transmute::<_, *const ZVal>(self_ptr).add(offset) }
    }

    pub unsafe fn get_arg(&self, i: u32) -> &ZVal {
        let base = self.get_arg_base();
        let val_ptr = base.add(i as usize);
        &*val_ptr
    }

    pub fn args(&self) -> IntoArgIterator {
        IntoArgIterator {
            base: self.get_arg_base(),
            count: self.num_args(),
            item: 0,
        }
    }
}

pub struct IntoArgIterator {
    base: *const ZVal,
    count: u32,
    item: u32,
}

impl Iterator for IntoArgIterator {
    type Item = PhpVal;

    fn next(&mut self) -> Option<Self::Item> {
        if self.item < self.count {
            let val = unsafe { (*self.base.add(self.item as usize)).as_php_val() };
            self.item += 1;
            Some(val)
        } else {
            Some(PhpVal::Undef)
        }
    }
}

unsafe fn zend_str_as_string(string: *const zend_string) -> String {
    let len = (*string).len;
    let base: *const u8 = transmute(string);
    let str_start = base.add(size_of::<ZendStringHeader>());

    let slice: &[u8] = std::slice::from_raw_parts(str_start, len);
    str::from_utf8_unchecked(slice).to_string()
}

#[repr(C)]
pub struct ZendStringHeader {
    gc: zend_refcounted_h,
    h: zend_ulong,
    len: usize,
}

fn string_into_zend_str(string: String) -> *mut zend_string {
    let len = string.len();

    let mut mem: Vec<u8> = Vec::with_capacity(len + size_of::<ZendStringHeader>());

    let header = ZendStringHeader {
        gc: zend_refcounted_h {
            refcount: 1, // ?? no clue actually,
            u: _zend_refcounted_h__bindgen_ty_1 { type_info: 0 },
        },
        h: 0,
        len,
    };
    let header_bytes: [u8; size_of::<ZendStringHeader>()] = unsafe { transmute(header) };
    mem.extend_from_slice(&header_bytes);

    let bytes = string.into_bytes();

    mem.extend_from_slice(bytes.as_slice());

    let ptr = mem.as_ptr();

    // rust shouldn't deallocate the data
    // this isn't perfect since all the extra vec bits (length, capacity)
    // will be leaked
    // maybe this should be copied instead
    forget(mem);

    unsafe { transmute(ptr) }
}

#[repr(transparent)]
pub struct ZVal(zval);

impl ZVal {
    pub fn get_type(&self) -> ZValType {
        unsafe { self.0.u1.v.type_.into() }
    }

    pub unsafe fn as_i64(&self) -> i64 {
        self.0.value.lval
    }

    pub unsafe fn as_f64(&self) -> f64 {
        self.0.value.dval
    }

    pub unsafe fn as_str(&self) -> String {
        zend_str_as_string(self.0.value.str)
    }

    pub unsafe fn as_array(&self) -> Vec<(ArrayKey, PhpVal)> {
        let arr = *self.0.value.arr;
        let len = arr.nNumUsed;
        let mut result = Vec::new();
        for i in 0..len {
            let elem = *arr.arData.add(i as usize);
            let key = if elem.key.is_null() {
                ArrayKey::Int(elem.h)
            } else {
                ArrayKey::String(zend_str_as_string(&*elem.key))
            };
            let val: PhpVal = ZVal(elem.val).as_php_val();
            match val {
                PhpVal::Undef => {}
                _ => result.push((key, val)),
            }
        }
        result
    }

    pub fn as_php_val(&self) -> PhpVal {
        match self.get_type() {
            ZValType::Undef => PhpVal::Undef,
            ZValType::Null => PhpVal::Null,
            ZValType::False => PhpVal::Bool(false),
            ZValType::True => PhpVal::Bool(true),
            ZValType::Long => PhpVal::Long(unsafe { self.as_i64() }),
            ZValType::Double => PhpVal::Double(unsafe { self.as_f64() }),
            ZValType::String => PhpVal::String(unsafe { self.as_str() }),
            ZValType::Array => PhpVal::Array(unsafe { self.as_array() }),
            _ => PhpVal::Undef,
        }
    }
}

#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum ZValType {
    Undef = 0,
    Null = 1,
    False = 2,
    True = 3,
    Long = 4,
    Double = 5,
    String = 6,
    Array = 7,
    Object = 8,
    Resource = 9,
    Reference = 10,
}

impl Display for ZValType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ZValType::Undef => write!(f, "undefined"),
            ZValType::Null => write!(f, "null"),
            ZValType::False => write!(f, "bool"),
            ZValType::True => write!(f, "bool"),
            ZValType::Long => write!(f, "long"),
            ZValType::Double => write!(f, "double"),
            ZValType::String => write!(f, "string"),
            ZValType::Array => write!(f, "array"),
            ZValType::Object => write!(f, "object"),
            ZValType::Resource => write!(f, "resource"),
            ZValType::Reference => write!(f, "reference"),
        }
    }
}

impl From<u8> for ZValType {
    fn from(val: u8) -> Self {
        if val > 10 {
            panic!("invalid zval type");
        }
        unsafe { transmute(val) }
    }
}

impl From<ZValType> for _zval_struct__bindgen_ty_1 {
    fn from(ty: ZValType) -> Self {
        _zval_struct__bindgen_ty_1 {
            v: _zval_struct__bindgen_ty_1__bindgen_ty_1 {
                type_: ty as zend_uchar,
                type_flags: 0,
                u: _zval_struct__bindgen_ty_1__bindgen_ty_1__bindgen_ty_1 { extra: 0 },
            },
        }
    }
}

impl From<PhpVal> for ZVal {
    fn from(input: PhpVal) -> Self {
        let ty = input.get_type();
        match input {
            PhpVal::Long(val) => ZVal(zval {
                value: zend_value { lval: val },
                u1: ty.into(),
                u2: _zval_struct__bindgen_ty_2 { extra: 0 },
            }),
            PhpVal::Double(val) => ZVal(zval {
                value: zend_value { dval: val },
                u1: ty.into(),
                u2: _zval_struct__bindgen_ty_2 { extra: 0 },
            }),
            PhpVal::Undef => ZVal(zval {
                value: zend_value { lval: 0 },
                u1: ty.into(),
                u2: _zval_struct__bindgen_ty_2 { extra: 0 },
            }),
            PhpVal::Null => ZVal(zval {
                value: zend_value { lval: 0 },
                u1: ty.into(),
                u2: _zval_struct__bindgen_ty_2 { extra: 0 },
            }),
            PhpVal::Bool(val) => ZVal(zval {
                value: zend_value { lval: 0 },
                u1: _zval_struct__bindgen_ty_1 {
                    v: _zval_struct__bindgen_ty_1__bindgen_ty_1 {
                        type_: {
                            if val {
                                ZValType::True
                            } else {
                                ZValType::False
                            }
                        } as zend_uchar,
                        type_flags: 0,
                        u: _zval_struct__bindgen_ty_1__bindgen_ty_1__bindgen_ty_1 { extra: 0 },
                    },
                },
                u2: _zval_struct__bindgen_ty_2 { extra: 0 },
            }),
            PhpVal::String(val) => ZVal(zval {
                value: zend_value {
                    str: string_into_zend_str(val),
                },
                u1: ty.into(),
                u2: _zval_struct__bindgen_ty_2 { extra: 0 },
            }),
            _ => unimplemented!(),
        }
    }
}
