

//! tests for metric backend (to test the global macro must be called at a crate root).

#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate metrics_backends;


pub use metrics_backends::*;
pub use metrics_backends::metrics_derive::{
  metrics_modules,
};

#[metrics_modules(pro,slogger)]
struct MetricStates {
  a_int_counter: Counter,
}

#[macro_export]
macro_rules! mets {
  (fast_only, $($exp:tt)*) => {
    $crate::metrics_backends::metrics_derive::metrics!(from_crate(metrics_backends_tests) [pro], $($exp)*)
	};
  ($($exp:tt)*) => {
    $crate::metrics_backends::metrics_derive::metrics!(from_crate(metrics_backends_tests) [pro, slogger], $($exp)*)
  };
}



#[cfg(test)]
mod test {
  use super::{
    Counter,
  };
  #[test]
  fn test_macros() {
    mets!(fast_only, a_int_counter, by(1), warn, target "anything", "some additional logs {}", 123);
    mets!(a_int_counter, inc());
  }
}
