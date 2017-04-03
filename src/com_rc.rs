use winapi::_core::ops::Deref;
use winapi::_core as core;
use winapi::ctypes::c_void;
use winapi::Interface;
use winapi::shared::wtypesbase::ULONG;
use winapi::shared::winerror::{HRESULT, S_OK};
use winapi::um::unknwnbase::IUnknown;

pub trait HresultMapping {
    fn hr(self) -> Result<(), HRESULT>;
}

impl HresultMapping for HRESULT {
    #[inline]
    fn hr(self) -> Result<(), HRESULT> {
        match self {
            S_OK => Ok(()),
            _ => Err(self),
        }
    }
}

#[inline]
pub unsafe fn to_mut_ref<T>(p: *const T) -> *mut T {
    p as *const _ as *mut _
}

pub trait QueryInterface {
    fn query_interface<U: Interface>(&self) -> Result<ComRc<U>, HRESULT>;
}

impl<T: Interface> QueryInterface for T {
    #[inline]
    fn query_interface<U: Interface>(&self) -> Result<ComRc<U>, HRESULT> {
        let unknown = unsafe {
            let p = self as *const T;
            let p_unknown = p as *const IUnknown;
            &*p_unknown
        };
        let riid = U::uuidof();
        let p = unsafe {
            let mut ppv: *mut c_void = core::ptr::null_mut();
            unknown.QueryInterface(&riid, &mut ppv).hr()?;
            ppv as *const U
        };
        Ok(ComRc::new(p))
    }
}

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
    #[inline]
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

#[cfg(test)]
mod tests {
    #![allow(unused_unsafe,non_snake_case,unused_variables,unused_mut)]
    use winapi::_core as core;
    use winapi::ctypes::c_void;
    use winapi::shared::guiddef::REFIID;
    use winapi::shared::wtypesbase::ULONG;
    use winapi::shared::winerror::{HRESULT, S_OK, E_FAIL};
    use winapi::um::unknwnbase::{IUnknown, IUnknownVtbl};
    use winapi::um::objidlbase::{ISequentialStream, ISequentialStreamVtbl};
    use super::*;

    #[repr(C)]
    struct TestSequentialStream {
        pub lpVtbl: *const ISequentialStreamVtbl,
        pub ref_count: ULONG,
    }

    unsafe extern "system" fn QueryInterface(This: *mut IUnknown,
                                             riid: REFIID,
                                             ppvObject: *mut *mut c_void)
                                             -> HRESULT {
        let guid = &*(riid);
        let check = guid.Data4[7];
        match check {
            0x46 => (),
            0x3d => (),
            _ => {
                return E_FAIL;
            }
        }
        *ppvObject = This as *mut c_void;
        S_OK
    }

    unsafe extern "system" fn AddRef(This: *mut IUnknown) -> ULONG {
        let mut test = &mut *(This as *mut TestSequentialStream);
        test.ref_count += 1;
        test.ref_count
    }

    unsafe extern "system" fn Release(This: *mut IUnknown) -> ULONG {
        let mut test = &mut *(This as *mut TestSequentialStream);
        test.ref_count -= 1;
        test.ref_count
    }

    unsafe extern "system" fn Read(_: *mut ISequentialStream,
                                   _: *mut c_void,
                                   _: ULONG,
                                   _: *mut ULONG)
                                   -> HRESULT {
        S_OK
    }
    unsafe extern "system" fn Write(_: *mut ISequentialStream,
                                    _: *const c_void,
                                    _: ULONG,
                                    _: *mut ULONG)
                                    -> HRESULT {
        E_FAIL
    }

    #[test]
    fn com_rc_test() {
        let vtbl = ISequentialStreamVtbl {
            parent: IUnknownVtbl {
                QueryInterface: QueryInterface,
                AddRef: AddRef,
                Release: Release,
            },
            Read: Read,
            Write: Write,
        };
        let test = TestSequentialStream {
            lpVtbl: &vtbl,
            ref_count: 0,
        };

        assert_eq!(0, test.ref_count);
        {
            let com = {
                let p = &test as *const TestSequentialStream;
                let obj = unsafe { p as *const ISequentialStream };
                ComRc::new(obj)
            };
            assert_eq!(1, test.ref_count);

            {
                let com2 = com.query_interface::<IUnknown>().unwrap();
                assert_eq!(2, test.ref_count);

                let com3 = com.query_interface::<ISequentialStream>().unwrap();
                assert_eq!(3, test.ref_count);

                let com4 = com3.query_interface::<IUnknown>().unwrap();
                assert_eq!(4, test.ref_count);
            }
            assert_eq!(1, test.ref_count);

            {
                let com_ref = &com;
                assert_eq!(1, test.ref_count);

                let com2 = com_ref.query_interface::<IUnknown>().unwrap();
                assert_eq!(2, test.ref_count);

                let com3 = com_ref.query_interface::<ISequentialStream>().unwrap();
                assert_eq!(3, test.ref_count);
            }
            assert_eq!(1, test.ref_count);

            unsafe {
                let mut pv: *mut c_void = core::ptr::null_mut();
                let cb: ULONG = 0;
                let buf: *mut ULONG = core::ptr::null_mut();
                com.Read(pv, cb, buf).hr().is_ok();
                com.Write(pv, cb, buf).hr().is_err();
                assert_eq!(2, com.AddRef());
                assert_eq!(2, test.ref_count);
                assert_eq!(1, com.Release());
                assert_eq!(1, test.ref_count);
            }
            assert_eq!(1, test.ref_count);
        }
        assert_eq!(0, test.ref_count);
    }

}
