# Example

## Usage

- build with `cargo build`
- run php with the module and call the defined method 
  ```bash
  php -d extension=../../target/debug/libhelloworld.so -r 'helloworld();'`
  ```