use bevy_ecs::prelude::*;
use taffy::prelude::*;
use wintf::ecs::layout::taffy::{TaffyComputedLayout, TaffyStyle};
use wintf::ecs::layout::{
    BoxMargin, BoxPadding, BoxSize, Dimension, FlexContainer, FlexItem, LengthPercentage,
    LengthPercentageAuto, Rect,
};

/// テスト1.1: BoxStyleがTaffyStyleに名称変更されていることを検証
#[test]
fn test_taffy_style_renamed_from_box_style() {
    // TaffyStyleがComponentとして登録可能であることを確認
    let mut world = World::new();
    let entity = world.spawn(TaffyStyle::default()).id();

    // TaffyStyleコンポーネントが存在することを確認
    assert!(world.entity(entity).contains::<TaffyStyle>());
}

/// テスト1.2: TaffyStyleが#[repr(transparent)]であることを検証
#[test]
fn test_taffy_style_transparent_wrapper() {
    use std::mem::size_of;

    // TaffyStyleとStyle (taffy)のメモリサイズが同じであることを確認
    assert_eq!(size_of::<TaffyStyle>(), size_of::<Style>());
}

/// テスト1.3: TaffyStyleのDefaultトレイト実装を検証
#[test]
fn test_taffy_style_default_implementation() {
    let taffy_style = TaffyStyle::default();

    // デフォルト値が作成できることを確認（内部のStyle::default()と同じ構造）
    // PartialEqトレイトでデフォルト同士の比較が可能
    assert_eq!(taffy_style, TaffyStyle::default());
}

/// テスト1.4: TaffyStyleが必要なトレイト（Clone, Debug, PartialEq）を実装していることを検証
#[test]
fn test_taffy_style_trait_implementations() {
    let style1 = TaffyStyle::default();
    let style2 = style1.clone(); // Clone

    // PartialEq
    assert_eq!(style1, style2);

    // Debug (panic時に出力されることを確認)
    let debug_str = format!("{:?}", style1);
    assert!(debug_str.contains("TaffyStyle"));
}

/// テスト1.5: BoxComputedLayoutがTaffyComputedLayoutに名称変更されていることを検証
#[test]
fn test_taffy_computed_layout_renamed_from_box_computed_layout() {
    let mut world = World::new();
    let entity = world.spawn(TaffyComputedLayout::default()).id();

    assert!(world.entity(entity).contains::<TaffyComputedLayout>());
}

/// テスト1.6: TaffyComputedLayoutが#[repr(transparent)]であることを検証
#[test]
fn test_taffy_computed_layout_transparent_wrapper() {
    use std::mem::size_of;

    assert_eq!(size_of::<TaffyComputedLayout>(), size_of::<Layout>());
}

/// テスト1.7: TaffyComputedLayoutのDefaultトレイト実装を検証
#[test]
fn test_taffy_computed_layout_default_implementation() {
    let computed = TaffyComputedLayout::default();

    // デフォルト値が作成できることを確認
    assert_eq!(computed, TaffyComputedLayout::default());
}

/// テスト1.8: TaffyComputedLayoutが必要なトレイト（Clone, Debug, PartialEq, Copy）を実装していることを検証
#[test]
fn test_taffy_computed_layout_trait_implementations() {
    let layout1 = TaffyComputedLayout::default();
    let layout2 = layout1.clone(); // Clone
    let layout3 = layout1; // Copy

    // PartialEq
    assert_eq!(layout1, layout2);
    assert_eq!(layout1, layout3);

    // Debug
    let debug_str = format!("{:?}", layout1);
    assert!(debug_str.contains("TaffyComputedLayout"));
}

// ===== タスク2: 高レベルレイアウトコンポーネント =====

/// テスト2.1: BoxSizeコンポーネントの実装を検証
#[test]
fn test_box_size_component() {
    let mut world = World::new();

    // BoxSizeがComponentとして登録可能
    let entity = world
        .spawn(BoxSize {
            width: Some(Dimension::Px(200.0)),
            height: Some(Dimension::Px(100.0)),
        })
        .id();

    assert!(world.entity(entity).contains::<BoxSize>());

    // Defaultは両方None
    let default_size = BoxSize::default();
    assert_eq!(default_size.width, None);
    assert_eq!(default_size.height, None);
}

/// テスト2.2: BoxMarginコンポーネントの実装を検証
#[test]
fn test_box_margin_component() {
    let mut world = World::new();

    let margin = BoxMargin(Rect {
        left: LengthPercentageAuto::Px(10.0),
        right: LengthPercentageAuto::Px(10.0),
        top: LengthPercentageAuto::Px(5.0),
        bottom: LengthPercentageAuto::Px(5.0),
    });

    let entity = world.spawn(margin.clone()).id();
    assert!(world.entity(entity).contains::<BoxMargin>());

    // Defaultはzero
    let default_margin = BoxMargin::default();
    assert_eq!(default_margin.0.left, LengthPercentageAuto::Px(0.0));
}

/// テスト2.3: BoxPaddingコンポーネントの実装を検証
#[test]
fn test_box_padding_component() {
    let mut world = World::new();

    let padding = BoxPadding(Rect {
        left: LengthPercentage::Px(10.0),
        right: LengthPercentage::Px(10.0),
        top: LengthPercentage::Px(5.0),
        bottom: LengthPercentage::Px(5.0),
    });

    let entity = world.spawn(padding.clone()).id();
    assert!(world.entity(entity).contains::<BoxPadding>());
}

/// テスト2.4: FlexContainerコンポーネントの実装を検証
#[test]
fn test_flex_container_component() {
    let mut world = World::new();

    let container = FlexContainer {
        direction: FlexDirection::Column,
        justify_content: Some(JustifyContent::Center),
        align_items: Some(AlignItems::Center),
    };

    let entity = world.spawn(container.clone()).id();
    assert!(world.entity(entity).contains::<FlexContainer>());

    // Defaultチェック
    let default_container = FlexContainer::default();
    assert_eq!(default_container.direction, FlexDirection::Row);
    assert_eq!(default_container.justify_content, None);
    assert_eq!(default_container.align_items, None);
}

/// テスト2.5: FlexItemコンポーネントの実装を検証
#[test]
fn test_flex_item_component() {
    let mut world = World::new();

    let item = FlexItem {
        grow: 1.0,
        shrink: 0.5,
        basis: Dimension::Px(100.0),
        align_self: Some(AlignSelf::End),
    };

    let entity = world.spawn(item.clone()).id();
    assert!(world.entity(entity).contains::<FlexItem>());

    // Defaultチェック
    let default_item = FlexItem::default();
    assert_eq!(default_item.grow, 0.0);
    assert_eq!(default_item.shrink, 1.0);
    assert_eq!(default_item.basis, Dimension::Auto);
    assert_eq!(default_item.align_self, None);
}
