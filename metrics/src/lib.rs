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


/// drafting some spec
/// Those could be set from command line or ext file not at compile time
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
extern crate lazy_static;
#[cfg(feature = "std")]
#[cfg(feature = "pro")]
#[macro_use]
extern crate prometheus;
#[macro_export]
macro_rules! error {
  (metric: $name:ident, target: $target:expr, $($arg:tt)+) => {
    $crate::backend::inc::$name();
    $crate::log::error!(target: $target, $($arg)+)
	};
  (metric: $name:ident, by: $laz:expr, target: $target:expr, $($arg:tt)+) => {
    $crate::backend::by::$name($laz);
    $crate::log::error!(target: $target, $($arg)+)
	};
	(target: $target:expr, $($arg:tt)+) => {
    $crate::log::error!(target: $target, $($arg)+)
	};
	($($arg:tt)+) => {
    $crate::log::error!($($arg)+)
	};
}


#[macro_export]
macro_rules! trace {
  (metric: $name:ident, target: $target:expr, $($arg:tt)+) => {
    $crate::backend::inc::$name();
    $crate::log::trace!(target: $target, $($arg)+)
	};
  (metric: $name:ident, by: $laz:expr, target: $target:expr, $($arg:tt)+) => {
    $crate::backend::by::$name($laz);
    $crate::log::trace!(target: $target, $($arg)+)
	};
	(target: $target:expr, $($arg:tt)+) => {
    $crate::log::trace!(target: $target, $($arg)+)
	};
	($($arg:tt)+) => {
    $crate::log::trace!($($arg)+)
	};
}

