use std::env;



enum CommandFlag {
    Quiet,
    CleanUp,
    Test,
    Watch,
}

#[derive(Debug, PartialEq)]
enum ParseResult {
    Error,
    Test,
    Watch,
    Queue(String, bool, bool)
}

fn parse_args(args: Vec<String>) -> ParseResult {
    match args.len() {
        0 | 1 => {
            ParseResult::Error
        },
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

fn queue(cmd: String, quiet: bool, cleanup: bool) {

}


// Flags
// -q / --quiet = no output to stdout
// -c / --cleanup = delete file after done
// -t / --tap? = check if all operations are done; return exit code 1 if not
// -w / --watch = wait until all operations are done

fn main() {
    let args = env::args().collect();
    match parse_args(args) {
        ParseResult::Error => {
            print_usage();
            panic!(); // Did not understand args
        },
        ParseResult::Test => {
            println!("Testing if operations are done...");
        },
        ParseResult::Watch => {
            println!("Watching until operations are done...");
        },
        ParseResult::Queue(cmd, quiet, cleanup) => {
            println!("Going to run...{}", cmd);
            queue(cmd, quiet, cleanup); // Will this error out
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



