#![allow(unused_unsafe)]
use super::com::*;
use super::consts::*;
use texture::*;
use winapi::_core::f32::consts::PI;
use winapi::_core::mem;
use winapi::shared::basetsd::UINT16;
use winapi::shared::minwindef::{FALSE, TRUE};
use winapi::shared::ntdef::HANDLE;
use winapi::shared::windef::HWND;
use winapi::shared::winerror::HRESULT;
use winapi::vc::limits::UINT_MAX;
use winit::Window;
use winit::os::windows::WindowExt;

struct ArrayIterator3<T> {
    item: [T; 3],
    index: usize,
}
impl<T: Copy> ArrayIterator3<T> {
    pub fn new(item: [T; 3]) -> ArrayIterator3<T> {
        ArrayIterator3 {
            item: item,
            index: 0,
        }
    }
}
impl<T: Copy> Iterator for ArrayIterator3<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        match self.index < self.item.len() {
            true => {
                let rc = self.item[self.index];
                self.index += 1;
                Some(rc)
            }
            false => None,
        }
    }
}

#[allow(dead_code)]
pub struct DxModel {
    // Window
    aspect_ratio: f32,

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
    pipeline_state: ComRc<ID3D12PipelineState>,
    command_list: ComRc<ID3D12GraphicsCommandList>,

    // App resources.
    vertex_buffer: ComRc<ID3D12Resource>,
    vertex_buffer_view: D3D12_VERTEX_BUFFER_VIEW,
    index_buffer: ComRc<ID3D12Resource>,
    index_buffer_view: D3D12_INDEX_BUFFER_VIEW,
    texture: ComRc<ID3D12Resource>,

    // Synchronization objects.
    fence: ComRc<ID3D12Fence>,
    fence_value: u64,
    fence_event: HANDLE,

    // Pipeline objects.
    rotation_radians: f32,
    viewport: D3D12_VIEWPORT,
    scissor_rect: D3D12_RECT,
}

