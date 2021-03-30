use std::io::Write;
use std::os::unix::prelude::*;
use std::{env, error, ffi, fs, io, panic, path, process, time};

use nix::{errno, fcntl, sys, unistd};

pub use queue::queue;
pub use tap::tap;
pub use wait::wait;

#[macro_use]
mod os_strings;
mod queue;
mod queue_files;
mod tap;
mod wait;

pub const QUEUE_FILE_PREFIX: &'static str = "fnq";

// enum CmdOpsError {
//     IO(io::Error)
// }
//
// impl From<io::Error> for CmdOpsError {
//     fn from(err: Error) -> Self {
//         CmdOpsError::IO(err)
//     }
// }

fn test_env<T>(test: T) -> ()
where
    T: FnOnce(path::PathBuf) -> () + panic::UnwindSafe,
{
    let mut test_queue_dir =
        env::current_dir().expect("Could not access current working directory");
    test_queue_dir.push("fnqtestdir");

    if test_queue_dir.exists() {
        fs::remove_dir_all(&test_queue_dir).expect("Could not remove everything within fnqtestdir");
    }

    fs::create_dir(&test_queue_dir).expect("Could not create test directory");

    let result = panic::catch_unwind(|| {
        test(test_queue_dir);
    });

    assert!(result.is_ok());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        test_env(|queue_dir: path::PathBuf| {
            println!("{}", queue_dir.to_string_lossy());

            assert!(true);
        })
    }
}
