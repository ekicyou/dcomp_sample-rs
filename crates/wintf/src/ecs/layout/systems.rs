use crate::ecs::common::tree_system::{
    mark_dirty_trees, propagate_parent_transforms, sync_simple_transforms, NodeQuery, WorkQueue,
};
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::prelude::*;

use super::metrics::{LayoutScale, Offset, Size};
use super::taffy::{TaffyComputedLayout, TaffyLayoutResource, TaffyStyle};
use super::{
    Arrangement, ArrangementTreeChanged, BoxMargin, BoxPadding, BoxSize, D2DRectExt, Dimension,
    FlexContainer, FlexItem, GlobalArrangement, LayoutRoot,
};
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

/// 高レベルレイアウトコンポーネントからTaffyStyleを構築
pub fn build_taffy_styles_system(
    mut commands: Commands,
    // TaffyStyleがないエンティティを検出（LayoutRootまたはレイアウト関連コンポーネントを持つ）
    without_style: Query<
        (
            Entity,
            Option<&BoxSize>,
            Option<&BoxMargin>,
            Option<&BoxPadding>,
            Option<&FlexContainer>,
            Option<&FlexItem>,
        ),
        (
            Or<(
                With<LayoutRoot>,
                With<BoxSize>,
                With<BoxMargin>,
                With<BoxPadding>,
                With<FlexContainer>,
                With<FlexItem>,
            )>,
            Without<TaffyStyle>,
        ),
    >,
    // 変更された高レベルコンポーネントを持つエンティティ
    mut changed: Query<
        (
            Entity,
            Option<&BoxSize>,
            Option<&BoxMargin>,
            Option<&BoxPadding>,
            Option<&FlexContainer>,
            Option<&FlexItem>,
            &mut TaffyStyle,
        ),
        Or<(
            Changed<BoxSize>,
            Changed<BoxMargin>,
            Changed<BoxPadding>,
            Changed<FlexContainer>,
            Changed<FlexItem>,
        )>,
    >,
) {
    // TaffyStyleを自動挿入（既存のコンポーネントから初期化）
    for (entity, box_size, box_margin, box_padding, flex_container, flex_item) in
        without_style.iter()
    {
        let mut style = Style::default();

        // BoxSize: width/height
        if let Some(size) = box_size {
            if let Some(width) = size.width {
                style.size.width = width.into();
            }
            if let Some(height) = size.height {
                style.size.height = height.into();
            }
        }

        // BoxMargin: margin
        if let Some(margin) = box_margin {
            style.margin = margin.0.into();
        }

        // BoxPadding: padding
        if let Some(padding) = box_padding {
            style.padding = padding.0.into();
        }

        // FlexContainer: display, flex_direction, justify_content, align_items
        if let Some(container) = flex_container {
            style.display = Display::Flex;
            style.flex_direction = container.direction;
            if let Some(justify) = container.justify_content {
                style.justify_content = Some(justify);
            }
            if let Some(align) = container.align_items {
                style.align_items = Some(align);
            }
        }

        // FlexItem: flex_grow, flex_shrink, flex_basis, align_self
        if let Some(item) = flex_item {
            style.flex_grow = item.grow;
            style.flex_shrink = item.shrink;
            style.flex_basis = item.basis.into();
            if let Some(align_self) = item.align_self {
                style.align_self = Some(align_self);
            }
        }

        commands.entity(entity).insert((
            TaffyStyle(style),
            TaffyComputedLayout::default(),
            ArrangementTreeChanged,
        ));
    }

    // 高レベルコンポーネントからTaffyStyleを構築
    for (_entity, box_size, box_margin, box_padding, flex_container, flex_item, mut taffy_style) in
        changed.iter_mut()
    {
        let mut style = Style::default();

        // BoxSize: width/height
        if let Some(size) = box_size {
            if let Some(width) = size.width {
                style.size.width = width.into();
            }
            if let Some(height) = size.height {
                style.size.height = height.into();
            }
        }

        // BoxMargin: margin
        if let Some(margin) = box_margin {
            style.margin = margin.0.into();
        }

        // BoxPadding: padding
        if let Some(padding) = box_padding {
            style.padding = padding.0.into();
        }

        // FlexContainer: display, flex_direction, justify_content, align_items
        if let Some(container) = flex_container {
            style.display = Display::Flex;
            style.flex_direction = container.direction;
            if let Some(justify) = container.justify_content {
                style.justify_content = Some(justify);
            }
            if let Some(align) = container.align_items {
                style.align_items = Some(align);
            }
        }

        // FlexItem: flex_grow, flex_shrink, flex_basis, align_self
        if let Some(item) = flex_item {
            style.flex_grow = item.grow;
            style.flex_shrink = item.shrink;
            style.flex_basis = item.basis.into();
            if let Some(align_self) = item.align_self {
                style.align_self = Some(align_self);
            }
        }

        taffy_style.0 = style;
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
        // TODO: 親から子を削除する処理
        // 現在この処理を有効にすると taffy_flex_layout_pure_test が壊れる
        // 原因を特定してから有効化する
        if let Some(_node_id) = taffy_res.get_node(entity) {
            // if let Some(parent_node) = taffy_res.taffy().parent(node_id) {
            //     let _ = taffy_res.taffy_mut().remove_child(parent_node, node_id);
            // }
        }
    }
}

/// Taffyレイアウト計算を実行
pub fn compute_taffy_layout_system(
    mut taffy_res: ResMut<TaffyLayoutResource>,
    // LayoutRootマーカーを持つエンティティをレイアウトルートとして扱う
    roots: Query<(Entity, Option<&BoxSize>), With<LayoutRoot>>,
    // 変更検知用
    changed_styles: Query<(), Changed<TaffyStyle>>,
    changed_hierarchy: Query<(), Changed<ChildOf>>,
    // TaffyComputedLayoutを書き込むクエリ
    mut all_taffy_entities: Query<(Entity, &mut TaffyComputedLayout), With<TaffyStyle>>,
) {
    let has_changes = !changed_styles.is_empty() || !changed_hierarchy.is_empty();

    // Changed検知時にレイアウト計算を実行
    if has_changes {
        for (root_entity, box_size) in roots.iter() {
            if let Some(root_node) = taffy_res.get_node(root_entity) {
                // LayoutRootのBoxSizeからavailable_spaceを構築
                let available_space = if let Some(size) = box_size {
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
        (Entity, &TaffyComputedLayout, Option<&mut Arrangement>),
        (Changed<TaffyComputedLayout>, With<TaffyStyle>),
    >,
) {
    for (entity, computed_layout, arrangement) in query.iter_mut() {
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

        eprintln!(
            "[update_arrangements] Entity={:?}, TaffyLayout: location=({}, {}), size=({}, {})",
            entity, layout.location.x, layout.location.y, layout.size.width, layout.size.height
        );
        eprintln!(
            "[update_arrangements] Entity={:?}, Arrangement: offset=({}, {}), scale=({}, {})",
            entity, new_arrangement.offset.x, new_arrangement.offset.y, new_arrangement.scale.x, new_arrangement.scale.y
        );

        if let Some(mut arr) = arrangement {
            // 値比較で変更検知を抑制
            if *arr != new_arrangement {
                *arr = new_arrangement;
                // Arrangementの変更でChanged<Arrangement>が検知される
                // ArrangementTreeChangedはmark_dirty_arrangement_treesで適切に処理される
            }
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
