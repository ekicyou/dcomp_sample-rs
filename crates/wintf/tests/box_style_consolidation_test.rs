//! BoxStyle統合コンポーネントのテスト
//!
//! Requirements: 1.1, 1.2, 1.4, 3.2

use bevy_ecs::prelude::*;

// タスク1.1: BoxStyle構造体の定義テスト

/// テスト: BoxStyleがComponentとして登録可能であること
#[test]
fn test_box_style_is_component() {
    use wintf::ecs::layout::BoxStyle;

    let mut world = World::new();
    let entity = world.spawn(BoxStyle::default()).id();

    // BoxStyleコンポーネントが存在することを確認
    assert!(world.entity(entity).contains::<BoxStyle>());
}

/// テスト: BoxStyleがデフォルトで全フィールドNoneであること
#[test]
fn test_box_style_default_all_none() {
    use wintf::ecs::layout::BoxStyle;

    let style = BoxStyle::default();

    // Box系プロパティがNone
    assert!(style.size.is_none());
    assert!(style.margin.is_none());
    assert!(style.padding.is_none());
    assert!(style.position.is_none());
    assert!(style.inset.is_none());

    // Flex系プロパティがNone
    assert!(style.flex_direction.is_none());
    assert!(style.justify_content.is_none());
    assert!(style.align_items.is_none());
    assert!(style.flex_grow.is_none());
    assert!(style.flex_shrink.is_none());
    assert!(style.flex_basis.is_none());
    assert!(style.align_self.is_none());
}

/// テスト: BoxStyleが必要なトレイト（Clone, Debug, PartialEq）を実装していること
#[test]
fn test_box_style_trait_implementations() {
    use wintf::ecs::layout::BoxStyle;

    let style1 = BoxStyle::default();
    let style2 = style1.clone(); // Clone

    // PartialEq
    assert_eq!(style1, style2);

    // Debug
    let debug_str = format!("{:?}", style1);
    assert!(debug_str.contains("BoxStyle"));
}

/// テスト: BoxStyleにBox系プロパティを設定できること
#[test]
fn test_box_style_with_box_properties() {
    use wintf::ecs::layout::{
        BoxInset, BoxMargin, BoxPadding, BoxPosition, BoxSize, BoxStyle, Dimension,
        LengthPercentage, LengthPercentageAuto, Rect,
    };

    let style = BoxStyle {
        size: Some(BoxSize {
            width: Some(Dimension::Px(200.0)),
            height: Some(Dimension::Px(100.0)),
        }),
        margin: Some(BoxMargin(Rect {
            left: LengthPercentageAuto::Px(10.0),
            right: LengthPercentageAuto::Px(10.0),
            top: LengthPercentageAuto::Px(5.0),
            bottom: LengthPercentageAuto::Px(5.0),
        })),
        padding: Some(BoxPadding(Rect {
            left: LengthPercentage::Px(5.0),
            right: LengthPercentage::Px(5.0),
            top: LengthPercentage::Px(3.0),
            bottom: LengthPercentage::Px(3.0),
        })),
        position: Some(BoxPosition::Absolute),
        inset: Some(BoxInset(Rect {
            left: LengthPercentageAuto::Px(100.0),
            top: LengthPercentageAuto::Px(50.0),
            right: LengthPercentageAuto::Auto,
            bottom: LengthPercentageAuto::Auto,
        })),
        ..Default::default()
    };

    assert!(style.size.is_some());
    assert!(style.margin.is_some());
    assert!(style.padding.is_some());
    assert_eq!(style.position, Some(BoxPosition::Absolute));
    assert!(style.inset.is_some());
}

/// テスト: BoxStyleにFlex系プロパティ（フラット構造）を設定できること
#[test]
fn test_box_style_with_flex_properties_flat() {
    use wintf::ecs::layout::{
        AlignItems, AlignSelf, BoxStyle, Dimension, FlexDirection, JustifyContent,
    };

    let style = BoxStyle {
        flex_direction: Some(FlexDirection::Row),
        justify_content: Some(JustifyContent::SpaceBetween),
        align_items: Some(AlignItems::Center),
        flex_grow: Some(1.0),
        flex_shrink: Some(0.5),
        flex_basis: Some(Dimension::Px(100.0)),
        align_self: Some(AlignSelf::FlexEnd),
        ..Default::default()
    };

    assert_eq!(style.flex_direction, Some(FlexDirection::Row));
    assert_eq!(style.justify_content, Some(JustifyContent::SpaceBetween));
    assert_eq!(style.align_items, Some(AlignItems::Center));
    assert_eq!(style.flex_grow, Some(1.0));
    assert_eq!(style.flex_shrink, Some(0.5));
    assert_eq!(style.flex_basis, Some(Dimension::Px(100.0)));
    assert_eq!(style.align_self, Some(AlignSelf::FlexEnd));
}

