use std::alloc::{alloc, Layout};
use std::cmp::max;
use std::mem::size_of;
use std::ptr;

use ivory_sys::*;

use crate::zend::string::{construct_zend_string, parse_zend_string};
use crate::zend::{ZVal, ZValType};
use crate::{ArrayKey, PhpVal};

pub(super) unsafe fn parse_zend_array(arr: zend_array) -> Vec<(ArrayKey, PhpVal)> {
    let len = arr.nNumUsed;
    let mut result = Vec::new();
    for i in 0..len {
        let elem = *arr.arData.add(i as usize);
        let key = if elem.key.is_null() {
            ArrayKey::Int(elem.h)
        } else {
            ArrayKey::String(parse_zend_string(&*elem.key))
        };
        let val: PhpVal = ZVal::from(elem.val).as_php_val();
        match val {
            PhpVal::Undef => {}
            _ => result.push((key, val)),
        }
    }
    result
}

// WIP pure rust implementation of array construction in order to be able to run conversion test without linking to php
pub(super) fn create_zend_array(vec: Vec<(ArrayKey, PhpVal)>) -> zend_array {
    let bucket_size = size_of::<Bucket>();
    let size_min = max(8, vec.len());
    let layout =
        Layout::from_size_align(bucket_size * size_min, bucket_size).expect("invalid layout");
    let bucket_mem: *mut Bucket = unsafe { alloc(layout) } as *mut Bucket;

    let array = zend_array {
        gc: _zend_refcounted_h {
            refcount: 1,
            u: _zend_refcounted_h__bindgen_ty_1 {
                type_info: u8::from(ZValType::Array) as u32,
            },
        },
        u: _zend_array__bindgen_ty_1 { flags: 1 << 3 }, // uninitialized
        nTableMask: u32::max_value() - 1,
        arData: bucket_mem,
        nNumUsed: vec.len() as u32,
        nNumOfElements: vec.len() as u32,
        nTableSize: size_min as u32,
        nInternalPointer: 0,
        nNextFreeElement: i64::min_value(),
        pDestructor: Some(zval_ptr_dtor),
    };

    let mut curr_bucket: *mut Bucket = bucket_mem;

    for (key, val) in vec.into_iter() {
        unsafe {
            match key {
                ArrayKey::Int(key) => {
                    (*curr_bucket).h = key;
                    (*curr_bucket).key = ptr::null_mut()
                }
                ArrayKey::String(key) => {
                    (*curr_bucket).h = hash_djbx33a(key.as_bytes());
                    (*curr_bucket).key = construct_zend_string(key)
                }
            }

            curr_bucket = curr_bucket.add(1);
        }
    }

    array
}

fn hash_djbx33a(data: &[u8]) -> u64 {
    let mut hash = 5381u64;
    for byte in data {
        hash = (hash * 33) + *byte as u64;
    }

    hash | 0x8000000000000000
}
