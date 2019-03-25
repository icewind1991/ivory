use std::intrinsics::transmute;

use ivory::*;
use ivory::externs::printf;
use ivory::zend::{ExecuteData, ZVal};

#[ivory_export]
fn hello_other(_other: String) {
    printf(format!("Hello ", ));
}

#[ivory_export]
fn hello_world() {
    printf("Hello world, Rust2!");
}


#[no_mangle]
pub extern "C" fn dump(data: *const ExecuteData, retval: *mut ZVal) {
    let data: &ExecuteData = unsafe { data.as_ref() }.unwrap();
    let num = data.num_args();
    for arg in data.args() {
        printf(format!("{}\n", unsafe { arg.as_i64() }));
    }
}

const FUNCTION_META_DUMP: ::ivory::zend::FunctionMeta = ::ivory::zend::FunctionMeta {
    name: { concat!("dump", "\0").as_ptr() as *const ::libc::c_char },
    func: dump,
    args: &[],
};

ivory_module!({
    name: "demo",
    version: "0.0.1",
    functions: &[hello_world, hello_other, dump],
    info: &[("demo extension", "enabled")]
});