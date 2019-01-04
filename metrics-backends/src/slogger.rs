
//! SLog backend (async write to log with basic structure)
//! Run stateless (TODO probably change that : this is focused on aggregation by external tool)


extern crate slog_json;
extern crate slog_async;
use super::{
  Error,
  GlobalCommonDef,
  OutputDest,
  OutputDelay,
  Duration,
  Backend,
};
use super::slog::Drain;
use std::io::Write;
use std::fs::{
  OpenOptions,
  File,
};

const CHANNEL_SIZE: usize = 262144;
#[derive(Clone)]
pub struct Counter(&'static str, GlobalStates);

#[derive(Clone)]
pub struct Timer(&'static str, GlobalStates);


impl Counter {
  pub fn init(name: &'static str, gl: &GlobalStates) -> Result<Self, Error> {
    Ok(Counter(name, gl.clone()))
  }
  pub fn inc(&self) {
    slog_info!(&(self.1).0, "counter"; self.0 => "1");
  }
  pub fn by(&self, nb: i64) {
    slog_info!(&(self.1).0, "counter"; self.0 => nb);
  }

}

impl Timer {
  pub fn init(name: &'static str, gl: &GlobalStates) -> Result<Self, Error> {
    Ok(Timer(name, gl.clone()))
  }

  pub fn start(&self) {
    slog_info!(&(self.1).0, "timer start"; self.0 => format!("{:?}", std::time::Instant::now()));
  }

  pub fn suspend(&self) {
    slog_info!(&(self.1).0, "timer stop"; self.0 => format!("{:?}", std::time::Instant::now()));
  }

  pub fn add(&self, dur: Duration) {
    slog_info!(&(self.1).0, "timer duration"; self.0 => format!("{:?}", dur));
  }

}



#[derive(Clone)]
pub struct GlobalStates(slog::Logger<std::sync::Arc<dyn slog::SendSyncRefUnwindSafeDrain<Ok=(), Err=slog::Never>>>);

/*impl Drop for GlobalStates {
  fn drop(&mut self) {
    std::io::stderr().flush();
  }
}*/

pub struct Slogger;

impl Backend for Slogger {
  type GlobalStates = GlobalStates;
  const DEFAULT_FILE_OUTPUT: &'static str = "./logjson";
  const FILE_ID: &'static str = "slogger";
  const DEFAULT_CONF: GlobalCommonDef = GlobalCommonDef {
    dest: OutputDest::Logger,
    out_delay: OutputDelay::Synch,
  };

  fn start_metrics(state: &GlobalStates, conf: GlobalCommonDef) -> Result<(), Error> {
    // no Self::start_delay_write()?; (except if we use our own channel to th: TODO meth for it)
    Ok(())
  }

  fn async_write(states: &GlobalStates) -> Result<(), Error> {
    // TODO if thread with chann
    Ok(())
  }

  fn init_states(config: &GlobalCommonDef) -> Result<GlobalStates, Error> {
    let slogger = match config.dest {
      OutputDest::Logger => {
        slog_async::Async::new(
        slog_json::Json::default(
          std::io::stderr()
        ).fuse()
        )
          .chan_size(CHANNEL_SIZE)
          .overflow_strategy(slog_async::OverflowStrategy::DropAndReport)
          .build().fuse()
      },
      OutputDest::File(ref opath) => {
        let path = Self::unwrap_file_path(opath);
        let file = if path.exists() {
          OpenOptions::new().write(true).append(true).open(path)?
        } else {
          File::create(path)?
        };
        slog_async::Async::new(
        slog_json::Json::default(
          file
        ).fuse()
        )
          .chan_size(CHANNEL_SIZE)
          .overflow_strategy(slog_async::OverflowStrategy::DropAndReport)
          .build().fuse()
      },
      _ => unimplemented!(),
    };

    let log = slog::Logger::root(slogger, o!());
    // TODO if not synch a thread and channel
    Ok(GlobalStates(log))
  }

}


