#![allow(unused_unsafe)]
#![allow(dead_code)]

use super::com_rc::*;
use super::unsafe_api::*;
use super::unsafe_util::*;
use std::ffi::CStr;
use winapi::Interface;
use winapi::_core::mem;
use winapi::_core::ptr::{self, null_mut};
use winapi::ctypes::c_void;
use winapi::shared::basetsd::{UINT16, UINT64};
pub use winapi::shared::dxgi::*;
pub use winapi::shared::dxgi1_2::*;
pub use winapi::shared::dxgi1_4::*;
pub use winapi::shared::dxgiformat::*;
pub use winapi::shared::dxgitype::*;
use winapi::shared::minwindef::{FALSE, INT, TRUE, UINT};
use winapi::shared::ntdef::{LPCSTR, LPCWSTR};
use winapi::shared::windef::HWND;
use winapi::shared::winerror::{E_FAIL, HRESULT};
pub use winapi::um::d3d12::*;
pub use winapi::um::d3d12sdklayers::*;
pub use winapi::um::d3dcommon::*;
pub use winapi::um::dcomp::*;
use winapi::um::unknwnbase::IUnknown;

pub const DXGI_MWA_NO_WINDOW_CHANGES: UINT = (1 << 0);
pub const DXGI_MWA_NO_ALT_ENTER: UINT = (1 << 1);
pub const DXGI_MWA_NO_PRINT_SCREEN: UINT = (1 << 2);
pub const DXGI_MWA_VALID: UINT = (0x7);

//=====================================================================
// fn
//=====================================================================

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
     flags1: UINT,
     flags2: UINT)
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


//=====================================================================
// Interface Extensions
//=====================================================================

pub trait IDXGIAdapter1Ext {
    fn get_desc1(&self) -> Result<DXGI_ADAPTER_DESC1, HRESULT>;
}
impl IDXGIAdapter1Ext for IDXGIAdapter1 {
    #[inline]
    fn get_desc1(&self) -> Result<DXGI_ADAPTER_DESC1, HRESULT> {
        unsafe {
            let mut desc = mem::uninitialized::<DXGI_ADAPTER_DESC1>();
            self.GetDesc1(&mut desc).hr()?;
            Ok(desc)
        }
    }
}


pub trait IDXGIFactory4Ext {
    fn enum_warp_adapter<U: Interface>(&self) -> ComResult<U>;
    fn enum_adapters1(&self, index: UINT) -> ComResult<IDXGIAdapter1>;
    fn create_swap_chain_for_composition(&self,
                                         device: &IUnknown,
                                         desc: &DXGI_SWAP_CHAIN_DESC1)
                                         -> ComResult<IDXGISwapChain1>;
    fn make_window_association(&self, hwnd: HWND, flags: UINT) -> Result<(), HRESULT>;
    fn d3d12_create_hardware_device(&self) -> ComResult<ID3D12Device>;
    fn d3d12_create_warp_device(&self) -> ComResult<ID3D12Device>;
    fn d3d12_create_best_device(&self) -> ComResult<ID3D12Device>;
}
impl IDXGIFactory4Ext for IDXGIFactory4 {
    #[inline]
    fn enum_warp_adapter<U: Interface>(&self) -> ComResult<U> {
        let riid = U::uuidof();
        let p = unsafe {
            let mut ppv: *mut c_void = null_mut();
            self.EnumWarpAdapter(&riid, &mut ppv).hr()?;
            ppv as *const U
        };
        Ok(ComRc::new(p))
    }
    #[inline]
    fn enum_adapters1(&self, index: UINT) -> ComResult<IDXGIAdapter1> {
        unsafe {
            let mut p = ptr::null_mut();
            self.EnumAdapters1(index, &mut p).hr()?;
            Ok(ComRc::new(p))
        }
    }
    #[inline]
    fn create_swap_chain_for_composition(&self,
                                         device: &IUnknown,
                                         desc: &DXGI_SWAP_CHAIN_DESC1)
                                         -> ComResult<IDXGISwapChain1> {
        unsafe {
            let mut p = ptr::null_mut();
            self.CreateSwapChainForComposition(to_mut_ptr(device), desc, ptr::null_mut(), &mut p)
                .hr()?;
            Ok(ComRc::new(p))
        }
    }
    #[inline]
    fn make_window_association(&self, hwnd: HWND, flags: UINT) -> Result<(), HRESULT> {
        unsafe { self.MakeWindowAssociation(hwnd, flags).hr() }
    }
    #[inline]
    fn d3d12_create_hardware_device(&self) -> ComResult<ID3D12Device> {
        for i in 0_u32.. {
            let adapter = self.enum_adapters1(i)?;
            let desc = adapter.get_desc1()?;
            if (desc.Flags & DXGI_ADAPTER_FLAG_SOFTWARE) != 0 {
                continue;
            }
            let rc = d3d12_create_device::<ID3D12Device>(&adapter, D3D_FEATURE_LEVEL_11_0);
            if rc.is_ok() {
                return rc;
            }
        }
        Err(E_FAIL)
    }
    #[inline]
    fn d3d12_create_warp_device(&self) -> ComResult<ID3D12Device> {
        let adapter = self.enum_warp_adapter::<IDXGIAdapter>()?;
        d3d12_create_device(&adapter, D3D_FEATURE_LEVEL_11_0)
    }
    #[inline]
    fn d3d12_create_best_device(&self) -> ComResult<ID3D12Device> {
        self.d3d12_create_hardware_device()
            .or_else(|_| self.d3d12_create_warp_device())
    }
}


