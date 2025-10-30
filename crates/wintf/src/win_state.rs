#![allow(non_snake_case)]
#![allow(unused_variables)]

use crate::api::get_window_long_ptr;
use crate::dpi::*;
use ambassador::*;
use windows::core::*;
use windows::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

#[delegatable_trait]
pub trait WinState {
    /// Get the HWND value.
    fn hwnd(&self) -> HWND;

    /// Set the HWND value.
    fn set_hwnd(&mut self, hwnd: HWND);

    /// Get whether mouse tracking is enabled.
    fn mouse_tracking(&self) -> bool {
        true
    }

    /// Set whether mouse tracking is enabled.
    fn set_mouse_tracking(&mut self, tracking: bool) {}

    /// Get the stored DPI value.
    fn dpi(&self) -> Dpi;

    /// Set the stored DPI value.
    fn set_dpi(&mut self, dpi: Dpi);

    /// Handle a WM_DPICHANGED message by extracting the new DPI and updating the stored DPI value.
    fn set_dpi_change_message(&mut self, wparam: WPARAM, lparam: LPARAM) {
        let dpi = Dpi::from_WM_DPICHANGED(wparam, lparam);
        self.set_dpi(dpi);
    }

    /// Calculate the effective window size (including borders, title bar, etc) for a given client area size.
    /// This is useful when creating a window with a specific client area size.
    fn effective_window_size(&self, client_size: DipSize) -> Result<PpxSize> {
        let dpi = self.dpi();
        let client_size: PpxSize = client_size.into_dpi(dpi);
        let mut rect = RECT {
            left: 0,
            top: 0,
            right: client_size.width.ceil() as i32,
            bottom: client_size.height.ceil() as i32,
        };
        let hwnd = self.hwnd();
        let style = WINDOW_STYLE(get_window_long_ptr(hwnd, GWL_STYLE)? as u32);
        let ex_style = WINDOW_EX_STYLE(get_window_long_ptr(hwnd, GWL_EXSTYLE)? as u32);
        unsafe { AdjustWindowRectEx(&mut rect, style, false, ex_style)? }
        Ok(PpxSize::new(
            (rect.right - rect.left) as f32,
            (rect.bottom - rect.top) as f32,
        ))
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
