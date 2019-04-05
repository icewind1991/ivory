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
fn is_positive(input: i64) -> bool {
    input >= 0
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
