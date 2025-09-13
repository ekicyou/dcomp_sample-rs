#![allow(non_snake_case)]
#![allow(unused_variables)]

use crate::dpi::*;
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

    fn dpi(&self) -> Dpi;

    fn set_dpi(&mut self, dpi: Dpi);

    fn set_dpi_change_message(&mut self, wparam: WPARAM, lparam: LPARAM) {
        let dpi = Dpi::from_WM_DPICHANGED(wparam, lparam);
        self.set_dpi(dpi);
    }
}

#[derive(Debug, Default)]
pub struct SimpleWinState {
    hwnd: HWND,
    mouse_tracking: bool,
    dpi: Dpi,
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

    fn dpi(&self) -> Dpi {
        self.dpi
    }

    fn set_dpi(&mut self, dpi: Dpi) {
        self.dpi = dpi;
    }
}
