use libc::*;
use std::ffi::CString;

extern "C" {
    pub fn php_info_print_table_start();
    pub fn php_info_print_table_row(num_cols: c_int, ...) -> c_void;
    pub fn php_info_print_table_end();
}

pub fn php_print_module_info(info: &[(&'static str, &'static str)]) {
    unsafe {
        php_info_print_table_start();
        for (key, value) in info {
            let v1 = CString::new(key.to_string()).unwrap();
            let v2 = CString::new(value.to_string()).unwrap();
            php_info_print_table_row(2, v1, v2);
        }
        php_info_print_table_end();
    }
}
