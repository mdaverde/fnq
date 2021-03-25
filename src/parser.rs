use std::ffi;

#[derive(Debug, PartialEq)]
pub enum ParseResult {
    Error,
    Test,
    Watch,
    Queue(ffi::OsString, Vec<ffi::OsString>, bool, bool),
}

pub fn parse_args(mut args: Vec<ffi::OsString>) -> ParseResult {
    match args.len() {
        0 | 1 => ParseResult::Error,
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
                } else if arg == "--clean" {
                    clean_up = true;
                } else {
                    cmd_index = Some(index + 1);
                    break; // We've hit user commands
                }
            }

            if let Some(index) = cmd_index {
                let task_cmd = args.drain(index..index + 1).next().unwrap();
                let task_args = args.drain(index..).collect();
                return ParseResult::Queue(task_cmd, task_args, quiet, clean_up);
            }

            ParseResult::Error
        }
    }
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
        assert_eq!(parse_args(args), ParseResult::Test);

        args = vec![ffi::OsString::from("fnq"), ffi::OsString::from("--watch")];
        assert_eq!(parse_args(args), ParseResult::Watch);

        args = vec![
            ffi::OsString::from("fnq"),
            ffi::OsString::from("--watch"),
            ffi::OsString::from("extra"),
        ];
        assert_eq!(parse_args(args), ParseResult::Watch);

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