impl DxModel {
    pub fn new(window: &Window) -> Result<DxModel, HRESULT> {
        // window params
        let (width, height) =
            window.get_inner_size_pixels().unwrap_or_default();
        println!("width={}, height={}", width, height);
        let hwnd = window.get_hwnd() as HWND;
        let aspect_ratio = (width as f32) / (height as f32);

        let viewport = D3D12_VIEWPORT {
            Width: width as _,
            Height: height as _,
            MaxDepth: 1.0_f32,
        };
        let scissor_rect = D3D12_RECT {
            right: width as _,
            bottom: height as _,
        };


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
        factory.make_window_association(hwnd, DXGI_MWA_NO_ALT_ENTER)?;
        let mut frame_index = swap_chain.get_current_back_buffer_index();

        // Create descriptor heaps.
        // Describe and create a render target view (RTV) descriptor heap.
        let rtv_heap = {
            let desc = D3D12_DESCRIPTOR_HEAP_DESC {
                NumDescriptors: FRAME_COUNT,
                Type: D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
                Flags: D3D12_DESCRIPTOR_HEAP_FLAG_NONE,
                NodeMask: 0,
            };
            device.create_descriptor_heap::<ID3D12DescriptorHeap>(&desc)?
        };

        // Describe and create a shader resource view (SRV) heap for the texture.
        let srv_heap = {
            let desc = D3D12_DESCRIPTOR_HEAP_DESC {
                NumDescriptors: 1,
                Type: D3D12_DESCRIPTOR_HEAP_TYPE_CBV_SRV_UAV,
                Flags: D3D12_DESCRIPTOR_HEAP_FLAG_SHADER_VISIBLE,
                NodeMask: 0,
            };
            device.create_descriptor_heap::<ID3D12DescriptorHeap>(&desc)?
        };
        let rtv_descriptor_size = device.get_descriptor_handle_increment_size(
            D3D12_DESCRIPTOR_HEAP_TYPE_RTV,
        );

        // フレームバッファの作成
        let render_targets = {
            let mut rtv_handle =
                rtv_heap.get_cpu_descriptor_handle_for_heap_start();
            let mut targets: Vec<ComRc<ID3D12Resource>> =
                Vec::with_capacity(FRAME_COUNT as usize);
            for n in 0..FRAME_COUNT {
                let target = swap_chain.get_buffer::<ID3D12Resource>(n)?;
                device.create_render_target_view(&target, None, rtv_handle);
                rtv_handle.offset(1, rtv_descriptor_size);
                targets.push(target);
            }
            targets
        };
        // コマンドアロケータ
        let command_allocator = device.create_command_allocator(
            D3D12_COMMAND_LIST_TYPE_DIRECT,
        )?;


        //------------------------------------------------------------------
        // LoadAssets(d3d12の描画初期化)
        //------------------------------------------------------------------

        // Create the root signature.
        let root_signature = {
            let ranges = {
                let range = D3D12_DESCRIPTOR_RANGE::new(
                    D3D12_DESCRIPTOR_RANGE_TYPE_SRV,
                    1,
                    0,
                );
                [range]
            };
            let root_parameters = {
                let a = D3D12_ROOT_PARAMETER::new_constants(
                    1,
                    0,
                    0,
                    D3D12_SHADER_VISIBILITY_VERTEX,
                );
                let b = D3D12_ROOT_PARAMETER::new_descriptor_table(
                    &ranges,
                    D3D12_SHADER_VISIBILITY_PIXEL,
                );
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
                sampler.BorderColor =
                    D3D12_STATIC_BORDER_COLOR_TRANSPARENT_BLACK;
                sampler.MinLOD = 0.0;
                sampler.MaxLOD = D3D12_FLOAT32_MAX;
                sampler.ShaderRegister = 0;
                sampler.RegisterSpace = 0;
                sampler.ShaderVisibility = D3D12_SHADER_VISIBILITY_PIXEL;
                [sampler]
            };
            let desc = D3D12_ROOT_SIGNATURE_DESC::new(
                &root_parameters,
                &samplers,
                D3D12_ROOT_SIGNATURE_FLAG_ALLOW_INPUT_ASSEMBLER_INPUT_LAYOUT,
            );
            let (signature, _error) = d3d12_serialize_root_signature(
                &desc,
                D3D_ROOT_SIGNATURE_VERSION_1,
            )?;
            device.create_root_signature::<ID3D12RootSignature>(
                0,
                signature.get_buffer_pointer(),
                signature.get_buffer_size(),
            )?
        };

        // Create the pipeline state, which includes compiling and loading shaders.
        let pipeline_state = {
            let flags: u32 = {
                #[cfg(debug)]
                {
                    D3DCOMPILE_DEBUG | D3DCOMPILE_SKIP_OPTIMIZATION
                }
                #[cfg(not(debug))]
                {
                    0
                }
            };
            let file = "resources\\shaders.hlsl";
            let (vertex_shader, _) = d3d_compile_from_file(
                file,
                None,
                None,
                "VSMain",
                "vs_5_0",
                flags,
                0,
            )?;
            let (pixel_shader, _) = d3d_compile_from_file(
                file,
                None,
                None,
                "PSMain",
                "ps_5_0",
                flags,
                0,
            )?;

            // Define the vertex input layout.
            let input_element_descs = {
                let a = D3D12_INPUT_ELEMENT_DESC::new(
                    *t::POSITION,
                    0,
                    DXGI_FORMAT_R32G32B32_FLOAT,
                    0,
                    0,
                    D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                    0,
                );
                let b = D3D12_INPUT_ELEMENT_DESC::new(
                    *t::TEXCOORD,
                    0,
                    DXGI_FORMAT_R32G32_FLOAT,
                    0,
                    12,
                    D3D12_INPUT_CLASSIFICATION_PER_VERTEX_DATA,
                    0,
                );
                [a, b]
            };
            let alpha_blend = {
                let mut desc: D3D12_BLEND_DESC = unsafe { mem::zeroed() };
                desc.AlphaToCoverageEnable = FALSE;
                desc.IndependentBlendEnable = FALSE;
                desc.RenderTarget[0] = D3D12_RENDER_TARGET_BLEND_DESC {
                    BlendEnable: TRUE,
                    LogicOpEnable: FALSE,
                    SrcBlend: D3D12_BLEND_ONE,
                    DestBlend: D3D12_BLEND_INV_SRC_ALPHA,
                    BlendOp: D3D12_BLEND_OP_ADD,
                    SrcBlendAlpha: D3D12_BLEND_ONE,
                    DestBlendAlpha: D3D12_BLEND_INV_SRC_ALPHA,
                    BlendOpAlpha: D3D12_BLEND_OP_ADD,
                    LogicOp: D3D12_LOGIC_OP_CLEAR,
                    RenderTargetWriteMask: D3D12_COLOR_WRITE_ENABLE_ALL as u8,
                };
                desc
            };

            // Describe and create the graphics pipeline state object (PSO).
            let pso_desc = {
                let mut desc: D3D12_GRAPHICS_PIPELINE_STATE_DESC = unsafe { mem::zeroed() };
                desc.InputLayout = input_element_descs.layout();
                desc.pRootSignature = to_mut_ptr(root_signature.as_ptr());
                desc.VS = D3D12_SHADER_BYTECODE::new(&vertex_shader);
                desc.PS = D3D12_SHADER_BYTECODE::new(&pixel_shader);
                desc.RasterizerState = D3D12_RASTERIZER_DESC::default();
                desc.RasterizerState.CullMode = D3D12_CULL_MODE_NONE;
                desc.BlendState = alpha_blend;
                desc.DepthStencilState.DepthEnable = FALSE;
                desc.DepthStencilState.StencilEnable = FALSE;
                desc.SampleMask = UINT_MAX;
                desc.PrimitiveTopologyType =
                    D3D12_PRIMITIVE_TOPOLOGY_TYPE_TRIANGLE;
                desc.NumRenderTargets = 1;
                desc.RTVFormats[0] = DXGI_FORMAT_R8G8B8A8_UNORM;
                desc.SampleDesc.Count = 1;
                desc
            };
            device.create_graphics_pipeline_state(&pso_desc)?
        };

        // Create the command list.
        let command_list = device
            .create_command_list::<ID3D12GraphicsCommandList>(
                0,
                D3D12_COMMAND_LIST_TYPE_DIRECT,
                &command_allocator,
                &pipeline_state,
            )?;

        // Create the vertex buffer.
        let (vertex_buffer, vertex_buffer_view) = {
            // Define the geometry for a circle.
            let items = (-1..CIRCLE_SEGMENTS)
                .map(|i| match i {
                    -1 => {
                        let pos = [0_f32, 0_f32, 0_f32];
                        let uv = [0.5_f32, 0.5_f32];
                        Vertex::new(pos, uv)
                    }
                    _ => {
                        let theta = PI * 2.0_f32 * (i as f32) /
                            (CIRCLE_SEGMENTS as f32);
                        let x = theta.sin();
                        let y = theta.cos();
                        let pos = [x, y * aspect_ratio, 0.0_f32];
                        let uv = [x * 0.5_f32 + 0.5_f32, y * 0.5_f32 + 0.5_f32];
                        Vertex::new(pos, uv)
                    }
                })
                .collect::<Vec<_>>();
            println!("{:?}", items);
            let size_of = mem::size_of::<Vertex>();
            let size = size_of * items.len();
            let p = items.as_ptr();

            // Note: using upload heaps to transfer static data like vert buffers is not
            // recommended. Every time the GPU needs it, the upload heap will be marshalled
            // over. Please read up on Default Heap usage. An upload heap is used here for
            // code simplicity and because there are very few verts to actually transfer.
            let properties = D3D12_HEAP_PROPERTIES::new(D3D12_HEAP_TYPE_UPLOAD);
            let desc = D3D12_RESOURCE_DESC::buffer(size as u64);
            let buffer = device.create_committed_resource::<ID3D12Resource>(
                &properties,
                D3D12_HEAP_FLAG_NONE,
                &desc,
                D3D12_RESOURCE_STATE_GENERIC_READ,
                None,
            )?;

            // Copy the triangle data to the vertex buffer.
            let read_range = D3D12_RANGE::new(0, 0); // We do not intend to read from this resource on the CPU.
            buffer.map(0, Some(&read_range))?.memcpy(p, size);

            // Initialize the vertex buffer view.
            let view = D3D12_VERTEX_BUFFER_VIEW {
                BufferLocation: buffer.get_gpu_virtual_address(),
                SizeInBytes: size as u32,
                StrideInBytes: size_of as u32,
            };

            (buffer, view)
        };

        // Create the index buffer
        let (index_buffer, index_buffer_view) = {
            // Define the geometry for a circle.
            let items = (0..CIRCLE_SEGMENTS)
                .map(|i| {
                    let a = 0 as UINT16;
                    let b = (1 + i) as UINT16;
                    let c = (2 + i) as UINT16;
                    [a, b, c]
                })
                .flat_map(|a| ArrayIterator3::new(a))
                .collect::<Vec<_>>();

            let size_of = mem::size_of::<UINT16>();
            let size = size_of * items.len();
            let p = items.as_ptr();

            // Note: using upload heaps to transfer static data like vert buffers is not
            // recommended. Every time the GPU needs it, the upload heap will be marshalled
            // over. Please read up on Default Heap usage. An upload heap is used here for
            // code simplicity and because there are very few verts to actually transfer.
            let properties = D3D12_HEAP_PROPERTIES::new(D3D12_HEAP_TYPE_UPLOAD);
            let desc = D3D12_RESOURCE_DESC::buffer(size as u64);
            let buffer = device.create_committed_resource::<ID3D12Resource>(
                &properties,
                D3D12_HEAP_FLAG_NONE,
                &desc,
                D3D12_RESOURCE_STATE_GENERIC_READ,
                None,
            )?;

            // Copy the index data to the index buffer.
            let read_range = D3D12_RANGE::new(0, 0); // We do not intend to read from this resource on the CPU.
            buffer.map(0, Some(&read_range))?.memcpy(p, size);

            // Intialize the index buffer view
            let view = D3D12_INDEX_BUFFER_VIEW {
                BufferLocation: buffer.get_gpu_virtual_address(),
                SizeInBytes: size as u32,
                Format: DXGI_FORMAT_R16_UINT,
            };

            (buffer, view)
        };

        // Create the texture.

        // Note: ComPtr's are CPU objects but this resource needs to stay in scope until
        // the command list that references it has finished executing on the GPU.
        // We will flush the GPU at the end of this method to ensure the resource is not
        // prematurely destroyed.

        // texture_upload_heapの開放タイミングがGPUへのフラッシュ後になるように
        // 所有権を関数スコープに追い出しておく
        let (_texture_upload_heap, texture) = {
            // Describe and create a Texture2D.
            let texture_desc = D3D12_RESOURCE_DESC::new(
                D3D12_RESOURCE_DIMENSION_TEXTURE2D,
                0,
                TEXTURE_WIDTH,
                TEXTURE_HEIGHT,
                1,
                1,
                DXGI_FORMAT_R8G8B8A8_UNORM,
                1,
                0,
                D3D12_TEXTURE_LAYOUT_UNKNOWN,
                D3D12_RESOURCE_FLAG_NONE,
            );

            let texture = {
                let properties =
                    D3D12_HEAP_PROPERTIES::new(D3D12_HEAP_TYPE_DEFAULT);
                device.create_committed_resource::<ID3D12Resource>(
                    &properties,
                    D3D12_HEAP_FLAG_NONE,
                    &texture_desc,
                    D3D12_RESOURCE_STATE_COPY_DEST,
                    None,
                )?
            };
            let upload_buffer_size =
                texture.get_required_intermediate_size(0, 1)?;

            // Create the GPU upload buffer.
            let texture_upload_heap = {
                let properties =
                    D3D12_HEAP_PROPERTIES::new(D3D12_HEAP_TYPE_UPLOAD);
                let desc = D3D12_RESOURCE_DESC::buffer(upload_buffer_size);
                device.create_committed_resource::<ID3D12Resource>(
                    &properties,
                    D3D12_HEAP_FLAG_NONE,
                    &desc,
                    D3D12_RESOURCE_STATE_GENERIC_READ,
                    None,
                )?
            };

            // Copy data to the intermediate upload heap and then schedule a copy
            // from the upload heap to the Texture2D.
            let texture_bytes = generate_texture_data();
            let texture_data = {
                let ptr = texture_bytes.as_ptr();
                let row_pitch =
                    ((TEXTURE_WIDTH as usize) * mem::size_of::<u32>()) as isize;
                let slice_pitch = row_pitch * (TEXTURE_HEIGHT as isize);
                [
                    D3D12_SUBRESOURCE_DATA {
                        pData: ptr as _,
                        RowPitch: row_pitch,
                        SlicePitch: slice_pitch,
                    },
                ]
            };
            let _ = command_list.update_subresources_as_heap(
                &texture,
                &texture_upload_heap,
                0,
                &texture_data,
            )?;
            {
                let barrier = D3D12_RESOURCE_BARRIER::transition(
                    &texture,
                    D3D12_RESOURCE_STATE_COPY_DEST,
                    D3D12_RESOURCE_STATE_PIXEL_SHADER_RESOURCE,
                );
                command_list.resource_barrier(1, &barrier);
            }

            // Describe and create a SRV for the texture.
            {
                let desc = unsafe {
                    let mut desc =
                        mem::zeroed::<D3D12_SHADER_RESOURCE_VIEW_DESC>();
                    desc.Shader4ComponentMapping =
                        D3D12_DEFAULT_SHADER_4_COMPONENT_MAPPING;
                    desc.Format = texture_desc.Format;
                    desc.ViewDimension = D3D12_SRV_DIMENSION_TEXTURE2D;
                    {
                        let mut t = desc.u.Texture2D_mut();
                        t.MipLevels = 1;
                    }
                    desc
                };
                device.create_shader_resource_view(
                    &texture,
                    &desc,
                    srv_heap.get_cpu_descriptor_handle_for_heap_start(),
                );
            }

            (texture_upload_heap, texture)
        };

        // Close the command list and execute it to begin the initial GPU setup.
        {
            command_list.close()?;
            let a: &ID3D12GraphicsCommandList = &command_list;
            command_queue.execute_command_lists(&[a]);
        }

        // Create synchronization objects and wait until assets have been uploaded to the GPU.
        let (fence, fence_value, fence_event) = {
            let fence =
                device.create_fence::<ID3D12Fence>(0, D3D12_FENCE_FLAG_NONE)?;
            let mut fence_value = 1_u64;

            // Create an event handle to use for frame synchronization.
            let fence_event = create_event(None, false, false, None)?;

            // Wait for the command list to execute; we are reusing the same command
            // list in our main loop but for now, we just want to wait for setup to
            // complete before continuing.
            wait_for_previous_frame(
                &swap_chain,
                &command_queue,
                &fence,
                fence_event,
                &mut fence_value,
                &mut frame_index,
            )?;

            (fence, fence_value, fence_event)
        };

        //------------------------------------------------------------------
        // result
        //------------------------------------------------------------------
        Ok(DxModel {
            aspect_ratio: aspect_ratio,
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
            pipeline_state: pipeline_state,
            command_list: command_list,
            vertex_buffer: vertex_buffer,
            vertex_buffer_view: vertex_buffer_view,
            index_buffer: index_buffer,
            index_buffer_view: index_buffer_view,
            texture: texture,
            fence: fence,
            fence_value: fence_value,
            fence_event: fence_event,
            rotation_radians: 0,
            viewport: viewport,
            scissor_rect: scissor_rect,
        })
    }

