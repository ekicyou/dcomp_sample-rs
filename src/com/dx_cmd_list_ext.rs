use super::com_rc::*;
use super::dx_com::*;
use super::dx_func::*;
use super::dx_pub_use::*;
use super::unsafe_util::*;
use rlibc;
use winapi::Interface;
use winapi::_core::mem;
use winapi::_core::ptr;
use winapi::ctypes::c_void;
use winapi::shared::ntdef::HANDLE;
use winapi::shared::windef::HWND;
use winapi::shared::winerror::{E_FAIL, HRESULT};
use winapi::um::unknwnbase::IUnknown;
use winapi::um::winbase::INFINITE;

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
                               row_size_in_bytes: & [usize],
        src_data: &D3D12_SUBRESOURCE_DATA
    )
                       -> Result<u64, HRESULT>
{
    // Minor validation
    let intermediate_desc = intermediate.get_desc();
    let destination_desc = destination_resource.get_desc();
    if intermediate_desc.Dimension != D3D12_RESOURCE_DIMENSION_BUFFER || 
        intermediate_desc.Width < required_size + layouts[0].Offset || 
        required_size > (SIZE_T)-1 || 
        (destination_desc.Dimension == D3D12_RESOURCE_DIMENSION_BUFFER && 
            (FirstSubresource != 0 || num_subresources != 1))
    {
        return Err(E_FALI);
    }
    
    {
        let map =  intermediate.map(0, None).hr()?;
    
    for i in  0..num_subresources
    {
        if (pRowSizesInBytes[i] > (SIZE_T)-1)  return Err(E_FALI);
        D3D12_MEMCPY_DEST DestData = { pData + layouts[i].Offset, layouts[i].Footprint.RowPitch, layouts[i].Footprint.RowPitch * pNumRows[i] };



        MemcpySubresource(&DestData, &pSrcData[i], (SIZE_T)pRowSizesInBytes[i], pNumRows[i], layouts[i].Footprint.Depth);
    }

    }
    
    if (destination_desc.Dimension == D3D12_RESOURCE_DIMENSION_BUFFER)
    {
        CD3DX12_BOX SrcBox( UINT( layouts[0].Offset ), UINT( layouts[0].Offset + layouts[0].Footprint.Width ) );
        pCmdList->CopyBufferRegion(
            pDestinationResource, 0, pIntermediate, layouts[0].Offset, layouts[0].Footprint.Width);
    }
    else
    {
        for i in 0..num_subresources
        {
            CD3DX12_TEXTURE_COPY_LOCATION Dst(pDestinationResource, i + FirstSubresource);
            CD3DX12_TEXTURE_COPY_LOCATION Src(pIntermediate, layouts[i]);
            pCmdList->CopyTextureRegion(&Dst, 0, 0, 0, &Src, nullptr);
        }
    }
    Ok( required_size)
}

}