use nix::{errno, fcntl, sys, unistd};
use std::io::Write;
use std::os::unix::prelude::*;
use std::{env, ffi, fs, io, panic, path, process, time};

macro_rules! concat_os_strings {
    ($($elem: expr),*) => {{
        let mut os_string = ffi::OsString::new();
        $(os_string.push($elem);)*
        os_string
    }};
}

fn os_string_starts_with(os_a: &ffi::OsStr, os_b: &ffi::OsStr) -> bool {
    let bytes_a = os_a.as_bytes();
    let bytes_b = os_b.as_bytes();
    if bytes_b.len() > bytes_a.len() {
        return false;
    }
    for (i, b) in bytes_a[..bytes_b.len()].iter().enumerate() {
        if !bytes_b[i].eq(b) {
            return false;
        }
    }

    true
}

const QUEUE_FILE_PREFIX: &'static str = "fnq";

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

    fn new(queue_dir: path::PathBuf, cmd: ffi::OsString, args: Vec<ffi::OsString>) -> Self {
        let now = time::SystemTime::now();
        let ms_since_epoch = match now.duration_since(time::UNIX_EPOCH) {
            Ok(duration) => duration.as_millis(),
            Err(err) => {
                // Should abort?
                todo!();
            }
        };
        Self {
            queue_dir,
            cmd,
            args,
            time_id: ms_since_epoch.to_string(),
            pid: None,
        }
    }

    fn filename(&self) -> ffi::OsString {
        match self.pid {
            None => todo!(),
            Some(pid) => {
                concat_os_strings!(
                    ffi::OsString::from(QUEUE_FILE_PREFIX),
                    ffi::OsString::from(&self.time_id),
                    ffi::OsString::from("."),
                    ffi::OsString::from(pid.to_string())
                )
            }
        }
    }

    fn path(&self) -> path::PathBuf {
        let mut file_path = self.queue_dir.clone();
        file_path.push(self.filename());
        file_path
    }
}

struct QueueFile {
    pub filepath: path::PathBuf,
    pub metadata: fs::Metadata,
}

fn queue_files_sorted(queue_dir: &path::PathBuf) -> Vec<QueueFile> {
    // TODO: Look into OsStrExt & ffi::OsStringExt for Unix
    let file_path_prefix = concat_os_strings!(
        queue_dir,
        ffi::OsString::from("/"),
        ffi::OsString::from(QUEUE_FILE_PREFIX)
    );

    let mut queue_files: Vec<QueueFile> = fs::read_dir(queue_dir)
        .unwrap()
        .into_iter()
        .map(|dir_entry| dir_entry.unwrap())
        .filter(|dir_entry| {
            let filepath = dir_entry.path();
            return filepath.is_file()
                && os_string_starts_with(filepath.as_os_str(), (&file_path_prefix).as_os_str());
        })
        .map(|dir_entry| QueueFile {
            filepath: dir_entry.path(),
            metadata: dir_entry.metadata().unwrap(),
        })
        .collect();

    queue_files.sort_by(|file_a, file_b| {
        let meta_b_created = file_b.metadata.created().unwrap();
        file_a.metadata.created().unwrap().cmp(&meta_b_created)
    });

    queue_files
}

pub fn queue_test(queue_dir: path::PathBuf) -> bool {
    let queue_files = queue_files_sorted(&queue_dir);

    for entry in queue_files {
        let opened_file: fs::File = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&entry.filepath)
            .unwrap();

        let lockable = fcntl::flock(opened_file.as_raw_fd(), fcntl::FlockArg::LockSharedNonblock);

        if lockable.is_err() {
            return false;
        }

        // Remove process lock
        unistd::close(opened_file.as_raw_fd());
    }
    true
}

pub fn queue_wait(queue_dir: path::PathBuf) -> bool {
    let queue_files = queue_files_sorted(&queue_dir);

    for entry in queue_files {
        let opened_file: fs::File = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(&entry.filepath)
            .unwrap();

        let lockable = fcntl::flock(opened_file.as_raw_fd(), fcntl::FlockArg::LockSharedNonblock);

        if lockable.is_err() {
            if (errno::EWOULDBLOCK as i32) == errno::errno() {
                fcntl::flock(opened_file.as_raw_fd(), fcntl::FlockArg::LockShared);
            }
        }

        // Remove process lock
        unistd::close(opened_file.as_raw_fd());
    }
    true
}

pub fn queue(
    task_cmd: ffi::OsString,
    task_args: Vec<ffi::OsString>,
    queue_dir: path::PathBuf,
    quiet: bool,
    clean: bool,
) -> Result<(), nix::Error> {
    let mut task_handler = TaskFileHandler::new(queue_dir, task_cmd, task_args);
    let pipe = unistd::pipe()?;
    let child_fork = unsafe { unistd::fork() };
    match child_fork {
        Ok(unistd::ForkResult::Parent { child }) => {
            // should end process when necessary setup is complete:
            // - child is backgrounded and ready for grandchild to start doing work
            let mut c: [u8; 1] = [0];
            unistd::close(pipe.1);
            // Will wait until grandchild process is ready
            unistd::read(pipe.0, &mut c);
            process::exit(0);
        }
        Ok(unistd::ForkResult::Child) => {
            unistd::close(pipe.0);

            let grandchild_fork = unsafe { unistd::fork() };
            match grandchild_fork {
                Ok(unistd::ForkResult::Parent { child }) => {
                    let child_pid = child.as_raw();
                    if child_pid.is_negative() {
                        // How is this ever negative?
                        todo!();
                    }
                    task_handler.set_pid(child_pid as u32);
                    let task_filename = task_handler.filename();
                    if !quiet {
                        writeln!(io::stdout(), "{}", task_filename.to_string_lossy());
                    }
                    unistd::close(io::stdin().as_raw_fd());
                    unistd::close(io::stdout().as_raw_fd());
                    unistd::close(io::stderr().as_raw_fd());

                    unistd::close(pipe.1);

                    let child_status = sys::wait::wait();

                    let mut task_file = fs::OpenOptions::new()
                        .append(true)
                        .open(task_handler.path())
                        .unwrap();
                    task_file.set_permissions(fs::Permissions::from_mode(0o600));

                    match child_status {
                        Err(err) => todo!(),
                        Ok(sys::wait::WaitStatus::Exited(_, exit_code)) => {
                            writeln!(task_file, "[exited with status {}.]", exit_code);
                            if clean && exit_code == 0 {
                                fs::remove_file(task_handler.path());
                            }
                        }
                        Ok(sys::wait::WaitStatus::Signaled(_, signal, _)) => {
                            writeln!(task_file, "[received signal {}.]", signal);
                        }
                        _ => {
                            todo!();
                        }
                    }

                    process::exit(0)
                }
                Ok(unistd::ForkResult::Child) => {
                    unistd::close(pipe.1);
                    task_handler.set_pid(process::id());

                    let task_file_path = task_handler.path();

                    // Creates scope to guarantee file close/drop at end
                    let mut task_file: fs::File = fs::OpenOptions::new()
                        .create_new(true)
                        .write(true)
                        .mode(0o600)
                        .open(&task_file_path)
                        .unwrap_or_else(|error| {
                            todo!();
                        });

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

                    for entry in queue_files_sorted(&task_handler.queue_dir) {
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
                Err(err) => {
                    todo!();
                }
            }
            Ok(())
        }
        Err(err) => {
            todo!();
        }
    }
}

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
