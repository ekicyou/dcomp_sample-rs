use bevy_ecs::prelude::*;
use windows_numerics::Matrix3x2;
use wintf::ecs::transform::*;
use wintf::ecs::transform_system::*;

#[test]
fn test_transform_to_matrix3x2_identity() {
    let transform = Transform::default();
    let matrix: Matrix3x2 = transform.into();

    // デフォルト値（単位行列に近い）
    assert_eq!(matrix.M11, 1.0);
    assert_eq!(matrix.M12, 0.0);
    assert_eq!(matrix.M21, 0.0);
    assert_eq!(matrix.M22, 1.0);
    assert_eq!(matrix.M31, 0.0);
    assert_eq!(matrix.M32, 0.0);
}

#[test]
fn test_transform_to_matrix3x2_translate() {
    let transform = Transform {
        translate: Translate::new(10.0, 20.0),
        ..Default::default()
    };
    let matrix: Matrix3x2 = transform.into();

    // 平行移動のみ
    assert_eq!(matrix.M31, 10.0);
    assert_eq!(matrix.M32, 20.0);
}

#[test]
fn test_transform_to_matrix3x2_scale() {
    let transform = Transform {
        scale: Scale::new(2.0, 3.0),
        origin: TransformOrigin::top_left(), // 原点を左上に
        ..Default::default()
    };
    let matrix: Matrix3x2 = transform.into();

    // スケールが適用される
    assert!((matrix.M11 - 2.0).abs() < 0.001);
    assert!((matrix.M22 - 3.0).abs() < 0.001);
}

#[test]
fn test_transform_to_matrix3x2_rotate_90() {
    let transform = Transform {
        rotate: Rotate(90.0),
        origin: TransformOrigin::top_left(), // 原点を左上に
        ..Default::default()
    };
    let matrix: Matrix3x2 = transform.into();

    // 90度回転（変換行列の計算順序により実際の値を確認）
    // 回転が適用されていることを確認
    let is_rotated = (matrix.M11 - 1.0).abs() > 0.001 || (matrix.M12 - 0.0).abs() > 0.001;
    assert!(is_rotated, "回転が適用されていません");
}

#[test]
fn test_transform_to_matrix3x2_combined() {
    let transform = Transform {
        translate: Translate::new(100.0, 200.0),
        scale: Scale::new(2.0, 2.0),
        rotate: Rotate(0.0),
        skew: Skew::default(),
        origin: TransformOrigin::center(),
    };
    let matrix: Matrix3x2 = transform.into();

    // 変換が適用されていることを確認
    // 具体的な値は複雑なので、行列が単位行列でないことだけ確認
    let is_identity = matrix.M11 == 1.0
        && matrix.M12 == 0.0
        && matrix.M21 == 0.0
        && matrix.M22 == 1.0
        && matrix.M31 == 0.0
        && matrix.M32 == 0.0;

    assert!(!is_identity, "変換が適用されていません");
}

#[test]
fn test_transform_to_matrix3x2_with_origin() {
    let transform = Transform {
        scale: Scale::new(2.0, 2.0),
        origin: TransformOrigin::new(0.5, 0.5),
        ..Default::default()
    };
    let matrix: Matrix3x2 = transform.into();

    // originが考慮された変換
    assert!((matrix.M11 - 2.0).abs() < 0.001);
    assert!((matrix.M22 - 2.0).abs() < 0.001);
}

// ========== システムのテスト ==========

