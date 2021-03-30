use std::{error, fs, path};
use std::os::unix::prelude::*;

use nix::{errno, fcntl, unistd};

use crate::cmd_ops::queue_files;

pub fn wait(queue_dir: path::PathBuf) -> Result<bool, Box<dyn error::Error>> {
    let queue_files = queue_files::queue_files_sorted(&queue_dir)?;

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
