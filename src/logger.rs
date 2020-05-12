use lazy_static::lazy_static;
use log::{Level, Metadata, Record, SetLoggerError};
use std::sync::{Arc, RwLock};
// use std::io::Write;

#[derive(Debug)]
struct Logger {
  prefix_stack: Arc<RwLock<Vec<String>>>,
}

lazy_static! {
  static ref LOGGER: Logger = Logger {
    prefix_stack: Arc::new(RwLock::new(vec!["".to_string()])),
  };
}

pub fn init() -> Result<(), SetLoggerError> {
  log::set_logger(&LOGGER).map(|()| log::set_max_level(log::LevelFilter::max()))
}

impl log::Log for LOGGER {
  fn enabled(&self, metadata: &Metadata) -> bool {
    metadata.level() <= Level::Debug
  }

  fn log(&self, record: &Record) {
    let meta = record.metadata();

    if self.enabled(meta) && meta.target().starts_with("rustmergency") {
      println!(
        "{:5} | {}{}",
        record.level().to_string().to_lowercase(),
        LOGGER.prefix_stack.read().unwrap().join(""),
        record.args()
      );
    }
  }

  fn flush(&self) {}
}

pub struct PrefixGuard;

pub fn proxy_prefix(prefix: String) -> PrefixGuard {
  let mut stack = LOGGER.prefix_stack.write().unwrap();
  stack.push(prefix);
  PrefixGuard {}
}

impl Drop for PrefixGuard {
  fn drop(&mut self) {
    LOGGER.prefix_stack.write().unwrap().pop();
  }
}