pub trait ID3D12DeviceExt {
    fn create_command_queue<U: Interface>(&self, desc: &D3D12_COMMAND_QUEUE_DESC) -> ComResult<U>;
    fn create_descriptor_heap<U: Interface>(&self,
                                            desc: &D3D12_DESCRIPTOR_HEAP_DESC)
                                            -> ComResult<U>;
    fn get_descriptor_handle_increment_size(&self,
                                            descriptor_heap_type: D3D12_DESCRIPTOR_HEAP_TYPE)
                                            -> UINT;
    fn create_render_target_view(&self,
                                 resource: &ID3D12Resource,
                                 desc: Option<&D3D12_RENDER_TARGET_VIEW_DESC>,
                                 dest_descriptor: D3D12_CPU_DESCRIPTOR_HANDLE)
                                 -> ();
    fn create_command_allocator<U: Interface>(&self,
                                              type_: D3D12_COMMAND_LIST_TYPE)
                                              -> ComResult<U>;
    fn create_root_signature<U: Interface>(&self,
                                           node_mask: UINT,
                                           blob_with_root_signature: *const c_void,
                                           blob_length_in_bytes: usize)
                                           -> ComResult<U>;
    fn create_graphics_pipeline_state<U: Interface>(&self,
                                                    desc: &D3D12_GRAPHICS_PIPELINE_STATE_DESC)
                                                    -> ComResult<U>;
    fn create_command_list<U: Interface>(&self,
                                         node_mask: UINT,
                                         list_type: D3D12_COMMAND_LIST_TYPE,
                                         command_allocator: &ID3D12CommandAllocator,
                                         initial_state: &ID3D12PipelineState)
                                         -> ComResult<U>;
    fn create_committed_resource<U: Interface>(&self,
                                               heap_properties: &D3D12_HEAP_PROPERTIES,
                                               heap_flags: D3D12_HEAP_FLAGS,
                                               resource_desc: &D3D12_RESOURCE_DESC,
                                               initial_resource_state: D3D12_RESOURCE_STATES,
                                               optimized_clear_value: Option<&D3D12_CLEAR_VALUE>)
                                               -> ComResult<U>;
}
impl ID3D12DeviceExt for ID3D12Device {
    #[inline]
    fn create_command_queue<U: Interface>(&self, desc: &D3D12_COMMAND_QUEUE_DESC) -> ComResult<U> {
        let riid = U::uuidof();
        let p = unsafe {
            let mut ppv: *mut c_void = null_mut();
            self.CreateCommandQueue(desc, &riid, &mut ppv).hr()?;
            ppv as *const U
        };
        Ok(ComRc::new(p))
    }
    #[inline]
    fn create_descriptor_heap<U: Interface>(&self,
                                            desc: &D3D12_DESCRIPTOR_HEAP_DESC)
                                            -> ComResult<U> {
        let riid = U::uuidof();
        let p = unsafe {
            let mut ppv: *mut c_void = null_mut();
            self.CreateDescriptorHeap(desc, &riid, &mut ppv).hr()?;
            ppv as *const U
        };
        Ok(ComRc::new(p))
    }
    #[inline]
    fn get_descriptor_handle_increment_size(&self,
                                            descriptor_heap_type: D3D12_DESCRIPTOR_HEAP_TYPE)
                                            -> UINT {
        unsafe { self.GetDescriptorHandleIncrementSize(descriptor_heap_type) }
    }
    #[inline]
    fn create_render_target_view(&self,
                                 resource: &ID3D12Resource,
                                 desc: Option<&D3D12_RENDER_TARGET_VIEW_DESC>,
                                 dest_descriptor: D3D12_CPU_DESCRIPTOR_HANDLE)
                                 -> () {
        unsafe {
            let p_resource = to_mut_ptr(resource);
            self.CreateRenderTargetView(p_resource, opt_to_ptr(desc), dest_descriptor)
        }
    }
    #[inline]
    fn create_command_allocator<U: Interface>(&self,
                                              type_: D3D12_COMMAND_LIST_TYPE)
                                              -> ComResult<U> {
        let riid = U::uuidof();
        let p = unsafe {
            let mut ppv: *mut c_void = null_mut();
            self.CreateCommandAllocator(type_, &riid, &mut ppv)
                .hr()?;
            ppv as *const U
        };
        Ok(ComRc::new(p))
    }
    #[inline]
    fn create_root_signature<U: Interface>(&self,
                                           node_mask: UINT,
                                           blob_with_root_signature: *const c_void,
                                           blob_length_in_bytes: usize)
                                           -> ComResult<U> {
        let riid = U::uuidof();
        let p = unsafe {
            let mut ppv: *mut c_void = null_mut();
            self.CreateRootSignature(node_mask,
                                     blob_with_root_signature,
                                     blob_length_in_bytes,
                                     &riid,
                                     &mut ppv)
                .hr()?;
            ppv as *const U
        };
        Ok(ComRc::new(p))
    }
    #[inline]
    fn create_graphics_pipeline_state<U: Interface>(&self,
                                                    desc: &D3D12_GRAPHICS_PIPELINE_STATE_DESC)
                                                    -> ComResult<U> {
        let riid = U::uuidof();
        let p = unsafe {
            let mut ppv: *mut c_void = null_mut();
            self.CreateGraphicsPipelineState(desc, &riid, &mut ppv)
                .hr()?;
            ppv as *const U
        };
        Ok(ComRc::new(p))
    }
    #[inline]
    fn create_command_list<U: Interface>(&self,
                                         node_mask: UINT,
                                         list_type: D3D12_COMMAND_LIST_TYPE,
                                         command_allocator: &ID3D12CommandAllocator,
                                         initial_state: &ID3D12PipelineState)
                                         -> ComResult<U> {
        let riid = U::uuidof();
        let p = unsafe {
            let mut ppv: *mut c_void = null_mut();
            self.CreateCommandList(node_mask,
                                   list_type,
                                   to_mut_ptr(command_allocator),
                                   to_mut_ptr(initial_state),
                                   &riid,
                                   &mut ppv)
                .hr()?;
            ppv as *const U
        };
        Ok(ComRc::new(p))
    }
    #[inline]
    fn create_committed_resource<U: Interface>(&self,
                                               heap_properties: &D3D12_HEAP_PROPERTIES,
                                               heap_flags: D3D12_HEAP_FLAGS,
                                               resource_desc: &D3D12_RESOURCE_DESC,
                                               initial_resource_state: D3D12_RESOURCE_STATES,
                                               optimized_clear_value: Option<&D3D12_CLEAR_VALUE>)
                                               -> ComResult<U> {
        let riid = U::uuidof();
        let p = unsafe {
            let mut ppv: *mut c_void = null_mut();
            self.CreateCommittedResource(heap_properties,
                                         heap_flags,
                                         resource_desc,
                                         initial_resource_state,
                                         opt_to_ptr(optimized_clear_value),
                                         &riid,
                                         &mut ppv)
                .hr()?;
            ppv as *const U
        };
        Ok(ComRc::new(p))
    }
}


