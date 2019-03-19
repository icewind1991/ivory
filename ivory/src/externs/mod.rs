use std::ffi::{CString};

use libc::*;

extern "C" {
    pub fn php_printf(format: *const c_char, ...) -> size_t;
}

pub fn printf<T: Into<Vec<u8>>>(string: T) {
    let cstr = CString::new(string).unwrap();
    unsafe { php_printf(cstr.as_ptr()); }
}