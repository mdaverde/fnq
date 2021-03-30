use nix::{errno, fcntl, sys, unistd};
use std::io::Write;
use std::os::unix::prelude::*;
use std::{error, ffi, fs, io, path, process, time};

use crate::cmd_ops::{os_strings, queue_files, QUEUE_FILE_PREFIX};

struct TaskFileHandler {
    pub queue_dir: path::PathBuf,
    cmd: ffi::OsString,
    args: Vec<ffi::OsString>,
    time_id: String,
    pid: Option<u32>,
}

impl TaskFileHandler {
    fn set_pid(&mut self, pid: u32) {
        self.pid = Some(pid);
    }

    fn new(
        queue_dir: path::PathBuf,
        cmd: ffi::OsString,
        args: Vec<ffi::OsString>,
    ) -> Result<Self, time::SystemTimeError> {
        let time_id = time::SystemTime::now()
            .duration_since(time::UNIX_EPOCH)?
            .as_millis()
            .to_string();
        Ok(Self {
            queue_dir,
            cmd,
            args,
            time_id,
            pid: None,
        })
    }

    fn filename(&self) -> Result<ffi::OsString, String> {
        match self.pid {
            None => Err("Has not set pid on task file handler yet".into()),
            Some(pid) => Ok(concat_os_strings!(
                ffi::OsString::from(QUEUE_FILE_PREFIX),
                ffi::OsString::from(&self.time_id),
                ffi::OsString::from("."),
                ffi::OsString::from(pid.to_string())
            )),
        }
    }

    fn path(&self) -> Result<path::PathBuf, String> {
        let mut file_path = self.queue_dir.clone();
        file_path.push(self.filename()?);
        Ok(file_path)
    }
}

pub fn queue(
    task_cmd: ffi::OsString,
    task_args: Vec<ffi::OsString>,
    queue_dir: path::PathBuf,
    quiet: bool,
    clean: bool,
) -> Result<(), Box<dyn error::Error>> {
    let mut task_handler = TaskFileHandler::new(queue_dir, task_cmd, task_args)?;
    let pipe = unistd::pipe()?;
    let child_fork = unsafe { unistd::fork()? };
    match child_fork {
        unistd::ForkResult::Parent { child: _ } => {
            let mut c: [u8; 1] = [0];
            unistd::close(pipe.1)?;
            // Will wait until grandchild process is ready
            unistd::read(pipe.0, &mut c)?;
        }
        unistd::ForkResult::Child => {
            unistd::close(pipe.0)?;
            let grandchild_fork = unsafe { unistd::fork()? };
            match grandchild_fork {
                unistd::ForkResult::Parent { child } => {
                    let child_pid = child.as_raw();
                    if child_pid.is_negative() {
                        return Err(format!("Child pid is negative {}", child_pid).into());
                    }

                    task_handler.set_pid(child_pid as u32);
                    let task_filename = task_handler.filename()?;

                    if !quiet {
                        writeln!(io::stdout(), "{}", task_filename.to_string_lossy());
                    }

                    let mut task_file = fs::OpenOptions::new()
                        .append(true)
                        .open(task_handler.path()?)?;
                    task_file.set_permissions(fs::Permissions::from_mode(0o600));

                    unistd::close(io::stdin().as_raw_fd())?;
                    unistd::close(io::stdout().as_raw_fd())?;
                    unistd::close(io::stderr().as_raw_fd())?;

                    // Release initiating process
                    unistd::close(pipe.1)?;

                    match sys::wait::wait() {
                        Err(err) => {
                            // TODO: test this
                            writeln!(task_file, "[child process has errored out: {}.]", err).ok();
                        }
                        Ok(sys::wait::WaitStatus::Exited(_, exit_code)) => {
                            writeln!(task_file, "[exited with status {}.]", exit_code).ok();
                            if clean && exit_code == 0 {
                                if let Err(err) = fs::remove_file(task_handler.path()?) {
                                    writeln!(task_file, "[failed to remove file: {}.]", err).ok();
                                }
                            }
                        }
                        Ok(sys::wait::WaitStatus::Signaled(_, signal, _)) => {
                            writeln!(task_file, "[received signal {}.]", signal).ok();
                        }
                        Ok(unknown_state) => {
                            // TODO: test this
                            writeln!(
                                task_file,
                                "[child process has exited with unknown state: {:?}.]",
                                unknown_state
                            )
                            .ok();
                        }
                    }
                }
                unistd::ForkResult::Child => {
                    unistd::close(pipe.1);
                    task_handler.set_pid(process::id());

                    let task_file_path = task_handler.path()?;

                    let mut task_file: fs::File = fs::OpenOptions::new()
                        .create_new(true)
                        .write(true)
                        .mode(0o600)
                        .open(&task_file_path)
                        .unwrap();
                    // .expect(format!("Could not create task's queue file at {:?}", &task_file_path).as_str());

                    let task_file_descriptor = task_file.as_raw_fd();

                    fcntl::flock(task_file_descriptor, fcntl::FlockArg::LockExclusive);

                    let cmd_str = task_handler.cmd.to_str().unwrap();
                    let args_str: Vec<&str> = task_handler
                        .args
                        .iter()
                        .map(|arg| arg.to_str().unwrap())
                        .collect();
                    writeln!(task_file, "exec {} {}", cmd_str, args_str.join(" "));

                    unistd::dup2(task_file_descriptor, io::stdout().as_raw_fd());
                    unistd::dup2(task_file_descriptor, io::stderr().as_raw_fd());

                    for entry in queue_files::queue_files_sorted(&task_handler.queue_dir).unwrap() {
                        if entry.filepath == task_file_path {
                            // TODO: How do we test this?
                            continue;
                        }
                        let opened_file: fs::File = fs::OpenOptions::new()
                            .read(true)
                            .write(true)
                            .open(&entry.filepath)
                            .unwrap();

                        let lockable = fcntl::flock(
                            opened_file.as_raw_fd(),
                            fcntl::FlockArg::LockSharedNonblock,
                        );
                        if lockable.is_err() {
                            if (errno::EWOULDBLOCK as i32) == errno::errno() {
                                fcntl::flock(opened_file.as_raw_fd(), fcntl::FlockArg::LockShared);
                            } else {
                                println!("can not open {} {}", errno::errno(), errno::EWOULDBLOCK);
                                unimplemented!();
                            }
                        }

                        // Remove process lock
                        unistd::close(opened_file.as_raw_fd());
                    }

                    writeln!(task_file, "");

                    task_file.set_permissions(fs::Permissions::from_mode(0o700));

                    let cmd_c: ffi::CString =
                        ffi::CString::new(task_handler.cmd.as_os_str().as_bytes()).unwrap();
                    task_handler.args.insert(0, task_handler.cmd);
                    let args_c: Vec<ffi::CString> = task_handler
                        .args
                        .iter()
                        .map(|arg| ffi::CString::new(arg.as_os_str().as_bytes()).unwrap())
                        .collect();

                    unistd::setsid().unwrap();
                    if let Err(err) = unistd::execvp(&cmd_c, &args_c) {
                        if nix::Error::Sys(errno::Errno::ENOENT) == err {
                            panic!("{:?}: Could not find {:?} in path", err, &cmd_c);
                        } else {
                            panic!("{:?}", err);
                        }
                    }
                }
            }
        }
    }
    Ok(())
}