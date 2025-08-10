#![allow(non_snake_case)]
#![allow(unused_variables)]
#![allow(dead_code)]

use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

pub trait WindowMessageHandler {
    fn hwnd(&self) -> HWND;

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
    fn WM_CREATE(&mut self, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        LRESULT(0)
    }

    #[inline]
    fn WM_WINDOWPOSCHANGING(&mut self, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        LRESULT(0)
    }

    #[inline]
    fn WM_DESTROY(&mut self, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        LRESULT(0)
    }
}
