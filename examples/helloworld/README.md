# Example

## Usage

- build with `cargo +nightly build`
- run php with the module and call the defined method 
  ```bash
  php -d extension=target/debug/libhelloworld.so -r 'helloworld();'`
  ```