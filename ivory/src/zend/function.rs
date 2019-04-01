use std;
use std::os::raw::{c_char, c_uchar};

use crate::zend::HandlerFunc;

#[derive(Clone)]
#[repr(C)]
pub struct ArgInfo {
    name: *const c_char,
    class_name: *const c_char,
    type_hint: c_uchar,
    pass_by_reference: c_uchar,
    allow_null: c_uchar,
    is_variadic: c_uchar,
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
    handler: Option<HandlerFunc>,
    arg_info: *const ArgInfo,
    num_args: u32,
    flags: u32,
}

impl Function {
    pub fn new(name: *const c_char, handler: HandlerFunc) -> Function {
        Function {
            fname: name,
            handler: Some(handler),
            arg_info: std::ptr::null(),
            num_args: 0,
            flags: 0,
        }
    }

    pub fn new_with_args(
        name: *const c_char,
        handler: HandlerFunc,
        args: &'static [ArgInfo],
    ) -> Function {
        let num_args = args.len() as u32;
        let mut args_vec = Vec::new();

        let arg_count = ArgInfo::new(num_args as *const c_char, false, false, false);
        args_vec.push(arg_count);
        args_vec.extend_from_slice(args);

        let arg_ptr = Box::into_raw(args_vec.into_boxed_slice()) as *const ArgInfo;

        Function {
            fname: name,
            handler: Some(handler),
            arg_info: arg_ptr,
            num_args,
            flags: 0,
        }
    }

    pub(crate) fn end() -> Function {
        Function {
            fname: std::ptr::null(),
            handler: None,
            arg_info: std::ptr::null(),
            num_args: 0,
            flags: 0,
        }
    }
}
