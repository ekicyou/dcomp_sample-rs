use winit::{Window, EventsLoop};
use winapi::shared::windef::HWND;
use winapi::shared::winerror::HRESULT;
use super::hwnd_window::HwndWindow;
use com_rc::*;
use super::dcomp_api::*;

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
    dc_dev: ComRc<IDCompositionDevice>,
    dc_target: ComRc<IDCompositionTarget>,
    dc_visual: ComRc<IDCompositionVisual>,
}

impl DxModel {
    pub fn new(events_loop: EventsLoop, window: Window) -> Result<DxModel, HRESULT> {
        let hwnd = window.hwnd();



        //------------------------------------------------------------------
        // Set up DirectComposition
        //------------------------------------------------------------------
        let dc_dev = create_device::<IDCompositionDevice>(None)?;
        let dc_target = dc_dev.create_target_for_hwnd(hwnd, true)?;
        let dc_visual = dc_dev.create_visual()?;


        //------------------------------------------------------------------
        // result
        //------------------------------------------------------------------
        Ok(DxModel {
               events_loop: events_loop,
               window: window,
               dc_dev: dc_dev,
               dc_target: dc_target,
               dc_visual: dc_visual,
           })
    }
    /*
// Load the rendering pipeline dependencies.
void DirectCompositeSample::LoadPipeline()
{
#if defined(_DEBUG)
	// Enable the D3D12 debug layer.
	{
		ComPtr<ID3D12Debug> debugController;
		if (SUCCEEDED(D3D12GetDebugInterface(IID_PPV_ARGS(&debugController))))
		{
			debugController->EnableDebugLayer();
		}
	}
#endif

	ComPtr<IDXGIFactory4> factory;
	ThrowIfFailed(CreateDXGIFactory1(IID_PPV_ARGS(&factory)));

	if (m_useWarpDevice)
	{
		ComPtr<IDXGIAdapter> warpAdapter;
		ThrowIfFailed(factory->EnumWarpAdapter(IID_PPV_ARGS(&warpAdapter)));

		ThrowIfFailed(D3D12CreateDevice(
			warpAdapter.Get(),
			D3D_FEATURE_LEVEL_11_0,
			IID_PPV_ARGS(&m_device)
			));
	}
	else
	{
		ComPtr<IDXGIAdapter1> hardwareAdapter;
		GetHardwareAdapter(factory.Get(), &hardwareAdapter);

		ThrowIfFailed(D3D12CreateDevice(
			hardwareAdapter.Get(),
			D3D_FEATURE_LEVEL_11_0,
			IID_PPV_ARGS(&m_device)
			));
	}

	// Describe and create the command queue.
	D3D12_COMMAND_QUEUE_DESC queueDesc = {};
	queueDesc.Flags = D3D12_COMMAND_QUEUE_FLAG_NONE;
	queueDesc.Type = D3D12_COMMAND_LIST_TYPE_DIRECT;

	ThrowIfFailed(m_device->CreateCommandQueue(&queueDesc, IID_PPV_ARGS(&m_commandQueue)));

	// Describe and create the swap chain.
	DXGI_SWAP_CHAIN_DESC1 swapChainDesc = {};
	swapChainDesc.BufferCount = FrameCount;
	swapChainDesc.Width = m_width;
	swapChainDesc.Height = m_height;
	swapChainDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
	swapChainDesc.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
	swapChainDesc.SwapEffect = DXGI_SWAP_EFFECT_FLIP_DISCARD;
	swapChainDesc.SampleDesc.Count = 1;
    swapChainDesc.AlphaMode = DXGI_ALPHA_MODE_PREMULTIPLIED;

	ComPtr<IDXGISwapChain1> swapChain;
	ThrowIfFailed(factory->CreateSwapChainForComposition(
		m_commandQueue.Get(),		// Swap chain needs the queue so that it can force a flush on it.
		&swapChainDesc,
		nullptr,
		&swapChain
		));

    //------------------------------------------------------------------
    // Set up DirectComposition
    //------------------------------------------------------------------
    
    // Create the DirectComposition device
    ThrowIfFailed(DCompositionCreateDevice(
        nullptr,
        IID_PPV_ARGS(m_dcompDevice.ReleaseAndGetAddressOf())));

    // Create a DirectComposition target associated with the window (pass in hWnd here)
    ThrowIfFailed(m_dcompDevice->CreateTargetForHwnd(
        Win32Application::GetHwnd(),
        true,
        m_dcompTarget.ReleaseAndGetAddressOf()));

    // Create a DirectComposition "visual"
    ThrowIfFailed(m_dcompDevice->CreateVisual(m_dcompVisual.ReleaseAndGetAddressOf()));

    // Associate the visual with the swap chain
    ThrowIfFailed(m_dcompVisual->SetContent(swapChain.Get()));

    // Set the visual as the root of the DirectComposition target's composition tree
    ThrowIfFailed(m_dcompTarget->SetRoot(m_dcompVisual.Get()));
    ThrowIfFailed(m_dcompDevice->Commit());

    //------------------------------------------------------------------
    // DirectComposition setup end
    //------------------------------------------------------------------


	// This sample does not support fullscreen transitions.
	ThrowIfFailed(factory->MakeWindowAssociation(Win32Application::GetHwnd(), DXGI_MWA_NO_ALT_ENTER));

	ThrowIfFailed(swapChain.As(&m_swapChain));
	m_frameIndex = m_swapChain->GetCurrentBackBufferIndex();

	// Create descriptor heaps.
	{
		// Describe and create a render target view (RTV) descriptor heap.
		D3D12_DESCRIPTOR_HEAP_DESC rtvHeapDesc = {};
		rtvHeapDesc.NumDescriptors = FrameCount;
		rtvHeapDesc.Type = D3D12_DESCRIPTOR_HEAP_TYPE_RTV;
		rtvHeapDesc.Flags = D3D12_DESCRIPTOR_HEAP_FLAG_NONE;
		ThrowIfFailed(m_device->CreateDescriptorHeap(&rtvHeapDesc, IID_PPV_ARGS(&m_rtvHeap)));

		// Describe and create a shader resource view (SRV) heap for the texture.
		D3D12_DESCRIPTOR_HEAP_DESC srvHeapDesc = {};
		srvHeapDesc.NumDescriptors = 1;
		srvHeapDesc.Type = D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV;
		srvHeapDesc.Flags = D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE;
		ThrowIfFailed(m_device->CreateDescriptorHeap(&srvHeapDesc, IID_PPV_ARGS(&m_srvHeap)));

		m_rtvDescriptorSize = m_device->GetDescriptorHandleIncrementSize(D3D12_DESCRIPTOR_HEAP_TYPE_RTV);
	}

	// Create frame resources.
	{
		CD3DX12_CPU_DESCRIPTOR_HANDLE rtvHandle(m_rtvHeap->GetCPUDescriptorHandleForHeapStart());

		// Create a RTV for each frame.
		for (UINT n = 0; n < FrameCount; n++)
		{
			ThrowIfFailed(m_swapChain->GetBuffer(n, IID_PPV_ARGS(&m_renderTargets[n])));
			m_device->CreateRenderTargetView(m_renderTargets[n].Get(), nullptr, rtvHandle);
			rtvHandle.Offset(1, m_rtvDescriptorSize);
		}
	}

	ThrowIfFailed(m_device->CreateCommandAllocator(D3D12_COMMAND_LIST_TYPE_DIRECT, IID_PPV_ARGS(&m_commandAllocator)));
}
    */

    pub fn events_loop(&self) -> &EventsLoop {
        &self.events_loop
    }
    pub fn window(&self) -> &Window {
        &self.window
    }
}