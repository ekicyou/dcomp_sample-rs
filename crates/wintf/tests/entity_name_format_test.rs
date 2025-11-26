//! format_entity_name関数のテスト (Requirement 4.4)

use bevy_ecs::name::Name;
use bevy_ecs::prelude::*;
use wintf::ecs::format_entity_name;

/// Nameコンポーネントが存在する場合、名前をそのまま返す
#[test]
fn format_entity_name_with_name_returns_name_string() {
    let mut world = World::new();
    let entity = world.spawn(Name::new("TestEntity")).id();
    let name = world.get::<Name>(entity);

    let result = format_entity_name(entity, name);

    assert_eq!(result, "TestEntity");
}

/// Nameコンポーネントが存在しない場合、Entity ID形式で返す
#[test]
fn format_entity_name_without_name_returns_entity_format() {
    let mut world = World::new();
    let entity = world.spawn_empty().id();

    let result = format_entity_name(entity, None);

    // Entity形式であることを確認（Entity(Xv1)のような形式）
    assert!(result.starts_with("Entity("));
    assert!(result.ends_with(")"));
}

/// 日本語名も正しく処理される
#[test]
fn format_entity_name_with_japanese_name() {
    let mut world = World::new();
    let entity = world.spawn(Name::new("赤いボックス")).id();
    let name = world.get::<Name>(entity);

    let result = format_entity_name(entity, name);

    assert_eq!(result, "赤いボックス");
}

/// ハイフンを含む名前も正しく処理される
#[test]
fn format_entity_name_with_hyphenated_name() {
    let mut world = World::new();
    let entity = world.spawn(Name::new("FlexDemo-Window")).id();
    let name = world.get::<Name>(entity);

    let result = format_entity_name(entity, name);

    assert_eq!(result, "FlexDemo-Window");
}
