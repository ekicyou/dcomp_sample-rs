use winapi::Interface;
use winapi::um::dcomp::*;
use winapi::shared::ntdef::HANDLE;
use winapi::_core as core;
use winapi::_core::ptr;
use winapi::ctypes::c_void;
use winapi::shared::winerror::{HRESULT, S_OK};

use super::com_rc::*;

pub trait DCompWindow {
    fn handle(&self) -> HANDLE;
    fn create_dev(&self) -> Result<(), HRESULT> {
        let hwnd = self.handle();
        unsafe {
            let iid = IDCompositionDevice::uuidof();
            let mut item = ptr::null_mut::<c_void>();
            DCompositionCreateDevice3(ptr::null(), &iid, &mut item).hr()?;
            let rc = item as *const IDCompositionDevice;

        }
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
