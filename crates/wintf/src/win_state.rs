#![allow(unused_variables)]

use windows::Win32::Foundation::HWND;

pub trait WinState {
    fn hwnd(&self) -> HWND;

    fn set_hwnd(&mut self, hwnd: HWND);

    fn mouse_tracking(&self) -> bool {
        true
    }

    fn set_mouse_tracking(&mut self, tracking: bool) {}
}
