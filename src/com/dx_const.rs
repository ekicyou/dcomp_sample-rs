#![allow(dead_code)]

use winapi::um::d3d12::{D3D12_SHADER_COMPONENT_MAPPING,
                        D3D12_SHADER_COMPONENT_MAPPING_ALWAYS_SET_BIT_AVOIDING_ZEROMEM_MISTAKES,
                        D3D12_SHADER_COMPONENT_MAPPING_MASK,
                        D3D12_SHADER_COMPONENT_MAPPING_SHIFT};

pub const DXGI_MWA_NO_WINDOW_CHANGES: u32 = (1 << 0);
pub const DXGI_MWA_NO_ALT_ENTER: u32 = (1 << 1);
pub const DXGI_MWA_NO_PRINT_SCREEN: u32 = (1 << 2);
pub const DXGI_MWA_VALID: u32 = (0x7);

#[allow(non_camel_case_types, non_snake_case)]
#[inline]
pub fn D3D12_ENCODE_SHADER_4_COMPONENT_MAPPING(
    s0: u32,
    s1: u32,
    s2: u32,
    s3: u32,
) -> u32 {
    let m0 = s0 & (D3D12_SHADER_COMPONENT_MAPPING_MASK << 0);
    let m1 = s1 &
        (D3D12_SHADER_COMPONENT_MAPPING_MASK <<
             D3D12_SHADER_COMPONENT_MAPPING_SHIFT);
    let m2 = s2 &
        (D3D12_SHADER_COMPONENT_MAPPING_MASK <<
             D3D12_SHADER_COMPONENT_MAPPING_SHIFT * 2);
    let m3 = s3 &
        (D3D12_SHADER_COMPONENT_MAPPING_MASK <<
             D3D12_SHADER_COMPONENT_MAPPING_SHIFT * 3);
    let m4 =
        D3D12_SHADER_COMPONENT_MAPPING_ALWAYS_SET_BIT_AVOIDING_ZEROMEM_MISTAKES;
    m0 | m1 | m2 | m3 | m4
}

#[allow(non_camel_case_types, non_snake_case)]
#[inline]
pub fn D3D12_DECODE_SHADER_4_COMPONENT_MAPPING(
    component_to_extract: u32,
    mapping: u32,
) -> D3D12_SHADER_COMPONENT_MAPPING {
    let rc = mapping >>
        (D3D12_SHADER_COMPONENT_MAPPING_SHIFT * component_to_extract) &
        D3D12_SHADER_COMPONENT_MAPPING_MASK;
    rc as D3D12_SHADER_COMPONENT_MAPPING
}

pub const D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING: u32 =
    (0 & (D3D12_SHADER_COMPONENT_MAPPING_MASK << 0)) |
        (1 &
             (D3D12_SHADER_COMPONENT_MAPPING_MASK <<
                  D3D12_SHADER_COMPONENT_MAPPING_SHIFT)) |
        (2 &
             (D3D12_SHADER_COMPONENT_MAPPING_MASK <<
                  D3D12_SHADER_COMPONENT_MAPPING_SHIFT * 2)) |
        (3 &
             (D3D12_SHADER_COMPONENT_MAPPING_MASK <<
                  D3D12_SHADER_COMPONENT_MAPPING_SHIFT * 3)) |
        D3D12_SHADER_COMPONENT_MAPPING_ALWAYS_SET_BIT_AVOIDING_ZEROMEM_MISTAKES;