pub trait IDCompositionDeviceExt {
    fn commit(&self) -> Result<(), HRESULT>;
    fn create_target_for_hwnd(&self, hwnd: HWND, topmost: bool) -> ComResult<IDCompositionTarget>;
    fn create_visual(&self) -> ComResult<IDCompositionVisual>;
}
impl IDCompositionDeviceExt for IDCompositionDevice {
    #[inline]
    fn commit(&self) -> Result<(), HRESULT> {
        unsafe { self.Commit().hr() }
    }
    #[inline]
    fn create_target_for_hwnd(&self, hwnd: HWND, topmost: bool) -> ComResult<IDCompositionTarget> {
        unsafe {
            let mut p: *mut IDCompositionTarget = ptr::null_mut();
            self.CreateTargetForHwnd(hwnd, BOOL(topmost), &mut p)
                .hr()?;
            Ok(ComRc::new(p))
        }
    }
    #[inline]
    fn create_visual(&self) -> ComResult<IDCompositionVisual> {
        unsafe {
            let mut p: *mut IDCompositionVisual = ptr::null_mut();
            self.CreateVisual(&mut p).hr()?;
            Ok(ComRc::new(p))
        }
    }
}


pub trait IDCompositionVisualExt {
    fn set_content(&self, content: &IUnknown) -> Result<(), HRESULT>;
}
impl IDCompositionVisualExt for IDCompositionVisual {
    #[inline]
    fn set_content(&self, content: &IUnknown) -> Result<(), HRESULT> {
        unsafe { self.SetContent(content).hr() }
    }
}


