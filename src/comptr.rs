use winapi::_core::ops::Deref;
use winapi::shared::wtypesbase::ULONG;
use winapi::shared::winerror::{HRESULT, E_FAIL};
use winapi::um::unknwnbase::IUnknown;

pub trait IUnknownInterface {
    unsafe fn query_interface<T: IUnknownInterface>(&self) -> Result<T, HRESULT>;
    unsafe fn add_ref(&mut self) -> ULONG;
    unsafe fn release(&mut self) -> ULONG;
}

impl IUnknownInterface for IUnknown {
    unsafe fn query_interface<T: IUnknownInterface>(&self) -> Result<T, HRESULT> {
        Err(E_FAIL)
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

pub struct ComPtr<T: IUnknownInterface> {
    raw: T,
}

impl<T: IUnknownInterface> ComPtr<T> {
    fn new(mut com: T) -> ComPtr<T> {
        unsafe { com.add_ref() };
        ComPtr { raw: com }
    }
}

impl<T: IUnknownInterface> Drop for ComPtr<T> {
    fn drop(&mut self) {
        unsafe { self.raw.release() };
    }
}

impl<T: IUnknownInterface> Deref for ComPtr<T> {
    type Target = T;
    fn deref(&self) -> &T {
        let com = &self.raw;
        com
    }
}
