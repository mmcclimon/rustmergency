use reqwest::Error as HttpError;
use std::error::Error;
use std::fmt;
use std::io::Error as IoError;
use std::string::FromUtf8Error;

use toml::de::Error as DeserializationError;

pub type MergerResult<T> = Result<T, MergerError>;

#[derive(Debug)]
pub enum MergerError {
  Io(IoError),
  De(String, DeserializationError),
  Http(HttpError),
  Utf(FromUtf8Error),
  Config(String),
  Local(String),
  // cmdargs, git output
  Git(String, String),
}

impl fmt::Display for MergerError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      MergerError::Io(err) => write!(f, "{}", err),
      MergerError::Utf(err) => write!(f, "{}", err),
      MergerError::De(filename, err) => {
        write!(f, "error reading {}:\n  {}", filename, err)
      }
      MergerError::Http(err) => write!(f, "http error: {}", err),
      MergerError::Config(err) => write!(f, "invalid config file:\n  {}", err),
      MergerError::Local(err) => {
        write!(f, "problem with local setup:\n  {}", err)
      }
      MergerError::Git(cmd, err) => write!(
        f,
        "problem encountered running git:\ncommand: git {}\n{}",
        cmd,
        err.trim_end()
      ),
    }
  }
}

impl Error for MergerError {}

impl From<IoError> for MergerError {
  fn from(err: IoError) -> Self {
    MergerError::Io(err)
  }
}

impl From<FromUtf8Error> for MergerError {
  fn from(err: FromUtf8Error) -> Self {
    MergerError::Utf(err)
  }
}

impl From<HttpError> for MergerError {
  fn from(err: HttpError) -> Self {
    MergerError::Http(err)
  }
}
