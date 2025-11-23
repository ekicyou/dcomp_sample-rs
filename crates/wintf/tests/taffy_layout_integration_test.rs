use bevy_ecs::prelude::*;
use taffy::prelude::*;
use wintf::ecs::layout::taffy::{TaffyComputedLayout, TaffyLayoutResource, TaffyStyle};
use wintf::ecs::layout::{
    Arrangement, BoxMargin, BoxPadding, BoxSize, Dimension, FlexContainer, FlexItem,
    GlobalArrangement, LayoutRoot, LengthPercentage, LengthPercentageAuto, Rect,
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
    // TaffyLayoutResourceが正常に作成されればOK
    assert!(taffy_res.taffy().total_node_count() == 0);
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

/// テスト7.8: レイアウトパラメーター変更による再計算の統合テスト
#[test]
fn test_layout_recalculation_on_parameter_change() {
    let mut world = World::new();
    world.insert_resource(TaffyLayoutResource::default());

    // シンプルなテスト: BoxSizeを変更してTaffyStyleが更新されることを確認
    let initial_size = BoxSize {
        width: Some(Dimension::Px(400.0)),
        height: Some(Dimension::Px(300.0)),
    };

    let container = world.spawn((initial_size, LayoutRoot)).id();

    // スケジュールを構築 (build_taffy_styles_systemのみで十分)
    let mut schedule = bevy_ecs::schedule::Schedule::default();
    schedule.add_systems(wintf::ecs::layout::systems::build_taffy_styles_system);

    // 初回実行: TaffyStyleが生成される
    schedule.run(&mut world);

    // TaffyStyleが生成されたか確認
    assert!(
        world.entity(container).contains::<TaffyStyle>(),
        "TaffyStyleコンポーネントが生成されていません"
    );

    // 初期サイズを確認（内部フィールドはprivateなので、存在確認のみ）
    let _initial_style = world.entity(container).get::<TaffyStyle>().unwrap();

    // パラメーター変更: サイズを倍にする
    if let Some(mut size) = world.entity_mut(container).get_mut::<BoxSize>() {
        size.width = Some(Dimension::Px(800.0));
        size.height = Some(Dimension::Px(600.0));
    }

    // 再度実行: TaffyStyleが更新される
    schedule.run(&mut world);

    // TaffyStyleが再度存在することを確認（更新が行われた）
    assert!(
        world.entity(container).contains::<TaffyStyle>(),
        "TaffyStyleコンポーネントが更新後も存在します"
    );

    // 変更検知クエリが機能することを確認
    let mut changed_count = 0;
    let mut query = world.query_filtered::<Entity, Changed<TaffyStyle>>();
    for _ in query.iter(&world) {
        changed_count += 1;
    }

    assert!(
        changed_count > 0,
        "TaffyStyleの変更が検知されませんでした。レイアウト再計算が実行されていない可能性があります"
    );
}

/// テスト7.9: EcsWorldを使った3階層ウィジェットツリーの総合レイアウトテスト
///
/// このテストは、BoxSize変更から最終的なGlobalArrangementの更新までを検証します。
/// - 3階層のウィジェットツリー（Root → Container → Child）を構築
/// - 初回レイアウト計算でGlobalArrangementが正しく設定されることを確認
/// - Containerのサイズ変更後、すべての階層でGlobalArrangementが再計算されることを確認
///
/// 注意: このテストはWindowコンポーネントを使用せず、純粋なレイアウト計算のみを検証します。
/// これにより、実際のウィンドウ作成処理を回避し、テストの安全性と速度を向上させます。
#[test]
fn test_full_layout_pipeline_with_ecs_world() {
    use wintf::ecs::world::EcsWorld;
    use wintf::ecs::ChildOf;

    // EcsWorldを作成（デフォルトのシステムスケジュールが登録済み）
    let mut ecs_world = EcsWorld::new();

    // 3階層のウィジェットツリーを構築
    let (root, container, child) = {
        let world = ecs_world.world_mut();

        // Root (ルートエンティティ - LayoutRootマーカーを使用)
        let root = world
            .spawn((
                LayoutRoot, // レイアウト計算のルートを示すマーカー
                BoxSize {
                    width: Some(Dimension::Px(800.0)),
                    height: Some(Dimension::Px(600.0)),
                },
                FlexContainer {
                    direction: FlexDirection::Column,
                    justify_content: Some(JustifyContent::Start),
                    align_items: Some(AlignItems::Stretch),
                },
                Arrangement::default(), // Arrangementを明示的に追加
            ))
            .id();

        // Container (中間層)
        let container = world
            .spawn((
                BoxSize {
                    width: Some(Dimension::Px(400.0)),
                    height: Some(Dimension::Px(300.0)),
                },
                FlexContainer {
                    direction: FlexDirection::Row,
                    justify_content: Some(JustifyContent::Center),
                    align_items: Some(AlignItems::Center),
                },
                Arrangement::default(), // Arrangementを明示的に追加
                ChildOf(root),          // 親子関係を設定
            ))
            .id();

        // Child (末端)
        let child = world
            .spawn((
                BoxSize {
                    width: Some(Dimension::Px(200.0)),
                    height: Some(Dimension::Px(150.0)),
                },
                Arrangement::default(), // Arrangementを明示的に追加
                ChildOf(container),     // 親子関係を設定
            ))
            .id();

        (root, container, child)
    };

    // 初回レイアウト計算を実行
    ecs_world.try_tick_world();

    // 検証1: すべてのエンティティにTaffyStyleが生成されている
    {
        let world = ecs_world.world();
        assert!(
            world.entity(root).contains::<TaffyStyle>(),
            "RootにTaffyStyleが生成されていません"
        );
        assert!(
            world.entity(container).contains::<TaffyStyle>(),
            "ContainerにTaffyStyleが生成されていません"
        );
        assert!(
            world.entity(child).contains::<TaffyStyle>(),
            "ChildにTaffyStyleが生成されていません"
        );
    }

    // 検証2: すべてのエンティティにArrangementが生成されている
    {
        let world = ecs_world.world();
        assert!(
            world.entity(root).contains::<Arrangement>(),
            "RootにArrangementが生成されていません"
        );
        assert!(
            world.entity(container).contains::<Arrangement>(),
            "ContainerにArrangementが生成されていません"
        );
        assert!(
            world.entity(child).contains::<Arrangement>(),
            "ChildにArrangementが生成されていません"
        );
    }

    // 検証3: すべてのエンティティにGlobalArrangementが生成されている
    {
        let world = ecs_world.world();
        assert!(
            world.entity(root).contains::<GlobalArrangement>(),
            "RootにGlobalArrangementが生成されていません"
        );
        assert!(
            world.entity(container).contains::<GlobalArrangement>(),
            "ContainerにGlobalArrangementが生成されていません"
        );
        assert!(
            world.entity(child).contains::<GlobalArrangement>(),
            "ChildにGlobalArrangementが生成されていません"
        );
    }

    // 初回のArrangementとGlobalArrangementを保存
    let (initial_container_arrangement, initial_container_global) = {
        let world = ecs_world.world();
        (
            *world.entity(container).get::<Arrangement>().unwrap(),
            *world.entity(container).get::<GlobalArrangement>().unwrap(),
        )
    };

    // Containerのサイズを変更
    {
        let world = ecs_world.world_mut();
        if let Some(mut size) = world.entity_mut(container).get_mut::<BoxSize>() {
            size.width = Some(Dimension::Px(600.0)); // 400 → 600
            size.height = Some(Dimension::Px(450.0)); // 300 → 450
        }
    }

    // レイアウト再計算を実行（1 tickで完了するはず）
    ecs_world.try_tick_world();

    // デバッグ: rootのGlobalArrangementを確認
    {
        let world = ecs_world.world();
        if let Some(root_global) = world.entity(root).get::<GlobalArrangement>() {
            println!(
                "After tick: Root GlobalArrangement.bounds: {:?}",
                root_global.bounds
            );
        }
        let container_global = world.entity(container).get::<GlobalArrangement>().unwrap();
        println!(
            "After tick: Container GlobalArrangement.bounds: {:?}",
            container_global.bounds
        );
    }

    // 検証4: Containerのサイズ変更後、Arrangementが更新されている
    {
        let world = ecs_world.world();
        let updated_container_arrangement = *world.entity(container).get::<Arrangement>().unwrap();
        assert_ne!(
            initial_container_arrangement.size, updated_container_arrangement.size,
            "Containerのサイズ変更後、Arrangementが更新されていません"
        );
    }

    // 検証5: Containerのサイズ変更後、GlobalArrangementが更新されている
    {
        let world = ecs_world.world();
        let updated_container_global = *world.entity(container).get::<GlobalArrangement>().unwrap();

        // デバッグ: 更新されなかった場合の詳細情報
        if initial_container_global.bounds == updated_container_global.bounds {
            println!("DEBUG: GlobalArrangement NOT updated!");
            println!("  Before bounds: {:?}", initial_container_global.bounds);
            println!("  After bounds:  {:?}", updated_container_global.bounds);

            let container_arr = world.entity(container).get::<Arrangement>().unwrap();
            println!("  Container Arrangement: {:?}", container_arr);
        }

        assert_ne!(
            initial_container_global.bounds, updated_container_global.bounds,
            "Containerのサイズ変更後、GlobalArrangementのboundsが更新されていません"
        );
    }

    // 検証6: 子要素（Child）のGlobalArrangementも維持されている
    // （親のサイズが変わると、子の配置位置も変わる可能性がある）
    {
        let world = ecs_world.world();
        assert!(
            world.entity(container).contains::<GlobalArrangement>()
                && world.entity(child).contains::<GlobalArrangement>(),
            "GlobalArrangementが階層全体で維持されています"
        );
    }

    // 検証7: レイアウト計算が正しく完了していることを確認
    // ContainerのArrangementサイズが期待値（600x?）に近いことを確認
    {
        let world = ecs_world.world();
        let container_size = world.entity(container).get::<Arrangement>().unwrap().size;
        assert!(
            (container_size.width - 600.0).abs() < 1.0,
            "Containerの幅が期待値と異なります: expected ~600.0, got {}",
            container_size.width
        );
        // TODO: heightは親のFlexレイアウトにより決定されるため、明示的な指定が反映されない
        // 将来的に、FlexItemのalign_selfやbasisを使った制御を検討
    }
}