    pub fn render(&mut self) -> Result<(), HRESULT> {
        self.populate_command_list()?;
        self.command_queue.execute_command_lists(&self.command_list);
        self.swap_chain.present(1, 0)?;
        self.wait_for_previous_frame()?;
        Ok(())
    }

    /// 描画コマンドリストを構築する
    fn populate_command_list(&mut self) -> Result<(), HRESULT> {
        let command_allocator = &self.command_allocator;
        let command_list = &self.command_list;
        let pipeline_state = &self.pipeline_state;
        let root_signature = &self.root_signature;
        let srv_heap = &self.srv_heap;
        let rtv_heap = &self.rtv_heap;
        let rtv_descriptor_size = self.rtv_descriptor_size;
        let viewport = &self.viewport;
        let scissor_rect = &self.scissor_rect;
        let render_targets = self.render_targets.as_slice();
        let frame_index = self.frame_index;
        let vertex_buffer_view = &self.vertex_buffer_view;
        let index_buffer_view = &self.index_buffer_view;


        // Command list allocators can only be reset when the associated
        // command lists have finished execution on the GPU; apps should use
        // fences to determine GPU execution progress.
        command_allocator.reset()?;

        // However, when ExecuteCommandList() is called on a particular command
        // list, that command list can then be reset at any time and must be before
        // re-recording.
        command_list.reset(command_allocator, pipeline_state)?;

        // Set necessary state.
        command_list.SetGraphicsRootSignature(root_signature.Get());

        let pp_heaps = [srv_heap];
        command_list.set_descriptor_heaps(&pp_heaps);

        self.rotation_radians += 0.02_f32;
        let rotation_radians = self.rotation_radians;
        command_list.set_graphics_root_f32_constant(0, rotation_radians, 0); // TODO
        command_list.set_graphics_root_descriptor_table(
            1,
            srv_heap.get_gpu_descriptor_handle_for_heap_start(),
        );
        command_list.rs_set_viewports(1, viewport);
        command_list.rs_set_scissor_rects(1, scissor_rect);

        // Indicate that the back buffer will be used as a render target.
        command_list.resource_barrier(
            1,
            D3DX12_RESOURCE_BARRIER::Transition(
                &render_targets[frame_index],
                D3D12_RESOURCE_STATE_PRESENT,
                D3D12_RESOURCE_STATE_RENDER_TARGET,
            ),
        );
        let rtv_handle = rtv_heap
            .get_cpu_descriptor_handle_for_heap_start()
            .frame_index(frame_index)
            .descriptor_size(rtv_descriptor_size);
        command_list.om_set_render_targets(1, &rtvHandle, false, None);

        // Record commands.
        let clear_color = [0_f32; 4];
        command_list.clear_render_target_view(
            &rtv_handle,
            clear_color,
            0,
            None,
        );
        command_list.ia_set_primitive_topology(
            D3D_PRIMITIVE_TOPOLOGY_TRIANGLELIST,
        );
        command_list.ia_set_vertex_buffers(0, 1, vertex_buffer_view);
        command_list.ia_set_index_buffer(index_buffer_view);
        command_list.draw_indexed_instanced(CIRCLE_SEGMENTS * 3, 1, 0, 0, 0);

        // Indicate that the back buffer will now be used to present.
        command_list.resource_barrier(
            1,
            D3DX12_RESOURCE_BARRIER::Transition(
                &render_targets[frame_index],
                D3D12_RESOURCE_STATE_RENDER_TARGET,
                D3D12_RESOURCE_STATE_PRESENT,
            ),
        );

        command_list.close()?;
        Ok(())
    }

