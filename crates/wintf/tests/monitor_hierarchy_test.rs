//! # Monitor階層システムの統合テスト
//!
//! 仮想デスクトップ・モニター階層システムの動作を検証する。

use bevy_ecs::prelude::*;
use bevy_ecs::hierarchy::ChildOf;
use wintf::ecs::*;

// ===== Task 11.1: LayoutRoot Singleton生成とMonitor列挙テスト =====

#[test]
fn test_layout_root_singleton_creation() {
    let mut world = World::new();
    world.insert_resource(TaffyLayoutResource::default());

    // initialize_layout_root_systemを実行
    let mut schedule = Schedule::default();
    schedule.add_systems(initialize_layout_root_system);
    schedule.run(&mut world);

    // LayoutRootが1つだけ生成されることを検証
    let layout_roots: Vec<Entity> = world
        .query_filtered::<Entity, With<LayoutRoot>>()
        .iter(&world)
        .collect();
    assert_eq!(layout_roots.len(), 1, "LayoutRoot should be created exactly once");

    // 2回目の実行でもLayoutRootが1つだけであることを検証
    schedule.run(&mut world);
    let layout_roots: Vec<Entity> = world
        .query_filtered::<Entity, With<LayoutRoot>>()
        .iter(&world)
        .collect();
    assert_eq!(
        layout_roots.len(),
        1,
        "LayoutRoot should not be duplicated on subsequent runs"
    );
}

#[test]
fn test_monitor_enumeration() {
    let mut world = World::new();
    world.insert_resource(TaffyLayoutResource::default());

    // initialize_layout_root_systemを実行
    let mut schedule = Schedule::default();
    schedule.add_systems(initialize_layout_root_system);
    schedule.run(&mut world);

    // Monitorエンティティが生成されることを検証
    let monitor_count = world.query::<&Monitor>().iter(&world).count();

    // システムに少なくとも1つのモニターが存在することを検証
    assert!(monitor_count >= 1, "At least one monitor should be enumerated");

    // LayoutRootエンティティを取得
    let mut layout_root_query = world.query_filtered::<Entity, With<LayoutRoot>>();
    let layout_root_entity = layout_root_query.iter(&world).next().expect("LayoutRoot should exist");

    // 各MonitorエンティティがLayoutRootの子であることを検証
    for (entity, monitor) in world.query::<(Entity, &Monitor)>().iter(&world) {
        let child_of = world
            .get::<ChildOf>(entity)
            .expect("Monitor should have ChildOf component");
        assert_eq!(
            child_of.parent(),
            layout_root_entity,
            "Monitor should be a child of LayoutRoot"
        );

        // Monitor情報の妥当性を検証
        assert!(monitor.dpi > 0, "Monitor DPI should be positive");
        assert!(
            monitor.bounds.right > monitor.bounds.left,
            "Monitor bounds width should be positive"
        );
        assert!(
            monitor.bounds.bottom > monitor.bounds.top,
            "Monitor bounds height should be positive"
        );
    }
}

// ===== Task 11.2: LayoutRoot → {Monitor, Window} → Widget階層構築テスト =====

#[test]
fn test_monitor_hierarchy_construction() {
    let mut world = World::new();
    world.insert_resource(TaffyLayoutResource::default());

    // initialize_layout_root_systemを実行
    let mut schedule = Schedule::default();
    schedule.add_systems(initialize_layout_root_system);
    schedule.run(&mut world);

    // LayoutRootを取得
    let mut layout_root_query = world.query_filtered::<Entity, With<LayoutRoot>>();
    let layout_root = layout_root_query.iter(&world).next().expect("LayoutRoot should exist");

    // Monitorエンティティを取得
    let monitors: Vec<Entity> = world
        .query_filtered::<Entity, With<Monitor>>()
        .iter(&world)
        .collect();

    // 各MonitorがLayoutRootの子であることを再検証
    for monitor_entity in monitors.iter() {
        let child_of = world
            .get::<ChildOf>(*monitor_entity)
            .expect("Monitor should have ChildOf");
        assert_eq!(child_of.parent(), layout_root);
    }

    // Monitorに必要なレイアウトコンポーネントが存在することを検証
    for monitor_entity in monitors.iter() {
        assert!(
            world.get::<BoxPosition>(*monitor_entity).is_some(),
            "Monitor should have BoxPosition component"
        );
        assert!(
            world.get::<BoxSize>(*monitor_entity).is_some(),
            "Monitor should have BoxSize component"
        );
        assert!(
            world.get::<BoxInset>(*monitor_entity).is_some(),
            "Monitor should have BoxInset component"
        );
        assert!(
            world.get::<Arrangement>(*monitor_entity).is_some(),
            "Monitor should have Arrangement component"
        );
        assert!(
            world.get::<GlobalArrangement>(*monitor_entity).is_some(),
            "Monitor should have GlobalArrangement component"
        );
    }
}

