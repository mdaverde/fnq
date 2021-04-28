use std::{error, ffi, fmt, io, time};

#[derive(Debug)]
#[non_exhaustive]
pub enum OpsError {
    StringConv,
    QueueEmpty,
    FileNotFound(ffi::OsString),
    IO(io::Error),
    Unix(String),
    SystemTime(time::SystemTimeError),
    Watcher(notify::Error),
    WatcherUnknown(String),
    Unknown(String),
}

impl fmt::Display for OpsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpsError::StringConv => write!(f, "UTF-8 conversion unsuccessful"),
            OpsError::FileNotFound(filename) => write!(f, "Could not find file: {:?}", filename),
            OpsError::QueueEmpty => write!(f, "Lock file queue is empty"),
            OpsError::IO(io_err) => io_err.fmt(f),
            OpsError::Unix(nix_err) => nix_err.fmt(f),
            OpsError::SystemTime(sys_time_err) => sys_time_err.fmt(f),
            OpsError::Watcher(notify_err) => notify_err.fmt(f),
            OpsError::WatcherUnknown(str_err) => write!(f, "Unknown error(watcher): {}", str_err),
            OpsError::Unknown(str_err) => write!(f, "Unknown error: {}", str_err),
        }
    }
}

impl error::Error for OpsError {}

impl From<io::Error> for OpsError {
    fn from(err: io::Error) -> Self {
        OpsError::IO(err)
    }
}

impl From<nix::Error> for OpsError {
    fn from(err: nix::Error) -> Self {
        OpsError::Unix(err.to_string())
    }
}

impl From<time::SystemTimeError> for OpsError {
    fn from(err: time::SystemTimeError) -> Self {
        OpsError::SystemTime(err)
    }
}

impl From<ffi::NulError> for OpsError {
    fn from(_: ffi::NulError) -> Self {
        OpsError::StringConv
    }
}

impl From<notify::Error> for OpsError {
    fn from(err: notify::Error) -> Self {
        OpsError::Watcher(err)
    }
}
