use winapi::Interface;
use winapi::um::dcomp::*;
use winapi::shared::ntdef::HANDLE;
use winapi::_core as core;
use winapi::_core::ptr;
use winapi::ctypes::c_void;
use winapi::shared::winerror::{HRESULT, S_OK};
use winapi::um::dcomp;
use winapi::shared::dxgi::IDXGIDevice;

use super::com_rc::*;

use winapi::um::unknwnbase::IUnknown;


fn dcomp_create_device<U: Interface>(dxgiDevice: Option<&IUnknown>) -> Result<ComRc<U>, HRESULT> {
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


pub trait DCompWindow {
    fn handle(&self) -> HANDLE;
    fn create_dev(&self) -> Result<(), HRESULT> {
        let hwnd = self.handle();
        let dc_dev = dcomp_create_device::<IDCompositionDevice>(None)?;
        Ok(())
    }
}

pub struct HWndProxy {
    handle: HANDLE,
}

impl HWndProxy {
    pub fn new(handle: HANDLE) -> HWndProxy {
        HWndProxy { handle: handle }
    }
}

impl DCompWindow for HWndProxy {
    fn handle(&self) -> HANDLE {
        self.handle
    }
}
