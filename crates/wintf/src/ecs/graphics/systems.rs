use super::command_list::GraphicsCommandList;
use crate::com::d2d::{D2D1DeviceContextExt, D2D1DeviceExt};
use crate::com::dcomp::{
    DCompositionDesktopDeviceExt, DCompositionDeviceExt, DCompositionSurfaceExt,
    DCompositionVisualExt,
};
use crate::ecs::graphics::{
    GraphicsCore, HasGraphicsResources, SurfaceCreationStats, SurfaceGraphics,
    SurfaceGraphicsDirty, VisualGraphics, WindowGraphics,
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

/// GlobalArrangementの境界から物理ピクセルサイズを計算する
///
/// # Arguments
/// * `global_arrangement` - GlobalArrangementコンポーネント
///
/// # Returns
/// * `Some((width, height))` - 有効なサイズ（幅と高さが両方1以上）
/// * `None` - 無効なサイズ（幅または高さが0以下）
///
/// # Requirements
/// - Req 3.1: GlobalArrangement.boundsから計算（物理ピクセルサイズ）
/// - Req 3.2: スケール適用後のサイズ（小数点切り上げ）
/// - Req 3.3: サイズ0の場合はNone
pub fn calculate_surface_size_from_global_arrangement(
    global_arrangement: &GlobalArrangement,
) -> Option<(u32, u32)> {
    let width = global_arrangement.bounds.right - global_arrangement.bounds.left;
    let height = global_arrangement.bounds.bottom - global_arrangement.bounds.top;

    // サイズが0以下の場合はNone
    if width <= 0.0 || height <= 0.0 {
        return None;
    }

    // 小数点以下を切り上げて物理ピクセルサイズを計算
    let width_px = width.ceil() as u32;
    let height_px = height.ceil() as u32;

    // 最低1ピクセルを保証
    if width_px == 0 || height_px == 0 {
        return None;
    }

    Some((width_px, height_px))
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
fn create_surface_for_visual(
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
/// - Changed<SurfaceGraphicsDirty>を持つEntityが対象（マーカー方式から移行）
/// - 自分のGraphicsCommandListのみを描画（子は描画しない）
/// - 子の描画は各子が自分のSurfaceで行う
/// - GlobalArrangementのスケール成分を適用（DPIスケール対応）
pub fn render_surface(
    surfaces: Query<
        (
            Entity,
            &SurfaceGraphics,
            &GlobalArrangement,
            Option<&GraphicsCommandList>,
            Option<&Name>,
        ),
        Changed<super::components::SurfaceGraphicsDirty>,
    >,
    _graphics_core: Option<Res<GraphicsCore>>,
    _frame_count: Res<crate::ecs::world::FrameCount>,
) {
    use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
    use windows::Win32::Graphics::Direct2D::Common::D2D1_COMPOSITE_MODE_SOURCE_OVER;
    use windows::Win32::Graphics::Direct2D::D2D1_INTERPOLATION_MODE_LINEAR;

    for (entity, surface, global_arrangement, cmd_list_opt, name) in surfaces.iter() {
        let entity_name = format_entity_name(entity, name);
        // 正常パスのログは抑制（毎フレーム出力されるため）
        // eprintln!("[Frame {}] [render_surface] === Self-rendering Entity={} ===", _frame_count.0, entity_name);

        if !surface.is_valid() {
            // Surface未初期化時のみログ出力
            // eprintln!("[render_surface] Surface invalid for Entity={}, skipping", entity_name);
            continue;
        }

        // Surface描画開始
        let surface_ref = match surface.surface() {
            Some(s) => s,
            None => continue,
        };

        let (dc, offset) = match surface_ref.begin_draw(None) {
            Ok(result) => {
                // 正常パスのログは抑制
                // eprintln!("[render_surface] BeginDraw succeeded for Entity={}, offset=({}, {})", entity_name, result.1.x, result.1.y);
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
        // - offset: Surface内の描画開始位置（物理ピクセル単位）- DirectCompositionがSurface Atlasを使用している場合に非ゼロ
        // - scale: GlobalArrangementから抽出したスケール成分（DPIスケール含む）
        // - Visual.SetOffsetX/Yで位置は設定済みなので、ここでは描画開始位置とスケールのみ
        unsafe {
            // GlobalArrangementからスケール成分のみを抽出
            let scale_x = global_arrangement.scale_x();
            let scale_y = global_arrangement.scale_y();

            // 変換順序: まずスケール、次にoffset平行移動
            // これによりスケール後の座標系でoffset位置に描画される
            let transform = windows_numerics::Matrix3x2 {
                M11: scale_x,
                M12: 0.0,
                M21: 0.0,
                M22: scale_y,
                M31: offset.x as f32,
                M32: offset.y as f32,
            };

            dc.SetTransform(&transform);

            // 正常パスのログは抑制
            // eprintln!("[render_surface] Transform for Entity={}: scale=({}, {}), offset=({}, {})", entity_name, scale_x, scale_y, offset.x, offset.y);
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
                // 正常パスのログは抑制
                // eprintln!("[render_surface] Drawing own CommandList for Entity={}", entity_name);
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

        // Note: Changed<SurfaceGraphicsDirty> は自動リセットされるため、
        // マーカー削除は不要（旧SurfaceUpdateRequestedパターンから移行済み）
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
            // 正常時はログ出力を抑制（毎フレーム出力されるため）
            // デバッグ時はコメントを外して有効化
            // eprintln!("[Frame {}] [commit_composition] Commit成功", frame_count.0);
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
///
/// GraphicsCoreが無効な場合に再作成し、HasGraphicsResourcesを持つ全エンティティに
/// 初期化リクエストを発行する。
///
/// Changed: GraphicsNeedsInitマーカー挿入から、HasGraphicsResources.request_init()に移行
pub fn init_graphics_core(
    graphics: Option<ResMut<GraphicsCore>>,
    mut query: Query<&mut HasGraphicsResources>,
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
                    eprintln!("[Frame {}] [init_graphics_core] {}個のエンティティにrequest_init()を呼び出し", frame_count.0, count);
                    for mut res in query.iter_mut() {
                        res.request_init();
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
                    eprintln!("[Frame {}] [init_graphics_core] {}個のエンティティにrequest_init()を呼び出し", frame_count.0, count);
                    for mut res in query.iter_mut() {
                        res.request_init();
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
///
/// Changed: GraphicsNeedsInitマーカーから、Changed<HasGraphicsResources> + needs_init()に移行
pub fn init_window_graphics(
    graphics: Res<GraphicsCore>,
    mut query: Query<
        (
            Entity,
            &crate::ecs::window::WindowHandle,
            &HasGraphicsResources,
            Option<&mut WindowGraphics>,
            Option<&Name>,
        ),
        Or<(Without<WindowGraphics>, Changed<HasGraphicsResources>)>,
    >,
    mut commands: Commands,
    frame_count: Res<crate::ecs::world::FrameCount>,
) {
    if !graphics.is_valid() {
        return;
    }

    for (entity, handle, res, window_graphics, name) in query.iter_mut() {
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
                // needs_init()がtrueの場合のみ再初期化
                if res.needs_init() && !wg.is_valid() {
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
///
/// Changed: GraphicsNeedsInitマーカーから、Changed<HasGraphicsResources> + needs_init()に移行
/// 現在はvisual_resource_management_systemが担当
pub fn init_window_visual(
    _graphics: Res<GraphicsCore>,
    _query: Query<
        (
            Entity,
            &WindowGraphics,
            &HasGraphicsResources,
            Option<&mut VisualGraphics>,
        ),
        Or<(Without<VisualGraphics>, Changed<HasGraphicsResources>)>,
    >,
    _commands: Commands,
    _frame_count: Res<crate::ecs::world::FrameCount>,
) {
    // Deprecated: Visual creation is now handled by visual_resource_management_system
}

// ========== Layout-to-Graphics Synchronization Systems ==========

/// Arrangement変更時にDirectComposition Surfaceを作成/リサイズ
/// ArrangementをSingle Source of Truthとして直接参照
///
/// # Deprecated
/// この関数は `deferred_surface_creation_system` に置き換えられました。
/// `deferred_surface_creation_system` は `GlobalArrangement.bounds` から
/// 物理ピクセルサイズを計算し、GraphicsCommandListの存在を条件として
/// Surface生成を行います。
///
/// Req 2.1: sync_surface_from_arrangement廃止
#[deprecated(
    since = "0.1.0",
    note = "Use deferred_surface_creation_system instead. This function will be removed in a future version."
)]
#[allow(dead_code)]
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
                    match create_surface_for_visual(&graphics, visual_graphics, width, height) {
                        Ok(new_surface) => {
                            eprintln!(
                                "[Frame {}] [sync_surface_from_arrangement] Entity={} Surface resized successfully",
                                frame_count.0, entity_name
                            );
                            commands.entity(entity).insert(new_surface);
                            // SurfaceGraphicsDirtyも挿入して描画をトリガー
                            commands
                                .entity(entity)
                                .insert(super::components::SurfaceGraphicsDirty::default());
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
                match create_surface_for_visual(&graphics, visual_graphics, width, height) {
                    Ok(new_surface) => {
                        eprintln!(
                            "[Frame {}] [sync_surface_from_arrangement] Entity={} Surface created successfully",
                            frame_count.0, entity_name
                        );
                        commands.entity(entity).insert(new_surface);
                        // SurfaceGraphicsDirtyも挿入して描画をトリガー
                        commands
                            .entity(entity)
                            .insert(super::components::SurfaceGraphicsDirty::default());
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
        // 小数点以下を切り上げて物理ピクセルサイズを計算
        // SurfaceGraphicsと同様にceilを使用
        let new_size = SIZE {
            cx: arrangement.size.width.ceil() as i32,
            cy: arrangement.size.height.ceil() as i32,
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

/// WindowPos変更時にSetWindowPosコマンドをキューに追加
///
/// クライアント領域座標をウィンドウ全体座標に変換してからコマンドを生成する。
/// `WindowPosChanged.0 == true` の場合は、WM_WINDOWPOSCHANGED由来の変更なので
/// SetWindowPosコマンドを生成しない（フィードバックループ防止）。
pub fn apply_window_pos_changes(
    mut query: Query<
        (
            Entity,
            &crate::ecs::window::WindowHandle,
            &mut crate::ecs::window::WindowPos,
            &crate::ecs::window::WindowPosChanged,
            Option<&Name>,
        ),
        (
            Changed<crate::ecs::window::WindowPos>,
            With<crate::ecs::window::Window>,
        ),
    >,
) {
    for (entity, window_handle, mut window_pos, wpc, name) in query.iter_mut() {
        let entity_name = format_entity_name(entity, name);

        // WindowPosChangedフラグがtrueの場合、WM_WINDOWPOSCHANGED由来の変更なのでスキップ
        // これにより、フィードバックループを防止する
        if wpc.0 {
            eprintln!(
                "[apply_window_pos_changes] Entity={}: WindowPosChanged=true, suppressing SetWindowPos",
                entity_name
            );
            continue;
        }

        // エコーバックチェック
        let position = window_pos.position.unwrap_or_default();
        let size = window_pos.size.unwrap_or_default();

        if window_pos.is_echo(position, size) {
            eprintln!(
                "[apply_window_pos_changes] Entity={}: Echo-back detected, skipping. pos=({}, {}), size=({}, {})",
                entity_name, position.x, position.y, size.cx, size.cy
            );
            continue;
        }

        // CW_USEDEFAULTが設定されている場合はスキップ（ウィンドウ作成時の初期値）
        // 座標変換をスキップし、ウィンドウ作成時の初期配置を優先
        if position.x == windows::Win32::UI::WindowsAndMessaging::CW_USEDEFAULT
            || size.cx == windows::Win32::UI::WindowsAndMessaging::CW_USEDEFAULT
        {
            eprintln!(
                "[apply_window_pos_changes] Entity={}: CW_USEDEFAULT detected, skipping. pos=({}, {}), size=({}, {})",
                entity_name, position.x, position.y, size.cx, size.cy
            );
            continue;
        }

        // クライアント領域座標をウィンドウ全体座標に変換
        let (x, y, width, height) = match window_pos.to_window_coords(window_handle) {
            Ok(coords) => coords,
            Err(e) => {
                // 変換失敗時はフォールバック：元の座標を使用
                eprintln!(
                    "[apply_window_pos_changes] Entity={}: Failed to transform window coordinates: {}. Using original values.",
                    entity_name, e
                );
                (position.x, position.y, size.cx, size.cy)
            }
        };

        // SetWindowPosコマンドを生成してキューに追加
        // 直接SetWindowPosを呼び出さない（World借用競合防止）
        let flags = window_pos.build_flags_for_system();
        let hwnd_insert_after = window_pos.get_hwnd_insert_after();

        eprintln!(
            "[apply_window_pos_changes] Entity={}: Enqueueing SetWindowPosCommand. client=({}, {}), size=({}, {}) -> window=({}, {}), size=({}, {})",
            entity_name, position.x, position.y, size.cx, size.cy, x, y, width, height
        );

        let cmd = crate::ecs::window::SetWindowPosCommand::new(
            window_handle.hwnd,
            x,
            y,
            width,
            height,
            flags,
            hwnd_insert_after,
        );
        crate::ecs::window::SetWindowPosCommand::enqueue(cmd);

        // last_sent値を記録（クライアント座標で記録）
        // WM_WINDOWPOSCHANGEDでの比較時に逆変換して一致するため
        let bypass = window_pos.bypass_change_detection();
        bypass.last_sent_position = Some((position.x, position.y));
        bypass.last_sent_size = Some((size.cx, size.cy));
        eprintln!(
            "[apply_window_pos_changes] Entity={}: Command enqueued, recorded client coords ({}, {}), ({}, {})",
            entity_name, position.x, position.y, size.cx, size.cy
        );
    }
}

/// GraphicsNeedsInitマーカー削除・初期化完了判定
///
/// Changed: GraphicsNeedsInitマーカー削除から、mark_initialized()に移行
pub fn cleanup_graphics_needs_init(
    mut query: Query<(
        Entity,
        &mut HasGraphicsResources,
        &WindowGraphics,
        &VisualGraphics,
        &SurfaceGraphics,
    )>,
) {
    for (entity, mut res, window_graphics, visual, surface) in query.iter_mut() {
        // needs_init()がtrueで、すべてのリソースが有効な場合に初期化完了をマーク
        if res.needs_init() && window_graphics.is_valid() && visual.is_valid() && surface.is_valid()
        {
            eprintln!(
                "[cleanup_graphics_needs_init] mark_initialized() (Entity: {:?})",
                entity
            );
            res.mark_initialized();
        }
    }
}

/// 再初期化時に古いGraphicsCommandListを削除
///
/// Changed: GraphicsNeedsInitマーカーから、needs_init()条件に移行
pub fn cleanup_command_list_on_reinit(
    query: Query<(Entity, &HasGraphicsResources), With<crate::ecs::graphics::GraphicsCommandList>>,
    mut commands: Commands,
) {
    for (entity, res) in query.iter() {
        if res.needs_init() {
            commands
                .entity(entity)
                .remove::<crate::ecs::graphics::GraphicsCommandList>();
            eprintln!(
                "[cleanup_command_list_on_reinit] GraphicsCommandList削除 (Entity: {:?})",
                entity
            );
        }
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

/// 変更検知システム：描画内容に変更があった場合、SurfaceGraphicsDirtyを更新する
///
/// Phase 4以降の自己描画方式では、各EntityがSurfaceを持ち、自分自身のみを描画するため、
/// 親をたどる必要はない。変更があったEntity自身のSurfaceGraphicsDirtyを更新する。
///
/// 検知対象:
/// - GraphicsCommandList: 描画コマンドの変更
/// - SurfaceGraphics: Surfaceの再作成（Added含む）
/// - GlobalArrangement: スケール成分の変更（DPIスケール対応）
///
/// Changed<SurfaceGraphicsDirty>を検出することで、render_surfaceが描画を実行する。
/// SurfaceUpdateRequestedマーカーは廃止され、フレーム番号更新方式に置き換えられた。
pub fn mark_dirty_surfaces(
    mut changed_query: Query<
        (
            Entity,
            &mut super::components::SurfaceGraphicsDirty,
            Option<&Name>,
        ),
        (
            Or<(
                Changed<GraphicsCommandList>,
                Changed<SurfaceGraphics>,
                Added<SurfaceGraphics>,
                Changed<GlobalArrangement>,
            )>,
            With<SurfaceGraphics>,
        ),
    >,
    frame_count: Res<crate::ecs::world::FrameCount>,
) {
    let mut count = 0;
    for (entity, mut dirty, name) in changed_query.iter_mut() {
        dirty.requested_frame = frame_count.0 as u64;
        count += 1;
        // 正常パスのログは抑制（毎フレーム出力されるため）
        let _entity_name = format_entity_name(entity, name);
        // eprintln!("[Frame {}] [mark_dirty_surfaces] Entity={} marked dirty", frame_count.0, _entity_name);
    }
    if count > 0 {
        // 正常パスのログは抑制（毎フレーム出力されるため）
        // eprintln!("[Frame {}] [mark_dirty_surfaces] Total {} entities marked dirty", frame_count.0, count);
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
/// DirectCompositionのVisual Offsetは物理ピクセル単位で指定する必要がある。
/// Arrangement.offset（論理座標）にGlobalArrangement.transform（累積DPIスケール）を適用して物理座標に変換。
///
/// Compositionスケジュールで実行。
pub fn visual_property_sync_system(
    changed_entities: Query<
        (
            Entity,
            &crate::ecs::layout::Arrangement,
            &crate::ecs::layout::GlobalArrangement,
            Option<&crate::ecs::layout::Opacity>,
            &VisualGraphics,
            Option<&Name>,
            Has<crate::ecs::window::Window>,
        ),
        Or<(
            Changed<crate::ecs::layout::Arrangement>,
            Changed<crate::ecs::layout::GlobalArrangement>,
            Changed<crate::ecs::layout::Opacity>,
        )>,
    >,
) {
    use crate::com::dcomp::DCompositionVisualExt;

    for (entity, arrangement, global_arrangement, opacity_opt, vg, name, is_window) in
        changed_entities.iter()
    {
        let Some(visual) = vg.visual() else {
            continue;
        };

        let entity_name = format_entity_name(entity, name);

        // Offset同期: Arrangementのoffset（論理座標）にGlobalArrangementのscale（累積DPIスケール）を適用
        // ただし、WindowエンティティはWin32がCompositionTargetを通じて位置を管理するため、
        // offsetを設定すると二重にオフセットが適用されてしまう。Windowはスキップする。
        if !is_window {
            // GlobalArrangementから累積スケールを取得（親からのDPIスケールを含む）
            let scale_x = global_arrangement.scale_x();
            let scale_y = global_arrangement.scale_y();
            // 論理座標 × 累積スケール = 物理ピクセル座標
            let offset_x = arrangement.offset.x * scale_x;
            let offset_y = arrangement.offset.y * scale_y;

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

            // 正常パスのログは抑制（毎フレーム出力されるため）
            // #[cfg(debug_assertions)]
            // eprintln!("[visual_property_sync] Entity={}, offset=({}, {}) [physical], scale=({}, {})", entity_name, offset_x, offset_y, scale_x, scale_y);
        } else {
            // 正常パスのログは抑制
            // #[cfg(debug_assertions)]
            // eprintln!("[visual_property_sync] Entity={} (Window): offset skipped", entity_name);
        }

        // Opacity同期: Opacityコンポーネントがあれば反映（Windowも含む）
        if let Some(opacity) = opacity_opt {
            let opacity_value = opacity.clamped();
            if let Err(e) = visual.set_opacity(opacity_value) {
                eprintln!(
                    "[visual_property_sync] SetOpacity failed for Entity={}: {:?}",
                    entity_name, e
                );
            }

            // 正常パスのログは抑制
            // #[cfg(debug_assertions)]
            // eprintln!("[visual_property_sync] Entity={}, opacity={}", entity_name, opacity_value);
        }
    }
}

/// 遅延Surface作成システム (最適化版)
///
/// GraphicsCommandListが存在するEntityに対してSurfaceを条件付きで作成する。
/// - 対象: VisualGraphics + GraphicsCommandList を持つEntity
/// - サイズ: GlobalArrangement.boundsから物理ピクセルサイズを計算
/// - 条件: サイズが有効（幅・高さが1以上）な場合のみ作成
/// - サイズ変更: 既存Surfaceとサイズ不一致の場合は再作成
///
/// # Requirements
/// - Req 1.1: CommandList追加時にSurface作成
/// - Req 1.2: CommandListなしならスキップ（クエリ条件で実現）
/// - Req 2.2: deferred_surface_creation唯一化
/// - Req 2.3: トリガーをCommandList存在のみに
/// - Req 3.1: GlobalArrangement.boundsから計算
/// - Req 3.2: スケール適用後のサイズ（物理ピクセル）
/// - Req 3.3: サイズ0の場合はスキップ
/// - Req 3.4: サイズ変更時にSurface再作成
/// - Req 5.1: スキップ理由ログ
/// - Req 5.2: 作成ログ（物理サイズ）
///
/// Note: SurfaceGraphicsは visual_resource_management_system で事前に空で配置されているため、
/// ここでは直接更新（set_surface）する。commands.insert() は使用しない。
pub fn deferred_surface_creation_system(
    graphics: Res<GraphicsCore>,
    // 統合クエリ: SurfaceGraphicsを持ち、GlobalArrangementまたはGraphicsCommandListが変更されたEntity
    // SurfaceGraphicsは事前配置されている前提
    mut query: Query<
        (
            Entity,
            &VisualGraphics,
            &GraphicsCommandList,
            &GlobalArrangement,
            &mut SurfaceGraphics,
            &mut SurfaceGraphicsDirty,
            Option<&Name>,
        ),
        Or<(Changed<GlobalArrangement>, Changed<GraphicsCommandList>)>,
    >,
    mut stats: ResMut<SurfaceCreationStats>,
) {
    use crate::com::dcomp::DCompositionDeviceExt;

    if !graphics.is_valid() {
        return;
    }

    let dcomp = match graphics.dcomp() {
        Some(d) => d,
        None => return,
    };

    for (
        entity,
        visual_graphics,
        _cmd_list,
        global_arrangement,
        mut surface_graphics,
        mut dirty,
        name,
    ) in query.iter_mut()
    {
        let entity_name = format_entity_name(entity, name);

        // GlobalArrangementからサイズを計算
        let Some((width, height)) =
            calculate_surface_size_from_global_arrangement(global_arrangement)
        else {
            // Req 5.1: スキップ理由ログ
            eprintln!(
                "[deferred_surface_creation] Entity={} skipped: invalid size from GlobalArrangement (bounds={:?})",
                entity_name, global_arrangement.bounds
            );
            stats.record_skipped();
            continue;
        };

        // サイズが同じなら何もしない（既にSurfaceが有効な場合）
        if surface_graphics.is_valid() && surface_graphics.size == (width, height) {
            continue;
        }

        // 新規作成かリサイズかを判定
        let is_new = !surface_graphics.is_valid();

        if is_new {
            eprintln!(
                "[deferred_surface_creation] Creating Surface for Entity={}, physical_size={}x{}",
                entity_name, width, height
            );
        } else {
            eprintln!(
                "[deferred_surface_creation] Resizing Surface for Entity={}: {:?} -> {}x{}",
                entity_name, surface_graphics.size, width, height
            );
        }

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
                        match visual.SetContent(&surface) {
                            Ok(_) => eprintln!(
                                "[deferred_surface_creation] Entity={} >>> SetContent SUCCESS",
                                entity_name
                            ),
                            Err(e) => eprintln!(
                                "[deferred_surface_creation] Entity={} >>> SetContent FAILED: {:?}",
                                entity_name, e
                            ),
                        }
                    }
                } else {
                    eprintln!(
                        "[deferred_surface_creation] Entity={} >>> NO VISUAL! SetContent skipped",
                        entity_name
                    );
                }

                // 直接更新（commands.insert()ではなく）
                surface_graphics.set_surface(surface, (width, height));
                // SurfaceGraphicsDirtyのChangedをトリガー
                dirty.requested_frame = dirty.requested_frame.wrapping_add(1);

                if is_new {
                    stats.record_created();
                    eprintln!(
                        "[deferred_surface_creation] Surface created successfully for Entity={}",
                        entity_name
                    );
                } else {
                    stats.record_resized();
                    eprintln!(
                        "[deferred_surface_creation] Surface resized successfully for Entity={}",
                        entity_name
                    );
                }
            }
            Err(e) => {
                eprintln!(
                    "[deferred_surface_creation] Failed to create/resize surface for Entity={}: {:?}",
                    entity_name, e
                );
            }
        }
    }
}

/// GraphicsCommandList削除時のSurface解放システム
///
/// GraphicsCommandListが削除されたEntityからSurfaceGraphicsをクリアする。
/// VisualGraphicsはVisual階層を維持するため削除しない。
/// SurfaceGraphicsコンポーネント自体は残し、内容をinvalidate()する。
///
/// # Requirements
/// - Req 1.3: CommandList削除時にSurface解放
/// - Req 1.4: 専用クリーンアップシステム
///
/// Note: commands.remove()ではなくinvalidate()を使用し、
/// コンポーネントの存在自体は維持する（事前配置パターン）。
pub fn cleanup_surface_on_commandlist_removed(
    mut removed: RemovedComponents<GraphicsCommandList>,
    mut query: Query<(Entity, &VisualGraphics, &mut SurfaceGraphics, Option<&Name>)>,
    mut stats: ResMut<SurfaceCreationStats>,
) {
    for entity in removed.read() {
        // SurfaceGraphicsを持つEntityのみ処理
        if let Ok((entity, visual_graphics, mut surface_graphics, name)) = query.get_mut(entity) {
            // 既にinvalidな場合はスキップ
            if !surface_graphics.is_valid() {
                continue;
            }

            let entity_name = format_entity_name(entity, name);

            eprintln!(
                "[cleanup_surface_on_commandlist_removed] Clearing SurfaceGraphics for Entity={}",
                entity_name
            );

            // VisualのContentをクリア（Req 1.3）
            if let Some(visual) = visual_graphics.visual() {
                unsafe {
                    // nullptrを設定してSurfaceを解除
                    let _ = visual.SetContent(None);
                }
            }

            // SurfaceGraphicsをクリア（コンポーネント自体は残す）
            surface_graphics.clear();

            stats.record_deleted();

            eprintln!(
                "[cleanup_surface_on_commandlist_removed] SurfaceGraphics cleared for Entity={}",
                entity_name
            );
        }
    }
}