// ===== Task 12.1: Monitor.boundsからTaffyStyle変換テスト =====

#[test]
fn test_monitor_to_taffy_style_conversion() {
    let mut world = World::new();
    world.insert_resource(TaffyLayoutResource::default());

    // initialize_layout_root_systemを実行
    let mut schedule = Schedule::default();
    schedule.add_systems(initialize_layout_root_system);
    schedule.run(&mut world);

    // build_taffy_styles_systemを実行してTaffyStyleを生成
    let mut schedule2 = Schedule::default();
    schedule2.add_systems(build_taffy_styles_system);
    schedule2.run(&mut world);

    // 各MonitorのTaffyStyleを検証
    let mut has_monitors = false;
    for (_monitor, _box_position, _box_size, _box_inset, taffy_style) in world
        .query::<(&Monitor, &BoxPosition, &BoxSize, &BoxInset, &TaffyStyle)>()
        .iter(&world)
    {
        has_monitors = true;

        // BoxPosition::Absoluteが設定されていることを検証
        // (TaffyのPositionは内部型のため直接検証できない)
        println!("Monitor has TaffyStyle");
    }

    assert!(has_monitors, "At least one monitor should have TaffyStyle");
}

// ===== Task 13.1: Taffyツリー同期とレイアウト計算テスト =====

#[test]
fn test_taffy_tree_sync_and_layout_computation() {
    let mut world = World::new();
    world.insert_resource(TaffyLayoutResource::default());

    // LayoutRootとMonitorを初期化
    let mut schedule = Schedule::default();
    schedule.add_systems(initialize_layout_root_system);
    schedule.run(&mut world);

    // TaffyStyleを構築
    let mut schedule2 = Schedule::default();
    schedule2.add_systems(build_taffy_styles_system);
    schedule2.run(&mut world);

    // Taffyツリーを同期
    let mut schedule3 = Schedule::default();
    schedule3.add_systems(sync_taffy_tree_system);
    schedule3.run(&mut world);

    // LayoutRootとMonitorのEntityを先に取得
    let layout_root = world
        .query_filtered::<Entity, With<LayoutRoot>>()
        .iter(&world)
        .next()
        .expect("LayoutRoot should exist");

    let monitors: Vec<Entity> = world
        .query_filtered::<Entity, With<Monitor>>()
        .iter(&world)
        .collect();

    // Entity↔NodeIdマッピングの検証
    {
        let taffy_res = world.resource::<TaffyLayoutResource>();

        // LayoutRootのマッピング検証
        assert!(
            taffy_res.get_node(layout_root).is_some(),
            "LayoutRoot should have a Taffy node"
        );

        // Monitorのマッピング検証
        for monitor_entity in monitors.iter() {
            assert!(
                taffy_res.get_node(*monitor_entity).is_some(),
                "Monitor should have a Taffy node"
            );
        }
    }

    // レイアウト計算を実行
    let mut schedule4 = Schedule::default();
    schedule4.add_systems(compute_taffy_layout_system);
    schedule4.run(&mut world);

    // TaffyComputedLayoutが配布されていることを検証
    let _layout_root_computed = world
        .get::<TaffyComputedLayout>(layout_root)
        .expect("LayoutRoot should have TaffyComputedLayout");

    for monitor_entity in monitors.iter() {
        let _monitor_computed = world
            .get::<TaffyComputedLayout>(*monitor_entity)
            .expect("Monitor should have TaffyComputedLayout");

        // レイアウトが計算されていることを検証（内部構造にアクセスできないため存在確認のみ）
        println!("Monitor {:?} has computed layout", monitor_entity);
    }
}

// ===== Task 14.1: DisplayConfigurationChangedフラグテスト =====

#[test]
fn test_display_configuration_changed_flag() {
    let mut app = App::new();

    // 初期状態ではフラグがfalse
    assert!(
        !app.display_configuration_changed(),
        "Flag should be false initially"
    );

    // mark_display_change()でフラグがtrueになる
    app.mark_display_change();
    assert!(
        app.display_configuration_changed(),
        "Flag should be true after mark_display_change"
    );

    // reset_display_change()でフラグがfalseになる
    app.reset_display_change();
    assert!(
        !app.display_configuration_changed(),
        "Flag should be false after reset_display_change"
    );
}

