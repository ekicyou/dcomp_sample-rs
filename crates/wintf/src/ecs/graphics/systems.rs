use super::command_list::GraphicsCommandList;
use super::components::SurfaceUpdateRequested;
use crate::com::d2d::{D2D1DeviceContextExt, D2D1DeviceExt};
use crate::com::dcomp::{
    DCompositionDesktopDeviceExt, DCompositionDeviceExt, DCompositionSurfaceExt,
    DCompositionTargetExt, DCompositionVisualExt,
};
use crate::ecs::graphics::{
    GraphicsCore, GraphicsNeedsInit, HasGraphicsResources, SurfaceGraphics, VisualGraphics,
    WindowGraphics,
};
use crate::ecs::layout::GlobalArrangement;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::prelude::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::DirectComposition::*;
use windows::Win32::Graphics::Dxgi::Common::*;

// ========== ヘルパー関数 ==========

/// HWNDに対してWindowGraphicsリソースを作成する
fn create_window_graphics_for_hwnd(
    graphics: &GraphicsCore,
    hwnd: HWND,
) -> windows::core::Result<WindowGraphics> {
    use windows::Win32::Graphics::Direct2D::D2D1_DEVICE_CONTEXT_OPTIONS_NONE;

    if !graphics.is_valid() {
        return Err(windows::core::Error::from(E_FAIL));
    }

    // 1. CompositionTarget作成
    let desktop = graphics
        .desktop()
        .ok_or(windows::core::Error::from(E_FAIL))?;
    let target = desktop.create_target_for_hwnd(hwnd, true)?;

    // 2. DeviceContext作成
    let d2d = graphics
        .d2d_device()
        .ok_or(windows::core::Error::from(E_FAIL))?;
    let device_context = d2d.create_device_context(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)?;

    Ok(WindowGraphics::new(target, device_context))
}

/// IDCompositionTargetに対してVisualを作成してルートに設定する
fn create_visual_for_target(
    graphics: &GraphicsCore,
    target: &IDCompositionTarget,
) -> windows::core::Result<VisualGraphics> {
    if !graphics.is_valid() {
        return Err(windows::core::Error::from(E_FAIL));
    }

    // 1. ビジュアル作成
    let dcomp = graphics.dcomp().ok_or(windows::core::Error::from(E_FAIL))?;
    let visual = dcomp.create_visual()?;

    // 2. ターゲットにルートとして設定
    target.set_root(&visual)?;

    Ok(VisualGraphics::new(visual))
}

/// Surfaceを作成してVisualに設定する
fn create_surface_for_window(
    graphics: &GraphicsCore,
    visual: &VisualGraphics,
    width: u32,
    height: u32,
) -> windows::core::Result<SurfaceGraphics> {
    if !graphics.is_valid() {
        return Err(windows::core::Error::from(E_FAIL));
    }

    // 1. IDCompositionSurface作成
    let dcomp = graphics.dcomp().ok_or(windows::core::Error::from(E_FAIL))?;
    let surface = dcomp.create_surface(
        width,
        height,
        DXGI_FORMAT_B8G8R8A8_UNORM,
        DXGI_ALPHA_MODE_PREMULTIPLIED,
    )?;

    // 2. VisualにSurfaceを設定
    let visual_ref = visual.visual().ok_or(windows::core::Error::from(E_FAIL))?;
    visual_ref.set_content(&surface)?;

    Ok(SurfaceGraphics::new(surface))
}

