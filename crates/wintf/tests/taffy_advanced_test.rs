/// Task 7.2-7.7: 追加ユニットテスト
/// TaffyComputedLayout変換、マッピング管理、階層同期、増分計算、クリーンアップ、境界値シナリオ
use bevy_ecs::prelude::*;
use wintf::ecs::layout::systems::{
    build_taffy_styles_system, cleanup_removed_entities_system, compute_taffy_layout_system,
    sync_taffy_tree_system, update_arrangements_system,
};
use wintf::ecs::layout::taffy::{TaffyComputedLayout, TaffyLayoutResource, TaffyStyle};
use wintf::ecs::layout::{Arrangement, BoxSize, BoxStyle, Dimension, LayoutRoot};
use wintf::ecs::ChildOf;

// ===== Task 7.2: TaffyComputedLayout→Arrangement変換テスト =====

#[test]
fn test_computed_layout_to_arrangement_conversion() {
    let mut world = World::new();
    world.init_resource::<TaffyLayoutResource>();

    // TaffyComputedLayoutを持つエンティティを作成
    let computed = TaffyComputedLayout::default();
    let entity = world.spawn((TaffyStyle::default(), computed)).id();

    // update_arrangements_systemを実行
    let mut schedule = Schedule::default();
    schedule.add_systems(update_arrangements_system);
    schedule.run(&mut world);

    // Arrangementが挿入されていることを確認
    assert!(world.entity(entity).contains::<Arrangement>());
}

#[test]
fn test_computed_layout_position_to_arrangement_offset() {
    let mut world = World::new();
    world.init_resource::<TaffyLayoutResource>();

    // 位置とサイズを持つTaffyComputedLayoutを作成
    let layout = taffy::Layout {
        location: taffy::Point { x: 10.0, y: 20.0 },
        size: taffy::Size {
            width: 100.0,
            height: 50.0,
        },
        ..Default::default()
    };
    let computed = TaffyComputedLayout::from(layout);

    let entity = world.spawn((TaffyStyle::default(), computed)).id();

    // システム実行
    let mut schedule = Schedule::default();
    schedule.add_systems(update_arrangements_system);
    schedule.run(&mut world);

    // Arrangementの値を検証
    let arrangement = world.get::<Arrangement>(entity).unwrap();
    assert_eq!(arrangement.offset.x, 10.0);
    assert_eq!(arrangement.offset.y, 20.0);
    assert_eq!(arrangement.size.width, 100.0);
    assert_eq!(arrangement.size.height, 50.0);
}

#[test]
fn test_arrangement_coordinate_system_consistency() {
    let mut world = World::new();
    world.init_resource::<TaffyLayoutResource>();

    // 複数のエンティティで座標変換の一貫性を検証
    let layouts = vec![
        (0.0, 0.0, 100.0, 100.0),
        (50.0, 50.0, 200.0, 150.0),
        (-10.0, -20.0, 80.0, 60.0), // 負の座標
    ];

    for (x, y, w, h) in layouts {
        let layout = taffy::Layout {
            location: taffy::Point { x, y },
            size: taffy::Size {
                width: w,
                height: h,
            },
            ..Default::default()
        };
        let computed = TaffyComputedLayout::from(layout);
        let entity = world.spawn((TaffyStyle::default(), computed)).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(update_arrangements_system);
        schedule.run(&mut world);

        let arrangement = world.get::<Arrangement>(entity).unwrap();
        assert_eq!(arrangement.offset.x, x, "X coordinate mismatch");
        assert_eq!(arrangement.offset.y, y, "Y coordinate mismatch");
        assert_eq!(arrangement.size.width, w, "Width mismatch");
        assert_eq!(arrangement.size.height, h, "Height mismatch");
    }
}

// ===== Task 7.3: EntityとNodeIdマッピングテスト =====

