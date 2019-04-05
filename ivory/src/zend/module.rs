use std;
use std::mem;
use std::os::raw::{c_char, c_int, c_uchar, c_uint, c_ushort, c_void};

use crate::zend::function::{ArgInfo, Function};

pub(crate) type StartupFunc = extern "C" fn(type_: c_int, module_number: c_int) -> c_int;
pub(crate) type ShutdownFunc = extern "C" fn(type_: c_int, module_number: c_int) -> c_int;
pub(crate) type InfoFunc = extern "C" fn();
pub(crate) type GlobalsCtorFunc = extern "C" fn(global: *const c_void) -> c_void;
pub(crate) type GlobalsDtorFunc = extern "C" fn(global: *const c_void) -> c_void;
pub(crate) type PostDeactivateFunc = extern "C" fn() -> c_int;

pub struct ModuleDep {}

pub struct INI {}

#[repr(C)]
pub struct ModuleInternal {
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
    globals_size: usize,
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

impl ModuleInternal {
    pub fn new(name: *const c_char, version: *const c_char) -> ModuleInternal {
        ModuleInternal {
            size: mem::size_of::<ModuleInternal>() as u16,
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

    pub fn set_functions(&mut self, funcs: &'static [Function]) {
        self.functions = funcs.as_ptr();
    }
}

pub struct FunctionMeta {
    pub name: *const c_char,
    pub func: *const c_void,
    pub args: &'static [ArgInfo],
}

impl FunctionMeta {
    pub fn as_function(&self) -> Function {
        if self.args.len() == 1 {
            // first arg is argument count which is always added
            Function::new(self.name, self.func)
        } else {
            Function::new_with_args(self.name, self.func, self.args, self.args.len() as u32 - 1)
        }
    }
}

pub struct PhpModule {
    pub name: *const c_char,
    pub version: *const c_char,
    pub info: &'static [(&'static str, &'static str)],
}
