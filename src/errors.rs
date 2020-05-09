use std::error::Error;
use std::fmt;
use std::io::Error as IoError;

use toml::de::Error as DeserializationError;

pub type MergerResult<T> = Result<T, MergerError>;

#[derive(Debug)]
pub enum MergerError {
  Io(IoError),
  Config(String, DeserializationError),
}

impl fmt::Display for MergerError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      MergerError::Io(err) => write!(f, "{}", err),
      MergerError::Config(filename, err) => {
        write!(f, "error reading {}:\n  {}", filename, err)
      },
    }
  }
}

impl Error for MergerError {}

impl From<IoError> for MergerError {
  fn from(err: IoError) -> Self { MergerError::Io(err) }
}
