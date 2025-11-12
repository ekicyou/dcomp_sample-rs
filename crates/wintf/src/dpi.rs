#![allow(non_snake_case)]

use bevy_ecs::prelude::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct2D::*;

fn dpi_to_scale_factor(dpi: f32) -> f32 {
    dpi / 96.0
}



#[derive(Component, Debug, Default, Clone, Copy)]
pub struct Dpi {
    dpi: f32,
    scale_factor: f32,
}

impl Dpi {
    pub fn new(dpi: f32) -> Self {
        Self {
            dpi,
            scale_factor: dpi_to_scale_factor(dpi),
        }
    }

    pub fn from_WM_DPICHANGED(wparam: WPARAM, _lparam: LPARAM) -> Self {
        let (x_dpi, _) = (wparam.0 as u16 as f32, (wparam.0 >> 16) as f32);
        Self::new(x_dpi)
    }

    #[inline]
    pub fn value(&self) -> f32 {
        self.dpi
    }

    #[inline]
    pub fn scale_factor(&self) -> f32 {
        self.scale_factor
    }
}



impl Dpi {
    pub fn set_render_target_dpi(&self, target: &ID2D1RenderTarget) {
        unsafe {
            target.SetDpi(self.dpi, self.dpi);
        }
    }
}




