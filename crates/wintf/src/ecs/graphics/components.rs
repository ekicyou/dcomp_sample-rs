use bevy_ecs::prelude::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectComposition::*;

/// グラフィックスリソースを使用するエンティティを宣言（静的マーカー）
#[derive(Component, Default)]
pub struct HasGraphicsResources;

/// 初期化が必要な状態を示す動的マーカー
#[derive(Component, Default)]
pub struct GraphicsNeedsInit;

#[derive(Debug)]
struct WindowGraphicsInner {
    pub target: IDCompositionTarget,
    pub device_context: ID2D1DeviceContext,
}

/// ウィンドウごとのグラフィックスリソース
#[derive(Component, Debug)]
#[component(storage = "SparseSet")]
pub struct WindowGraphics {
    inner: Option<WindowGraphicsInner>,
    generation: u32,
}

unsafe impl Send for WindowGraphics {}
unsafe impl Sync for WindowGraphics {}

impl WindowGraphics {
    pub fn new(target: IDCompositionTarget, device_context: ID2D1DeviceContext) -> Self {
        Self {
            inner: Some(WindowGraphicsInner { target, device_context }),
            generation: 0,
        }
    }

    pub fn invalidate(&mut self) {
        self.inner = None;
    }

    pub fn is_valid(&self) -> bool {
        self.inner.is_some()
    }

    pub fn generation(&self) -> u32 {
        self.generation
    }

    pub fn increment_generation(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    /// IDCompositionTargetへの参照を取得する
    pub fn target(&self) -> Option<&IDCompositionTarget> {
        self.inner.as_ref().map(|i| &i.target)
    }

    /// ID2D1DeviceContextへの参照を取得する
    pub fn device_context(&self) -> Option<&ID2D1DeviceContext> {
        self.inner.as_ref().map(|i| &i.device_context)
    }
}

/// ウィンドウのルートビジュアルノード
#[derive(Component, Debug)]
pub struct VisualGraphics {
    inner: Option<IDCompositionVisual3>,
}

unsafe impl Send for VisualGraphics {}
unsafe impl Sync for VisualGraphics {}

impl VisualGraphics {
    pub fn new(visual: IDCompositionVisual3) -> Self {
        Self { inner: Some(visual) }
    }

    pub fn invalidate(&mut self) {
        self.inner = None;
    }

    pub fn is_valid(&self) -> bool {
        self.inner.is_some()
    }

    /// IDCompositionVisual3への参照を取得する
    pub fn visual(&self) -> Option<&IDCompositionVisual3> {
        self.inner.as_ref()
    }
}

/// ウィンドウの描画サーフェス
#[derive(Component, Debug)]
pub struct SurfaceGraphics {
    inner: Option<IDCompositionSurface>,
}

unsafe impl Send for SurfaceGraphics {}
unsafe impl Sync for SurfaceGraphics {}

impl SurfaceGraphics {
    pub fn new(surface: IDCompositionSurface) -> Self {
        Self { inner: Some(surface) }
    }

    pub fn invalidate(&mut self) {
        self.inner = None;
    }

    pub fn is_valid(&self) -> bool {
        self.inner.is_some()
    }

    /// IDCompositionSurfaceへの参照を取得
    pub fn surface(&self) -> Option<&IDCompositionSurface> {
        self.inner.as_ref()
    }
}
