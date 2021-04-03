use std::{env, ffi, fs, path, process};

mod ops;
mod parser;

fn print_usage() {
    let usage = "Wrong usage";
    println!("{}", usage);
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
    // TODO: should use absolute directory?
    let fnq_dir = env::var_os("FNQ_DIR").unwrap_or(ffi::OsString::from("."));
    let dir_path = ensure_dir(fnq_dir);
    match parser::parse_args(args) {
        ParseResult::Error => {
            print_usage();
            panic!(); // Did not understand args
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
        ParseResult::Wait(queue_file) => match get_queue_path(&dir_path, queue_file) {
            Err(err) => {
                eprintln!("{}", err);
            }
            Ok(queue_path) => {
                if let Err(err) = ops::wait(dir_path, queue_path) {
                    eprintln!("Error {:?}", err);
                }
            }
        },
        ParseResult::Queue(task_cmd, task_args, quiet, clean) => {
            if let Err(err) = ops::queue(task_cmd, task_args, dir_path, quiet, clean) {
                // Note: possibly could be another process in which this writes to a different stdout
                eprintln!("Error: {:?}", err)
            }
        }
    }
}
