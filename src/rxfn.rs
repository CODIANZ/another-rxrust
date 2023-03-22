use crate::all::*;
use std::{cell::RefCell, rc::Rc};

pub fn rxfn<T>(x: T) -> RxFn<T> {
  Rc::new(RefCell::new(x))
}
