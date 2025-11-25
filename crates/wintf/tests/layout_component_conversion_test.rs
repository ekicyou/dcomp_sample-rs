/// Task 7.1: 高レベルコンポーネント→TaffyStyle変換テスト
/// BoxStyle統合後の新API使用版
use bevy_ecs::prelude::*;
use wintf::ecs::layout::*;

#[test]
fn test_box_size_to_taffy_style() {
    let mut world = World::new();

    // BoxStyleコンポーネントを持つエンティティを作成
    let entity = world
        .spawn(BoxStyle {
            size: Some(BoxSize {
                width: Some(Dimension::Px(100.0)),
                height: Some(Dimension::Px(200.0)),
            }),
            ..Default::default()
        })
        .id();

    // TaffyStyleを自動挿入するシステムを実行
    let mut schedule = Schedule::default();
    schedule.add_systems(build_taffy_styles_system);
    schedule.run(&mut world);

    // TaffyStyleが挿入され、正しく変換されていることを確認
    let taffy_style = world
        .get::<TaffyStyle>(entity)
        .expect("TaffyStyle should be inserted");
    let style = taffy_style.style();

    // taffy::Dimension::length() で期待値を作成
    assert_eq!(style.size.width, ::taffy::Dimension::length(100.0));
    assert_eq!(style.size.height, ::taffy::Dimension::length(200.0));
}

#[test]
fn test_box_size_none_defaults_to_auto() {
    let mut world = World::new();

    let entity = world
        .spawn(BoxStyle {
            size: Some(BoxSize {
                width: None,
                height: None,
            }),
            ..Default::default()
        })
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(build_taffy_styles_system);
    schedule.run(&mut world);

    let taffy_style = world.get::<TaffyStyle>(entity).unwrap();
    let style = taffy_style.style();
    assert_eq!(style.size.width, ::taffy::Dimension::auto());
    assert_eq!(style.size.height, ::taffy::Dimension::auto());
}

#[test]
fn test_box_margin_to_taffy_style() {
    let mut world = World::new();

    let entity = world
        .spawn(BoxStyle {
            margin: Some(BoxMargin(Rect {
                left: LengthPercentageAuto::Px(10.0),
                right: LengthPercentageAuto::Px(20.0),
                top: LengthPercentageAuto::Px(30.0),
                bottom: LengthPercentageAuto::Px(40.0),
            })),
            ..Default::default()
        })
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(build_taffy_styles_system);
    schedule.run(&mut world);

    let taffy_style = world.get::<TaffyStyle>(entity).unwrap();
    let style = taffy_style.style();
    assert_eq!(
        style.margin.left,
        ::taffy::LengthPercentageAuto::length(10.0)
    );
    assert_eq!(
        style.margin.right,
        ::taffy::LengthPercentageAuto::length(20.0)
    );
    assert_eq!(
        style.margin.top,
        ::taffy::LengthPercentageAuto::length(30.0)
    );
    assert_eq!(
        style.margin.bottom,
        ::taffy::LengthPercentageAuto::length(40.0)
    );
}

#[test]
fn test_box_padding_to_taffy_style() {
    let mut world = World::new();

    let entity = world
        .spawn(BoxStyle {
            padding: Some(BoxPadding(Rect {
                left: LengthPercentage::Px(5.0),
                right: LengthPercentage::Px(10.0),
                top: LengthPercentage::Px(15.0),
                bottom: LengthPercentage::Px(20.0),
            })),
            ..Default::default()
        })
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(build_taffy_styles_system);
    schedule.run(&mut world);

    let taffy_style = world.get::<TaffyStyle>(entity).unwrap();
    let style = taffy_style.style();
    assert_eq!(style.padding.left, ::taffy::LengthPercentage::length(5.0));
    assert_eq!(style.padding.right, ::taffy::LengthPercentage::length(10.0));
    assert_eq!(style.padding.top, ::taffy::LengthPercentage::length(15.0));
    assert_eq!(
        style.padding.bottom,
        ::taffy::LengthPercentage::length(20.0)
    );
}

#[test]
fn test_flex_container_to_taffy_style() {
    let mut world = World::new();

    let entity = world
        .spawn(BoxStyle {
            flex_direction: Some(FlexDirection::Column),
            justify_content: Some(JustifyContent::Center),
            align_items: Some(AlignItems::FlexEnd),
            ..Default::default()
        })
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(build_taffy_styles_system);
    schedule.run(&mut world);

    let taffy_style = world.get::<TaffyStyle>(entity).unwrap();
    let style = taffy_style.style();
    assert_eq!(style.flex_direction, FlexDirection::Column);
    assert_eq!(style.justify_content, Some(JustifyContent::Center));
    assert_eq!(style.align_items, Some(AlignItems::FlexEnd));
}

#[test]
fn test_flex_item_to_taffy_style() {
    let mut world = World::new();

    let entity = world
        .spawn(BoxStyle {
            flex_grow: Some(2.0),
            flex_shrink: Some(0.5),
            flex_basis: Some(Dimension::Px(100.0)),
            align_self: Some(AlignSelf::Center),
            ..Default::default()
        })
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(build_taffy_styles_system);
    schedule.run(&mut world);

    let taffy_style = world.get::<TaffyStyle>(entity).unwrap();
    let style = taffy_style.style();
    assert_eq!(style.flex_grow, 2.0);
    assert_eq!(style.flex_shrink, 0.5);
    assert_eq!(style.flex_basis, ::taffy::Dimension::length(100.0));
    assert_eq!(style.align_self, Some(AlignSelf::Center));
}

#[test]
fn test_multiple_components_combined() {
    let mut world = World::new();

    let entity = world
        .spawn(BoxStyle {
            size: Some(BoxSize {
                width: Some(Dimension::Px(200.0)),
                height: Some(Dimension::Px(150.0)),
            }),
            margin: Some(BoxMargin(Rect {
                left: LengthPercentageAuto::Px(10.0),
                right: LengthPercentageAuto::Px(10.0),
                top: LengthPercentageAuto::Px(10.0),
                bottom: LengthPercentageAuto::Px(10.0),
            })),
            flex_grow: Some(1.0),
            flex_shrink: Some(1.0),
            flex_basis: Some(Dimension::Auto),
            ..Default::default()
        })
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(build_taffy_styles_system);
    schedule.run(&mut world);

    let taffy_style = world.get::<TaffyStyle>(entity).unwrap();
    let style = taffy_style.style();

    // BoxSize
    assert_eq!(style.size.width, ::taffy::Dimension::length(200.0));
    assert_eq!(style.size.height, ::taffy::Dimension::length(150.0));

    // BoxMargin
    assert_eq!(
        style.margin.left,
        ::taffy::LengthPercentageAuto::length(10.0)
    );
    assert_eq!(
        style.margin.right,
        ::taffy::LengthPercentageAuto::length(10.0)
    );
    assert_eq!(
        style.margin.top,
        ::taffy::LengthPercentageAuto::length(10.0)
    );
    assert_eq!(
        style.margin.bottom,
        ::taffy::LengthPercentageAuto::length(10.0)
    );

    // FlexItem
    assert_eq!(style.flex_grow, 1.0);
    assert_eq!(style.flex_shrink, 1.0);
    assert_eq!(style.flex_basis, ::taffy::Dimension::auto());
}
