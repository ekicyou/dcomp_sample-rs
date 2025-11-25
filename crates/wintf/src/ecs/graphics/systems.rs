use super::command_list::GraphicsCommandList;
use super::components::SurfaceUpdateRequested;
use crate::com::d2d::{D2D1DeviceContextExt, D2D1DeviceExt};
use crate::com::dcomp::{
    DCompositionDesktopDeviceExt, DCompositionDeviceExt, DCompositionSurfaceExt,
    DCompositionVisualExt,
};
use crate::ecs::graphics::{
    GraphicsCore, GraphicsNeedsInit, HasGraphicsResources, SurfaceGraphics, VisualGraphics,
    WindowGraphics,
};
use crate::ecs::layout::GlobalArrangement;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::prelude::*;
use windows::Win32::Foundation::*;
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

    Ok(SurfaceGraphics::new(surface, (width, height)))
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
            eprintln!(
                "[draw_recursive] Entity={:?}, setting transform: [{},{},{},{}], bounds=({},{},{},{})",
                entity,
                arr.transform.M11, arr.transform.M12, arr.transform.M31, arr.transform.M32,
                arr.bounds.left, arr.bounds.top, arr.bounds.right, arr.bounds.bottom
            );
            dc.set_transform(&arr.transform);
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

/// Visual初期化・再初期化 (Deprecated: Use Visual component)
pub fn init_window_visual(
    _graphics: Res<GraphicsCore>,
    _query: Query<
        (Entity, &WindowGraphics, Option<&mut VisualGraphics>),
        Or<(Without<VisualGraphics>, With<GraphicsNeedsInit>)>,
    >,
    _commands: Commands,
    _frame_count: Res<crate::ecs::world::FrameCount>,
) {
    // Deprecated: Visual creation is now handled by visual_resource_management_system
}

// ========== Layout-to-Graphics Synchronization Systems ==========

/// GlobalArrangementからVisualへのサイズ同期
/// Windowコンポーネントを持つエンティティのみ処理
pub fn sync_visual_from_layout_root(
    mut query: Query<
        (&GlobalArrangement, &mut super::Visual),
        (With<crate::ecs::window::Window>, Changed<GlobalArrangement>),
    >,
) {
    for (global_arr, mut visual) in query.iter_mut() {
        let width = (global_arr.bounds.right - global_arr.bounds.left) as f32;
        let height = (global_arr.bounds.bottom - global_arr.bounds.top) as f32;
        visual.size.X = width;
        visual.size.Y = height;
    }
}

/// Visual.size変更時にDirectComposition Surfaceを再作成
pub fn resize_surface_from_visual(
    graphics: Option<Res<GraphicsCore>>,
    mut query: Query<
        (
            Entity,
            &VisualGraphics,
            &super::Visual,
            Option<&mut SurfaceGraphics>,
        ),
        Changed<super::Visual>,
    >,
    mut commands: Commands,
) {
    let Some(graphics) = graphics else {
        return;
    };

    if !graphics.is_valid() {
        return;
    }

    for (entity, visual_graphics, visual, surface) in query.iter_mut() {
        if !visual_graphics.is_valid() {
            continue;
        }

        let width = visual.size.X as u32;
        let height = visual.size.Y as u32;

        match surface {
            Some(mut surf) => {
                // サイズ不一致の場合のみ再作成
                if surf.size != (width, height) {
                    match create_surface_for_window(&graphics, visual_graphics, width, height) {
                        Ok(new_surface) => {
                            commands.entity(entity).insert(new_surface);
                        }
                        Err(e) => {
                            eprintln!("[resize_surface_from_visual] エラー: {:?}", e);
                            surf.invalidate();
                        }
                    }
                }
            }
            None => {
                // Surfaceがまだ作成されていない場合は作成
                match create_surface_for_window(&graphics, visual_graphics, width, height) {
                    Ok(new_surface) => {
                        commands.entity(entity).insert(new_surface);
                    }
                    Err(e) => {
                        eprintln!("[resize_surface_from_visual] エラー (新規作成): {:?}", e);
                    }
                }
            }
        }
    }
}

/// GlobalArrangementとVisualからWindowPosの位置・サイズを更新
/// Windowコンポーネントを持つエンティティのみ処理
pub fn sync_window_pos(
    mut query: Query<
        (
            &GlobalArrangement,
            &super::Visual,
            &mut crate::ecs::window::WindowPos,
        ),
        (
            With<crate::ecs::window::Window>,
            Or<(Changed<GlobalArrangement>, Changed<super::Visual>)>,
        ),
    >,
) {
    use windows::Win32::Foundation::{POINT, SIZE};

    for (global_arr, visual, mut window_pos) in query.iter_mut() {
        // GlobalArrangementが有効な値を持つ場合のみ更新
        // (0,0,0,0)のような初期値は無視
        let width = global_arr.bounds.right - global_arr.bounds.left;
        let height = global_arr.bounds.bottom - global_arr.bounds.top;

        if width <= 0.0 || height <= 0.0 {
            continue; // 無効なboundsはスキップ
        }

        let new_position = POINT {
            x: global_arr.bounds.left as i32,
            y: global_arr.bounds.top as i32,
        };
        let new_size = SIZE {
            cx: visual.size.X as i32,
            cy: visual.size.Y as i32,
        };

        // 実際に変更があった場合のみ更新
        let position_changed = window_pos.position != Some(new_position);
        let size_changed = window_pos.size != Some(new_size);

        if position_changed || size_changed {
            window_pos.position = Some(new_position);
            window_pos.size = Some(new_size);
        }
    }
}

