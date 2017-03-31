use winapi::Interface;



RIDL!{#[uuid(0x00000000, 0x0000, 0x0000, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46)]
interface IUnknown(IUnknownVtbl) {
    fn QueryInterface(
        riid: REFIID,
        ppvObject: *mut *mut c_void,
    ) -> HRESULT,
    fn AddRef() -> ULONG,
    fn Release() -> ULONG,
}}

RIDL!(#[uuid(0x0c733a30, 0x2a1c, 0x11ce, 0xad, 0xe5, 0x00, 0xaa, 0x00, 0x44, 0x77, 0x3d)]
interface ISequentialStream(ISequentialStreamVtbl): IUnknown(IUnknownVtbl) {
    fn Read(
        pv: *mut c_void,
        cb: ULONG,
        pcbRead: *mut ULONG,
    ) -> HRESULT,
    fn Write(
        pv: *const c_void,
        cb: ULONG,
        pcbWritten: *mut ULONG,
    ) -> HRESULT,
}
);

#[macro_export]
macro_rules! RIDL {
    (#[uuid($($uuid:expr),+)]
    interface $interface:ident ($vtbl:ident) {$(
        fn $method:ident($($p:ident : $t:ty,)*) -> $rtr:ty,
    )+}) => (
        #[repr(C)]
        pub struct $vtbl {
            $(pub $method: unsafe extern "system" fn(
                This: *mut $interface,
                $($p: $t),*
            ) -> $rtr,)+
        }
        #[repr(C)]
        pub struct $interface {
            pub lpVtbl: *const $vtbl,
        }
        RIDL!{@impl $interface {$(fn $method($($p: $t,)*) -> $rtr,)+}}
        RIDL!{@uuid $interface $($uuid),+}
    );
    (#[uuid($($uuid:expr),+)]
    interface $interface:ident ($vtbl:ident) : $pinterface:ident ($pvtbl:ident) {
    }) => (
        #[repr(C)]
        pub struct $vtbl {
            pub parent: $pvtbl,
        }
        #[repr(C)]
        pub struct $interface {
            pub lpVtbl: *const $vtbl,
        }
        RIDL!{@deref $interface $pinterface}
        RIDL!{@uuid $interface $($uuid),+}
    );
    (#[uuid($($uuid:expr),+)]
    interface $interface:ident ($vtbl:ident) : $pinterface:ident ($pvtbl:ident) {$(
        fn $method:ident($($p:ident : $t:ty,)*) -> $rtr:ty,
    )+}) => (
        #[repr(C)]
        pub struct $vtbl {
            pub parent: $pvtbl,
            $(pub $method: unsafe extern "system" fn(
                This: *mut $interface,
                $($p: $t,)*
            ) -> $rtr,)+
        }
        #[repr(C)]
        pub struct $interface {
            pub lpVtbl: *const $vtbl,
        }
        RIDL!{@impl $interface {$(fn $method($($p: $t,)*) -> $rtr,)+}}
        RIDL!{@deref $interface $pinterface}
        RIDL!{@uuid $interface $($uuid),+}
    );
    (@deref $interface:ident $pinterface:ident) => (
        impl $crate::_core::ops::Deref for $interface {
            type Target = $pinterface;
            #[inline]
            fn deref(&self) -> &$pinterface {
                unsafe { &*(self as *const $interface as *const $pinterface) }
            }
        }
    );
    (@impl $interface:ident {$(
        fn $method:ident($($p:ident : $t:ty,)*) -> $rtr:ty,
    )+}) => (
        impl $interface {
            $(#[inline] pub unsafe fn $method(&self, $($p: $t,)*) -> $rtr {
                ((*self.lpVtbl).$method)(self as *const _ as *mut _, $($p,)*)
            })+
        }
    );
    (@uuid $interface:ident
        $l:expr, $w1:expr, $w2:expr,
        $b1:expr, $b2:expr, $b3:expr, $b4:expr, $b5:expr, $b6:expr, $b7:expr, $b8:expr
    ) => (
        impl $crate::Interface for $interface {
            #[inline]
            fn uuidof() -> $crate::shared::guiddef::GUID {
                $crate::shared::guiddef::GUID {
                    Data1: $l,
                    Data2: $w1,
                    Data3: $w2,
                    Data4: [$b1, $b2, $b3, $b4, $b5, $b6, $b7, $b8],
                }
            }
        }
    );
}
