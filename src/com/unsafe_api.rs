#![allow(non_snake_case)]
#![allow(dead_code)]

use winapi::ctypes::c_void;
use winapi::shared::basetsd::SIZE_T;
use winapi::shared::guiddef::REFIID;
use winapi::shared::minwindef::{BOOL, DWORD, LPCVOID, LPVOID, UINT, ULONG};
use winapi::shared::ntdef::{HANDLE, LPCSTR, LPCWSTR, PHANDLE, PVOID};
use winapi::shared::winerror::HRESULT;
use winapi::um::d3d12::{D3D12_ROOT_SIGNATURE_DESC, D3D_ROOT_SIGNATURE_VERSION};
use winapi::um::d3dcommon::{D3D_FEATURE_LEVEL, D3D_SHADER_MACRO, ID3DBlob,
                            ID3DInclude};
use winapi::um::d3dcompiler::{D3D_BLOB_PART, D3D_SHADER_DATA};
use winapi::um::minwinbase::SECURITY_ATTRIBUTES;
use winapi::um::unknwnbase::IUnknown;
use winapi::um::winnt::WAITORTIMERCALLBACK;

#[link(name = "kernel32")]
extern "system" {
    pub fn CreateEventA(
        lpEventAttributes: *const SECURITY_ATTRIBUTES,
        bManualReset: BOOL,
        bInitialState: BOOL,
        lpName: LPCSTR,
    ) -> HANDLE;
    pub fn WaitForSingleObject(hHandle: HANDLE, dwMilliseconds: DWORD)
        -> DWORD;
}

#[link(name = "d3d12")]
extern "system" {
    pub fn D3D12CreateDevice(
        pAdapter: *const IUnknown,
        MinimumFeatureLevel: D3D_FEATURE_LEVEL,
        riid: REFIID,
        ppDevice: *mut *mut c_void,
    ) -> HRESULT;
    pub fn D3D12CreateRootSignatureDeserializer(
        pSrcData: LPCVOID,
        SrcDataSizeInBytes: SIZE_T,
        pRootSignatureDeserializerInterface: REFIID,
        ppRootSignatureDeserializer: *mut *mut c_void,
    ) -> HRESULT;
    pub fn D3D12GetDebugInterface(
        riid: REFIID,
        ppvDebug: *mut *mut c_void,
    ) -> HRESULT;
    pub fn D3D12SerializeRootSignature(
        pRootSignature: *const D3D12_ROOT_SIGNATURE_DESC,
        Version: D3D_ROOT_SIGNATURE_VERSION,
        ppBlob: *mut *mut ID3DBlob,
        ppErrorBlob: *mut *mut ID3DBlob,
    ) -> HRESULT;
}

#[link(name = "dxgi")]
extern "system" {
    pub fn CreateDXGIFactory(
        riid: REFIID,
        ppFactory: *mut *mut c_void,
    ) -> HRESULT;
    pub fn CreateDXGIFactory1(
        riid: REFIID,
        ppFactory: *mut *mut c_void,
    ) -> HRESULT;
    pub fn CreateDXGIFactory2(
        Flags: UINT,
        riid: REFIID,
        ppFactory: *mut *mut c_void,
    ) -> HRESULT;
    pub fn DXGIGetDebugInterface1(
        Flags: UINT,
        riid: REFIID,
        pDebug: *mut *mut c_void,
    ) -> HRESULT;
}

