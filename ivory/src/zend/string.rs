use ivory_sys::*;
use std::alloc::{alloc, Layout};
use std::mem::size_of;
use std::ptr;
use std::str;

pub(super) unsafe fn parse_zend_string(string: *const zend_string) -> String {
    let len = (*string).len;
    let base = string as *const u8;
    let str_start = base.add(size_of::<ZendStringHeader>());

    let slice: &[u8] = std::slice::from_raw_parts(str_start, len);
    str::from_utf8_unchecked(slice).to_string()
}

#[repr(C)]
pub struct ZendStringHeader {
    gc: zend_refcounted_h,
    h: zend_ulong,
    len: usize,
}

pub(super) fn construct_zend_string(string: String) -> *mut zend_string {
    let len = string.len();

    let header_size = size_of::<ZendStringHeader>();

    let layout = Layout::from_size_align(len + header_size, size_of::<zend_string>())
        .expect("invalid layout");
    let raw = unsafe { alloc(layout) };

    let header = ZendStringHeader {
        gc: zend_refcounted_h {
            refcount: 1, // ?? no clue actually,
            u: _zend_refcounted_h__bindgen_ty_1 { type_info: 0 },
        },
        h: 0,
        len,
    };
    unsafe {
        ptr::write(raw as *mut ZendStringHeader, header);
        ptr::copy(string.as_ptr(), raw.add(header_size), len);
    };

    raw as *mut zend_string
}
