#![allow(unused_variables)]

use ambassador::*;
use windows::Win32::Foundation::*;

#[delegatable_trait]
pub trait WinState {
    fn hwnd(&self) -> HWND;

    fn set_hwnd(&mut self, hwnd: HWND);

    fn mouse_tracking(&self) -> bool {
        true
    }

    fn set_mouse_tracking(&mut self, tracking: bool) {}
}

#[derive(Debug, Default)]
pub struct SimpleWinState {
    hwnd: HWND,
    mouse_tracking: bool,
}

impl WinState for SimpleWinState {
    fn hwnd(&self) -> HWND {
        self.hwnd
    }

    fn set_hwnd(&mut self, hwnd: HWND) {
        self.hwnd = hwnd;
    }

    fn mouse_tracking(&self) -> bool {
        self.mouse_tracking
    }

    fn set_mouse_tracking(&mut self, tracking: bool) {
        self.mouse_tracking = tracking;
    }
}
