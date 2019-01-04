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

//! noops backend

use super::*;

pub use self::emptytimers::*;

#[derive(Clone)]
pub struct Counter;

impl ::Counter for Counter {
	type GlobalStates = GlobalStates;
	fn init(_name: &'static str, _gl: &GlobalStates) -> Result<Self, Error> {
		Ok(Counter)
	}
	fn inc(&self) {
	}
	fn by(&self, _nb: i64) {
	}
}

pub mod emptytimers {
	use super::Duration;
	use super::Error;

	#[derive(Clone)]
	pub struct Timer;


	impl ::Timer for Timer {
		type GlobalStates = super::GlobalStates;
		fn init(_name: &'static str, _gl: &Self::GlobalStates) -> Result<Self, Error> {
			Ok(Timer)
		}

		fn start(&self) {
		}

		fn suspend(&self) {
		}

		fn add(&self, _dur: Duration) {
		}
	}
}

#[derive(Clone)]
pub struct GlobalStates;

pub struct Empty;

impl Backend for Empty {
	type GlobalStates = GlobalStates;
	type Counter = Counter;
	type Timer = emptytimers::Timer;
	const DEFAULT_FILE_OUTPUT: &'static str = "./dummy"; // never written
	const FILE_ID: &'static str = "empty";
	const DEFAULT_CONF: GlobalCommonDef = GlobalCommonDef {
		dest: OutputDest::Logger,
		out_delay: OutputDelay::Synch,
	};

	fn start_metrics(_state: &GlobalStates, _conf: GlobalCommonDef) -> Result<(), Error> {
		Ok(())
	}

	fn async_write(_states: &GlobalStates) -> Result<(), Error> {
		Ok(())
	}

	fn init_states(_config: &GlobalCommonDef) -> Result<GlobalStates, Error> {
		Ok(GlobalStates)
	}
}
