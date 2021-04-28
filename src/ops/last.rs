use std::{io, path};

use crate::ops::{files, open_file, OpsError};

pub fn last(queue_dir: path::PathBuf) -> Result<(), OpsError> {
    let queue_files = files::files(&queue_dir)?;
    let last_queue_file = queue_files.last();
    if let None = last_queue_file {
        return Err(OpsError::QueueEmpty);
    } else if let Some(queue_file) = last_queue_file {
        let mut opened = open_file(&queue_file.filepath)?;
        io::copy(&mut opened, &mut io::stdout())?;
    }

    Ok(())
}
