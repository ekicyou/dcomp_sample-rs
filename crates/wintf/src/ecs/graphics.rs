use crate::com::d2d::*;
use crate::com::d3d11::*;
use crate::com::dcomp::*;
use bevy_ecs::prelude::*;
use windows::core::{Interface, Result};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::Direct3D::*;
use windows::Win32::Graphics::Direct3D11::*;
use windows::Win32::Graphics::DirectComposition::*;
use windows::Win32::Graphics::Dxgi::*;

#[derive(Resource, Debug)]
pub struct GraphicsDevices {
    pub d3d: ID3D11Device,
    pub dxgi: IDXGIDevice4,
    pub d2d: ID2D1Device,
    pub desktop: IDCompositionDesktopDevice,
    pub dcomp: IDCompositionDevice3,
}

unsafe impl Send for GraphicsDevices {}
unsafe impl Sync for GraphicsDevices {}

impl GraphicsDevices {
    pub fn new() -> Result<Self> {
        let d3d = create_device_3d()?;
        let dxgi = d3d.cast()?;
        let d2d = d2d_create_device(&dxgi)?;
        let desktop = dcomp_create_desktop_device(&d2d)?;
        let dcomp: IDCompositionDevice3 = desktop.cast()?;
        Ok(Self {
            d3d,
            dxgi,
            d2d,
            desktop,
            dcomp,
        })
    }
}

fn create_device_3d() -> Result<ID3D11Device> {
    d3d11_create_device(
        None,
        D3D_DRIVER_TYPE_HARDWARE,
        HMODULE::default(),
        D3D11_CREATE_DEVICE_BGRA_SUPPORT,
        None,
        D3D11_SDK_VERSION,
        None,
        None,
    )
}

/// GraphicsDevicesが存在しない場合に作成するシステム
pub fn ensure_graphics_devices(devices: Option<Res<GraphicsDevices>>, mut commands: Commands) {
    if devices.is_none() {
        match GraphicsDevices::new() {
            Ok(graphics) => {
                commands.insert_resource(graphics);
                eprintln!("Graphics devices created successfully");
            }
            Err(e) => {
                eprintln!("Failed to create graphics devices: {:?}", e);
            }
        }
    }
}
