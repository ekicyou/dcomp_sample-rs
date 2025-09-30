pub mod command;
pub use command::*;

use windows::core::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::Dxgi::*;

/// D2D1CreateDevice
pub fn d2d_create_device(dxgi: &IDXGIDevice4) -> Result<ID2D1Device> {
    unsafe { D2D1CreateDevice(dxgi, None) }
}

pub trait ID2D1DeviceExt {
    fn create_device_context(
        &self,
        options: D2D1_DEVICE_CONTEXT_OPTIONS,
    ) -> Result<ID2D1DeviceContext>;
}

impl ID2D1DeviceExt for ID2D1Device {
    #[inline(always)]
    fn create_device_context(
        &self,
        options: D2D1_DEVICE_CONTEXT_OPTIONS,
    ) -> Result<ID2D1DeviceContext> {
        unsafe { self.CreateDeviceContext(options) }
    }
}