pub trait IDCompositionTargetExt {
    fn set_root(&self, visual: &IDCompositionVisual) -> Result<(), HRESULT>;
}
impl IDCompositionTargetExt for IDCompositionTarget {
    #[inline]
    fn set_root(&self, visual: &IDCompositionVisual) -> Result<(), HRESULT> {
        unsafe { self.SetRoot(visual).hr() }
    }
}

pub trait IDXGISwapChain3Ext {
    fn get_current_back_buffer_index(&self) -> UINT;
    fn get_buffer<U: Interface>(&self, buffer: UINT) -> ComResult<U>;
}
impl IDXGISwapChain3Ext for IDXGISwapChain3 {
    #[inline]
    fn get_current_back_buffer_index(&self) -> UINT {
        unsafe { self.GetCurrentBackBufferIndex() }
    }
    #[inline]
    fn get_buffer<U: Interface>(&self, buffer: UINT) -> ComResult<U> {
        let riid = U::uuidof();
        let p = unsafe {
            let mut ppv: *mut c_void = null_mut();
            self.GetBuffer(buffer, &riid, &mut ppv).hr()?;
            ppv as *const U
        };
        Ok(ComRc::new(p))
    }
}

pub trait ID3D12DescriptorHeapExt {
    fn get_desc(&self) -> D3D12_DESCRIPTOR_HEAP_DESC;
    fn get_cpu_descriptor_handle_for_heap_start(&self) -> D3D12_CPU_DESCRIPTOR_HANDLE;
    fn get_gpu_descriptor_handle_for_heap_start(&self) -> D3D12_GPU_DESCRIPTOR_HANDLE;
}
impl ID3D12DescriptorHeapExt for ID3D12DescriptorHeap {
    #[inline]
    fn get_desc(&self) -> D3D12_DESCRIPTOR_HEAP_DESC {
        unsafe {
            let mut rc = mem::uninitialized();
            self.GetDesc(&mut rc);
            rc
        }
    }
    #[inline]
    fn get_cpu_descriptor_handle_for_heap_start(&self) -> D3D12_CPU_DESCRIPTOR_HANDLE {
        unsafe {
            let mut rc = mem::uninitialized();
            self.GetCPUDescriptorHandleForHeapStart(&mut rc);
            rc
        }
    }
    #[inline]
    fn get_gpu_descriptor_handle_for_heap_start(&self) -> D3D12_GPU_DESCRIPTOR_HANDLE {
        unsafe {
            let mut rc = mem::uninitialized();
            self.GetGPUDescriptorHandleForHeapStart(&mut rc);
            rc
        }
    }
}

pub trait ID3D10BlobExt {
    fn get_buffer_pointer(&self) -> *const c_void;
    fn get_buffer_size(&self) -> usize;
}
impl ID3D10BlobExt for ID3D10Blob {
    #[inline]
    fn get_buffer_pointer(&self) -> *const c_void {
        unsafe { self.GetBufferPointer() }
    }
    #[inline]
    fn get_buffer_size(&self) -> usize {
        unsafe { self.GetBufferSize() }
    }
}



//=====================================================================
// Struct Extensions
//=====================================================================


