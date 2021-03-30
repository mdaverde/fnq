use std::{ffi, fs, io, path, time};

use crate::ops::os_strings::OsStringStartsWithExt;
use crate::ops::QUEUE_FILE_PREFIX;

pub struct QueueEntry {
    pub filepath: path::PathBuf,
    created: time::SystemTime,
}

pub fn files(queue_dir: &path::PathBuf) -> Result<Vec<QueueEntry>, io::Error> {
    let file_path_prefix = concat_os_strings!(
        queue_dir,
        ffi::OsString::from("/"),
        ffi::OsString::from(QUEUE_FILE_PREFIX)
    );

    let mut queue_files = fs::read_dir(queue_dir)?
        .into_iter()
        .filter(|dir_entry| {
            if let Ok(dir_entry) = dir_entry {
                let filepath = dir_entry.path();
                return filepath.is_file()
                    && filepath
                        .as_os_str()
                        .starts_with(file_path_prefix.as_os_str());
            }
            return false;
        })
        .map(|dir_entry| {
            dir_entry.and_then(|dir_entry| {
                Ok(QueueEntry {
                    filepath: dir_entry.path(),
                    created: dir_entry.metadata()?.created()?,
                })
            })
        })
        .collect::<Result<Vec<QueueEntry>, _>>()?;

    queue_files.sort_by(|file_a, file_b| file_a.created.cmp(&file_b.created));

    Ok(queue_files)
}
