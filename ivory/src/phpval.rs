use std::collections::HashMap;
use std::hash::Hash;

use crate::zend::ZValType;
use crate::CastError;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
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

#[derive(Debug, PartialEq, Clone)]
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

impl From<()> for PhpVal {
    fn from(_input: ()) -> Self {
        PhpVal::Null
    }
}

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

impl<K: Into<ArrayKey> + Hash + Eq + Ord, T: Into<PhpVal>> From<HashMap<K, T>> for PhpVal {
    fn from(input: HashMap<K, T>) -> Self {
        let mut vec: Vec<(K, T)> = input.into_iter().collect();
        // since hashmap doesn't contain any stable order we sort it to get predictable results
        vec.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));
        vec.into()
    }
}
