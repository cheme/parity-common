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


//! SLog json backend (async write to log with basic structure)

extern crate slog_json;
extern crate slog_async;
use super::{
	Error,
	GlobalCommonDef,
	OutputDest,
	OutputDelay,
	Duration,
	Backend,
};
use super::slog::Drain;
use std::fs::{
	OpenOptions,
	File,
};

const CHANNEL_SIZE: usize = 262144;
#[derive(Clone)]
pub struct Counter(&'static str, GlobalStates);

#[derive(Clone)]
pub struct Timer(&'static str, GlobalStates);


impl super::Counter for Counter {
	type GlobalStates = GlobalStates;
	fn init(name: &'static str, gl: &GlobalStates) -> Result<Self, Error> {
		Ok(Counter(name, gl.clone()))
	}
	fn inc(&self) {
		slog_info!(&(self.1).0, "counter"; self.0 => "1");
	}
	fn by(&self, nb: i64) {
		slog_info!(&(self.1).0, "counter"; self.0 => nb);
	}

}

impl super::Timer for Timer {
	type GlobalStates = GlobalStates;
	fn init(name: &'static str, gl: &GlobalStates) -> Result<Self, Error> {
		Ok(Timer(name, gl.clone()))
	}

	fn start(&self) {
		slog_info!(&(self.1).0, "timer start"; self.0 => format!("{:?}", std::time::Instant::now()));
	}

	fn suspend(&self) {
		slog_info!(&(self.1).0, "timer stop"; self.0 => format!("{:?}", std::time::Instant::now()));
	}

	fn add(&self, dur: Duration) {
		slog_info!(&(self.1).0, "timer duration"; self.0 => format!("{:?}", dur));
	}

}



#[derive(Clone)]
pub struct GlobalStates(slog::Logger<std::sync::Arc<dyn slog::SendSyncRefUnwindSafeDrain<Ok=(), Err=slog::Never>>>);

/*impl Drop for GlobalStates {
	fn drop(&mut self) {
		std::io::stderr().flush();
	}
}*/

pub struct Slogger;

impl Backend for Slogger {
	type GlobalStates = GlobalStates;
	type Counter = Counter;
	type Timer = Timer;
	const DEFAULT_FILE_OUTPUT: &'static str = "./logjson";
	const FILE_ID: &'static str = "slogger";
	const DEFAULT_CONF: GlobalCommonDef = GlobalCommonDef {
		dest: OutputDest::Logger,
		out_delay: OutputDelay::Synch,
	};

	fn start_metrics(_state: &GlobalStates, _conf: GlobalCommonDef) -> Result<(), Error> {
		// no Self::start_delay_write()?; (except if we use our own channel to th: TODO meth for it)
		Ok(())
	}

	fn async_write(_states: &GlobalStates) -> Result<(), Error> {
		// TODO if thread with chann
		Ok(())
	}

	fn init_states(config: &GlobalCommonDef) -> Result<GlobalStates, Error> {
		let slogger = match config.dest {
			OutputDest::Logger => {
				slog_async::Async::new(
					slog_json::Json::default(
						std::io::stderr()
					).fuse()
				)
					.chan_size(CHANNEL_SIZE)
					.overflow_strategy(slog_async::OverflowStrategy::DropAndReport)
					.build().fuse()
			},
			OutputDest::File(ref opath) => {
				let path = Self::unwrap_file_path(opath);
				let file = if path.exists() {
					OpenOptions::new().write(true).append(true).open(path)?
				} else {
					File::create(path)?
				};
				slog_async::Async::new(
					slog_json::Json::default(
						file
					).fuse()
				)
					.chan_size(CHANNEL_SIZE)
					.overflow_strategy(slog_async::OverflowStrategy::DropAndReport)
					.build().fuse()
			},
			_ => unimplemented!(),
		};

		let log = slog::Logger::root(slogger, o!());
		// TODO if not synch a thread and channel
		Ok(GlobalStates(log))
	}
}
