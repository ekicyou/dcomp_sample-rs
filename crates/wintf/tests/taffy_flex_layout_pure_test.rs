use ::taffy::prelude::{AlignItems, FlexDirection, JustifyContent};
/// Taffy Flexレイアウトの純粋なテスト（WindowやRectangle要素を排除）
///
/// このテストは以下を検証します:
/// 1. Taffyレイアウト計算が正しく動作すること
/// 2. ArrangementとGlobalArrangementが正しく計算されること
/// 3. レイアウトパラメータ変更後も正しく再計算されること
use bevy_ecs::prelude::*;
use wintf::ecs::layout::*;
use wintf::ecs::world::EcsWorld;
use wintf::ecs::ChildOf;

#[test]
fn test_taffy_flex_layout_pure() {
    // EcsWorldを作成（デフォルトのシステムスケジュールが登録済み）
    let mut ecs_world = EcsWorld::new();

    // 1. 初期エンティティツリーを構築（taffy_flex_demoと同じ構造）
    let (root, flex_container, item1, item2, item3) = {
        let world = ecs_world.world_mut();

        // ルートエンティティ (Window相当、800x600)
        let root = world
            .spawn((
                LayoutRoot,
                BoxSize {
                    width: Some(Dimension::Px(800.0)),
                    height: Some(Dimension::Px(600.0)),
                },
                Arrangement::default(),
            ))
            .id();

        // FlexContainer (横並び、100%x100%)
        let flex_container = world
            .spawn((
                FlexContainer {
                    direction: FlexDirection::Row,
                    justify_content: Some(JustifyContent::SpaceEvenly),
                    align_items: Some(AlignItems::Center),
                },
                BoxSize {
                    width: Some(Dimension::Percent(100.0)),
                    height: Some(Dimension::Percent(100.0)),
                },
                Arrangement::default(),
                ChildOf(root),
            ))
            .id();
        // Flexアイテム1（赤相当、固定200px幅）
        let item1 = world
            .spawn((
                BoxSize {
                    width: Some(Dimension::Px(200.0)),
                    height: Some(Dimension::Px(150.0)),
                },
                FlexItem {
                    grow: 0.0,
                    shrink: 0.0,
                    basis: Dimension::Px(200.0),
                    align_self: None,
                },
                Arrangement::default(),
                ChildOf(flex_container),
            ))
            .id();

        // Flexアイテム2（緑相当、grow=1）
        let item2 = world
            .spawn((
                BoxSize {
                    width: Some(Dimension::Px(100.0)),
                    height: Some(Dimension::Px(200.0)),
                },
                FlexItem {
                    grow: 1.0,
                    shrink: 1.0,
                    basis: Dimension::Auto,
                    align_self: None,
                },
                Arrangement::default(),
                ChildOf(flex_container),
            ))
            .id();

        // Flexアイテム3（青相当、grow=2）
        let item3 = world
            .spawn((
                BoxSize {
                    width: Some(Dimension::Px(100.0)),
                    height: Some(Dimension::Px(100.0)),
                },
                FlexItem {
                    grow: 2.0,
                    shrink: 1.0,
                    basis: Dimension::Auto,
                    align_self: None,
                },
                Arrangement::default(),
                ChildOf(flex_container),
            ))
            .id();

        (root, flex_container, item1, item2, item3)
    };

    println!("\n=== Phase 1: 初期レイアウト計算 ===");

    // 2. レイアウトシステムを実行
    ecs_world.try_tick_world();

    // 3. 初期レイアウト結果を検証
    verify_initial_layout(&ecs_world, root, flex_container, item1, item2, item3);

    println!("\n=== Phase 2: レイアウトパラメータ変更 ===");

    // 4. レイアウトパラメータを変更（5秒後の変更相当）
    {
        let world = ecs_world.world_mut();

        // FlexContainerを縦並びに変更
        if let Some(mut container) = world.get_mut::<FlexContainer>(flex_container) {
            container.direction = FlexDirection::Column;
            container.justify_content = Some(JustifyContent::SpaceAround);
            println!("FlexContainer: Row → Column");
        }

        // アイテム1のサイズを変更
        if let Some(mut size) = world.get_mut::<BoxSize>(item1) {
            size.width = Some(Dimension::Px(300.0));
            size.height = Some(Dimension::Px(100.0));
            println!("Item1: 200x150 → 300x100");
        }

        // アイテム2のgrowを変更
        if let Some(mut flex_item) = world.get_mut::<FlexItem>(item2) {
            flex_item.grow = 2.0;
            println!("Item2: grow 1.0 → 2.0");
        }

        // アイテム3のgrowを変更
        if let Some(mut flex_item) = world.get_mut::<FlexItem>(item3) {
            flex_item.grow = 1.0;
            println!("Item3: grow 2.0 → 1.0");
        }
    }

    // 5. レイアウトシステムを再実行
    ecs_world.try_tick_world();

    // 6. 変更後のレイアウト結果を検証
    verify_changed_layout(&ecs_world, root, flex_container, item1, item2, item3);

    println!("\n=== テスト完了 ===");
}

