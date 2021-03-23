use std::{env, ffi, fs, path};

mod parser;
mod queue;


fn print_usage() {
    let usage = "Wrong usage";
    println!("{}", usage);
}

fn ensure_dir(dir: &ffi::OsStr) -> path::PathBuf {
    let dir_path = path::PathBuf::from(dir);
    if !dir_path.exists() {
        fs::create_dir(&dir_path).unwrap();
        // TODO: change to correct permissions: 0777
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
    match parser::parse_args(args) {
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
            queue::queue(task_cmd, task_args, dir_path, quiet, cleanup); // How do we want to handle errors here?
        }
    }
}
