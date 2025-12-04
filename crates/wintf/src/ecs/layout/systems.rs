use crate::ecs::common::tree_system::{
    mark_dirty_trees, propagate_parent_transforms, sync_simple_transforms, NodeQuery, WorkQueue,
};
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::name::Name;
use bevy_ecs::prelude::*;
use tracing::{debug, error, info, trace, warn};

use super::metrics::{LayoutScale, Offset, Size};
use super::taffy::{TaffyComputedLayout, TaffyLayoutResource, TaffyStyle};
use super::{
    Arrangement, ArrangementTreeChanged, D2DRectExt, Dimension, GlobalArrangement, LayoutRoot,
};
use crate::ecs::graphics::format_entity_name;
use crate::ecs::window::{Window, WindowPos, DPI};
use taffy::prelude::*;

/// 階層に属していないEntity（ルートWindow）のGlobalArrangementを更新
pub fn sync_simple_arrangements(
    query: ParamSet<(
        Query<
            (&Arrangement, &mut GlobalArrangement),
            (
                Or<(Changed<Arrangement>, Added<GlobalArrangement>)>,
                Without<ChildOf>,
                Without<Children>,
            ),
        >,
        Query<(Ref<Arrangement>, &mut GlobalArrangement), (Without<ChildOf>, Without<Children>)>,
    )>,
    orphaned: RemovedComponents<ChildOf>,
) {
    sync_simple_transforms::<Arrangement, GlobalArrangement, ArrangementTreeChanged>(
        query, orphaned,
    );
}

/// 「ダーティビット」を階層の祖先に向かって伝播
pub fn mark_dirty_arrangement_trees(
    changed: Query<
        Entity,
        Or<(
            Changed<Arrangement>,
            Changed<ChildOf>,
            Added<GlobalArrangement>,
        )>,
    >,
    orphaned: RemovedComponents<ChildOf>,
    transforms: Query<(Option<&ChildOf>, &mut ArrangementTreeChanged)>,
) {
    let changed_count = changed.iter().count();
    if changed_count > 0 {
        // Note: 頻繁に呼ばれるためログは抑制
        // tracing::info!(
        //     changed_count,
        //     "[mark_dirty_arrangement_trees] Detected changed arrangements"
        // );
    }
    mark_dirty_trees::<Arrangement, GlobalArrangement, ArrangementTreeChanged>(
        changed, orphaned, transforms,
    );
}

/// 親から子へGlobalArrangementを伝播
pub fn propagate_global_arrangements(
    queue: Local<WorkQueue>,
    roots: Query<
        (Entity, Ref<Arrangement>, &mut GlobalArrangement, &Children),
        (Without<ChildOf>, Changed<ArrangementTreeChanged>),
    >,
    nodes: NodeQuery<Arrangement, GlobalArrangement, ArrangementTreeChanged>,
) {
    // デバッグログ: ルートエンティティの情報
    for (entity, arr, global, _) in roots.iter() {
        trace!(
            entity = ?entity,
            offset_x = arr.offset.x,
            offset_y = arr.offset.y,
            scale_x = arr.scale.x,
            scale_y = arr.scale.y,
            "[propagate_global_arrangements] Root Entity Arrangement"
        );
        trace!(
            entity = ?entity,
            m11 = global.transform.M11,
            m12 = global.transform.M12,
            m31 = global.transform.M31,
            m32 = global.transform.M32,
            bounds_left = global.bounds.left,
            bounds_top = global.bounds.top,
            bounds_right = global.bounds.right,
            bounds_bottom = global.bounds.bottom,
            "[propagate_global_arrangements] Root Entity GlobalArrangement"
        );
    }

    propagate_parent_transforms::<Arrangement, GlobalArrangement, ArrangementTreeChanged>(
        queue, roots, nodes,
    );
}

// ===== Taffyレイアウトシステム =====

use super::BoxStyle;

