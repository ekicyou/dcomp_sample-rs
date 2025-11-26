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
use bevy_ecs::name::Name;
use bevy_ecs::prelude::*;
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Dxgi::Common::*;

// ========== ヘルパー関数 ==========

/// エンティティ名をログ用にフォーマットする
///
/// # Arguments
/// * `entity` - エンティティID
/// * `name` - Nameコンポーネント（オプション）
///
/// # Returns
/// Nameがあれば名前をそのまま返し、なければ`Entity(0v1)`形式でEntity IDを返す
pub fn format_entity_name(entity: Entity, name: Option<&Name>) -> String {
    match name {
        Some(n) => n.to_string(),
        None => format!("Entity({:?})", entity),
    }
}

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

/// 旧描画方式: 親Surfaceが子を再帰的に描画
/// Phase 4で廃止。自己描画方式（render_surface）に置き換え。
/// ロールバック用に残している。
#[allow(dead_code)]
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

// ========== 描画システム ==========

/// Surfaceへの自己描画（Phase 4: 自己描画方式）
///
/// 各Entityが自分のSurfaceに自分のGraphicsCommandListのみを描画する。
/// draw_recursive方式を廃止し、DirectCompositionのVisual階層で合成を行う。
///
/// - SurfaceUpdateRequestedマーカーを持つEntityが対象
/// - 自分のGraphicsCommandListのみを描画（子は描画しない）
/// - 子の描画は各子が自分のSurfaceで行う
/// - GlobalArrangementのスケール成分を適用（DPIスケール対応）
pub fn render_surface(
    mut commands: Commands,
    surfaces: Query<
        (
            Entity,
            &SurfaceGraphics,
            &GlobalArrangement,
            Option<&GraphicsCommandList>,
            Option<&Name>,
        ),
        With<SurfaceUpdateRequested>,
    >,
    _graphics_core: Option<Res<GraphicsCore>>,
    frame_count: Res<crate::ecs::world::FrameCount>,
) {
    use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
    use windows::Win32::Graphics::Direct2D::Common::D2D1_COMPOSITE_MODE_SOURCE_OVER;
    use windows::Win32::Graphics::Direct2D::D2D1_INTERPOLATION_MODE_LINEAR;

    for (entity, surface, global_arrangement, cmd_list_opt, name) in surfaces.iter() {
        let entity_name = format_entity_name(entity, name);
        eprintln!(
            "[Frame {}] [render_surface] === Self-rendering Entity={} ===",
            frame_count.0, entity_name
        );

        if !surface.is_valid() {
            eprintln!(
                "[render_surface] Surface invalid for Entity={}, skipping",
                entity_name
            );
            continue;
        }

        // Surface描画開始
        let surface_ref = match surface.surface() {
            Some(s) => s,
            None => continue,
        };

        let (dc, offset) = match surface_ref.begin_draw(None) {
            Ok(result) => {
                eprintln!(
                    "[render_surface] BeginDraw succeeded for Entity={}, offset=({}, {})",
                    entity_name, result.1.x, result.1.y
                );
                result
            }
            Err(err) => {
                eprintln!(
                    "[render_surface] BeginDraw failed for Entity={}: {:?}, HRESULT: 0x{:08X}",
                    entity_name,
                    err,
                    err.code().0
                );
                continue;
            }
        };

        // BeginDrawのoffsetとGlobalArrangementのスケールを適用
        // - offset: Surface内の描画開始位置（物理ピクセル単位）
        // - scale: GlobalArrangementから抽出したスケール成分（DPIスケール含む）
        // - Visual.SetOffsetX/Yで位置は設定済みなので、ここではスケールのみ
        unsafe {
            // GlobalArrangementからスケール成分のみを抽出（M11=scaleX, M22=scaleY）
            let scale_transform = windows_numerics::Matrix3x2 {
                M11: global_arrangement.transform.M11,
                M12: 0.0,
                M21: 0.0,
                M22: global_arrangement.transform.M22,
                M31: 0.0,
                M32: 0.0,
            };

            // offset（物理ピクセル）で平行移動
            let offset_transform =
                windows_numerics::Matrix3x2::translation(offset.x as f32, offset.y as f32);

            // 変換順序: offset → scale
            // GraphicsCommandListは論理座標で描画されているため、
            // まずoffsetで描画開始位置に移動し、その後scaleで拡大
            let transform = offset_transform * scale_transform;
            dc.SetTransform(&transform);

            eprintln!(
                "[render_surface] Transform for Entity={}: scale=({}, {}), offset=({}, {})",
                entity_name, scale_transform.M11, scale_transform.M22, offset.x, offset.y
            );
        }

        // 透明色クリア
        dc.clear(Some(&D2D1_COLOR_F {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }));

        // 自己描画: 自分のGraphicsCommandListのみを描画
        // 子の描画は行わない（各子が自分のSurfaceで行う）
        if let Some(cmd_list) = cmd_list_opt {
            if let Some(command_list) = cmd_list.command_list() {
                eprintln!(
                    "[render_surface] Drawing own CommandList for Entity={}",
                    entity_name
                );
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

    match dcomp.commit() {
        Ok(()) => {
            // 成功時は最初の数フレームだけログ出力
            if frame_count.0 <= 5 {
                eprintln!("[Frame {}] [commit_composition] Commit成功", frame_count.0);
            }
        }
        Err(e) => {
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
            Option<&Name>,
        ),
        Or<(Without<WindowGraphics>, With<GraphicsNeedsInit>)>,
    >,
    mut commands: Commands,
    frame_count: Res<crate::ecs::world::FrameCount>,
) {
    if !graphics.is_valid() {
        return;
    }

    for (entity, handle, window_graphics, name) in query.iter_mut() {
        let entity_name = format_entity_name(entity, name);
        match window_graphics {
            None => {
                eprintln!(
                    "[Frame {}] [init_window_graphics] WindowGraphics新規作成 (Entity: {})",
                    frame_count.0, entity_name
                );
                match create_window_graphics_for_hwnd(&graphics, handle.hwnd) {
                    Ok(wg) => {
                        eprintln!(
                            "[Frame {}] [init_window_graphics] WindowGraphics作成完了 (Entity: {})",
                            frame_count.0, entity_name
                        );
                        commands.entity(entity).insert(wg);
                    }
                    Err(e) => {
                        eprintln!(
                            "[Frame {}] [init_window_graphics] エラー: Entity {}, HRESULT {:?}",
                            frame_count.0, entity_name, e
                        );
                    }
                }
            }
            Some(mut wg) => {
                if !wg.is_valid() {
                    eprintln!(
                        "[Frame {}] [init_window_graphics] WindowGraphics再初期化 (Entity: {})",
                        frame_count.0, entity_name
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
                            eprintln!("[Frame {}] [init_window_graphics] WindowGraphics再初期化完了 (Entity: {}, generation: {} -> {})", 
                                frame_count.0, entity_name, old_generation, wg.generation());
                        }
                        Err(e) => {
                            eprintln!("[Frame {}] [init_window_graphics] 再初期化エラー: Entity {}, HRESULT {:?}", frame_count.0, entity_name, e);
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

/// Arrangement変更時にDirectComposition Surfaceを作成/リサイズ
/// ArrangementをSingle Source of Truthとして直接参照
pub fn sync_surface_from_arrangement(
    graphics: Option<Res<GraphicsCore>>,
    mut query: Query<
        (
            Entity,
            &VisualGraphics,
            &crate::ecs::layout::Arrangement,
            Option<&mut SurfaceGraphics>,
            Option<&Name>,
        ),
        Changed<crate::ecs::layout::Arrangement>,
    >,
    mut commands: Commands,
    frame_count: Res<crate::ecs::world::FrameCount>,
) {
    let Some(graphics) = graphics else {
        return;
    };

    if !graphics.is_valid() {
        return;
    }

    let mut _processed_count = 0;

    for (entity, visual_graphics, arrangement, surface, name) in query.iter_mut() {
        let entity_name = format_entity_name(entity, name);
        if !visual_graphics.is_valid() {
            eprintln!(
                "[Frame {}] [sync_surface_from_arrangement] Entity={} has invalid VisualGraphics, skipping",
                frame_count.0, entity_name
            );
            continue;
        }

        let width = arrangement.size.width as u32;
        let height = arrangement.size.height as u32;

        // サイズが0の場合はスキップ（まだレイアウトされていない）
        if width == 0 || height == 0 {
            eprintln!(
                "[Frame {}] [sync_surface_from_arrangement] Entity={} has zero size ({}x{}), skipping",
                frame_count.0, entity_name, width, height
            );
            continue;
        }

        _processed_count += 1;
        eprintln!(
            "[Frame {}] [sync_surface_from_arrangement] Processing Entity={}, size={}x{}, has_surface={}",
            frame_count.0, entity_name, width, height, surface.is_some()
        );

        match surface {
            Some(mut surf) => {
                // サイズ不一致の場合のみ再作成
                if surf.size != (width, height) {
                    eprintln!(
                        "[Frame {}] [sync_surface_from_arrangement] Entity={} resizing from {:?} to {}x{}",
                        frame_count.0, entity_name, surf.size, width, height
                    );
                    eprintln!(
                        "[Frame {}] [sync_surface_from_arrangement] Entity={} >>> SetContent calling",
                        frame_count.0, entity_name
                    );
                    match create_surface_for_window(&graphics, visual_graphics, width, height) {
                        Ok(new_surface) => {
                            eprintln!(
                                "[Frame {}] [sync_surface_from_arrangement] Entity={} Surface resized successfully",
                                frame_count.0, entity_name
                            );
                            commands.entity(entity).insert(new_surface);
                        }
                        Err(e) => {
                            eprintln!("[sync_surface_from_arrangement] エラー: {:?}", e);
                            surf.invalidate();
                        }
                    }
                }
            }
            None => {
                // Surfaceがまだ作成されていない場合は作成
                eprintln!(
                    "[Frame {}] [sync_surface_from_arrangement] Entity={} creating new Surface {}x{}",
                    frame_count.0, entity_name, width, height
                );
                eprintln!(
                    "[Frame {}] [sync_surface_from_arrangement] Entity={} >>> SetContent calling",
                    frame_count.0, entity_name
                );
                match create_surface_for_window(&graphics, visual_graphics, width, height) {
                    Ok(new_surface) => {
                        eprintln!(
                            "[Frame {}] [sync_surface_from_arrangement] Entity={} Surface created successfully",
                            frame_count.0, entity_name
                        );
                        commands.entity(entity).insert(new_surface);
                    }
                    Err(e) => {
                        eprintln!("[sync_surface_from_arrangement] エラー (新規作成): {:?}", e);
                    }
                }
            }
        }
    }
}

/// GlobalArrangementとArrangementからWindowPosの位置・サイズを更新
/// Windowコンポーネントを持つエンティティのみ処理
pub fn sync_window_pos(
    mut query: Query<
        (
            &GlobalArrangement,
            &crate::ecs::layout::Arrangement,
            &mut crate::ecs::window::WindowPos,
        ),
        (
            With<crate::ecs::window::Window>,
            Or<(
                Changed<GlobalArrangement>,
                Changed<crate::ecs::layout::Arrangement>,
            )>,
        ),
    >,
) {
    use windows::Win32::Foundation::{POINT, SIZE};

    for (global_arr, arrangement, mut window_pos) in query.iter_mut() {
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
            cx: arrangement.size.width as i32,
            cy: arrangement.size.height as i32,
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

/// 変更検知システム：描画内容に変更があった場合、そのEntityに更新要求マーカーを付与する
///
/// Phase 4以降の自己描画方式では、各EntityがSurfaceを持ち、自分自身のみを描画するため、
/// 親をたどる必要はない。変更があったEntity自身にマーカーを付与する。
///
/// **Note**: 将来的にはSurfaceUpdateRequestedを廃止し、Changed<GraphicsCommandList>を
/// render_surfaceのフィルターとして直接使用する予定。
pub fn mark_dirty_surfaces(
    mut commands: Commands,
    changed_query: Query<
        Entity,
        (
            Or<(Changed<GraphicsCommandList>, Changed<SurfaceGraphics>)>,
            With<SurfaceGraphics>,
        ),
    >,
) {
    for entity in changed_query.iter() {
        commands.entity(entity).insert(SurfaceUpdateRequested);
    }
}

/// ChildOf変更またはVisualGraphics追加を検出してVisual階層を同期するシステム (R3, R6, R7)
///
/// ECSのChildOf/Children階層とDirectCompositionのVisual階層を同期する。
/// Phase 6改善: parent_visual.is_none()で未同期を検出し、毎フレーム処理可能に。
/// - ChildOf追加/変更時: 子VisualGraphicsを親VisualGraphicsに追加
/// - VisualGraphics追加時: 既存のChildOf関係を元に親Visualに追加
/// - parent_visualがNone: 階層未構築なので同期実行
/// - 親がVisualGraphicsを持たない場合（LayoutRootなど）: Visual階層のルートとして処理済みマーク
///
/// 重要: 親→子の順序でAddVisualを呼ぶ必要がある（DirectComposition要件）。
/// 深さでソートして親が先に処理されるようにする。
///
/// 注意: エラーは無視する（親が先に削除されている場合など）
pub fn visual_hierarchy_sync_system(
    mut vg_queries: ParamSet<(
        // ChildOf + VisualGraphics を持つ全Entity
        Query<(Entity, &ChildOf, &mut VisualGraphics, Option<&Name>)>,
        // 親エンティティクエリ
        Query<(&VisualGraphics, Option<&Name>)>,
    )>,
    child_of_query: Query<&ChildOf>,
) {
    use crate::com::dcomp::DCompositionVisualExt;

    // 1. まず未同期（parent_visual==None）のエンティティと親情報とName情報を収集
    // (child_entity, parent_entity, child_name, parent_name, depth)
    let mut updates: Vec<(Entity, Entity, Option<String>, Option<String>, usize)> = Vec::new();
    {
        let child_query = vg_queries.p0();
        for (entity, child_of, child_vg, child_name) in child_query.iter() {
            // parent_visualがNoneなら未同期
            if child_vg.parent_visual().is_none() {
                let child_name_str = child_name.map(|n| n.to_string());
                // 深さを計算
                let mut depth = 0;
                let mut current = entity;
                while let Ok(co) = child_of_query.get(current) {
                    depth += 1;
                    current = co.parent();
                }
                updates.push((entity, child_of.parent(), child_name_str, None, depth));
            }
        }
    }

    // 親のNameを取得
    {
        let parent_query = vg_queries.p1();
        for item in updates.iter_mut() {
            let parent_entity = item.1;
            if let Ok((_, parent_name)) = parent_query.get(parent_entity) {
                item.3 = parent_name.map(|n| n.to_string());
            }
        }
    }

    // 2. 深さでソート（浅い＝親が先）
    updates.sort_by_key(|item| item.4);

    if !updates.is_empty() {
        eprintln!(
            "[visual_hierarchy_sync] Processing {} entities (sorted by depth)",
            updates.len()
        );
    }

    // 3. 各子エンティティに対して処理
    for (child_entity, parent_entity, child_name_str, parent_name_str, depth) in updates {
        // フォーマット済み名前を生成
        let child_fallback = format!("Entity({:?})", child_entity);
        let parent_fallback = format!("Entity({:?})", parent_entity);
        let child_display = child_name_str.as_deref().unwrap_or(&child_fallback);
        let parent_display = parent_name_str.as_deref().unwrap_or(&parent_fallback);

        // 親のVisualを取得
        let parent_visual = {
            let parent_query = vg_queries.p1();
            parent_query
                .get(parent_entity)
                .ok()
                .and_then(|(pv, _)| pv.visual().cloned())
        };

        // 子のVisualGraphicsを更新
        let mut child_query = vg_queries.p0();
        if let Ok((_, _, mut child_vg, _)) = child_query.get_mut(child_entity) {
            // 子のvisualを取得
            let child_visual = match child_vg.visual() {
                Some(v) => v.clone(),
                None => continue,
            };

            // 新しい親に追加
            if let Some(ref parent_visual) = parent_visual {
                // 親がVisualGraphicsを持つ場合: 親Visualに追加
                if let Err(e) = parent_visual.add_visual(&child_visual, false, None) {
                    eprintln!(
                        "[visual_hierarchy_sync] AddVisual failed: child=\"{}\" (depth={}), parent=\"{}\", error={:?}",
                        child_display, depth, parent_display, e
                    );
                } else {
                    eprintln!(
                        "[visual_hierarchy_sync] AddVisual success: child=\"{}\" (depth={}) -> parent=\"{}\"",
                        child_display, depth, parent_display
                    );
                }
                // parent_visualキャッシュを更新
                child_vg.set_parent_visual(Some(parent_visual.clone()));
            } else {
                // 親がVisualGraphicsを持たない場合: Visual階層のルート
                eprintln!(
                    "[visual_hierarchy_sync] Visual hierarchy root: name=\"{}\" (depth={})",
                    child_display, depth
                );
                // 自分自身のVisualを設定して「処理済み」とマーク
                child_vg.set_parent_visual(Some(child_visual.clone()));
            }
        }
    }
}

/// Visual プロパティ同期システム (R8)
///
/// ArrangementまたはOpacity変更を検知してVisualのプロパティを同期する。
/// - Arrangement.offset → Visual.SetOffsetX/SetOffsetY
/// - Opacity → Visual.SetOpacity
///
/// DirectCompositionのVisual Offsetは親Visual相対なので、Arrangement.offsetをそのまま使用。
///
/// Compositionスケジュールで実行。
pub fn visual_property_sync_system(
    changed_entities: Query<
        (
            Entity,
            &crate::ecs::layout::Arrangement,
            Option<&crate::ecs::layout::Opacity>,
            &VisualGraphics,
            Option<&Name>,
        ),
        Or<(
            Changed<crate::ecs::layout::Arrangement>,
            Changed<crate::ecs::layout::Opacity>,
        )>,
    >,
) {
    use crate::com::dcomp::DCompositionVisualExt;

    for (entity, arrangement, opacity_opt, vg, name) in changed_entities.iter() {
        let Some(visual) = vg.visual() else {
            continue;
        };

        let entity_name = format_entity_name(entity, name);

        // Offset同期: Arrangementのoffsetはローカル座標（親Entity相対）
        let offset_x = arrangement.offset.x;
        let offset_y = arrangement.offset.y;

        if let Err(e) = visual.set_offset_x(offset_x) {
            eprintln!(
                "[visual_property_sync] SetOffsetX failed for Entity={}: {:?}",
                entity_name, e
            );
        }
        if let Err(e) = visual.set_offset_y(offset_y) {
            eprintln!(
                "[visual_property_sync] SetOffsetY failed for Entity={}: {:?}",
                entity_name, e
            );
        }

        // Opacity同期: Opacityコンポーネントがあれば反映
        if let Some(opacity) = opacity_opt {
            let opacity_value = opacity.clamped();
            if let Err(e) = visual.set_opacity(opacity_value) {
                eprintln!(
                    "[visual_property_sync] SetOpacity failed for Entity={}: {:?}",
                    entity_name, e
                );
            }

            #[cfg(debug_assertions)]
            eprintln!(
                "[visual_property_sync] Entity={}, offset=({}, {}), opacity={}",
                entity_name, offset_x, offset_y, opacity_value
            );
        } else {
            #[cfg(debug_assertions)]
            eprintln!(
                "[visual_property_sync] Entity={}, offset=({}, {})",
                entity_name, offset_x, offset_y
            );
        }
    }
}

/// 遅延Surface作成システム (Phase 6)
///
/// CommandListが存在するEntityに対してSurfaceを遅延作成する。
/// - 対象: VisualGraphics + CommandList を持ち、SurfaceGraphicsを持たないEntity
/// - タイミング: Drawスケジュール（draw_rectangles/draw_labels の後）
/// - サイズ: Arrangementから取得（レイアウト済み）
///
/// これにより、Visual作成（PreLayout）とSurface作成（Draw）を分離し、
/// タイミング問題を解決する。
pub fn deferred_surface_creation_system(
    mut commands: Commands,
    graphics: Res<GraphicsCore>,
    query: Query<
        (
            Entity,
            &VisualGraphics,
            &GraphicsCommandList,
            Option<&crate::ecs::layout::Arrangement>,
            Option<&Name>,
        ),
        Without<SurfaceGraphics>,
    >,
) {
    use crate::com::dcomp::DCompositionDeviceExt;

    if !graphics.is_valid() {
        return;
    }

    let dcomp = match graphics.dcomp() {
        Some(d) => d,
        None => return,
    };

    for (entity, visual_graphics, _cmd_list, arrangement_opt, name) in query.iter() {
        let entity_name = format_entity_name(entity, name);
        // サイズをArrangementから取得（なければデフォルト1x1）
        let (width, height) = if let Some(arr) = arrangement_opt {
            let w = arr.size.width.max(1.0) as u32;
            let h = arr.size.height.max(1.0) as u32;
            (w.max(1), h.max(1))
        } else {
            (1, 1)
        };

        eprintln!(
            "[deferred_surface_creation] Creating Surface for Entity={}, size={}x{}",
            entity_name, width, height
        );

        // Surface作成
        let surface_res = dcomp.create_surface(
            width,
            height,
            DXGI_FORMAT_B8G8R8A8_UNORM,
            DXGI_ALPHA_MODE_PREMULTIPLIED,
        );

        match surface_res {
            Ok(surface) => {
                // VisualにSurfaceを設定
                if let Some(visual) = visual_graphics.visual() {
                    eprintln!(
                        "[deferred_surface_creation] Entity={} >>> SetContent calling",
                        entity_name
                    );
                    unsafe {
                        let _ = visual.SetContent(&surface);
                    }
                }
                commands
                    .entity(entity)
                    .insert(SurfaceGraphics::new(surface, (width, height)));
                // SurfaceUpdateRequestedも挿入して描画をトリガー
                commands
                    .entity(entity)
                    .insert(super::components::SurfaceUpdateRequested);
                eprintln!(
                    "[deferred_surface_creation] Surface created successfully for Entity={}",
                    entity_name
                );
            }
            Err(e) => {
                eprintln!(
                    "[deferred_surface_creation] Failed to create surface for Entity={}: {:?}",
                    entity_name, e
                );
            }
        }
    }
}
