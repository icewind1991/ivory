use libc::*;
use php_rs::info::{print_table_start, print_table_row, print_table_end};
use php_rs::zend::*;
use php_rs::*;

extern "C" {
    pub fn php_printf(format: *const c_char, ...) -> size_t;
}

#[no_mangle]
pub extern "C" fn php_module_startup(_type: c_int, _module_number: c_int) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn php_module_shutdown(_type: c_int, _module_number: c_int) -> c_int {
    0
}

#[no_mangle]
pub extern "C" fn php_module_info() {
    print_table_start();
    print_table_row(&["A demo PHP extension written in Rust", "enabled"]);
    print_table_end();
}

#[no_mangle]
pub extern "C" fn helloworld(_data: &ExecuteData, _retval: &Value) {
    unsafe { php_printf(c_str!("Hello world, Rust!")) };
}

#[no_mangle]
pub extern "C" fn get_module() -> *mut zend::Module {
    let mut entry = Box::new(zend::Module::new(c_str!("demo"), c_str!("0.1.0-dev")));

    entry.set_info_func(php_module_info);

    let args = vec![
        ArgInfo::new(c_str!("name"), false, false, false),
        ArgInfo::new(c_str!("foo"), false, false, false),
    ];

    let funcs = vec![
        Function::new(c_str!("helloworld"), helloworld),
        Function::new_with_args(c_str!("helloworld2"), helloworld, args),
    ];

    entry.set_functions(funcs);

    Box::into_raw(entry)
}
