use windows::core::*;
use windows::Win32::Graphics::DirectComposition::*;

pub fn dcomp_create_desktop_device<P0>(renderingdevice: P0) -> Result<IDCompositionDesktopDevice>
where
    P0: Param<IUnknown>,
{
    unsafe { DCompositionCreateDevice3(renderingdevice) }
}
