use std::{env, ffi, fs, path, process};

mod cmd_ops;
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
        ParseResult::TapAll => {
            if let Err(err) = cmd_ops::tap(dir_path)
                .map(|is_queue_ready| process::exit(if is_queue_ready { 0 } else { 1 }))
            {
                eprintln!("Queue test error {:?}", err);
            }
        }
        ParseResult::WatchAll => {
            if let Err(err) = cmd_ops::wait(dir_path) {
                eprintln!("Queue watch error {:?}", err);
            }
        }
        ParseResult::Queue(task_cmd, task_args, quiet, clean) => {
            if let Err(err) = cmd_ops::queue(task_cmd, task_args, dir_path, quiet, clean) {
                // Note: possibly could be another process in which this writes to a different stdout
                eprintln!("Queue error: {:?}", err)
            }
        }
        _ => {
            println!("other cmd")
        }
    }
}
