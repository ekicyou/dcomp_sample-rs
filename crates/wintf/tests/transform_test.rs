use bevy_ecs::prelude::*;
use windows_numerics::Matrix3x2;
use wintf::ecs::transform::*;

#[test]
fn test_update_local_transform() {
    let mut world = World::new();
    let system_id = world.register_system(update_local_transform);

    let entity = world
        .spawn((
            Translate::new(10.0, 20.0),
            Scale::new(2.0, 2.0),
            Rotate(45.0),
            LocalTransform(Matrix3x2::identity()),
        ))
        .id();

    world.run_system(system_id).unwrap();

    let local_transform = world.get::<LocalTransform>(entity).unwrap();
    
    // 期待される変換行列を手動で計算
    // 適用順序: Scale → Rotate → Translate
    let expected = compute_transform_matrix(
        Some(Translate::new(10.0, 20.0)),
        Some(Scale::new(2.0, 2.0)),
        Some(Rotate(45.0)),
        None,
        None,
    );
    
    assert_eq!(local_transform.0, expected);
}

#[test]
fn test_update_local_transform_with_origin() {
    let mut world = World::new();
    let system_id = world.register_system(update_local_transform);

    let entity = world
        .spawn((
            Translate::new(100.0, 100.0),
            Scale::new(2.0, 2.0),
            TransformOrigin::new(50.0, 50.0),
            LocalTransform(Matrix3x2::identity()),
        ))
        .id();

    world.run_system(system_id).unwrap();

    let local_transform = world.get::<LocalTransform>(entity).unwrap();
    
    // TransformOriginを考慮した期待値
    let expected = compute_transform_matrix(
        Some(Translate::new(100.0, 100.0)),
        Some(Scale::new(2.0, 2.0)),
        None,
        None,
        Some(TransformOrigin::new(50.0, 50.0)),
    );
    
    assert_eq!(local_transform.0, expected);
}

#[test]
fn test_update_global_transform_no_parent() {
    let mut world = World::new();

    let local_matrix = Matrix3x2::translation(10.0, 20.0);
    let entity = world
        .spawn((
            LocalTransform(local_matrix),
            GlobalTransform(Matrix3x2::identity()),
        ))
        .id();

    update_global_transform(&mut world);

    let global = world.get::<GlobalTransform>(entity).unwrap();
    assert_eq!(global.0, local_matrix);
}

#[test]
fn test_update_global_transform_with_parent() {
    let mut world = World::new();

    let parent_matrix = Matrix3x2::translation(100.0, 100.0);
    let child_matrix = Matrix3x2::translation(10.0, 20.0);

    let parent = world
        .spawn((
            LocalTransform(parent_matrix),
            GlobalTransform(parent_matrix),
        ))
        .id();

    let child = world.spawn_empty().id();
    world.entity_mut(child).insert((
        ChildOf(parent),
        LocalTransform(child_matrix),
        GlobalTransform(Matrix3x2::identity()),
    ));

    update_global_transform(&mut world);

    let child_global = world.get::<GlobalTransform>(child).unwrap();
    let expected = parent_matrix * child_matrix;
    assert_eq!(child_global.0, expected);
}

#[test]
fn test_update_global_transform_hierarchy() {
    let mut world = World::new();

    let root_matrix = Matrix3x2::translation(100.0, 100.0);
    let child_matrix = Matrix3x2::translation(10.0, 20.0);
    let grandchild_matrix = Matrix3x2::translation(5.0, 5.0);

    let root = world
        .spawn((LocalTransform(root_matrix), GlobalTransform(root_matrix)))
        .id();

    let child = world.spawn_empty().id();
    world.entity_mut(child).insert((
        ChildOf(root),
        LocalTransform(child_matrix),
        GlobalTransform(Matrix3x2::identity()),
    ));

    let grandchild = world.spawn_empty().id();
    world.entity_mut(grandchild).insert((
        ChildOf(child),
        LocalTransform(grandchild_matrix),
        GlobalTransform(Matrix3x2::identity()),
    ));

    update_global_transform(&mut world);

    let child_global = world.get::<GlobalTransform>(child).unwrap();
    assert_eq!(child_global.0, root_matrix * child_matrix);

    let grandchild_global = world.get::<GlobalTransform>(grandchild).unwrap();
    assert_eq!(
        grandchild_global.0,
        root_matrix * child_matrix * grandchild_matrix
    );
}

#[test]
fn test_full_transform_pipeline() {
    let mut world = World::new();
    let system_id = world.register_system(update_local_transform);

    let parent = world
        .spawn((
            Translate::new(100.0, 100.0),
            Scale::new(2.0, 2.0),
            LocalTransform(Matrix3x2::identity()),
            GlobalTransform(Matrix3x2::identity()),
        ))
        .id();

    let child = world.spawn_empty().id();
    world.entity_mut(child).insert((
        ChildOf(parent),
        Translate::new(10.0, 20.0),
        LocalTransform(Matrix3x2::identity()),
        GlobalTransform(Matrix3x2::identity()),
    ));

    world.run_system(system_id).unwrap();
    update_global_transform(&mut world);

    let parent_global = world.get::<GlobalTransform>(parent).unwrap();
    let parent_local = world.get::<LocalTransform>(parent).unwrap();
    assert_eq!(parent_global.0, parent_local.0);

    let child_global = world.get::<GlobalTransform>(child).unwrap();
    let child_local = world.get::<LocalTransform>(child).unwrap();
    assert_eq!(child_global.0, parent_local.0 * child_local.0);
}