fn draw_recursive(
    entity: Entity,
    dc: &windows::Win32::Graphics::Direct2D::ID2D1DeviceContext,
    widgets: &Query<(Option<&GlobalArrangement>, Option<&GraphicsCommandList>)>,
    hierarchy: &Query<&Children>,
    surface_query: &Query<&SurfaceGraphics>,
    is_root: bool,
) {
    use windows::Win32::Graphics::Direct2D::Common::D2D1_COMPOSITE_MODE_SOURCE_OVER;
    use windows::Win32::Graphics::Direct2D::D2D1_INTERPOLATION_MODE_LINEAR;

    // Nested Surface Check
    if !is_root && surface_query.contains(entity) {
        return;
    }

    // Draw current entity
    if let Ok((global_arr, cmd_list)) = widgets.get(entity) {
        if let Some(arr) = global_arr {
            dc.set_transform(&arr.0);
        }
        if let Some(list) = cmd_list {
            if let Some(command_list) = list.command_list() {
                unsafe {
                    dc.DrawImage(
                        command_list,
                        None,
                        None,
                        D2D1_INTERPOLATION_MODE_LINEAR,
                        D2D1_COMPOSITE_MODE_SOURCE_OVER,
                    );
                }
            }
        }
    }

    // Recurse
    if let Ok(children) = hierarchy.get(entity) {
        for child in children.iter() {
            draw_recursive(child, dc, widgets, hierarchy, surface_query, false);
        }
    }
}

// ========== 描画システム ==========/// Surfaceへの描画（GraphicsCommandListの有無を統合処理）
pub fn render_surface(
    mut commands: Commands,
    surfaces: Query<(Entity, &SurfaceGraphics), With<SurfaceUpdateRequested>>,
    widgets: Query<(Option<&GlobalArrangement>, Option<&GraphicsCommandList>)>,
    hierarchy: Query<&Children>,
    surface_query: Query<&SurfaceGraphics>,
    _graphics_core: Option<Res<GraphicsCore>>,
    frame_count: Res<crate::ecs::world::FrameCount>,
) {
    use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;

    for (entity, surface) in surfaces.iter() {
        eprintln!(
            "[Frame {}] [render_surface] === Processing Surface Entity={:?} ===",
            frame_count.0, entity
        );

        if !surface.is_valid() {
            eprintln!(
                "[render_surface] Surface invalid for Entity={:?}, skipping",
                entity
            );
            continue;
        }

        // Surface描画開始
        let surface_ref = match surface.surface() {
            Some(s) => s,
            None => continue,
        };

        let (dc, _offset) =
            match surface_ref.begin_draw(None) {
                Ok(result) => {
                    eprintln!(
                        "[render_surface] BeginDraw succeeded for Entity={:?}, offset=({}, {})",
                        entity, result.1.x, result.1.y
                    );
                    result
                }
                Err(err) => {
                    eprintln!(
                    "[render_surface] BeginDraw failed for Entity={:?}: {:?}, HRESULT: 0x{:08X}",
                    entity, err, err.code().0
                );
                    continue;
                }
            };

        // 透明色クリア（常に実行）
        dc.clear(Some(&D2D1_COLOR_F {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }));

        // Recursive draw
        draw_recursive(entity, &dc, &widgets, &hierarchy, &surface_query, true);

        // EndDraw
        if let Err(e) = surface_ref.end_draw() {
            eprintln!("[render_surface] EndDraw failed: {:?}", e);
        }

        // Remove marker
        commands.entity(entity).remove::<SurfaceUpdateRequested>();
    }
}

/// DirectCompositionのすべての変更を確定する
pub fn commit_composition(
    graphics: Option<Res<GraphicsCore>>,
    frame_count: Res<crate::ecs::world::FrameCount>,
) {
    let Some(graphics) = graphics else {
        eprintln!(
            "[Frame {}] [commit_composition] GraphicsCore not available",
            frame_count.0
        );
        return;
    };

    if !graphics.is_valid() {
        eprintln!(
            "[Frame {}] [commit_composition] GraphicsCore is invalid",
            frame_count.0
        );
        return;
    }

    let dcomp = match graphics.dcomp() {
        Some(d) => d,
        None => {
            eprintln!(
                "[Frame {}] [commit_composition] DComp device not available",
                frame_count.0
            );
            return;
        }
    };

    if let Err(e) = dcomp.commit() {
        eprintln!(
            "[Frame {}] [commit_composition] Commit失敗: HRESULT {:?}",
            frame_count.0, e
        );
        eprintln!(
            "[Frame {}] [commit_composition] Commit失敗 HRESULT: 0x{:08X}",
            frame_count.0,
            e.code().0
        );
    }
}

