use std::os::unix::prelude::*;
use std::{fs, io, path};

use nix::fcntl;

use crate::ops::{files, OpsError};

fn is_locked(raw_fd: RawFd) -> Result<bool, OpsError> {
    match fcntl::flock(raw_fd, fcntl::FlockArg::LockSharedNonblock) {
        Ok(_) => {
            fcntl::flock(raw_fd, fcntl::FlockArg::Unlock)?;
            Ok(false)
        }
        Err(nix::Error::Sys(nix::errno::EWOULDBLOCK)) => Ok(true),
        Err(err) => Err(OpsError::from(err)),
    }
}

pub fn watch(queue_dir: path::PathBuf) -> Result<(), OpsError> {
    let queue_files = files::files(&queue_dir)?;
    for entry in queue_files {
        let mut queue_file = fs::OpenOptions::new().read(true).open(&entry.filepath)?;
        let fd: RawFd = queue_file.as_raw_fd();
        match fcntl::flock(fd, fcntl::FlockArg::LockSharedNonblock) {
            Ok(_) => {
                fcntl::flock(fd, fcntl::FlockArg::Unlock)?;
            }
            Err(nix::Error::Sys(nix::errno::EWOULDBLOCK)) => {
                use notify::{raw_watcher, Op, RawEvent, RecursiveMode, Watcher};
                use std::sync::mpsc::channel;

                println!("===> {}", entry.filepath.to_string_lossy());

                io::copy(&mut queue_file, &mut io::stdout())?;

                let (tx, rx) = channel();
                let mut watcher = raw_watcher(tx)?;
                watcher
                    .watch(&entry.filepath, RecursiveMode::NonRecursive)?;

                // We have to wait for 2 close events: the child process with the exclusive flock and
                // the parent that determines exit status both have a file handle that closes
                let mut close_count = 0;

                while is_locked(fd)? || close_count < 2 {
                    match rx.recv() {
                        Ok(RawEvent {
                            path: _path,
                            op: Ok(op),
                            cookie: _cookie,
                        }) => match op {
                            Op::WRITE => {
                                io::copy(&mut queue_file, &mut io::stdout())?;
                            }
                            Op::CLOSE_WRITE => {
                                io::copy(&mut queue_file, &mut io::stdout())?;
                                close_count = close_count + 1;
                            }
                            Op::RENAME => {
                                return Err(OpsError::Unknown(
                                    "Queue file was renamed or deleted".into(),
                                ));
                            }
                            _ => {}
                        },
                        Ok(event) => return Err(OpsError::WatcherUnknown(format!("Broken event: {:?}", event))),
                        Err(e) => return Err(OpsError::WatcherUnknown(format!("Watch error: {:?}", e))),
                    };
                }
            }
            Err(err) => return Err(OpsError::from(err)),
        };
    }

    Ok(())
}
