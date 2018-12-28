

//! tests for metric backend (to test the global macro must be called at a crate root).

#![feature(proc_macro_hygiene)]

#[macro_use]
extern crate metrics_backends;


pub use metrics_backends::*;
pub use metrics_backends::metrics_derive::*;

#[metrics_modules(pro,slogger)]
struct MetricStates {
  a_int_counter: Counter,
  a_timer_counter: Timer,
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

#[macro_export]
macro_rules! timer_enclose_backends {
  ($($exp:tt)*) => {
    $crate::metrics_backends::metrics_derive::metrics!(from_crate(metrics_backends_tests) [pro], $($exp)*)
  };
}
#[macro_export]
macro_rules! timer_enclose_backends_alt {
  ($($exp:tt)*) => {
    $crate::metrics_backends::metrics_derive::metrics!(from_crate(metrics_backends_tests) [pro, slogger], $($exp)*)
  };
}

#[cfg(test)]
mod test {
  use metrics_backends::metrics_derive::{
    timer_enclose,
    metrics,
  };


  #[timer_enclose(a_timer_counter)]
  fn to_time() -> usize {
    // some content
    let mut a = 5;
    a += 1;
    a
  }

  #[timer_enclose(a_timer_counter, timer_enclose_backends_alt)]
  fn to_time_alt() -> usize {
    // some content
    let mut a = 5;
    a += 2;
    a
  }

  use super::{
    Counter,
  };
  #[test]
  fn test_macros() {
    mets!(fast_only, a_int_counter, by(1), warn, target "anything", "some additional logs {}", 123);
    mets!(a_int_counter, inc());
  }
  #[test]
  fn test_timers() {
    mets!(a_timer_counter, start());
    mets!(a_timer_counter, suspend());
    let a = to_time();
    assert_eq!(a, 6);
    let a = to_time_alt();
    assert_eq!(a, 7);
  }

}
