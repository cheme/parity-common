
extern crate parking_lot;

use super::{
  Error,
};
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
pub const DEFAULT_CONF: super::GlobalCommonDef = super::GlobalCommonDef {
  dest: super::OutputDest::File(None),
  out_mode: super::OutputMode::Overwrite,
  out_delay: super::OutputDelay::Periodic(std::time::Duration::from_secs(10)),
  out_onclose: true,
  chan_write: false,
};

#[derive(Clone)]
pub struct Counter(IntCounter);

/// TODO allow plugin of a future runtime (a variant of the method (lazy starting will start 
/// with this method).
/// TODO only spawn if needed (if on close only do not)
pub fn start_metrics(state: &GlobalStates, conf: super::GlobalCommonDef) -> Result<(), Error> {
  let state_th = state.clone();
  std::thread::spawn(move || {

    let state = state_th;
    if let super::OutputDelay::Periodic(dur) = conf.out_delay {
      loop {
        std::thread::sleep(dur);
        async_write(&state);
      }

    } else {
    // TODO put those thread or other mechanism behind out mode with out delay as par
    }

  });
  Ok(())
}

#[derive(Clone)]
pub struct GlobalStates {
  // we do not use default registry as we already have notion of global state
  pub registry: Registry,
  pub file_handle: std::sync::Arc<RwLock<std::fs::File>>, // TODO a lib dest enum with a trait ptr?? with open fn ...
}

impl Counter {
  pub fn init(name: &str, gl: &GlobalStates) -> Self {
    let a_counter = IntCounter::with_opts(Opts::new(name, "help..."))
      .expect("do we renturn error here: probably yes");
    gl.registry.register(Box::new(a_counter.clone()))
      .expect("do we renturn error here: probably yes"); // TODO
    Counter(a_counter)
  }

  pub fn inc(&self) {
    self.0.inc();
  }

  pub fn by(&self, nb: i64) {
    self.0.inc_by(nb);
  }
}

pub fn init_states(config: &super::GlobalCommonDef) -> GlobalStates {

  let file_handle = std::sync::Arc::new(RwLock::new({
  if let super::OutputDest::File(ref opath) = config.dest {
    // TODO use Path instead of clone.
    let path = opath.clone().unwrap_or_else(||std::path::PathBuf::from(DEFAULT_FILE_OUTPUT.to_string()));
    // TODO support for append (need a dest type)
    std::fs::File::create(path).unwrap()
  } else {
    panic!("TODO move in a dest object to instantiate the write use by backend");
  }
  }));
  GlobalStates {
    registry: Registry::new(),
    file_handle,
  }
}

// TODO define and use error type (unwrap for now)
pub fn async_write(states: &GlobalStates) {
  use std::io::Write;
  let encoder = TextEncoder::new();
  let metric_families = states.registry.gather();
  {
    let mut fh = states.file_handle.write();
    // TODO rewrite file
    encoder.encode(&metric_families, &mut *fh).unwrap();
    fh.flush().unwrap();
  }
}
