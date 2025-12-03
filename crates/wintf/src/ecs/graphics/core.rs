use crate::com::animation::*;
use crate::com::d2d::*;
use crate::com::d3d11::*;
use crate::com::dcomp::*;
use crate::com::dwrite::*;
use bevy_ecs::prelude::*;
use tracing::{debug, info, warn};
use windows::core::{Interface, Result};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::DirectComposition::*;
use windows::Win32::Graphics::DirectWrite::*;
use windows::Win32::Graphics::Dxgi::*;
use windows::Win32::UI::Animation::*;

#[derive(Debug)]
struct GraphicsCoreInner {
    pub d3d: ID3D11Device,
    pub dxgi: IDXGIDevice4,
    pub d2d_factory: ID2D1Factory,
    pub d2d: ID2D1Device,
    pub d2d_device_context: ID2D1DeviceContext, // グローバル共有DeviceContext
    pub dwrite_factory: IDWriteFactory2,
    pub desktop: IDCompositionDesktopDevice,
    pub dcomp: IDCompositionDevice3,
}

#[derive(Resource, Debug)]
pub struct GraphicsCore {
    inner: Option<GraphicsCoreInner>,
}

unsafe impl Send for GraphicsCore {}
unsafe impl Sync for GraphicsCore {}

impl GraphicsCore {
    pub fn new() -> Result<Self> {
        info!("[GraphicsCore] Initialization started");

        let d3d = create_device_3d()?;
        let dxgi = d3d.cast()?;
        let d2d_factory = create_d2d_factory()?;
        let d2d = d2d_create_device(&dxgi)?;

        // グローバル共有DeviceContextを作成
        let d2d_device_context = d2d.create_device_context(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;
        debug!("[GraphicsCore] Global DeviceContext created");

        let dwrite_factory = dwrite_create_factory(DWRITE_FACTORY_TYPE_SHARED)?;
        let desktop = dcomp_create_desktop_device(&d2d)?;
        let dcomp: IDCompositionDevice3 = desktop.cast()?;

        info!("[GraphicsCore] Initialization completed");

        Ok(Self {
            inner: Some(GraphicsCoreInner {
                d3d,
                dxgi,
                d2d_factory,
                d2d,
                d2d_device_context,
                dwrite_factory,
                desktop,
                dcomp,
            }),
        })
    }

    pub fn invalidate(&mut self) {
        self.inner = None;
    }

    pub fn is_valid(&self) -> bool {
        self.inner.is_some()
    }

    pub fn d2d_factory(&self) -> Option<&ID2D1Factory> {
        self.inner.as_ref().map(|i| &i.d2d_factory)
    }

    pub fn d2d_device(&self) -> Option<&ID2D1Device> {
        self.inner.as_ref().map(|i| &i.d2d)
    }

    pub fn dcomp(&self) -> Option<&IDCompositionDevice3> {
        self.inner.as_ref().map(|i| &i.dcomp)
    }

    pub fn desktop(&self) -> Option<&IDCompositionDesktopDevice> {
        self.inner.as_ref().map(|i| &i.desktop)
    }

    pub fn dwrite_factory(&self) -> Option<&IDWriteFactory2> {
        self.inner.as_ref().map(|i| &i.dwrite_factory)
    }

    /// グローバル共有DeviceContextへの参照を取得
    pub fn device_context(&self) -> Option<&ID2D1DeviceContext> {
        self.inner.as_ref().map(|i| &i.d2d_device_context)
    }

    pub fn d3d(&self) -> Option<&ID3D11Device> {
        self.inner.as_ref().map(|i| &i.d3d)
    }

    pub fn dxgi(&self) -> Option<&IDXGIDevice4> {
        self.inner.as_ref().map(|i| &i.dxgi)
    }
}

/// D2DFactoryを作成（マルチスレッド対応）
fn create_d2d_factory() -> Result<ID2D1Factory> {
    #[allow(unused_imports)]
    use windows::Win32::Graphics::Direct2D::Common::*;

    unsafe { D2D1CreateFactory::<ID2D1Factory>(D2D1_FACTORY_TYPE_MULTI_THREADED, None) }
}

fn create_device_3d() -> Result<ID3D11Device> {
    #[cfg(debug_assertions)]
    let flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_DEBUG;

    #[cfg(not(debug_assertions))]
    let flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT;

    d3d11_create_device(
        None,
        D3D_DRIVER_TYPE_HARDWARE,
        HMODULE::default(),
        flags,
        None,
        D3D11_SDK_VERSION,
        None,
        None,
    )
}

// ============================================================
// AnimationCore - Windows Animation API 統合リソース
// ============================================================

/// AnimationCore - Windows Animation API 統合リソース
///
/// WicCoreと同様のパターンで、CPUリソースのためEcsWorld::new()で即座に初期化される。
/// Device Lostの影響を受けない独立リソース。
///
/// # 保持するCOMオブジェクト
/// - `IUIAnimationTimer`: システム時刻取得
/// - `IUIAnimationManager2`: アニメーション状態管理
/// - `IUIAnimationTransitionLibrary2`: トランジション生成
#[derive(Resource)]
pub struct AnimationCore {
    timer: IUIAnimationTimer,
    manager: IUIAnimationManager2,
    transition_library: IUIAnimationTransitionLibrary2,
}

unsafe impl Send for AnimationCore {}
unsafe impl Sync for AnimationCore {}

impl AnimationCore {
    /// リソース作成
    pub fn new() -> Result<Self> {
        info!("[AnimationCore] Initialization started");

        let timer = create_animation_timer()?;
        let manager = create_animation_manager()?;
        let transition_library = create_animation_transition_library()?;

        info!("[AnimationCore] Initialization completed");

        Ok(Self {
            timer,
            manager,
            transition_library,
        })
    }

    /// 現在時刻取得 (f64秒)
    pub fn get_time(&self) -> Result<f64> {
        self.timer.get_time()
    }

    /// タイマー更新（毎フレーム呼び出し）
    /// 現在時刻を取得し、マネージャーを更新する
    pub fn tick(&self) -> Result<f64> {
        let time = self.timer.get_time()?;
        self.manager.update(time)?;
        Ok(time)
    }

    /// マネージャー参照
    pub fn manager(&self) -> &IUIAnimationManager2 {
        &self.manager
    }

    /// トランジションライブラリ参照
    pub fn transition_library(&self) -> &IUIAnimationTransitionLibrary2 {
        &self.transition_library
    }
}

impl std::fmt::Debug for AnimationCore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AnimationCore").finish_non_exhaustive()
    }
}

/// アニメーションタイマー更新システム
/// Input スケジュール先頭で実行（他システムより先に時刻確定）
pub fn animation_tick_system(animation_core: Option<Res<AnimationCore>>) {
    if let Some(core) = animation_core {
        if let Err(e) = core.tick() {
            warn!("Animation tick failed: {:?}", e);
        }
    }
}
