
//! SLog backend (async write to log with basic structure)


extern crate slog_json;
extern crate slog_async;
use super::*;
use self::slog::Drain;
use std::io::Write;
const CHANNEL_SIZE: usize = 262144;
const DEFAULT_CONF: GlobalCommonDef = GlobalCommonDef {
  dest: OutputDest::Logger,
  out_mode: OutputMode::Append,
  out_delay: OutputDelay::Synch,
  out_onclose: true,
  chan_write: false,
};

#[derive(Clone)]
pub struct States (slog::Logger<std::sync::Arc<dyn slog::SendSyncRefUnwindSafeDrain<Ok=(), Err=slog::Never>>>);

pub fn init_states(config: &super::GlobalCommonDef) -> States {
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
	return States(log);
}

impl Drop for States {
  fn drop(&mut self) {
    std::io::stderr().flush();
  }
}
fn start_metrics(state: States, conf: super::GlobalCommonDef) {
}

metrics_defaults!();
impl States {
  fn a_int_counter_inc(&self) {
    slog_info!(&self.0, "counter"; "a_int_counter" => "1");
  }
  fn a_int_counter_inc_by(&self, nb: i64) {
    slog_info!(&self.0, "counter"; "a_int_counter" => nb);
  }
}