#[link(name = "d3dcompiler")]
extern "system" {
    pub fn D3DCompile(
        pSrcData: LPCVOID,
        SrcDataSize: SIZE_T,
        pSourceName: LPCSTR,
        pDefines: *const D3D_SHADER_MACRO,
        pInclude: *mut ID3DInclude,
        pEntrypoint: LPCSTR,
        pTarget: LPCSTR,
        Flags1: UINT,
        Flags2: UINT,
        ppCode: *mut *mut ID3DBlob,
        ppErrorMsgs: *mut *mut ID3DBlob,
    ) -> HRESULT;
    pub fn D3DCompile2(
        pSrcData: LPCVOID,
        SrcDataSize: SIZE_T,
        pSourceName: LPCSTR,
        pDefines: *const D3D_SHADER_MACRO,
        pInclude: *mut ID3DInclude,
        pEntrypoint: LPCSTR,
        pTarget: LPCSTR,
        Flags1: UINT,
        Flags2: UINT,
        SecondaryDataFlags: UINT,
        pSecondaryData: LPCVOID,
        SecondaryDataSize: SIZE_T,
        ppCode: *mut *mut ID3DBlob,
        ppErrorMsgs: *mut *mut ID3DBlob,
    ) -> HRESULT;
    pub fn D3DCompileFromFile(
        pFileName: LPCWSTR,
        pDefines: *const D3D_SHADER_MACRO,
        pInclude: *mut ID3DInclude,
        pEntrypoint: LPCSTR,
        pTarget: LPCSTR,
        Flags1: UINT,
        Flags2: UINT,
        ppCode: *mut *mut ID3DBlob,
        ppErrorMsgs: *mut *mut ID3DBlob,
    ) -> HRESULT;
    pub fn D3DCompressShaders(
        uNumShaders: UINT,
        pShaderData: *mut D3D_SHADER_DATA,
        uFlags: UINT,
        ppCompressedData: *mut *mut ID3DBlob,
    ) -> HRESULT;
    pub fn D3DCreateBlob(Size: SIZE_T, ppBlob: *mut *mut ID3DBlob) -> HRESULT;
    pub fn D3DDecompressShaders(
        pSrcData: LPCVOID,
        SrcDataSize: SIZE_T,
        uNumShaders: UINT,
        uStartIndex: UINT,
        pIndices: *mut UINT,
        uFlags: UINT,
        ppShaders: *mut *mut ID3DBlob,
        pTotalShaders: *mut UINT,
    ) -> HRESULT;
    pub fn D3DDisassemble(
        pSrcData: LPCVOID,
        SrcDataSize: SIZE_T,
        Flags: UINT,
        szComments: LPCSTR,
        ppDisassembly: *mut *mut ID3DBlob,
    ) -> HRESULT;
    // pub fn D3DDisassemble10Effect(
    //     pEffect: *mut ID3D10Effect, Flags: UINT, ppDisassembly: *mut *mut ID3DBlob,
    // ) -> HRESULT;
    // pub fn D3DDisassemble11Trace();
    pub fn D3DDisassembleRegion(
        pSrcData: LPCVOID,
        SrcDataSize: SIZE_T,
        Flags: UINT,
        szComments: LPCSTR,
        StartByteOffset: SIZE_T,
        NumInsts: SIZE_T,
        pFinishByteOffset: *mut SIZE_T,
        ppDisassembly: *mut *mut ID3DBlob,
    ) -> HRESULT;
    pub fn D3DGetBlobPart(
        pSrcData: LPCVOID,
        SrcDataSize: SIZE_T,
        Part: D3D_BLOB_PART,
        Flags: UINT,
        ppPart: *mut *mut ID3DBlob,
    ) -> HRESULT;
    pub fn D3DGetDebugInfo(
        pSrcData: LPCVOID,
        SrcDataSize: SIZE_T,
        ppDebugInfo: *mut *mut ID3DBlob,
    ) -> HRESULT;
    pub fn D3DGetInputAndOutputSignatureBlob(
        pSrcData: LPCVOID,
        SrcDataSize: SIZE_T,
        ppSignatureBlob: *mut *mut ID3DBlob,
    ) -> HRESULT;
    pub fn D3DGetInputSignatureBlob(
        pSrcData: LPCVOID,
        SrcDataSize: SIZE_T,
        ppSignatureBlob: *mut *mut ID3DBlob,
    ) -> HRESULT;
    pub fn D3DGetOutputSignatureBlob(
        pSrcData: LPCVOID,
        SrcDataSize: SIZE_T,
        ppSignatureBlob: *mut *mut ID3DBlob,
    ) -> HRESULT;
    pub fn D3DGetTraceInstructionOffsets(
        pSrcData: LPCVOID,
        SrcDataSize: SIZE_T,
        Flags: UINT,
        StartInstIndex: SIZE_T,
        NumInsts: SIZE_T,
        pOffsets: *mut SIZE_T,
        pTotalInsts: *mut SIZE_T,
    ) -> HRESULT;
    pub fn D3DPreprocess(
        pSrcData: LPCVOID,
        SrcDataSize: SIZE_T,
        pSourceName: LPCSTR,
        pDefines: *const D3D_SHADER_MACRO,
        pInclude: *mut ID3DInclude,
        ppCodeText: *mut *mut ID3DBlob,
        ppErrorMsgs: *mut *mut ID3DBlob,
    ) -> HRESULT;
    pub fn D3DReadFileToBlob(
        pFileName: LPCWSTR,
        ppContents: *mut *mut ID3DBlob,
    ) -> HRESULT;
    pub fn D3DReflect(
        pSrcData: LPCVOID,
        SrcDataSize: SIZE_T,
        pInterface: REFIID,
        ppReflector: *mut *mut c_void,
    ) -> HRESULT;
    pub fn D3DReflectLibrary(
        pSrcData: LPCVOID,
        SrcDataSize: SIZE_T,
        riid: REFIID,
        ppReflector: *mut LPVOID,
    ) -> HRESULT;
    // pub fn D3DReturnFailure1();
    pub fn D3DSetBlobPart(
        pSrcData: LPCVOID,
        SrcDataSize: SIZE_T,
        Part: D3D_BLOB_PART,
        Flags: UINT,
        pPart: LPCVOID,
        PartSize: SIZE_T,
        ppNewShader: *mut *mut ID3DBlob,
    ) -> HRESULT;
    pub fn D3DStripShader(
        pShaderBytecode: LPCVOID,
        BytecodeLength: SIZE_T,
        uStripFlags: UINT,
        ppStrippedBlob: *mut *mut ID3DBlob,
    ) -> HRESULT;
    pub fn D3DWriteBlobToFile(
        pBlob: *mut ID3DBlob,
        pFileName: LPCWSTR,
        bOverwrite: BOOL,
    ) -> HRESULT;
}
