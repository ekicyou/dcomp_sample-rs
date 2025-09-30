pub mod command;
pub use command::*;

use windows::core::*;
use windows::Win32::Graphics::Direct2D::Common::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::Graphics::Imaging::*;
use windows_numerics::*;

/// D2D1CreateDevice
pub fn d2d_create_device(dxgi: &IDXGIDevice4) -> Result<ID2D1Device> {
    unsafe { D2D1CreateDevice(dxgi, None) }
}

pub trait D2D1DeviceExt {
    fn create_device_context(
        &self,
        options: D2D1_DEVICE_CONTEXT_OPTIONS,
    ) -> Result<ID2D1DeviceContext>;
}

impl D2D1DeviceExt for ID2D1Device {
    #[inline(always)]
    fn create_device_context(
        &self,
        options: D2D1_DEVICE_CONTEXT_OPTIONS,
    ) -> Result<ID2D1DeviceContext> {
        unsafe { self.CreateDeviceContext(options) }
    }
}

pub trait D2D1DeviceContextExt {
    /// CreateBitmapFromWicBitmap
    fn create_bitmap_from_wic_bitmap<P0>(&self, wicbitmapsource: P0) -> Result<ID2D1Bitmap1>
    where
        P0: Param<IWICBitmapSource>;
    /// SetTransform
    fn set_transform(&self, transform: &Matrix3x2);
    /// Clear
    fn clear(&self, color: Option<*const D2D1_COLOR_F>);
}

impl D2D1DeviceContextExt for ID2D1DeviceContext {
    #[inline(always)]
    fn create_bitmap_from_wic_bitmap<P0>(&self, wicbitmapsource: P0) -> Result<ID2D1Bitmap1>
    where
        P0: Param<IWICBitmapSource>,
    {
        unsafe { self.CreateBitmapFromWicBitmap(wicbitmapsource, None) }
    }
    #[inline(always)]
    fn set_transform(&self, transform: &Matrix3x2) {
        unsafe { self.SetTransform(transform) }
    }
    #[inline(always)]
    fn clear(&self, color: Option<*const D2D1_COLOR_F>) {
        let color_ptr = color.map(|c| c as *const D2D1_COLOR_F);
        unsafe { self.Clear(color_ptr) }
    }
}
