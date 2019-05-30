use std::fmt;
use std::fmt::Display;
use std::intrinsics::transmute;
use std::mem::size_of;

use ivory_sys::*;

use crate::zend::array::parse_zend_array;
use crate::zend::string::{construct_zend_string, parse_zend_string};
use crate::{ArrayKey, PhpVal};

#[repr(transparent)]
pub struct ExecuteData(zend_execute_data);

impl ExecuteData {
    pub fn num_args(&self) -> u32 {
        unsafe { self.0.This.u2.num_args }
    }

    fn get_arg_base(&self) -> *const ZVal {
        let offset = (size_of::<zend_execute_data>() + size_of::<zval>() - 1) / size_of::<zval>();
        let self_ptr = (&self.0 as *const zend_execute_data) as *const ZVal;
        unsafe { self_ptr.add(offset) }
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

#[repr(transparent)]
pub struct ZVal(zval);

impl From<zval> for ZVal {
    fn from(val: zval) -> Self {
        ZVal(val)
    }
}

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
        parse_zend_string(self.0.value.str)
    }

    pub unsafe fn as_array(&self) -> Vec<(ArrayKey, PhpVal)> {
        parse_zend_array(*self.0.value.arr)
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

    // internal types
    ConstantAST = 11,
    Indirect = 13,
    Ptr = 14,
    Err = 15,

    // fake types for type hinting
    Bool = 16,
    Callable = 17,
    Iterable = 18,
    Void = 19,
    Number = 20,
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
            ZValType::ConstantAST => write!(f, "constant ast"),
            ZValType::Indirect => write!(f, "indirect"),
            ZValType::Ptr => write!(f, "pointer"),
            ZValType::Err => write!(f, "error"),
            ZValType::Bool => write!(f, "bool"),
            ZValType::Callable => write!(f, "callable"),
            ZValType::Iterable => write!(f, "iterable"),
            ZValType::Void => write!(f, "void"),
            ZValType::Number => write!(f, "number"),
        }
    }
}

impl From<ZValType> for u8 {
    fn from(val: ZValType) -> Self {
        unsafe { transmute(val) }
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
                    str: construct_zend_string(val),
                },
                u1: ty.into(),
                u2: _zval_struct__bindgen_ty_2 { extra: 0 },
            }),
            PhpVal::Array(vec) => {
                // unlike reading, when creating a zval array all the extra bits actually matter
                // so we just use zend's builtin methods for creating arrays
                unsafe {
                    let map: *mut _zend_array = _zend_new_array(vec.len() as u32);
                    let mut arr = zval {
                        value: zend_value { arr: map },
                        u1: ty.into(),
                        u2: _zval_struct__bindgen_ty_2 { extra: 0 },
                    };
                    let arr_ptr: *mut zval = &mut arr;
                    for (key, val) in vec.into_iter() {
                        let val_ptr: *mut zval =
                            Box::into_raw(Box::new(ZVal::from(val))) as *mut zval;
                        match key {
                            ArrayKey::Int(index) => {
                                add_index_zval(arr_ptr, index, val_ptr);
                            }
                            ArrayKey::String(key) => {
                                let key_len = key.len();
                                let bytes =
                                    Box::into_raw(key.into_bytes().into_boxed_slice()) as *const i8;
                                add_assoc_zval_ex(arr_ptr, bytes, key_len, val_ptr);
                            }
                        }
                    }

                    ZVal(arr)
                }
            }
            _ => unimplemented!(),
        }
    }
}
