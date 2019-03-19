use ivory::*;
use ivory::externs::printf;
use ivory::zend::{ExecuteData, Value};

#[ivory_export]
fn hello_other(_other: String) {
    printf(format!("Hello ",));
}

#[ivory_export]
fn hello_world() {
    printf("Hello world, Rust!");
}

ivory_module!({
    name: "demo",
    version: "0.0.1",
    functions: &[hello_world, hello_other],
    info: &[("demo extension", "enabled")]
});