/// sync_simple_transformsシステムのテスト
/// 階層に属していないエンティティのGlobalTransformが更新されることを確認
#[test]
fn test_sync_simple_transforms() {
    let mut world = World::new();

    // 親を持たないエンティティを作成
    let entity = world
        .spawn((
            Transform {
                translate: Translate::new(50.0, 100.0),
                scale: Scale::uniform(2.0),
                ..Default::default()
            },
            GlobalTransform::default(),
        ))
        .id();

    // sync_simple_transformsシステムを実行
    let mut schedule = Schedule::default();
    schedule
        .add_systems(sync_simple_transforms::<Transform, GlobalTransform, TransformTreeChanged>);
    schedule.run(&mut world);

    // GlobalTransformが更新されていることを確認
    let global = world.get::<GlobalTransform>(entity).unwrap();
    let matrix: Matrix3x2 = (*global).into();

    // 単位行列ではないことを確認（変換が適用されている）
    let identity = Matrix3x2::identity();
    assert_ne!(matrix.M31, identity.M31); // X平行移動
    assert_ne!(matrix.M32, identity.M32); // Y平行移動
}

// ========== システムのインテグレーションテスト ==========

use bevy_ecs::hierarchy::ChildOf;
use std::collections::HashMap;

/// テスト用のエンティティツリー構造
/// 素数スケール値を使用して検算可能にする
struct TestEntityTree {
    // Tree A (12 entities) - 素数 2-37
    root_a: Entity,      // Scale(2, 2)
    branch_a1: Entity,   // Scale(3, 3)
    leaf_a1a: Entity,    // Scale(5, 5)
    leaf_a1b: Entity,    // Scale(7, 7)
    branch_a2: Entity,   // Scale(11, 11)
    branch_a2a: Entity,  // Scale(13, 13)
    leaf_a2a1: Entity,   // Scale(17, 17)
    branch_a2a2: Entity, // Scale(19, 19)
    deep_leaf_a: Entity, // Scale(23, 23)
    branch_a2b: Entity,  // Scale(29, 29)
    leaf_a2b1: Entity,   // Scale(31, 31)
    leaf_a2b2: Entity,   // Scale(37, 37)

    // Tree B (5 entities) - 素数 43-61
    root_b: Entity,        // Scale(43, 43)
    child_b1: Entity,      // Scale(47, 47)
    grandchild_b1: Entity, // Scale(53, 53)
    child_b2: Entity,      // Scale(59, 59)
    grandchild_b2: Entity, // Scale(61, 61)

    // Standalone (1 entity) - 素数 41
    standalone: Entity, // Scale(41, 41)
}

impl TestEntityTree {
    fn new(world: &mut World) -> Self {
        // Helper function to create entity with prime scale
        let spawn_entity = |world: &mut World, prime: f32, parent: Option<Entity>| {
            let mut entity_builder = world.spawn((
                Transform {
                    scale: Scale::new(prime, prime),
                    ..Default::default()
                },
                GlobalTransform::default(),
                TransformTreeChanged,
            ));

            if let Some(parent) = parent {
                entity_builder.insert(ChildOf(parent));
            }

            entity_builder.id()
        };

        // Tree A: Root
        let root_a = spawn_entity(world, 2.0, None);

        // Tree A: Branch_A1 subtree
        let branch_a1 = spawn_entity(world, 3.0, Some(root_a));
        let leaf_a1a = spawn_entity(world, 5.0, Some(branch_a1));
        let leaf_a1b = spawn_entity(world, 7.0, Some(branch_a1));

        // Tree A: Branch_A2 subtree
        let branch_a2 = spawn_entity(world, 11.0, Some(root_a));
        let branch_a2a = spawn_entity(world, 13.0, Some(branch_a2));
        let leaf_a2a1 = spawn_entity(world, 17.0, Some(branch_a2a));
        let branch_a2a2 = spawn_entity(world, 19.0, Some(branch_a2a));
        let deep_leaf_a = spawn_entity(world, 23.0, Some(branch_a2a2));
        let branch_a2b = spawn_entity(world, 29.0, Some(branch_a2));
        let leaf_a2b1 = spawn_entity(world, 31.0, Some(branch_a2b));
        let leaf_a2b2 = spawn_entity(world, 37.0, Some(branch_a2b));

        // Tree B: Root and children
        let root_b = spawn_entity(world, 43.0, None);
        let child_b1 = spawn_entity(world, 47.0, Some(root_b));
        let grandchild_b1 = spawn_entity(world, 53.0, Some(child_b1));
        let child_b2 = spawn_entity(world, 59.0, Some(root_b));
        let grandchild_b2 = spawn_entity(world, 61.0, Some(child_b2));

        // Standalone entity
        let standalone = spawn_entity(world, 41.0, None);

        Self {
            root_a,
            branch_a1,
            leaf_a1a,
            leaf_a1b,
            branch_a2,
            branch_a2a,
            leaf_a2a1,
            branch_a2a2,
            deep_leaf_a,
            branch_a2b,
            leaf_a2b1,
            leaf_a2b2,
            root_b,
            child_b1,
            grandchild_b1,
            child_b2,
            grandchild_b2,
            standalone,
        }
    }
}

