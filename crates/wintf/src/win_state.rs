#![allow(unused_variables)]

use ambassador::*;
use windows::Win32::Foundation::*;

#[derive(Debug, Default, Clone, Copy)]
pub struct DpiValue(f32);

#[derive(Debug, Default, Clone, Copy)]
pub struct Dpi {
    pub x: DpiValue,
    pub y: DpiValue,
}

impl Dpi {
    pub fn new(dpi: (f32, f32)) -> Self {
        Self {
            x: DpiValue(dpi.0),
            y: DpiValue(dpi.1),
        }
    }

    pub fn from_dpi_change_message(wparam: WPARAM, _lparam: LPARAM) -> Self {
        let dpi = (wparam.0 as u16 as f32, (wparam.0 >> 16) as f32);
        Self::new(dpi)
    }
}

impl DpiValue {
    pub fn get(&self) -> f32 {
        self.0
    }

    pub fn physical_to_logical(&self, pixel: f32) -> f32 {
        pixel * 96.0 / self.get()
    }

    pub fn logical_to_physical(&self, pixel: f32) -> f32 {
        pixel * self.get() / 96.0
    }
}

#[delegatable_trait]
pub trait WinState {
    fn hwnd(&self) -> HWND;

    fn set_hwnd(&mut self, hwnd: HWND);

    fn mouse_tracking(&self) -> bool {
        true
    }

    fn set_mouse_tracking(&mut self, tracking: bool) {}

    fn dpi(&self) -> Dpi;

    fn set_dpi(&mut self, x: f32, y: f32);

    fn set_dpi_change_message(&mut self, wparam: WPARAM, _lparam: LPARAM) {
        let dpi = Dpi::from_dpi_change_message(wparam, _lparam);
        self.set_dpi(dpi.x.get(), dpi.y.get());
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

    fn set_dpi(&mut self, x: f32, y: f32) {
        self.dpi = Dpi::new((x, y));
    }
}
