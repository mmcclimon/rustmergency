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

// This is an empty struct with special drop semantics. You get one by calling
// push_proxy_prefix(), which pushes a prefix onto the stack. When this guard
// object is dropped, the prefix will be popped off the stack and logging will
// continue as normal. This is a bit of a pain to manage manually, so there's a
// macro, proxy_prefix!(...) which will do it for you.
#[derive(Debug)]
pub struct LoggerPrefixGuard;

pub fn push_proxy_prefix(prefix: String) -> LoggerPrefixGuard {
  let mut stack = LOGGER.prefix_stack.write().unwrap();
  stack.push(prefix);
  LoggerPrefixGuard {}
}

#[macro_export]
macro_rules! proxy_prefix {
  ($($arg:tt)*) => {
    let _proxy_prefix_guard_object = $crate::logger::push_proxy_prefix(
      std::fmt::format(format_args!($($arg)*))
    );
  };
}

impl Drop for LoggerPrefixGuard {
  fn drop(&mut self) {
    LOGGER.prefix_stack.write().unwrap().pop();
  }
}
