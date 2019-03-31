use std::ffi::CString;
use std::intrinsics::transmute;

use libc::*;

use ivory_sys::zend_error;

extern "C" {
    pub fn php_printf(format: *const c_char, ...) -> size_t;
}

pub fn printf<T: Into<Vec<u8>>>(string: T) {
    let cstr = CString::new(string).unwrap();
    unsafe { php_printf(cstr.as_ptr()); }
}

#[repr(i32)]
pub enum ErrorLevel {
    Error = 1,
    Warning = 2,
    Parse = 4,
    Notice = 8,
    CoreError = 16,
    CoreWarning = 32,
    CompilerError = 64,
    CompilerWarning = 128,
    UserError = 256,
    UserWarning = 512,
    UserNotice = 1024,
    Strict = 2048,
    RecoverableError = 4096,
    Deprecated = 8192,
    UserDeprecated = 16384,
}

impl From<ErrorLevel> for i32 {
    fn from(from: ErrorLevel) -> Self {
        unsafe { transmute(from) }
    }
}

pub fn error<T: Into<Vec<u8>>>(level: ErrorLevel, message: T) {
    let cstr = CString::new(message).unwrap();
    unsafe { zend_error(level.into(), cstr.as_ptr()); }
}