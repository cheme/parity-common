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

//! prometheus backend

extern crate parking_lot;

use super::{
	Error,
	GlobalCommonDef,
	OutputDest,
	OutputDelay,
	Backend,
};
use self::parking_lot::{
	RwLock,
};
use prometheus::{
	IntCounter,
	IntGauge,
	Registry,
	Opts,
	TextEncoder,
	Encoder,
};
use std::fs::{
	self,
	File,
};
use std::path::{
	PathBuf,
};

use super::Duration;
use super::TimerState;
#[cfg(not(feature = "enable_timer"))]
pub use super::empty::emptytimers::*;

#[derive(Clone)]
pub struct Counter(IntCounter);

pub struct Pro;

impl Backend for Pro {
	type GlobalStates = GlobalStates;
	type Counter = Counter;
	type Timer = Timer;
	const DEFAULT_FILE_OUTPUT: &'static str = "./test_metrics";
	const FILE_ID: &'static str = "pro";
	const DEFAULT_CONF: GlobalCommonDef = GlobalCommonDef {
		dest: OutputDest::File(None),
		out_delay: OutputDelay::Periodic(Duration::from_secs(10)),
	};

	fn start_metrics(state: &GlobalStates, conf: GlobalCommonDef) -> Result<(), Error> {
		Self::start_delay_write(state, conf)?;
		Ok(())
	}

	fn async_write(states: &GlobalStates) -> Result<(), Error> {

		async_write_timers(states);

		use std::io::Write;
		let encoder = TextEncoder::new();
		let metric_families = states.registry.gather();
		{
			fs::copy(&states.path_file, &states.path_bu)?; // warn could be bad with open file handle
			let mut fh = states.file_handle.write();
			encoder.encode(&metric_families, &mut *fh)?;
			fh.flush().unwrap();
		}
		Ok(())
	}

	fn init_states(config: &GlobalCommonDef) -> Result<GlobalStates, Error> {

		let (file_handle, path_file, path_bu) = 
			if let super::OutputDest::File(ref opath) = config.dest {
				// TODO use Path instead of clone. TODO move to Output dest as util with param replace!!
				let path = Self::unwrap_file_path(opath);
				let mut path_old = path.clone();
				path_old.set_extension("old");
				let mut path_bu = path.clone();
				path_bu.set_extension("prev");
				if path.exists() {
					fs::copy(&path, path_old)?;
				}
				(std::sync::Arc::new(RwLock::new(File::create(&path)?)), path, path_bu)
			} else {
				panic!("TODO implements other dests");
			};
		Ok(GlobalStates {
			registry: Registry::new(),
			file_handle,
			path_file,
			path_bu,
			timers: std::sync::Arc::new(RwLock::new(Vec::new())),
		})
	}


}

/// bad def (secs and nanos), should use single IntCounter on ms (here we stick with Duration
/// format). -> TODO check feature `duration_as_u128` -> TODO redesign TimerState to be Sync?
/// or to be include in prom crate (registry should be over timer)
#[cfg(feature = "enable_timer")]
#[derive(Clone)]
pub struct Timer(std::sync::Arc<RwLock<TimerState>>);


#[derive(Clone)]
pub struct GlobalStates {
	// we do not use default registry as we already have notion of global state
	pub registry: Registry,
	pub file_handle: std::sync::Arc<RwLock<File>>, // TODO a lib dest enum with a trait ptr?? with open fn ...
	pub path_file: PathBuf,
	pub path_bu: PathBuf, // TODO a lib dest enum with a trait ptr?? with open fn ...
	pub timers: std::sync::Arc<RwLock<Vec<(IntGauge, IntGauge, Timer)>>>,
}

impl super::Counter for Counter {
	type GlobalStates = GlobalStates;
	fn init(name: &str, gl: &GlobalStates) -> Result<Self, Error> {
		let a_counter = IntCounter::with_opts(Opts::new(name, "help..."))?;
		gl.registry.register(Box::new(a_counter.clone()))?;
		Ok(Counter(a_counter))
	}

	fn inc(&self) {
		self.0.inc();
	}

	fn by(&self, nb: i64) {
		self.0.inc_by(nb);
	}
}

#[cfg(feature = "enable_timer")]
impl super::Timer for Timer {
	type GlobalStates = GlobalStates;
	fn init(name: &str, gl: &GlobalStates) -> Result<Self, Error> {
		let secs_counter = IntGauge::with_opts(Opts::new(name, "secs"))?;
		let nanos_counter = IntGauge::with_opts(Opts::new(name.to_string() + "_n", "nanos".to_string()))?;
		gl.registry.register(Box::new(secs_counter.clone()))?;
		gl.registry.register(Box::new(nanos_counter.clone()))?;
		let state = Timer(std::sync::Arc::new(RwLock::new(TimerState::new())));
		let mut timers = gl.timers.write();
		timers.push((secs_counter, nanos_counter, state.clone()));
		Ok(state)
	}

	/// TODO remove in favor of stack allocated?
	/// could be used in non stack permited context
	fn start(&self) {
		let mut state = self.0.write();
		// after acquiring lock
		let now = std::time::Instant::now();
		state.assert_tick_start(now);
	}

	/// TODO remove in favor of stack allocated?
	fn suspend(&self) {
		// befor acquiring lock
		let now = std::time::Instant::now();
		let mut state = self.0.write();
		state.assert_tick_stop(now);
	}

	fn add(&self, dur: Duration) {
		let mut state = self.0.write();
		state.duration += dur;
	}
}

#[cfg(feature = "enable_timer")]
fn async_write_timers(states: &GlobalStates) {
	for (sec, nsec, state) in states.timers.read().iter() {
		let state_l = state.0.read();
		sec.set(state_l.duration.as_secs() as i64);
		nsec.set(state_l.duration.subsec_nanos() as i64);
	}
}

#[cfg(not(feature = "enable_timer"))]
fn async_write_timers(states: &GlobalStates) { }
