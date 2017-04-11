#![allow(unused_unsafe)]
use super::dx_api::*;
use super::hwnd_window::HwndWindow;
use winapi::_core::mem;
use winapi::shared::windef::HWND;
use winapi::shared::winerror::HRESULT;
use winit::{Window, EventsLoop};

const FRAME_COUNT: u32 = 2;

impl HwndWindow for Window {
    fn hwnd(&self) -> HWND {
        unsafe {
            #[allow(deprecated)]
            let p = self.platform_window();
            p as HWND
        }
    }
}

#[allow(dead_code)]
pub struct DxModel {
    // Window
    events_loop: EventsLoop,
    window: Window,

    // D3D12 Targets
    device: ComRc<ID3D12Device>,
    command_queue: ComRc<ID3D12CommandQueue>,
    swap_chain: ComRc<IDXGISwapChain3>,
    dc_dev: ComRc<IDCompositionDevice>,
    dc_target: ComRc<IDCompositionTarget>,
    dc_visual: ComRc<IDCompositionVisual>,
    frame_index: u32,
    rtv_heap: ComRc<ID3D12DescriptorHeap>,
    srv_heap: ComRc<ID3D12DescriptorHeap>,
    rtv_descriptor_size: u32,
    render_targets: Vec<ComRc<ID3D12Resource>>,
    command_allocator: ComRc<ID3D12CommandAllocator>,

    // D3D12 Assets
    root_signature: ComRc<ID3D12RootSignature>,
}

