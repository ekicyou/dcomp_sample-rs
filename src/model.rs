use winit::{Window, EventsLoop};
use winapi::shared::windef::HWND;
use winapi::shared::winerror::HRESULT;
use super::hwnd_window::HwndWindow;
use super::dx_api::*;

const FrameCount: u32 = 2;

impl HwndWindow for Window {
    fn hwnd(&self) -> HWND {
        unsafe {
            #[allow(deprecated)]
            let p = self.platform_window();
            p as HWND
        }
    }
}

pub struct DxModel {
    events_loop: EventsLoop,
    window: Window,
    device: ComRc<ID3D12Device>,
    command_queue: ComRc<ID3D12CommandQueue>,
    swap_chain: ComRc<IDXGISwapChain3>,
    dc_dev: ComRc<IDCompositionDevice>,
    dc_target: ComRc<IDCompositionTarget>,
    dc_visual: ComRc<IDCompositionVisual>,
    frame_index: u32,
    rtvHeap: ComRc<ID3D12DescriptorHeap>,
    srvHeap: ComRc<ID3D12DescriptorHeap>,
    rtvDescriptorSize: u32,
    renderTargets: Vec<ComRc<ID3D12Resource>>,
    command_allocator: ComRc<ID3D12CommandAllocator>,
}

impl DxModel {
    pub fn new(events_loop: EventsLoop, window: Window) -> Result<DxModel, HRESULT> {
        // window params
        let (width, height) = window.get_inner_size_pixels().unwrap_or_default();
        println!("width={}, height={}", width, height);
        let hwnd = window.hwnd();

        // Enable the D3D12 debug layer.
        #[cfg(build = "debug")]
        {
            let debugController = d3d12_get_debug_interface::<ID3D12Debug>()?;
            unsafe { debugController.EnableDebugLayer() }
        }
        let factory = create_dxgi_factory1::<IDXGIFactory4>()?;

        // d3d12デバイスの作成
        // ハードウェアデバイスが取得できなければ
        // WARPデバイスを取得する
        let device = factory.d3d12_create_best_device()?;

        // コマンドキューの作成
        let command_queue = {
            let desc = D3D12_COMMAND_QUEUE_DESC {
                Flags: D3D12_COMMAND_QUEUE_FLAG_NONE,
                Type: D3D12_COMMAND_LIST_TYPE_DIRECT,
                NodeMask: 0,
                Priority: 0,
            };
            device.create_command_queue::<ID3D12CommandQueue>(&desc)?
        };

        // swap chainの作成
        let swap_chain = {
            let desc = DXGI_SWAP_CHAIN_DESC1 {
                BufferCount: FrameCount,
                Width: width,
                Height: height,
                Format: DXGI_FORMAT_R8G8B8A8_UNORM,
                BufferUsage: DXGI_USAGE_RENDER_TARGET_OUTPUT,
                SwapEffect: DXGI_SWAP_EFFECT_FLIP_DISCARD,
                SampleDesc: DXGI_SAMPLE_DESC {
                    Count: 1,
                    Quality: 0,
                },
                AlphaMode: DXGI_ALPHA_MODE_PREMULTIPLIED,
                Flags: 0,
                Scaling: 0,
                Stereo: 0,
            };
            factory.create_swap_chain_for_composition(&command_queue, &desc)?
                .query_interface::<IDXGISwapChain3>()?
        };

        // DirectComposition 設定
        let dc_dev = dcomp_create_device::<IDCompositionDevice>(None)?;
        let dc_target = dc_dev.create_target_for_hwnd(hwnd, true)?;
        let dc_visual = dc_dev.create_visual()?;
        dc_visual.set_content(&swap_chain)?;
        dc_target.set_root(&dc_visual)?;
        dc_dev.commit()?;

        // This sample does not support fullscreen transitions.
        factory.make_window_association(hwnd, DXGI_MWA_NO_ALT_ENTER)?;
        let frame_index = swap_chain.get_current_back_buffer_index();

        // Create descriptor heaps.
        // Describe and create a render target view (RTV) descriptor heap.
        let rtvHeap = {
            let desc = D3D12_DESCRIPTOR_HEAP_DESC {
                NumDescriptors: FrameCount,
                Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
                NodeMask: 0,
            };
            device.create_descriptor_heap::<ID3D12DescriptorHeap>(&desc)?
        };

        // Describe and create a shader resource view (SRV) heap for the texture.
        let srvHeap = {
            let desc = D3D12_DESCRIPTOR_HEAP_DESC {
                NumDescriptors: 1,
                Type: D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
                Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
                NodeMask: 0,
            };
            device.create_descriptor_heap::<ID3D12DescriptorHeap>(&desc)?
        };
        let rtvDescriptorSize =
            device.get_descriptor_handle_increment_size(D3D12_DESCRIPTOR_HEAP_TYPE_RTV);

        // Create frame resources.
        let renderTargets = {
            let mut rtvHandle = rtvHeap.get_cpu_descriptor_handle_for_heap_start();
            let mut targets: Vec<ComRc<ID3D12Resource>> = Vec::with_capacity(FrameCount as usize);
            for n in 0..FrameCount {
                let target = swap_chain.get_buffer::<ID3D12Resource>(n)?;
                device.create_render_target_view(&target, None, rtvHandle);
                rtvHandle.offset(1, rtvDescriptorSize);
                targets.push(target);
            }
            targets
        };
        let command_allocator = device.create_command_allocator(D3D12_COMMAND_LIST_TYPE_DIRECT)?;

        //------------------------------------------------------------------
        // result
        //------------------------------------------------------------------
        Ok(DxModel {
               events_loop: events_loop,
               window: window,
               device: device,
               command_queue: command_queue,
               swap_chain: swap_chain,
               dc_dev: dc_dev,
               dc_target: dc_target,
               dc_visual: dc_visual,
               frame_index: frame_index,
               rtvHeap: rtvHeap,
               srvHeap: srvHeap,
               rtvDescriptorSize: rtvDescriptorSize,
               renderTargets: renderTargets,
               command_allocator: command_allocator,
           })
    }
    pub fn events_loop(&self) -> &EventsLoop {
        &self.events_loop
    }
    pub fn window(&self) -> &Window {
        &self.window
    }
}