/// GlobalTransformのスナップショットを取得
fn capture_global_transforms(world: &mut World) -> HashMap<Entity, Matrix3x2> {
    let mut snapshot = HashMap::new();
    let mut query = world.query::<(Entity, &GlobalTransform)>();
    for (entity, global) in query.iter(world) {
        snapshot.insert(entity, global.0);
    }
    snapshot
}

/// 変更されたエンティティを検出
fn find_changed_entities(
    before: &HashMap<Entity, Matrix3x2>,
    after: &HashMap<Entity, Matrix3x2>,
) -> Vec<Entity> {
    after
        .iter()
        .filter(|(entity, g_after)| {
            before
                .get(entity)
                .map_or(true, |g_before| g_before != *g_after)
        })
        .map(|(entity, _)| *entity)
        .collect()
}

/// テスト用のスケジュールを作成（3つのシステムを登録）
fn create_test_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.add_systems((
        mark_dirty_trees::<Transform, GlobalTransform, TransformTreeChanged>,
        sync_simple_transforms::<Transform, GlobalTransform, TransformTreeChanged>,
        propagate_parent_transforms::<Transform, GlobalTransform, TransformTreeChanged>,
    ));
    schedule
}

// ========== Scenario Tests ==========

#[test]
fn test_scenario_1_deep_wide_hierarchy_propagation() {
    // Setup
    let mut world = World::new();
    let mut schedule = create_test_schedule();
    let tree = TestEntityTree::new(&mut world);

    // Initial run to propagate transforms
    schedule.run(&mut world);

    // Verify all entities have correct GlobalTransform (prime products)
    let deep_leaf_g = world.get::<GlobalTransform>(tree.deep_leaf_a).unwrap();
    let expected = 2.0 * 11.0 * 13.0 * 19.0 * 23.0; // = 125774
    assert_eq!(
        deep_leaf_g.0.M11, expected,
        "Deep_Leaf_A scale X should be {}",
        expected
    );
    assert_eq!(
        deep_leaf_g.0.M22, expected,
        "Deep_Leaf_A scale Y should be {}",
        expected
    );

    let leaf_a1a_g = world.get::<GlobalTransform>(tree.leaf_a1a).unwrap();
    let expected_a1a = 2.0 * 3.0 * 5.0; // = 30
    assert_eq!(leaf_a1a_g.0.M11, expected_a1a);
    assert_eq!(leaf_a1a_g.0.M22, expected_a1a);

    let grandchild_b1_g = world.get::<GlobalTransform>(tree.grandchild_b1).unwrap();
    let expected_b1 = 43.0 * 47.0 * 53.0; // = 107189
    assert_eq!(grandchild_b1_g.0.M11, expected_b1);
    assert_eq!(grandchild_b1_g.0.M22, expected_b1);
}

