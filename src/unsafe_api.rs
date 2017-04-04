#![allow(non_snake_case)]
#![allow(dead_code)]

use winapi::ctypes::c_void;
use winapi::shared::basetsd::SIZE_T;
use winapi::shared::minwindef::{LPCVOID, UINT};
use winapi::shared::winerror::HRESULT;
use winapi::shared::guiddef::REFIID;
use winapi::um::d3dcommon::ID3DBlob;
use winapi::um::unknwnbase::IUnknown;
use winapi::um::d3dcommon::D3D_FEATURE_LEVEL;
use winapi::um::d3d12::D3D12_ROOT_SIGNATURE_DESC;
use winapi::um::d3d12::D3D_ROOT_SIGNATURE_VERSION;
use winapi::um::d3d12::D3D12_DESCRIPTOR_HEAP_DESC;
use winapi::um::d3d12::D3D12_CPU_DESCRIPTOR_HANDLE;
use winapi::um::d3d12::D3D12_GPU_DESCRIPTOR_HANDLE;
use winapi::um::d3d12::ID3D12Pageable;
use winapi::um::d3d12::ID3D12PageableVtbl;

#[link(name = "d3d12")]
extern "system" {
    pub fn D3D12CreateDevice(pAdapter: *const IUnknown,
                             MinimumFeatureLevel: D3D_FEATURE_LEVEL,
                             riid: REFIID,
                             ppDevice: *mut *mut c_void)
                             -> HRESULT;
    pub fn D3D12CreateRootSignatureDeserializer(pSrcData: LPCVOID,
                                                SrcDataSizeInBytes: SIZE_T,
                                                pRootSignatureDeserializerInterface: REFIID,
                                                ppRootSignatureDeserializer: *mut *mut c_void)
                                                -> HRESULT;
    pub fn D3D12GetDebugInterface(riid: REFIID, ppvDebug: *mut *mut c_void) -> HRESULT;
    pub fn D3D12SerializeRootSignature(pRootSignature: *const D3D12_ROOT_SIGNATURE_DESC,
                                       Version: D3D_ROOT_SIGNATURE_VERSION,
                                       ppBlob: *mut *mut ID3DBlob,
                                       ppErrorBlob: *mut *mut ID3DBlob)
                                       -> HRESULT;
}

#[link(name = "dxgi")]
extern "system" {
    pub fn CreateDXGIFactory(riid: REFIID, ppFactory: *mut *mut c_void) -> HRESULT;
    pub fn CreateDXGIFactory1(riid: REFIID, ppFactory: *mut *mut c_void) -> HRESULT;
    pub fn CreateDXGIFactory2(Flags: UINT, riid: REFIID, ppFactory: *mut *mut c_void) -> HRESULT;
    pub fn DXGIGetDebugInterface1(Flags: UINT, riid: REFIID, pDebug: *mut *mut c_void) -> HRESULT;
}


RIDL!(#[uuid(0x8efb471d, 0x616c, 0x4f49, 0x90, 0xf7, 0x12, 0x7b, 0xb7, 0x63, 0xfa, 0x51)]
interface ID3D12DescriptorHeap(ID3D12DescriptorHeapVtbl): ID3D12Pageable(ID3D12PageableVtbl) {
     fn GetDesc() -> D3D12_DESCRIPTOR_HEAP_DESC,
     fn GetCPUDescriptorHandleForHeapStart()-> D3D12_CPU_DESCRIPTOR_HANDLE,
     fn GetGPUDescriptorHandleForHeapStart()-> D3D12_GPU_DESCRIPTOR_HANDLE,
});
