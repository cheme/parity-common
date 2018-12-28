
//! SLog backend (async write to log with basic structure)
//! Run stateless (TODO probably change that : this is focused on aggregation by external tool)


extern crate slog_json;
extern crate slog_async;
use super::*;
use self::slog::Drain;
use std::io::Write;
const CHANNEL_SIZE: usize = 262144;
pub const DEFAULT_CONF: GlobalCommonDef = GlobalCommonDef {
  dest: OutputDest::Logger,
  out_mode: OutputMode::Append,
  out_delay: OutputDelay::Synch,
  out_onclose: true,
  chan_write: false,
};

#[derive(Clone)]
pub struct Counter(&'static str, GlobalStates);

#[derive(Clone)]
pub struct Timer(&'static str, GlobalStates);


impl Counter {
  pub fn init(name: &'static str, gl: &GlobalStates) -> Self {
    Counter(name, gl.clone())
  }
  pub fn inc(&self) {
    slog_info!(&(self.1).0, "counter"; self.0 => "1");
  }
  pub fn by(&self, nb: i64) {
    slog_info!(&(self.1).0, "counter"; self.0 => nb);
  }

}

impl Timer {
  pub fn init(name: &'static str, gl: &GlobalStates) -> Self {
    Timer(name, gl.clone())
  }

  pub fn start(&self) {
    slog_info!(&(self.1).0, "timer start"; self.0 => format!("{:?}", std::time::Instant::now()));
  }

  pub fn suspend(&self) {
    slog_info!(&(self.1).0, "timer stop"; self.0 => format!("{:?}", std::time::Instant::now()));
  }
}



#[derive(Clone)]
pub struct GlobalStates(slog::Logger<std::sync::Arc<dyn slog::SendSyncRefUnwindSafeDrain<Ok=(), Err=slog::Never>>>);

pub fn async_write(states: &GlobalStates) { }

pub fn init_states(config: &super::GlobalCommonDef) -> GlobalStates {
  let out_sync = std::io::stderr();

  let log = slog::Logger::root(
  slog_async::Async::new(
	slog_json::Json::default(
     out_sync 
      ).fuse()
	).chan_size(CHANNEL_SIZE)
	.overflow_strategy(slog_async::OverflowStrategy::DropAndReport)
	.build().fuse(), o!()
  );
  // TODO if not synch a thread and channel
  GlobalStates(log)
}

/*impl Drop for GlobalStates {
  fn drop(&mut self) {
    std::io::stderr().flush();
  }
}*/

pub fn start_metrics(state: &GlobalStates, conf: super::GlobalCommonDef) -> Result<(), Error> {
  Ok(())
}
