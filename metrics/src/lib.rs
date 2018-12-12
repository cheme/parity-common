// Copyright 2015-2018 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! POC/design for impactless metrics inclusion in parity-ethereum.
//! Single backend prometheus, but still a plugoff feature (here using `std` as activation feature)
//! The poc allows only one action per logging but target is multiple possible actions.

#![cfg_attr(not(feature = "std"), no_std)]
//#![cfg_attr(not(feature = "std"), feature(core_intrinsics))]
//#![cfg_attr(not(feature = "std"), feature(alloc))]

/// METRICS_DEF is byte content to define the metrics configuration
/// format is json as sample but undefined at this point.
/// The point is that we want direct pointer to things to store (no key value mapping for
/// instance), so initializing it at compilation from a conf seems to be an idea.
/// Note that it will need a procedural macro to do it, but it 
#[cfg(feature = "conf_proj1")]
const METRICS_DEF: [u8] = include_bytes!("./config/parity-ethereum.json"); // either json
#[cfg(feature = "conf_proj2")]
const METRICS_DEF: [u8] = include_bytes!("./config/parity-zcash.json"); // either json


#[cfg(feature = "std")]
#[macro_use]
extern crate lazy_static;
#[cfg(feature = "std")]
#[macro_use]
extern crate prometheus;


#[macro_export]
macro_rules! warnm {
  (metric($name:ident), $exp:expr) => {
    $crate::backend::inc::$name();
    warn!($exp)
	};
  (metric($name:ident, $laz:expr), $exp:expr) => {
    $crate::backend::by::$name($laz);
    warn!($exp)
	};
	($exp:expr) => {
    warn!($exp)
	};
}
#[macro_export]
macro_rules! infom {
  (metric($name:ident), $exp:expr) => {
    $crate::backend::inc::$name();
    info!($exp)
	};
  (metric($name:ident, $laz:expr), $exp:expr) => {
    $crate::backend::by::$name($laz);
    info!($exp)
	};
	($exp:expr) => {
    info!($exp)
	};
}
#[macro_export]
macro_rules! debugm {
  (metric($name:ident), $exp:expr) => {
    $crate::backend::inc::$name();
    debug!($exp)
	};
  (metric($name:ident, $laz:expr), $exp:expr) => {
    $crate::backend::by::$name($laz);
    debug!($exp)
	};
	($exp:expr) => {
    debug!($exp)
	};
}
#[macro_export]
macro_rules! do_metric {
  ($name:ident) => {
    $crate::backend::inc::$name();
	};
  ($name:ident, $laz:expr) => {
    $crate::backend::by::$name($laz);
    debug!($exp)
	};
}


/// this module should be genereated by METRICS_DEF by a simple proc_macro
/// (adding some named ref counter to the struct and other variants).
/// That way the configuration do not need to be directly in the crate but 
/// in any metrics project specific linked crate.
/// TODO not that good idea writing it directly is probably simplier.
/// still proc_macro could do quick recompile without changing code base.
/// Idea of having a proc_macro fetching the CONF??
/// -> not sure that it is doable (I could understand that being blocked).
#[cfg(feature = "std")]
pub mod backend {
  use prometheus::{IntCounter, IntCounterVec, IntGauge, IntGaugeVec};
  lazy_static! {
      static ref A_INT_COUNTER: IntCounter =
        register_int_counter!("A_int_counter", "foobar").expect("prometheus metrics start failure");
  }

  pub mod inc {
    /// generated function for metrics config defined counter
    pub fn a_int_counter() {
      super::A_INT_COUNTER.inc()
    }
  }
  /// mod for poc without proc macro: with a proc macro having a secific fn name is easy
  pub mod by {
    /// generated function for metrics config defined counter
    pub fn inc_by_A_INT_COUNTER(lazy_nb : impl Fn() -> i64) {
      super::A_INT_COUNTER.inc_by(lazy_nb())
    }
  }

}

#[cfg(not(feature = "std"))]
pub mod backend {
  pub mod inc {
    /// generated function for metrics config defined counter
    pub fn a_int_counter() {
    }
  }
  /// mod for poc without proc macro: with a proc macro having a secific fn name is easy
  pub mod by {
    /// generated function for metrics config defined counter
    pub fn inc_by_A_INT_COUNTER(_lazy_nb : impl Fn() -> i64) {
    }
  }
}