// ========== 新しい再初期化システム ==========

/// GraphicsCore初期化・再初期化・一括マーキング
pub fn init_graphics_core(
    graphics: Option<ResMut<GraphicsCore>>,
    query: Query<Entity, With<HasGraphicsResources>>,
    mut commands: Commands,
    frame_count: Res<crate::ecs::world::FrameCount>,
) {
    match graphics {
        Some(mut gc) => {
            if gc.is_valid() {
                return;
            }

            eprintln!(
                "[Frame {}] [init_graphics_core] GraphicsCore再初期化を開始",
                frame_count.0
            );
            match GraphicsCore::new() {
                Ok(new_gc) => {
                    *gc = new_gc;
                    eprintln!(
                        "[Frame {}] [init_graphics_core] GraphicsCore再初期化完了",
                        frame_count.0
                    );

                    let count = query.iter().count();
                    eprintln!("[Frame {}] [init_graphics_core] {}個のエンティティにGraphicsNeedsInitマーカーを追加", frame_count.0, count);
                    for entity in query.iter() {
                        commands.entity(entity).insert(GraphicsNeedsInit);
                    }
                }
                Err(e) => {
                    eprintln!(
                        "[Frame {}] [init_graphics_core] GraphicsCore再初期化失敗: {:?}",
                        frame_count.0, e
                    );
                }
            }
        }
        None => {
            eprintln!(
                "[Frame {}] [init_graphics_core] GraphicsCore初期化を開始",
                frame_count.0
            );
            match GraphicsCore::new() {
                Ok(gc) => {
                    eprintln!(
                        "[Frame {}] [init_graphics_core] GraphicsCore初期化完了",
                        frame_count.0
                    );
                    commands.insert_resource(gc);

                    let count = query.iter().count();
                    eprintln!("[Frame {}] [init_graphics_core] {}個のエンティティにGraphicsNeedsInitマーカーを追加", frame_count.0, count);
                    for entity in query.iter() {
                        commands.entity(entity).insert(GraphicsNeedsInit);
                    }
                }
                Err(e) => {
                    eprintln!(
                        "[Frame {}] [init_graphics_core] GraphicsCore初期化失敗: {:?}",
                        frame_count.0, e
                    );
                }
            }
        }
    }
}

/// WindowGraphics初期化・再初期化
pub fn init_window_graphics(
    graphics: Res<GraphicsCore>,
    mut query: Query<
        (
            Entity,
            &crate::ecs::window::WindowHandle,
            Option<&mut WindowGraphics>,
        ),
        Or<(Without<WindowGraphics>, With<GraphicsNeedsInit>)>,
    >,
    mut commands: Commands,
    frame_count: Res<crate::ecs::world::FrameCount>,
) {
    if !graphics.is_valid() {
        return;
    }

    for (entity, handle, window_graphics) in query.iter_mut() {
        match window_graphics {
            None => {
                eprintln!(
                    "[Frame {}] [init_window_graphics] WindowGraphics新規作成 (Entity: {:?})",
                    frame_count.0, entity
                );
                match create_window_graphics_for_hwnd(&graphics, handle.hwnd) {
                    Ok(wg) => {
                        eprintln!("[Frame {}] [init_window_graphics] WindowGraphics作成完了 (Entity: {:?})", frame_count.0, entity);
                        commands.entity(entity).insert(wg);
                    }
                    Err(e) => {
                        eprintln!(
                            "[Frame {}] [init_window_graphics] エラー: Entity {:?}, HRESULT {:?}",
                            frame_count.0, entity, e
                        );
                    }
                }
            }
            Some(mut wg) => {
                if !wg.is_valid() {
                    eprintln!(
                        "[Frame {}] [init_window_graphics] WindowGraphics再初期化 (Entity: {:?})",
                        frame_count.0, entity
                    );
                    let old_generation = wg.generation();
                    match create_window_graphics_for_hwnd(&graphics, handle.hwnd) {
                        Ok(new_wg) => {
                            // 古いgenerationを引き継いでインクリメント
                            let new_generation = old_generation.wrapping_add(1);
                            *wg = new_wg;
                            // generation を手動で設定（newのデフォルトは0なので）
                            while wg.generation() < new_generation {
                                wg.increment_generation();
                            }
                            eprintln!("[Frame {}] [init_window_graphics] WindowGraphics再初期化完了 (Entity: {:?}, generation: {} -> {})", 
                                frame_count.0, entity, old_generation, wg.generation());
                        }
                        Err(e) => {
                            eprintln!("[Frame {}] [init_window_graphics] 再初期化エラー: Entity {:?}, HRESULT {:?}", frame_count.0, entity, e);
                        }
                    }
                }
            }
        }
    }
}

