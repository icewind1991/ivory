use ivory::externs::printf;
use ivory::{ivory_export, ivory_module};

#[ivory_export]
fn hello_other(other: String) {
    printf(format!("Hello {}", other));
}

#[ivory_export]
fn hello_world() {
    printf("Hello world, Rust!");
}

#[ivory_export]
fn add1(input: i64) -> i64 {
    input + 1
}

#[ivory_export]
fn format_hello(other: String) -> String {
    format!("Hello {}", other)
}

ivory_module!({
    name: "demo",
    version: "0.0.1",
    info: &[("demo extension", "enabled")]
});