// タスク1.3: From変換トレイトテスト

/// テスト: BoxStyleからtaffy::Styleへの変換（全フィールドNone時）
#[test]
fn test_box_style_to_taffy_style_default() {
    use wintf::ecs::layout::BoxStyle;

    let box_style = BoxStyle::default();
    let taffy_style: taffy::Style = (&box_style).into();

    // デフォルト値がtaffyデフォルトと互換
    assert_eq!(taffy_style.flex_grow, 0.0);
    assert_eq!(taffy_style.flex_shrink, 1.0);
}

/// テスト: BoxStyleからtaffy::Styleへの変換（サイズ設定時）
#[test]
fn test_box_style_to_taffy_style_with_size() {
    use wintf::ecs::layout::{BoxSize, BoxStyle, Dimension};

    let box_style = BoxStyle {
        size: Some(BoxSize {
            width: Some(Dimension::Px(200.0)),
            height: Some(Dimension::Px(100.0)),
        }),
        ..Default::default()
    };

    let taffy_style: taffy::Style = (&box_style).into();

    assert_eq!(taffy_style.size.width, taffy::Dimension::length(200.0));
    assert_eq!(taffy_style.size.height, taffy::Dimension::length(100.0));
}

/// テスト: BoxStyleからtaffy::Styleへの変換（Flexコンテナープロパティ設定時にdisplay: Flex）
#[test]
fn test_box_style_to_taffy_style_flex_container_sets_display() {
    use wintf::ecs::layout::{BoxStyle, FlexDirection};

    let box_style = BoxStyle {
        flex_direction: Some(FlexDirection::Column),
        ..Default::default()
    };

    let taffy_style: taffy::Style = (&box_style).into();

    // flex_direction設定時にdisplay: Flexが自動設定される
    assert_eq!(taffy_style.display, taffy::Display::Flex);
    assert_eq!(taffy_style.flex_direction, taffy::FlexDirection::Column);
}

/// テスト: BoxStyleからtaffy::Styleへの変換（flex_growのNone時デフォルト）
#[test]
fn test_box_style_to_taffy_style_flex_grow_default() {
    use wintf::ecs::layout::BoxStyle;

    let box_style = BoxStyle {
        flex_grow: None,
        ..Default::default()
    };

    let taffy_style: taffy::Style = (&box_style).into();

    // flex_grow: None → 0.0（taffyデフォルト）
    assert_eq!(taffy_style.flex_grow, 0.0);
}

/// テスト: BoxStyleからtaffy::Styleへの変換（flex_shrinkのNone時デフォルト）
#[test]
fn test_box_style_to_taffy_style_flex_shrink_default() {
    use wintf::ecs::layout::BoxStyle;

    let box_style = BoxStyle {
        flex_shrink: None,
        ..Default::default()
    };

    let taffy_style: taffy::Style = (&box_style).into();

    // flex_shrink: None → 1.0（taffyデフォルト）
    assert_eq!(taffy_style.flex_shrink, 1.0);
}

/// テスト: BoxStyleからtaffy::Styleへの変換（絶対配置時）
#[test]
fn test_box_style_to_taffy_style_absolute_position() {
    use wintf::ecs::layout::{BoxInset, BoxPosition, BoxStyle, LengthPercentageAuto, Rect};

    let box_style = BoxStyle {
        position: Some(BoxPosition::Absolute),
        inset: Some(BoxInset(Rect {
            left: LengthPercentageAuto::Px(100.0),
            top: LengthPercentageAuto::Px(50.0),
            right: LengthPercentageAuto::Auto,
            bottom: LengthPercentageAuto::Auto,
        })),
        ..Default::default()
    };

    let taffy_style: taffy::Style = (&box_style).into();

    assert_eq!(taffy_style.position, taffy::Position::Absolute);
}

// タスク1.2: 非コンポーネント化テスト（BoxSize等がComponentでなくなっていること）
// 注: 実装後にコンパイルエラーで検証される

/// テスト: BoxSize、BoxMargin等は通常の構造体として機能すること
#[test]
fn test_legacy_types_as_value_objects() {
    use wintf::ecs::layout::{
        BoxInset, BoxMargin, BoxPadding, BoxPosition, BoxSize, Dimension, Rect,
    };

    // 値オブジェクトとして構築可能
    let size = BoxSize {
        width: Some(Dimension::Px(100.0)),
        height: None,
    };
    let margin = BoxMargin(Rect::zero());
    let padding = BoxPadding(Rect::zero());
    let position = BoxPosition::Relative;
    let inset = BoxInset(Rect::auto());

    // Clone可能
    let _size_clone = size.clone();
    let _margin_clone = margin.clone();
    let _padding_clone = padding.clone();
    let _position_clone = position.clone();
    let _inset_clone = inset.clone();

    // 比較可能
    assert_eq!(position, BoxPosition::Relative);
}
