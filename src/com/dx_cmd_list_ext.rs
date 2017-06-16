use super::com_rc::*;
use super::dx_cd3dx12::*;
use super::dx_com::*;
use super::dx_func::*;
use super::dx_pub_use::*;
use super::dx_struct::*;
use super::unsafe_util::*;
use winapi::_core as core;
use winapi::_core::mem;
use winapi::_core::ptr;
use winapi::ctypes::c_void;
use winapi::shared::ntdef::HANDLE;
use winapi::shared::windef::HWND;
use winapi::shared::winerror::{E_FAIL, HRESULT};
use winapi::um::unknwnbase::IUnknown;
use winapi::um::winbase::INFINITE;

const LIMIT_SIZE: u64 = (core::isize::MAX as u64);


pub trait ID3D12GraphicsCommandListExt {
    fn close(&self) -> Result<(), HRESULT>;
    fn resource_barrier(
        &self,
        mum_barriers: u32,
        barriers: &D3D12_RESOURCE_BARRIER,
    ) -> ();
    fn copy_buffer_region(
        &self,
        dst_buffer: &ID3D12Resource,
        dst_offset: u64,
        src_buffer: &ID3D12Resource,
        src_offset: u64,
        num_bytes: u64,
    ) -> ();
    fn copy_texture_region(
        &self,
        dst: &D3D12_TEXTURE_COPY_LOCATION,
        x: u32,
        y: u32,
        z: u32,
        src: &D3D12_TEXTURE_COPY_LOCATION,
        src_box: Option<&D3D12_BOX>,
    ) -> ();
    fn update_subresources_as_heap(
        &self,
        destination_resource: &ID3D12Resource,
        intermediate: &ID3D12Resource,
        intermediate_offset: usize,
        src_data: &[D3D12_SUBRESOURCE_DATA],
    ) -> Result<u64, HRESULT>;
    fn update_subresources(
        &self,
        destination_resource: &ID3D12Resource,
        intermediate: &ID3D12Resource,
        required_size: u64,
        layouts: &[D3D12_PLACED_SUBRESOURCE_FOOTPRINT],
        num_rows: &[u32],
        row_sizes_in_bytes: &[u64],
        src_data: &[D3D12_SUBRESOURCE_DATA],
    ) -> Result<u64, HRESULT>;
}
impl ID3D12GraphicsCommandListExt for ID3D12GraphicsCommandList {
    #[inline]
    fn close(&self) -> Result<(), HRESULT> { unsafe { self.Close().hr() } }
    #[inline]
    fn resource_barrier(
        &self,
        mum_barriers: u32,
        barriers: &D3D12_RESOURCE_BARRIER,
    ) -> () {
        unsafe { self.ResourceBarrier(mum_barriers, barriers) }
    }
    #[inline]
    fn copy_buffer_region(
        &self,
        dst_buffer: &ID3D12Resource,
        dst_offset: u64,
        src_buffer: &ID3D12Resource,
        src_offset: u64,
        num_bytes: u64,
    ) -> () {
        unsafe {
            self.CopyBufferRegion(
                dst_buffer as *const _ as *mut _,
                dst_offset,
                src_buffer as *const _ as *mut _,
                src_offset,
                num_bytes,
            )
        }
    }
    #[inline]
    fn copy_texture_region(
        &self,
        dst: &D3D12_TEXTURE_COPY_LOCATION,
        x: u32,
        y: u32,
        z: u32,
        src: &D3D12_TEXTURE_COPY_LOCATION,
        src_box: Option<&D3D12_BOX>,
    ) -> () {
        unsafe {
            self.CopyTextureRegion(
                dst as *const _,
                x,
                y,
                z,
                src as *const _,
                opt_to_ptr(src_box),
            )
        }
    }