#[allow(non_camel_case_types)]
pub trait CD3DX12_CPU_DESCRIPTOR_HANDLE {
    fn offset(&mut self, offset_in_descriptors: INT, descriptor_increment_size: UINT);
}
impl CD3DX12_CPU_DESCRIPTOR_HANDLE for D3D12_CPU_DESCRIPTOR_HANDLE {
    #[inline]
    fn offset(&mut self, offset_in_descriptors: INT, descriptor_increment_size: UINT) {
        unsafe {
            let offset = ((descriptor_increment_size as i64) * (offset_in_descriptors as i64)) as
                         usize;
            self.ptr += offset;
        }
    }
}

#[allow(non_camel_case_types)]
pub trait CD3DX12_DESCRIPTOR_RANGE {
    fn new(range_type: D3D12_DESCRIPTOR_RANGE_TYPE,
           num_descriptors: UINT,
           base_shader_register: UINT)
           -> D3D12_DESCRIPTOR_RANGE;
}
impl CD3DX12_DESCRIPTOR_RANGE for D3D12_DESCRIPTOR_RANGE {
    #[inline]
    fn new(range_type: D3D12_DESCRIPTOR_RANGE_TYPE,
           num_descriptors: UINT,
           base_shader_register: UINT)
           -> D3D12_DESCRIPTOR_RANGE {
        D3D12_DESCRIPTOR_RANGE {
            RangeType: range_type,
            NumDescriptors: num_descriptors,
            BaseShaderRegister: base_shader_register,
            RegisterSpace: 0,
            OffsetInDescriptorsFromTableStart: D3D12_DESCRIPTOR_RANGE_OFFSET_APPEND,
        }
    }
}

#[allow(non_camel_case_types)]
pub trait CD3DX12_ROOT_PARAMETER {
    fn new_constants(num32_bit_values: UINT,
                     shader_register: UINT,
                     register_space: UINT,
                     visibility: D3D12_SHADER_VISIBILITY)
                     -> D3D12_ROOT_PARAMETER;
    fn new_descriptor_table(descriptor_ranges: &[D3D12_DESCRIPTOR_RANGE],
                            visibility: D3D12_SHADER_VISIBILITY)
                            -> D3D12_ROOT_PARAMETER;
}
impl CD3DX12_ROOT_PARAMETER for D3D12_ROOT_PARAMETER {
    #[inline]
    fn new_constants(num32_bit_values: UINT,
                     shader_register: UINT,
                     register_space: UINT,
                     visibility: D3D12_SHADER_VISIBILITY)
                     -> D3D12_ROOT_PARAMETER {
        unsafe {
            let mut rc = mem::zeroed::<D3D12_ROOT_PARAMETER>();
            rc.ParameterType = D3D12_ROOT_PARAMETER_TYPE_32BIT_CONSTANTS;
            rc.ShaderVisibility = visibility;
            rc.Constants_mut()
                .init(num32_bit_values, shader_register, register_space);
            rc
        }
    }

    #[inline]
    fn new_descriptor_table(descriptor_ranges: &[D3D12_DESCRIPTOR_RANGE],
                            visibility: D3D12_SHADER_VISIBILITY)
                            -> D3D12_ROOT_PARAMETER {
        unsafe {
            let mut rc = mem::zeroed::<D3D12_ROOT_PARAMETER>();
            rc.ParameterType = D3D12_ROOT_PARAMETER_TYPE_DESCRIPTOR_TABLE;
            rc.ShaderVisibility = visibility;
            rc.DescriptorTable_mut().init(descriptor_ranges);
            rc
        }
    }
}

#[allow(non_camel_case_types)]
pub trait CD3DX12_ROOT_CONSTANTS {
    fn init(&mut self, num32_bit_value: UINT, shader_register: UINT, register_space: UINT) -> ();
}
impl CD3DX12_ROOT_CONSTANTS for D3D12_ROOT_CONSTANTS {
    #[inline]
    fn init(&mut self, num32_bit_value: UINT, shader_register: UINT, register_space: UINT) -> () {
        self.Num32BitValues = num32_bit_value;
        self.ShaderRegister = shader_register;
        self.RegisterSpace = register_space;
    }
}

