use std::path;

use crate::ops::{block_on_locked_file, files, OpsError};

pub fn block(queue_dir: path::PathBuf, queue_file: Option<path::PathBuf>) -> Result<(), OpsError> {
    let queue_files = files::files(&queue_dir)?;
    if let Some(queue_file) = queue_file {
        match queue_files
            .iter()
            .find(|entry| entry.filepath.eq(&queue_file))
        {
            None => {
                return Err(OpsError::FileNotFound(queue_file.into()))
            }
            Some(entry) => {
                block_on_locked_file(&entry.filepath)?;
            }
        }
    } else {
        for entry in queue_files {
            block_on_locked_file(&entry.filepath)?;
        }
    }

    Ok(())
}
