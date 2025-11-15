use crate::com::d2d::*;
use crate::com::dcomp::*;
use crate::ecs::graphics::{GraphicsCore, Surface, Visual, WindowGraphics};
use bevy_ecs::prelude::*;
use windows::core::{Interface, Result};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Direct2D::*;
use windows::Win32::Graphics::DirectComposition::*;
use windows::Win32::Graphics::Dxgi::Common::*;

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
    let target = graphics.desktop.create_target_for_hwnd(hwnd, true)?;

    // 2. DeviceContext作成
    let device_context = graphics.d2d.create_device_context(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;

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
    let visual = graphics.dcomp.create_visual()?;

    // 2. ターゲットにルートとして設定
    target.set_root(&visual)?;

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
    // 1. IDCompositionSurface作成
    let surface = graphics.dcomp.create_surface(
        width,
        height,
        DXGI_FORMAT_B8G8R8A8_UNORM,
        DXGI_ALPHA_MODE_PREMULTIPLIED,
    )?;

    // 2. VisualにSurfaceを設定
    visual.visual.set_content(&surface)?;

    Ok(Surface { surface })
}

/// Surfaceへの描画（GraphicsCommandListの有無を統合処理）
pub fn render_surface(
    query: Query<
        (Entity, Option<&crate::ecs::graphics::GraphicsCommandList>, &Surface),
        Or<(Changed<crate::ecs::graphics::GraphicsCommandList>, Changed<Surface>)>
    >,
    _graphics_core: Option<Res<GraphicsCore>>,
) {
    use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
    
    for (entity, command_list, surface) in query.iter() {
        eprintln!(
            "[render_surface] Entity={:?}, has_command_list={}",
            entity,
            command_list.is_some()
        );

        // Surface描画開始
        let (dc, _offset) = match surface.surface().begin_draw(None) {
            Ok(result) => result,
            Err(err) => {
                eprintln!("[render_surface] Failed to begin draw for Entity={:?}: {:?}", entity, err);
                continue;
            }
        };

        unsafe {
            // 透明色クリア（常に実行）
            dc.clear(Some(&D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }));

            // CommandListがある場合のみ描画
            if let Some(command_list) = command_list {
                if let Some(cmd_list) = command_list.command_list() {
                    dc.draw_image(cmd_list);
                }
            }

            if let Err(err) = dc.EndDraw(None, None) {
                eprintln!("[render_surface] EndDraw failed for Entity={:?}: {:?}", entity, err);
                let _ = surface.surface().end_draw();
                continue;
            }
        }

        // Surface描画終了
        if let Err(err) = surface.surface().end_draw() {
            eprintln!("[render_surface] Failed to end draw for Entity={:?}: {:?}", entity, err);
            continue;
        }

        if command_list.is_some() {
            eprintln!("[render_surface] Surface rendered with CommandList for Entity={:?}", entity);
        } else {
            eprintln!("[render_surface] Surface cleared (no CommandList) for Entity={:?}", entity);
        }
    }
}

/// DirectCompositionのすべての変更を確定する
pub fn commit_composition(graphics: Option<Res<GraphicsCore>>) {
    let Some(graphics) = graphics else {
        return;
    };

    if let Err(e) = graphics.dcomp.commit() {
        eprintln!("[commit_composition] Commit失敗: HRESULT {:?}", e);
    }
}
