pub mod defer;
pub mod empty;
pub mod error;
pub mod from_iter;
pub mod from_result;
pub mod just;
pub mod never;
pub mod range;
pub mod repeat;
pub mod start;

#[cfg(not(feature = "web"))]
pub mod interval;
#[cfg(not(feature = "web"))]
pub mod timer;

pub mod observables {
  pub use crate::observables::defer::*;
  pub use crate::observables::empty::*;
  pub use crate::observables::error::*;
  pub use crate::observables::from_iter::*;
  pub use crate::observables::from_result::*;
  pub use crate::observables::just::*;
  pub use crate::observables::never::*;
  pub use crate::observables::range::*;
  pub use crate::observables::repeat::*;
  pub use crate::observables::start::*;

  #[cfg(not(feature = "web"))]
  pub use crate::observables::interval::*;
  #[cfg(not(feature = "web"))]
  pub use crate::observables::timer::*;
}
