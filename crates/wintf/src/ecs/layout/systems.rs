use crate::ecs::common::tree_system::{
    mark_dirty_trees, propagate_parent_transforms, sync_simple_transforms, NodeQuery, WorkQueue,
};
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::name::Name;
use bevy_ecs::prelude::*;

use super::metrics::{LayoutScale, Offset, Size};
use super::taffy::{TaffyComputedLayout, TaffyLayoutResource, TaffyStyle};
use super::{
    Arrangement, ArrangementTreeChanged, D2DRectExt, Dimension, GlobalArrangement, LayoutRoot,
};
use crate::ecs::graphics::format_entity_name;
use crate::ecs::window::{Window, WindowPos};
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
        eprintln!(
            "[propagate_global_arrangements] Root Entity={:?}, Arrangement: offset=({}, {}), scale=({}, {})",
            entity, arr.offset.x, arr.offset.y, arr.scale.x, arr.scale.y
        );
        eprintln!(
            "[propagate_global_arrangements] Root Entity={:?}, GlobalArrangement: transform=[{},{},{},{}],bounds=({},{},{},{})",
            entity, 
            global.transform.M11, global.transform.M12, global.transform.M31, global.transform.M32,
            global.bounds.left, global.bounds.top, global.bounds.right, global.bounds.bottom
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
        (
            Or<(With<LayoutRoot>, With<BoxStyle>)>,
            Without<TaffyStyle>,
        ),
    >,
    // BoxStyleが変更されたエンティティ
    mut changed: Query<(&BoxStyle, &mut TaffyStyle), Changed<BoxStyle>>,
) {
    // TaffyStyle自動挿入
    for (entity, box_style) in without_style.iter() {
        // BoxStyleがない場合（LayoutRootのみ）はデフォルトスタイル
        let taffy_style: taffy::Style = box_style
            .map(|s| s.into())
            .unwrap_or_default();
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
    changed_hierarchy: Query<(), Changed<ChildOf>>,
    // TaffyComputedLayoutを書き込むクエリ
    mut all_taffy_entities: Query<(Entity, &mut TaffyComputedLayout), With<TaffyStyle>>,
) {
    let has_changes = !changed_styles.is_empty() || !changed_hierarchy.is_empty();

    // Changed検知時にレイアウト計算を実行
    if has_changes {
        for (root_entity, box_style) in roots.iter() {
            if let Some(root_node) = taffy_res.get_node(root_entity) {
                // LayoutRootのBoxStyleからavailable_spaceを構築
                let available_space = if let Some(style) = box_style {
                    if let Some(size) = &style.size {
                        taffy::Size {
                            width: size.width.as_ref().map_or(
                                AvailableSpace::MaxContent,
                                |d| match d {
                                    Dimension::Px(px) => AvailableSpace::Definite(*px),
                                    _ => AvailableSpace::MaxContent,
                                },
                            ),
                            height: size.height.as_ref().map_or(
                                AvailableSpace::MaxContent,
                                |d| match d {
                                    Dimension::Px(px) => AvailableSpace::Definite(*px),
                                    _ => AvailableSpace::MaxContent,
                                },
                            ),
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

/// TaffyComputedLayoutの結果をArrangementに反映
pub fn update_arrangements_system(
    mut commands: Commands,
    mut query: Query<
        (Entity, &TaffyComputedLayout, Option<&mut Arrangement>, Option<&Name>),
        (Changed<TaffyComputedLayout>, With<TaffyStyle>),
    >,
) {
    for (entity, computed_layout, arrangement, name) in query.iter_mut() {
        let layout = &computed_layout.0;

        let new_arrangement = Arrangement {
            offset: Offset {
                x: layout.location.x,
                y: layout.location.y,
            },
            scale: LayoutScale::default(),
            size: Size {
                width: layout.size.width,
                height: layout.size.height,
            },
        };

        let entity_name = format_entity_name(entity, name);
        eprintln!(
            "[update_arrangements] Entity={}, TaffyLayout: location=({}, {}), size=({}, {})",
            entity_name, layout.location.x, layout.location.y, layout.size.width, layout.size.height
        );
        eprintln!(
            "[update_arrangements] Entity={}, Arrangement: offset=({}, {}), size=({}, {})",
            entity_name, new_arrangement.offset.x, new_arrangement.offset.y, new_arrangement.size.width, new_arrangement.size.height
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

// ===== Monitor階層管理システム =====

use super::{BoxInset, BoxPosition, BoxSize, LengthPercentageAuto};
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
    SM_YVIRTUALSCREEN,
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
    let existing = world.query_filtered::<Entity, With<LayoutRoot>>().iter(world).next();
    if existing.is_some() {
        return;
    }

    eprintln!("[initialize_layout_root] Creating LayoutRoot singleton");

    // 仮想デスクトップの矩形を取得
    let (vx, vy, vw, vh) = get_virtual_desktop_bounds();
    eprintln!(
        "[initialize_layout_root] Virtual desktop bounds: x={}, y={}, width={}, height={}",
        vx, vy, vw, vh
    );

    // LayoutRootエンティティを作成（仮想デスクトップ矩形を設定）
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
            Arrangement::default(),
            GlobalArrangement::default(),
        ))
        .id();

    // LayoutRoot用のTaffyノード作成
    {
        let mut taffy_res = world.resource_mut::<TaffyLayoutResource>();
        if let Err(e) = taffy_res.create_node(layout_root) {
            eprintln!("[initialize_layout_root] Failed to create Taffy node for LayoutRoot: {:?}", e);
            return;
        }
    }

    // 全モニターを列挙
    let monitors = crate::ecs::monitor::enumerate_monitors();
    eprintln!(
        "[initialize_layout_root] Enumerated {} monitors",
        monitors.len()
    );

    // 各Monitorエンティティを生成
    for monitor in monitors {
        let (width, height) = monitor.physical_size();
        let (left, top) = monitor.top_left();

        eprintln!(
            "[initialize_layout_root] Creating Monitor entity: bounds=({},{},{},{}), dpi={}, primary={}",
            monitor.bounds.left, monitor.bounds.top, monitor.bounds.right, monitor.bounds.bottom,
            monitor.dpi, monitor.is_primary
        );

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
                Arrangement::default(),
                GlobalArrangement::default(),
            ))
            .id();

        // Monitor用のTaffyノード作成
        let mut taffy_res = world.resource_mut::<TaffyLayoutResource>();
        if let Err(e) = taffy_res.create_node(monitor_entity) {
            eprintln!(
                "[initialize_layout_root] Failed to create Taffy node for Monitor: {:?}",
                e
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

        eprintln!(
            "[update_monitor_layout_system] Updating Monitor layout: size=({}, {}), position=({}, {})",
            width, height, left, top
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

    eprintln!("[detect_display_change_system] Display configuration changed, updating monitors");

    // LayoutRootを取得
    let Ok(root_entity) = layout_root.single() else {
        eprintln!("[detect_display_change_system] LayoutRoot not found, skipping");
        app.reset_display_change();
        return;
    };

    // 新しいモニターリストを取得
    let new_monitors = crate::ecs::monitor::enumerate_monitors();
    eprintln!(
        "[detect_display_change_system] Found {} monitors",
        new_monitors.len()
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
                eprintln!(
                    "[detect_display_change_system] Updating Monitor entity {:?}",
                    entity
                );
                if let Ok((_, mut monitor)) = existing_monitors.get_mut(entity) {
                    *monitor = new_monitor;
                }
            }
        } else {
            // 新規Monitorの追加
            eprintln!(
                "[detect_display_change_system] Adding new Monitor: handle={}",
                new_monitor.handle
            );

            let (width, height) = new_monitor.physical_size();
            let (left, top) = new_monitor.top_left();

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
                    Arrangement::default(),
                    GlobalArrangement::default(),
                ))
                .id();

            if let Err(e) = taffy_res.create_node(monitor_entity) {
                eprintln!(
                    "[detect_display_change_system] Failed to create Taffy node for new Monitor: {:?}",
                    e
                );
            }
        }
    }

    // 削除されたMonitorの処理
    for (entity, monitor) in existing_map.values() {
        eprintln!(
            "[detect_display_change_system] Removing Monitor entity {:?} (handle={})",
            entity, monitor.handle
        );
        if let Err(e) = taffy_res.remove_node(*entity) {
            eprintln!(
                "[detect_display_change_system] Failed to remove Taffy node: {:?}",
                e
            );
        }
        commands.entity(*entity).despawn();
    }

    // フラグをリセット
    app.reset_display_change();
}
