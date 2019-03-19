use std;

use libc::*;
use crate::zend::HandlerFunc;

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
    pub fn new(
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
        mut args: Vec<ArgInfo>,
    ) -> Function {
        let num_args = args.len() as u32;

        let arg_count = ArgInfo::new(num_args as *const c_char, false, false, false);
        args.insert(0, arg_count);

        let arg_ptr = Box::into_raw(args.into_boxed_slice()) as *const ArgInfo;

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