use std::os::unix::prelude::*;
use std::{env, fs, thread, time, path, io, process};
use nix::{unistd, sys, Error, fcntl};
use nix::sys::wait::WaitStatus;
use std::process::exit;
use std::io::Write;

#[derive(Debug, PartialEq)]
enum ParseResult {
    Error,
    Test,
    Watch,
    Queue(String, bool, bool),
}

fn parse_args(args: Vec<String>) -> ParseResult {
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
                    cmd_index = Some(index);
                    break; // We've hit user commands
                }
            }

            if cmd_index.is_none() {
                ParseResult::Error
            } else {
                let index = cmd_index.unwrap() + 1;
                let cmd = (&args[index..]).join(" ").to_owned();
                ParseResult::Queue(cmd, quiet, clean_up)
            }
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
    time_id: String,
    pid: Option<u32>,
}

impl TaskFileHandler {
    fn set_pid(&mut self, pid: u32) {
        self.pid = Some(pid);
    }

    fn new(queue_dir: path::PathBuf, cmd: String) -> Self {
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
        println!("Creating file identifier with time: {}", ms_since_epoch);
        Self {
            queue_dir,
            cmd,
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
        file_path.set_file_name(self.filename());
        file_path
    }
}

fn queue(task_cmd: String, queue_dir: path::PathBuf, quiet: bool, cleanup: bool) -> Result<(), Error> {
    let mut task_handler = TaskFileHandler::new(queue_dir, task_cmd);
    let pipe = unistd::pipe()?;
    let child_fork = unsafe { unistd::fork() };
    match child_fork {
        Ok(unistd::ForkResult::Parent { child }) => {
            println!("From parent process, new child pid: {}", child);
            // should end process when necessary setup is complete:
            // - child is backgrounded and ready for grandchild to start doing work
            let mut c: [u8; 1] = [0];
            unistd::close(pipe.1);
            // Will wait until grandchild process is ready
            unistd::read(pipe.0, &mut c);
            println!("Exiting parent process");
            process::exit(0);
        }
        Ok(unistd::ForkResult::Child) => {
            println!("This is the child process");
            unistd::close(pipe.0);
            let grandchild_fork = unsafe { unistd::fork() };
            match grandchild_fork {
                Ok(unistd::ForkResult::Parent { child }) => {
                    println!("This is the child process, new grandchild pid: {}", child);
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

                    // Grandchild process has changed status; remove this
                    println!("Blahhhhh We should never see this");

                    let mut task_file = fs::OpenOptions::new()
                        .append(true)
                        .open(task_handler.to_path())
                        .unwrap_or_else(|error| {
                            todo!();
                            panic!();
                        });
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
                    println!("This is the grandchild process");

                    let mut task_file: fs::File = fs::OpenOptions::new()
                        .create_new(true)
                        .write(true)
                        .mode(0o600)
                        .open(task_handler.to_path())
                        .unwrap_or_else(|error| {
                            todo!();
                            panic!();
                        });

                    fcntl::flock(task_file.as_raw_fd(), fcntl::FlockArg::LockExclusive);

                    writeln!(task_file, "exec {}", &task_handler.cmd);
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
// -w / --watch = wait until all operations are done
// --kill-all = kill all currently queued up operations; also does clean up

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
        ParseResult::Queue(task_cmd, quiet, cleanup) => {
            println!("Going to run...{}", task_cmd);
            let dir_path = ensure_dir(&fnq_dir);
            queue(task_cmd, dir_path, quiet, cleanup); // How do we want to handle errors here?
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
        assert_eq!(parse_args(args), ParseResult::Queue(String::from("sleep 2"), true, false));

        args = vec![String::from("fnq"), String::from("--cleanup"), String::from("sleep 2")];
        assert_eq!(parse_args(args), ParseResult::Queue(String::from("sleep 2"), false, true));

        args = vec![String::from("fnq"), String::from("--cleanup"), String::from("--quiet"), String::from("sleep 2")];
        assert_eq!(parse_args(args), ParseResult::Queue(String::from("sleep 2"), true, true));

        args = vec![String::from("fnq"), String::from("sleep 2")];
        assert_eq!(parse_args(args), ParseResult::Queue(String::from("sleep 2"), false, false));

        args = vec![String::from("fnq"), String::from("sleep 2"), String::from("&&"), String::from("echo hello")];
        assert_eq!(parse_args(args), ParseResult::Queue(String::from("sleep 2 && echo hello"), false, false));
    }
}



