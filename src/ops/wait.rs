use std::{fs, path};
use std::os::unix::prelude::*;

use nix::{errno, fcntl, unistd};

use crate::ops::files;
use crate::ops::error::OpsError;

pub fn wait(queue_dir: path::PathBuf) -> Result<bool, OpsError> {
    let queue_files = files::files(&queue_dir)?;

    for entry in queue_files {
        let opened_file: fs::File = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&entry.filepath)?;

        if fcntl::flock(opened_file.as_raw_fd(), fcntl::FlockArg::LockSharedNonblock).is_err() {
            if (errno::EWOULDBLOCK as i32) == errno::errno() {
                fcntl::flock(opened_file.as_raw_fd(), fcntl::FlockArg::LockShared)?;
            }
        }

        // Remove process lock
        unistd::close(opened_file.as_raw_fd())?;
    }

    Ok(true)
}
