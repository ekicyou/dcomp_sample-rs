#![allow(non_snake_case)]

use winapi::Interface;
use winapi::shared::windef::HWND;
use winapi::_core as core;
use winapi::_core::ptr;
use winapi::ctypes::c_void;
use winapi::shared::winerror::HRESULT;
use winapi::um::dcomp;
use winapi::shared::minwindef::{BOOL, TRUE, FALSE};
use winapi::um::unknwnbase::IUnknown;

pub use winapi::um::dcomp::*;
pub use com_rc::*;

#[inline]
fn BOOL(flag: bool) -> BOOL {
    match flag {
        false => FALSE,
        true => TRUE,
    }
}

pub fn create_device<U: Interface>(dxgiDevice: Option<&IUnknown>) -> Result<ComRc<U>, HRESULT> {
    let src: *const IUnknown = match dxgiDevice {
        Some(a) => a,
        None => ptr::null(),
    };
    let riid = U::uuidof();
    let p = unsafe {
        let mut ppv: *mut c_void = core::ptr::null_mut();
        dcomp::DCompositionCreateDevice3(src, &riid, &mut ppv).hr()?;
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