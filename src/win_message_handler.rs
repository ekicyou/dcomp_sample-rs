#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(dead_code)]

use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub trait WindowMessageHandler: Sized {
    fn hwnd(&self) -> HWND;

    fn set_hwnd(&mut self, hwnd: HWND);

    extern "system" fn wndproc(
        window: HWND,
        message: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        unsafe {
            if message == WM_NCCREATE {
                let cs = lparam.0 as *const CREATESTRUCTA;
                let this = (*cs).lpCreateParams as *mut Self;
                (*this).set_hwnd(window);

                SetWindowLongPtrA(window, GWLP_USERDATA, this as _);
            } else {
                let this = GetWindowLongPtrA(window, GWLP_USERDATA) as *mut Self;
                if !this.is_null() {
                    return (*this).message_handler(message, wparam, lparam);
                }
            }

            DefWindowProcA(window, message, wparam, lparam)
        }
    }

    #[inline(always)]
    fn message_handler(&mut self, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        match message {
            WM_LBUTTONUP => self.WM_LBUTTONUP(wparam, lparam),
            WM_PAINT => self.WM_PAINT(wparam, lparam),
            WM_DPICHANGED => self.WM_DPICHANGED(wparam, lparam),
            WM_CREATE => self.WM_CREATE(wparam, lparam),
            WM_WINDOWPOSCHANGING => self.WM_WINDOWPOSCHANGING(wparam, lparam),
            WM_DESTROY => self.WM_DESTROY(wparam, lparam),
            _ => return unsafe { DefWindowProcA(self.hwnd(), message, wparam, lparam) },
        }
    }

    #[inline]
    fn WM_CREATE(&mut self, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        LRESULT(0)
    }

    #[inline]
    fn WM_DESTROY(&mut self, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        LRESULT(0)
    }

    #[inline]
    fn WM_LBUTTONUP(&mut self, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        LRESULT(0)
    }

    #[inline]
    fn WM_PAINT(&mut self, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        LRESULT(0)
    }

    #[inline]
    fn WM_DPICHANGED(&mut self, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        LRESULT(0)
    }

    #[inline]
    fn WM_WINDOWPOSCHANGING(&mut self, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        LRESULT(0)
    }
}
