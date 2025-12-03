use crate::com::d2d::*;
use crate::com::d3d11::*;
use crate::com::dcomp::*;
use crate::com::dwrite::*;
use bevy_ecs::prelude::*;
use tracing::{debug, info};
use windows::core::{Interface, Result};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::DirectComposition::*;
use windows::Win32::Graphics::DirectWrite::*;
use windows::Win32::Graphics::Dxgi::*;

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
// FrameTime - 高精度フレーム時刻リソース
// ============================================================

/// FrameTime - 高精度フレーム時刻リソース
///
/// GetSystemTimePreciseAsFileTime（100ナノ秒単位）を使用。
/// Windows 8以降で利用可能な最高精度のシステム時刻API。
/// スレッドセーフ、どのスケジュールからでもアクセス可能。
#[derive(Resource, Debug)]
pub struct FrameTime {
    /// 起動時の時刻値（100ナノ秒単位）
    start_time: u64,
}

impl FrameTime {
    /// リソース作成
    pub fn new() -> Self {
        Self {
            start_time: Self::get_precise_time(),
        }
    }

    /// 現在時刻取得 (f64秒、起動時からの経過時間)
    pub fn elapsed_secs(&self) -> f64 {
        let now = Self::get_precise_time();
        let elapsed_100ns = now.saturating_sub(self.start_time);
        // 100ナノ秒 → 秒: 1秒 = 10,000,000 * 100ナノ秒
        elapsed_100ns as f64 / 10_000_000.0
    }

    /// 現在時刻を100ナノ秒単位のu64で取得
    pub fn elapsed_100ns(&self) -> u64 {
        Self::get_precise_time().saturating_sub(self.start_time)
    }

    /// 高精度システム時刻を取得（100ナノ秒単位のu64）
    fn get_precise_time() -> u64 {
        use windows::Win32::Foundation::FILETIME;
        use windows::Win32::System::SystemInformation::GetSystemTimePreciseAsFileTime;

        let ft: FILETIME = unsafe { GetSystemTimePreciseAsFileTime() };
        ((ft.dwHighDateTime as u64) << 32) | (ft.dwLowDateTime as u64)
    }
}

impl Default for FrameTime {
    fn default() -> Self {
        Self::new()
    }
}