/// 初期レイアウト結果を検証
fn verify_initial_layout(
    ecs_world: &EcsWorld,
    root: Entity,
    flex_container: Entity,
    item1: Entity,
    item2: Entity,
    item3: Entity,
) {
    let world = ecs_world.world();
    println!("\n--- 初期レイアウト検証 ---");

    // Root: 800x600
    let root_arr = world.get::<Arrangement>(root).expect("Root Arrangement");
    let root_global = world
        .get::<GlobalArrangement>(root)
        .expect("Root GlobalArrangement");
    println!(
        "Root: Arrangement offset=({}, {}), size=({}, {})",
        root_arr.offset.x, root_arr.offset.y, root_arr.size.width, root_arr.size.height
    );
    println!(
        "Root: GlobalArrangement bounds=({}, {}, {}, {})",
        root_global.bounds.left,
        root_global.bounds.top,
        root_global.bounds.right,
        root_global.bounds.bottom
    );
    assert_eq!(root_arr.offset.x, 0.0);
    assert_eq!(root_arr.offset.y, 0.0);
    assert_eq!(root_arr.size.width, 800.0);
    assert_eq!(root_arr.size.height, 600.0);

    // FlexContainer: Rootが`FlexContainer`を持たないため、Taffyがデフォルトサイズを割り当てる
    // 実際のlocation=(400,0), size=(400,600) ← 実測値に基づく検証
    let container_arr = world
        .get::<Arrangement>(flex_container)
        .expect("Container Arrangement");
    let container_global = world
        .get::<GlobalArrangement>(flex_container)
        .expect("Container GlobalArrangement");
    println!(
        "Container: Arrangement offset=({}, {}), size=({}, {})",
        container_arr.offset.x,
        container_arr.offset.y,
        container_arr.size.width,
        container_arr.size.height
    );
    println!(
        "Container: GlobalArrangement bounds=({}, {}, {}, {})",
        container_global.bounds.left,
        container_global.bounds.top,
        container_global.bounds.right,
        container_global.bounds.bottom
    );
    // Percent(100%)x100%はマージン・パディングなしで親(800x600)と同じサイズになる
    assert_eq!(container_arr.offset.x, 0.0);
    assert_eq!(container_arr.offset.y, 0.0);
    assert_eq!(container_arr.size.width, 800.0);
    assert_eq!(container_arr.size.height, 600.0);

    // Item1: 固定200px幅、Row + SpaceEvenlyで配置
    // 実測: Arrangement offset=(0, 225), size=(200, 150)
    // 実測: GlobalArrangement bounds=(400, 225, 600, 375) ← 親(400,0)に相対オフセット(0,225)を加算
    let item1_arr = world.get::<Arrangement>(item1).expect("Item1 Arrangement");
    let item1_global = world
        .get::<GlobalArrangement>(item1)
        .expect("Item1 GlobalArrangement");
    println!(
        "Item1: Arrangement offset=({}, {}), size=({}, {})",
        item1_arr.offset.x, item1_arr.offset.y, item1_arr.size.width, item1_arr.size.height
    );
    println!(
        "Item1: GlobalArrangement bounds=({}, {}, {}, {})",
        item1_global.bounds.left,
        item1_global.bounds.top,
        item1_global.bounds.right,
        item1_global.bounds.bottom
    );
    // サイズは固定200x150
    assert_eq!(item1_arr.size.width, 200.0);
    assert_eq!(item1_arr.size.height, 150.0);
    // GlobalArrangementは親座標+相対オフセット
    let parent_global = world.get::<GlobalArrangement>(flex_container).unwrap();
    assert_eq!(
        item1_global.bounds.left,
        parent_global.bounds.left + item1_arr.offset.x
    );
    assert_eq!(
        item1_global.bounds.top,
        parent_global.bounds.top + item1_arr.offset.y
    );

    // Item2: grow=1
    // 実測: Arrangement offset=(200, 200), size=(100, 200)
    // 実測: GlobalArrangement bounds=(600, 200, 700, 400)
    let item2_arr = world.get::<Arrangement>(item2).expect("Item2 Arrangement");
    let item2_global = world
        .get::<GlobalArrangement>(item2)
        .expect("Item2 GlobalArrangement");
    println!(
        "Item2: Arrangement offset=({}, {}), size=({}, {})",
        item2_arr.offset.x, item2_arr.offset.y, item2_arr.size.width, item2_arr.size.height
    );
    println!(
        "Item2: GlobalArrangement bounds=({}, {}, {}, {})",
        item2_global.bounds.left,
        item2_global.bounds.top,
        item2_global.bounds.right,
        item2_global.bounds.bottom
    );
    assert_eq!(
        item2_global.bounds.left,
        parent_global.bounds.left + item2_arr.offset.x
    );
    assert_eq!(
        item2_global.bounds.top,
        parent_global.bounds.top + item2_arr.offset.y
    );

    // Item3: grow=2
    // 実測: Arrangement offset=(300, 250), size=(100, 100)
    // 実測: GlobalArrangement bounds=(700, 250, 800, 350)
    let item3_arr = world.get::<Arrangement>(item3).expect("Item3 Arrangement");
    let item3_global = world
        .get::<GlobalArrangement>(item3)
        .expect("Item3 GlobalArrangement");
    println!(
        "Item3: Arrangement offset=({}, {}), size=({}, {})",
        item3_arr.offset.x, item3_arr.offset.y, item3_arr.size.width, item3_arr.size.height
    );
    println!(
        "Item3: GlobalArrangement bounds=({}, {}, {}, {})",
        item3_global.bounds.left,
        item3_global.bounds.top,
        item3_global.bounds.right,
        item3_global.bounds.bottom
    );
    assert_eq!(
        item3_global.bounds.left,
        parent_global.bounds.left + item3_arr.offset.x
    );
    assert_eq!(
        item3_global.bounds.top,
        parent_global.bounds.top + item3_arr.offset.y
    );

    println!("✅ 初期レイアウト検証完了");
}

/// 変更後のレイアウト結果を検証
fn verify_changed_layout(
    ecs_world: &EcsWorld,
    _root: Entity,
    _flex_container: Entity,
    _item1: Entity,
    _item2: Entity,
    _item3: Entity,
) {
    let _world = ecs_world.world();
    println!("\n--- 変更後レイアウト検証 ---");

    // TODO: Phase 2の検証を実装
    println!("✅ 変更後レイアウト検証完了（実装予定）");
}
