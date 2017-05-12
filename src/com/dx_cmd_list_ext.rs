use super::com_rc::*;
use super::unsafe_util::*;
use super::dx_com::*;
use super::dx_func::*;
use super::dx_struct::*;
use super::dx_pub_use::*;
use winapi::_core::mem;
use winapi::_core::ptr;
use winapi::ctypes::c_void;
use winapi::shared::ntdef::HANDLE;
use winapi::shared::windef::HWND;
use winapi::shared::winerror::{E_FAIL, HRESULT};
use winapi::um::unknwnbase::IUnknown;
use winapi::um::winbase::INFINITE;

const LIMIT_SIZE: usize = (isize::max_value() as usize);

pub trait ID3D12GraphicsCommandListDx12 {
fn update_subresources_as_heap(
    &self,
                       destination_resource: &ID3D12Resource,
                       intermediate: &ID3D12Resource,
                       intermediate_offset: u64,
                       first_subresource: u32,
                       num_subresources: usize,
                       src_data: &D3D12_SUBRESOURCE_DATA)
                       -> Result<u64, HRESULT> ;
 fn update_subresources(
    &self,
                       destination_resource: &ID3D12Resource,
                       intermediate: &ID3D12Resource,
                       first_subresource: u32,
                       num_subresources: usize,
     required_size:u64,
                               layouts: & D3D12_PLACED_SUBRESOURCE_FOOTPRINT,
                               num_rows: &u32,
                               row_size_in_bytes: & [usize],
        src_data: &D3D12_SUBRESOURCE_DATA
    )
                       -> Result<u64, HRESULT>;}
impl ID3D12GraphicsCommandListDx12 for ID3D12GraphicsCommandList {
// サブリソースをヒープに配置します。
    #[inline]
fn update_subresources_as_heap(
    &self,
                       destination_resource: &ID3D12Resource,
                       intermediate: &ID3D12Resource,
                       intermediate_offset: u64,
                       first_subresource: u32,
                       num_subresources: usize,
                       src_data: &D3D12_SUBRESOURCE_DATA)
                       -> Result<u64, HRESULT> {
    let mut required_size = 0_u64;
    let mem_block = mem::size_of::<D3D12_PLACED_SUBRESOURCE_FOOTPRINT>() + mem::size_of::<u64>() +
                    mem::size_of::<u32>();
    let mem_to_alloc = mem_block * num_subresources;
    let mut mem = {
        let mut mem: Vec<u8> = Vec::new();
        mem.resize(mem_to_alloc, 0_u8);
        mem.into_boxed_slice()
    };
    let mut offset = 0_usize;
    let mut layouts = offset_to_mut_ref::<D3D12_PLACED_SUBRESOURCE_FOOTPRINT>(&mem, &mut offset);
    let mut row_sizes_in_bytes = offset_to_mut_ref::<u64>(&mem, &mut offset);
    let mut num_rows = offset_to_mut_ref::<u32>(&mem, &mut offset);

    let desc = destination_resource.get_desc();
    let device = destination_resource.get_device::<ID3D12Device>()?;
    device.get_copyable_footprints(&desc,
                                   first_subresource,
                                   num_subresources as u32,
                                   intermediate_offset,
                                   &mut layouts,
                                   &mut num_rows,
                                   &mut row_sizes_in_bytes,
                                   &mut required_size);
    let rc = self.update_subresources(
                                 destination_resource,
                                 intermediate,
                                 first_subresource,
                                 num_subresources,
                                 required_size,
                                 &layouts,
                                 &num_rows,
                                 &row_sizes_in_bytes,
                                 src_data)?;
    Ok(rc)
}

// All arrays must be populated (e.g. by calling GetCopyableFootprints)
    #[inline]
 fn update_subresources(
    &self,
                       destination_resource: &ID3D12Resource,
                       intermediate: &ID3D12Resource,
                       first_subresource: u32,
                       num_subresources: usize,
     required_size:u64,
                               layouts: & D3D12_PLACED_SUBRESOURCE_FOOTPRINT,
                               num_rows: &u32,
                               row_sizes_in_bytes: & [usize],
        src_data: &D3D12_SUBRESOURCE_DATA
    )
                       -> Result<u64, HRESULT>
{
    // Minor validation
    let intermediate_desc = intermediate.get_desc();
    let destination_desc = destination_resource.get_desc();
    if intermediate_desc.Dimension != D3D12_RESOURCE_DIMENSION_BUFFER || 
        intermediate_desc.Width < required_size + layouts[0].Offset || 
        required_size > LIMIT_SIZE || 
        (destination_desc.Dimension == D3D12_RESOURCE_DIMENSION_BUFFER && 
            (first_subresource != 0 || num_subresources != 1))
    {
        return Err(E_FAIL);
    }
    {
        let map =  intermediate.map(0,None).hr()?;
    for i in  0..num_subresources
    {
        if row_sizes_in_bytes[i] > LIMIT_SIZE { return Err(E_FAIL);}
   let   dest_data    = D3D12_MEMCPY_DEST{
       pData: map.offset(layouts[i].Offset), 
      RowPitch:    layouts[i].Footprint.RowPitch, 
     SlicePitch:     layouts[i].Footprint.RowPitch * num_rows[i] 
          };
        memcpy_subresource(
            &dest_data, 
            &src_data[i], 
            row_sizes_in_bytes[i] as usize, 
            num_rows[i], 
            layouts[i].Footprint.Depth);
    }

    }
    
  match destination_desc.Dimension {
D3D12_RESOURCE_DIMENSION_BUFFER=>
    {
       let src_box = CD3DX12_BOX::new( 
           ( layouts[0].Offset ) as u32, 
           ( layouts[0].Offset + layouts[0].Footprint.Width )as u32 );
        self.copy_buffer_region(
            destination_resource, 0, intermediate, layouts[0].Offset, layouts[0].Footprint.Width);
    }
_=>    {
        for i in 0..num_subresources
        {
            let dst = D3D12_TEXTURE_COPY_LOCATION::from_footprint(destination_resource, i + first_subresource);
            let src= D3D12_TEXTURE_COPY_LOCATION::from_index  (intermediate, layouts[i]);
             self.copy_texture_region(&dst, 0, 0, 0, &src, None);
        }
    }
  } 
    Ok( required_size)
}
}

//------------------------------------------------------------------------------------------------
// Row-by-row memcpy
#[inline]
fn memcpy_subresource(
     dst:&D3D12_MEMCPY_DEST,
     src:&D3D12_SUBRESOURCE_DATA,
     row_size_in_bytes:usize,
     num_rows:u32,
     num_slices:u32)
{
    for z in 0.. num_slices
    {
        let dst_slice = dst.offset_slice(z);
        let src_slice = src.offset_slice(z);
        for y in 0 .. num_rows
        {
            let dst_ptr = dst.ptr_offset(dst_slice + dst.offset_row(y));
            let src_ptr = src.ptr_offset(src_slice + dst.offset_row(y));
            unsafe{ memcpy(dst_ptr, src_ptr, row_size_in_bytes)};
        }
    }
}
