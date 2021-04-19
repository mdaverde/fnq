use std::{env, ffi, fs, path, process};

mod ops;
mod parser;

static USAGE: &'static str = "fnq - A flock-based approach to queuing Unix tasks & processes

USAGE:
    fnq [FLAGS] <command>
    fnq --tap <queue file>
    fnq --wait <queue file>

FLAGS:
    -c, --clean       Removes queue file after process complete
    -q, --quiet       No print out of queue file to stdout
    -t, --tap         Determines if queue file's process is complete. If no
                      queue file specified, then checks all in FNQ_DIR
    -b, --block       Will block if queue file's process is not complete. If no
                      queue file specified, then blocks on all in FNQ_DIR
    -w, --watch       Similar to --block but will print to stdout contents of the
                      currently running queue files
    -v, --version     Prints version information
    -h, --help        Prints help information
";

static VERSION: &'static str = env!("CARGO_PKG_VERSION");

fn print_usage() {
    println!("{}", USAGE);
}

fn ensure_dir(dir: ffi::OsString) -> path::PathBuf {
    let dir_path: path::PathBuf = dir.into();
    if !dir_path.exists() {
        // TODO: change to correct permissions? (0777)
        fs::create_dir(&dir_path)
            .expect(format!("Was unable to create dir {:?}", dir_path).as_str());
    } else if !dir_path.is_dir() {
        panic!("$FNQ_DIR is not a directory");
    }
    dir_path
}

fn get_queue_path(
    dir_path: &path::PathBuf,
    queue_file: Option<ffi::OsString>,
) -> Result<Option<path::PathBuf>, String> {
    if let Some(file) = queue_file {
        let mut queue_path = path::PathBuf::from(&dir_path);
        queue_path.push(&file);
        if !queue_path.exists() {
            return Err(format!("{:?} does not exist", queue_path));
        }
        return Ok(Some(queue_path));
    }
    Ok(None)
}

fn main() {
    use parser::ParseResult;

    let args = env::args_os().collect();
    let fnq_dir = env::var_os("FNQ_DIR").unwrap_or(ffi::OsString::from("."));
    let dir_path = ensure_dir(fnq_dir);
    match parser::parse_args(args) {
        ParseResult::Version => {
            println!("{}", VERSION);
        }
        ParseResult::Help => {
            print_usage();
        }
        ParseResult::Error => {
            print_usage();
            process::exit(1);
        }
        ParseResult::Tap(queue_file) => match get_queue_path(&dir_path, queue_file) {
            Err(err) => {
                eprintln!("Error: {}", err);
            }
            Ok(queue_path) => {
                if let Err(err) = ops::tap(&dir_path, queue_path).map(|is_running| {
                    if is_running {
                        println!("running!");
                        process::exit(1);
                    } else {
                        println!("not running!");
                        process::exit(0);
                    }
                }) {
                    eprintln!("Error {:?}", err);
                }
            }
        },
        ParseResult::Block(queue_file) => match get_queue_path(&dir_path, queue_file) {
            Err(err) => {
                eprintln!("{}", err);
            }
            Ok(queue_path) => {
                if let Err(err) = ops::block(dir_path, queue_path) {
                    eprintln!("Error {:?}", err);
                }
            }
        },
        ParseResult::Queue(fnd_cmd, task_cmd, task_args, quiet, clean) => {
            if let Err(err) = ops::queue(fnd_cmd, task_cmd, task_args, dir_path, quiet, clean) {
                // Note: possibly could be another process in which this writes to a different stdout
                eprintln!("Error: {:?}", err)
            }
        }
        ParseResult::Watch => {
            if let Err(err) = ops::watch(dir_path) {
                eprintln!("Error: {:?}", err)
            }
        }
    }
}