/// Visual初期化・再初期化
pub fn init_window_visual(
    graphics: Res<GraphicsCore>,
    mut query: Query<
        (Entity, &WindowGraphics, Option<&mut VisualGraphics>),
        Or<(Without<VisualGraphics>, With<GraphicsNeedsInit>)>,
    >,
    mut commands: Commands,
    frame_count: Res<crate::ecs::world::FrameCount>,
) {
    if !graphics.is_valid() {
        return;
    }

    for (entity, window_graphics, visual) in query.iter_mut() {
        if !window_graphics.is_valid() {
            continue;
        }

        let target = match window_graphics.get_target() {
            Some(t) => t,
            None => continue,
        };

        match visual {
            None => {
                eprintln!(
                    "[Frame {}] [init_window_visual] Visual新規作成 (Entity: {:?})",
                    frame_count.0, entity
                );
                match create_visual_for_target(&graphics, target) {
                    Ok(v) => {
                        eprintln!(
                            "[Frame {}] [init_window_visual] Visual作成完了 (Entity: {:?})",
                            frame_count.0, entity
                        );
                        commands.entity(entity).insert(v);
                    }
                    Err(e) => {
                        eprintln!(
                            "[Frame {}] [init_window_visual] エラー: Entity {:?}, HRESULT {:?}",
                            frame_count.0, entity, e
                        );
                    }
                }
            }
            Some(mut v) => {
                if !v.is_valid() {
                    eprintln!(
                        "[Frame {}] [init_window_visual] Visual再初期化 (Entity: {:?})",
                        frame_count.0, entity
                    );
                    match create_visual_for_target(&graphics, target) {
                        Ok(new_v) => {
                            *v = new_v;
                            eprintln!(
                                "[Frame {}] [init_window_visual] Visual再初期化完了 (Entity: {:?})",
                                frame_count.0, entity
                            );
                        }
                        Err(e) => {
                            eprintln!("[Frame {}] [init_window_visual] 再初期化エラー: Entity {:?}, HRESULT {:?}", frame_count.0, entity, e);
                        }
                    }
                }
            }
        }
    }
}

/// Surface初期化・再初期化
pub fn init_window_surface(
    graphics: Res<GraphicsCore>,
    mut query: Query<
        (
            Entity,
            &WindowGraphics,
            &VisualGraphics,
            Option<&mut SurfaceGraphics>,
            Option<&crate::ecs::window::WindowPos>,
        ),
        Or<(Without<SurfaceGraphics>, With<GraphicsNeedsInit>)>,
    >,
    mut commands: Commands,
    frame_count: Res<crate::ecs::world::FrameCount>,
) {
    if !graphics.is_valid() {
        return;
    }

    for (entity, window_graphics, visual, surface, window_pos) in query.iter_mut() {
        if !window_graphics.is_valid() || !visual.is_valid() {
            continue;
        }

        let (width, height) = window_pos
            .and_then(|pos| pos.size.map(|s| (s.cx as u32, s.cy as u32)))
            .unwrap_or((800, 600));

        match surface {
            None => {
                eprintln!(
                    "[Frame {}] [init_window_surface] Surface新規作成 (Entity: {:?}, Size: {}x{})",
                    frame_count.0, entity, width, height
                );
                match create_surface_for_window(&graphics, visual, width, height) {
                    Ok(s) => {
                        eprintln!(
                            "[Frame {}] [init_window_surface] Surface作成完了 (Entity: {:?})",
                            frame_count.0, entity
                        );
                        commands.entity(entity).insert(s);
                    }
                    Err(e) => {
                        eprintln!(
                            "[Frame {}] [init_window_surface] エラー: Entity {:?}, HRESULT {:?}",
                            frame_count.0, entity, e
                        );
                    }
                }
            }
            Some(s) => {
                if !s.is_valid() {
                    eprintln!("[Frame {}] [init_window_surface] Surface再初期化 (Entity: {:?}, Size: {}x{})", frame_count.0, entity, width, height);
                    match create_surface_for_window(&graphics, visual, width, height) {
                        Ok(new_s) => {
                            // Use commands to trigger on_replace hook
                            commands.entity(entity).insert(new_s);
                            eprintln!("[Frame {}] [init_window_surface] Surface再初期化完了 (Entity: {:?})", frame_count.0, entity);
                        }
                        Err(e) => {
                            eprintln!("[Frame {}] [init_window_surface] 再初期化エラー: Entity {:?}, HRESULT {:?}", frame_count.0, entity, e);
                        }
                    }
                }
            }
        }
    }
}

