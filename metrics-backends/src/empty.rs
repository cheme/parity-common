
use super::*;
const DEFAULT_FILE_OUTPUT: &'static str = "./dummy"; // never write
const DEFAULT_CONF: GlobalCommonDef = GlobalCommonDef {
  dest: OutputDest::Logger,
  out_mode: OutputMode::Append,
  out_delay: OutputDelay::Synch,
  out_onclose: true,
  chan_write: false,
};

#[derive(Clone)]
pub struct States;

fn init_states(config: &super::GlobalCommonDef) -> States { States }

fn start_metrics(state: States, conf: super::GlobalCommonDef) -> Result<(), Error> {
  Ok(())
}

impl States {
  fn a_int_counter_inc(&self) {
  }
  fn a_int_counter_inc_by(&self, nb: i64) {
  }
}

metrics_defaults!();
