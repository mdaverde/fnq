use std::ffi;

#[derive(Debug, PartialEq)]
pub enum ParseResult {
    Error,
    TestAll,
    TestSingle(ffi::OsString),
    WatchAll,
    WatchSingle(ffi::OsString),
    Queue(ffi::OsString, Vec<ffi::OsString>, bool, bool),
}

pub fn parse_args(mut args: Vec<ffi::OsString>) -> ParseResult {
    let len = args.len();
    if len < 2 {
        return ParseResult::Error;
    }

    let arg = &args[1];
    if arg == "--test" {
        return if len == 2 {
            ParseResult::TestAll
        } else if len == 3 {
            ParseResult::TestSingle(args.drain(2..3).next().unwrap())
        } else {
            ParseResult::Error
        }
    } else if arg == "--watch" {
        return if len == 2 {
            ParseResult::WatchAll
        } else if len == 3 {
            ParseResult::WatchSingle(args.drain(2..3).next().unwrap())
        } else {
            ParseResult::Error
        }
    }

    let mut index: usize = 1;
    let mut quiet = false;
    let mut clean = false;

    for arg in &args[1..] {
       if arg == "--quiet" {
           quiet = true;
           index += 1;
       } else if arg == "--clean" {
           clean = true;
           index += 1;
       } else {
           break;
       }
    }

    if index < len {
        let task_cmd = args.drain(index..index + 1).next().unwrap();
        let task_args = args.drain(index..).collect();
        return ParseResult::Queue(task_cmd, task_args, quiet, clean);
    }

    ParseResult::Error
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_args() {
        let mut args: Vec<ffi::OsString> = vec![];
        assert_eq!(parse_args(args), ParseResult::Error);

        args = vec![ffi::OsString::from("fnq")];
        assert_eq!(parse_args(args), ParseResult::Error);

        args = vec![ffi::OsString::from("fnq"), ffi::OsString::from("--test")];
        assert_eq!(parse_args(args), ParseResult::TestAll);

        args = vec![ffi::OsString::from("fnq"), ffi::OsString::from("--watch")];
        assert_eq!(parse_args(args), ParseResult::WatchAll);

        args = vec![
            ffi::OsString::from("fnq"),
            ffi::OsString::from("--watch"),
            ffi::OsString::from("some_random_file"),
        ];
        assert_eq!(parse_args(args), ParseResult::WatchSingle("some_random_file".into()));

        args = vec![ffi::OsString::from("fnq"), ffi::OsString::from("--quiet")];
        assert_eq!(parse_args(args), ParseResult::Error);

        args = vec![ffi::OsString::from("fnq"), ffi::OsString::from("--clean")];
        assert_eq!(parse_args(args), ParseResult::Error);

        args = vec![
            ffi::OsString::from("fnq"),
            ffi::OsString::from("--quiet"),
            ffi::OsString::from("sleep"),
            ffi::OsString::from("2"),
        ];
        assert_eq!(
            parse_args(args),
            ParseResult::Queue(
                ffi::OsString::from("sleep"),
                vec!(ffi::OsString::from("2")),
                true,
                false
            )
        );

        args = vec![
            ffi::OsString::from("fnq"),
            ffi::OsString::from("--clean"),
            ffi::OsString::from("sleep"),
            ffi::OsString::from("2"),
        ];

        assert_eq!(
            parse_args(args),
            ParseResult::Queue(
                ffi::OsString::from("sleep"),
                vec!(ffi::OsString::from("2")),
                false,
                true,
            )
        );

        args = vec![
            ffi::OsString::from("fnq"),
            ffi::OsString::from("--clean"),
            ffi::OsString::from("--quiet"),
            ffi::OsString::from("sleep"),
            ffi::OsString::from("2"),
        ];
        assert_eq!(
            parse_args(args),
            ParseResult::Queue(
                ffi::OsString::from("sleep"),
                vec!(ffi::OsString::from("2")),
                true,
                true
            )
        );

        args = vec![ffi::OsString::from("fnq"), ffi::OsString::from("sleep")];
        assert_eq!(
            parse_args(args),
            ParseResult::Queue(ffi::OsString::from("sleep"), vec!(), false, false,)
        );
    }
}
