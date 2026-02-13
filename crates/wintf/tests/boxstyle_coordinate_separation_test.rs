//! BoxStyle座標分離テスト
//!
//! boxstyle-coordinate-separation仕様の統合テスト:
//! - Task 7.1: BoxStyle.inset 不変性
//! - Task 7.2: Changed<BoxStyle> 発火タイミング
//! - Task 7.3: ドラッグ終了同期
//! - Task 7.4: WindowDragging ライフサイクル
//! - Task 7.5: update_arrangements Window offset スキップ

use bevy_ecs::prelude::*;
use std::time::Instant;
use windows::Win32::Foundation::{POINT, SIZE};
use wintf::ecs::drag::{
    DragAccumulatorResource, DragConfig, DragEndEvent, DragEvent, DragStartEvent, DragTransition,
    WindowDragContextResource, WindowDragging,
};
use wintf::ecs::layout::systems::update_arrangements_system;
use wintf::ecs::layout::taffy::{TaffyComputedLayout, TaffyStyle};
use wintf::ecs::layout::{BoxSize, BoxStyle, Dimension, sync_window_arrangement_from_window_pos};
use wintf::ecs::pointer::PhysicalPoint;
use wintf::ecs::window::{DPI, Window, WindowPos};
use wintf::ecs::world::FrameCount;
use wintf::ecs::{Arrangement, GlobalArrangement, LayoutScale, Offset, Size, dispatch_drag_events};

// =============================================================================
// Task 7.1: BoxStyle.inset 不変性テスト
// =============================================================================

/// WM_WINDOWPOSCHANGED 相当の処理後（PostLayoutパイプライン経由）で
/// Window entity の BoxStyle.inset が変更されていないことを検証
#[test]
fn test_boxstyle_inset_unchanged_after_window_pos_update() {
    let mut world = World::new();
    world.insert_resource(FrameCount(0));

    // Window entity を作成（BoxStyle.inset は None のまま）
    let entity = world
        .spawn((
            Window::default(),
            WindowPos {
                position: Some(POINT { x: 100, y: 200 }),
                size: Some(SIZE { cx: 800, cy: 600 }),
                ..Default::default()
            },
            DPI::default(),
            BoxStyle::default(),
            Arrangement {
                offset: Offset { x: 0.0, y: 0.0 },
                scale: LayoutScale { x: 1.0, y: 1.0 },
                size: Size {
                    width: 800.0,
                    height: 600.0,
                },
            },
            GlobalArrangement::default(),
        ))
        .id();

    // PostLayout スケジュール（sync_window_arrangement_from_window_pos）を実行
    let mut schedule = Schedule::default();
    schedule.add_systems(sync_window_arrangement_from_window_pos);
    schedule.run(&mut world);

    // BoxStyle.inset は変更されていないこと（None のまま）
    let box_style = world.get::<BoxStyle>(entity).unwrap();
    assert!(
        box_style.inset.is_none(),
        "BoxStyle.inset は sync_window_arrangement_from_window_pos 後も None であるべき"
    );

    // Arrangement.offset は WindowPos.position に追従していること
    let arr = world.get::<Arrangement>(entity).unwrap();
    assert_eq!(arr.offset.x, 100.0);
    assert_eq!(arr.offset.y, 200.0);
}

