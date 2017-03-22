//use winapi::shared::dcomp::dcomptypes::*;
use winapi::shared::ntdef::HANDLE;
use winit;

pub trait DCompWindow {
    fn handle(&self) -> HANDLE;
}

impl DCompWindow for winit::Window {
    fn handle(&self) -> HANDLE {
        unsafe {
            #[allow(deprecated)]
            let p = self.platform_window();
            p as HANDLE
        }
    }
}

pub struct WindowHandler {
    handle: HANDLE,
}

impl WindowHandler {
    pub fn new(handle: HANDLE) -> WindowHandler {
        WindowHandler { handle: handle }
    }
}

impl DCompWindow for WindowHandler {
    fn handle(&self) -> HANDLE {
        self.handle
    }
}
