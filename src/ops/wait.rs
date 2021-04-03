use std::os::unix::prelude::*;
use std::path;

use nix::{errno, fcntl, unistd};

use crate::ops::{files, open_file, OpsError};

pub fn wait(queue_dir: path::PathBuf, queue_file: Option<path::PathBuf>) -> Result<(), OpsError> {
    let queue_files = files::files(&queue_dir)?;
 // TODO: this shouldn't work actually?
    if let Some(queue_file) = queue_file {
        match queue_files
            .iter()
            .find(|entry| entry.filepath.eq(&queue_file))
        {
            None => {
                return Err(OpsError::Unknown(format!(
                    "Could not find {:?} file",
                    queue_file
                )))
            }
            Some(entry) => {
                let opened_file = open_file(&entry.filepath)?;

                if fcntl::flock(opened_file.as_raw_fd(), fcntl::FlockArg::LockSharedNonblock)
                    .is_err()
                {
                    if (errno::EWOULDBLOCK as i32) == errno::errno() {
                        fcntl::flock(opened_file.as_raw_fd(), fcntl::FlockArg::LockShared)?;
                    }
                }

                unistd::close(opened_file.as_raw_fd())?;
            }
        }
    } else {
        for entry in queue_files {
            let opened_file = open_file(&entry.filepath)?;

            if fcntl::flock(opened_file.as_raw_fd(), fcntl::FlockArg::LockSharedNonblock).is_err() {
                if (errno::EWOULDBLOCK as i32) == errno::errno() {
                    fcntl::flock(opened_file.as_raw_fd(), fcntl::FlockArg::LockShared)?;
                }
            }

            // Remove process lock
            unistd::close(opened_file.as_raw_fd())?;
        }
    }

    Ok(())
}
