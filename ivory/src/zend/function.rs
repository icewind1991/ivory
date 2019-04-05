use std;
use std::os::raw::{c_char, c_uchar, c_void};

#[derive(Clone)]
#[repr(C)]
pub struct ArgInfo {
    pub name: *const c_char,
    pub class_name: *const c_char,
    pub type_hint: c_uchar,
    pub pass_by_reference: c_uchar,
    pub allow_null: c_uchar,
    pub is_variadic: c_uchar,
}

impl ArgInfo {
    pub const fn new(
        name: *const c_char,
        allow_null: bool,
        is_variadic: bool,
        by_reference: bool,
    ) -> ArgInfo {
        ArgInfo {
            name,
            class_name: std::ptr::null(),
            type_hint: 0,
            pass_by_reference: by_reference as c_uchar,
            allow_null: allow_null as c_uchar,
            is_variadic: is_variadic as c_uchar,
        }
    }
}

#[repr(C)]
pub struct Function {
    fname: *const c_char,
    handler: *const c_void,
    arg_info: *const ArgInfo,
    num_args: u32,
    flags: u32,
}

impl Function {
    pub const fn new(name: *const c_char, handler: *const c_void) -> Function {
        Function {
            fname: name,
            handler,
            arg_info: std::ptr::null(),
            num_args: 0,
            flags: 0,
        }
    }

    pub const fn new_with_args(
        name: *const c_char,
        handler: *const c_void,
        args: &'static [ArgInfo],
        num_args: u32,
    ) -> Function {
        let arg_ptr = args.as_ptr();

        Function {
            fname: name,
            handler,
            arg_info: arg_ptr,
            num_args,
            flags: 0,
        }
    }

    pub const fn end() -> Function {
        Function {
            fname: std::ptr::null(),
            handler: std::ptr::null(),
            arg_info: std::ptr::null(),
            num_args: 0,
            flags: 0,
        }
    }
}
