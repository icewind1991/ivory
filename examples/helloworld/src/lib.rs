use std::intrinsics::transmute;

use ivory::*;
use ivory::externs::printf;
use ivory::zend::{ExecuteData, PhpVal, ZVal};

#[ivory_export]
fn hello_other(other: String) {
    printf(format!("Hello {}", other));
}

#[ivory_export]
fn hello_world() {
    printf("Hello world, Rust2!");
}

ivory_module!({
    name: "demo",
    version: "0.0.1",
    functions: &[hello_world, hello_other],
    info: &[("demo extension", "enabled")]
});