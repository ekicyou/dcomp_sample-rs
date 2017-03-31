use winapi::_core::ops::{Deref, DerefMut};
use winapi::_core as core;
use winapi::Interface;
use winapi::shared::wtypesbase::ULONG;
use winapi::shared::winerror::{HRESULT, S_OK};

//use winapi::um::unknwnbase::IUnknown;
use super::raw_com_if_sample::IUnknown;


pub struct ComPtr<T: Interface> {
    raw: *const T,
}

impl<T: Interface> ComPtr<T> {
    #[inline]
    pub fn new(com: *const T) -> ComPtr<T> {
        let mut rc = ComPtr { raw: com };
        rc.add_ref();
        rc
    }




    #[inline]
    fn unknown(&self) -> &IUnknown {
        unsafe {
            let p_unknown = self.raw as *const IUnknown;
            &*p_unknown
        }
    }
    #[inline]
    fn add_ref(&mut self) -> ULONG {
        unsafe { self.unknown().AddRef() }
    }
    #[inline]
    fn release(&mut self) -> ULONG {
        unsafe { self.unknown().Release() }
    }
}

impl<T: Interface> Drop for ComPtr<T> {
    fn drop(&mut self) {
        self.release();
    }
}

impl<T: Interface> Deref for ComPtr<T> {
    type Target = T;
    #[inline]
    fn deref(&self) -> &T {
        unsafe { &*self.raw }
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
