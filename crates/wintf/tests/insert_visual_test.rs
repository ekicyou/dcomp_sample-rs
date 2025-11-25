//! insert_visual ヘルパー関数テスト (R3)
//!
//! ウィジェットの on_add フックから呼び出し可能な Visual 挿入ヘルパー関数をテストする。

use bevy_ecs::prelude::*;
use windows_numerics::Vector2;
use wintf::ecs::{insert_visual, insert_visual_with, Visual};

/// insert_visual: デフォルト Visual を挿入できることを確認
#[test]
fn test_insert_visual_adds_default_visual() {
    let mut world = World::new();

    let entity = world.spawn_empty().id();

    // insert_visual を呼び出し
    insert_visual(&mut world, entity);

    // Visual コンポーネントが追加されていることを確認
    let visual = world.get::<Visual>(entity);
    assert!(visual.is_some(), "Visual should be added by insert_visual");

    // デフォルト値の確認
    let visual = visual.unwrap();
    assert!(visual.is_visible, "Default is_visible should be true");
    assert_eq!(visual.opacity, 1.0, "Default opacity should be 1.0");
}

/// insert_visual_with: カスタム Visual を挿入できることを確認
#[test]
fn test_insert_visual_with_adds_custom_visual() {
    let mut world = World::new();

    let entity = world.spawn_empty().id();

    // カスタム Visual を作成
    let custom_visual = Visual {
        is_visible: false,
        opacity: 0.5,
        transform_origin: Vector2 { X: 100.0, Y: 100.0 },
        size: Vector2 { X: 200.0, Y: 150.0 },
    };

    // insert_visual_with を呼び出し
    insert_visual_with(&mut world, entity, custom_visual.clone());

    // Visual コンポーネントが追加されていることを確認
    let visual = world.get::<Visual>(entity).expect("Visual should be added");

    assert_eq!(visual.is_visible, false);
    assert_eq!(visual.opacity, 0.5);
    assert_eq!(visual.size.X, 200.0);
    assert_eq!(visual.size.Y, 150.0);
}

/// insert_visual: 既に Visual を持つ Entity に対しては上書きされることを確認
#[test]
fn test_insert_visual_overwrites_existing() {
    let mut world = World::new();

    // 既存の Visual を持つ Entity
    let entity = world
        .spawn(Visual {
            is_visible: false,
            opacity: 0.3,
            ..Default::default()
        })
        .id();

    // insert_visual でデフォルト値に上書き
    insert_visual(&mut world, entity);

    let visual = world.get::<Visual>(entity).expect("Visual should exist");
    assert!(visual.is_visible, "Should be overwritten to default true");
    assert_eq!(visual.opacity, 1.0, "Should be overwritten to default 1.0");
}

/// insert_visual: 存在しない Entity に対しては何もしない（パニックしない）
#[test]
fn test_insert_visual_nonexistent_entity() {
    let mut world = World::new();

    // 存在しない Entity ID を作成
    let fake_entity = match Entity::from_raw_u32(9999) {
        Some(e) => e,
        None => {
            // Entity が作成できない場合はテストをスキップ
            return;
        }
    };

    // パニックしないことを確認
    insert_visual(&mut world, fake_entity);
    // テストが完了すれば成功
}

/// 複数の Entity に対して insert_visual を連続で呼び出せることを確認
#[test]
fn test_insert_visual_multiple_entities() {
    let mut world = World::new();

    let entity1 = world.spawn_empty().id();
    let entity2 = world.spawn_empty().id();
    let entity3 = world.spawn_empty().id();

    insert_visual(&mut world, entity1);
    insert_visual(&mut world, entity2);
    insert_visual(&mut world, entity3);

    assert!(world.get::<Visual>(entity1).is_some());
    assert!(world.get::<Visual>(entity2).is_some());
    assert!(world.get::<Visual>(entity3).is_some());
}
