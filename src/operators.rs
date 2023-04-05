pub mod all;
pub mod amb;
pub mod buffer_with_count;
pub mod combine_latest;
pub mod concat;
pub mod contains;
pub mod count;
pub mod default_if_empty;
pub mod distinct_until_changed;
pub mod element_at;
pub mod filter;
pub mod first;
pub mod flat_map;
pub mod group_by;
pub mod ignore_elements;
pub mod last;
pub mod map;
pub mod max;
pub mod merge;
pub mod min;
pub mod observe_on;
pub mod on_error_resume_next;
pub mod publish;
pub mod reduce;
pub mod ref_count;
pub mod retry;
pub mod retry_when;
pub mod sample;
pub mod scan;
pub mod sequence_equal;
pub mod skip;
pub mod skip_last;
pub mod skip_until;
pub mod skip_while;
pub mod start_with;
pub mod subscribe_on;
pub mod sum;
pub mod sum_and_count;
pub mod switch_on_next;
pub mod take;
pub mod take_last;
pub mod take_until;
pub mod take_while;
pub mod tap;
pub mod to_vec;
pub mod window_with_count;
pub mod zip;

#[cfg(not(feature = "web"))]
pub mod debounce;
#[cfg(not(feature = "web"))]
pub mod delay;
#[cfg(not(feature = "web"))]
pub mod time_interval;
#[cfg(not(feature = "web"))]
pub mod timeout;
#[cfg(not(feature = "web"))]
pub mod timestamp;

pub mod operators {
  pub use crate::operators::all::*;
  pub use crate::operators::amb::*;
  pub use crate::operators::buffer_with_count::*;
  pub use crate::operators::combine_latest::*;
  pub use crate::operators::concat::*;
  pub use crate::operators::contains::*;
  pub use crate::operators::count::*;
  pub use crate::operators::default_if_empty::*;
  pub use crate::operators::distinct_until_changed::*;
  pub use crate::operators::element_at::*;
  pub use crate::operators::filter::*;
  pub use crate::operators::first::*;
  pub use crate::operators::flat_map::*;
  pub use crate::operators::group_by::*;
  pub use crate::operators::ignore_elements::*;
  pub use crate::operators::last::*;
  pub use crate::operators::map::*;
  pub use crate::operators::max::*;
  pub use crate::operators::merge::*;
  pub use crate::operators::min::*;
  pub use crate::operators::observe_on::*;
  pub use crate::operators::on_error_resume_next::*;
  pub use crate::operators::publish::*;
  pub use crate::operators::reduce::*;
  pub use crate::operators::ref_count::*;
  pub use crate::operators::retry::*;
  pub use crate::operators::retry_when::*;
  pub use crate::operators::sample::*;
  pub use crate::operators::scan::*;
  pub use crate::operators::sequence_equal::*;
  pub use crate::operators::skip::*;
  pub use crate::operators::skip_last::*;
  pub use crate::operators::skip_until::*;
  pub use crate::operators::skip_while::*;
  pub use crate::operators::start_with::*;
  pub use crate::operators::subscribe_on::*;
  pub use crate::operators::sum::*;
  pub use crate::operators::sum_and_count::*;
  pub use crate::operators::switch_on_next::*;
  pub use crate::operators::take::*;
  pub use crate::operators::take_last::*;
  pub use crate::operators::take_until::*;
  pub use crate::operators::take_while::*;
  pub use crate::operators::tap::*;
  pub use crate::operators::to_vec::*;
  pub use crate::operators::window_with_count::*;
  pub use crate::operators::zip::*;

  #[cfg(not(feature = "web"))]
  pub use crate::operators::debounce::*;
  #[cfg(not(feature = "web"))]
  pub use crate::operators::delay::*;
  #[cfg(not(feature = "web"))]
  pub use crate::operators::time_interval::*;
  #[cfg(not(feature = "web"))]
  pub use crate::operators::timeout::*;
  #[cfg(not(feature = "web"))]
  pub use crate::operators::timestamp::*;
}
