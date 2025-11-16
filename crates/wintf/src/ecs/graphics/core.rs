use crate::com::d2d::*;
use crate::com::d3d11::*;
use crate::com::dcomp::*;
use crate::com::dwrite::*;
use bevy_ecs::prelude::*;
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
        eprintln!("[GraphicsCore] 初期化開始");
        
        let d3d = create_device_3d()?;
        let dxgi = d3d.cast()?;
        let d2d_factory = create_d2d_factory()?;
        let d2d = d2d_create_device(&dxgi)?;
        let dwrite_factory = dwrite_create_factory(DWRITE_FACTORY_TYPE_SHARED)?;
        let desktop = dcomp_create_desktop_device(&d2d)?;
        let dcomp: IDCompositionDevice3 = desktop.cast()?;
        
        eprintln!("[GraphicsCore] 初期化完了");
        
        Ok(Self {
            inner: Some(GraphicsCoreInner {
                d3d,
                dxgi,
                d2d_factory,
                d2d,
                dwrite_factory,
                desktop,
                dcomp,
            })
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
    
    unsafe {
        D2D1CreateFactory::<ID2D1Factory>(
            D2D1_FACTORY_TYPE_MULTI_THREADED,
            None,
        )
    }
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
