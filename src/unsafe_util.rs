#![allow(unused_unsafe)]
use winapi::shared::minwindef::BOOL;
use winapi::shared::minwindef::FALSE;

#[allow(non_snake_case)]
#[inline]
fn BOOL(flag: bool) -> BOOL {
    match flag {
        false => FALSE,
        true => TRUE,
    }
}


#[inline]
fn slice_to_ptr<T>(s: &[T]) -> (UINT, *const T) {
    let len = s.len() as UINT;
    let p: *const T = match len {
        0 => ptr::null(),
        _ => &s[0],
    };
    (len, p)
}

#[inline]
fn opt_to_ptr<T>(src: Option<&T>) -> *const T {
    match src {
        Some(a) => a,
        None => ptr::null(),
    }
}

#[inline]
fn to_utf16_chars<'a, S: Into<&'a str>>(s: S) -> Vec<u16> {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    OsStr::new(s.into())
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect::<Vec<_>>()
}

#[inline]
fn to_utf8_chars<'a, S: Into<&'a str>>(s: S) -> Vec<u8> {
    let bytes = s.into().as_bytes();
    let iter = bytes.into_iter();
    iter.chain(Some(0_u8).into_iter()).collect::<Vec<_>>()
}
