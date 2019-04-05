use lazy_static::lazy_static;
use std::sync::Mutex;

use crate::FunctionDefinition;

lazy_static! {
    static ref FUNCTION_NAMES: Mutex<Vec<FunctionDefinition>> = Mutex::new(Vec::new());
}

pub(crate) fn cache_function(func: FunctionDefinition) {
    FUNCTION_NAMES.lock().unwrap().push(func);
}

pub(crate) fn get_functions() -> Vec<FunctionDefinition> {
    FUNCTION_NAMES.lock().unwrap().clone()
}