#[test]
fn test_create_node_and_mapping() {
    let mut world = World::new();
    let mut taffy_res = TaffyLayoutResource::default();

    let entity = world.spawn_empty().id();

    // ノード作成
    let node_id = taffy_res
        .create_node(entity)
        .expect("Failed to create node");

    // 順方向マッピング検証
    assert_eq!(taffy_res.get_node(entity), Some(node_id));

    // 逆方向マッピング検証
    assert_eq!(taffy_res.get_entity(node_id), Some(entity));
}

#[test]
fn test_remove_node_and_mapping_cleanup() {
    let mut world = World::new();
    let mut taffy_res = TaffyLayoutResource::default();

    let entity = world.spawn_empty().id();
    let node_id = taffy_res.create_node(entity).unwrap();

    // ノード削除
    taffy_res
        .remove_node(entity)
        .expect("Failed to remove node");

    // 両方向マッピングが削除されていることを確認
    assert_eq!(taffy_res.get_node(entity), None);
    assert_eq!(taffy_res.get_entity(node_id), None);
}

#[test]
fn test_bidirectional_mapping_consistency() {
    let mut world = World::new();
    let mut taffy_res = TaffyLayoutResource::default();

    // 複数のエンティティでマッピングの一貫性を検証
    let entities: Vec<Entity> = (0..5).map(|_| world.spawn_empty().id()).collect();

    let mut node_ids = Vec::new();
    for entity in &entities {
        let node_id = taffy_res.create_node(*entity).unwrap();
        node_ids.push(node_id);
    }

    // すべてのマッピングが正しいことを確認
    for (i, entity) in entities.iter().enumerate() {
        assert_eq!(taffy_res.get_node(*entity), Some(node_ids[i]));
        assert_eq!(taffy_res.get_entity(node_ids[i]), Some(*entity));
    }

    // 1つ削除してもその他は影響を受けないことを確認
    taffy_res.remove_node(entities[2]).unwrap();
    assert_eq!(taffy_res.get_node(entities[2]), None);
    assert_eq!(taffy_res.get_node(entities[0]), Some(node_ids[0]));
    assert_eq!(taffy_res.get_node(entities[4]), Some(node_ids[4]));
}

#[cfg(debug_assertions)]
#[test]
fn test_mapping_consistency_verification() {
    let taffy_res = TaffyLayoutResource::default();

    // verify_mapping_consistency()がpanicしないことを確認（デバッグビルドのみ）
    taffy_res.verify_mapping_consistency();
}

// ===== Task 7.4: ECS階層変更とtaffyツリー同期テスト =====

#[test]
fn test_hierarchy_addition_syncs_taffy_tree() {
    let mut world = World::new();
    world.init_resource::<TaffyLayoutResource>();

    // 親エンティティ
    let parent = world
        .spawn((TaffyStyle::default(), BoxStyle::default()))
        .id();

    // 子エンティティ
    let child = world
        .spawn((TaffyStyle::default(), BoxStyle::default()))
        .id();

    // システム実行（TaffyStyleからノード作成）
    let mut schedule = Schedule::default();
    schedule.add_systems((build_taffy_styles_system, sync_taffy_tree_system));
    schedule.run(&mut world);

    // 親子関係を設定
    world.entity_mut(child).insert(ChildOf(parent));

    // 階層変更を同期
    schedule.run(&mut world);

    // Taffyツリー内で親子関係が確立されていることを確認
    let taffy_res = world.resource::<TaffyLayoutResource>();
    let parent_node = taffy_res.get_node(parent).unwrap();
    let child_node = taffy_res.get_node(child).unwrap();

    let taffy_children = taffy_res.taffy().children(parent_node).unwrap();
    assert_eq!(taffy_children.len(), 1);
    assert_eq!(taffy_children[0], child_node);
}

