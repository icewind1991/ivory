# Ivory integration tests

A basic php module to test the transition between php and rust

# Usage

Due to the particularities with loading dynamic libraries in rust tests, the tests have to be rebuild *before* running the tests after making changes.

- `cargo build`
- `cargo test`