/// BoxStyleからTaffyStyleを構築するシステム（統合後）
///
/// # 変更点（BoxStyle統合）
/// - 旧: 8コンポーネント（BoxSize, BoxMargin, BoxPadding, BoxPosition, BoxInset, FlexContainer, FlexItem, LayoutRoot）
/// - 新: BoxStyle + LayoutRoot の2コンポーネントのみ
///
/// # 動作
/// 1. BoxStyleまたはLayoutRootを持ち、TaffyStyleがないエンティティにTaffyStyleを自動挿入
/// 2. BoxStyleが変更されたエンティティのTaffyStyleを更新
/// 3. LayoutRootのみでBoxStyleがないエンティティにはBoxStyle::default()相当のスタイルを適用
pub fn build_taffy_styles_system(
    mut commands: Commands,
    // LayoutRootまたはBoxStyleがあるがTaffyStyleがないエンティティ
    // LayoutRootのみの場合はBoxStyle::default()相当のスタイルを適用
    without_style: Query<
        (Entity, Option<&BoxStyle>),
        (Or<(With<LayoutRoot>, With<BoxStyle>)>, Without<TaffyStyle>),
    >,
    // BoxStyleが変更されたエンティティ
    mut changed: Query<(&BoxStyle, &mut TaffyStyle), Changed<BoxStyle>>,
) {
    // TaffyStyle自動挿入
    for (entity, box_style) in without_style.iter() {
        // BoxStyleがない場合（LayoutRootのみ）はデフォルトスタイル
        let taffy_style: taffy::Style = box_style.map(|s| s.into()).unwrap_or_default();
        commands.entity(entity).insert((
            TaffyStyle(taffy_style),
            TaffyComputedLayout::default(),
            ArrangementTreeChanged,
        ));
    }

    // 変更反映
    for (box_style, mut taffy_style) in changed.iter_mut() {
        taffy_style.0 = box_style.into();
    }
}

/// TaffyツリーをECS階層と同期
pub fn sync_taffy_tree_system(
    mut taffy_res: ResMut<TaffyLayoutResource>,
    // 新規エンティティ（TaffyStyleが追加されたがノードがまだ作成されていない）
    new_entities: Query<(Entity, Option<&ChildOf>), Added<TaffyStyle>>,
    // TaffyStyleが変更されたエンティティ
    changed_styles: Query<(Entity, &TaffyStyle), Changed<TaffyStyle>>,
    // 階層が変更されたエンティティ
    changed_hierarchy: Query<(Entity, Option<&ChildOf>), Changed<ChildOf>>,
    // ChildOfが削除されたエンティティ
    mut removed_hierarchy: RemovedComponents<ChildOf>,
) {
    // 新規エンティティにtaffyノードを作成
    for (entity, _) in new_entities.iter() {
        if taffy_res.get_node(entity).is_none() {
            let _ = taffy_res.create_node(entity);
        }
    }

    // TaffyStyleの変更をtaffyツリーに反映
    for (entity, style) in changed_styles.iter() {
        if let Some(node_id) = taffy_res.get_node(entity) {
            let _ = taffy_res.taffy_mut().set_style(node_id, style.0.clone());
        }
    }

    // 階層変更を処理（新規エンティティの親子関係もここで設定）
    for (entity, child_of) in changed_hierarchy.iter() {
        if let Some(node_id) = taffy_res.get_node(entity) {
            if let Some(parent_ref) = child_of {
                let parent_entity = parent_ref.parent();
                if let Some(parent_node) = taffy_res.get_node(parent_entity) {
                    // 新しい親に追加（taffyが自動的に既存の親から削除する）
                    let _ = taffy_res.taffy_mut().add_child(parent_node, node_id);
                }
            }
        }
    }

    // ChildOfが削除された場合の処理
    for entity in removed_hierarchy.read() {
        if let Some(node_id) = taffy_res.get_node(entity) {
            if let Some(parent_node) = taffy_res.taffy().parent(node_id) {
                let _ = taffy_res.taffy_mut().remove_child(parent_node, node_id);
            }
        }
    }
}