impl DxModel {
    #[allow(dead_code)]
    pub fn events_loop(&self) -> &EventsLoop {
        &self.events_loop
    }
    #[allow(dead_code)]
    pub fn window(&self) -> &Window {
        &self.window
    }

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
                BufferCount: FRAME_COUNT,
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
            factory
                .create_swap_chain_for_composition(&command_queue, &desc)?
                .query_interface::<IDXGISwapChain3>()?
        };

        // DirectComposition 設定
        let dc_dev = dcomp_create_device::<IDCompositionDevice>(None)?;
        let dc_target = dc_dev.create_target_for_hwnd(hwnd, true)?;
        let dc_visual = dc_dev.create_visual()?;
        dc_visual.set_content(&swap_chain)?;
        dc_target.set_root(&dc_visual)?;
        dc_dev.commit()?;

        // このサンプルはフルスクリーンへの遷移をサポートしません。
        factory
            .make_window_association(hwnd, DXGI_MWA_NO_ALT_ENTER)?;
        let frame_index = swap_chain.get_current_back_buffer_index();

        // Create descriptor heaps.
        // Describe and create a render target view (RTV) descriptor heap.
        let rtv_heap = {
            let desc = D3D12_DESCRIPTOR_HEAP_DESC {
                NumDescriptors: FRAME_COUNT,
                Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
                NodeMask: 0,
            };
            device
                .create_descriptor_heap::<ID3D12DescriptorHeap>(&desc)?
        };

        // Describe and create a shader resource view (SRV) heap for the texture.
        let srv_heap = {
            let desc = D3D12_DESCRIPTOR_HEAP_DESC {
                NumDescriptors: 1,
                Type: D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
                Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
                NodeMask: 0,
            };
            device
                .create_descriptor_heap::<ID3D12DescriptorHeap>(&desc)?
        };
        let rtv_descriptor_size =
            device.get_descriptor_handle_increment_size(D3D12_DESCRIPTOR_HEAP_TYPE_RTV);

        // フレームバッファの作成
        let render_targets = {
            let mut rtv_handle = rtv_heap.get_cpu_descriptor_handle_for_heap_start();
            let mut targets: Vec<ComRc<ID3D12Resource>> = Vec::with_capacity(FRAME_COUNT as usize);
            for n in 0..FRAME_COUNT {
                let target = swap_chain.get_buffer::<ID3D12Resource>(n)?;
                device.create_render_target_view(&target, None, rtv_handle);
                rtv_handle.offset(1, rtv_descriptor_size);
                targets.push(target);
            }
            targets
        };
        // コマンドアロケータ
        let command_allocator = device
            .create_command_allocator(D3D12_COMMAND_LIST_TYPE_DIRECT)?;


        //------------------------------------------------------------------
        // LoadAssets(d3d12の描画初期化)
        //------------------------------------------------------------------

        // Create the root signature.
        let root_signature = {
            let ranges = {
                let range = D3D12_DESCRIPTOR_RANGE::new(D3D12_DESCRIPTOR_RANGE_TYPE_SRV, 1, 0);
                [range]
            };
            let root_parameters = {
                let a =
                    D3D12_ROOT_PARAMETER::new_constants(1, 0, 0, D3D12_SHADER_VISIBILITY_VERTEX);
                let b = D3D12_ROOT_PARAMETER::new_descriptor_table(&ranges,
                                                                   D3D12_SHADER_VISIBILITY_PIXEL);
                [a, b]
            };
            let samplers = unsafe {
                let mut sampler = mem::zeroed::<D3D12_STATIC_SAMPLER_DESC>();
                sampler.Filter = D3D12_FILTER_MIN_MAG_MIP_POINT;
                sampler.AddressU = D3D12_TEXTURE_ADDRESS_MODE_WRAP;
                sampler.AddressV = D3D12_TEXTURE_ADDRESS_MODE_WRAP;
                sampler.AddressW = D3D12_TEXTURE_ADDRESS_MODE_WRAP;
                sampler.MipLODBias = 0.0;
                sampler.MaxAnisotropy = 0;
                sampler.ComparisonFunc = D3D12_COMPARISON_FUNC_NEVER;
                sampler.BorderColor = D3D12_STATIC_BORDER_COLOR_TRANSPARENT_BLACK;
                sampler.MinLOD = 0.0;
                sampler.MaxLOD = D3D12_FLOAT32_MAX;
                sampler.ShaderRegister = 0;
                sampler.RegisterSpace = 0;
                sampler.ShaderVisibility = D3D12_SHADER_VISIBILITY_PIXEL;
                [sampler]
            };
            let desc= D3D12_ROOT_SIGNATURE_DESC::new(
		        &root_parameters, 
                &samplers, 
                D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT);
            let (signature, _error) = d3d12_serialize_root_signature(&desc,
                                                                     D3D_ROOT_SIGNATURE_VERSION_1)?;
            device
                .create_root_signature::<ID3D12RootSignature>(0,
                                                              signature.get_buffer_pointer(),
                                                              signature.get_buffer_size())?
        };

        // Create the pipeline state, which includes compiling and loading shaders.
        {
            let compileFlags: u32 = {
                #[cfg(debug)]
                {
                    D3DCOMPILE_DEBUG | D3DCOMPILE_SKIP_OPTIMIZATION
                }
                #[cfg(not(debug))]
                {
                    0
                }
            };
            let vertexShader = d3d_compile_from_file("shaders.hlsl",
                                                     None,
                                                     None,
                                                     "VSMain",
                                                     "vs_5_0",
                                                     compileFlags,
                                                     0)?;
            let pixelShader = d3d_compile_from_file("shaders.hlsl",
                                                    None,
                                                    None,
                                                    "PSMain",
                                                    "ps_5_0",
                                                    compileFlags,
                                                    0)?;

        };



        /*
	// Create the pipeline state, which includes compiling and loading shaders.
	{
		ComPtr<ID3DBlob> vertexShader;
		ComPtr<ID3DBlob> pixelShader;

#if defined(_DEBUG)
		// Enable better shader debugging with the graphics debugging tools.
		UINT compileFlags = D3DCOMPILE_DEBUG | D3DCOMPILE_SKIP_OPTIMIZATION;
#else
		UINT compileFlags = 0;
#endif

		ThrowIfFailed(D3DCompileFromFile(GetAssetFullPath(L"shaders.hlsl").c_str(), nullptr, nullptr, "VSMain", "vs_5_0", compileFlags, 0, &vertexShader, nullptr));
		ThrowIfFailed(D3DCompileFromFile(GetAssetFullPath(L"shaders.hlsl").c_str(), nullptr, nullptr, "PSMain", "ps_5_0", compileFlags, 0, &pixelShader, nullptr));

		// Define the vertex input layout.
		D3D12_INPUT_ELEMENT_DESC inputElementDescs[] =
		{
			{ "POSITION", 0, DXGI_FORMAT_R32G32B32_FLOAT, 0, 0, D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA, 0 },
			{ "TEXCOORD", 0, DXGI_FORMAT_R32G32_FLOAT, 0, 12, D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA, 0 }
		};

        const D3D12_BLEND_DESC AlphaBlend =
        {
            FALSE, // AlphaToCoverageEnable
            FALSE, // IndependentBlendEnable
            {
                TRUE, // BlendEnable
                FALSE, // LogicOpEnable
            D3D12_BLEND_ONE, // SrcBlend
            D3D12_BLEND_INV_SRC_ALPHA, // DestBlend
            D3D12_BLEND_OP_ADD, // BlendOp
            D3D12_BLEND_ONE, // SrcBlendAlpha
            D3D12_BLEND_INV_SRC_ALPHA, // DestBlendAlpha
            D3D12_BLEND_OP_ADD, // BlendOpAlpha
            D3D12_LOGIC_OP_CLEAR, // LogicOp
            D3D12_COLOR_WRITE_ENABLE_ALL // RenderTargetWriteMask
            }
        };
        
        // Describe and create the graphics pipeline state object (PSO).
		D3D12_GRAPHICS_PIPELINE_STATE_DESC psoDesc = {};
		psoDesc.InputLayout = { inputElementDescs, _countof(inputElementDescs) };
		psoDesc.pRootSignature = m_rootSignature.Get();
		psoDesc.VS = CD3DX12_SHADER_BYTECODE(vertexShader.Get());
		psoDesc.PS = CD3DX12_SHADER_BYTECODE(pixelShader.Get());
		psoDesc.RasterizerState = CD3DX12_RASTERIZER_DESC(D3D12_DEFAULT);
        psoDesc.RasterizerState.CullMode = D3D12_CULL_MODE_NONE;
        psoDesc.BlendState = AlphaBlend;
		psoDesc.DepthStencilState.DepthEnable = FALSE;
		psoDesc.DepthStencilState.StencilEnable = FALSE;
		psoDesc.SampleMask = UINT_MAX;
		psoDesc.PrimitiveTopologyType = D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE;
		psoDesc.NumRenderTargets = 1;
		psoDesc.RTVFormats[0] = DXGI_FORMAT_R8G8B8A8_UNORM;
		psoDesc.SampleDesc.Count = 1;
		ThrowIfFailed(m_device->CreateGraphicsPipelineState(&psoDesc, IID_PPV_ARGS(&m_pipelineState)));
	}

	// Create the command list.
	ThrowIfFailed(m_device->CreateCommandList(0, D3D12_COMMAND_LIST_TYPE_DIRECT, m_commandAllocator.Get(), m_pipelineState.Get(), IID_PPV_ARGS(&m_commandList)));

	// Create the vertex buffer.
	{
		// Define the geometry for a circle.
		Vertex triangleVertices[CircleSegments + 1] =
		{
			{ { 0.0f, 0.0f, 0.0f }, { 0.5f, 0.5f } }
		};

        for (UINT i = 0; i < CircleSegments; ++i)
        {
            float theta = 2  * DirectX::XM_PI * i / (float)(CircleSegments - 1);
            float x = sinf(theta);
            float y = cosf(theta);

            Vertex& v = triangleVertices[i + 1];
            v.position = DirectX::XMFLOAT3(x, y * m_aspectRatio, 0.0f);
            v.uv = DirectX::XMFLOAT2(x * 0.5f + 0.5f, y * 0.5f + 0.5f);
        }

		const UINT vertexBufferSize = sizeof(triangleVertices);

		// Note: using upload heaps to transfer static data like vert buffers is not 
		// recommended. Every time the GPU needs it, the upload heap will be marshalled 
		// over. Please read up on Default Heap usage. An upload heap is used here for 
		// code simplicity and because there are very few verts to actually transfer.
		ThrowIfFailed(m_device->CreateCommittedResource(
			&CD3DX12_HEAP_PROPERTIES(D3D12_HEAP_TYPE_UPLOAD),
			D3D12_HEAP_FLAG_NONE,
			&CD3DX12_RESOURCE_DESC::Buffer(vertexBufferSize),
			D3D12_RESOURCE_STATE_GENERIC_READ,
			nullptr,
			IID_PPV_ARGS(&m_vertexBuffer)));

		// Copy the triangle data to the vertex buffer.
		UINT8* pVertexDataBegin;
		CD3DX12_RANGE readRange(0, 0);		// We do not intend to read from this resource on the CPU.
		ThrowIfFailed(m_vertexBuffer->Map(0, &readRange, reinterpret_cast<void**>(&pVertexDataBegin)));
		memcpy(pVertexDataBegin, triangleVertices, sizeof(triangleVertices));
		m_vertexBuffer->Unmap(0, nullptr);

		// Initialize the vertex buffer view.
		m_vertexBufferView.BufferLocation = m_vertexBuffer->GetGPUVirtualAddress();
		m_vertexBufferView.StrideInBytes = sizeof(Vertex);
		m_vertexBufferView.SizeInBytes = vertexBufferSize;
	}

    // Create the index buffer
    {
        // Define the geometry for a circle.
        UINT16 triangleIndices[3 * CircleSegments];

        for (UINT i = 0; i < CircleSegments; ++i)
        {
            triangleIndices[i * 3 + 0] = 0;
            triangleIndices[i * 3 + 1] = 1 + i;
            triangleIndices[i * 3 + 2] = 2 + i;
        }

        const UINT indexBufferSize = sizeof(triangleIndices);

        // Note: using upload heaps to transfer static data like vert buffers is not 
        // recommended. Every time the GPU needs it, the upload heap will be marshalled 
        // over. Please read up on Default Heap usage. An upload heap is used here for 
        // code simplicity and because there are very few verts to actually transfer.
        ThrowIfFailed(m_device->CreateCommittedResource(
            &CD3DX12_HEAP_PROPERTIES(D3D12_HEAP_TYPE_UPLOAD),
            D3D12_HEAP_FLAG_NONE,
            &CD3DX12_RESOURCE_DESC::Buffer(indexBufferSize),
            D3D12_RESOURCE_STATE_GENERIC_READ,
            nullptr,
            IID_PPV_ARGS(&m_indexBuffer)));

        // Copy the index data to the index buffer.
        UINT8* pIndexDataBegin;
        CD3DX12_RANGE readRange(0, 0);		// We do not intend to read from this resource on the CPU.
        ThrowIfFailed(m_indexBuffer->Map(0, &readRange, reinterpret_cast<void**>(&pIndexDataBegin)));
        memcpy(pIndexDataBegin, triangleIndices, sizeof(triangleIndices));
        m_indexBuffer->Unmap(0, nullptr);

        // Intialize the index buffer view
        m_indexBufferView.BufferLocation = m_indexBuffer->GetGPUVirtualAddress();
        m_indexBufferView.Format = DXGI_FORMAT_R16_UINT;
        m_indexBufferView.SizeInBytes = indexBufferSize;
    }

	// Note: ComPtr's are CPU objects but this resource needs to stay in scope until
	// the command list that references it has finished executing on the GPU.
	// We will flush the GPU at the end of this method to ensure the resource is not
	// prematurely destroyed.
	ComPtr<ID3D12Resource> textureUploadHeap;

	// Create the texture.
	{
		// Describe and create a Texture2D.
		D3D12_RESOURCE_DESC textureDesc = {};
		textureDesc.MipLevels = 1;
		textureDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
		textureDesc.Width = TextureWidth;
		textureDesc.Height = TextureHeight;
		textureDesc.Flags = D3D12_RESOURCE_FLAG_NONE;
		textureDesc.DepthOrArraySize = 1;
		textureDesc.SampleDesc.Count = 1;
		textureDesc.SampleDesc.Quality = 0;
		textureDesc.Dimension = D3D12_RESOURCE_DIMENSION_TEXTURE2D;

		ThrowIfFailed(m_device->CreateCommittedResource(
			&CD3DX12_HEAP_PROPERTIES(D3D12_HEAP_TYPE_DEFAULT),
			D3D12_HEAP_FLAG_NONE,
			&textureDesc,
			D3D12_RESOURCE_STATE_COPY_DEST,
			nullptr,
			IID_PPV_ARGS(&m_texture)));

		const UINT64 uploadBufferSize = GetRequiredIntermediateSize(m_texture.Get(), 0, 1);

		// Create the GPU upload buffer.
		ThrowIfFailed(m_device->CreateCommittedResource(
			&CD3DX12_HEAP_PROPERTIES(D3D12_HEAP_TYPE_UPLOAD),
			D3D12_HEAP_FLAG_NONE,
			&CD3DX12_RESOURCE_DESC::Buffer(uploadBufferSize),
			D3D12_RESOURCE_STATE_GENERIC_READ,
			nullptr,
			IID_PPV_ARGS(&textureUploadHeap)));

		// Copy data to the intermediate upload heap and then schedule a copy 
		// from the upload heap to the Texture2D.
		std::vector<UINT8> texture = GenerateTextureData();

		D3D12_SUBRESOURCE_DATA textureData = {};
		textureData.pData = &texture[0];
		textureData.RowPitch = TextureWidth * sizeof(UINT);
		textureData.SlicePitch = textureData.RowPitch * TextureHeight;

		UpdateSubresources(m_commandList.Get(), m_texture.Get(), textureUploadHeap.Get(), 0, 0, 1, &textureData);
		m_commandList->ResourceBarrier(1, &CD3DX12_RESOURCE_BARRIER::Transition(m_texture.Get(), D3D12_RESOURCE_STATE_COPY_DEST, D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE));

		// Describe and create a SRV for the texture.
		D3D12_SHADER_RESOURCE_VIEW_DESC srvDesc = {};
		srvDesc.Shader4ComponentMapping = D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING;
		srvDesc.Format = textureDesc.Format;
		srvDesc.ViewDimension = D3D12_SRV_DIMENSION_TEXTURE2D;
		srvDesc.Texture2D.MipLevels = 1;
		m_device->CreateShaderResourceView(m_texture.Get(), &srvDesc, m_srvHeap->GetCPUDescriptorHandleForHeapStart());
	}
	
	// Close the command list and execute it to begin the initial GPU setup.
	ThrowIfFailed(m_commandList->Close());
	ID3D12CommandList* ppCommandLists[] = { m_commandList.Get() };
	m_commandQueue->ExecuteCommandLists(_countof(ppCommandLists), ppCommandLists);

	// Create synchronization objects and wait until assets have been uploaded to the GPU.
	{
		ThrowIfFailed(m_device->CreateFence(0, D3D12_FENCE_FLAG_NONE, IID_PPV_ARGS(&m_fence)));
		m_fenceValue = 1;

		// Create an event handle to use for frame synchronization.
		m_fenceEvent = CreateEvent(nullptr, FALSE, FALSE, nullptr);
		if (m_fenceEvent == nullptr)
		{
			ThrowIfFailed(HRESULT_FROM_WIN32(GetLastError()));
		}

		// Wait for the command list to execute; we are reusing the same command 
		// list in our main loop but for now, we just want to wait for setup to 
		// complete before continuing.
		WaitForPreviousFrame();
	}
}

*/



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
               rtv_heap: rtv_heap,
               srv_heap: srv_heap,
               rtv_descriptor_size: rtv_descriptor_size,
               render_targets: render_targets,
               command_allocator: command_allocator,
               root_signature: root_signature,
           })
    }



    /*
    // Load the rendering pipeline dependencies.

*/
}