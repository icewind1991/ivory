use std::fmt::Debug;

use crate::imported::imported_fn;
use ivory::externs::printf;
use ivory::PhpVal;
use ivory::{ivory_export, ivory_module};

mod imported;

fn dump<T: Debug>(arg: T) {
    printf(format!("{:?}", arg));
}

#[ivory_export]
fn dump_arg(arg: PhpVal) {
    dump(arg);
}

#[ivory_export]
fn expect_long(arg: i64) {
    dump(arg);
}

#[ivory_export]
fn expect_double(arg: f64) {
    dump(arg);
}

#[ivory_export]
fn expect_string(arg: String) {
    dump(arg);
}

#[ivory_export]
fn expect_bool(arg: bool) {
    dump(arg);
}

#[ivory_export]
fn expect_option_bool(arg: Option<bool>) {
    dump(arg);
}
ivory_module!({
    name: "tests",
    version: "0.0.1",
    info: &[("test extension", "enabled")]
});
