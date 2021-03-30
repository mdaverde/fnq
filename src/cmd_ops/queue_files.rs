use std::{ffi, fs, io, path, time};

use crate::cmd_ops::{os_strings, QUEUE_FILE_PREFIX};

pub struct QueueFile {
    pub filepath: path::PathBuf,
    created: time::SystemTime,
}

pub fn queue_files_sorted(queue_dir: &path::PathBuf) -> Result<Vec<QueueFile>, io::Error> {
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
                    && os_strings::os_string_starts_with(
                        filepath.as_os_str(),
                        (&file_path_prefix).as_os_str(),
                    );
            }
            return false;
        })
        .map(|dir_entry| {
            dir_entry.and_then(|dir_entry| {
                Ok(QueueFile {
                    filepath: dir_entry.path(),
                    created: dir_entry.metadata()?.created()?,
                })
            })
        })
        .collect::<Result<Vec<QueueFile>, _>>()?;

    queue_files.sort_by(|file_a, file_b| file_a.created.cmp(&file_b.created));

    Ok(queue_files)
}
