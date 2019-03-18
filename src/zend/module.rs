use std;
use std::mem;

use libc::*;

type StartupFunc = extern "C" fn(type_: c_int, module_number: c_int) -> c_int;
type ShutdownFunc = extern "C" fn(type_: c_int, module_number: c_int) -> c_int;
type InfoFunc = extern "C" fn();
type GlobalsCtorFunc = extern "C" fn(global: *const c_void) -> c_void;
type GlobalsDtorFunc = extern "C" fn(global: *const c_void) -> c_void;
type PostDeactivateFunc = extern "C" fn() -> c_int;
type HandlerFunc = extern "C" fn(execute_data: &ExecuteData, retval: &Value);

pub struct ExecuteData {}

pub struct Value {}

pub struct ModuleDep {}

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
        allow_null: c_uchar,
        is_variadic: c_uchar,
        by_reference: c_uchar,
    ) -> ArgInfo {
        ArgInfo {
            name,
            class_name: std::ptr::null(),
            type_hint: 0,
            pass_by_reference: by_reference,
            allow_null,
            is_variadic,
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

        let arg_count = ArgInfo::new(num_args as *const c_char, 0, 0, 0);
        args.insert(0, arg_count);

        let arg_ptr = args.as_ptr();
        mem::forget(args);

        Function {
            fname: name,
            handler: Some(handler),
            arg_info: arg_ptr,
            num_args,
            flags: 0,
        }
    }

    fn end() -> Function {
        Function {
            fname: std::ptr::null(),
            handler: None,
            arg_info: std::ptr::null(),
            num_args: 0,
            flags: 0,
        }
    }
}

pub struct INI {}

#[repr(C)]
pub struct Module {
    size: c_ushort,
    zend_api: c_uint,
    zend_debug: c_uchar,
    zts: c_uchar,
    ini_entry: *const INI,
    deps: *const ModuleDep,
    name: *const c_char,
    functions: *const Function,
    module_startup_func: Option<StartupFunc>,
    module_shutdown_func: Option<ShutdownFunc>,
    request_startup_func: Option<StartupFunc>,
    request_shutdown_func: Option<ShutdownFunc>,
    info_func: Option<InfoFunc>,
    version: *const c_char,
    globals_size: size_t,
    globals_ptr: *const c_void,
    globals_ctor: Option<GlobalsCtorFunc>,
    globals_dtor: Option<GlobalsDtorFunc>,
    post_deactivate_func: Option<PostDeactivateFunc>,
    module_started: c_int,
    type_: c_uchar,
    handle: *const c_void,
    module_number: c_int,
    build_id: *const c_char,
}

impl Module {
    pub fn new(name: *const c_char, version: *const c_char) -> Module {
        Module {
            size: mem::size_of::<Module>() as u16,
            zend_api: 20180731,
            zend_debug: 0,
            zts: 0,
            ini_entry: std::ptr::null(),
            deps: std::ptr::null(),
            name,
            functions: std::ptr::null(),
            module_startup_func: None,
            module_shutdown_func: None,
            request_startup_func: None,
            request_shutdown_func: None,
            info_func: None,
            version,
            globals_size: 0,
            globals_ptr: std::ptr::null(),
            globals_ctor: None,
            globals_dtor: None,
            post_deactivate_func: None,
            module_started: 0,
            type_: 0,
            handle: std::ptr::null(),
            module_number: 0,
            build_id: c_str!("API20180731,NTS"),
        }
    }

    pub fn set_startup_func(&mut self, func: StartupFunc) {
        self.module_startup_func = Some(func);
    }

    pub fn set_shutdown_func(&mut self, func: ShutdownFunc) {
        self.module_shutdown_func = Some(func);
    }

    pub fn set_info_func(&mut self, func: InfoFunc) {
        self.info_func = Some(func);
    }

    pub fn set_functions(&mut self, mut funcs: Vec<Function>) {
        funcs.push(Function::end());
        self.functions = funcs.as_ptr();
        mem::forget(funcs);
    }
}

unsafe impl Sync for Module {}
