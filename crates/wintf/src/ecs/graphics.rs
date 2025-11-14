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

/// ウィンドウの描画サーフェス
#[derive(Component, Debug)]
pub struct Surface {
    /// DirectComposition Surface (描画ターゲット)
    pub surface: IDCompositionSurface,
}

unsafe impl Send for Surface {}
unsafe impl Sync for Surface {}

impl Surface {
    /// IDCompositionSurfaceへの参照を取得
    pub fn surface(&self) -> &IDCompositionSurface {
        &self.surface
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

/// WindowGraphicsとVisualが存在するエンティティに対してSurfaceコンポーネントを作成する
pub fn create_window_surface(
    query: Query<(Entity, &WindowGraphics, &Visual, Option<&crate::ecs::window::WindowPos>), Without<Surface>>,
    graphics: Option<Res<GraphicsCore>>,
    mut commands: Commands,
) {
    // GraphicsCoreが存在しない場合は警告してスキップ
    let Some(graphics) = graphics else {
        if !query.is_empty() {
            eprintln!("[create_window_surface] 警告: GraphicsCoreが存在しないため処理をスキップします");
        }
        return;
    };

    for (entity, _wg, visual, window_pos) in query.iter() {
        // サイズ取得: WindowPosから、なければデフォルト (800, 600)
        let (width, height) = window_pos
            .and_then(|pos| pos.size.map(|s| (s.cx as u32, s.cy as u32)))
            .unwrap_or((800, 600));

        eprintln!(
            "[create_window_surface] Surface作成開始 (Entity: {:?}, Size: {}x{})",
            entity, width, height
        );

        match create_surface_for_window(&graphics, visual, width, height) {
            Ok(surface_comp) => {
                eprintln!(
                    "[create_window_surface] Surface作成完了 (Entity: {:?})",
                    entity
                );
                commands.entity(entity).insert(surface_comp);
            }
            Err(e) => {
                eprintln!(
                    "[create_window_surface] エラー: Entity {:?}, HRESULT {:?}",
                    entity, e
                );
                // エンティティをスキップして処理を継続
            }
        }
    }
}

/// Surfaceを作成してVisualに設定する
fn create_surface_for_window(
    graphics: &GraphicsCore,
    visual: &Visual,
    width: u32,
    height: u32,
) -> Result<Surface> {
    use windows::Win32::Graphics::Dxgi::Common::*;

    // 1. IDCompositionSurface作成
    eprintln!("[create_window_surface] IDCompositionSurface作成中...");
    let surface = graphics.dcomp.create_surface(
        width,
        height,
        DXGI_FORMAT_B8G8R8A8_UNORM,
        DXGI_ALPHA_MODE_PREMULTIPLIED,
    )?;
    eprintln!("[create_window_surface] IDCompositionSurface作成完了");

    // 2. VisualにSurfaceを設定
    eprintln!("[create_window_surface] Visual.SetContent()実行中...");
    visual.visual.set_content(&surface)?;
    eprintln!("[create_window_surface] Visual.SetContent()完了");

    Ok(Surface { surface })
}

/// Surfaceに対して図形を描画する
pub fn render_window(
    query: Query<(Entity, &Surface), Added<Surface>>,
    graphics: Option<Res<GraphicsCore>>,
) {
    let Some(graphics) = graphics else {
        return;
    };

    for (entity, surface) in query.iter() {
        eprintln!("[render_window] 描画処理開始 (Entity: {:?})", entity);
        
        if let Err(e) = render_shapes(&graphics, &surface.surface) {
            eprintln!("[render_window] 描画エラー (Entity: {:?}): {:?}", entity, e);
        } else {
            eprintln!("[render_window] 描画処理完了 (Entity: {:?})", entity);
        }
    }
}

/// 図形を描画する
fn render_shapes(
    _graphics: &GraphicsCore,
    surface: &IDCompositionSurface,
) -> Result<()> {
    use windows::Win32::Graphics::Direct2D::Common::*;
    use windows_numerics::Vector2;

    // 1. BeginDraw() → DeviceContext取得（描画準備完了）
    let (dc, _offset) = surface.begin_draw(None)?;

    // 2. Clear(transparent)
    dc.clear(Some(&D2D1_COLOR_F {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    }));

    // 3. 赤い円を描画
    match dc.create_solid_color_brush(
        &D2D1_COLOR_F { r: 1.0, g: 0.0, b: 0.0, a: 1.0 },
        None,
    ) {
        Ok(red_brush) => {
            dc.fill_ellipse(
                &D2D1_ELLIPSE {
                    point: Vector2 { X: 100.0, Y: 100.0 },
                    radiusX: 50.0,
                    radiusY: 50.0,
                },
                &red_brush,
            );
        }
        Err(e) => {
            eprintln!("[render_shapes] 赤いブラシ作成失敗: {:?}", e);
        }
    }

    // 4. 緑の四角を描画
    match dc.create_solid_color_brush(
        &D2D1_COLOR_F { r: 0.0, g: 1.0, b: 0.0, a: 1.0 },
        None,
    ) {
        Ok(green_brush) => {
            dc.fill_rectangle(
                &D2D_RECT_F {
                    left: 200.0,
                    top: 50.0,
                    right: 300.0,
                    bottom: 150.0,
                },
                &green_brush,
            );
        }
        Err(e) => {
            eprintln!("[render_shapes] 緑のブラシ作成失敗: {:?}", e);
        }
    }

    // 5. 青い三角を描画
    // DeviceContextからFactoryを取得してPathGeometryを作成
    let factory: ID2D1Factory = unsafe { dc.GetFactory()? };
    match create_triangle_geometry(&factory) {
        Ok(triangle) => {
            match dc.create_solid_color_brush(
                &D2D1_COLOR_F { r: 0.0, g: 0.0, b: 1.0, a: 1.0 },
                None,
            ) {
                Ok(blue_brush) => {
                    dc.fill_geometry(&triangle, &blue_brush);
                }
                Err(e) => {
                    eprintln!("[render_shapes] 青いブラシ作成失敗: {:?}", e);
                }
            }
        }
        Err(e) => {
            eprintln!("[render_shapes] PathGeometry作成失敗: {:?}", e);
        }
    }

    // 6. EndDraw()
    unsafe { dc.EndDraw(None, None)? };

    // 7. Surface.EndDraw()
    surface.end_draw()?;

    Ok(())
}

/// 三角形のPathGeometryを作成する
fn create_triangle_geometry(factory: &ID2D1Factory) -> Result<ID2D1PathGeometry> {
    use windows::Win32::Graphics::Direct2D::Common::*;
    use windows_numerics::Vector2;

    // PathGeometry作成
    let geometry = factory.create_path_geometry()?;
    
    // GeometrySink取得
    let sink = unsafe { geometry.Open()? };

    // 三角形の頂点を定義
    unsafe {
        sink.BeginFigure(
            Vector2 { X: 350.0, Y: 50.0 }, // 第1頂点
            D2D1_FIGURE_BEGIN_FILLED,
        );
        
        sink.AddLine(Vector2 { X: 425.0, Y: 150.0 }); // 第2頂点
        sink.AddLine(Vector2 { X: 275.0, Y: 150.0 }); // 第3頂点
        
        sink.EndFigure(D2D1_FIGURE_END_CLOSED);
        
        sink.Close()?;
    }

    Ok(geometry)
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
