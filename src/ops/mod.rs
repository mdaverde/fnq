pub use queue::queue;
pub use tap::tap;
pub use wait::wait;
pub use error::OpsError;

use std::{fs, io, path};

#[macro_use]
mod os_strings;
mod error;
mod files;
mod queue;
mod tap;
mod wait;

pub const QUEUE_FILE_PREFIX: &'static str = "fnq";

fn open_file(path_buf: &path::PathBuf) -> Result<fs::File, io::Error> {
    fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(path_buf)
}

// use std::{env, fs, panic, path};
// fn test_env<T>(test: T) -> ()
// where
//     T: FnOnce(path::PathBuf) -> () + panic::UnwindSafe,
// {
//     let mut test_queue_dir =
//         env::current_dir().expect("Could not access current working directory");
//     test_queue_dir.push("fnqtestdir");
//
//     if test_queue_dir.exists() {
//         fs::remove_dir_all(&test_queue_dir).expect("Could not remove everything within fnqtestdir");
//     }
//
//     fs::create_dir(&test_queue_dir).expect("Could not create test directory");
//
//     let result = panic::catch_unwind(|| {
//         test(test_queue_dir);
//     });
//
//     assert!(result.is_ok());
// }
//
// #[cfg(test)]
// mod tests {
//     use super::*;
//
//     #[test]
//     fn test() {
//         test_env(|queue_dir: path::PathBuf| {
//             println!("{}", queue_dir.to_string_lossy());
//
//             assert!(true);
//         })
//     }
// }
