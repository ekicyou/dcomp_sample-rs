#![allow(unused_unsafe)]
use winapi::Interface;
use winapi::shared::windef::HWND;
use winapi::_core::ptr::{self, null_mut};
use winapi::_core::mem;
use winapi::ctypes::c_void;
use winapi::shared::winerror::{HRESULT, E_FAIL};
use winapi::shared::minwindef::{BOOL, TRUE, FALSE, UINT, INT};
use winapi::um::unknwnbase::IUnknown;

pub use winapi::um::d3dcommon::*;
pub use winapi::shared::dxgitype::*;
pub use winapi::shared::dxgiformat::*;
pub use winapi::shared::dxgi::*;
pub use winapi::shared::dxgi1_2::*;
pub use winapi::shared::dxgi1_4::*;
pub use winapi::um::d3d12sdklayers::*;
pub use winapi::um::d3d12::*;
pub use winapi::um::dcomp::*;
pub use unsafe_api::*;
pub use com_rc::*;

#[allow(dead_code)]
pub const DXGI_MWA_NO_WINDOW_CHANGES: UINT = (1 << 0);
#[allow(dead_code)]
pub const DXGI_MWA_NO_ALT_ENTER: UINT = (1 << 1);
#[allow(dead_code)]
pub const DXGI_MWA_NO_PRINT_SCREEN: UINT = (1 << 2);
#[allow(dead_code)]
pub const DXGI_MWA_VALID: UINT = (0x7);


pub type ComResult<U> = Result<ComRc<U>, HRESULT>;

#[allow(non_snake_case)]
#[inline]
fn BOOL(flag: bool) -> BOOL {
    match flag {
        false => FALSE,
        true => TRUE,
    }
}

#[inline]
pub fn d3d12_create_device<U: Interface>(adapter: &IUnknown,
                                         minimum_feature_level: D3D_FEATURE_LEVEL)
                                         -> ComResult<U> {
    let riid = U::uuidof();
    let p = unsafe {
        let mut ppv: *mut c_void = null_mut();
        D3D12CreateDevice(adapter, minimum_feature_level, &riid, &mut ppv).hr()?;
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
    let src: *const IUnknown = match dxgi_device {
        Some(a) => a,
        None => ptr::null(),
    };
    let riid = U::uuidof();
    let p = unsafe {
        let mut ppv: *mut c_void = null_mut();
        DCompositionCreateDevice3(src, &riid, &mut ppv).hr()?;
        ppv as *const U
    };
    Ok(ComRc::new(p))
}

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
            self.CreateSwapChainForComposition(to_mut_ref(device), desc, ptr::null_mut(), &mut p)
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
        let p_desc: *const D3D12_RENDER_TARGET_VIEW_DESC = match desc {
            Some(p) => p,
            _ => ptr::null(),
        };
        unsafe {
            let p_resource = to_mut_ref(resource);
            self.CreateRenderTargetView(p_resource, p_desc, dest_descriptor)
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
