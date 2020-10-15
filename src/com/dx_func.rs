#![allow(unused_unsafe)]
#![allow(dead_code)]

use super::com_rc::*;
use super::dx_pub_use::*;
use super::unsafe_util::*;
use winapi::Interface;
use winapi::_core::ptr;
use winapi::ctypes::c_void;
use winapi::shared::dxgi::CreateDXGIFactory1;
use winapi::shared::ntdef::HANDLE;
use winapi::shared::ntdef::{LPCSTR, LPCWSTR};
use winapi::shared::winerror::{FACILITY_WIN32, HRESULT};
use winapi::um::d3dcompiler::D3DCompileFromFile;
use winapi::um::errhandlingapi::GetLastError;
use winapi::um::minwinbase::SECURITY_ATTRIBUTES;
use winapi::um::synchapi::CreateEventA;
use winapi::um::synchapi::WaitForSingleObject;
use winapi::um::unknwnbase::IUnknown;

#[inline]
pub fn d3d12_create_device<U: Interface>(
    adapter: &IUnknown,
    minimum_feature_level: D3D_FEATURE_LEVEL,
) -> ComResult<U> {
    let riid = U::uuidof();
    let p = unsafe {
        let mut ppv: *mut c_void = ptr::null_mut();
        D3D12CreateDevice(
            adapter as *const _ as *mut _,
            minimum_feature_level,
            &riid,
            &mut ppv,
        )
        .hr()?;
        ppv as *const U
    };
    Ok(ComRc::new(p))
}

#[inline]
pub fn create_dxgi_factory1<U: Interface>() -> ComResult<U> {
    let riid = U::uuidof();
    let p = unsafe {
        let mut ppv: *mut c_void = ptr::null_mut();
        CreateDXGIFactory1(&riid, &mut ppv).hr()?;
        ppv as *const U
    };
    Ok(ComRc::new(p))
}

#[allow(dead_code)]
#[inline]
pub fn d3d12_get_debug_interface<U: Interface>() -> ComResult<U> {
    let riid = U::uuidof();
    let p = unsafe {
        let mut ppv: *mut c_void = ptr::null_mut();
        D3D12GetDebugInterface(&riid, &mut ppv).hr()?;
        ppv as *const U
    };
    Ok(ComRc::new(p))
}

#[inline]
pub fn dcomp_create_device<U: Interface>(dxgi_device: Option<&IUnknown>) -> ComResult<U> {
    let riid = U::uuidof();
    let p = unsafe {
        let mut ppv: *mut c_void = ptr::null_mut();
        DCompositionCreateDevice3(opt_to_ptr(dxgi_device), &riid, &mut ppv).hr()?;
        ppv as *const U
    };
    Ok(ComRc::new(p))
}

#[inline]
pub fn d3d12_serialize_root_signature(
    root_signature: &D3D12_ROOT_SIGNATURE_DESC,
    version: D3D_ROOT_SIGNATURE_VERSION,
) -> Result<(ComRc<ID3DBlob>, ComRc<ID3DBlob>), HRESULT> {
    unsafe {
        let mut p1: *mut ID3DBlob = ptr::null_mut();
        let mut p2: *mut ID3DBlob = ptr::null_mut();
        D3D12SerializeRootSignature(root_signature, version, &mut p1, &mut p2).hr()?;
        Ok((ComRc::new(p1), ComRc::new(p2)))
    }
}

#[inline]
pub fn d3d_compile_from_file<'a, S: Into<&'a str>>(
    file_name: S,
    defines: Option<&D3D_SHADER_MACRO>,
    include: Option<&ID3DInclude>,
    entrypoint: S,
    target: S,
    flags1: u32,
    flags2: u32,
) -> Result<(ComRc<ID3DBlob>, ComRc<ID3DBlob>), HRESULT> {
    let file_name = to_utf16_chars(file_name);
    let entrypoint = to_utf8_chars(entrypoint);
    let target = to_utf8_chars(target);
    unsafe {
        let mut p1: *mut ID3DBlob = ptr::null_mut();
        let mut p2: *mut ID3DBlob = ptr::null_mut();
        D3DCompileFromFile(
            file_name.as_ptr() as LPCWSTR,
            opt_to_ptr(defines),
            to_mut_ptr(opt_to_ptr(include)),
            entrypoint.as_ptr() as LPCSTR,
            target.as_ptr() as LPCSTR,
            flags1,
            flags2,
            &mut p1,
            &mut p2,
        )
        .hr()?;
        Ok((ComRc::new(p1), ComRc::new(p2)))
    }
}

#[inline]
pub fn get_last_error() -> u32 {
    unsafe { GetLastError() }
}
#[inline]
pub fn hr_from_win32(x: u32) -> HRESULT {
    let x = x as HRESULT;
    if x <= 0 {
        return x;
    }
    let x = x as u32;
    let x = (x & 0x0000FFFFu32) | ((FACILITY_WIN32 as u32) << 16) | 0x80000000u32;
    x as HRESULT
}
#[inline]
pub fn hr_last_error() -> HRESULT {
    hr_from_win32(get_last_error())
}

#[inline]
pub fn create_event(
    attr: Option<&SECURITY_ATTRIBUTES>,
    reset: bool,
    init_state: bool,
    name: Option<LPCSTR>,
) -> Result<HANDLE, HRESULT> {
    let reset = BOOL(reset);
    let init_state = BOOL(init_state);
    let name: LPCSTR = match name {
        None => ptr::null(),
        Some(a) => a,
    };
    let h = unsafe { CreateEventA(opt_to_ptr_mut(attr), reset, init_state, name) };
    if h == ptr::null_mut() {
        hr_last_error().hr()?
    }
    Ok(h)
}

pub fn wait_for_single_object(handle: HANDLE, ms: u32) -> u32 {
    unsafe { WaitForSingleObject(handle, ms) }
}
