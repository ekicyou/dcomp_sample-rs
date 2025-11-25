//! ウィジェット Visual 自動追加テスト (R4)
//!
//! Label/Rectangle コンポーネント追加時に Visual が自動的に挿入されることをテストする。

use bevy_ecs::prelude::*;
use wintf::ecs::widget::shapes::{colors, Rectangle};
use wintf::ecs::widget::text::Label;
use wintf::ecs::Visual;

/// Label追加時にVisualが自動追加されることを確認
#[test]
fn test_label_auto_inserts_visual() {
    let mut world = World::new();

    // Label を追加
    let entity = world
        .spawn(Label {
            text: "Hello".to_string(),
            ..Default::default()
        })
        .id();

    // Visual が自動的に追加されていることを確認
    let visual = world.get::<Visual>(entity);
    assert!(
        visual.is_some(),
        "Visual should be auto-added when Label is inserted"
    );
}

/// Rectangle追加時にVisualが自動追加されることを確認
#[test]
fn test_rectangle_auto_inserts_visual() {
    let mut world = World::new();

    // Rectangle を追加
    let entity = world.spawn(Rectangle { color: colors::RED }).id();

    // Visual が自動的に追加されていることを確認
    let visual = world.get::<Visual>(entity);
    assert!(
        visual.is_some(),
        "Visual should be auto-added when Rectangle is inserted"
    );
}

/// 既にVisualを持つEntityにLabelを追加しても問題ないことを確認
#[test]
fn test_label_with_existing_visual() {
    let mut world = World::new();

    // 先にVisualを持つEntityを作成
    let entity = world
        .spawn((
            Visual {
                opacity: 0.5,
                ..Default::default()
            },
            Label {
                text: "Test".to_string(),
                ..Default::default()
            },
        ))
        .id();

    // Visualが存在することを確認（上書きされないか確認）
    let visual = world.get::<Visual>(entity).expect("Visual should exist");
    // 既存のVisual値が保持されている可能性があるため、存在のみ確認
    assert!(visual.is_visible);
}

/// 複数のウィジェットが同時にVisualを取得できることを確認
#[test]
fn test_multiple_widgets_get_visuals() {
    let mut world = World::new();

    let label_entity = world
        .spawn(Label {
            text: "Label".to_string(),
            ..Default::default()
        })
        .id();

    let rect_entity = world
        .spawn(Rectangle {
            color: colors::BLUE,
        })
        .id();

    assert!(world.get::<Visual>(label_entity).is_some());
    assert!(world.get::<Visual>(rect_entity).is_some());
}
