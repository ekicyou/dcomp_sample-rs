use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::Dxgi::*;

/// D3D11CreateDevice
pub fn d3d11_create_device<P0>(
    padapter: P0,
    drivertype: D3D_DRIVER_TYPE,
    software: HMODULE,
    flags: D3D11_CREATE_DEVICE_FLAG,
    pfeaturelevels: Option<&[D3D_FEATURE_LEVEL]>,
    sdkversion: u32,
    featurelevel: Option<&mut D3D_FEATURE_LEVEL>,
    immediatecontext: Option<&mut Option<ID3D11DeviceContext>>,
) -> Result<ID3D11Device>
where
    P0: Param<IDXGIAdapter>,
{
    let featurelevel = featurelevel.map(|v| v as *mut _);
    let immediatecontext = immediatecontext.map(|v| v as *mut _);
    let mut device = None;
    unsafe {
        D3D11CreateDevice(
            padapter,
            drivertype,
            software,
            flags,
            pfeaturelevels,
            sdkversion,
            Some(&mut device),
            featurelevel,
            immediatecontext,
        )
        .map(|()| device.unwrap())
    }
}
