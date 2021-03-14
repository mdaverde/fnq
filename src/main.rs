use std::os::unix::prelude::*;
use std::{env, fs, thread, time, path, io, process, ffi};
use nix::{unistd, sys, Error, fcntl};
use nix::sys::wait::WaitStatus;
use std::process::exit;
use std::io::Write;
use std::pin::Pin;
use std::ffi::{OsString, OsStr};

#[derive(Debug, PartialEq)]
enum ParseResult {
    Error,
    Test,
    Watch,
    Queue(String, Vec<String>, bool, bool),
}

fn parse_args(mut args: Vec<String>) -> ParseResult {
    match args.len() {
        0 | 1 => {
            ParseResult::Error
        }
        _ => {
            let mut cmd_index: Option<usize> = None;
            let mut quiet = false;
            let mut clean_up = false;

            for (index, arg) in (&args[1..]).iter().enumerate() {
                if arg == "--test" {
                    return ParseResult::Test;
                } else if arg == "--watch" {
                    return ParseResult::Watch;
                } else if arg == "--quiet" {
                    quiet = true;
                } else if arg == "--cleanup" {
                    clean_up = true;
                } else {
                    cmd_index = Some(index + 1);
                    break; // We've hit user commands
                }
            }

            if let Some(index) = cmd_index {
                let task_cmd = args.drain(index..index + 1).collect();
                let task_args = args.drain(index..).collect();
                return ParseResult::Queue(task_cmd, task_args, quiet, clean_up);
            }

            ParseResult::Error
        }
    }
}

fn print_usage() {
    let usage = "Wrong usage";
    println!("{}", usage);
}

struct TaskFileHandler {
    queue_dir: path::PathBuf,
    cmd: String,
    args: Vec<String>,
    time_id: String,
    pid: Option<u32>,
}

impl TaskFileHandler {
    fn set_pid(&mut self, pid: u32) {
        self.pid = Some(pid);
    }