#[macro_export]
macro_rules! warn {
  (metric: $name:ident, target: $target:expr, $($arg:tt)+) => {
    $crate::backend::inc::$name();
    $crate::log::warn!(target: $target, $($arg)+)
	};
  (metric: $name:ident, by: $laz:expr, target: $target:expr, $($arg:tt)+) => {
    $crate::backend::by::$name($laz);
    $crate::log::warn!(target: $target, $($arg)+)
	};
	(target: $target:expr, $($arg:tt)+) => {
    $crate::log::warn!(target: $target, $($arg)+)
	};
	($($arg:tt)+) => {
    $crate::log::warn!($($arg)+)
	};

}
#[macro_export]
macro_rules! info {
  (metric: $name:ident, target: $target:expr, $($arg:tt)+) => {
    $crate::backend::inc::$name();
    $crate::log::info!(target: $target, $($arg)+)
	};
  (metric: $name:ident, by: $laz:expr, target: $target:expr, $($arg:tt)+) => {
    $crate::backend::by::$name($laz);
    $crate::log::info!(target: $target, $($arg)+)
	};
	(target: $target:expr, $($arg:tt)+) => {
    $crate::log::info!(target: $target, $($arg)+)
	};
	($($arg:tt)+) => {
    $crate::log::info!($($arg)+)
	};
}
#[macro_export]
macro_rules! debug {
  (metric: $name:ident, target: $target:expr, $($arg:tt)+) => {
    $crate::backend::inc::$name();
    $crate::log::debug!(target: $target, $($arg)+)
	};
  (metric: $name:ident, by: $laz:expr, target: $target:expr, $($arg:tt)+) => {
    $crate::backend::by::$name($laz);
    $crate::log::debug!(target: $target, $($arg)+)
	};
	(target: $target:expr, $($arg:tt)+) => {
    $crate::log::debug!(target: $target, $($arg)+)
	};
	($($arg:tt)+) => {
    $crate::log::debug!($($arg)+)
	};
}
#[macro_export]
macro_rules! do_metric {
  ($name:ident) => {
    $crate::backend::inc::$name();
	};
  ($name:ident, $laz:expr, $($arg:tt)+) => {
    $crate::backend::by::$name($laz);
    $crate::log::debug!($exp)
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
#[cfg(feature = "pro")]
pub mod backend {

  extern crate parking_lot;

  use self::parking_lot::{
    RwLock,
  };
  use prometheus::{
    IntCounter,
    IntCounterVec,
    IntGauge,
    IntGaugeVec,
    Registry,
    Opts,
    TextEncoder,
    Encoder,
  };
  use std::str::FromStr;
const DEFAULT_FILE_OUTPUT: &'static str = "./test_metrics";
const DEFAULT_CONF: super::GlobalCommonDef = super::GlobalCommonDef {
  dest: super::OutputDest::File(None),
  out_mode: super::OutputMode::Overwrite,
  out_delay: super::OutputDelay::Periodic(std::time::Duration::from_secs(10)),
  out_onclose: true,
  chan_write: false,
};
  /// TODO allow plugin of a future runtime (a variant of the method (lazy starting will start 
  /// with this method).
  /// TODO only spawn if needed (if on close only do not)
  fn start_telemetry(state: States, conf: super::GlobalCommonDef) {
    std::thread::spawn(move || {

      let state = state;
      if let super::OutputDelay::Periodic(dur) = conf.out_delay {
        loop {
          std::thread::sleep(dur);
          collect_write(&state);
        }

      } else {
      // TODO put those thread or other mechanism behind out mode with out delay as par
      }

    });
  }


impl Drop for States {
  fn drop(&mut self) {
    // TODO if right mode (no need to gate that behind macro)
    collect_write(&STATE)
  }
}

  /// called States and not State to indicate it should not be seen as a single struct (only
  /// convenient for istantiation.
  ///
  /// TODO design in such a way that you can only get states from lazy_init (once_call)
  ///
  ///
  /// TODO a_int_counter in a lazy_init is not really an acceptable overhead -> try to find a way
  /// to get the atomic from constant function. (still a good thing to lazy init both in case we 
  /// do not have a call to init function: once_call is better for that TODO check what use they
  /// make of their additional atomic).
  #[derive(Clone)]
  pub struct States {
    pub global_state: GlobalState,
    pub a_int_counter: IntCounter,
  }

#[derive(Clone)]
  pub struct GlobalState {
    // we do not use default registry as we already have notion of global state
    pub registry: Registry,
    pub file_handle: std::sync::Arc<RwLock<std::fs::File>>, // TODO a lib dest enum with a trait ptr?? with open fn ...
  }
  pub fn init_states(config: &super::GlobalCommonDef) -> States {

    let a_int_counter = IntCounter::with_opts(Opts::new("A_int_counter", "help..."))
      .expect("do we renturn error here: probably yes");

    let file_handle = std::sync::Arc::new(RwLock::new({
    if let super::OutputDest::File(ref opath) = config.dest {
      // TODO use Path instead of clone.
      let path = opath.clone().unwrap_or_else(||std::path::PathBuf::from_str(DEFAULT_FILE_OUTPUT)
                                      .expect("TODO create err type"));
      // TODO support for append (need a dest type)
      std::fs::File::create(path).unwrap()
    } else {
      panic!("TODO move in a dest object to instantiate the write use by backend");
    }
    }));
    let global_state = GlobalState {
      registry: Registry::new(),
      file_handle,
    };

    global_state.registry.register(Box::new(a_int_counter.clone()))
      .expect("do we renturn error here: probably yes"); // TODO

    States {
      global_state,
      a_int_counter,
    }
  }

  // TODO define and use error type (unwrap for now)
  pub fn collect_write(states: &States) {
    use std::io::Write;
    let encoder = TextEncoder::new();
    let metric_families = states.global_state.registry.gather();
    {
      let mut fh = states.global_state.file_handle.write();
      // TODO rewrite file
      encoder.encode(&metric_families, &mut *fh).unwrap();
      fh.flush().unwrap();
    }
  }


  lazy_static! {
    // getting conf from cmd line | other will be extra shitty crate `once_cell` is probably way
    // more appropriate to do thing nicely!!
    static ref STATE: States = {
      let conf = &DEFAULT_CONF;
      let st = init_states(conf);
      start_telemetry(st.clone(), conf.clone());
      st
    };
  }

  pub mod inc {
    /// generated function for metrics config defined counter
    pub fn a_int_counter() {
      println!("s");
      super::STATE.a_int_counter.inc()
    }
  }

  /// mod for poc without proc macro: with a proc macro having a secific fn name is easy
  pub mod by {
    /// generated function for metrics config defined counter
    pub fn a_int_counter(nb : i64) {
      super::STATE.a_int_counter.inc_by(nb)
    }
  }
}

// incomplet feature test : move backend in their own file behind prom and put no std variant
// import here to make it right
#[cfg(not(feature = "std"))]
pub mod backend {

const DEFAULT_FILE_OUTPUT: &'static str = "./dummy"; // never write
  const DEFAULT_CONF: GlobalCommonDef = GlobalCommonDef {
    dest: OutputDest::Logger,
    out_mode: OutputMode::Append,
    out_delay: OutputDelay::Synch,
    out_onclose: true,
    chan_write: false,
  };

  pub mod inc {
    /// generated function for metrics config defined counter
    pub fn a_int_counter() {
    }
  }

  /// mod for poc without proc macro: with a proc macro having a secific fn name is easy
  pub mod by {
    /// generated function for metrics config defined counter
    pub fn a_int_counter(_nb : i64) {
    }
  }
}

// TODO csv backend (reuse substrate telemetry code for json direct log format)...