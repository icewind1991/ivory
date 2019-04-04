use std::collections::HashMap;
use std::fmt;
use std::intrinsics::transmute;
use std::mem::size_of;
use std::os::raw::c_char;
use std::str;

use ivory_sys::{zend_execute_data, zend_string, zval};

use crate::CastError;
use std::fmt::Display;
use std::hash::Hash;

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

unsafe fn zend_str_as_string(str: &zend_string) -> String {
    let len = str.len;
    let base: *const c_char = &str.val[0];
    let slice: &[u8] = std::slice::from_raw_parts(base as *const u8, len);
    str::from_utf8_unchecked(slice).to_string()
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
        let str = *self.0.value.str;
        zend_str_as_string(&str)
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

#[derive(Debug, PartialEq)]
pub enum ArrayKey {
    String(String),
    Int(u64),
}

macro_rules! impl_from_array_key {
    ($type:ty, $variant:ident, $type2:ty) => {
        impl From<$type> for ArrayKey {
            fn from(input: $type) -> Self {
                ArrayKey::$variant(input as $type2)
            }
        }
    };
}

impl_from_array_key!(String, String, String);
impl_from_array_key!(u64, Int, u64);
impl_from_array_key!(u32, Int, u64);
impl_from_array_key!(u16, Int, u64);
impl_from_array_key!(u8, Int, u64);
impl_from_array_key!(usize, Int, u64);

#[derive(Debug, PartialEq)]
pub enum PhpVal {
    Undef,
    Null,
    Bool(bool),
    Long(i64),
    Double(f64),
    String(String),
    Array(Vec<(ArrayKey, PhpVal)>),
    Object(HashMap<String, PhpVal>),
    Resource(u64),
    Reference(),
}

impl PhpVal {
    pub fn get_type(&self) -> ZValType {
        match self {
            PhpVal::Undef => ZValType::Undef,
            PhpVal::Null => ZValType::Null,
            PhpVal::Bool(true) => ZValType::True,
            PhpVal::Bool(false) => ZValType::False,
            PhpVal::Long(_) => ZValType::Long,
            PhpVal::Double(_) => ZValType::Double,
            PhpVal::String(_) => ZValType::String,
            PhpVal::Array(_) => ZValType::Array,
            PhpVal::Object(_) => ZValType::Object,
            PhpVal::Resource(_) => ZValType::Resource,
            PhpVal::Reference() => ZValType::Reference,
        }
    }
}

impl Default for PhpVal {
    fn default() -> Self {
        PhpVal::Undef
    }
}

impl From<PhpVal> for Result<PhpVal, CastError> {
    fn from(val: PhpVal) -> Self {
        Ok(val)
    }
}

macro_rules! impl_from_phpval {
    ($type:ty, $variant:ident) => {
        // non nullable version
        impl From<PhpVal> for Result<$type, CastError> {
            fn from(val: PhpVal) -> Self {
                match val {
                    PhpVal::$variant(val) => Ok(val),
                    _ => Err(CastError {
                        actual: val.get_type(),
                    }),
                }
            }
        }

        // nullable version
        impl From<PhpVal> for Result<Option<$type>, CastError> {
            fn from(val: PhpVal) -> Self {
                match val {
                    PhpVal::Null => Ok(None),
                    PhpVal::Undef => Ok(None),
                    PhpVal::$variant(val) => Ok(Some(val)),
                    _ => Err(CastError {
                        actual: val.get_type(),
                    }),
                }
            }
        }

        impl From<$type> for PhpVal {
            fn from(input: $type) -> Self {
                PhpVal::$variant(input)
            }
        }
    };
}

impl_from_phpval!(i64, Long);
impl_from_phpval!(f64, Double);
impl_from_phpval!(bool, Bool);
impl_from_phpval!(String, String);

impl<T: Into<PhpVal>> From<Option<T>> for PhpVal {
    fn from(input: Option<T>) -> Self {
        match input {
            Some(inner) => inner.into(),
            None => PhpVal::Null,
        }
    }
}

impl<T: Into<PhpVal>> From<Vec<T>> for PhpVal {
    fn from(input: Vec<T>) -> Self {
        PhpVal::Array(
            input
                .into_iter()
                .enumerate()
                .map(|(key, value)| (key.into(), value.into()))
                .collect(),
        )
    }
}

impl<K: Into<ArrayKey>, T: Into<PhpVal>> From<Vec<(K, T)>> for PhpVal {
    fn from(input: Vec<(K, T)>) -> Self {
        PhpVal::Array(
            input
                .into_iter()
                .map(|(key, value)| (key.into(), value.into()))
                .collect(),
        )
    }
}

impl<K: Into<ArrayKey> + Hash + Eq, T: Into<PhpVal>> From<HashMap<K, T>> for PhpVal {
    fn from(input: HashMap<K, T>) -> Self {
        PhpVal::Array(
            input
                .into_iter()
                .map(|(key, value)| (key.into(), value.into()))
                .collect(),
        )
    }
}