/// WindowPos.position を変更してもBoxStyle.insetが影響を受けないことを検証
#[test]
fn test_boxstyle_inset_unaffected_by_window_position_change() {
    let mut world = World::new();
    world.insert_resource(FrameCount(0));

    let entity = world
        .spawn((
            Window::default(),
            WindowPos {
                position: Some(POINT { x: 100, y: 200 }),
                size: Some(SIZE { cx: 800, cy: 600 }),
                ..Default::default()
            },
            DPI::default(),
            BoxStyle::default(),
            Arrangement {
                offset: Offset { x: 100.0, y: 200.0 },
                scale: LayoutScale { x: 1.0, y: 1.0 },
                size: Size {
                    width: 800.0,
                    height: 600.0,
                },
            },
            GlobalArrangement::default(),
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(sync_window_arrangement_from_window_pos);

    // 初回実行（Added 扱い）
    schedule.run(&mut world);

    // WindowPos.position を変更（ウィンドウ移動を模擬）
    world.get_mut::<WindowPos>(entity).unwrap().position = Some(POINT { x: 500, y: 300 });
    schedule.run(&mut world);

    // BoxStyle.inset は変更されていないこと
    let box_style = world.get::<BoxStyle>(entity).unwrap();
    assert!(
        box_style.inset.is_none(),
        "位置変更後も BoxStyle.inset は None であるべき"
    );

    // Arrangement.offset は新しい位置に更新されること
    let arr = world.get::<Arrangement>(entity).unwrap();
    assert_eq!(arr.offset.x, 500.0);
    assert_eq!(arr.offset.y, 300.0);
}

// =============================================================================
// Task 7.2: Changed<BoxStyle> 発火タイミングテスト
// =============================================================================

/// 位置のみ変更時に Changed<BoxStyle> が発火しないことを検証
///
/// WindowPos.position の変更は sync_window_arrangement_from_window_pos 経由で
/// Arrangement.offset を更新するが、BoxStyle には触らない。
#[test]
fn test_changed_boxstyle_not_fired_on_position_only_change() {
    let mut world = World::new();
    world.insert_resource(FrameCount(0));

    let entity = world
        .spawn((
            Window::default(),
            WindowPos {
                position: Some(POINT { x: 100, y: 200 }),
                size: Some(SIZE { cx: 800, cy: 600 }),
                ..Default::default()
            },
            DPI::default(),
            BoxStyle {
                size: Some(BoxSize {
                    width: Some(Dimension::Px(800.0)),
                    height: Some(Dimension::Px(600.0)),
                }),
                ..Default::default()
            },
            Arrangement::default(),
            GlobalArrangement::default(),
        ))
        .id();

    // Changed<BoxStyle> 検知用システム
    fn detect_boxstyle_change(
        query: Query<Entity, Changed<BoxStyle>>,
        mut changed_count: ResMut<ChangedBoxStyleCount>,
    ) {
        for _entity in query.iter() {
            changed_count.0 += 1;
        }
    }

    #[derive(Resource, Default)]
    struct ChangedBoxStyleCount(u32);

    world.insert_resource(ChangedBoxStyleCount::default());

    let mut schedule = Schedule::default();
    schedule.add_systems((
        sync_window_arrangement_from_window_pos,
        detect_boxstyle_change.after(sync_window_arrangement_from_window_pos),
    ));

    // 初回実行: Added 扱いで Changed<BoxStyle> が 1回発火
    schedule.run(&mut world);
    let initial_count = world.resource::<ChangedBoxStyleCount>().0;

    // WindowPos.position のみ変更（BoxStyle は触らない）
    world.get_mut::<WindowPos>(entity).unwrap().position = Some(POINT { x: 500, y: 300 });
    schedule.run(&mut world);

    let after_pos_change = world.resource::<ChangedBoxStyleCount>().0;
    assert_eq!(
        initial_count, after_pos_change,
        "位置のみ変更で Changed<BoxStyle> が発火してはいけない"
    );
}

/// サイズ変更時に Changed<BoxStyle> が発火することを検証
#[test]
fn test_changed_boxstyle_fired_on_size_change() {
    let mut world = World::new();
    world.insert_resource(FrameCount(0));

    let entity = world
        .spawn((
            Window::default(),
            WindowPos {
                position: Some(POINT { x: 100, y: 200 }),
                size: Some(SIZE { cx: 800, cy: 600 }),
                ..Default::default()
            },
            DPI::default(),
            BoxStyle {
                size: Some(BoxSize {
                    width: Some(Dimension::Px(800.0)),
                    height: Some(Dimension::Px(600.0)),
                }),
                ..Default::default()
            },
            Arrangement::default(),
            GlobalArrangement::default(),
        ))
        .id();

    #[derive(Resource, Default)]
    struct ChangedBoxStyleCount2(u32);

    fn detect_boxstyle_change2(
        query: Query<Entity, Changed<BoxStyle>>,
        mut changed_count: ResMut<ChangedBoxStyleCount2>,
    ) {
        for _entity in query.iter() {
            changed_count.0 += 1;
        }
    }

    world.insert_resource(ChangedBoxStyleCount2::default());

    let mut schedule = Schedule::default();
    schedule.add_systems(detect_boxstyle_change2);

    // 初回実行（Added → Changed 発火）
    schedule.run(&mut world);
    let count_after_first = world.resource::<ChangedBoxStyleCount2>().0;
    assert_eq!(count_after_first, 1, "初回は Added で発火");

    // 2回目: 変更なし → 発火しない
    schedule.run(&mut world);
    let count_after_second = world.resource::<ChangedBoxStyleCount2>().0;
    assert_eq!(count_after_second, 1, "変更なしで発火してはいけない");

    // BoxStyle.size を変更 → Changed<BoxStyle> 発火
    world.get_mut::<BoxStyle>(entity).unwrap().size = Some(BoxSize {
        width: Some(Dimension::Px(600.0)),
        height: Some(Dimension::Px(400.0)),
    });

    schedule.run(&mut world);
    let count_after_size = world.resource::<ChangedBoxStyleCount2>().0;
    assert_eq!(
        count_after_size, 2,
        "サイズ変更後に Changed<BoxStyle> が発火すること"
    );
}

// =============================================================================
// Task 7.3: ドラッグ終了同期テスト
// =============================================================================

/// ドラッグ終了後に WindowPos に set_changed() が呼ばれることを検証
#[test]
fn test_drag_end_syncs_window_pos_changed() {
    let mut world = World::new();
    world.insert_resource(FrameCount(0));
    world.insert_resource(DragAccumulatorResource::new());
    world.insert_resource(WindowDragContextResource::new());
    world.init_resource::<bevy_ecs::message::Messages<DragStartEvent>>();
    world.init_resource::<bevy_ecs::message::Messages<DragEvent>>();
    world.init_resource::<bevy_ecs::message::Messages<DragEndEvent>>();

    // Window entity（WindowDragging マーカー付き = ドラッグ中を模擬）
    let window_entity = world
        .spawn((
            Window::default(),
            WindowPos {
                position: Some(POINT { x: 300, y: 400 }),
                size: Some(SIZE { cx: 800, cy: 600 }),
                ..Default::default()
            },
            DPI::default(),
            Arrangement {
                offset: Offset { x: 300.0, y: 400.0 },
                scale: LayoutScale { x: 1.0, y: 1.0 },
                size: Size {
                    width: 800.0,
                    height: 600.0,
                },
            },
            GlobalArrangement::default(),
            WindowDragging, // ドラッグ中状態
        ))
        .id();

    // ドラッグ対象エンティティ（Window の子）
    let drag_entity = world
        .spawn((
            DragConfig::default(),
            bevy_ecs::hierarchy::ChildOf(window_entity),
        ))
        .id();

    // Changed<WindowPos> 検知用
    #[derive(Resource, Default)]
    struct WindowPosChangedCount(u32);

    fn detect_window_pos_change(
        query: Query<Entity, (With<Window>, Changed<WindowPos>)>,
        mut count: ResMut<WindowPosChangedCount>,
    ) {
        for _e in query.iter() {
            count.0 += 1;
        }
    }

    world.insert_resource(WindowPosChangedCount::default());

    let mut schedule = Schedule::default();
    schedule.add_systems(detect_window_pos_change);

    // 初回: Added で発火
    schedule.run(&mut world);
    let initial = world.resource::<WindowPosChangedCount>().0;

    // Ended 遷移を設定
    world
        .resource::<DragAccumulatorResource>()
        .set_transition(DragTransition::Ended {
            entity: drag_entity,
            end_pos: PhysicalPoint::new(350, 450),
            cancelled: false,
        });

    // dispatch_drag_events を実行（ドラッグ終了処理）
    dispatch_drag_events(&mut world);

    // Changed<WindowPos> 検知を実行
    schedule.run(&mut world);
    let after_end = world.resource::<WindowPosChangedCount>().0;

    assert!(
        after_end > initial,
        "ドラッグ終了後に Changed<WindowPos> が発火すること"
    );
}

/// ドラッグ終了後に WindowDragContextResource がクリアされることを検証
#[test]
fn test_drag_end_clears_context_resource() {
    let mut world = World::new();
    world.insert_resource(FrameCount(0));
    world.insert_resource(DragAccumulatorResource::new());
    world.insert_resource(WindowDragContextResource::new());
    world.init_resource::<bevy_ecs::message::Messages<DragStartEvent>>();
    world.init_resource::<bevy_ecs::message::Messages<DragEvent>>();
    world.init_resource::<bevy_ecs::message::Messages<DragEndEvent>>();

    let window_entity = world
        .spawn((
            Window::default(),
            WindowPos {
                position: Some(POINT { x: 100, y: 200 }),
                ..Default::default()
            },
            DPI::default(),
            Arrangement::default(),
            GlobalArrangement::default(),
            WindowDragging,
        ))
        .id();

    let drag_entity = world
        .spawn((
            DragConfig::default(),
            bevy_ecs::hierarchy::ChildOf(window_entity),
        ))
        .id();

    // コンテキストをセットしておく
    world
        .resource::<WindowDragContextResource>()
        .set(wintf::ecs::drag::WindowDragContext {
            hwnd: None,
            initial_window_pos: Some(POINT { x: 100, y: 200 }),
            move_window: true,
            constraint: None,
        });

    // Ended を設定
    world
        .resource::<DragAccumulatorResource>()
        .set_transition(DragTransition::Ended {
            entity: drag_entity,
            end_pos: PhysicalPoint::new(200, 300),
            cancelled: false,
        });

    dispatch_drag_events(&mut world);

    // コンテキストがクリアされたことを確認
    let ctx = world.resource::<WindowDragContextResource>().get();
    if let Some(ctx) = ctx {
        assert!(
            ctx.hwnd.is_none() && ctx.initial_window_pos.is_none(),
            "ドラッグ終了後にコンテキストがクリアされるべき"
        );
    }
    // get() が None を返すか、中身が空であればOK
}

// =============================================================================
// Task 7.4: WindowDragging ライフサイクルテスト
// =============================================================================

/// ドラッグ開始で WindowDragging がWindow entityに挿入されることを検証
#[test]
fn test_window_dragging_inserted_on_drag_start() {
    let mut world = World::new();
    world.insert_resource(FrameCount(0));
    world.insert_resource(DragAccumulatorResource::new());
    world.insert_resource(WindowDragContextResource::new());
    world.init_resource::<bevy_ecs::message::Messages<DragStartEvent>>();
    world.init_resource::<bevy_ecs::message::Messages<DragEvent>>();
    world.init_resource::<bevy_ecs::message::Messages<DragEndEvent>>();

    let window_entity = world
        .spawn((
            Window::default(),
            WindowPos {
                position: Some(POINT { x: 100, y: 200 }),
                size: Some(SIZE { cx: 800, cy: 600 }),
                ..Default::default()
            },
            DPI::default(),
            Arrangement::default(),
            GlobalArrangement::default(),
        ))
        .id();

    let drag_entity = world
        .spawn((
            DragConfig {
                move_window: true,
                ..Default::default()
            },
            bevy_ecs::hierarchy::ChildOf(window_entity),
        ))
        .id();

    // ドラッグ開始前: WindowDragging なし
    assert!(
        world.get::<WindowDragging>(window_entity).is_none(),
        "ドラッグ開始前は WindowDragging がないこと"
    );

    // Started 遷移を設定
    world
        .resource::<DragAccumulatorResource>()
        .set_transition(DragTransition::Started {
            entity: drag_entity,
            start_pos: PhysicalPoint::new(150, 250),
            timestamp: Instant::now(),
        });

    dispatch_drag_events(&mut world);

    // ドラッグ開始後: WindowDragging が挿入されていること
    assert!(
        world.get::<WindowDragging>(window_entity).is_some(),
        "ドラッグ開始後は WindowDragging がWindow entityに存在すること"
    );
}

/// ドラッグ終了で WindowDragging がWindow entityから除去されることを検証
#[test]
fn test_window_dragging_removed_on_drag_end() {
    let mut world = World::new();
    world.insert_resource(FrameCount(0));
    world.insert_resource(DragAccumulatorResource::new());
    world.insert_resource(WindowDragContextResource::new());
    world.init_resource::<bevy_ecs::message::Messages<DragStartEvent>>();
    world.init_resource::<bevy_ecs::message::Messages<DragEvent>>();
    world.init_resource::<bevy_ecs::message::Messages<DragEndEvent>>();

    // Window entity（ドラッグ中状態）
    let window_entity = world
        .spawn((
            Window::default(),
            WindowPos {
                position: Some(POINT { x: 300, y: 400 }),
                size: Some(SIZE { cx: 800, cy: 600 }),
                ..Default::default()
            },
            DPI::default(),
            Arrangement::default(),
            GlobalArrangement::default(),
            WindowDragging, // 事前に挿入
        ))
        .id();

    let drag_entity = world
        .spawn((
            DragConfig::default(),
            bevy_ecs::hierarchy::ChildOf(window_entity),
        ))
        .id();

    // ドラッグ中: WindowDragging あり
    assert!(
        world.get::<WindowDragging>(window_entity).is_some(),
        "ドラッグ中は WindowDragging が存在すること"
    );

    // Ended 遷移を設定
    world
        .resource::<DragAccumulatorResource>()
        .set_transition(DragTransition::Ended {
            entity: drag_entity,
            end_pos: PhysicalPoint::new(350, 450),
            cancelled: false,
        });

    dispatch_drag_events(&mut world);

    // ドラッグ終了後: WindowDragging が除去されていること
    assert!(
        world.get::<WindowDragging>(window_entity).is_none(),
        "ドラッグ終了後は WindowDragging が除去されること"
    );
}

/// ドラッグ全ライフサイクル（Started → Ended）を通じた WindowDragging の挿入/除去
#[test]
fn test_window_dragging_full_lifecycle() {
    let mut world = World::new();
    world.insert_resource(FrameCount(0));
    world.insert_resource(DragAccumulatorResource::new());
    world.insert_resource(WindowDragContextResource::new());
    world.init_resource::<bevy_ecs::message::Messages<DragStartEvent>>();
    world.init_resource::<bevy_ecs::message::Messages<DragEvent>>();
    world.init_resource::<bevy_ecs::message::Messages<DragEndEvent>>();

    let window_entity = world
        .spawn((
            Window::default(),
            WindowPos {
                position: Some(POINT { x: 100, y: 200 }),
                size: Some(SIZE { cx: 800, cy: 600 }),
                ..Default::default()
            },
            DPI::default(),
            Arrangement::default(),
            GlobalArrangement::default(),
        ))
        .id();

    let drag_entity = world
        .spawn((
            DragConfig {
                move_window: true,
                ..Default::default()
            },
            bevy_ecs::hierarchy::ChildOf(window_entity),
        ))
        .id();

    // Step 1: 開始前 → なし
    assert!(world.get::<WindowDragging>(window_entity).is_none());

    // Step 2: Started → あり
    world
        .resource::<DragAccumulatorResource>()
        .set_transition(DragTransition::Started {
            entity: drag_entity,
            start_pos: PhysicalPoint::new(150, 250),
            timestamp: Instant::now(),
        });
    dispatch_drag_events(&mut world);
    assert!(
        world.get::<WindowDragging>(window_entity).is_some(),
        "Started後: WindowDragging が存在すること"
    );

    // Step 3: Ended → なし
    world
        .resource::<DragAccumulatorResource>()
        .set_transition(DragTransition::Ended {
            entity: drag_entity,
            end_pos: PhysicalPoint::new(200, 300),
            cancelled: false,
        });
    dispatch_drag_events(&mut world);
    assert!(
        world.get::<WindowDragging>(window_entity).is_none(),
        "Ended後: WindowDragging が除去されること"
    );
}

// =============================================================================
// Task 7.5: update_arrangements Window offset スキップテスト
// =============================================================================

/// Window entity の Arrangement.offset が taffy の layout.location で上書きされないことを検証
#[test]
fn test_update_arrangements_skips_window_offset() {
    let mut world = World::new();

    // Window entity: taffy layout location = (50, 60) だが offset は (100, 200) を維持するはず
    let layout = taffy::Layout {
        location: taffy::Point { x: 50.0, y: 60.0 },
        size: taffy::Size {
            width: 800.0,
            height: 600.0,
        },
        ..Default::default()
    };

    let window_entity = world
        .spawn((
            Window::default(),
            TaffyStyle::default(),
            TaffyComputedLayout::from(layout),
            Arrangement {
                offset: Offset { x: 100.0, y: 200.0 },
                scale: LayoutScale::default(),
                size: Size {
                    width: 800.0,
                    height: 600.0,
                },
            },
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(update_arrangements_system);
    schedule.run(&mut world);

    let arr = world.get::<Arrangement>(window_entity).unwrap();
    // Window entity の offset は taffy の location (50, 60) ではなく、元の (100, 200) を維持
    assert_eq!(
        arr.offset.x, 100.0,
        "Window entity の offset.x は taffy location で上書きされないこと"
    );
    assert_eq!(
        arr.offset.y, 200.0,
        "Window entity の offset.y は taffy location で上書きされないこと"
    );
    // サイズは taffy 結果で更新される
    assert_eq!(arr.size.width, 800.0);
    assert_eq!(arr.size.height, 600.0);
}

/// 非 Window entity の Arrangement.offset は taffy layout.location で正しく更新されることを検証
#[test]
fn test_update_arrangements_applies_offset_for_non_window() {
    let mut world = World::new();

    let layout = taffy::Layout {
        location: taffy::Point { x: 50.0, y: 60.0 },
        size: taffy::Size {
            width: 200.0,
            height: 150.0,
        },
        ..Default::default()
    };

    let entity = world
        .spawn((TaffyStyle::default(), TaffyComputedLayout::from(layout)))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(update_arrangements_system);
    schedule.run(&mut world);

    let arr = world.get::<Arrangement>(entity).unwrap();
    // 非 Window entity は taffy location がそのまま offset に設定される
    assert_eq!(
        arr.offset.x, 50.0,
        "非 Window entity の offset.x は taffy location の値であるべき"
    );
    assert_eq!(
        arr.offset.y, 60.0,
        "非 Window entity の offset.y は taffy location の値であるべき"
    );
}

/// Window entity で Arrangement が未作成の場合、offset は (0, 0) で作成されることを検証
#[test]
fn test_update_arrangements_window_without_existing_arrangement() {
    let mut world = World::new();

    let layout = taffy::Layout {
        location: taffy::Point { x: 50.0, y: 60.0 },
        size: taffy::Size {
            width: 800.0,
            height: 600.0,
        },
        ..Default::default()
    };

    let window_entity = world
        .spawn((
            Window::default(),
            TaffyStyle::default(),
            TaffyComputedLayout::from(layout),
            // Arrangement なし
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems(update_arrangements_system);
    schedule.run(&mut world);

    let arr = world.get::<Arrangement>(window_entity).unwrap();
    // Arrangement が新規作成される場合、offset は (0, 0)（taffy の 50, 60 ではない）
    assert_eq!(
        arr.offset.x, 0.0,
        "新規 Window Arrangement の offset.x は 0.0 であるべき"
    );
    assert_eq!(
        arr.offset.y, 0.0,
        "新規 Window Arrangement の offset.y は 0.0 であるべき"
    );
    // サイズは設定される
    assert_eq!(arr.size.width, 800.0);
    assert_eq!(arr.size.height, 600.0);
}
