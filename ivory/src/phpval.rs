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
                #[allow(clippy::cast_lossless)]
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
    ($type:ty, $variant:ident, $type2:ty) => {
        // non nullable version
        impl From<PhpVal> for Result<$type2, CastError> {
            fn from(val: PhpVal) -> Self {
                match val {
                    PhpVal::$variant(val) => Ok(val as $type2),
                    _ => Err(CastError {
                        actual: val.get_type(),
                    }),
                }
            }
        }

        // nullable version
        impl From<PhpVal> for Result<Option<$type2>, CastError> {
            fn from(val: PhpVal) -> Self {
                match val {
                    PhpVal::Null => Ok(None),
                    PhpVal::Undef => Ok(None),
                    PhpVal::$variant(val) => Ok(Some(val as $type2)),
                    _ => Err(CastError {
                        actual: val.get_type(),
                    }),
                }
            }
        }

        impl From<$type2> for PhpVal {
            fn from(input: $type2) -> Self {
                PhpVal::$variant(input as $type)
            }
        }
    };
}

impl_from_phpval!(i64, Long, i64);
impl_from_phpval!(i64, Long, i32);
impl_from_phpval!(i64, Long, i16);
impl_from_phpval!(i64, Long, i8);
impl_from_phpval!(i64, Long, u64);
impl_from_phpval!(i64, Long, u32);
impl_from_phpval!(i64, Long, u16);
impl_from_phpval!(i64, Long, u8);
impl_from_phpval!(f64, Double, f64);
impl_from_phpval!(f64, Double, f32);
impl_from_phpval!(bool, Bool, bool);
impl_from_phpval!(String, String, String);

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
