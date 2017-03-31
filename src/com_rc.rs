use winapi::_core::ops::{Deref, DerefMut};
use winapi::_core as core;
use winapi::ctypes::c_void;
use winapi::Interface;
use winapi::shared::wtypesbase::ULONG;
use winapi::shared::winerror::{HRESULT, S_OK};
use winapi::um::unknwnbase::IUnknown;

pub struct ComRc<T: Interface> {
    raw: *const T,
}

impl<T: Interface> ComRc<T> {
    #[inline]
    pub fn new(com: *const T) -> ComRc<T> {
        let mut rc = ComRc { raw: com };
        rc.add_ref();
        rc
    }

    #[inline]
    pub fn query_interface<U: Interface>(&self) -> Result<ComRc<U>, HRESULT> {
        let riid = U::uuidof();
        let p = unsafe {
            let mut ppvObject: *mut c_void = core::ptr::null_mut();
            self.unknown()
                .QueryInterface(&riid, &mut ppvObject)
                .hr()?;
            ppvObject as *const U
        };
        Ok(ComRc::new(p))
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

impl<T: Interface> Drop for ComRc<T> {
    fn drop(&mut self) {
        self.release();
    }
}

impl<T: Interface> Deref for ComRc<T> {
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