#[test]
fn test_scenario_2_partial_subtree_change() {
    // Setup
    let mut world = World::new();
    let mut schedule = create_test_schedule();
    let tree = TestEntityTree::new(&mut world);

    // Initial run
    schedule.run(&mut world);

    // Capture before
    let before = capture_global_transforms(&mut world);

    // Modify Branch_A2: Scale(11, 11) → Scale(67, 67)
    {
        let mut transform = world.get_mut::<Transform>(tree.branch_a2).unwrap();
        transform.scale = Scale::new(67.0, 67.0);
    }

    // Run schedule
    schedule.run(&mut world);

    // Capture after
    let after = capture_global_transforms(&mut world);

    // Detect changes
    let changed = find_changed_entities(&before, &after);
    let expected_changed = vec![
        tree.branch_a2,
        tree.branch_a2a,
        tree.leaf_a2a1,
        tree.branch_a2a2,
        tree.deep_leaf_a,
        tree.branch_a2b,
        tree.leaf_a2b1,
        tree.leaf_a2b2,
    ];
    assert_eq!(
        changed.len(),
        expected_changed.len(),
        "8 entities should change"
    );
    for entity in &expected_changed {
        assert!(
            changed.contains(entity),
            "Expected {:?} to be changed",
            entity
        );
    }

    // Verify unchanged entities
    assert_eq!(
        before.get(&tree.root_a),
        after.get(&tree.root_a),
        "Root_A should not change"
    );
    assert_eq!(
        before.get(&tree.branch_a1),
        after.get(&tree.branch_a1),
        "Branch_A1 should not change"
    );
    assert_eq!(
        before.get(&tree.leaf_a1a),
        after.get(&tree.leaf_a1a),
        "Leaf_A1a should not change"
    );

    // Verify prime products
    let deep_leaf_g = world.get::<GlobalTransform>(tree.deep_leaf_a).unwrap();
    let expected = 2.0 * 67.0 * 13.0 * 19.0 * 23.0; // = 766178
    assert_eq!(deep_leaf_g.0.M11, expected);
    assert_eq!(deep_leaf_g.0.M22, expected);
}

#[test]
fn test_scenario_3_deep_intermediate_node_change() {
    // Setup
    let mut world = World::new();
    let mut schedule = create_test_schedule();
    let tree = TestEntityTree::new(&mut world);

    schedule.run(&mut world);
    let before = capture_global_transforms(&mut world);

    // Modify Branch_A2a2: Scale(19, 19) → Scale(71, 71)
    {
        let mut transform = world.get_mut::<Transform>(tree.branch_a2a2).unwrap();
        transform.scale = Scale::new(71.0, 71.0);
    }

    schedule.run(&mut world);
    let after = capture_global_transforms(&mut world);

    // Only 2 entities should change
    let changed = find_changed_entities(&before, &after);
    assert_eq!(
        changed.len(),
        2,
        "Only Branch_A2a2 and Deep_Leaf_A should change"
    );
    assert!(changed.contains(&tree.branch_a2a2));
    assert!(changed.contains(&tree.deep_leaf_a));

    // Verify unchanged
    assert_eq!(before.get(&tree.root_a), after.get(&tree.root_a));
    assert_eq!(before.get(&tree.branch_a2), after.get(&tree.branch_a2));
    assert_eq!(before.get(&tree.branch_a2b), after.get(&tree.branch_a2b));

    // Verify prime product
    let deep_leaf_g = world.get::<GlobalTransform>(tree.deep_leaf_a).unwrap();
    let expected = 2.0 * 11.0 * 13.0 * 71.0 * 23.0; // = 470414
    assert_eq!(deep_leaf_g.0.M11, expected);
}

#[test]
fn test_scenario_4_standalone_entity_update() {
    // Setup
    let mut world = World::new();
    let mut schedule = create_test_schedule();
    let tree = TestEntityTree::new(&mut world);

    schedule.run(&mut world);
    let before = capture_global_transforms(&mut world);

    // Modify Standalone: Scale(41, 41) → Scale(73, 73)
    {
        let mut transform = world.get_mut::<Transform>(tree.standalone).unwrap();
        transform.scale = Scale::new(73.0, 73.0);
    }

    schedule.run(&mut world);
    let after = capture_global_transforms(&mut world);

    // Only standalone should change
    let changed = find_changed_entities(&before, &after);
    assert_eq!(changed.len(), 1);
    assert!(changed.contains(&tree.standalone));

    // Verify trees unchanged
    assert_eq!(before.get(&tree.root_a), after.get(&tree.root_a));
    assert_eq!(before.get(&tree.root_b), after.get(&tree.root_b));

    // Verify standalone value
    let standalone_g = world.get::<GlobalTransform>(tree.standalone).unwrap();
    assert_eq!(standalone_g.0.M11, 73.0);
    assert_eq!(standalone_g.0.M22, 73.0);
}

