
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
fn start_metrics(state: States, conf: super::GlobalCommonDef) {
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
    let path = opath.clone().unwrap_or_else(||std::path::PathBuf::from(DEFAULT_FILE_OUTPUT.to_string()));
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

// TODO gen it (also states)
impl States {
  fn a_int_counter_inc(&self) {
    self.a_int_counter.inc();
  }
  fn a_int_counter_inc_by(&self, nb: i64) {
    self.a_int_counter.inc_by(nb);
  }
}

metrics_defaults!();