/// Taffyレイアウト計算を実行
pub fn compute_taffy_layout_system(
    mut taffy_res: ResMut<TaffyLayoutResource>,
    // LayoutRootマーカーを持つエンティティをレイアウトルートとして扱う
    roots: Query<(Entity, Option<&BoxStyle>), With<LayoutRoot>>,
    // 変更検知用
    changed_styles: Query<(), Changed<TaffyStyle>>,
    changed_box_styles: Query<(), Changed<BoxStyle>>,
    changed_hierarchy: Query<(), Changed<ChildOf>>,
    // TaffyComputedLayoutを書き込むクエリ
    mut all_taffy_entities: Query<(Entity, &mut TaffyComputedLayout), With<TaffyStyle>>,
) {
    // BoxStyleまたはTaffyStyleの変更、階層変更のいずれかで再計算
    let has_changes = !changed_styles.is_empty() 
        || !changed_box_styles.is_empty() 
        || !changed_hierarchy.is_empty();

    // Changed検知時にレイアウト計算を実行
    if has_changes {
        for (root_entity, box_style) in roots.iter() {
            if let Some(root_node) = taffy_res.get_node(root_entity) {
                // LayoutRootのBoxStyleからavailable_spaceを構築
                let available_space = if let Some(style) = box_style {
                    if let Some(size) = &style.size {
                        taffy::Size {
                            width: size.width.as_ref().map_or(AvailableSpace::MaxContent, |d| {
                                match d {
                                    Dimension::Px(px) => AvailableSpace::Definite(*px),
                                    _ => AvailableSpace::MaxContent,
                                }
                            }),
                            height: size
                                .height
                                .as_ref()
                                .map_or(AvailableSpace::MaxContent, |d| match d {
                                    Dimension::Px(px) => AvailableSpace::Definite(*px),
                                    _ => AvailableSpace::MaxContent,
                                }),
                        }
                    } else {
                        taffy::Size {
                            width: AvailableSpace::MaxContent,
                            height: AvailableSpace::MaxContent,
                        }
                    }
                } else {
                    taffy::Size {
                        width: AvailableSpace::MaxContent,
                        height: AvailableSpace::MaxContent,
                    }
                };

                let result = taffy_res
                    .taffy_mut()
                    .compute_layout(root_node, available_space);

                // 計算成功時、全エンティティにTaffyComputedLayoutを書き込む
                if result.is_ok() {
                    for (entity, mut computed) in all_taffy_entities.iter_mut() {
                        if let Some(node_id) = taffy_res.get_node(entity) {
                            if let Ok(layout) = taffy_res.taffy().layout(node_id) {
                                let new_layout = TaffyComputedLayout(*layout);
                                // 値比較で変更検知を抑制
                                if *computed != new_layout {
                                    tracing::info!(
                                        entity = ?entity,
                                        old_width = computed.0.size.width,
                                        old_height = computed.0.size.height,
                                        new_width = new_layout.0.size.width,
                                        new_height = new_layout.0.size.height,
                                        "[compute_taffy_layout] Layout changed"
                                    );
                                    *computed = new_layout;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// TaffyComputedLayoutまたはDPIの変更をArrangementに反映
///
/// # Triggers
/// - TaffyComputedLayoutの追加・変更
/// - DPIコンポーネントの変更
///
/// # Behavior
/// どちらのトリガーでも全フィールド（offset, size, scale）を再計算
pub fn update_arrangements_system(
    mut commands: Commands,
    mut query: Query<
        (
            Entity,
            &TaffyComputedLayout,
            Option<&mut Arrangement>,
            Option<&Name>,
            Option<Ref<DPI>>,
        ),
        (
            Or<(Changed<TaffyComputedLayout>, Changed<DPI>)>,
            With<TaffyStyle>,
        ),
    >,
) {
    for (entity, computed_layout, arrangement, name, dpi) in query.iter_mut() {
        let layout = &computed_layout.0;

        // DPIが存在する場合はスケールファクターを使用、なければデフォルト(1.0, 1.0)
        let scale = dpi
            .as_ref()
            .map_or(LayoutScale::default(), |d| LayoutScale {
                x: d.scale_x(),
                y: d.scale_y(),
            });

        // デバッグ: DPI変更検知の確認
        if let Some(ref d) = dpi {
            if d.is_changed() {
                debug!(entity = ?entity, "[update_arrangements] DPI is_changed=true");
            }
        }

        let new_arrangement = Arrangement {
            offset: Offset {
                x: layout.location.x,
                y: layout.location.y,
            },
            scale,
            size: Size {
                width: layout.size.width,
                height: layout.size.height,
            },
        };

        let entity_name = format_entity_name(entity, name);
        trace!(
            entity = %entity_name,
            location_x = layout.location.x,
            location_y = layout.location.y,
            size_width = layout.size.width,
            size_height = layout.size.height,
            "[update_arrangements] TaffyLayout"
        );
        trace!(
            entity = %entity_name,
            offset_x = new_arrangement.offset.x,
            offset_y = new_arrangement.offset.y,
            scale_x = new_arrangement.scale.x,
            scale_y = new_arrangement.scale.y,
            size_width = new_arrangement.size.width,
            size_height = new_arrangement.size.height,
            "[update_arrangements] Arrangement"
        );

        if let Some(mut arr) = arrangement {
            arr.set_if_neq(new_arrangement);
        } else {
            commands.entity(entity).insert(new_arrangement);
        }
    }
}

/// 削除されたエンティティのTaffyノードをクリーンアップ
pub fn cleanup_removed_entities_system(
    mut taffy_res: ResMut<TaffyLayoutResource>,
    mut removed: RemovedComponents<TaffyStyle>,
) {
    for entity in removed.read() {
        let _ = taffy_res.remove_node(entity);
    }
}

/// GlobalArrangementの変更をWindowPosに反映
pub fn update_window_pos_system(
    mut query: Query<
        (&GlobalArrangement, &mut WindowPos),
        (With<Window>, Changed<GlobalArrangement>),
    >,
) {
    use windows::Win32::Foundation::{POINT, SIZE};

    for (global_arrangement, mut window_pos) in query.iter_mut() {
        // GlobalArrangementのboundsからWindowPosに変換
        let bounds = &global_arrangement.bounds;

        // boundsの位置とサイズをWindowPosに反映
        // Windowは LayoutRoot の子であり、Taffy が BoxInset を考慮した
        // location を計算済み
        window_pos.position = Some(POINT {
            x: bounds.left as i32,
            y: bounds.top as i32,
        });
        window_pos.size = Some(SIZE {
            cx: bounds.width() as i32,
            cy: bounds.height() as i32,
        });
    }
}

/// WindowPos.position の変更を Window の Arrangement.offset に反映
/// 
/// WM_WINDOWPOSCHANGED で更新された WindowPos.position（クライアント領域のスクリーン座標）を
/// Window の Arrangement.offset に反映する。
/// 
/// これにより GlobalArrangement.bounds が正しいスクリーン座標を持つようになり、
/// hit_test が正しく動作する。
/// 
/// 毎フレーム Window の WindowPos と Arrangement.offset を同期する。
/// 変更がない場合は何もしない。
pub fn sync_window_arrangement_from_window_pos(
    mut query: Query<
        (Entity, &WindowPos, &DPI, &mut Arrangement, Option<&Name>),
        With<Window>,
    >,
) {
    use crate::ecs::graphics::format_entity_name;
    
    for (entity, window_pos, dpi, mut arrangement, name) in query.iter_mut() {
        let Some(position) = window_pos.position else {
            continue;
        };

        // CW_USEDEFAULT の場合はスキップ（ウィンドウ作成時の初期値）
        if position.x == windows::Win32::UI::WindowsAndMessaging::CW_USEDEFAULT {
            continue;
        }

        // WindowPos.position は物理ピクセル座標
        // Arrangement.offset は DIP 座標なので、DPI スケールで割る
        let scale_x = dpi.scale_x();
        let scale_y = dpi.scale_y();

        // ゼロ除算防止
        if scale_x <= 0.0 || scale_y <= 0.0 {
            continue;
        }

        let new_offset = Offset {
            x: position.x as f32 / scale_x,
            y: position.y as f32 / scale_y,
        };

        // 変更があった場合のみ更新
        if arrangement.offset != new_offset {
            let entity_name = format_entity_name(entity, name);
            tracing::debug!(
                entity = %entity_name,
                old_x = arrangement.offset.x,
                old_y = arrangement.offset.y,
                new_x = new_offset.x,
                new_y = new_offset.y,
                position_x = position.x,
                position_y = position.y,
                scale_x,
                scale_y,
                "[sync_window_arrangement_from_window_pos] Updating Arrangement.offset"
            );
            arrangement.offset = new_offset;
        }
    }
}

// ===== Monitor階層管理システム =====

use super::{BoxInset, BoxPosition, BoxSize, LengthPercentageAuto};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN,
};

/// 仮想デスクトップの矩形を取得
///
/// # 戻り値
/// (x, y, width, height) - 仮想デスクトップの左上座標とサイズ
pub fn get_virtual_desktop_bounds() -> (i32, i32, i32, i32) {
    unsafe {
        let x = GetSystemMetrics(SM_XVIRTUALSCREEN);
        let y = GetSystemMetrics(SM_YVIRTUALSCREEN);
        let width = GetSystemMetrics(SM_CXVIRTUALSCREEN);
        let height = GetSystemMetrics(SM_CYVIRTUALSCREEN);
        (x, y, width, height)
    }
}

/// LayoutRootとMonitor階層をワールド初期化時に作成する
/// world.rsのEcsWorld::new()から直接呼び出される
pub fn initialize_layout_root(world: &mut World) {
    // 既にLayoutRootが存在する場合はスキップ
    let existing = world
        .query_filtered::<Entity, With<LayoutRoot>>()
        .iter(world)
        .next();
    if existing.is_some() {
        return;
    }

    info!("[initialize_layout_root] Creating LayoutRoot singleton");

    // 仮想デスクトップの矩形を取得
    let (vx, vy, vw, vh) = get_virtual_desktop_bounds();
    debug!(
        x = vx,
        y = vy,
        width = vw,
        height = vh,
        "[initialize_layout_root] Virtual desktop bounds"
    );

    // LayoutRootエンティティを作成（仮想デスクトップ矩形を設定）
    // Note: Arrangement/GlobalArrangementはLayoutRoot::on_addフックで自動挿入される
    let layout_root = world
        .spawn((
            LayoutRoot,
            BoxStyle {
                size: Some(BoxSize {
                    width: Some(Dimension::Px(vw as f32)),
                    height: Some(Dimension::Px(vh as f32)),
                }),
                position: Some(BoxPosition::Absolute),
                inset: Some(BoxInset(super::Rect {
                    left: LengthPercentageAuto::Px(vx as f32),
                    top: LengthPercentageAuto::Px(vy as f32),
                    right: LengthPercentageAuto::Auto,
                    bottom: LengthPercentageAuto::Auto,
                })),
                ..Default::default()
            },
        ))
        .id();

    // LayoutRoot用のTaffyノード作成
    {
        let mut taffy_res = world.resource_mut::<TaffyLayoutResource>();
        if let Err(e) = taffy_res.create_node(layout_root) {
            error!(error = ?e, "[initialize_layout_root] Failed to create Taffy node for LayoutRoot");
            return;
        }
    }

    // 全モニターを列挙
    let monitors = crate::ecs::monitor::enumerate_monitors();
    debug!(
        count = monitors.len(),
        "[initialize_layout_root] Enumerated monitors"
    );

    // 各Monitorエンティティを生成
    for monitor in monitors {
        let (width, height) = monitor.physical_size();
        let (left, top) = monitor.top_left();

        debug!(
            bounds_left = monitor.bounds.left,
            bounds_top = monitor.bounds.top,
            bounds_right = monitor.bounds.right,
            bounds_bottom = monitor.bounds.bottom,
            dpi = monitor.dpi,
            is_primary = monitor.is_primary,
            "[initialize_layout_root] Creating Monitor entity"
        );

        // Note: Arrangement/GlobalArrangementはMonitor::on_addフックで自動挿入される
        let monitor_entity = world
            .spawn((
                monitor,
                ChildOf(layout_root),
                BoxStyle {
                    size: Some(BoxSize {
                        width: Some(Dimension::Px(width)),
                        height: Some(Dimension::Px(height)),
                    }),
                    position: Some(BoxPosition::Absolute),
                    inset: Some(BoxInset(super::Rect {
                        left: LengthPercentageAuto::Px(left),
                        top: LengthPercentageAuto::Px(top),
                        right: LengthPercentageAuto::Auto,
                        bottom: LengthPercentageAuto::Auto,
                    })),
                    ..Default::default()
                },
            ))
            .id();

        // Monitor用のTaffyノード作成
        let mut taffy_res = world.resource_mut::<TaffyLayoutResource>();
        if let Err(e) = taffy_res.create_node(monitor_entity) {
            error!(
                error = ?e,
                "[initialize_layout_root] Failed to create Taffy node for Monitor"
            );
        }
    }
}

/// Monitorの情報が変更された際に、レイアウトコンポーネントを更新
pub fn update_monitor_layout_system(
    mut query: Query<(&crate::ecs::Monitor, &mut BoxStyle), Changed<crate::ecs::Monitor>>,
) {
    for (monitor, mut box_style) in query.iter_mut() {
        let (width, height) = monitor.physical_size();
        let (left, top) = monitor.top_left();

        debug!(
            width = width,
            height = height,
            left = left,
            top = top,
            "[update_monitor_layout_system] Updating Monitor layout"
        );

        box_style.size = Some(BoxSize {
            width: Some(Dimension::Px(width)),
            height: Some(Dimension::Px(height)),
        });
        box_style.inset = Some(BoxInset(super::Rect {
            left: LengthPercentageAuto::Px(left),
            top: LengthPercentageAuto::Px(top),
            right: LengthPercentageAuto::Auto,
            bottom: LengthPercentageAuto::Auto,
        }));
    }
}

/// ディスプレイ構成変更を検知し、Monitorエンティティを更新
pub fn detect_display_change_system(
    mut commands: Commands,
    mut app: ResMut<crate::ecs::App>,
    layout_root: Query<Entity, With<LayoutRoot>>,
    mut existing_monitors: Query<(Entity, &mut crate::ecs::Monitor), With<crate::ecs::Monitor>>,
    mut taffy_res: ResMut<TaffyLayoutResource>,
) {
    // ディスプレイ構成変更フラグをチェック
    if !app.display_configuration_changed() {
        return;
    }

    info!("[detect_display_change_system] Display configuration changed, updating monitors");

    // LayoutRootを取得
    let Ok(root_entity) = layout_root.single() else {
        warn!("[detect_display_change_system] LayoutRoot not found, skipping");
        app.reset_display_change();
        return;
    };

    // 新しいモニターリストを取得
    let new_monitors = crate::ecs::monitor::enumerate_monitors();
    debug!(
        count = new_monitors.len(),
        "[detect_display_change_system] Found monitors"
    );

    // 既存のMonitorエンティティをマップに変換（handle → entity）
    let mut existing_map: std::collections::HashMap<isize, (Entity, crate::ecs::Monitor)> =
        existing_monitors
            .iter()
            .map(|(e, m)| (m.handle, (e, m.clone())))
            .collect();

    // 新規・更新Monitorの処理
    for new_monitor in new_monitors {
        if let Some((entity, existing_monitor)) = existing_map.remove(&new_monitor.handle) {
            // 既存Monitorの更新
            if existing_monitor != new_monitor {
                debug!(
                    entity = ?entity,
                    "[detect_display_change_system] Updating Monitor entity"
                );
                if let Ok((_, mut monitor)) = existing_monitors.get_mut(entity) {
                    *monitor = new_monitor;
                }
            }
        } else {
            // 新規Monitorの追加
            debug!(
                handle = new_monitor.handle,
                "[detect_display_change_system] Adding new Monitor"
            );

            let (width, height) = new_monitor.physical_size();
            let (left, top) = new_monitor.top_left();

            // Note: Arrangement/GlobalArrangementはMonitor::on_addフックで自動挿入される
            let monitor_entity = commands
                .spawn((
                    new_monitor,
                    ChildOf(root_entity),
                    BoxStyle {
                        size: Some(BoxSize {
                            width: Some(Dimension::Px(width)),
                            height: Some(Dimension::Px(height)),
                        }),
                        position: Some(BoxPosition::Absolute),
                        inset: Some(BoxInset(super::Rect {
                            left: LengthPercentageAuto::Px(left),
                            top: LengthPercentageAuto::Px(top),
                            right: LengthPercentageAuto::Auto,
                            bottom: LengthPercentageAuto::Auto,
                        })),
                        ..Default::default()
                    },
                ))
                .id();

            if let Err(e) = taffy_res.create_node(monitor_entity) {
                error!(
                    error = ?e,
                    "[detect_display_change_system] Failed to create Taffy node for new Monitor"
                );
            }
        }
    }

    // 削除されたMonitorの処理
    for (entity, monitor) in existing_map.values() {
        debug!(
            entity = ?entity,
            handle = monitor.handle,
            "[detect_display_change_system] Removing Monitor entity"
        );
        if let Err(e) = taffy_res.remove_node(*entity) {
            error!(
                error = ?e,
                "[detect_display_change_system] Failed to remove Taffy node"
            );
        }
        commands.entity(*entity).despawn();
    }

    // フラグをリセット
    app.reset_display_change();
}
