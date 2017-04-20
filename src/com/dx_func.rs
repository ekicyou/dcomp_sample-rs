#![allow(unused_unsafe)]
#![allow(dead_code)]

use super::com_rc::*;
use super::dx_pub_use::*;
use super::unsafe_api::*;
use super::unsafe_util::*;
use winapi::Interface;
use winapi::_core::ptr::null_mut;
use winapi::ctypes::c_void;
use winapi::shared::ntdef::{LPCSTR, LPCWSTR};
use winapi::shared::winerror::HRESULT;
use winapi::um::unknwnbase::IUnknown;


#[inline]
pub fn d3d12_create_device<U: Interface>(adapter: &IUnknown,
                                         minimum_feature_level: D3D_FEATURE_LEVEL)
                                         -> ComResult<U> {
    let riid = U::uuidof();
    let p = unsafe {
        let mut ppv: *mut c_void = null_mut();
        D3D12CreateDevice(adapter, minimum_feature_level, &riid, &mut ppv)
            .hr()?;
        ppv as *const U
    };
    Ok(ComRc::new(p))
}

#[inline]
pub fn create_dxgi_factory1<U: Interface>() -> ComResult<U> {
    let riid = U::uuidof();
    let p = unsafe {
        let mut ppv: *mut c_void = null_mut();
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
        let mut ppv: *mut c_void = null_mut();
        D3D12GetDebugInterface(&riid, &mut ppv).hr()?;
        ppv as *const U
    };
    Ok(ComRc::new(p))
}

#[inline]
pub fn dcomp_create_device<U: Interface>(dxgi_device: Option<&IUnknown>) -> ComResult<U> {
    let riid = U::uuidof();
    let p = unsafe {
        let mut ppv: *mut c_void = null_mut();
        DCompositionCreateDevice3(opt_to_ptr(dxgi_device), &riid, &mut ppv)
            .hr()?;
        ppv as *const U
    };
    Ok(ComRc::new(p))
}

#[inline]
pub fn d3d12_serialize_root_signature(root_signature: &D3D12_ROOT_SIGNATURE_DESC,
                                      version: D3D_ROOT_SIGNATURE_VERSION)
                                      -> Result<(ComRc<ID3DBlob>, ComRc<ID3DBlob>), HRESULT> {
    unsafe {
        let mut p1: *mut ID3DBlob = null_mut();
        let mut p2: *mut ID3DBlob = null_mut();
        D3D12SerializeRootSignature(root_signature, version, &mut p1, &mut p2)
            .hr()?;
        Ok((ComRc::new(p1), ComRc::new(p2)))
    }
}

#[inline]
pub fn d3d_compile_from_file<'a, S: Into<&'a str>>
    (file_name: S,
     defines: Option<&D3D_SHADER_MACRO>,
     include: Option<&ID3DInclude>,
     entrypoint: S,
     target: S,
     flags1: u32,
     flags2: u32)
     -> Result<(ComRc<ID3DBlob>, ComRc<ID3DBlob>), HRESULT> {
    let file_name = to_utf16_chars(file_name);
    let entrypoint = to_utf8_chars(entrypoint);
    let target = to_utf8_chars(target);
    unsafe {
        let mut p1: *mut ID3DBlob = null_mut();
        let mut p2: *mut ID3DBlob = null_mut();
        D3DCompileFromFile(file_name.as_ptr() as LPCWSTR,
                           opt_to_ptr(defines),
                           to_mut_ptr(opt_to_ptr(include)),
                           entrypoint.as_ptr() as LPCSTR,
                           target.as_ptr() as LPCSTR,
                           flags1,
                           flags2,
                           &mut p1,
                           &mut p2)
                .hr()?;
        Ok((ComRc::new(p1), ComRc::new(p2)))
    }
}
