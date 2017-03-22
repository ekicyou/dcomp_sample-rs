//use winapi::shared::dcomp::dcomptypes::*;
use winapi::shared::ntdef::HANDLE;
use winit;

pub trait DCompWindow {
    fn handle(&self) -> HANDLE;
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