// ===== Task 14.2: モニター追加・削除・更新テスト =====

#[test]
fn test_monitor_update_on_change() {
    let mut world = World::new();
    world.insert_resource(TaffyLayoutResource::default());
    world.insert_resource(App::new());

    // LayoutRootとMonitorを初期化
    let mut schedule = Schedule::default();
    schedule.add_systems(initialize_layout_root_system);
    schedule.run(&mut world);

    // 初期のMonitor数を取得
    let initial_count = world.query::<&Monitor>().iter(&world).count();

    // ディスプレイ構成変更をシミュレート
    {
        let mut app = world.resource_mut::<App>();
        app.mark_display_change();
    }

    // detect_display_change_systemを実行
    let mut schedule2 = Schedule::default();
    schedule2.add_systems(detect_display_change_system);
    schedule2.run(&mut world);

    // フラグがリセットされていることを検証
    let app = world.resource::<App>();
    assert!(
        !app.display_configuration_changed(),
        "Flag should be reset after detect_display_change_system"
    );

    // Monitor数が維持されていることを検証（実際の環境では変化しない想定）
    let current_count = world.query::<&Monitor>().iter(&world).count();
    assert_eq!(
        initial_count, current_count,
        "Monitor count should remain the same in stable environment"
    );
}

// ===== Task 15.1: 既存システム互換性テスト =====

#[test]
fn test_backward_compatibility_without_layout_root() {
    let mut world = World::new();
    world.insert_resource(TaffyLayoutResource::default());

    // LayoutRootなしでWidgetエンティティを作成
    let widget = world
        .spawn((
            BoxSize {
                width: Some(Dimension::Px(100.0)),
                height: Some(Dimension::Px(50.0)),
            },
            Arrangement::default(),
            GlobalArrangement::default(),
        ))
        .id();

    // build_taffy_styles_systemを実行
    let mut schedule = Schedule::default();
    schedule.add_systems(build_taffy_styles_system);
    schedule.run(&mut world);

    // TaffyStyleが自動生成されることを検証
    assert!(
        world.get::<TaffyStyle>(widget).is_some(),
        "TaffyStyle should be auto-generated even without LayoutRoot"
    );

    // sync_taffy_tree_systemを実行
    let mut schedule2 = Schedule::default();
    schedule2.add_systems(sync_taffy_tree_system);
    schedule2.run(&mut world);

    // Taffyノードが作成されることを検証
    let taffy_res = world.resource::<TaffyLayoutResource>();
    assert!(
        taffy_res.get_node(widget).is_some(),
        "Taffy node should be created even without LayoutRoot"
    );
}

#[test]
fn test_existing_tests_still_pass() {
    // 既存のレイアウトシステムが正常に動作することを検証
    let mut world = World::new();
    world.insert_resource(TaffyLayoutResource::default());

    // LayoutRootを作成（新システム）
    world.insert_resource(App::new());
    let mut schedule = Schedule::default();
    schedule.add_systems(initialize_layout_root_system);
    schedule.run(&mut world);

    // 既存のWindow/Widgetエンティティを作成
    let window = world
        .spawn((
            BoxSize {
                width: Some(Dimension::Px(800.0)),
                height: Some(Dimension::Px(600.0)),
            },
            FlexContainer::default(),
            Arrangement::default(),
            GlobalArrangement::default(),
        ))
        .id();

    let widget = world
        .spawn((
            BoxSize {
                width: Some(Dimension::Px(200.0)),
                height: Some(Dimension::Px(100.0)),
            },
            ChildOf(window),
            Arrangement::default(),
            GlobalArrangement::default(),
        ))
        .id();

    // レイアウトシステムを実行
    let mut schedule2 = Schedule::default();
    schedule2.add_systems((
        build_taffy_styles_system,
        sync_taffy_tree_system,
        compute_taffy_layout_system,
    ));
    schedule2.run(&mut world);

    // Window/Widgetのレイアウトが正しく計算されることを検証
    assert!(
        world.get::<TaffyComputedLayout>(window).is_some(),
        "Window should have computed layout"
    );
    assert!(
        world.get::<TaffyComputedLayout>(widget).is_some(),
        "Widget should have computed layout"
    );
}
