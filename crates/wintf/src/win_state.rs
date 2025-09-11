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

    fn dpi(&self) -> (f32, f32);

    fn set_dpi(&mut self, dpi: (f32, f32));

    fn set_dpi_from_message(&mut self, wparam: WPARAM, _lparam: LPARAM) {
        let dpi = (wparam.0 as u16 as f32, (wparam.0 >> 16) as f32);
        self.set_dpi(dpi);
    }
}

#[derive(Debug, Default)]
pub struct SimpleWinState {
    hwnd: HWND,
    mouse_tracking: bool,
    dpi: (f32, f32),
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

    fn dpi(&self) -> (f32, f32) {
        self.dpi
    }

    fn set_dpi(&mut self, dpi: (f32, f32)) {
        self.dpi = dpi;
    }
}
