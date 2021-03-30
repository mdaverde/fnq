use std::fmt::Formatter;
use std::{error, fmt, io};

#[derive(Debug)]
pub enum OpsError {
    UTF8,
    IO(io::Error),
    Unix(nix::Error),
}

impl fmt::Display for OpsError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            OpsError::UTF8 => write!(f, "UTF-8 conversion unsuccessful"),
            OpsError::IO(io_err) => io_err.fmt(f),
            OpsError::Unix(nix_err) => nix_err.fmt(f),
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
        OpsError::Unix(err)
    }
}
