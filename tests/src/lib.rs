use std::fmt::Debug;

use crate::imported::imported_fn;
use ivory::externs::printf;
use ivory::{ivory_export, ivory_module};
use ivory::{ArrayKey, PhpVal};

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

#[ivory_export]
fn return_long() -> i64 {
    1
}

#[ivory_export]
fn return_double() -> f64 {
    0.5
}

#[ivory_export]
fn return_true() -> bool {
    true
}

#[ivory_export]
fn return_false() -> bool {
    false
}

#[ivory_export]
fn return_string() -> String {
    "some string data".to_string()
}

#[ivory_export]
fn return_array_simple() -> Vec<i32> {
    vec![-10, 10, 0]
}

#[ivory_export]
fn return_array_gap() -> Vec<(u32, i32)> {
    vec![(0u32, -10), (1, 10), (10, 0)]
}

#[ivory_export]
fn return_array_mixed() -> Vec<(ArrayKey, PhpVal)> {
    vec![
        (0u32.into(), PhpVal::from(-10)),
        ("foo".to_string().into(), PhpVal::from(10)),
        ("bar".to_string().into(), PhpVal::from(0.5)),
    ]
}

#[ivory_export]
fn return_array_nested() -> Vec<Vec<i32>> {
    vec![vec![1, 2], vec![3, 4]]
}

ivory_module!({
    name: "tests",
    version: "0.0.1",
    info: &[("test extension", "enabled")]
});
