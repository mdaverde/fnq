use std::{fs, path};
use std::os::unix::prelude::*;

use nix::{fcntl, unistd};

use crate::ops::files;
use crate::ops::error::OpsError;

pub fn tap(queue_dir: path::PathBuf) -> Result<bool, OpsError> {
    let queue_files = files::files(&queue_dir)?;

    for entry in queue_files {
        let opened_file: fs::File = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&entry.filepath)?;

        if fcntl::flock(opened_file.as_raw_fd(), fcntl::FlockArg::LockSharedNonblock).is_err() {
            return Ok(false);
        }

        // Remove process lock
        unistd::close(opened_file.as_raw_fd())?;
    }

    Ok(true)
}
