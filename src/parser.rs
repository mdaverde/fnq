use std::ffi;

#[derive(Debug, PartialEq)]
pub enum ParseResult {
    Error,
    TapAll,
    TapSingle(ffi::OsString),
    WaitAll,
    WaitSingle(ffi::OsString),
    Queue(ffi::OsString, Vec<ffi::OsString>, bool, bool),
}

pub fn parse_args(mut args: Vec<ffi::OsString>) -> ParseResult {
    let len = args.len();
    if len < 2 {
        return ParseResult::Error;
    }

    let arg = &args[1];
    if arg == "--tap" {
        return if len == 2 {
            ParseResult::TapAll
        } else if len == 3 {
            ParseResult::TapSingle(args.drain(2..3).next().unwrap())
        } else {
            ParseResult::Error
        };
    } else if arg == "--wait" {
        return if len == 2 {
            ParseResult::WaitAll
        } else if len == 3 {
            ParseResult::WaitSingle(args.drain(2..3).next().unwrap())
        } else {
            ParseResult::Error
        };
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

        args = vec!["fnq".into()];
        assert_eq!(parse_args(args), ParseResult::Error);

        args = vec!["fnq".into(), "--tap".into()];
        assert_eq!(parse_args(args), ParseResult::TapAll);

        args = vec!["fnq".into(), "--wait".into()];
        assert_eq!(parse_args(args), ParseResult::WaitAll);

        args = vec!["fnq".into(), "--wait".into(), "some_random_file".into()];
        assert_eq!(
            parse_args(args),
            ParseResult::WaitSingle("some_random_file".into())
        );

        args = vec!["fnq".into(), "--quiet".into()];
        assert_eq!(parse_args(args), ParseResult::Error);

        args = vec!["fnq".into(), "--clean".into()];
        assert_eq!(parse_args(args), ParseResult::Error);

        args = vec!["fnq".into(), "--quiet".into(), "sleep".into(), "2".into()];
        assert_eq!(
            parse_args(args),
            ParseResult::Queue("sleep".into(), vec!("2".into()), true, false)
        );

        args = vec!["fnq".into(), "--clean".into(), "sleep".into(), "2".into()];

        assert_eq!(
            parse_args(args),
            ParseResult::Queue("sleep".into(), vec!("2".into()), false, true,)
        );

        args = vec![
            "fnq".into(),
            "--clean".into(),
            "--quiet".into(),
            "sleep".into(),
            "2".into(),
        ];
        assert_eq!(
            parse_args(args),
            ParseResult::Queue("sleep".into(), vec!("2".into()), true, true)
        );

        args = vec!["fnq".into(), "sleep".into()];
        assert_eq!(
            parse_args(args),
            ParseResult::Queue("sleep".into(), vec!(), false, false,)
        );

        args = vec!["fnq".into(), "--tap".into()];
        assert_eq!(parse_args(args), ParseResult::TapAll);

        args = vec![
            "fnq".into(),
            "--tap".into(),
            ffi::OsString::from("queue_file.pid"),
        ];
        assert_eq!(
            parse_args(args),
            ParseResult::TapSingle("queue_file.pid".into())
        );

        args = vec!["fnq".into(), "--wait".into()];
        assert_eq!(parse_args(args), ParseResult::WaitAll);

        args = vec!["fnq".into(), "--wait".into(), "queue_file.pid".into()];
        assert_eq!(
            parse_args(args),
            ParseResult::WaitSingle("queue_file.pid".into())
        );
    }
}