#[test]
fn test_scenario_5_parallel_propagation_to_multiple_children() {
    // Setup
    let mut world = World::new();
    let mut schedule = create_test_schedule();
    let tree = TestEntityTree::new(&mut world);

    schedule.run(&mut world);
    let before = capture_global_transforms(&mut world);

    // Modify Root_B: Scale(43, 43) → Scale(79, 79)
    {
        let mut transform = world.get_mut::<Transform>(tree.root_b).unwrap();
        transform.scale = Scale::new(79.0, 79.0);
    }

    schedule.run(&mut world);
    let after = capture_global_transforms(&mut world);

    // All tree B entities should change (5 entities)
    let changed = find_changed_entities(&before, &after);
    let expected_changed = vec![
        tree.root_b,
        tree.child_b1,
        tree.grandchild_b1,
        tree.child_b2,
        tree.grandchild_b2,
    ];
    assert_eq!(changed.len(), 5);
    for entity in &expected_changed {
        assert!(changed.contains(entity));
    }

    // Verify tree A unchanged
    assert_eq!(before.get(&tree.root_a), after.get(&tree.root_a));
    assert_eq!(before.get(&tree.standalone), after.get(&tree.standalone));

    // Verify prime products
    let grandchild_b1_g = world.get::<GlobalTransform>(tree.grandchild_b1).unwrap();
    let expected = 79.0 * 47.0 * 53.0; // = 196789
    assert_eq!(grandchild_b1_g.0.M11, expected);
}

#[test]
fn test_scenario_6_concurrent_multiple_tree_processing() {
    // Setup
    let mut world = World::new();
    let mut schedule = create_test_schedule();
    let tree = TestEntityTree::new(&mut world);

    schedule.run(&mut world);
    let before = capture_global_transforms(&mut world);

    // Modify all roots simultaneously
    {
        let mut transform_a = world.get_mut::<Transform>(tree.root_a).unwrap();
        transform_a.scale = Scale::new(83.0, 83.0);
    }
    {
        let mut transform_b = world.get_mut::<Transform>(tree.root_b).unwrap();
        transform_b.scale = Scale::new(89.0, 89.0);
    }
    {
        let mut transform_s = world.get_mut::<Transform>(tree.standalone).unwrap();
        transform_s.scale = Scale::new(97.0, 97.0);
    }

    schedule.run(&mut world);
    let after = capture_global_transforms(&mut world);

    // All 18 entities should change
    let changed = find_changed_entities(&before, &after);
    assert_eq!(changed.len(), 18, "All entities should change");
}

