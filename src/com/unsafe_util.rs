#![allow(unused_unsafe)]
use libc;
use raw_window_handle::*;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use winapi::_core::ptr;
use winapi::shared::minwindef::{BOOL, FALSE, TRUE, UINT};
use winapi::shared::windef::HWND;

#[allow(non_snake_case)]
#[inline]
pub fn BOOL(flag: bool) -> BOOL {
    match flag {
        false => FALSE,
        true => TRUE,
    }
}

#[inline]
pub fn slice_to_ptr<T>(s: &[T]) -> (UINT, *const T) {
    let len = s.len() as UINT;
    let p: *const T = match len {
        0 => ptr::null(),
        _ => &s[0],
    };
    (len, p)
}

#[inline]
pub fn opt_to_ptr<T>(src: Option<&T>) -> *const T {
    match src {
        Some(a) => a,
        None => ptr::null(),
    }
}

#[inline]
pub unsafe fn opt_to_ptr_mut<T>(src: Option<&T>) -> *mut T {
    opt_to_ptr(src) as *mut _
}

#[inline]
pub fn to_utf16_chars<'a, S: Into<&'a str>>(s: S) -> Vec<u16> {
    let s = s.into();
    println!("to_utf16_chars({})", s);
    let v = OsStr::new(s)
        .encode_wide()
        .chain(Some(0).into_iter())
        .collect::<Vec<_>>();
    println!(" --> {:?}", v);
    v
}

#[inline]
pub fn to_utf8_chars<'a, S: Into<&'a str>>(s: S) -> Vec<u8> {
    let s = s.into();
    println!("to_utf8_chars({})", s);
    let mut v = s.as_bytes().to_vec();
    v.push(0);
    println!(" --> {:?}", v);
    v
}

#[inline]
pub unsafe fn memcpy(dst: *mut u8, src: *const u8, size: usize) -> *mut u8 {
    let dst = dst as *mut libc::c_void;
    let src = src as *const libc::c_void;
    libc::memcpy(dst, src, size) as *mut u8
}

#[inline]
pub unsafe fn HWND(handle: RawWindowHandle) -> HWND {
    match handle {
        RawWindowHandle::Windows(h) => h.hwnd as HWND,
        _ => ptr::null_mut() as HWND,
    }
}