/// GraphicsNeedsInitマーカー削除・初期化完了判定
pub fn cleanup_graphics_needs_init(
    query: Query<
        (Entity, &WindowGraphics, &VisualGraphics, &SurfaceGraphics),
        With<GraphicsNeedsInit>,
    >,
    mut commands: Commands,
) {
    for (entity, window_graphics, visual, surface) in query.iter() {
        if window_graphics.is_valid() && visual.is_valid() && surface.is_valid() {
            eprintln!(
                "[cleanup_graphics_needs_init] GraphicsNeedsInitマーカー削除 (Entity: {:?})",
                entity
            );
            commands.entity(entity).remove::<GraphicsNeedsInit>();
        }
    }
}

/// GraphicsNeedsInit時に古いGraphicsCommandListを削除
pub fn cleanup_command_list_on_reinit(
    query: Query<
        Entity,
        (
            With<GraphicsNeedsInit>,
            With<crate::ecs::graphics::GraphicsCommandList>,
        ),
    >,
    mut commands: Commands,
) {
    for entity in query.iter() {
        commands
            .entity(entity)
            .remove::<crate::ecs::graphics::GraphicsCommandList>();
        eprintln!(
            "[cleanup_command_list_on_reinit] GraphicsCommandList削除 (Entity: {:?})",
            entity
        );
    }
}

/// 依存コンポーネント無効化
pub fn invalidate_dependent_components(
    graphics: Option<Res<GraphicsCore>>,
    mut window_graphics_query: Query<&mut WindowGraphics>,
    mut visual_query: Query<&mut VisualGraphics>,
    mut surface_query: Query<&mut SurfaceGraphics>,
) {
    if let Some(gc) = graphics {
        if !gc.is_valid() {
            eprintln!(
                "[invalidate_dependent_components] GraphicsCore無効 - 全依存コンポーネントを無効化"
            );

            for mut wg in window_graphics_query.iter_mut() {
                wg.invalidate();
            }
            for mut v in visual_query.iter_mut() {
                v.invalidate();
            }
            for mut s in surface_query.iter_mut() {
                s.invalidate();
            }
        }
    }
}

/// 変更検知システム：描画内容や配置に変更があった場合、親サーフェスに更新要求マーカーを付与する
pub fn mark_dirty_surfaces(
    mut commands: Commands,
    changed_query: Query<
        Entity,
        (
            Or<(
                Changed<GraphicsCommandList>,
                Changed<GlobalArrangement>,
                Changed<Children>,
            )>,
        ),
    >,
    parent_query: Query<&ChildOf>,
    surface_query: Query<&SurfaceGraphics>,
) {
    for entity in changed_query.iter() {
        let mut current = entity;
        loop {
            if surface_query.contains(current) {
                commands.entity(current).insert(SurfaceUpdateRequested);
                break;
            }
            if let Ok(parent) = parent_query.get(current) {
                current = parent.parent();
            } else {
                break;
            }
        }
    }
}