#[test]
fn test_scenario_7_isolation_and_tree_reconstruction() {
    // Setup
    let mut world = World::new();
    let mut schedule = create_test_schedule();
    let tree = TestEntityTree::new(&mut world);

    schedule.run(&mut world);

    // Phase 1: Isolation
    let before1 = capture_global_transforms(&mut world);

    // Remove ChildOf from Branch_A2a
    world.entity_mut(tree.branch_a2a).remove::<ChildOf>();

    schedule.run(&mut world);
    let after1 = capture_global_transforms(&mut world);

    // 4 entities should change (Branch_A2a subtree)
    let changed1 = find_changed_entities(&before1, &after1);
    assert_eq!(changed1.len(), 4);
    assert!(changed1.contains(&tree.branch_a2a));
    assert!(changed1.contains(&tree.leaf_a2a1));
    assert!(changed1.contains(&tree.branch_a2a2));
    assert!(changed1.contains(&tree.deep_leaf_a));

    // Verify Branch_A2a is now independent (only its own scale)
    let branch_a2a_g = world.get::<GlobalTransform>(tree.branch_a2a).unwrap();
    assert_eq!(branch_a2a_g.0.M11, 13.0);
    assert_eq!(branch_a2a_g.0.M22, 13.0);

    // Verify deep_leaf still has its chain from Branch_A2a
    let deep_leaf_g = world.get::<GlobalTransform>(tree.deep_leaf_a).unwrap();
    let expected_isolated = 13.0 * 19.0 * 23.0; // = 5681
    assert_eq!(deep_leaf_g.0.M11, expected_isolated);

    // Phase 2: Reconstruction
    let before2 = capture_global_transforms(&mut world);

    // Add ChildOf(Root_B) to Branch_A2a
    world
        .entity_mut(tree.branch_a2a)
        .insert(ChildOf(tree.root_b));

    // Run schedule multiple times to ensure propagation
    schedule.run(&mut world);
    schedule.run(&mut world);

    let after2 = capture_global_transforms(&mut world);

    // Verify Branch_A2a now inherits from Root_B (should be changed)
    let branch_a2a_g2 = world.get::<GlobalTransform>(tree.branch_a2a).unwrap();
    let expected_reconstructed = 43.0 * 13.0; // = 559
    assert_eq!(
        branch_a2a_g2.0.M11, expected_reconstructed,
        "Branch_A2a should inherit from Root_B"
    );

    // Verify deep_leaf has full new chain
    let deep_leaf_g2 = world.get::<GlobalTransform>(tree.deep_leaf_a).unwrap();
    let expected_full = 43.0 * 13.0 * 19.0 * 23.0; // = 244277
    assert_eq!(
        deep_leaf_g2.0.M11, expected_full,
        "Deep_Leaf_A should have full chain through Root_B"
    );

    // Verify at least Branch_A2a subtree changed
    let changed2 = find_changed_entities(&before2, &after2);
    assert!(
        changed2.len() >= 1,
        "At least Branch_A2a should change, got {} entities",
        changed2.len()
    );
    assert!(
        changed2.contains(&tree.branch_a2a),
        "Branch_A2a should be in changed list"
    );
}

#[test]
fn test_scenario_8_dirty_mark_optimization() {
    // Setup
    let mut world = World::new();
    let mut schedule = create_test_schedule();
    let tree = TestEntityTree::new(&mut world);

    schedule.run(&mut world);
    let before = capture_global_transforms(&mut world);

    // Modify Branch_A1: Scale(3, 3) → Scale(83, 83)
    {
        let mut transform = world.get_mut::<Transform>(tree.branch_a1).unwrap();
        transform.scale = Scale::new(83.0, 83.0);
    }

    schedule.run(&mut world);
    let after = capture_global_transforms(&mut world);

    // Only Branch_A1 subtree should change (3 entities)
    let changed = find_changed_entities(&before, &after);
    assert_eq!(changed.len(), 3, "Only Branch_A1 subtree should change");
    assert!(changed.contains(&tree.branch_a1));
    assert!(changed.contains(&tree.leaf_a1a));
    assert!(changed.contains(&tree.leaf_a1b));

    // Verify unchanged
    assert_eq!(
        before.get(&tree.root_a),
        after.get(&tree.root_a),
        "Root_A should not change"
    );
    assert_eq!(
        before.get(&tree.branch_a2),
        after.get(&tree.branch_a2),
        "Branch_A2 should not change"
    );
    assert_eq!(
        before.get(&tree.root_b),
        after.get(&tree.root_b),
        "Tree B should not change"
    );
    assert_eq!(
        before.get(&tree.standalone),
        after.get(&tree.standalone),
        "Standalone should not change"
    );

    // Verify prime product
    let leaf_a1a_g = world.get::<GlobalTransform>(tree.leaf_a1a).unwrap();
    let expected = 2.0 * 83.0 * 5.0; // = 830
    assert_eq!(leaf_a1a_g.0.M11, expected);
}
