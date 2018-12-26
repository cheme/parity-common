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

#![feature(proc_macro_hygiene)]
#![cfg_attr(not(feature = "std"), no_std)]
//#![cfg_attr(not(feature = "std"), feature(core_intrinsics))]
//#![cfg_attr(not(feature = "std"), feature(alloc))]

#[cfg(feature = "std")]
#[cfg(feature = "slogger")]
#[macro_use]
extern crate slog;

#[macro_use]
extern crate failure;

#[cfg(std)]
pub type Error = failure::Error;

#[cfg(not(std))]
pub type Error = ();


#[macro_use]
pub extern crate metrics_procedural;



// Currently unused TODO delete?
#[derive(Debug, Fail)]
pub enum MetricsError {
  #[fail(display = "an error: {}", label)]
  AnError{ label: String },
}

/// METRICS_DEF is byte content to define the metrics configuration
/// format is json as sample but undefined at this point.
/// The point is that we want direct pointer to things to store (no key value mapping for
/// instance), so initializing it at compilation from a conf seems to be an idea.
/// Note that it will need a procedural macro to do it, but it 
#[cfg(feature = "conf_proj1")]
const METRICS_DEF: [u8] = include_bytes!("./config/parity-ethereum.json"); // either json
#[cfg(feature = "conf_proj2")]
const METRICS_DEF: [u8] = include_bytes!("./config/parity-zcash.json"); // either json


/// drafting some spec
/// Those could be set from command line or ext file not at compile time
/// TODO some of those items does not make sense (specific should only be option delay...)
#[derive(Clone)]
pub struct GlobalCommonDef {
  dest: OutputDest,
  out_mode: OutputMode,
  /// delay between each write (if undefined no regular write)
  /// implies an async process
  out_delay: OutputDelay,
  /// write/flush on drop
  out_onclose: bool,
  /// listening chanel for write manually
  /// implies an async process
  chan_write: bool,
  // should we use slog_async enum?
  //overflow_strategie: OverflowStrategy,
}

/// static outdesc, resulting object will simply be `impl Write`
#[derive(Clone)]
pub enum OutputDest {
  File(Option<std::path::PathBuf>), // if None use pathbuf from constant str
  /// use the log macro to push content
  Logger,
  /// substrate telemetry crate to push on ws ?
  Telemetry,
  /// probably not a good idea (Trait)
  Custo(String),
}

#[derive(Clone)]
pub enum OutputDelay {
  Synch,
  Periodic(std::time::Duration),
  Unlimited,
}
#[derive(Clone)]
pub enum OutputMode {
  /// append content to existing content
  Append,
  /// append but delete existing content on init
  AppendNew,
  /// overwrite on each periodic or called write operation
  Overwrite,
}

#[macro_use]
pub extern crate log;

#[cfg(feature = "std")]
#[macro_use]
extern crate once_cell;
#[cfg(feature = "std")]
#[cfg(feature = "pro")]
#[macro_use]
extern crate prometheus;

// static all backends definition
/*macro_rules! BACKENDS(() => {
  [pro,empty]
}
);*/

/*#[macro_export]
macro_rules! metrics {
  ([$($be:ident),*], $name:ident, $action:ident: $laz:expr, $level:ident, target: $target:expr, $($arg:tt)+) => {
    $($crate::$be::$action::$name($laz);)*
    $crate::log::$level!(target: $target, $($arg)+)
	};
  ([$($be:ident),*], $name:ident, $action:ident: $laz:expr) => {
    $($crate::$be::$action::$name($laz);)*
	};
}*/

#[macro_export]
macro_rules! mets {
  ($($exp:tt)*) => {
    metrics!([pro, slogger], $($exp)*)
  };
  (fast_only, $($exp:tt)*) => {
    metrics!([pro], $($exp)*)
	};
}

macro_rules! metrics_defaults { () => {
  #[cfg(feature = "std")]
  static STATE: once_cell::sync::OnceCell<States> = 
    once_cell::sync::OnceCell::INIT;

  #[cfg(feature = "std")]
  pub fn get_metrics_states() -> &'static States {
//    STATE.get_or_try_init(|| {
    STATE.get_or_init(|| {
      let conf = &DEFAULT_CONF;
      let st = init_states(conf);
      start_metrics(st.clone(), conf.clone())
        .expect("Fail on metrics states initialization");
      st
//      Ok(st)
    })
  }

  #[cfg(feature = "std")]
  pub mod inc {
    /// generated function for metrics config defined counter
    pub fn a_int_counter() {
      super::get_metrics_states().a_int_counter_inc()
    }
  }

  #[cfg(feature = "std")]
  /// mod for poc without proc macro: with a proc macro having a secific fn name is easy
  pub mod by {
    /// generated function for metrics config defined counter
    pub fn a_int_counter(nb : i64) {
      super::get_metrics_states().a_int_counter_inc_by(nb)
    }
  }

}}

#[cfg(feature = "std")]
#[cfg(feature = "slog")]
pub mod slogger;

#[cfg(not(all(feature = "std",feature = "slog")))]
#[path = "empty.rs"]
pub mod slogger;
 
/// this module should be genereated by METRICS_DEF by a simple proc_macro
/// (adding some named ref counter to the struct and other variants).
/// That way the configuration do not need to be directly in the crate but 
/// in any metrics project specific linked crate.
/// TODO not that good idea writing it directly is probably simplier.
/// still proc_macro could do quick recompile without changing code base.
/// Idea of having a proc_macro fetching the CONF??
/// -> not sure that it is doable (I could understand that being blocked).
#[cfg(feature = "std")]
#[cfg(feature = "pro")]
pub mod pro;
#[cfg(not(all(feature = "std",feature = "pro")))]
#[path = "empty.rs"]
pub mod pro;
 
pub mod empty;


#[cfg(not(all(feature = "std",feature = "slogger")))]
#[path = "empty.rs"]
pub mod slogger;
 

#[test]
fn test_metrics() {
  mets!(a_int_counter, by(1), warn, target "anything", "some additional logs {}", 123);
  mets!(a_int_counter, inc());
}
