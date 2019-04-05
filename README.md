# Ivory

Writing php extensions in rust made easy

## Usage

```
use ivory::{ivory_export, ivory_module};
use ivory::externs::printf;

/// Basic methods
#[ivory_export]
fn hello_world() {
    printf("Hello world, Rust!");
}

/// Automatically casts function arguments for php
#[ivory_export]
fn hello_other(other: String) {
    printf(format!("Hello {}", other));
}

/// And casts return types back to php
#[ivory_export]
fn add_one(input: i64) -> i64 {
    input + 1
}

/// Optional arguments
#[ivory_export]
fn hello(input: Option<String>) {
    printf(format!("Hello {}", other.unwrap_or("Rust".to_string())));
}

ivory_module!({
    name: "demo",
    version: "0.0.1",
    info: &[("demo extension", "enabled")]
});
```