#[allow(non_camel_case_types)]
pub trait CD3DX12_ROOT_DESCRIPTOR_TABLE {
    fn init(&mut self, descriptor_ranges: &[D3D12_DESCRIPTOR_RANGE]) -> ();
}
impl CD3DX12_ROOT_DESCRIPTOR_TABLE for D3D12_ROOT_DESCRIPTOR_TABLE {
    #[inline]
    fn init(&mut self, descriptor_ranges: &[D3D12_DESCRIPTOR_RANGE]) -> () {
        let (num, p) = slice_to_ptr(descriptor_ranges);
        self.NumDescriptorRanges = num;
        self.pDescriptorRanges = p;
    }
}

#[allow(non_camel_case_types)]
pub trait CD3DX12_ROOT_SIGNATURE_DESC {
    fn new(parameters: &[D3D12_ROOT_PARAMETER],
           static_samplers: &[D3D12_STATIC_SAMPLER_DESC],
           flags: D3D12_ROOT_SIGNATURE_FLAGS)
           -> D3D12_ROOT_SIGNATURE_DESC;
}
impl CD3DX12_ROOT_SIGNATURE_DESC for D3D12_ROOT_SIGNATURE_DESC {
    #[inline]
    fn new(parameters: &[D3D12_ROOT_PARAMETER],
           static_samplers: &[D3D12_STATIC_SAMPLER_DESC],
           flags: D3D12_ROOT_SIGNATURE_FLAGS)
           -> D3D12_ROOT_SIGNATURE_DESC {
        let (num_parameters, p_parameters) = slice_to_ptr(parameters);
        let (num_static_samplers, p_static_samplers) = slice_to_ptr(static_samplers);
        D3D12_ROOT_SIGNATURE_DESC {
            NumParameters: num_parameters,
            pParameters: p_parameters,
            NumStaticSamplers: num_static_samplers,
            pStaticSamplers: p_static_samplers,
            Flags: flags,
        }
    }
}

#[allow(non_camel_case_types)]
pub trait D3D12_INPUT_ELEMENT_DESC_EXT {
    fn new(semantic_name: &CStr,
           semantic_index: UINT,
           format: DXGI_FORMAT,
           input_slot: UINT,
           aligned_byte_offset: UINT,
           input_slot_class: D3D12_INPUT_CLASSIFICATION,
           instance_data_step_rate: UINT)
           -> D3D12_INPUT_ELEMENT_DESC;
}
impl D3D12_INPUT_ELEMENT_DESC_EXT for D3D12_INPUT_ELEMENT_DESC {
    #[inline]
    fn new(semantic_name: &CStr,
           semantic_index: UINT,
           format: DXGI_FORMAT,
           input_slot: UINT,
           aligned_byte_offset: UINT,
           input_slot_class: D3D12_INPUT_CLASSIFICATION,
           instance_data_step_rate: UINT)
           -> D3D12_INPUT_ELEMENT_DESC {
        D3D12_INPUT_ELEMENT_DESC {
            SemanticName: semantic_name.as_ptr(),
            SemanticIndex: semantic_index,
            Format: format,
            InputSlot: input_slot,
            AlignedByteOffset: aligned_byte_offset,
            InputSlotClass: input_slot_class,
            InstanceDataStepRate: instance_data_step_rate,
        }
    }
}

#[allow(non_camel_case_types)]
pub trait D3D12_INPUT_LAYOUT_DESC_EXT {
    fn layout(&self) -> D3D12_INPUT_LAYOUT_DESC;
}
impl D3D12_INPUT_LAYOUT_DESC_EXT for [D3D12_INPUT_ELEMENT_DESC] {
    #[inline]
    fn layout(&self) -> D3D12_INPUT_LAYOUT_DESC {
        let (len, p) = slice_to_ptr(&self);
        D3D12_INPUT_LAYOUT_DESC {
            pInputElementDescs: p,
            NumElements: len,
        }
    }
}

#[allow(non_camel_case_types)]
pub trait CD3DX12_SHADER_BYTECODE {
    fn new(blob: &ID3DBlob) -> D3D12_SHADER_BYTECODE;
}
impl CD3DX12_SHADER_BYTECODE for D3D12_SHADER_BYTECODE {
    #[inline]
    fn new(blob: &ID3DBlob) -> D3D12_SHADER_BYTECODE {
        unsafe {
            D3D12_SHADER_BYTECODE {
                pShaderBytecode: blob.GetBufferPointer(),
                BytecodeLength: blob.GetBufferSize(),
            }
        }
    }
}