    // サブリソースをヒープに配置します。
    #[inline]
    fn update_subresources_as_heap(
        &self,
        destination_resource: &ID3D12Resource,
        intermediate: &ID3D12Resource,
        intermediate_offset: usize,
        src_data: &[D3D12_SUBRESOURCE_DATA],
    ) -> Result<u64, HRESULT> {
        let desc = destination_resource.get_desc();
        let device = destination_resource.get_device::<ID3D12Device>()?;
        let (layouts, num_rows, row_sizes_in_bytes, required_size) =
            device.get_copyable_footprints(
                &desc,
                src_data.len(),
                intermediate_offset,
            );
        let rc = self.update_subresources(
            destination_resource,
            intermediate,
            required_size,
            &layouts,
            &num_rows,
            &row_sizes_in_bytes,
            src_data,
        )?;
        Ok(rc)
    }

    // All arrays must be populated (e.g. by calling GetCopyableFootprints)
    #[inline]
    fn update_subresources(
        &self,
        destination_resource: &ID3D12Resource,
        intermediate: &ID3D12Resource,
        required_size: u64,
        layouts: &[D3D12_PLACED_SUBRESOURCE_FOOTPRINT],
        num_rows: &[u32],
        row_sizes_in_bytes: &[u64],
        src_data: &[D3D12_SUBRESOURCE_DATA],
    ) -> Result<u64, HRESULT> {
        let num_subresources = src_data.len();
        // Minor validation
        let intermediate_desc = intermediate.get_desc();
        let destination_desc = destination_resource.get_desc();
        if intermediate_desc.Dimension != D3D12_RESOURCE_DIMENSION_BUFFER ||
            intermediate_desc.Width < required_size + layouts[0].Offset ||
            required_size > LIMIT_SIZE ||
            (destination_desc.Dimension == D3D12_RESOURCE_DIMENSION_BUFFER &&
                 (num_subresources != 1))
        {
            return Err(E_FAIL);
        }
        {
            let map = intermediate.map(0, None)?;
            for i in 0..num_subresources {
                if row_sizes_in_bytes[i] > LIMIT_SIZE {
                    return Err(E_FAIL);
                }
                let dest_data = D3D12_MEMCPY_DEST {
                    pData: map.offset(layouts[i].Offset as _),
                    RowPitch: layouts[i].Footprint.RowPitch as _,
                    SlicePitch: (layouts[i].Footprint.RowPitch * num_rows[i]) as
                        _,
                };
                memcpy_subresource(
                    &dest_data,
                    &src_data[i],
                    row_sizes_in_bytes[i] as _,
                    num_rows[i],
                    layouts[i].Footprint.Depth,
                );
            }

        }

        match destination_desc.Dimension {
            D3D12_RESOURCE_DIMENSION_BUFFER => {
                let src_box = D3D12_BOX::new(
                    (layouts[0].Offset) as _,
                    (layouts[0].Offset as u32 +
                         layouts[0].Footprint.Width),
                );
                self.copy_buffer_region(
                    destination_resource,
                    0,
                    intermediate,
                    layouts[0].Offset,
                    layouts[0].Footprint.Width as u64,
                );
            }
            _ => {
                for i in 0..num_subresources {
                    let dst = D3D12_TEXTURE_COPY_LOCATION::from_index(
                        destination_resource,
                        i as u32,
                    );
                    let src = D3D12_TEXTURE_COPY_LOCATION::from_footprint(
                        intermediate,
                        &layouts[i],
                    );
                    self.copy_texture_region(&dst, 0, 0, 0, &src, None);
                }
            }
        }
        Ok(required_size)
    }
}

//------------------------------------------------------------------------------------------------
// Row-by-row memcpy
#[inline]
fn memcpy_subresource(
    dst: &D3D12_MEMCPY_DEST,
    src: &D3D12_SUBRESOURCE_DATA,
    row_size_in_bytes: usize,
    num_rows: u32,
    num_slices: u32,
) {
    for z in 0..num_slices {
        let dst_slice = dst.offset_slice(z);
        let src_slice = src.offset_slice(z);
        for y in 0..num_rows {
            let dst_ptr = dst.ptr_offset(dst_slice + dst.offset_row(y));
            let src_ptr = src.ptr_offset(src_slice + dst.offset_row(y));
            unsafe { memcpy(dst_ptr, src_ptr, row_size_in_bytes) };
        }
    }
}
