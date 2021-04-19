use std::ffi;

#[derive(Debug, PartialEq)]
pub enum ParseResult {
    Error,
    Tap(Option<ffi::OsString>),
    Wait(Option<ffi::OsString>),
    Queue(ffi::OsString, ffi::OsString, Vec<ffi::OsString>, bool, bool),
    Watch,
    Help,
    Version,
}

pub fn parse_args(mut args: Vec<ffi::OsString>) -> ParseResult {
    let len = args.len();
    if len < 2 {
        return ParseResult::Error;
    }

    let arg = &args[1];
    if arg == "--help" || arg == "-h" {
        return ParseResult::Help;
    } else if arg == "--version" || arg == "-v" {
        return ParseResult::Version;
    } else if arg == "--watch" {
        return ParseResult::Watch
    } else if arg == "--tap" || arg == "-t" {
        return if len == 2 {
            ParseResult::Tap(None)
        } else if len == 3 {
            ParseResult::Tap(args.drain(2..3).next())
        } else {
            ParseResult::Error
        };
    } else if arg == "--wait" || arg == "-w" {
        return if len == 2 {
            ParseResult::Wait(None)
        } else if len == 3 {
            ParseResult::Wait(args.drain(2..3).next())
        } else {
            ParseResult::Error
        };
    }

    let mut index: usize = 1;
    let mut quiet = false;
    let mut clean = false;

    for arg in &args[1..] {
        if arg == "--quiet" || arg == "-q" {
            quiet = true;
            index += 1;
        } else if arg == "--clean" || arg == "-c" {
            clean = true;
            index += 1;
        } else {
            break;
        }
    }

    if index < len {
        let task_cmd = args.drain(index..index + 1).next().unwrap();
        let task_args = args.drain(index..).collect();
        let fnq_cmd = args.drain(0..1).next().unwrap();
        return ParseResult::Queue(fnq_cmd, task_cmd, task_args, quiet, clean);
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
        assert_eq!(parse_args(args), ParseResult::Tap(None));

        args = vec!["fnq".into(), "-t".into()];
        assert_eq!(parse_args(args), ParseResult::Tap(None));

        args = vec!["fnq".into(), "--quiet".into()];
        assert_eq!(parse_args(args), ParseResult::Error);

        args = vec!["fnq".into(), "-q".into()];
        assert_eq!(parse_args(args), ParseResult::Error);

        args = vec!["fnq".into(), "--clean".into()];
        assert_eq!(parse_args(args), ParseResult::Error);

        args = vec!["fnq".into(), "-c".into()];
        assert_eq!(parse_args(args), ParseResult::Error);

        args = vec!["fnq".into(), "--quiet".into(), "sleep".into(), "2".into()];
        assert_eq!(
            parse_args(args),
            ParseResult::Queue("fnq".into(), "sleep".into(), vec!("2".into()), true, false)
        );
        args = vec!["fnq".into(), "-q".into(), "sleep".into(), "2".into()];
        assert_eq!(
            parse_args(args),
            ParseResult::Queue("fnq".into(), "sleep".into(), vec!("2".into()), true, false)
        );

        args = vec!["fnq".into(), "--clean".into(), "sleep".into(), "2".into()];
        assert_eq!(
            parse_args(args),
            ParseResult::Queue("fnq".into(), "sleep".into(), vec!("2".into()), false, true)
        );

        args = vec!["fnq".into(), "-c".into(), "sleep".into(), "2".into()];
        assert_eq!(
            parse_args(args),
            ParseResult::Queue("fnq".into(), "sleep".into(), vec!("2".into()), false, true)
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
            ParseResult::Queue("fnq".into(), "sleep".into(), vec!("2".into()), true, true)
        );

        args = vec![
            "fnq".into(),
            "-c".into(),
            "-q".into(),
            "sleep".into(),
            "2".into(),
        ];
        assert_eq!(
            parse_args(args),
            ParseResult::Queue("fnq".into(), "sleep".into(), vec!("2".into()), true, true)
        );

        args = vec!["fnq".into(), "sleep".into()];
        assert_eq!(
            parse_args(args),
            ParseResult::Queue("fnq".into(), "sleep".into(), vec!(), false, false)
        );

        args = vec!["fnq".into(), "--tap".into()];
        assert_eq!(parse_args(args), ParseResult::Tap(None));

        args = vec![
            "fnq".into(),
            "--tap".into(),
            ffi::OsString::from("queue_file.pid"),
        ];
        assert_eq!(
            parse_args(args),
            ParseResult::Tap(Some("queue_file.pid".into()))
        );

        assert_eq!(parse_args(vec!["fnq".into(), "--version".into()]), ParseResult::Version);
        assert_eq!(parse_args(vec!["fnq".into(), "-v".into()]), ParseResult::Version);
        assert_eq!(parse_args(vec!["fnq".into(), "-v".into(), "somethingelse".into()]), ParseResult::Version);

        assert_eq!(parse_args(vec!["fnq".into(), "--help".into()]), ParseResult::Help);
        assert_eq!(parse_args(vec!["fnq".into(), "-h".into()]), ParseResult::Help);
        assert_eq!(parse_args(vec!["fnq".into(), "-h".into(), "somethingelse".into()]), ParseResult::Help);

        assert_eq!(parse_args(vec!["fnq".into(), "--watch".into()]), ParseResult::Watch);
    }
}
