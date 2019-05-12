//! Writing php extensions in rust made easy
//!
//! # Type casting
//!
//! Ivory automatically converts method parameters and return value from and to php compatible zval's
//!
//! The following types are supported for conversion.
//!
//! - rust signed and unsigned types up to 64bit to/from php `long`
//! - rust `f64` and `f32` to/from php `double`
//! - rust `bool` to/from php `bool`
//! - rust `String` to/from php `string`
//! - rust `Vec<T>` to/from php `array`
//! - rust `Vec<(u64, T)>` to/from php `array`
//!
//! Where `T` is a type that can be converted from/to php

#[macro_use]
pub mod macros;
pub mod error;

pub mod externs;
pub mod info;
mod phpval;
pub mod zend;
pub use crate::error::{ArgError, CastError};
pub use crate::phpval::{ArrayKey, PhpVal};
pub use ivory_macro::{ivory_export, ivory_module};
