use ivory::*;
use ivory::externs::printf;
use ivory::zend::{ExecuteData, Value};

#[ivory_export]
fn hello_world() {
    printf("Hello world, Rust!");
}

ivory_module!({
    name: "demo",
    version: "0.0.1",
    functions: &[hello_world],
    info: &[("demo extension", "enabled")]
});