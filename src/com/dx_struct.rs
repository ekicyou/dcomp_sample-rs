#![allow(unused_unsafe)]
#![allow(dead_code)]

use super::dx_pub_use::*;
use super::unsafe_util::*;
use std::ffi::CStr;
use winapi::_core::mem;
use winapi::shared::basetsd::{SIZE_T, UINT16};
use winapi::shared::minwindef::{FALSE, INT, TRUE};

pub struct Vertex {
    pos: [f32; 3],
    uv: [f32; 2],
}
impl Vertex {
    pub fn new(pos: [f32; 3], uv: [f32; 2]) -> Vertex {
        Vertex { pos: pos, uv: uv }
    }
}

#[allow(non_camel_case_types)]
pub struct XMFLOAT4 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}
impl XMFLOAT4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> XMFLOAT4 {
        XMFLOAT4 {
            x: x,
            y: y,
            z: z,
            w: w,
        }
    }
}

#[allow(non_camel_case_types)]
pub trait D3D12_INPUT_ELEMENT_DESC_EXT {
    fn new(semantic_name: &CStr,
           semantic_index: u32,
           format: DXGI_FORMAT,
           input_slot: u32,
           aligned_byte_offset: u32,
           input_slot_class: D3D12_INPUT_CLASSIFICATION,
           instance_data_step_rate: u32)
           -> D3D12_INPUT_ELEMENT_DESC;
}
impl D3D12_INPUT_ELEMENT_DESC_EXT for D3D12_INPUT_ELEMENT_DESC {
    #[inline]
    fn new(semantic_name: &CStr,
           semantic_index: u32,
           format: DXGI_FORMAT,
           input_slot: u32,
           aligned_byte_offset: u32,
           input_slot_class: D3D12_INPUT_CLASSIFICATION,
           instance_data_step_rate: u32)
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
pub trait D3D12_MEMCPY_EXT {
    fn offset_slice(&self,slice: u32)->usize;
    fn offset_row(&self,slice: u32)->usize;
    fn ptr_offset(&self, offset: usize)->*mut u8;
}
impl D3D12_MEMCPY_EXT for D3D12_MEMCPY_DEST {
    #[inline]
    fn offset_slice(&self,slice: u32)->usize
    {
        (self.SlicePitch as usize) * (slice as usize)
    }
     #[inline]
    fn offset_row(&self, row: u32)->usize
    {
        (self.RowPitch as usize) * (row as usize)
    }
    #[inline]
    fn ptr_offset(&self, offset: usize)->*mut u8
    {
        unsafe{
            let mut a:usize = mem::transmute(self.pData);
            a += offset;
            mem::transmute::<_, _>(a)
        }
    }
}
impl D3D12_MEMCPY_EXT for D3D12_SUBRESOURCE_DATA {
    #[inline]
    fn offset_slice(&self,slice: u32)->usize
    {
        (self.SlicePitch as usize) * (slice as usize)
    }
     #[inline]
    fn offset_row(&self, row: u32)->usize
    {
        (self.RowPitch as usize) * (row as usize)
    }
    #[inline]
    fn ptr_offset(&self, offset: usize)->*mut u8
    {
        unsafe{
            let mut a:usize = mem::transmute(self.pData);
            a += offset;
            mem::transmute(a)
        }
    }
}



