use std::{cell::RefCell, rc::Rc, time};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::window;

pub fn set_timeout<F>(f: F, dur: time::Duration)
where
  F: Fn() + 'static,
{
  let c = Rc::new(RefCell::new(None::<Closure<dyn Fn()>>));
  let c_in_closure = Rc::clone(&c);
  let closure = Closure::new({
    move || {
      f();
      c_in_closure.borrow(); // keep alive
    }
  });
  *c.borrow_mut() = Some(closure);

  let binding = c.borrow();
  let cc = binding.as_ref().unwrap().as_ref().unchecked_ref();
  window()
    .unwrap()
    .set_interval_with_callback_and_timeout_and_arguments_0(
      cc,
      dur.as_millis() as i32,
    )
    .expect("what??");
}
