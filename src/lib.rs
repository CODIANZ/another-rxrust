//! A different implementation than `rxRust` for easier use of `ReactiveX` in `Rust`.
//! This library based on [ReactiveX](http://reactivex.io/).

#![doc = include_str!("../README.md")]

#[macro_use]
mod macros;

pub mod internals;
pub mod material;
pub mod observable;
pub mod observables;
pub mod observer;
pub mod operators;
pub mod rx_error;
pub mod schedulers;
pub mod subjects;
pub mod subscription;
pub mod utils;

#[cfg(feature = "web")]
pub mod web;

pub mod prelude {
  pub use crate::material::*;
  pub use crate::observable::*;
  pub use crate::observables::*;
  pub use crate::observer::*;
  pub use crate::operators::*;
  pub use crate::rx_error::*;
  pub use crate::schedulers::*;
  pub use crate::subjects::*;
  pub use crate::subscription::*;
  pub use crate::utils::*;

  #[cfg(feature = "web")]
  pub use crate::web::*;
}

#[cfg(test)]
pub mod tests;
