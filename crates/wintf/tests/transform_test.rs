use bevy_ecs::prelude::*;
use windows_numerics::Matrix3x2;
use wintf::ecs::transform::*;

#[test]
fn test_update_local_transform_default() {
    let mut world = World::new();
    let system_id = world.register_system(update_local_transform);

    let transform = Transform::default();
    let entity = world.spawn(transform).id();

    world.run_system(system_id).unwrap();

    let local_transform = world.get::<LocalTransform>(entity).unwrap();

    // デフォルト値での期待値: identity行列に近い
    let expected: Matrix3x2 = Transform::default().into();
    assert_eq!(local_transform.0, expected);
}

#[test]
fn test_update_local_transform_translate() {
    let mut world = World::new();
    let system_id = world.register_system(update_local_transform);

    let transform = Transform {
        translate: Translate::new(10.0, 20.0),
        ..Default::default()
    };
    let entity = world.spawn(transform).id();

    world.run_system(system_id).unwrap();

    let local_transform = world.get::<LocalTransform>(entity).unwrap();

    let expected: Matrix3x2 = transform.into();
    assert_eq!(local_transform.0, expected);
}

#[test]
fn test_update_local_transform_full() {
    let mut world = World::new();
    let system_id = world.register_system(update_local_transform);

    let transform = Transform {
        translate: Translate::new(10.0, 20.0),
        scale: Scale::new(2.0, 2.0),
        rotate: Rotate(45.0),
        skew: Skew::new(0.0, 0.0),
        origin: TransformOrigin::new(0.5, 0.5),
    };
    let entity = world.spawn(transform).id();

    world.run_system(system_id).unwrap();

    let local_transform = world.get::<LocalTransform>(entity).unwrap();

    // 期待される変換行列を計算
    let expected: Matrix3x2 = transform.into();
    assert_eq!(local_transform.0, expected);
}

#[test]
fn test_update_local_transform_with_origin() {
    let mut world = World::new();
    let system_id = world.register_system(update_local_transform);

    let transform = Transform {
        translate: Translate::new(100.0, 100.0),
        scale: Scale::new(2.0, 2.0),
        origin: TransformOrigin::new(50.0, 50.0),
        ..Default::default()
    };
    let entity = world.spawn(transform).id();

    world.run_system(system_id).unwrap();

    let local_transform = world.get::<LocalTransform>(entity).unwrap();

    // TransformOriginを考慮した期待値
    let expected: Matrix3x2 = transform.into();
    assert_eq!(local_transform.0, expected);
}

#[test]
fn test_transform_changed_marker() {
    let mut world = World::new();
    let system_id = world.register_system(update_local_transform);

    let transform = Transform {
        translate: Translate::new(10.0, 20.0),
        ..Default::default()
    };
    let entity = world.spawn(transform).id();

    world.run_system(system_id).unwrap();

    // LocalTransformChangedマーカーが追加されていることを確認
    assert!(world.get::<LocalTransformChanged>(entity).is_some());
}

#[test]
fn test_transform_from_matrix3x2() {
    let transform = Transform {
        translate: Translate::new(10.0, 20.0),
        scale: Scale::new(2.0, 3.0),
        rotate: Rotate(90.0),
        skew: Skew::new(5.0, 10.0),
        origin: TransformOrigin::new(25.0, 50.0),
    };

    let matrix: Matrix3x2 = transform.into();

    // 変換が実行され、適切な行列が生成されることを確認
    // （正確な値の検証ではなく、変換が成功することを確認）
    assert!(matrix.M11.is_finite());
    assert!(matrix.M12.is_finite());
    assert!(matrix.M21.is_finite());
    assert!(matrix.M22.is_finite());
    assert!(matrix.M31.is_finite());
    assert!(matrix.M32.is_finite());
}
