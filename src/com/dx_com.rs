#![allow(unused_unsafe)]
#![allow(dead_code)]

use super::com_rc::*;
use super::dx_func::*;
use super::dx_pub_use::*;
use super::unsafe_util::*;
use winapi::Interface;
use winapi::_core::mem;
use winapi::_core::ptr::{self, null_mut};
use winapi::ctypes::c_void;
use winapi::shared::minwindef::UINT;
use winapi::shared::windef::HWND;
use winapi::shared::winerror::{E_FAIL, HRESULT};
use winapi::um::unknwnbase::IUnknown;


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
