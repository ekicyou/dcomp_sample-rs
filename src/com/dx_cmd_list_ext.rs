use super::com_rc::*;
use super::dx_cd3dx12::*;
use super::dx_com::*;
use super::dx_pub_use::*;
use super::dx_struct::*;
use super::unsafe_util::*;
use winapi::_core as core;
use winapi::_core::mem;
use winapi::_core::ptr;
use winapi::shared::winerror::{E_FAIL, HRESULT};

const LIMIT_SIZE: u64 = (core::isize::MAX as u64);


pub trait ID3D12GraphicsCommandListExt {
    fn set_graphics_root_descriptor_table(
        &self,
        root_parameter_index: u32,
        base_descriptor: D3D12_GPU_DESCRIPTOR_HANDLE,
    ) -> ();
    fn set_graphics_root_u32_constant(
        &self,
        root_parameter_index: u32,
        src_data: u32,
        dest_offset_in_u32_values: u32,
    ) -> ();
    fn set_graphics_root_f32_constant(
        &self,
        root_parameter_index: u32,
        src_data: f32,
        dest_offset_in_u32_values: u32,
    ) -> ();
    fn set_descriptor_heaps(&self, heaps: &[&ID3D12DescriptorHeap]) -> ();
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
    fn set_graphics_root_signature(
        &self,
        root_signature: &ID3D12RootSignature,
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
    fn reset(
        &self,
        allocator: &ID3D12CommandAllocator,
        initial_state: &ID3D12PipelineState,
    ) -> Result<(), HRESULT>;
    fn rs_set_viewports(&self, viewports: &[D3D12_VIEWPORT]) -> ();
    fn rs_set_scissor_rects(&self, rects: &[D3D12_RECT]) -> ();
    fn om_set_render_targets(
        &self,
        render_target_descriptors: &[D3D12_CPU_DESCRIPTOR_HANDLE],
        rts_single_handle_to_descriptor_range: bool,
        depth_stencil_descriptor: Option<D3D12_CPU_DESCRIPTOR_HANDLE>,
    ) -> ();
    fn clear_render_target_view(
        &self,
        render_target_view: D3D12_CPU_DESCRIPTOR_HANDLE,
        rgba: &[f32; 4],
        rects: &[D3D12_RECT],
    ) -> ();
    fn ia_set_primitive_topology(
        &self,
        primitive_topology: D3D12_PRIMITIVE_TOPOLOGY,
    ) -> ();
    fn ia_set_vertex_buffers(
        &self,
        start_slot: u32,
        views: &[D3D12_VERTEX_BUFFER_VIEW],
    ) -> ();
    fn ia_set_index_buffer(&self, view: &D3D12_INDEX_BUFFER_VIEW) -> ();
    fn draw_indexed_instanced(
        &self,
        index_count_per_instance: u32,
        instance_count: u32,
        start_index_location: u32,
        base_vertex_location: i32,
        start_instance_location: u32,
    ) -> ();
}

impl ID3D12GraphicsCommandListExt for ID3D12GraphicsCommandList {
    #[inline]
    fn draw_indexed_instanced(
        &self,
        index_count_per_instance: u32,
        instance_count: u32,
        start_index_location: u32,
        base_vertex_location: i32,
        start_instance_location: u32,
    ) -> () {
        unsafe {
            self.DrawIndexedInstanced(
                index_count_per_instance,
                instance_count,
                start_index_location,
                base_vertex_location,
                start_instance_location,
            )
        }
    }
    #[inline]
    fn ia_set_index_buffer(&self, view: &D3D12_INDEX_BUFFER_VIEW) -> () {
        unsafe {
            let p = view as *const _;
            self.IASetIndexBuffer(p)
        }
    }
    #[inline]
    fn ia_set_vertex_buffers(
        &self,
        start_slot: u32,
        views: &[D3D12_VERTEX_BUFFER_VIEW],
    ) -> () {
        unsafe {
            let num = views.len() as _;
            let p = views.as_ptr() as *const _;
            self.IASetVertexBuffers(start_slot, num, p)
        }
    }
    #[inline]
    fn ia_set_primitive_topology(
        &self,
        primitive_topology: D3D12_PRIMITIVE_TOPOLOGY,
    ) -> () {
        unsafe { self.IASetPrimitiveTopology(primitive_topology) }
    }
    #[inline]
    fn clear_render_target_view(
        &self,
        render_target_view: D3D12_CPU_DESCRIPTOR_HANDLE,
        rgba: &[f32; 4],
        rects: &[D3D12_RECT],
    ) -> () {
        unsafe {
            let (num, p) = match rects.len() {
                0 => (0, ptr::null()),
                n => (n, rects.as_ptr() as *const _),
            };
            self.ClearRenderTargetView(render_target_view, rgba, num as _, p)
        }
    }
    #[inline]
    fn om_set_render_targets(
        &self,
        render_target_descriptors: &[D3D12_CPU_DESCRIPTOR_HANDLE],
        rts_single_handle_to_descriptor_range: bool,
        depth_stencil_descriptor: Option<D3D12_CPU_DESCRIPTOR_HANDLE>,
    ) -> () {
        unsafe {
            let num = render_target_descriptors.len() as _;
            let p1 = render_target_descriptors.as_ptr() as *const _;
            let p2 = match depth_stencil_descriptor {
                None => ptr::null(),
                Some(a) => &a as *const _,
            };
            self.OMSetRenderTargets(
                num,
                p1,
                BOOL(rts_single_handle_to_descriptor_range),
                p2,
            )
        }
    }
    #[inline]
    fn rs_set_viewports(&self, viewports: &[D3D12_VIEWPORT]) -> () {
        unsafe {
            let num = viewports.len() as u32;
            let ptr = viewports.as_ptr() as *const _;
            self.RSSetViewports(num, ptr)
        }
    }
    #[inline]
    fn rs_set_scissor_rects(&self, rects: &[D3D12_RECT]) -> () {
        unsafe {
            let num = rects.len() as u32;
            let ptr = rects.as_ptr() as *const _;
            self.RSSetScissorRects(num, ptr)
        }
    }
    #[inline]
    fn set_graphics_root_descriptor_table(
        &self,
        root_parameter_index: u32,
        base_descriptor: D3D12_GPU_DESCRIPTOR_HANDLE,
    ) -> () {
        unsafe {
            self.SetGraphicsRootDescriptorTable(
                root_parameter_index,
                base_descriptor,
            )
        }
    }
    #[inline]
    fn set_graphics_root_u32_constant(
        &self,
        root_parameter_index: u32,
        src_data: u32,
        dest_offset_in_u32_values: u32,
    ) -> () {
        unsafe {
            self.SetGraphicsRoot32BitConstant(
                root_parameter_index,
                src_data,
                dest_offset_in_u32_values,
            )
        }
    }
    #[inline]
    fn set_graphics_root_f32_constant(
        &self,
        root_parameter_index: u32,
        src_data: f32,
        dest_offset_in_u32_values: u32,
    ) -> () {
        let src_data = unsafe { mem::transmute(src_data) };
        self.set_graphics_root_u32_constant(
            root_parameter_index,
            src_data,
            dest_offset_in_u32_values,
        )
    }
    #[inline]
    fn set_descriptor_heaps(&self, heaps: &[&ID3D12DescriptorHeap]) -> () {
        unsafe {
            let num = heaps.len() as u32;
            let ptr = heaps.as_ptr() as *mut *mut ID3D12DescriptorHeap;
            self.SetDescriptorHeaps(num, ptr)
        }
    }
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
    #[inline]
    fn reset(
        &self,
        allocator: &ID3D12CommandAllocator,
        initial_state: &ID3D12PipelineState,
    ) -> Result<(), HRESULT> {
        unsafe {
            self.Reset(
                allocator as *const _ as *mut _,
                initial_state as *const _ as *mut _,
            ).hr()
        }
    }
    #[inline]
    fn set_graphics_root_signature(
        &self,
        root_signature: &ID3D12RootSignature,
    ) -> () {
        unsafe {
            self.SetGraphicsRootSignature(root_signature as *const _ as *mut _)
        }
    }

    // サブリソースをヒープにコピーします。
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
                // ??
                //let src_box = D3D12_BOX::new(
                //    (layouts[0].Offset) as _,
                //    (layouts[0].Offset as u32 +
                //         layouts[0].Footprint.Width),
                //);
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