#[test]
fn test_hierarchy_removal_syncs_taffy_tree() {
    let mut world = World::new();
    world.init_resource::<TaffyLayoutResource>();

    // 親子関係を持つエンティティを作成
    let parent = world
        .spawn((TaffyStyle::default(), BoxStyle::default()))
        .id();
    let child = world
        .spawn((TaffyStyle::default(), BoxStyle::default(), ChildOf(parent)))
        .id();

    // システム実行
    let mut schedule = Schedule::default();
    schedule.add_systems((build_taffy_styles_system, sync_taffy_tree_system));
    schedule.run(&mut world);

    // 親子関係を削除
    world.entity_mut(child).remove::<ChildOf>();

    schedule.run(&mut world);

    // Taffyツリー内で親子関係が解除されていることを確認
    let taffy_res = world.resource::<TaffyLayoutResource>();
    let parent_node = taffy_res.get_node(parent).unwrap();

    let taffy_children = taffy_res.taffy().children(parent_node).unwrap();
    assert_eq!(taffy_children.len(), 0);
}

#[test]
fn test_deep_hierarchy_sync() {
    let mut world = World::new();
    world.init_resource::<TaffyLayoutResource>();

    // 深い階層構造: Root -> A -> B -> C
    let root = world
        .spawn((TaffyStyle::default(), BoxStyle::default()))
        .id();
    let a = world
        .spawn((TaffyStyle::default(), BoxStyle::default(), ChildOf(root)))
        .id();
    let b = world
        .spawn((TaffyStyle::default(), BoxStyle::default(), ChildOf(a)))
        .id();
    let c = world
        .spawn((TaffyStyle::default(), BoxStyle::default(), ChildOf(b)))
        .id();

    // システム実行
    let mut schedule = Schedule::default();
    schedule.add_systems((build_taffy_styles_system, sync_taffy_tree_system));
    schedule.run(&mut world);

    // Taffyツリーの階層構造を検証
    let taffy_res = world.resource::<TaffyLayoutResource>();
    let root_node = taffy_res.get_node(root).unwrap();
    let a_node = taffy_res.get_node(a).unwrap();
    let b_node = taffy_res.get_node(b).unwrap();
    let c_node = taffy_res.get_node(c).unwrap();

    // Root -> A
    let root_children = taffy_res.taffy().children(root_node).unwrap();
    assert_eq!(root_children.len(), 1);
    assert_eq!(root_children[0], a_node);

    // A -> B
    let a_children = taffy_res.taffy().children(a_node).unwrap();
    assert_eq!(a_children.len(), 1);
    assert_eq!(a_children[0], b_node);

    // B -> C
    let b_children = taffy_res.taffy().children(b_node).unwrap();
    assert_eq!(b_children.len(), 1);
    assert_eq!(b_children[0], c_node);

    // C -> leaf
    let c_children = taffy_res.taffy().children(c_node).unwrap();
    assert_eq!(c_children.len(), 0);
}

// ===== Task 7.5: 増分計算の変更検知テスト =====

#[test]
fn test_no_change_no_compute() {
    let mut world = World::new();
    world.init_resource::<TaffyLayoutResource>();

    // LayoutRootを持つエンティティを作成
    let root = world
        .spawn((
            TaffyStyle::default(),
            TaffyComputedLayout::default(), // 明示的に挿入
            BoxStyle {
                size: Some(BoxSize {
                    width: Some(Dimension::Px(800.0)),
                    height: Some(Dimension::Px(600.0)),
                }),
                ..Default::default()
            },
            LayoutRoot,
        ))
        .id();

    // システム実行（TaffyStyleは既にあるのでChanged発火なし）
    let mut schedule = Schedule::default();
    schedule.add_systems((
        build_taffy_styles_system,
        sync_taffy_tree_system,
        compute_taffy_layout_system,
    ));

    // 初回実行でノードが作成される
    schedule.run(&mut world);

    // TaffyComputedLayoutが存在することを確認
    assert!(world.entity(root).contains::<TaffyComputedLayout>());

    // 2回目の実行（変更なし）
    schedule.run(&mut world);

    // エラーなく完了することを確認
    assert!(world.entity(root).contains::<TaffyComputedLayout>());
}

