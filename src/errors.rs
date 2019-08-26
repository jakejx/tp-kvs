//! When using the KV store goes wrong

use std::error::Error;
use std::fmt;
use std::io;
use std::result;

/// This type represents all possible errors that occur when using the library
#[derive(Debug)]
pub enum KvError {
    Serde(serde_json::Error),
    Io(io::Error),
    KeyNotFound,
    InternalError,
    MissingLogFile,
}

impl From<serde_json::Error> for KvError {
    fn from(err: serde_json::Error) -> KvError {
        KvError::Serde(err)
    }
}

impl From<io::Error> for KvError {
    fn from(err: io::Error) -> KvError {
        KvError::Io(err)
    }
}

impl fmt::Display for KvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            KvError::Serde(ref err) => err.fmt(f),
            KvError::Io(ref err) => err.fmt(f),
            KvError::KeyNotFound => write!(f, "Key not found"),
            KvError::InternalError => write!(f, "Internal error"),
            KvError::MissingLogFile => write!(f, "There is a missing log file"),
        }
    }
}

impl Error for KvError {
    fn description(&self) -> &str {
        match self {
            KvError::Serde(ref err) => err.description(),
            KvError::Io(ref err) => err.description(),
            KvError::KeyNotFound => "Key not found",
            KvError::InternalError => "Internal error",
            KvError::MissingLogFile => "Missing log file",
        }
    }
}

/// Alias for a `Result` with the error type `kvs::KvError`
pub type Result<T> = result::Result<T, KvError>;
