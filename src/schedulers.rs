pub mod async_function_queue;
pub mod default_scheduler;
pub mod scheduler;

#[cfg(not(feature = "web"))]
pub mod new_thread_scheduler;

pub mod schedulers {
  pub use crate::schedulers::async_function_queue::*;
  pub use crate::schedulers::default_scheduler::*;
  pub use crate::schedulers::scheduler::*;

  #[cfg(not(feature = "web"))]
  pub use crate::schedulers::new_thread_scheduler::*;
}
