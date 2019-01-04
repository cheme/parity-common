
use super::*;



pub use self::emptytimers::*;

#[derive(Clone)]
pub struct Counter;

impl super::Counter for Counter {
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


