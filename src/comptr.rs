use winapi::_core::ops::{Deref, DerefMut};
use winapi::_core as core;
use winapi::Interface;
use winapi::shared::wtypesbase::ULONG;
use winapi::shared::winerror::{HRESULT, S_OK, E_FAIL};
use winapi::um::unknwnbase::IUnknown;

pub trait IUnknownInterface {
    unsafe fn query_interface<T: IUnknownInterface + Interface>(&self) -> Result<*mut T, HRESULT>;
    unsafe fn add_ref(&mut self) -> ULONG;
    unsafe fn release(&mut self) -> ULONG;
}

impl IUnknownInterface for IUnknown {
    unsafe fn query_interface<T: IUnknownInterface + Interface>(&self) -> Result<*mut T, HRESULT> {
        let guid = T::uuidof();
        let mut ptr = core::ptr::null_mut();
        unsafe {
            match self.QueryInterface(&guid, &mut ptr) {
                S_OK => Ok(ptr as *mut T),
                hr => Err(hr),
            }
        }
    }
    unsafe fn add_ref(&mut self) -> ULONG {
        let count = self.AddRef();
        count
    }
    unsafe fn release(&mut self) -> ULONG {
        let count = self.Release();
        count
    }
}

pub struct ComPtr<T: IUnknownInterface + Interface> {
    raw: T,
}

impl<T: IUnknownInterface + Interface> ComPtr<T> {
    fn new(mut com: T) -> ComPtr<T> {
        unsafe { com.add_ref() };
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
