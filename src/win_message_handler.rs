#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(dead_code)]

use std::ffi::c_void;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

// Object-safe trait（winitスタイル）
pub trait WindowMessageHandler {
    fn hwnd(&self) -> HWND;
    fn set_hwnd(&mut self, hwnd: HWND);

    #[inline(always)]
    fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match message {
            WM_LBUTTONUP => self.WM_LBUTTONUP(wparam, lparam),
            WM_PAINT => self.WM_PAINT(wparam, lparam),
            WM_DPICHANGED => self.WM_DPICHANGED(wparam, lparam),
            WM_CREATE => self.WM_CREATE(wparam, lparam),
            WM_WINDOWPOSCHANGING => self.WM_WINDOWPOSCHANGING(wparam, lparam),
            WM_DESTROY => self.WM_DESTROY(wparam, lparam),
            _ => unsafe { DefWindowProcW(self.hwnd(), message, wparam, lparam) }, // A→W
        }
    }

    // デフォルト実装
    fn WM_CREATE(&mut self, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        LRESULT(0)
    }
    fn WM_DESTROY(&mut self, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        LRESULT(0)
    }
    fn WM_LBUTTONUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        LRESULT(0)
    }
    fn WM_PAINT(&mut self, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        LRESULT(0)
    }
    fn WM_DPICHANGED(&mut self, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        LRESULT(0)
    }
    fn WM_WINDOWPOSCHANGING(&mut self, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        LRESULT(0)
    }
}

pub(crate) trait WindowMessageHandlerExt: WindowMessageHandler {
    fn into_raw(self) -> *mut c_void
    where
        Self: Sized,
    {
        let b1: Box<dyn WindowMessageHandler> = Box::new(self);
        let b2: Box<Box<dyn WindowMessageHandler>> = Box::new(b1);
        let ptr = Box::into_raw(b2) as *mut c_void;
        ptr
    }
}

impl<T: WindowMessageHandler + Sized> WindowMessageHandlerExt for T {}

pub extern "system" fn wndproc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match message {
            WM_NCCREATE => {
                let cs = lparam.0 as *const CREATESTRUCTW;
                let ptr = (*cs).lpCreateParams;
                let boxed_handler = ptr as *mut Box<Box<dyn WindowMessageHandler>>;
                if !boxed_handler.is_null() {
                    (**boxed_handler).set_hwnd(hwnd);
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, boxed_handler as _);
                }
                LRESULT(0)
            }
            WM_NCDESTROY => {
                let boxed_handler = GetWindowLongPtrW(hwnd, GWLP_USERDATA)
                    as *mut Box<Box<dyn WindowMessageHandler>>;
                if !boxed_handler.is_null() {
                    let rc = (**boxed_handler).message_handler(message, wparam, lparam);
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                    let _ = Box::from_raw(boxed_handler);
                    rc
                } else {
                    DefWindowProcW(hwnd, message, wparam, lparam)
                }
            }
            _ => {
                let boxed_handler = GetWindowLongPtrW(hwnd, GWLP_USERDATA)
                    as *mut Box<Box<dyn WindowMessageHandler>>;
                if !boxed_handler.is_null() {
                    (**boxed_handler).message_handler(message, wparam, lparam)
                } else {
                    DefWindowProcW(hwnd, message, wparam, lparam)
                }
            }
        }
    }
}
