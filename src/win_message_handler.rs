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
            _ => unsafe { DefWindowProcA(self.hwnd(), message, wparam, lparam) },
        }
    }

    // デフォルト実装（winitスタイル）
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

#[inline]
pub fn box_handler<T: WindowMessageHandler + 'static>(
    handler: T,
) -> Box<Box<dyn WindowMessageHandler>> {
    Box::new(Box::new(handler))
}

pub extern "system" fn wndproc(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        if message == WM_NCCREATE {
            let cs = lparam.0 as *const CREATESTRUCTA;
            let boxed_handler = (*cs).lpCreateParams as *mut Box<dyn WindowMessageHandler>;

            if !boxed_handler.is_null() {
                (**boxed_handler).set_hwnd(window);
                SetWindowLongPtrA(window, GWLP_USERDATA, boxed_handler as _);
            }
        } else {
            let boxed_handler =
                GetWindowLongPtrA(window, GWLP_USERDATA) as *mut Box<dyn WindowMessageHandler>;
            if !boxed_handler.is_null() {
                return (**boxed_handler).message_handler(message, wparam, lparam);
            }
        }

        DefWindowProcA(window, message, wparam, lparam)
    }
}
