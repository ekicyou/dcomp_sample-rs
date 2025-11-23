use bevy_ecs::prelude::*;
use taffy::prelude::*;
use wintf::ecs::layout::taffy::{TaffyComputedLayout, TaffyLayoutResource, TaffyStyle};
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

// ===== タスク7: ユニットテスト =====

/// テスト7.1: 高レベルコンポーネント→TaffyStyle変換テスト（BoxSize）
#[test]
fn test_box_size_to_taffy_style_conversion() {
    // BoxSize.widthとheightがtaffy::Dimensionに正しく変換されることを検証
    let box_size = BoxSize {
        width: Some(wintf::ecs::layout::Dimension::Px(200.0)),
        height: Some(wintf::ecs::layout::Dimension::Px(100.0)),
    };

    // Dimension→taffy::Dimensionの変換をテスト
    let taffy_width: taffy::Dimension = box_size.width.unwrap().into();
    let taffy_height: taffy::Dimension = box_size.height.unwrap().into();

    // 期待値と比較（taffy 0.9.1ではDimension::lengthを使用）
    assert_eq!(taffy_width, taffy::Dimension::length(200.0));
    assert_eq!(taffy_height, taffy::Dimension::length(100.0));
}

/// テスト7.2: EntityとNodeIdマッピングテスト
#[test]
fn test_entity_node_id_mapping() {
    let mut taffy_res = TaffyLayoutResource::default();
    let mut world = World::new();

    // エンティティ作成
    let entity1 = world.spawn_empty().id();
    let entity2 = world.spawn_empty().id();

    // ノード作成とマッピング登録
    let node1 = taffy_res.create_node(entity1).unwrap();
    let node2 = taffy_res.create_node(entity2).unwrap();

    // 順方向マッピング検証
    assert_eq!(taffy_res.get_node(entity1), Some(node1));
    assert_eq!(taffy_res.get_node(entity2), Some(node2));

    // 逆方向マッピング検証
    assert_eq!(taffy_res.get_entity(node1), Some(entity1));
    assert_eq!(taffy_res.get_entity(node2), Some(entity2));

    // 削除後のマッピング検証
    taffy_res.remove_node(entity1).unwrap();
    assert_eq!(taffy_res.get_node(entity1), None);
    assert_eq!(taffy_res.get_entity(node1), None);

    // entity2は削除されていないことを確認
    assert_eq!(taffy_res.get_node(entity2), Some(node2));
}

/// テスト7.3: マッピング整合性検証テスト
#[cfg(debug_assertions)]
#[test]
fn test_mapping_consistency() {
    let mut taffy_res = TaffyLayoutResource::default();
    let mut world = World::new();

    let entity = world.spawn_empty().id();
    let _node = taffy_res.create_node(entity).unwrap();

    // マッピング整合性検証（パニックしないことを確認）
    taffy_res.verify_mapping_consistency();
}

/// テスト7.4: 境界値シナリオテスト - 空ツリー
#[test]
fn test_empty_tree_scenario() {
    let taffy_res = TaffyLayoutResource::default();

    // 空のツリーでクラッシュしないことを検証
    assert_eq!(taffy_res.first_layout_done(), false);
}

/// テスト7.5: Dimensionのconst constructorテスト
#[test]
fn test_dimension_const_constructors() {
    const DIM_LENGTH: Dimension = Dimension::length(100.0);
    const DIM_PERCENT: Dimension = Dimension::percent(50.0);
    const DIM_AUTO: Dimension = Dimension::auto();
    const DIM_ZERO: Dimension = Dimension::zero();

    assert_eq!(DIM_LENGTH, Dimension::Px(100.0));
    assert_eq!(DIM_PERCENT, Dimension::Percent(50.0));
    assert_eq!(DIM_AUTO, Dimension::Auto);
    assert_eq!(DIM_ZERO, Dimension::Px(0.0));
}

/// テスト7.6: 複数エンティティの削除テスト
#[test]
fn test_multiple_entity_removal() {
    let mut taffy_res = TaffyLayoutResource::default();
    let mut world = World::new();

    // 3つのエンティティを作成
    let entity1 = world.spawn_empty().id();
    let entity2 = world.spawn_empty().id();
    let entity3 = world.spawn_empty().id();

    let _node1 = taffy_res.create_node(entity1).unwrap();
    let node2 = taffy_res.create_node(entity2).unwrap();
    let _node3 = taffy_res.create_node(entity3).unwrap();

    // entity1とentity3を削除
    taffy_res.remove_node(entity1).unwrap();
    taffy_res.remove_node(entity3).unwrap();

    // entity2のみ残っていることを確認
    assert_eq!(taffy_res.get_node(entity1), None);
    assert_eq!(taffy_res.get_node(entity2), Some(node2));
    assert_eq!(taffy_res.get_node(entity3), None);
}

/// テスト7.7: FlexContainerとFlexItemの統合テスト
#[test]
fn test_flex_container_and_item_integration() {
    let mut world = World::new();

    // FlexContainerを作成
    let container = FlexContainer {
        direction: taffy::FlexDirection::Row,
        align_items: Some(taffy::AlignItems::Center),
        justify_content: Some(taffy::JustifyContent::SpaceBetween),
    };

    // FlexItemを作成
    let item = FlexItem {
        grow: 1.0,
        shrink: 0.0,
        basis: Dimension::Percent(50.0),
        align_self: None,
    };

    let container_entity = world.spawn(container).id();
    let item_entity = world.spawn(item).id();

    // 両方のコンポーネントが正しく挿入されていることを確認
    assert!(world.entity(container_entity).contains::<FlexContainer>());
    assert!(world.entity(item_entity).contains::<FlexItem>());
}
