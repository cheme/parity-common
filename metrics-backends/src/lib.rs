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

#![cfg_attr(not(feature = "std"), no_std)]
//#![cfg_attr(not(feature = "std"), feature(core_intrinsics))]
//#![cfg_attr(not(feature = "std"), feature(alloc))]

#[cfg(feature = "std")]
#[cfg(feature = "slogger")]
#[macro_use]
extern crate slog;

extern crate failure;

#[cfg(feature = "std")]
pub use std::time::Duration;
#[cfg(not(feature = "std"))]
pub use core::time::Duration;

#[cfg(feature = "std")]
pub type Error = failure::Error;

#[cfg(not(feature = "std"))]
pub type Error = (); // TODO switch to dummy error with required conversions


pub extern crate metrics_procedural as metrics_derive;
pub use metrics_derive::*;

#[macro_export]
macro_rules! metrics {
	(from_crate($cn:ident) [$($be:ident),*], $name:ident, $action:ident$laz:tt, $level:ident, target $target:expr, $($arg:tt)+) => {
		{
			use $cn::*;
			$(
				let __ds = $cn::$be::get_metrics_states().derived_state.$name.$action$laz;
			)*
			use $cn::log::log;
			$cn::log::$level!(target: $target, $($arg)+)
		}
	};
	(from_crate($cn:ident) [$($be:ident),*], $name:ident, $action:ident$laz:tt) => {
		{
			use $cn::*;
			$(
				let __ds = $cn::$be::get_metrics_states().derived_state.$name.$action$laz;
			)*
		}
	};
}

/// drafting some spec
/// Those could be set from command line or ext file not at compile time
/// Those conf are dynamic confs that can be feed at any time (eg through command parameter)
/// TODO some of those items does not make sense (specific should only be option delay...)
#[derive(Clone)]
pub struct GlobalCommonDef {
	pub dest: OutputDest,
	/// delay between each write (if undefined no regular write)
	/// implies an async process
	pub out_delay: OutputDelay,
}

/// static outdesc, resulting object will simply be `impl Write`
#[derive(Clone)]
pub enum OutputDest {
	File(Option<std::path::PathBuf>), // if None use pathbuf from constant str
	/// use the log macro to push content
	Logger,
	/// TODO substrate telemetry crate to push on ws ?
	Telemetry,
}

#[derive(Clone)]
pub enum OutputDelay {
	Synch,
	Periodic(Duration),
	Unlimited,
}

pub extern crate log;

#[cfg(feature = "std")]
pub extern crate once_cell;

#[cfg(feature = "std")]
#[cfg(feature = "pro")]
extern crate prometheus;

#[cfg(feature = "std")]
#[cfg(feature = "slog")]
pub mod slogger;

#[cfg(not(all(feature = "std",feature = "slog")))]
#[path = "empty.rs"]
pub mod slogger;
 
#[cfg(feature = "std")]
#[cfg(feature = "pro")]
pub mod pro;
#[cfg(not(all(feature = "std",feature = "pro")))]
pub mod pro {
	pub use super::empty::Empty as Pro;
}
 
pub mod empty;


#[cfg(not(all(feature = "std",feature = "slogger")))]
#[path = "empty.rs"]
pub mod slogger {
	pub use super::empty::Empty as Slogger;
}
 

/// Define an integer counter
pub trait Counter: Sized {
	type GlobalStates;
	fn init(name: &'static str, gl: &Self::GlobalStates) -> Result<Self, Error>;
	fn inc(&self);
	fn by(&self, nb: i64);
}

/// Define an timer counter
/// Note that if feature `enable_timer`
/// is not set this is a noops.
pub trait Timer: Sized {
	type GlobalStates;
	fn init(name: &'static str, gl: &Self::GlobalStates) -> Result<Self, Error>;

	fn start(&self);

	fn suspend(&self);

	fn add(&self, dur: Duration);
}


#[cfg(not(feature = "std"))]
#[derive(Clone, Debug)]
pub struct TimerStart;

#[cfg(feature = "std")]
#[derive(Clone, Debug)]
//#[derive(Copy)]
/// timer state to metter duration between
/// tagger metrics.
/// For no_std we need to plug an instrinsec
/// to get clock (for instance expose a cffi 
/// on i128 instant).
pub struct TimerState {
	pub last_start: Option<std::time::Instant>,
	pub duration: Duration,
	// unsound rec call support only for single thread
	// TODO swith to stack allocated local state
	pub depth: usize,
}

#[cfg(feature = "std")]
impl TimerState {
	/// stopped on instantiation
	pub fn new() -> Self {
		TimerState {
			last_start: None,
			duration: Duration::new(0, 0),
			depth: 0,
		}
	}

	pub fn from_dur(duration: Duration) -> Self {
		TimerState {
			last_start: None,
			duration,
			depth: 0,
		}
	}


	/// tick measure stop if running or start if not running.
	pub fn tick(&mut self, now: std::time::Instant) {
		if self.last_start.is_some() {
			let ld = std::mem::replace(&mut self.last_start, None);
			let ld = ld.expect("Tested above; qed");
			self.duration = self.duration + now.duration_since(ld);
		} else {
			self.last_start = Some(now);
		}
	}

	/// tick measure stop if running or start if not running.
	/// tick with state assertion for debugging.
	pub fn assert_tick_start(&mut self, now: std::time::Instant) {
		//assert!(self.last_start.is_none());
		if self.last_start.is_none() {
			self.tick(now);
		} else {
			self.depth += 1;
		}
	}

	/// tick measure stop if running or start if not running.
	/// tick with state assertion for debugging.
	pub fn assert_tick_stop(&mut self, now: std::time::Instant) {
		assert!(self.last_start.is_some());
		if self.depth > 0 {
			self.depth -= 1;
		} else {
			self.tick(now);
		}
	}

}

pub trait Backend {
	type GlobalStates: 'static + Clone + Send; // TODO switch simply to sync TODO check if use elsewhere
	type Counter: Counter;
	type Timer: Timer;
	const FILE_ID: &'static str;
	const DEFAULT_CONF: GlobalCommonDef;
	const DEFAULT_FILE_OUTPUT: &'static str;
	fn async_write(&Self::GlobalStates) -> Result<(), Error>;

	fn start_metrics(state: &Self::GlobalStates, conf: GlobalCommonDef) -> Result<(), Error>;

	/// utility to start delay processs
	fn start_delay_write(state: &Self::GlobalStates, conf: GlobalCommonDef) -> Result<(), Error> {
		if let OutputDelay::Periodic(dur) = conf.out_delay {
			let state_th = state.clone();
			std::thread::spawn(move || {

				let state = state_th;
					loop {
						std::thread::sleep(dur);
						Self::async_write(&state).unwrap(); // TODO manage panic on write
					}
			});
		}
		Ok(())
	}

	fn init_states(config: &GlobalCommonDef) -> Result<Self::GlobalStates, Error>;

	#[cfg(feature = "std")]
	fn unwrap_file_path(opath: &Option<std::path::PathBuf>) -> std::path::PathBuf {
		opath.clone().unwrap_or_else(||std::path::PathBuf::from(Self::DEFAULT_FILE_OUTPUT.to_string() + "_" + Self::FILE_ID))
	}
}
