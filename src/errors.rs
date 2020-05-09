use std::error::Error;
use std::fmt;
use std::io::Error as IoError;

use toml::de::Error as DeserializationError;

pub type MergerResult<T> = Result<T, MergerError>;

#[derive(Debug)]
pub enum MergerError {
  Io(IoError),
  De(String, DeserializationError),
  Config(String),
}

impl fmt::Display for MergerError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      MergerError::Io(err) => write!(f, "{}", err),
      MergerError::De(filename, err) => {
        write!(f, "error reading {}:\n  {}", filename, err)
      },
      MergerError::Config(err) => write!(f, "invalid config file:\n  {}", err),
    }
  }
}

impl Error for MergerError {}

impl From<IoError> for MergerError {
  fn from(err: IoError) -> Self { MergerError::Io(err) }
}