#[allow(non_camel_case_types)]
pub trait CD3DX12_RASTERIZER_DESC {
    fn default() -> D3D12_RASTERIZER_DESC;
}
impl CD3DX12_RASTERIZER_DESC for D3D12_RASTERIZER_DESC {
    #[inline]
    fn default() -> D3D12_RASTERIZER_DESC {
        D3D12_RASTERIZER_DESC {
            FillMode: D3D12_FILL_MODE_SOLID,
            CullMode: D3D12_CULL_MODE_BACK,
            FrontCounterClockwise: FALSE,
            DepthBias: D3D12_DEFAULT_DEPTH_BIAS as INT,
            DepthBiasClamp: D3D12_DEFAULT_DEPTH_BIAS_CLAMP,
            SlopeScaledDepthBias: D3D12_DEFAULT_SLOPE_SCALED_DEPTH_BIAS,
            DepthClipEnable: TRUE,
            MultisampleEnable: FALSE,
            AntialiasedLineEnable: FALSE,
            ForcedSampleCount: 0,
            ConservativeRaster: D3D12_CONSERVATIVE_RASTERIZATION_MODE_OFF,
        }
    }
}

#[allow(non_camel_case_types)]
pub trait CD3DX12_HEAP_PROPERTIES {
    fn new(heap_type: D3D12_HEAP_TYPE) -> D3D12_HEAP_PROPERTIES;
}
impl CD3DX12_HEAP_PROPERTIES for D3D12_HEAP_PROPERTIES {
    #[inline]
    fn new(heap_type: D3D12_HEAP_TYPE) -> D3D12_HEAP_PROPERTIES {
        D3D12_HEAP_PROPERTIES {
            Type: heap_type,
            CPUPageProperty: D3D12_CPU_PAGE_PROPERTY_UNKNOWN,
            MemoryPoolPreference: D3D12_MEMORY_POOL_UNKNOWN,
            CreationNodeMask: 1,
            VisibleNodeMask: 1,
        }
    }
}

#[allow(non_camel_case_types)]
pub trait CD3DX12_RESOURCE_DESC {
    fn new(dimension: D3D12_RESOURCE_DIMENSION,
           alignment: UINT64,
           width: UINT64,
           height: UINT,
           depth_or_array_size: UINT16,
           mip_levels: UINT16,
           format: DXGI_FORMAT,
           sample_count: UINT,
           sample_quality: UINT,
           layout: D3D12_TEXTURE_LAYOUT,
           flags: D3D12_RESOURCE_FLAGS)
           -> D3D12_RESOURCE_DESC;
    fn buffer(width: usize) -> D3D12_RESOURCE_DESC;
}
impl CD3DX12_RESOURCE_DESC for D3D12_RESOURCE_DESC {
    #[inline]
    fn new(dimension: D3D12_RESOURCE_DIMENSION,
           alignment: UINT64,
           width: UINT64,
           height: UINT,
           depth_or_array_size: UINT16,
           mip_levels: UINT16,
           format: DXGI_FORMAT,
           sample_count: UINT,
           sample_quality: UINT,
           layout: D3D12_TEXTURE_LAYOUT,
           flags: D3D12_RESOURCE_FLAGS)
           -> D3D12_RESOURCE_DESC {
        D3D12_RESOURCE_DESC {
            Dimension: dimension,
            Alignment: alignment,
            Width: width,
            Height: height,
            DepthOrArraySize: depth_or_array_size,
            MipLevels: mip_levels,
            Format: format,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: sample_count,
                Quality: sample_quality,
            },
            Layout: layout,
            Flags: flags,
        }
    }
    #[inline]
    fn buffer(width: usize) -> D3D12_RESOURCE_DESC {
        let flags = D3D12_RESOURCE_FLAG_NONE;
        let alignment = 0_u64;
        D3D12_RESOURCE_DESC::new(D3D12_RESOURCE_DIMENSION_BUFFER,
                                 alignment,
                                 width as UINT64,
                                 1,
                                 1,
                                 1,
                                 DXGI_FORMAT_UNKNOWN,
                                 1,
                                 0,
                                 D3D12_TEXTURE_LAYOUT_ROW_MAJOR,
                                 flags)
    }
}




pub struct Vertex {
    pos: [f32; 3],
    uv: [f32; 2],
}
impl Vertex {
    pub fn new(pos: [f32; 3], uv: [f32; 2]) -> Vertex {
        Vertex { pos: pos, uv: uv }
    }
}