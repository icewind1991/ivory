use std::intrinsics::transmute;
use std::mem::size_of;

use crate::externs::printf;

use super::bindings::{zend_execute_data, zval};

#[derive(Clone)]
#[repr(transparent)]
pub struct ZVal(zval);

impl ZVal {
    pub unsafe fn as_i64(&self) -> i64 {
        self.0.value.lval as i64
        //self.0.u1.type_info as i64
    }
}

#[repr(transparent)]
pub struct ExecuteData(zend_execute_data);

impl ExecuteData {
    pub fn num_args(&self) -> u32 {
        unsafe { self.0.This.u2.num_args }
    }

    fn get_arg_base(&self) -> *const ZVal {
        let offset = (size_of::<zend_execute_data>() + size_of::<zval>() - 1) / size_of::<zval>();
        let self_ptr: *const zend_execute_data = &self.0;
        unsafe {
            transmute::<_, *const ZVal>(self_ptr).add(5)
        }
    }

    pub unsafe fn get_arg(&self, i: u32) -> &ZVal {
        unsafe {
            let base = self.get_arg_base();
            let val_ptr = base.add(i as usize);
            &*val_ptr
        }
    }

    pub fn args<'a>(&'a self) -> ArgIterator<'a> {
        ArgIterator {
            base: self.get_arg_base(),
            count: self.num_args(),
            item: 0,
            lifetime: &()
        }
    }
}

pub struct ArgIterator<'a> {
    base: *const ZVal,
    count: u32,
    item: u32,
    lifetime: &'a ()
}

impl<'a> Iterator for ArgIterator<'a> {
    type Item = &'a ZVal;

    fn next(&mut self) -> Option<Self::Item> {
        if self.item < self.count {
            let val = unsafe { &*(self.base.add(self.item as usize)) };
            self.item += 1;
            Some(val)
        } else {
            None
        }
    }
}

#[repr(u8)]
enum ZValType {
    Undef = 0,
    Null = 1,
    False = 2,
    True = 3,
    Long = 4,
    Double = 5,
    String = 6,
    Array = 7,
    Object = 8,
    Resource = 9,
    Reference = 10,
}

//impl From<ZVal> for Option<i64> {
//    fn from(val: ZVal) -> Self {}
//}