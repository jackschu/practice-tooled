use std::cell::RefCell;

thread_local! {pub static TIME: RefCell<f64> = RefCell::new(0.0)}
