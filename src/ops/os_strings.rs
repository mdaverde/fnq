use std::ffi;
use std::os::unix::prelude::*;

// TODO: should count amount of $elem and use with capacity
#[macro_export]
macro_rules! concat_os_strings {
    ($($elem: expr),*) => {{
        let mut os_string = ffi::OsString::new();
        $(os_string.push($elem);)*
        os_string
    }};
}

pub trait OsStringStartsWithExt {
    fn starts_with(&self, rhs: &ffi::OsStr) -> bool;
}

impl OsStringStartsWithExt for &ffi::OsStr {
    fn starts_with(&self, rhs: &ffi::OsStr) -> bool {
        let bytes_a = self.as_bytes();
        let bytes_b = rhs.as_bytes();
        if bytes_b.len() > bytes_a.len() {
            return false;
        }
        for (i, b) in bytes_a[..bytes_b.len()].iter().enumerate() {
            if !bytes_b[i].eq(b) {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_concat_macro() {
        let hello: ffi::OsString = "hello".into();
        let world: ffi::OsString = "world".into();

        assert_eq!(
            concat_os_strings!(hello, world),
            ffi::OsString::from("helloworld")
        )
    }

    #[test]
    fn test_starts_with() {
        let foobar: ffi::OsString = "foobar".into();
        assert!(foobar
            .as_os_str()
            .starts_with(ffi::OsString::from("foo").as_os_str()));
    }
}