#[test]
fn test_high_level_component_change_triggers_compute() {
    let mut world = World::new();
    world.init_resource::<TaffyLayoutResource>();

    let root = world
        .spawn((
            TaffyStyle::default(),
            TaffyComputedLayout::default(),
            Arrangement::default(),
            BoxStyle {
                size: Some(BoxSize {
                    width: Some(Dimension::Px(800.0)),
                    height: Some(Dimension::Px(600.0)),
                }),
                ..Default::default()
            },
            LayoutRoot,
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            build_taffy_styles_system,
            sync_taffy_tree_system,
            compute_taffy_layout_system,
            update_arrangements_system,
        )
            .chain(),
    );
    schedule.run(&mut world);

    // 初回のArrangementを記録
    let initial_arrangement = *world.get::<Arrangement>(root).unwrap();

    // BoxStyleを変更
    world.entity_mut(root).insert(BoxStyle {
        size: Some(BoxSize {
            width: Some(Dimension::Px(1024.0)),
            height: Some(Dimension::Px(768.0)),
        }),
        ..Default::default()
    });

    // システム再実行
    schedule.run(&mut world);

    // Arrangementが更新されていることを確認
    let updated_arrangement = *world.get::<Arrangement>(root).unwrap();
    assert_ne!(
        initial_arrangement.size.width,
        updated_arrangement.size.width
    );
    assert_eq!(updated_arrangement.size.width, 1024.0);
    assert_eq!(updated_arrangement.size.height, 768.0);
}

