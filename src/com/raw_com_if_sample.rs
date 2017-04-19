use winapi::_core::ops::Deref;
use winapi::Interface;
use winapi::shared::ntdef::{HRESULT, ULONG};
use winapi::ctypes::{c_void, c_ulong, c_ushort, c_uchar};
use winapi::shared::guiddef::{GUID, REFIID};

#[inline]
fn uuid(a: c_ulong,
        b: c_ushort,
        c: c_ushort,
        d1: c_uchar,
        d2: c_uchar,
        d3: c_uchar,
        d4: c_uchar,
        d5: c_uchar,
        d6: c_uchar,
        d7: c_uchar,
        d8: c_uchar)
        -> GUID {
    GUID {
        Data1: a,
        Data2: b,
        Data3: c,
        Data4: [d1, d2, d3, d4, d5, d6, d7, d8],
    }
}

#[repr(C)]
pub struct IUnknownVtbl {
    pub QueryInterface: unsafe extern "system" fn(This: *mut IUnknown,
                                                  riid: REFIID,
                                                  ppvObject: *mut *mut c_void)
                                                  -> HRESULT,
    pub AddRef: unsafe extern "system" fn(This: *mut IUnknown) -> ULONG,
    pub Release: unsafe extern "system" fn(This: *mut IUnknown) -> ULONG,
}
#[repr(C)]
pub struct IUnknown {
    pub lpVtbl: *const IUnknownVtbl,
}
impl IUnknown {
    #[inline]
    pub unsafe fn QueryInterface(&self, riid: REFIID, ppvObject: *mut *mut c_void) -> HRESULT {
        ((*self.lpVtbl).QueryInterface)(self as *const _ as *mut _, riid, ppvObject)
    }

    #[inline]
    pub unsafe fn AddRef(&self) -> ULONG {
        ((*self.lpVtbl).AddRef)(self as *const _ as *mut _)
    }

    #[inline]
    pub unsafe fn Release(&self) -> ULONG {
        ((*self.lpVtbl).Release)(self as *const _ as *mut _)
    }
}
impl Interface for IUnknown {
    #[inline]
    fn uuidof() -> GUID {
        uuid(0x00000000,
             0x0000,
             0x0000,
             0xc0,
             0x00,
             0x00,
             0x00,
             0x00,
             0x00,
             0x00,
             0x46)
    }
}



#[repr(C)]
pub struct ISequentialStreamVtbl {
    pub parent: IUnknownVtbl,
    pub Read: unsafe extern "system" fn(This: *mut ISequentialStream,
                                        pv: *mut c_void,
                                        cb: ULONG,
                                        pcbRead: *mut ULONG)
                                        -> HRESULT,
    pub Write: unsafe extern "system" fn(This: *mut ISequentialStream,
                                         pv: *const c_void,
                                         cb: ULONG,
                                         pcbWritten: *mut ULONG)
                                         -> HRESULT,
}
#[repr(C)]
pub struct ISequentialStream {
    pub lpVtbl: *const ISequentialStreamVtbl,
}
impl ISequentialStream {
    #[inline]
    pub unsafe fn Read(&self, pv: *mut c_void, cb: ULONG, pcbRead: *mut ULONG) -> HRESULT {
        ((*self.lpVtbl).Read)(self as *const _ as *mut _, pv, cb, pcbRead)
    }
    #[inline]
    pub unsafe fn Write(&self, pv: *mut c_void, cb: ULONG, pcbWritten: *mut ULONG) -> HRESULT {
        ((*self.lpVtbl).Read)(self as *const _ as *mut _, pv, cb, pcbWritten)
    }
}
impl Deref for ISequentialStream {
    type Target = IUnknown;
    #[inline]
    fn deref(&self) -> &IUnknown {
        unsafe { &*(self as *const ISequentialStream as *const IUnknown) }
    }
}
impl Interface for ISequentialStream {
    #[inline]
    fn uuidof() -> GUID {
        uuid(0x0c733a30,
             0x2a1c,
             0x11ce,
             0xad,
             0xe5,
             0x00,
             0xaa,
             0x00,
             0x44,
             0x77,
             0x3d)
    }
}
