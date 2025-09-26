use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::DirectComposition::*;

pub fn dcomp_create_desktop_device<P0>(renderingdevice: P0) -> Result<IDCompositionDesktopDevice>
where
    P0: Param<IUnknown>,
{
    unsafe { DCompositionCreateDevice3(renderingdevice) }
}

pub trait DCompositionDesktopDeviceExt {
    /// CreateTargetForHwnd
    fn create_target_for_hwnd(&self, hwnd: HWND, topmost: bool) -> Result<IDCompositionTarget>;
}

impl DCompositionDesktopDeviceExt for IDCompositionDesktopDevice {
    #[inline(always)]
    fn create_target_for_hwnd(&self, hwnd: HWND, topmost: bool) -> Result<IDCompositionTarget> {
        unsafe { self.CreateTargetForHwnd(hwnd, topmost) }
    }
}

pub trait IDCompositionDeviceExt {
    fn create_visual(&self) -> Result<IDCompositionVisual3>;
}

impl IDCompositionDeviceExt for IDCompositionDevice3 {
    #[inline(always)]
    fn create_visual(&self) -> Result<IDCompositionVisual3> {
        unsafe { self.CreateVisual()?.cast() }
    }
}