/// WindowPos変更時にSetWindowPos Win32 APIを呼び出し、エコーバック値を記録
/// クライアント領域座標をウィンドウ全体座標に変換してからSetWindowPosを呼び出す
pub fn apply_window_pos_changes(
    mut query: Query<
        (
            Entity,
            &crate::ecs::window::WindowHandle,
            &mut crate::ecs::window::WindowPos,
        ),
        (
            Changed<crate::ecs::window::WindowPos>,
            With<crate::ecs::window::Window>,
        ),
    >,
) {
    for (_entity, window_handle, mut window_pos) in query.iter_mut() {
        // エコーバックチェック
        let position = window_pos.position.unwrap_or_default();
        let size = window_pos.size.unwrap_or_default();

        if window_pos.is_echo(position, size) {
            continue; // エコーバックなのでスキップ
        }

        // CW_USEDEFAULTが設定されている場合はスキップ（ウィンドウ作成時の初期値）
        // 座標変換をスキップし、ウィンドウ作成時の初期配置を優先
        if position.x == windows::Win32::UI::WindowsAndMessaging::CW_USEDEFAULT
            || size.cx == windows::Win32::UI::WindowsAndMessaging::CW_USEDEFAULT
        {
            continue;
        }

        // クライアント領域座標をウィンドウ全体座標に変換
        let (x, y, width, height) = match window_pos.to_window_coords(window_handle.hwnd) {
            Ok(coords) => coords,
            Err(e) => {
                // 変換失敗時はフォールバック：元の座標でSetWindowPosを呼び出す
                eprintln!(
                    "Failed to transform window coordinates: {}. Using original values.",
                    e
                );
                (position.x, position.y, size.cx, size.cy)
            }
        };

        // SetWindowPos呼び出し（変換後の座標を使用）
        let flags = window_pos.build_flags_for_system();
        let hwnd_insert_after = window_pos.get_hwnd_insert_after();

        let result = unsafe {
            windows::Win32::UI::WindowsAndMessaging::SetWindowPos(
                window_handle.hwnd,
                hwnd_insert_after,
                x,
                y,
                width,
                height,
                flags,
            )
        };

        if result.is_ok() {
            // 成功時のみlast_sent値を記録（変換後の値を記録）
            let bypass = window_pos.bypass_change_detection();
            bypass.last_sent_position = Some((x, y));
            bypass.last_sent_size = Some((width, height));
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

/// ChildOf変更を検出してVisual階層を同期するシステム (R3, R6, R7)
///
/// ECSのChildOf/Children階層とDirectCompositionのVisual階層を同期する。
/// - ChildOf追加時: 子VisualGraphicsを親VisualGraphicsに追加し、parent_visualをキャッシュ
/// - ChildOf変更時: 旧親から削除→新親に追加
///
/// 注意: エラーは無視する（親が先に削除されている場合など）
pub fn visual_hierarchy_sync_system(
    mut vg_queries: ParamSet<(
        Query<(Entity, &ChildOf, &mut VisualGraphics), Changed<ChildOf>>,
        Query<&VisualGraphics>,
    )>,
) {
    use crate::com::dcomp::DCompositionVisualExt;

    // 1. まず変更があったエンティティと親情報を収集
    let mut updates: Vec<(Entity, Entity)> = Vec::new(); // (child_entity, parent_entity)
    {
        let child_query = vg_queries.p0();
        for (entity, child_of, _child_vg) in child_query.iter() {
            updates.push((entity, child_of.parent()));
        }
    }

    // 2. 各子エンティティに対して処理
    for (child_entity, parent_entity) in updates {
        // 親のVisualを取得
        let parent_visual = {
            let parent_query = vg_queries.p1();
            parent_query
                .get(parent_entity)
                .ok()
                .and_then(|pv| pv.visual().cloned())
        };

        // 子のVisualGraphicsを更新
        let mut child_query = vg_queries.p0();
        if let Ok((_, _, mut child_vg)) = child_query.get_mut(child_entity) {
            // 子のvisualを取得
            let child_visual = match child_vg.visual() {
                Some(v) => v.clone(),
                None => continue,
            };

            // 旧親からの削除（parent_visualキャッシュを使用）
            if let Some(ref old_parent) = child_vg.parent_visual() {
                let _ = old_parent.remove_visual(&child_visual); // エラー無視
            }

            // 新しい親に追加
            if let Some(ref parent_visual) = parent_visual {
                // 親の末尾に追加（insertabove=false, referencevisual=None）
                let _ = parent_visual.add_visual(&child_visual, false, None); // エラー無視
                                                                              // parent_visualキャッシュを更新
                child_vg.set_parent_visual(Some(parent_visual.clone()));
            }
        }
    }
}
