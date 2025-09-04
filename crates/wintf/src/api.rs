#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(dead_code)]

use windows::core::*;
use windows::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

/// GetWindowLongPtrWのラッパー
#[inline(always)]
pub(crate) fn get_window_long_ptr(hwnd: HWND, index: WINDOW_LONG_PTR_INDEX) -> Result<isize> {
    unsafe {
        SetLastError(ERROR_SUCCESS);
        let res = GetWindowLongPtrW(hwnd, index);
        let err = Error::from_win32();
        if err.code() != S_OK {
            return Err(err);
        }
        Ok(res)
    }
}

/// SetWindowLongPtrWのラッパー
#[inline(always)]
pub(crate) fn set_window_long_ptr(
    hwnd: HWND,
    index: WINDOW_LONG_PTR_INDEX,
    value: isize,
) -> Result<isize> {
    unsafe {
        SetLastError(ERROR_SUCCESS);
        let res = SetWindowLongPtrW(hwnd, index, value);
        if res == 0 {
            let err = Error::from_win32();
            if err.code() != S_OK {
                return Err(err);
            }
        }
        Ok(res)
    }
}
