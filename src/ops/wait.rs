use std::path;

use crate::ops::{files, lock_on_blocked_file, OpsError};

pub fn wait(queue_dir: path::PathBuf, queue_file: Option<path::PathBuf>) -> Result<(), OpsError> {
    let queue_files = files::files(&queue_dir)?;
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
                lock_on_blocked_file(&entry.filepath)?;
            }
        }
    } else {
        for entry in queue_files {
            lock_on_blocked_file(&entry.filepath)?;
        }
    }

    Ok(())
}
