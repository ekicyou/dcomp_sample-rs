#![allow(dead_code)]
use winapi::shared::windef::HWND;

pub trait HwndWindow {
    fn hwnd(&self) -> HWND;
}

pub struct HWndProxy {
    hwnd: HWND,
}

impl HWndProxy {
    pub fn new(hwnd: HWND) -> HWndProxy {
        HWndProxy { hwnd: hwnd }
    }
}

impl HwndWindow for HWndProxy {
    fn hwnd(&self) -> HWND {
        self.hwnd
    }
}
