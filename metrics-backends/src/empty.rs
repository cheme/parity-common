
use super::*;


const DEFAULT_FILE_OUTPUT: &'static str = "./dummy"; // never write
pub const DEFAULT_CONF: GlobalCommonDef = GlobalCommonDef {
  dest: OutputDest::Logger,
  out_mode: OutputMode::Append,
  out_delay: OutputDelay::Synch,
  out_onclose: true,
  chan_write: false,
};

pub use self::emptytimers::*;

#[derive(Clone)]
pub struct Counter;

impl Counter {
  pub fn init(name: &'static str, gl: &GlobalStates) -> Self {
    Counter
  }
  pub fn inc(&self) {
  }
  pub fn by(&self, _nb: i64) {
  }
}

pub mod emptytimers {
  use super::Duration;

  #[derive(Clone)]
  pub struct Timer;


  impl Timer {
    pub fn init<GS>(name: &'static str, _gl: &GS) -> Self {
      Timer
    }

    pub fn start(&self) {
    }

    pub fn suspend(&self) {
    }

    pub fn add(&self, dur: Duration) {
    }
  }
}

#[derive(Clone)]
pub struct GlobalStates;

pub fn async_write(states: &GlobalStates) { }

pub fn init_states(config: &super::GlobalCommonDef) -> GlobalStates {
  GlobalStates
}

pub fn start_metrics(state: &GlobalStates, conf: super::GlobalCommonDef) -> Result<(), Error> {
  Ok(())
}
