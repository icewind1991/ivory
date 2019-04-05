use lazy_static::lazy_static;
use std::sync::Mutex;

lazy_static! {
    static ref FUNCTION_NAMES: Mutex<Vec<String>> = Mutex::new(Vec::new());
}

pub fn cache_function(name: String) {
    FUNCTION_NAMES.lock().unwrap().push(name);
}

pub fn get_functions() -> Vec<String> {
    FUNCTION_NAMES.lock().unwrap().clone()
}
