use std::ffi;
use std::os::unix::prelude::*;

#[macro_export]
macro_rules! concat_os_strings {
    ($($elem: expr),*) => {{
        let mut os_string = ffi::OsString::new();
        $(os_string.push($elem);)*
        os_string
    }};
}

pub fn os_string_starts_with(os_a: &ffi::OsStr, os_b: &ffi::OsStr) -> bool {
    let bytes_a = os_a.as_bytes();
    let bytes_b = os_b.as_bytes();
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
