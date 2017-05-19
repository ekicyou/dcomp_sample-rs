#![allow(unused_unsafe)]
#![allow(dead_code)]
#![allow(non_camel_case_types)]

use super::dx_pub_use::*;
use super::unsafe_util::*;
use std::ffi::CStr;
use winapi::_core::mem;
use winapi::shared::basetsd::{SIZE_T, UINT16};
use winapi::shared::minwindef::{FALSE, INT, TRUE};

pub trait CD3DX12_CPU_DESCRIPTOR_HANDLE {
    fn offset(&mut self, offset_in_descriptors: INT, descriptor_increment_size: u32);
}
impl CD3DX12_CPU_DESCRIPTOR_HANDLE for D3D12_CPU_DESCRIPTOR_HANDLE {
    #[inline]
    fn offset(&mut self, offset_in_descriptors: INT, descriptor_increment_size: u32) {
        unsafe {
            let offset = ((descriptor_increment_size as i64) * (offset_in_descriptors as i64)) as
                         usize;
            self.ptr += offset;
        }
    }
}

pub trait CD3DX12_DESCRIPTOR_RANGE {
    fn new(range_type: D3D12_DESCRIPTOR_RANGE_TYPE,
           num_descriptors: u32,
           base_shader_register: u32)
           -> D3D12_DESCRIPTOR_RANGE;
}
impl CD3DX12_DESCRIPTOR_RANGE for D3D12_DESCRIPTOR_RANGE {
    #[inline]
    fn new(range_type: D3D12_DESCRIPTOR_RANGE_TYPE,
           num_descriptors: u32,
           base_shader_register: u32)
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

pub trait CD3DX12_ROOT_PARAMETER {
    fn new_constants(num32_bit_values: u32,
                     shader_register: u32,
                     register_space: u32,
                     visibility: D3D12_SHADER_VISIBILITY)
                     -> D3D12_ROOT_PARAMETER;
    fn new_descriptor_table(descriptor_ranges: &[D3D12_DESCRIPTOR_RANGE],
                            visibility: D3D12_SHADER_VISIBILITY)
                            -> D3D12_ROOT_PARAMETER;
}
impl CD3DX12_ROOT_PARAMETER for D3D12_ROOT_PARAMETER {
    #[inline]
    fn new_constants(num32_bit_values: u32,
                     shader_register: u32,
                     register_space: u32,
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

pub trait CD3DX12_ROOT_CONSTANTS {
    fn init(&mut self, num32_bit_value: u32, shader_register: u32, register_space: u32) -> ();
}
impl CD3DX12_ROOT_CONSTANTS for D3D12_ROOT_CONSTANTS {
    #[inline]
    fn init(&mut self, num32_bit_value: u32, shader_register: u32, register_space: u32) -> () {
        self.Num32BitValues = num32_bit_value;
        self.ShaderRegister = shader_register;
        self.RegisterSpace = register_space;
    }
}

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

pub trait CD3DX12_RESOURCE_DESC {
    fn new(dimension: D3D12_RESOURCE_DIMENSION,
           alignment: u64,
           width: u64,
           height: u32,
           depth_or_array_size: UINT16,
           mip_levels: UINT16,
           format: DXGI_FORMAT,
           sample_count: u32,
           sample_quality: u32,
           layout: D3D12_TEXTURE_LAYOUT,
           flags: D3D12_RESOURCE_FLAGS)
           -> D3D12_RESOURCE_DESC;
    fn buffer(width: u64) -> D3D12_RESOURCE_DESC;
}
impl CD3DX12_RESOURCE_DESC for D3D12_RESOURCE_DESC {
    #[inline]
    fn new(dimension: D3D12_RESOURCE_DIMENSION,
           alignment: u64,
           width: u64,
           height: u32,
           depth_or_array_size: UINT16,
           mip_levels: UINT16,
           format: DXGI_FORMAT,
           sample_count: u32,
           sample_quality: u32,
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
    fn buffer(width: u64) -> D3D12_RESOURCE_DESC {
        let flags = D3D12_RESOURCE_FLAG_NONE;
        let alignment = 0_u64;
        D3D12_RESOURCE_DESC::new(D3D12_RESOURCE_DIMENSION_BUFFER,
                                 alignment,
                                 width,
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

pub trait CD3DX12_RANGE {
    fn new(begin: usize, end: usize) -> D3D12_RANGE;
}
impl CD3DX12_RANGE for D3D12_RANGE {
    #[inline]
    fn new(begin: usize, end: usize) -> D3D12_RANGE {
        D3D12_RANGE {
            Begin: begin as SIZE_T,
            End: end as SIZE_T,
        }
    }
}

pub trait CD3DX12_RESOURCE_BARRIER {
    fn transition(pResource: &ID3D12Resource,
                  stateBefore: D3D12_RESOURCE_STATES,
                  stateAfter: D3D12_RESOURCE_STATES)
                  -> D3D12_RESOURCE_BARRIER;
}
impl CD3DX12_RESOURCE_BARRIER for D3D12_RESOURCE_BARRIER {
    #[inline]
    fn transition(resource: &ID3D12Resource,
                  state_before: D3D12_RESOURCE_STATES,
                  state_after: D3D12_RESOURCE_STATES)
                  -> D3D12_RESOURCE_BARRIER {
        let subresource = D3D12_RESOURCE_BARRIER_ALL_SUBRESOURCES;
        let flags = D3D12_RESOURCE_BARRIER_FLAG_NONE;
        unsafe {
            let mut barrier = mem::zeroed::<D3D12_RESOURCE_BARRIER>();
            barrier.Type = D3D12_RESOURCE_BARRIER_TYPE_TRANSITION;
            barrier.Flags = flags;
            {
                let mut transition = barrier.u.Transition_mut();
                transition.pResource = resource as *const _ as *mut _;
                transition.StateBefore = state_before;
                transition.StateAfter = state_after;
                transition.Subresource = subresource;
            }
            barrier
        }
    }
}

pub trait CD3DX12_TEXTURE_COPY_LOCATION {
    fn from_footprint(res: &ID3D12Resource, footprint:&D3D12_PLACED_SUBRESOURCE_FOOTPRINT)->D3D12_TEXTURE_COPY_LOCATION;
    fn from_index(res: &ID3D12Resource, sub:u32)->D3D12_TEXTURE_COPY_LOCATION;
}
impl CD3DX12_TEXTURE_COPY_LOCATION for D3D12_TEXTURE_COPY_LOCATION {
    #[inline]
    fn from_footprint(res: &ID3D12Resource, footprint:&D3D12_PLACED_SUBRESOURCE_FOOTPRINT)->D3D12_TEXTURE_COPY_LOCATION
    {
        unsafe{
            let mut rc = mem::uninitialized::<D3D12_TEXTURE_COPY_LOCATION>();
            rc.pResource = res as *const _;
            rc.        Type = D3D12_TEXTURE_COPY_TYPE_PLACED_FOOTPRINT;
            {
                let mut u = rc.u.PlacedFootprint_mut();
                u = footprint as *const _;
            }
rc
        }
    }
    #[inline]
    fn from_index(res: &ID3D12Resource, sub:u32)->D3D12_TEXTURE_COPY_LOCATION
    {
        unsafe{
            let mut rc = mem::uninitialized::<D3D12_TEXTURE_COPY_LOCATION>();
            rc.pResource = res as *const _;
            rc.        Type = D3D12_TEXTURE_COPY_TYPE_SUBRESOURCE_INDEX;
            {
                let mut u = rc.u.SubresourceIndex_mut();
                u = sub;
            }
rc
        }
    }
}
