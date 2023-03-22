use std::{cell::RefCell, rc::Rc};

pub type RxError = Rc<anyhow::Error>;
pub type RxFn<T> = Rc<RefCell<T>>;
