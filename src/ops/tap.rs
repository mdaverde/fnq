use std::os::unix::prelude::*;
use std::path;

use nix::{fcntl, unistd};

use crate::ops::{files, open_file, OpsError};

pub fn tap(queue_dir: &path::PathBuf, queue_file: Option<path::PathBuf>) -> Result<bool, OpsError> {
    let queue_files = files::files(&queue_dir)?;

    if let Some(queue_file) = queue_file {
        let entry = queue_files
            .iter()
            .find(|&entry| entry.filepath.eq(&queue_file));

        match entry {
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
                    return Ok(true);
                }

                // Remove process lock
                unistd::close(opened_file.as_raw_fd())?;
            }
        }
    } else {
        for entry in queue_files {
            let opened_file = open_file(&entry.filepath)?;

            if fcntl::flock(opened_file.as_raw_fd(), fcntl::FlockArg::LockSharedNonblock).is_err() {
                return Ok(true);
            }

            // Remove lock
            unistd::close(opened_file.as_raw_fd())?;
        }
    }

    Ok(false)
}
