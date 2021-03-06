#![allow(unused_unsafe)]
#![allow(dead_code)]

use super::com_rc::*;
use super::dx_func::*;
use super::dx_pub_use::*;
use super::unsafe_util::*;
use raw_window_handle::*;
use winapi::Interface;
use winapi::_core::mem;
use winapi::_core::ptr;
use winapi::ctypes::c_void;
use winapi::shared::ntdef::HANDLE;
use winapi::shared::winerror::{E_FAIL, HRESULT};
use winapi::um::unknwnbase::IUnknown;
use winapi::um::winbase::INFINITE;

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
    fn enum_adapters1(&self, index: u32) -> ComResult<IDXGIAdapter1>;
    fn create_swap_chain_for_composition(
        &self,
        device: &IUnknown,
        desc: &DXGI_SWAP_CHAIN_DESC1,
    ) -> ComResult<IDXGISwapChain1>;
    fn make_window_association(&self, hwnd: RawWindowHandle, flags: u32) -> Result<(), HRESULT>;
    fn d3d12_create_hardware_device(&self) -> ComResult<ID3D12Device>;
    fn d3d12_create_warp_device(&self) -> ComResult<ID3D12Device>;
    fn d3d12_create_best_device(&self) -> ComResult<ID3D12Device>;
}
impl IDXGIFactory4Ext for IDXGIFactory4 {
    #[inline]
    fn enum_warp_adapter<U: Interface>(&self) -> ComResult<U> {
        let riid = U::uuidof();
        let p = unsafe {
            let mut ppv: *mut c_void = ptr::null_mut();
            self.EnumWarpAdapter(&riid, &mut ppv).hr()?;
            ppv as *const U
        };
        Ok(ComRc::new(p))
    }
    #[inline]
    fn enum_adapters1(&self, index: u32) -> ComResult<IDXGIAdapter1> {
        unsafe {
            let mut p = ptr::null_mut();
            self.EnumAdapters1(index, &mut p).hr()?;
            Ok(ComRc::new(p))
        }
    }
    #[inline]
    fn create_swap_chain_for_composition(
        &self,
        device: &IUnknown,
        desc: &DXGI_SWAP_CHAIN_DESC1,
    ) -> ComResult<IDXGISwapChain1> {
        unsafe {
            let mut p = ptr::null_mut();
            self.CreateSwapChainForComposition(to_mut_ptr(device), desc, ptr::null_mut(), &mut p)
                .hr()?;
            Ok(ComRc::new(p))
        }
    }
    #[inline]
    fn make_window_association(&self, hwnd: RawWindowHandle, flags: u32) -> Result<(), HRESULT> {
        unsafe { self.MakeWindowAssociation(HWND(hwnd), flags).hr() }
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
    fn create_descriptor_heap<U: Interface>(
        &self,
        desc: &D3D12_DESCRIPTOR_HEAP_DESC,
    ) -> ComResult<U>;
    fn get_descriptor_handle_increment_size(
        &self,
        descriptor_heap_type: D3D12_DESCRIPTOR_HEAP_TYPE,
    ) -> u32;
    fn create_render_target_view(
        &self,
        resource: &ID3D12Resource,
        desc: Option<&D3D12_RENDER_TARGET_VIEW_DESC>,
        dest_descriptor: D3D12_CPU_DESCRIPTOR_HANDLE,
    ) -> ();
    fn create_command_allocator<U: Interface>(
        &self,
        type_: D3D12_COMMAND_LIST_TYPE,
    ) -> ComResult<U>;
    fn create_root_signature<U: Interface>(
        &self,
        node_mask: u32,
        blob_with_root_signature: *const c_void,
        blob_length_in_bytes: usize,
    ) -> ComResult<U>;
    fn create_graphics_pipeline_state<U: Interface>(
        &self,
        desc: &D3D12_GRAPHICS_PIPELINE_STATE_DESC,
    ) -> ComResult<U>;
    fn create_command_list<U: Interface>(
        &self,
        node_mask: u32,
        list_type: D3D12_COMMAND_LIST_TYPE,
        command_allocator: &ID3D12CommandAllocator,
        initial_state: &ID3D12PipelineState,
    ) -> ComResult<U>;
    fn create_committed_resource<U: Interface>(
        &self,
        heap_properties: &D3D12_HEAP_PROPERTIES,
        heap_flags: D3D12_HEAP_FLAGS,
        resource_desc: &D3D12_RESOURCE_DESC,
        initial_resource_state: D3D12_RESOURCE_STATES,
        optimized_clear_value: Option<&D3D12_CLEAR_VALUE>,
    ) -> ComResult<U>;
    fn create_shader_resource_view(
        &self,
        resource: &ID3D12Resource,
        desc: &D3D12_SHADER_RESOURCE_VIEW_DESC,
        dest_descriptor: D3D12_CPU_DESCRIPTOR_HANDLE,
    ) -> ();
    fn create_fence<U: Interface>(
        &self,
        initial_value: u64,
        flags: D3D12_FENCE_FLAGS,
    ) -> ComResult<U>;
    fn get_copyable_footprints(
        &self,
        resource_desc: &D3D12_RESOURCE_DESC,
        num_subresources: usize,
        base_offset: usize,
    ) -> (
        Box<[D3D12_PLACED_SUBRESOURCE_FOOTPRINT]>,
        Box<[u32]>,
        Box<[u64]>,
        u64,
    );
}
impl ID3D12DeviceExt for ID3D12Device {
    #[inline]
    fn create_command_queue<U: Interface>(&self, desc: &D3D12_COMMAND_QUEUE_DESC) -> ComResult<U> {
        let riid = U::uuidof();
        let p = unsafe {
            let mut ppv: *mut c_void = ptr::null_mut();
            self.CreateCommandQueue(desc, &riid, &mut ppv).hr()?;
            ppv as *const U
        };
        Ok(ComRc::new(p))
    }
    #[inline]
    fn create_descriptor_heap<U: Interface>(
        &self,
        desc: &D3D12_DESCRIPTOR_HEAP_DESC,
    ) -> ComResult<U> {
        let riid = U::uuidof();
        let p = unsafe {
            let mut ppv: *mut c_void = ptr::null_mut();
            self.CreateDescriptorHeap(desc, &riid, &mut ppv).hr()?;
            ppv as *const U
        };
        Ok(ComRc::new(p))
    }
    #[inline]
    fn get_descriptor_handle_increment_size(
        &self,
        descriptor_heap_type: D3D12_DESCRIPTOR_HEAP_TYPE,
    ) -> u32 {
        unsafe { self.GetDescriptorHandleIncrementSize(descriptor_heap_type) }
    }
    #[inline]
    fn create_render_target_view(
        &self,
        resource: &ID3D12Resource,
        desc: Option<&D3D12_RENDER_TARGET_VIEW_DESC>,
        dest_descriptor: D3D12_CPU_DESCRIPTOR_HANDLE,
    ) -> () {
        unsafe {
            let p_resource = to_mut_ptr(resource);
            self.CreateRenderTargetView(p_resource, opt_to_ptr(desc), dest_descriptor)
        }
    }
    #[inline]
    fn create_command_allocator<U: Interface>(
        &self,
        type_: D3D12_COMMAND_LIST_TYPE,
    ) -> ComResult<U> {
        let riid = U::uuidof();
        let p = unsafe {
            let mut ppv: *mut c_void = ptr::null_mut();
            self.CreateCommandAllocator(type_, &riid, &mut ppv).hr()?;
            ppv as *const U
        };
        Ok(ComRc::new(p))
    }
    #[inline]
    fn create_root_signature<U: Interface>(
        &self,
        node_mask: u32,
        blob_with_root_signature: *const c_void,
        blob_length_in_bytes: usize,
    ) -> ComResult<U> {
        let riid = U::uuidof();
        let p = unsafe {
            let mut ppv: *mut c_void = ptr::null_mut();
            self.CreateRootSignature(
                node_mask,
                blob_with_root_signature,
                blob_length_in_bytes,
                &riid,
                &mut ppv,
            )
            .hr()?;
            ppv as *const U
        };
        Ok(ComRc::new(p))
    }
    #[inline]
    fn create_graphics_pipeline_state<U: Interface>(
        &self,
        desc: &D3D12_GRAPHICS_PIPELINE_STATE_DESC,
    ) -> ComResult<U> {
        let riid = U::uuidof();
        let p = unsafe {
            let mut ppv: *mut c_void = ptr::null_mut();
            self.CreateGraphicsPipelineState(desc, &riid, &mut ppv)
                .hr()?;
            ppv as *const U
        };
        Ok(ComRc::new(p))
    }
    #[inline]
    fn create_command_list<U: Interface>(
        &self,
        node_mask: u32,
        list_type: D3D12_COMMAND_LIST_TYPE,
        command_allocator: &ID3D12CommandAllocator,
        initial_state: &ID3D12PipelineState,
    ) -> ComResult<U> {
        let riid = U::uuidof();
        let p = unsafe {
            let mut ppv: *mut c_void = ptr::null_mut();
            self.CreateCommandList(
                node_mask,
                list_type,
                to_mut_ptr(command_allocator),
                to_mut_ptr(initial_state),
                &riid,
                &mut ppv,
            )
            .hr()?;
            ppv as *const U
        };
        Ok(ComRc::new(p))
    }
    #[inline]
    fn create_committed_resource<U: Interface>(
        &self,
        heap_properties: &D3D12_HEAP_PROPERTIES,
        heap_flags: D3D12_HEAP_FLAGS,
        resource_desc: &D3D12_RESOURCE_DESC,
        initial_resource_state: D3D12_RESOURCE_STATES,
        optimized_clear_value: Option<&D3D12_CLEAR_VALUE>,
    ) -> ComResult<U> {
        let riid = U::uuidof();
        let p = unsafe {
            let mut ppv: *mut c_void = ptr::null_mut();
            self.CreateCommittedResource(
                heap_properties,
                heap_flags,
                resource_desc,
                initial_resource_state,
                opt_to_ptr(optimized_clear_value),
                &riid,
                &mut ppv,
            )
            .hr()?;
            ppv as *const U
        };
        Ok(ComRc::new(p))
    }
    #[inline]
    fn create_shader_resource_view(
        &self,
        resource: &ID3D12Resource,
        desc: &D3D12_SHADER_RESOURCE_VIEW_DESC,
        dest_descriptor: D3D12_CPU_DESCRIPTOR_HANDLE,
    ) -> () {
        unsafe {
            self.CreateShaderResourceView(resource as *const _ as *mut _, desc, dest_descriptor)
        }
    }
    #[inline]
    fn create_fence<U: Interface>(
        &self,
        initial_value: u64,
        flags: D3D12_FENCE_FLAGS,
    ) -> ComResult<U> {
        let riid = U::uuidof();
        let p = unsafe {
            let mut ppv: *mut c_void = ptr::null_mut();
            self.CreateFence(initial_value, flags, &riid, &mut ppv)
                .hr()?;
            ppv as *const U
        };
        Ok(ComRc::new(p))
    }
    #[inline]
    fn get_copyable_footprints(
        &self,
        resource_desc: &D3D12_RESOURCE_DESC,
        num_subresources: usize,
        base_offset: usize,
    ) -> (
        Box<[D3D12_PLACED_SUBRESOURCE_FOOTPRINT]>,
        Box<[u32]>,
        Box<[u64]>,
        u64,
    ) {
        let mut layouts = Vec::with_capacity(num_subresources);
        let mut num_rows = Vec::with_capacity(num_subresources);
        let mut row_size_in_bytes = Vec::with_capacity(num_subresources);
        let mut total_bytes = 0_u64;
        unsafe {
            layouts.set_len(num_subresources);
            num_rows.set_len(num_subresources);
            row_size_in_bytes.set_len(num_subresources);
            self.GetCopyableFootprints(
                resource_desc,
                0,
                num_subresources as _,
                base_offset as _,
                layouts.as_mut_ptr(),
                num_rows.as_mut_ptr(),
                row_size_in_bytes.as_mut_ptr(),
                &mut total_bytes,
            );
        }
        (
            layouts.into_boxed_slice(),
            num_rows.into_boxed_slice(),
            row_size_in_bytes.into_boxed_slice(),
            total_bytes,
        )
    }
}

pub trait ID3D12CommandAllocatorExt {
    fn reset(&self) -> Result<(), HRESULT>;
}

impl ID3D12CommandAllocatorExt for ID3D12CommandAllocator {
    fn reset(&self) -> Result<(), HRESULT> {
        unsafe { self.Reset().hr() }
    }
}

pub trait IDCompositionDeviceExt {
    fn commit(&self) -> Result<(), HRESULT>;
    fn create_target_for_hwnd(
        &self,
        hwnd: RawWindowHandle,
        topmost: bool,
    ) -> ComResult<IDCompositionTarget>;
    fn create_visual(&self) -> ComResult<IDCompositionVisual>;
}
impl IDCompositionDeviceExt for IDCompositionDevice {
    #[inline]
    fn commit(&self) -> Result<(), HRESULT> {
        unsafe { self.Commit().hr() }
    }
    #[inline]
    fn create_target_for_hwnd(
        &self,
        hwnd: RawWindowHandle,
        topmost: bool,
    ) -> ComResult<IDCompositionTarget> {
        unsafe {
            let mut p: *mut IDCompositionTarget = ptr::null_mut();
            self.CreateTargetForHwnd(HWND(hwnd), BOOL(topmost), &mut p)
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

pub trait IDXGISwapChainExt {
    fn get_buffer<U: Interface>(&self, buffer: u32) -> ComResult<U>;
    fn present(&self, sync_interval: u32, flags: u32) -> Result<(), HRESULT>;
}
impl IDXGISwapChainExt for IDXGISwapChain {
    #[inline]
    fn get_buffer<U: Interface>(&self, buffer: u32) -> ComResult<U> {
        let riid = U::uuidof();
        let p = unsafe {
            let mut ppv: *mut c_void = ptr::null_mut();
            self.GetBuffer(buffer, &riid, &mut ppv).hr()?;
            ppv as *const U
        };
        Ok(ComRc::new(p))
    }
    #[inline]
    fn present(&self, sync_interval: u32, flags: u32) -> Result<(), HRESULT> {
        unsafe { self.Present(sync_interval, flags).hr() }
    }
}

pub trait IDXGISwapChain3Ext {
    fn get_current_back_buffer_index(&self) -> u32;
}
impl IDXGISwapChain3Ext for IDXGISwapChain3 {
    #[inline]
    fn get_current_back_buffer_index(&self) -> u32 {
        unsafe { self.GetCurrentBackBufferIndex() }
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
        unsafe { self.GetDesc() }
    }
    #[inline]
    fn get_cpu_descriptor_handle_for_heap_start(&self) -> D3D12_CPU_DESCRIPTOR_HANDLE {
        unsafe { self.GetCPUDescriptorHandleForHeapStart() }
    }
    #[inline]
    fn get_gpu_descriptor_handle_for_heap_start(&self) -> D3D12_GPU_DESCRIPTOR_HANDLE {
        unsafe { self.GetGPUDescriptorHandleForHeapStart() }
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

pub trait ID3D12ResourceExt {
    fn map(
        &self,
        subresource: u32,
        read_range: Option<&D3D12_RANGE>,
    ) -> Result<ResourceMap, HRESULT>;
    fn get_gpu_virtual_address(&self) -> D3D12_GPU_VIRTUAL_ADDRESS;
    fn get_desc(&self) -> D3D12_RESOURCE_DESC;
    fn get_device<U: Interface>(&self) -> ComResult<U>;
    fn get_required_intermediate_size(
        &self,
        first_subresource: u32,
        num_subresources: u32,
    ) -> Result<u64, HRESULT>;
}
impl ID3D12ResourceExt for ID3D12Resource {
    #[inline]
    fn map(
        &self,
        subresource: u32,
        read_range: Option<&D3D12_RANGE>,
    ) -> Result<ResourceMap, HRESULT> {
        ResourceMap::new(self, subresource, read_range)
    }
    #[inline]
    fn get_gpu_virtual_address(&self) -> D3D12_GPU_VIRTUAL_ADDRESS {
        unsafe { self.GetGPUVirtualAddress() }
    }
    #[inline]
    fn get_desc(&self) -> D3D12_RESOURCE_DESC {
        unsafe { self.GetDesc() }
    }
    #[inline]
    fn get_device<U: Interface>(&self) -> ComResult<U> {
        let riid = U::uuidof();
        let p = unsafe {
            let mut ppv: *mut c_void = ptr::null_mut();
            self.GetDevice(&riid, &mut ppv).hr()?;
            ppv as *const U
        };
        Ok(ComRc::new(p))
    }
    #[inline]
    fn get_required_intermediate_size(
        &self,
        first_subresource: u32,
        num_subresources: u32,
    ) -> Result<u64, HRESULT> {
        let mut required_size: u64 = 0;
        let device = self.get_device::<ID3D12Device>()?;
        let desc = self.get_desc();
        unsafe {
            device.GetCopyableFootprints(
                &desc,
                first_subresource,
                num_subresources,
                0,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                &mut required_size,
            );
            println!("GetCopyableFootprints");
        }
        Ok(required_size)
    }
}

pub struct ResourceMap<'a> {
    resource: &'a ID3D12Resource,
    subresource: u32,
    data_begin: *mut c_void,
}
impl<'a> ResourceMap<'a> {
    #[inline]
    fn new(
        resource: &'a ID3D12Resource,
        subresource: u32,
        read_range: Option<&D3D12_RANGE>,
    ) -> Result<ResourceMap<'a>, HRESULT> {
        let mut data_begin: *mut c_void = ptr::null_mut();
        unsafe {
            resource
                .Map(subresource, opt_to_ptr(read_range), &mut data_begin)
                .hr()?;
            Ok(ResourceMap {
                resource: resource,
                subresource: subresource,
                data_begin: data_begin,
            })
        }
    }
    #[inline]
    pub fn offset(&self, offset: usize) -> *mut c_void {
        unsafe {
            let mut a: usize = mem::transmute(self.data_begin);
            a += offset;
            mem::transmute(a)
        }
    }
    #[inline]
    pub fn memcpy<T>(&self, src: *const T, size: usize) {
        let dst = self.data_begin as *mut u8;
        let src = src as *const u8;
        unsafe { memcpy(dst, src, size) };
    }
    #[inline]
    pub fn memcpy_subresource<T>(
        &self,
        dst_offset: usize,
        dst_row_pitch: usize,
        dst_slice_pitch: usize,
        src: &D3D12_SUBRESOURCE_DATA,
        row_size_in_bytes: usize,
        num_rows: usize,
        num_slices: usize,
    ) {
        for z in 0_usize..num_slices {
            let dst_slice = (self.data_begin as usize) + dst_offset + dst_slice_pitch * z;
            let src_slice = (src.pData as usize) + (src.SlicePitch as usize) * z;
            for y in 0_usize..num_rows {
                let dst = dst_slice + dst_row_pitch * y;
                let src = src_slice + (src.RowPitch as usize) * y;
                unsafe {
                    let dst = dst as *const u8 as *mut u8;
                    let src = src as *const u8;
                    unsafe { memcpy(dst, src, row_size_in_bytes) };
                }
            }
        }
    }
}
impl<'a> Drop for ResourceMap<'a> {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            self.resource.Unmap(self.subresource, ptr::null());
        }
    }
}

pub trait ID3D12CommandQueueExt {
    fn execute_command_lists(&self, command_lists: &[&ID3D12GraphicsCommandList]);
    fn signal(&self, fence: &ID3D12Fence, value: u64) -> Result<(), HRESULT>;
}
impl ID3D12CommandQueueExt for ID3D12CommandQueue {
    #[inline]
    fn execute_command_lists(&self, command_lists: &[&ID3D12GraphicsCommandList]) {
        unsafe {
            let num = command_lists.len() as u32;
            let ptr = command_lists.as_ptr() as *mut *mut ID3D12CommandList;
            self.ExecuteCommandLists(num, ptr)
        }
    }
    #[inline]
    fn signal(&self, fence: &ID3D12Fence, value: u64) -> Result<(), HRESULT> {
        unsafe { self.Signal(fence as *const _ as *mut _, value).hr() }
    }
}

pub trait ID3D12FenceExt {
    fn get_completed_value(&self) -> u64;
    fn set_event_on_completion(&self, value: u64, event: HANDLE) -> Result<(), HRESULT>;
    fn signal(&self, value: u64) -> Result<(), HRESULT>;
    fn wait_infinite(&self, value: u64, event: HANDLE) -> Result<(), HRESULT>;
}
impl ID3D12FenceExt for ID3D12Fence {
    #[inline]
    fn get_completed_value(&self) -> u64 {
        unsafe { self.GetCompletedValue() }
    }
    #[inline]
    fn set_event_on_completion(&self, value: u64, event: HANDLE) -> Result<(), HRESULT> {
        unsafe { self.SetEventOnCompletion(value, event).hr() }
    }
    #[inline]
    fn signal(&self, value: u64) -> Result<(), HRESULT> {
        unsafe { self.Signal(value).hr() }
    }
    #[inline]
    fn wait_infinite(&self, value: u64, event: HANDLE) -> Result<(), HRESULT> {
        if self.get_completed_value() < value {
            self.set_event_on_completion(value, event)?;
            wait_for_single_object(event, INFINITE);
        }
        Ok(())
    }
}
