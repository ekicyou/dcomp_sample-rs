use winapi::ctypes::c_void;
use winapi::shared::basetsd::SIZE_T;
use winapi::shared::minwindef::LPCVOID;
use winapi::shared::winerror::HRESULT;
use winapi::shared::guiddef::REFGUID;
use winapi::um::d3dcommon::ID3DBlob;
use winapi::um::unknwnbase::IUnknown;
use winapi::um::d3dcommon::D3D_FEATURE_LEVEL;
use winapi::um::d3d12::D3D12_ROOT_SIGNATURE_DESC;
use winapi::um::d3d12::D3D_ROOT_SIGNATURE_VERSION;

#[link(name = "d3d12")]
extern "system" {
    pub fn D3D12CreateDevice(pAdapter: *mut IUnknown,
                             MinimumFeatureLevel: D3D_FEATURE_LEVEL,
                             riid: REFGUID,
                             ppDevice: *mut *mut c_void)
                             -> HRESULT;
    pub fn D3D12CreateRootSignatureDeserializer(pSrcData: LPCVOID,
                                                SrcDataSizeInBytes: SIZE_T,
                                                pRootSignatureDeserializerInterface: REFGUID,
                                                ppRootSignatureDeserializer: *mut *mut c_void)
                                                -> HRESULT;
    pub fn D3D12GetDebugInterface(riid: REFGUID, ppvDebug: *mut *mut c_void) -> HRESULT;
    pub fn D3D12SerializeRootSignature(pRootSignature: *const D3D12_ROOT_SIGNATURE_DESC,
                                       Version: D3D_ROOT_SIGNATURE_VERSION,
                                       ppBlob: *mut *mut ID3DBlob,
                                       ppErrorBlob: *mut *mut ID3DBlob)
                                       -> HRESULT;
}
