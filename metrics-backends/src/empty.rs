
use super::*;



pub use self::emptytimers::*;

#[derive(Clone)]
pub struct Counter;

impl Counter {
  pub fn init(name: &'static str, gl: &GlobalStates) -> Result<Self, Error> {
    Ok(Counter)
  }
  pub fn inc(&self) {
  }
  pub fn by(&self, _nb: i64) {
  }
}

pub mod emptytimers {
  use super::Duration;
  use super::Error;

  #[derive(Clone)]
  pub struct Timer;


  impl Timer {
    pub fn init<GS>(name: &'static str, _gl: &GS) -> Result<Self, Error> {
      Ok(Timer)
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

pub struct Empty;

impl Backend for Empty {
  type GlobalStates = GlobalStates;
  const DEFAULT_FILE_OUTPUT: &'static str = "./dummy"; // never written
  const FILE_ID: &'static str = "empty";
  const DEFAULT_CONF: GlobalCommonDef = GlobalCommonDef {
    dest: OutputDest::Logger,
    out_delay: OutputDelay::Synch,
  };

  fn start_metrics(state: &GlobalStates, conf: GlobalCommonDef) -> Result<(), Error> {
    Ok(())
  }

  fn async_write(states: &GlobalStates) -> Result<(), Error> {
    Ok(())
  }

  fn init_states(config: &GlobalCommonDef) -> Result<GlobalStates, Error> {
    Ok(GlobalStates)
  }
}


