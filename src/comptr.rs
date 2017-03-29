use winapi::_core::ops::{Deref, DerefMut};
use winapi::_core as core;
use winapi::Interface;
use winapi::shared::wtypesbase::ULONG;
use winapi::shared::winerror::{HRESULT, S_OK};
use winapi::um::unknwnbase::IUnknown;

pub trait IUnknownInterface {
    fn unknown(&self) -> &IUnknown;
    fn unknown_mut(&mut self) -> &IUnknown;
    unsafe fn query_interface<T: IUnknownInterface + Interface>(&self) -> Result<*mut T, HRESULT> {
        let unknown = self.unknown();
        let guid = T::uuidof();
        let mut ptr = core::ptr::null_mut();
        unsafe {
            match unknown.QueryInterface(&guid, &mut ptr) {
                S_OK => Ok(ptr as *mut T),
                hr => Err(hr),
            }
        }
    }
    unsafe fn add_ref(&mut self) -> ULONG {
        let mut unknown = self.unknown_mut();
        let count = unknown.AddRef();
        count
    }
    unsafe fn release(&mut self) -> ULONG {
        let mut unknown = self.unknown_mut();
        let count = unknown.Release();
        count
    }
}

pub struct ComPtr<T: IUnknownInterface + Interface> {
    raw: *const T,
}

impl<T: IUnknownInterface + Interface> IUnknownInterface for ComPtr<T> {
    fn unknown(&self) -> &IUnknown {
        self.raw
    }
    fn unknown_mut(&mut self) -> &IUnknown {
        self.raw
    }
}


impl<T: IUnknownInterface + Interface> ComPtr<T> {
    fn new(com: *const T) -> ComPtr<T> {
        unsafe { &com.add_ref() };
        ComPtr { raw: com }
    }
}

impl<T: IUnknownInterface + Interface> Drop for ComPtr<T> {
    fn drop(&mut self) {
        unsafe { self.raw.release() };
    }
}

impl<T: IUnknownInterface + Interface> Deref for ComPtr<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.raw
    }
}

impl<T: IUnknownInterface + Interface> DerefMut for ComPtr<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.raw
    }
}

pub trait HresultMapping {
    fn hr(self) -> Result<(), HRESULT>;
}

impl HresultMapping for HRESULT {
    fn hr(self) -> Result<(), HRESULT> {
        match self {
            S_OK => Ok(()),
            _ => Err(self),
        }
    }
}
