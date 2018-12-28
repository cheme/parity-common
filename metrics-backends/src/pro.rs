
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
use super::Duration;
const DEFAULT_FILE_OUTPUT: &'static str = "./test_metrics";
pub const DEFAULT_CONF: super::GlobalCommonDef = super::GlobalCommonDef {
  dest: super::OutputDest::File(None),
  out_mode: super::OutputMode::Overwrite,
  out_delay: super::OutputDelay::Periodic(Duration::from_secs(10)),
  out_onclose: true,
  chan_write: false,
};
use super::TimerState;
#[cfg(not(feature = "enable_timer"))]
pub use super::empty::emptytimers::*;

#[derive(Clone)]
pub struct Counter(IntCounter);


/// bad def (secs and nanos), should use single IntCounter on ms (here we stick with Duration
/// format). -> TODO check feature `duration_as_u128` -> TODO redesign TimerState to be Sync?
/// or to be include in prom crate (registry should be over timer)
#[cfg(feature = "enable_timer")]
#[derive(Clone)]
pub struct Timer(std::sync::Arc<RwLock<TimerState>>);


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
  pub timers: std::sync::Arc<RwLock<Vec<(IntGauge, IntGauge, Timer)>>>,
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

#[cfg(feature = "enable_timer")]
impl Timer {
  pub fn init(name: &str, gl: &GlobalStates) -> Self {
    let secs_counter = IntGauge::with_opts(Opts::new(name, "secs"))
      .expect("do we renturn error here: probably yes");
    let nanos_counter = IntGauge::with_opts(Opts::new(name.to_string() + "_n", "nanos".to_string()))
      .expect("do we renturn error here: probably yes");
    gl.registry.register(Box::new(secs_counter.clone()))
      .expect("do we renturn error here: probably yes"); // TODO
    gl.registry.register(Box::new(nanos_counter.clone()))
      .expect("do we renturn error here: probably yes"); // TODO
    let state = Timer(std::sync::Arc::new(RwLock::new(TimerState::new())));
    let mut timers = gl.timers.write();
    timers.push((secs_counter, nanos_counter, state.clone()));
    state
  }

  /// TODO remove in favor of stack allocated?
  /// could be used in non stack permited context
  pub fn start(&self) {
    let mut state = self.0.write();
    state.assert_tick_start();
//    state.tick()
  }

  /// TODO remove in favor of stack allocated?
  pub fn suspend(&self) {
    let mut state = self.0.write();
    state.assert_tick_stop();
//    state.tick();
  }

  pub fn add(&self, dur: Duration) {
    let mut state = self.0.write();
    state.duration += dur;
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
    timers: std::sync::Arc::new(RwLock::new(Vec::new())),
  }
}

#[cfg(feature = "enable_timer")]
pub fn async_write_timers(states: &GlobalStates) {
  for (sec, nsec, state) in states.timers.read().iter() {
    let state_l = state.0.read();
    sec.set(state_l.duration.as_secs() as i64);
    nsec.set(state_l.duration.subsec_nanos() as i64);
  }
}

#[cfg(not(feature = "enable_timer"))]
pub fn async_write_timers(states: &GlobalStates) { }

// TODO define and use error type (unwrap for now)
pub fn async_write(states: &GlobalStates) {


  async_write_timers(states);
    /*let d = Duration::new(self.0.get() as u64, self.1.get() as u32);
    TimerState::from_dur(d)*/
/*
    self.0.set(state.duration.as_secs() as i64);
    self.0.set(state.duration.subsec_nanos() as i64);
*/
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