    fn new(queue_dir: path::PathBuf, cmd: String, args: Vec<String>) -> Self {
        let now = time::SystemTime::now();
        let ms_since_epoch = match now.duration_since(time::UNIX_EPOCH) {
            Ok(duration) => {
                duration.as_millis()
            }
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

    fn filename(&self) -> String {
        match self.pid {
            None => todo!(),
            Some(pid) => {
                format!("fnq{}.{}", self.time_id, pid)
            }
        }
    }

    fn to_path(&self) -> path::PathBuf {
        let mut file_path = self.queue_dir.clone();
        file_path.push(self.filename());
        file_path
    }
}

fn queue(task_cmd: String, task_args: Vec<String>, queue_dir: path::PathBuf, quiet: bool, cleanup: bool) -> Result<(), Error> {
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
                    let task_filename: String = task_handler.filename();
                    if !quiet {
                        println!("{}", task_filename)
                    }
                    unistd::close(io::stdin().as_raw_fd());
                    unistd::close(io::stdout().as_raw_fd());
                    unistd::close(io::stderr().as_raw_fd());

                    unistd::close(pipe.1);

                    let child_status = sys::wait::wait();


                    let mut task_file = fs::OpenOptions::new()
                        .append(true)
                        .open(task_handler.to_path())
                        .unwrap();
                    task_file.set_permissions(fs::Permissions::from_mode(0o600));

                    match child_status {
                        Err(err) => todo!(),
                        Ok(WaitStatus::Exited(_, exit_code)) => {
                            writeln!(task_file, "[exited with status {}.]", exit_code);
                            if cleanup && exit_code == 0 {
                                fs::remove_file(task_handler.to_path());
                            }
                        }
                        Ok(WaitStatus::Signaled(_, signal, _)) => {
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

                    { // Creates scope to guarantee file close/drop at end
                        let mut task_file: fs::File = fs::OpenOptions::new()
                            .create_new(true)
                            .write(true)
                            .mode(0o600)
                            .open(task_handler.to_path())
                            .unwrap_or_else(|error| {
                                todo!();
                                panic!();
                            });

                        let task_file_descriptor = task_file.as_raw_fd();

                        fcntl::flock(task_file_descriptor, fcntl::FlockArg::LockExclusive);

                        writeln!(task_file, "exec {} {}", &task_handler.cmd, task_handler.args.join(" "));

                        // TODO: handle errors here
                        unistd::dup2(task_file_descriptor, io::stdout().as_raw_fd());
                        unistd::dup2(task_file_descriptor, io::stderr().as_raw_fd());

                        // TODO: handle waiting based on other files here

                        // Actually run command
                        writeln!(task_file, "");

                        task_file.set_permissions(fs::Permissions::from_mode(0o700));
                    }

                    unistd::setsid();

                    let cmd_os: OsString = ffi::OsString::from(task_handler.cmd);
                    let cmd_os_ref: &OsStr = cmd_os.as_ref();
                    let cmd_c: ffi::CString = ffi::CString::new(cmd_os_ref.as_bytes()).unwrap();

                    let mut args_os: Vec<OsString> = task_handler.args.iter().map(|arg| OsString::from(arg)).collect();
                    args_os.insert(0, cmd_os);
                    let args_c: Vec<ffi::CString> = args_os.iter().map(|arg| ffi::CString::new(arg.as_os_str().as_bytes()).unwrap()).collect();

                    unistd::execvp(&cmd_c, &args_c).unwrap();
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

fn ensure_dir(dir: &str) -> path::PathBuf {
    let dir_path = path::PathBuf::from(dir);
    if !dir_path.exists() {
        fs::create_dir(&dir_path).unwrap();
        // TODO: change to correct permissions: 0777
    } else if !dir_path.is_dir() {
        panic!("$FNQ_DIR is not a directory");
    }
    dir_path
}


// Flags
// -q / --quiet = no output to stdout
// -c / --cleanup = delete file after done
// -t / --test = check if all operations are done; return exit code 1 if not
// -w / --watch = wait until all operations are done (should have a verbose mode to log which operations are going on)
// --kill-all = kill all currently queued up operations; also does clean up
// -t & -w should allow a specific file to be awaited upon

fn main() {
    let args = env::args().collect();
    // TODO: should use absolute directory?
    let fnq_dir = env::var("FNQ_DIR").unwrap_or(String::from("."));
    match parse_args(args) {
        ParseResult::Error => {
            print_usage();
            panic!(); // Did not understand args
        }
        ParseResult::Test => {
            println!("Testing if operations are done...");
        }
        ParseResult::Watch => {
            println!("Watching until operations are done...");
        }
        ParseResult::Queue(task_cmd, task_args, quiet, cleanup) => {
            let dir_path = ensure_dir(&fnq_dir);
            queue(task_cmd, task_args, dir_path, quiet, cleanup); // How do we want to handle errors here?
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args() {
        let mut args: Vec<String> = vec![];
        assert_eq!(parse_args(args), ParseResult::Error);

        args = vec![String::from("fnq")];
        assert_eq!(parse_args(args), ParseResult::Error);

        args = vec![String::from("fnq"), String::from("--test")];
        assert_eq!(parse_args(args), ParseResult::Test);

        args = vec![String::from("fnq"), String::from("--watch")];
        assert_eq!(parse_args(args), ParseResult::Watch);

        args = vec![String::from("fnq"), String::from("--watch"), String::from("extra")];
        assert_eq!(parse_args(args), ParseResult::Watch);

        args = vec![String::from("fnq"), String::from("--quiet")];
        assert_eq!(parse_args(args), ParseResult::Error);

        args = vec![String::from("fnq"), String::from("--cleanup")];
        assert_eq!(parse_args(args), ParseResult::Error);

        args = vec![String::from("fnq"), String::from("--quiet"), String::from("sleep 2")];
        assert_eq!(parse_args(args), ParseResult::Queue(String::from("sleep"), vec!(String::from("2")), true, false));

        args = vec![String::from("fnq"), String::from("--cleanup"), String::from("sleep 2")];
        assert_eq!(parse_args(args), ParseResult::Queue(String::from("sleep 2"), vec!(String::from("2")), false, true));

        args = vec![String::from("fnq"), String::from("--cleanup"), String::from("--quiet"), String::from("sleep 2")];
        assert_eq!(parse_args(args), ParseResult::Queue(String::from("sleep 2"), vec!(String::from("2")), true, true));

        args = vec![String::from("fnq"), String::from("sleep 2")];
        assert_eq!(parse_args(args), ParseResult::Queue(String::from("sleep 2"), vec!(String::from("2")), false, false));

        args = vec![String::from("fnq"), String::from("sleep 2"), String::from("&&"), String::from("echo hello")];
        assert_eq!(parse_args(args), ParseResult::Queue(String::from("sleep 2 && echo hello"), vec!(String::from("2")), false, false));
    }
}



