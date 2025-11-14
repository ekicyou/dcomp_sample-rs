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

#[derive(Resource, Debug)]
pub struct GraphicsCore {
    pub d3d: ID3D11Device,
    pub dxgi: IDXGIDevice4,
    pub d2d_factory: ID2D1Factory,
    pub d2d: ID2D1Device,
    pub dwrite_factory: IDWriteFactory2,
    pub desktop: IDCompositionDesktopDevice,
    pub dcomp: IDCompositionDevice3,
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
            d3d,
            dxgi,
            d2d_factory,
            d2d,
            dwrite_factory,
            desktop,
            dcomp,
        })
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

/// GraphicsCoreが存在しない場合に作成するシステム
pub fn ensure_graphics_core(graphics: Option<Res<GraphicsCore>>, mut commands: Commands) {
    if graphics.is_none() {
        match GraphicsCore::new() {
            Ok(graphics) => {
                commands.insert_resource(graphics);
            }
            Err(e) => {
                eprintln!("[GraphicsCore] 初期化失敗: {:?}", e);
                panic!("GraphicsCoreの初期化に失敗しました。アプリケーションを終了します。");
            }
        }
    }
}

/// ウィンドウごとのグラフィックスリソース
#[derive(Component, Debug)]
pub struct WindowGraphics {
    /// DirectComposition Target (HWNDに関連付けられた合成ターゲット)
    pub target: IDCompositionTarget,
    /// Direct2D DeviceContext (このウィンドウでの描画に使用)
    pub device_context: ID2D1DeviceContext,
}

unsafe impl Send for WindowGraphics {}
unsafe impl Sync for WindowGraphics {}

impl WindowGraphics {
    /// IDCompositionTargetへの参照を取得する
    pub fn target(&self) -> &IDCompositionTarget {
        &self.target
    }

    /// ID2D1DeviceContextへの参照を取得する
    pub fn device_context(&self) -> &ID2D1DeviceContext {
        &self.device_context
    }
}

/// ウィンドウのルートビジュアルノード
#[derive(Component, Debug)]
pub struct Visual {
    /// ルートビジュアル（ビジュアルツリーの最上位ノード）
    pub visual: IDCompositionVisual3,
}

unsafe impl Send for Visual {}
unsafe impl Sync for Visual {}

impl Visual {
    /// IDCompositionVisual3への参照を取得する
    pub fn visual(&self) -> &IDCompositionVisual3 {
        &self.visual
    }
}

/// WindowHandleが付与されたエンティティに対してWindowGraphicsコンポーネントを作成する
pub fn create_window_graphics(
    query: Query<(Entity, &crate::ecs::window::WindowHandle), Without<WindowGraphics>>,
    graphics: Option<Res<GraphicsCore>>,
    mut commands: Commands,
) {
    // GraphicsCoreが存在しない場合は警告して処理をスキップ
    let Some(graphics) = graphics else {
        if !query.is_empty() {
            eprintln!("[create_window_graphics] 警告: GraphicsCoreが存在しないため処理をスキップします");
        }
        return;
    };

    for (entity, handle) in query.iter() {
        eprintln!(
            "[create_window_graphics] WindowGraphics作成開始 (Entity: {:?}, HWND: {:?})",
            entity, handle.hwnd
        );

        match create_window_graphics_for_hwnd(&graphics, handle.hwnd) {
            Ok(wg) => {
                eprintln!(
                    "[create_window_graphics] WindowGraphics作成完了 (Entity: {:?})",
                    entity
                );
                commands.entity(entity).insert(wg);
            }
            Err(e) => {
                eprintln!(
                    "[create_window_graphics] エラー: Entity {:?}, HWND {:?}, HRESULT {:?}",
                    entity, handle.hwnd, e
                );
                // エンティティをスキップして処理を継続
            }
        }
    }
}

/// HWNDに対してWindowGraphicsリソースを作成する
fn create_window_graphics_for_hwnd(
    graphics: &GraphicsCore,
    hwnd: HWND,
) -> Result<WindowGraphics> {
    use windows::Win32::Graphics::Direct2D::D2D1_DEVICE_CONTEXT_OPTIONS_NONE;

    // 1. CompositionTarget作成
    eprintln!("[create_window_graphics] IDCompositionTarget作成中...");
    let target = graphics.desktop.create_target_for_hwnd(hwnd, true)?;
    eprintln!("[create_window_graphics] IDCompositionTarget作成完了");

    // 2. DeviceContext作成
    eprintln!("[create_window_graphics] ID2D1DeviceContext作成中...");
    let device_context = graphics.d2d.create_device_context(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;
    eprintln!("[create_window_graphics] ID2D1DeviceContext作成完了");

    Ok(WindowGraphics {
        target,
        device_context,
    })
}

/// WindowGraphicsが存在するエンティティに対してVisualコンポーネントを作成する
pub fn create_window_visual(
    query: Query<(Entity, &WindowGraphics), Without<Visual>>,
    graphics: Option<Res<GraphicsCore>>,
    mut commands: Commands,
) {
    // GraphicsCoreが存在しない場合は警告して処理をスキップ
    let Some(graphics) = graphics else {
        if !query.is_empty() {
            eprintln!("[create_window_visual] 警告: GraphicsCoreが存在しないため処理をスキップします");
        }
        return;
    };

    for (entity, wg) in query.iter() {
        eprintln!(
            "[create_window_visual] Visual作成開始 (Entity: {:?})",
            entity
        );

        match create_visual_for_target(&graphics, &wg.target) {
            Ok(visual_comp) => {
                eprintln!(
                    "[create_window_visual] Visual作成完了 (Entity: {:?})",
                    entity
                );
                commands.entity(entity).insert(visual_comp);
            }
            Err(e) => {
                eprintln!(
                    "[create_window_visual] エラー: Entity {:?}, HRESULT {:?}",
                    entity, e
                );
                // エンティティをスキップして処理を継続
            }
        }
    }
}

/// IDCompositionTargetに対してVisualを作成してルートに設定する
fn create_visual_for_target(
    graphics: &GraphicsCore,
    target: &IDCompositionTarget,
) -> Result<Visual> {
    // 1. ビジュアル作成
    eprintln!("[create_window_visual] IDCompositionVisual3作成中...");
    let visual = graphics.dcomp.create_visual()?;
    eprintln!("[create_window_visual] IDCompositionVisual3作成完了");

    // 2. ターゲットにルートとして設定
    eprintln!("[create_window_visual] SetRoot実行中...");
    target.set_root(&visual)?;
    eprintln!("[create_window_visual] SetRoot完了");

    Ok(Visual { visual })
}

/// DirectCompositionのすべての変更を確定する
pub fn commit_composition(graphics: Option<Res<GraphicsCore>>) {
    let Some(graphics) = graphics else {
        return;
    };

    eprintln!("[commit_composition] Commit開始");
    if let Err(e) = graphics.dcomp.commit() {
        eprintln!("[commit_composition] Commit失敗: HRESULT {:?}", e);
    } else {
        eprintln!("[commit_composition] Commit完了");
    }
}

#[cfg(test)]
#[path = "graphics_tests.rs"]
mod graphics_tests;
