use std::ffi;

#[derive(Debug, PartialEq)]
pub enum ParseResult {
    Error,
    Tap(Option<ffi::OsString>),
    Block(Option<ffi::OsString>),
    Queue(ffi::OsString, ffi::OsString, Vec<ffi::OsString>, bool, bool),
    Watch,
    Last,
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
    } else if arg == "--watch" || arg == "-w" {
        return ParseResult::Watch;
    } else if arg == "--last" || arg == "-l" {
        return ParseResult::Last;
    } else if arg == "--tap" || arg == "-t" {
        return if len == 2 {
            ParseResult::Tap(None)
        } else if len == 3 {
            ParseResult::Tap(args.drain(2..3).next())
        } else {
            ParseResult::Error
        };
    } else if arg == "--block" || arg == "-b" {
        return if len == 2 {
            ParseResult::Block(None)
        } else if len == 3 {
            ParseResult::Block(args.drain(2..3).next())
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

    macro_rules! vec_into {
        ($($x:expr),+ $(,)?) => (
            vec![$($x.into()),+]
        );
    }

    #[test]
    fn test_parse_args() {
        let mut args: Vec<ffi::OsString> = vec![];
        assert_eq!(parse_args(args), ParseResult::Error);

        args = vec_into!["fnq"];
        assert_eq!(parse_args(args), ParseResult::Error);

        args = vec_into!["fnq", "--tap"];
        assert_eq!(parse_args(args), ParseResult::Tap(None));

        args = vec_into!["fnq", "-t"];
        assert_eq!(parse_args(args), ParseResult::Tap(None));

        args = vec_into!["fnq", "--quiet"];
        assert_eq!(parse_args(args), ParseResult::Error);

        args = vec_into!["fnq", "-q"];
        assert_eq!(parse_args(args), ParseResult::Error);

        args = vec_into!["fnq", "--clean"];
        assert_eq!(parse_args(args), ParseResult::Error);

        args = vec_into!["fnq", "-c"];
        assert_eq!(parse_args(args), ParseResult::Error);

        args = vec_into!["fnq", "--block"];
        assert_eq!(parse_args(args), ParseResult::Block(None));

        args = vec_into!["fnq", "-b"];
        assert_eq!(parse_args(args), ParseResult::Block(None));

        args = vec_into!["fnq", "--block", "queue_file.pid"];
        assert_eq!(
            parse_args(args),
            ParseResult::Block(Some("queue_file.pid".into()))
        );

        args = vec_into!["fnq", "-b", "queue_file.pid"];
        assert_eq!(
            parse_args(args),
            ParseResult::Block(Some("queue_file.pid".into()))
        );

        args = vec_into!["fnq", "--quiet", "sleep", "2"];
        assert_eq!(
            parse_args(args),
            ParseResult::Queue("fnq".into(), "sleep".into(), vec_into!["2"], true, false)
        );
        args = vec_into!["fnq", "-q", "sleep", "2"];
        assert_eq!(
            parse_args(args),
            ParseResult::Queue("fnq".into(), "sleep".into(), vec_into!["2"], true, false)
        );

        args = vec_into!["fnq", "--clean", "sleep", "2"];
        assert_eq!(
            parse_args(args),
            ParseResult::Queue("fnq".into(), "sleep".into(), vec_into!["2"], false, true)
        );

        args = vec_into!["fnq", "-c", "sleep", "2"];
        assert_eq!(
            parse_args(args),
            ParseResult::Queue("fnq".into(), "sleep".into(), vec_into!["2"], false, true)
        );

        args = vec_into![
            "fnq",
            "--clean",
            "--quiet",
            "sleep",
            "2",
        ];
        assert_eq!(
            parse_args(args),
            ParseResult::Queue("fnq".into(), "sleep".into(), vec_into!["2"], true, true)
        );

        args = vec_into![
            "fnq",
            "-c",
            "-q",
            "sleep",
            "2",
        ];
        assert_eq!(
            parse_args(args),
            ParseResult::Queue("fnq".into(), "sleep".into(), vec_into!["2"], true, true)
        );

        args = vec_into!["fnq", "sleep"];
        assert_eq!(
            parse_args(args),
            ParseResult::Queue("fnq".into(), "sleep".into(), vec!(), false, false)
        );

        args = vec_into!["fnq", "--tap"];
        assert_eq!(parse_args(args), ParseResult::Tap(None));

        args = vec_into![
            "fnq",
            "--tap",
            ffi::OsString::from("queue_file.pid"),
        ];
        assert_eq!(
            parse_args(args),
            ParseResult::Tap(Some("queue_file.pid".into()))
        );

        assert_eq!(
            parse_args(vec_into!["fnq", "--version"]),
            ParseResult::Version
        );
        assert_eq!(
            parse_args(vec_into!["fnq", "-v"]),
            ParseResult::Version
        );
        assert_eq!(
            parse_args(vec_into!["fnq", "-v", "somethingelse"]),
            ParseResult::Version
        );

        assert_eq!(
            parse_args(vec_into!["fnq", "--help"]),
            ParseResult::Help
        );
        assert_eq!(
            parse_args(vec_into!["fnq", "-h"]),
            ParseResult::Help
        );
        assert_eq!(
            parse_args(vec_into!["fnq", "-h", "somethingelse"]),
            ParseResult::Help
        );

        assert_eq!(
            parse_args(vec_into!["fnq", "--watch"]),
            ParseResult::Watch
        );
        assert_eq!(
            parse_args(vec_into!["fnq", "-w"]),
            ParseResult::Watch
        );

        assert_eq!(
            parse_args(vec_into!["fnq", "--last"]),
            ParseResult::Last
        );
        assert_eq!(
            parse_args(vec_into!["fnq", "-l"]),
            ParseResult::Last
        );
    }
}
