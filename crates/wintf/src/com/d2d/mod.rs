pub mod command;
pub use command::*;

use windows::core::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::Dxgi::*;

/// D2D1CreateDevice
pub fn d2d_create_device(dxgi: &IDXGIDevice4) -> Result<ID2D1Device> {
    unsafe { D2D1CreateDevice(dxgi, None) }
}
