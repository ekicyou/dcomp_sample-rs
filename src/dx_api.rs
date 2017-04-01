#![allow(non_snake_case)]

use winapi::Interface;
use winapi::shared::windef::HWND;
use winapi::_core as core;
use winapi::_core::ptr;
use winapi::ctypes::c_void;
use winapi::shared::winerror::HRESULT;
use winapi::shared::minwindef::{BOOL, TRUE, FALSE};
use winapi::um::unknwnbase::IUnknown;

pub use winapi::um::d3dcommon::*;
pub use winapi::shared::dxgi1_4::*;
pub use winapi::um::d3d12sdklayers::*;
pub use winapi::um::d3d12::*;
pub use winapi::um::dcomp::*;
pub use unsafe_api::*;
pub use com_rc::*;

#[inline]
fn BOOL(flag: bool) -> BOOL {
    match flag {
        false => FALSE,
        true => TRUE,
    }
}

pub fn d3d12_create_device<U: Interface>(pAdapter: &IUnknown,
                                         MinimumFeatureLevel: D3D_FEATURE_LEVEL)
                                         -> Result<ComRc<U>, HRESULT> {
    let riid = U::uuidof();
    let p = unsafe {
        let mut ppv: *mut c_void = core::ptr::null_mut();
        D3D12CreateDevice(pAdapter, MinimumFeatureLevel, &riid, &mut ppv).hr()?;
        ppv as *const U
    };
    Ok(ComRc::new(p))
}

pub fn d3d12_create_hardware_device(factory: &IDXGIFactory4)
                                    -> Result<ComRc<ID3D12Device>, HRESULT> {
    /*
		ComPtr<IDXGIAdapter1> hardwareAdapter;
		GetHardwareAdapter(factory.Get(), &hardwareAdapter);

		ThrowIfFailed(D3D12CreateDevice(
			hardwareAdapter.Get(),
			D3D_FEATURE_LEVEL_11_0,
			IID_PPV_ARGS(&m_device)
			));
*/
    unimplemented!()
}

pub fn d3d12_create_warp_device(factory: &IDXGIFactory4) -> Result<ComRc<ID3D12Device>, HRESULT> {
    /*
        // WARPデバイス(ソフトウェアレンダラ)を使う場合
		ComPtr<IDXGIAdapter> warpAdapter;
		ThrowIfFailed(factory->EnumWarpAdapter(IID_PPV_ARGS(&warpAdapter)));

		ThrowIfFailed(D3D12CreateDevice(
			warpAdapter.Get(),
			D3D_FEATURE_LEVEL_11_0,
			IID_PPV_ARGS(&m_device)
			));
*/
    unimplemented!()
}





pub fn create_dxgi_factory1<U: Interface>() -> Result<ComRc<U>, HRESULT> {
    let riid = U::uuidof();
    let p = unsafe {
        let mut ppv: *mut c_void = core::ptr::null_mut();
        CreateDXGIFactory1(&riid, &mut ppv).hr()?;
        ppv as *const U
    };
    Ok(ComRc::new(p))
}

pub fn d3d12_get_debug_interface<U: Interface>() -> Result<ComRc<U>, HRESULT> {
    let riid = U::uuidof();
    let p = unsafe {
        let mut ppv: *mut c_void = core::ptr::null_mut();
        D3D12GetDebugInterface(&riid, &mut ppv).hr()?;
        ppv as *const U
    };
    Ok(ComRc::new(p))
}

pub fn dcomp_create_device<U: Interface>(dxgiDevice: Option<&IUnknown>)
                                         -> Result<ComRc<U>, HRESULT> {
    let src: *const IUnknown = match dxgiDevice {
        Some(a) => a,
        None => ptr::null(),
    };
    let riid = U::uuidof();
    let p = unsafe {
        let mut ppv: *mut c_void = core::ptr::null_mut();
        DCompositionCreateDevice3(src, &riid, &mut ppv).hr()?;
        ppv as *const U
    };
    Ok(ComRc::new(p))
}

pub trait IDCompositionDeviceExt {
    fn create_target_for_hwnd(&self,
                              hwnd: HWND,
                              topmost: bool)
                              -> Result<ComRc<IDCompositionTarget>, HRESULT>;

    fn create_visual(&self) -> Result<ComRc<IDCompositionVisual>, HRESULT>;
}

impl IDCompositionDeviceExt for IDCompositionDevice {
    #[inline]
    fn create_target_for_hwnd(&self,
                              hwnd: HWND,
                              topmost: bool)
                              -> Result<ComRc<IDCompositionTarget>, HRESULT> {
        unsafe {
            let mut p: *mut IDCompositionTarget = ptr::null_mut();
            self.CreateTargetForHwnd(hwnd, BOOL(topmost), &mut p)
                .hr()?;
            Ok(ComRc::new(p))
        }
    }

    #[inline]
    fn create_visual(&self) -> Result<ComRc<IDCompositionVisual>, HRESULT> {
        unsafe {
            let mut p: *mut IDCompositionVisual = ptr::null_mut();
            self.CreateVisual(&mut p).hr()?;
            Ok(ComRc::new(p))
        }
    }
}