    fn wait_for_previous_frame(&mut self) -> Result<(), HRESULT> {
        let mut fence_value = self.fence_value;
        let mut frame_index = self.frame_index;
        wait_for_previous_frame(
            &self.swap_chain,
            &self.command_queue,
            &self.fence,
            self.fence_event,
            &mut fence_value,
            &mut frame_index,
        )?;
        self.fence_value = fence_value;
        self.frame_index = frame_index;
        Ok(())
    }
}



// WAITING FOR THE FRAME TO COMPLETE BEFORE CONTINUING IS NOT BEST PRACTICE.
// This is code implemented as such for simplicity. The D3D12HelloFrameBuffering
// sample illustrates how to use fences for efficient resource usage and to
// maximize GPU utilization.
fn wait_for_previous_frame(
    swap_chain: &IDXGISwapChain3,
    command_queue: &ID3D12CommandQueue,
    fence: &ID3D12Fence,
    event: HANDLE,
    fence_value: &mut u64,
    frame_index: &mut u32,
) -> Result<(), HRESULT> {
    println!("wait_for_previous_frame start fence_value={}", *fence_value);
    // Signal and increment the fence value.
    let old_fence_value = *fence_value;
    command_queue.signal(fence, old_fence_value)?;
    *fence_value += 1;
    println!("wait_for_previous_frame set command_queue.signal");

    // Wait until the previous frame is finished.
    fence.wait_infinite(old_fence_value, event)?;
    println!("wait_for_previous_frame end fence.wait_infinite");

    *frame_index = swap_chain.get_current_back_buffer_index();
    println!("wait_for_previous_frame end");
    Ok(())
}