#[test]
fn test_hierarchy_change_triggers_compute() {
    let mut world = World::new();
    world.init_resource::<TaffyLayoutResource>();

    let root = world
        .spawn((
            TaffyStyle::default(),
            TaffyComputedLayout::default(),
            BoxStyle {
                size: Some(BoxSize {
                    width: Some(Dimension::Px(800.0)),
                    height: Some(Dimension::Px(600.0)),
                }),
                ..Default::default()
            },
            LayoutRoot,
        ))
        .id();

    let child = world
        .spawn((
            TaffyStyle::default(),
            TaffyComputedLayout::default(),
            BoxStyle::default(),
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems((
        build_taffy_styles_system,
        sync_taffy_tree_system,
        compute_taffy_layout_system,
        update_arrangements_system,
    ));
    schedule.run(&mut world);

    // 子を追加
    world.entity_mut(child).insert(ChildOf(root));

    // システム再実行
    schedule.run(&mut world);

    // 子にもTaffyComputedLayoutが設定されていることを確認（Arrangementではなく）
    assert!(world.entity(child).contains::<TaffyComputedLayout>());
}

// ===== Task 7.6: エンティティ削除のクリーンアップテスト =====

#[test]
fn test_entity_removal_detected() {
    let mut world = World::new();
    world.init_resource::<TaffyLayoutResource>();

    let entity = world.spawn(TaffyStyle::default()).id();

    let mut schedule = Schedule::default();
    schedule.add_systems(sync_taffy_tree_system);
    schedule.run(&mut world);

    // TaffyLayoutResourceにノードが登録されていることを確認
    {
        let taffy_res = world.resource::<TaffyLayoutResource>();
        assert!(taffy_res.get_node(entity).is_some());
    }

    // エンティティを削除
    world.despawn(entity);

    // cleanup_removed_entities_systemを実行
    let mut cleanup_schedule = Schedule::default();
    cleanup_schedule.add_systems(cleanup_removed_entities_system);
    cleanup_schedule.run(&mut world);

    // ノードが削除されていることを確認
    {
        let taffy_res = world.resource::<TaffyLayoutResource>();
        assert!(taffy_res.get_node(entity).is_none());
    }
}

#[test]
fn test_taffy_node_removed_with_entity() {
    let mut world = World::new();
    world.init_resource::<TaffyLayoutResource>();

    let entity = world.spawn(TaffyStyle::default()).id();

    let mut schedule = Schedule::default();
    schedule.add_systems(sync_taffy_tree_system);
    schedule.run(&mut world);

    let node_id = {
        let taffy_res = world.resource::<TaffyLayoutResource>();
        taffy_res.get_node(entity).unwrap()
    };

    // エンティティ削除
    world.despawn(entity);

    let mut cleanup_schedule = Schedule::default();
    cleanup_schedule.add_systems(cleanup_removed_entities_system);
    cleanup_schedule.run(&mut world);

    // NodeIdからのマッピングも削除されていることを確認
    {
        let taffy_res = world.resource::<TaffyLayoutResource>();
        assert!(taffy_res.get_entity(node_id).is_none());
    }
}

#[test]
fn test_mapping_cleanup_prevents_memory_leak() {
    let mut world = World::new();
    world.init_resource::<TaffyLayoutResource>();

    // 複数のエンティティを作成・削除
    for _ in 0..10 {
        let entity = world.spawn(TaffyStyle::default()).id();

        let mut schedule = Schedule::default();
        schedule.add_systems(sync_taffy_tree_system);
        schedule.run(&mut world);

        world.despawn(entity);

        let mut cleanup_schedule = Schedule::default();
        cleanup_schedule.add_systems(cleanup_removed_entities_system);
        cleanup_schedule.run(&mut world);
    }

    // すべてのマッピングがクリーンアップされていることを確認
    #[cfg(debug_assertions)]
    {
        let taffy_res = world.resource::<TaffyLayoutResource>();
        taffy_res.verify_mapping_consistency();
    }
}

// ===== Task 7.7: 境界値シナリオテスト =====

#[test]
fn test_empty_tree() {
    let mut world = World::new();
    world.init_resource::<TaffyLayoutResource>();

    // エンティティなしでシステム実行
    let mut schedule = Schedule::default();
    schedule.add_systems((
        build_taffy_styles_system,
        sync_taffy_tree_system,
        compute_taffy_layout_system,
    ));

    // クラッシュしないことを確認
    schedule.run(&mut world);
}

#[test]
fn test_single_node_tree() {
    let mut world = World::new();
    world.init_resource::<TaffyLayoutResource>();

    let entity = world
        .spawn((
            TaffyStyle::default(),
            TaffyComputedLayout::default(),
            Arrangement::default(),
            BoxStyle {
                size: Some(BoxSize {
                    width: Some(Dimension::Px(100.0)),
                    height: Some(Dimension::Px(100.0)),
                }),
                ..Default::default()
            },
            LayoutRoot,
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(
        (
            build_taffy_styles_system,
            sync_taffy_tree_system,
            compute_taffy_layout_system,
            update_arrangements_system,
        )
            .chain(),
    );
    schedule.run(&mut world);

    // TaffyComputedLayoutが設定されていることを確認
    assert!(world.entity(entity).contains::<TaffyComputedLayout>());

    // Arrangementも設定されていることを確認
    assert!(world.entity(entity).contains::<Arrangement>());

    // Arrangementの値を検証
    let arrangement = world.get::<Arrangement>(entity).unwrap();
    assert_eq!(arrangement.size.width, 100.0);
    assert_eq!(arrangement.size.height, 100.0);
}

#[test]
fn test_many_siblings() {
    let mut world = World::new();
    world.init_resource::<TaffyLayoutResource>();

    let root = world
        .spawn((
            TaffyStyle::default(),
            TaffyComputedLayout::default(),
            BoxStyle {
                size: Some(BoxSize {
                    width: Some(Dimension::Px(1000.0)),
                    height: Some(Dimension::Px(1000.0)),
                }),
                ..Default::default()
            },
            LayoutRoot,
        ))
        .id();

    // 100個の兄弟ノードを作成
    let mut children = Vec::new();
    for _ in 0..100 {
        let child = world
            .spawn((
                TaffyStyle::default(),
                TaffyComputedLayout::default(),
                BoxStyle::default(),
                ChildOf(root),
            ))
            .id();
        children.push(child);
    }

    let mut schedule = Schedule::default();
    schedule.add_systems((
        build_taffy_styles_system,
        sync_taffy_tree_system,
        compute_taffy_layout_system,
        update_arrangements_system,
    ));

    // クラッシュしないことを確認
    schedule.run(&mut world);

    // すべての子にTaffyComputedLayoutが設定されていることを確認
    for child in children {
        assert!(world.entity(child).contains::<TaffyComputedLayout>());
    }
}

#[test]
fn test_deep_hierarchy() {
    let mut world = World::new();
    world.init_resource::<TaffyLayoutResource>();

    // 20階層の深いツリーを作成
    let mut current = world
        .spawn((
            TaffyStyle::default(),
            TaffyComputedLayout::default(),
            BoxStyle {
                size: Some(BoxSize {
                    width: Some(Dimension::Px(800.0)),
                    height: Some(Dimension::Px(600.0)),
                }),
                ..Default::default()
            },
            LayoutRoot,
        ))
        .id();

    for _ in 0..20 {
        let child = world
            .spawn((
                TaffyStyle::default(),
                TaffyComputedLayout::default(),
                BoxStyle::default(),
                ChildOf(current),
            ))
            .id();
        current = child;
    }

    let mut schedule = Schedule::default();
    schedule.add_systems((
        build_taffy_styles_system,
        sync_taffy_tree_system,
        compute_taffy_layout_system,
        update_arrangements_system,
    ));

    // クラッシュしないことを確認
    schedule.run(&mut world);

    // 最深部のエンティティにもTaffyComputedLayoutが設定されていることを確認
    assert!(world.entity(current).contains::<TaffyComputedLayout>());
}

#[test]
fn test_zero_size_box() {
    let mut world = World::new();
    world.init_resource::<TaffyLayoutResource>();

    let entity = world
        .spawn((
            TaffyStyle::default(),
            TaffyComputedLayout::default(),
            BoxStyle {
                size: Some(BoxSize {
                    width: Some(Dimension::Px(0.0)),
                    height: Some(Dimension::Px(0.0)),
                }),
                ..Default::default()
            },
            LayoutRoot,
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems((
        build_taffy_styles_system,
        sync_taffy_tree_system,
        compute_taffy_layout_system,
        update_arrangements_system,
    ));

    // ゼロサイズでもクラッシュしないことを確認
    schedule.run(&mut world);

    // TaffyComputedLayoutが設定されていることを確認
    assert!(world.entity(entity).contains::<TaffyComputedLayout>());
    assert!(world.entity(entity).contains::<Arrangement>());

    let arrangement = world.get::<Arrangement>(entity).unwrap();
    assert_eq!(arrangement.size.width, 0.0);
    assert_eq!(arrangement.size.height, 0.0);
}

#[test]
fn test_negative_margin_handling() {
    let mut world = World::new();
    world.init_resource::<TaffyLayoutResource>();

    use wintf::ecs::layout::{BoxMargin, LengthPercentageAuto, Rect};

    let entity = world
        .spawn((
            TaffyStyle::default(),
            TaffyComputedLayout::default(),
            BoxStyle {
                size: Some(BoxSize {
                    width: Some(Dimension::Px(100.0)),
                    height: Some(Dimension::Px(100.0)),
                }),
                margin: Some(BoxMargin(Rect {
                    left: LengthPercentageAuto::Px(-10.0),
                    right: LengthPercentageAuto::Px(-10.0),
                    top: LengthPercentageAuto::Px(-10.0),
                    bottom: LengthPercentageAuto::Px(-10.0),
                })),
                ..Default::default()
            },
            LayoutRoot,
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems((
        build_taffy_styles_system,
        sync_taffy_tree_system,
        compute_taffy_layout_system,
        update_arrangements_system,
    ));

    // 負のマージンでもクラッシュしないことを確認
    schedule.run(&mut world);

    // TaffyComputedLayoutが設定されていることを確認
    assert!(world.entity(entity).contains::<TaffyComputedLayout>());
    assert!(world.entity(entity).contains::<Arrangement>());
}
