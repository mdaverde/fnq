pub use error::OpsError;
pub use queue::queue;
pub use tap::tap;
pub use wait::wait;
pub use watch::watch;

use nix::fcntl;
use std::os::unix::prelude::*;
use std::{fs, io, path};

#[macro_use]
mod os_strings;
mod error;
mod files;
mod queue;
mod tap;
mod wait;
mod watch;

pub const QUEUE_FILE_PREFIX: &'static str = "fnq";

fn open_file(path_buf: &path::PathBuf) -> Result<fs::File, io::Error> {
    fs::OpenOptions::new().read(true).write(true).open(path_buf)
}

fn lock_on_blocked_file(path_buf: &path::PathBuf) -> Result<(), OpsError> {
    // File handler needs to be alive for the scope of file descriptor
    let stay_alive = open_file(path_buf)?;
    let fd: RawFd = stay_alive.as_raw_fd();
    match fcntl::flock(fd, fcntl::FlockArg::LockSharedNonblock) {
        Ok(_) => {
            fcntl::flock(fd, fcntl::FlockArg::Unlock)?;
        }
        Err(nix::Error::Sys(nix::errno::EWOULDBLOCK)) => {
            fcntl::flock(fd, fcntl::FlockArg::LockShared)?;
            fcntl::flock(fd, fcntl::FlockArg::Unlock)?;
        }
        Err(err) => return Err(OpsError::from(err)),
    };
    Ok(())
}
