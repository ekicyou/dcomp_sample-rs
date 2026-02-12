/// フィードバックループ収束検証テスト
///
/// R4-AC1: ユーザーがウィンドウをドラッグ移動した場合、
/// WindowPos → Arrangement → GlobalArrangement → WindowPos の更新連鎖が
/// 1フレーム内で収束し、次フレームで同一値の再更新が発生しないこと
///
/// R1-AC1: Changed<WindowPos> フィルタにより WindowPos が変更された Window のみ処理すること
use bevy_ecs::prelude::*;
use windows::Win32::Foundation::{POINT, SIZE};
use wintf::ecs::layout::{
    mark_dirty_arrangement_trees, propagate_global_arrangements, sync_simple_arrangements,
    sync_window_arrangement_from_window_pos, window_pos_sync_system,
};
use wintf::ecs::window::{DPI, Window, WindowPos};
use wintf::ecs::world::FrameCount;
use wintf::ecs::{Arrangement, GlobalArrangement, LayoutScale, Offset, Size};

// =============================================================================
// Task 1.1: Changed<WindowPos> フィルタの検証
// =============================================================================

/// sync_window_arrangement_from_window_pos が Changed<WindowPos> フィルタを持ち、
/// WindowPos が変更されたエンティティのみ処理することを検証する
#[test]
fn test_sync_window_arrangement_only_processes_changed_window_pos() {
    let mut world = World::new();
    world.insert_resource(FrameCount(0));

    // DPI 96 (scale=1.0) でウィンドウエンティティを作成
    let entity = world
        .spawn((
            Window::default(),
            WindowPos {
                position: Some(POINT { x: 100, y: 200 }),
                size: Some(SIZE { cx: 800, cy: 600 }),
                ..Default::default()
            },
            DPI::default(), // 96 DPI → scale=1.0
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

    // PostLayout スケジュールを構築（chain で直列実行）
    let mut schedule = Schedule::default();
    schedule.add_systems((
        sync_window_arrangement_from_window_pos,
        sync_simple_arrangements.after(sync_window_arrangement_from_window_pos),
        mark_dirty_arrangement_trees.after(sync_simple_arrangements),
        propagate_global_arrangements.after(mark_dirty_arrangement_trees),
        window_pos_sync_system.after(propagate_global_arrangements),
    ));

    // --- 初回実行: WindowPos は "新規追加" なので Changed 扱い ---
    schedule.run(&mut world);

    // Arrangement.offset が WindowPos.position / scale = (100, 200) / 1.0 に更新されていること
    let arrangement = world.get::<Arrangement>(entity).unwrap();
    assert_eq!(
        arrangement.offset.x, 100.0,
        "初回: offset.x should be 100.0"
    );
    assert_eq!(
        arrangement.offset.y, 200.0,
        "初回: offset.y should be 200.0"
    );

    // --- 2回目実行: WindowPos 未変更 → Changed<WindowPos> なし → スキップ ---
    let arrangement_before = *world.get::<Arrangement>(entity).unwrap();
    schedule.run(&mut world);
    let arrangement_after = *world.get::<Arrangement>(entity).unwrap();

    assert_eq!(
        arrangement_before, arrangement_after,
        "2回目: WindowPos未変更のためArrangementは変化しないこと"
    );

    // --- 3回目: WindowPos を変更 → Changed<WindowPos> 発火 → 処理される ---
    {
        let mut wp = world.get_mut::<WindowPos>(entity).unwrap();
        wp.position = Some(POINT { x: 300, y: 400 });
    }
    schedule.run(&mut world);

    let arrangement = world.get::<Arrangement>(entity).unwrap();
    assert_eq!(
        arrangement.offset.x, 300.0,
        "3回目: offset.x should be 300.0"
    );
    assert_eq!(
        arrangement.offset.y, 400.0,
        "3回目: offset.y should be 400.0"
    );
}

// =============================================================================
// Task 1.1 + R2: DPI 変換の正確性検証
// =============================================================================

/// DPI 192 (200%) 環境で WindowPos.position がそのまま Arrangement.offset に設定されること
/// Window は LayoutRoot (scale=1.0) の子なので offset = position（DPI除算なし）
#[test]
fn test_sync_window_arrangement_dpi_192_no_division() {
    let mut world = World::new();
    world.insert_resource(FrameCount(0));

    let entity = world
        .spawn((
            Window::default(),
            WindowPos {
                position: Some(POINT { x: 200, y: 400 }),
                size: Some(SIZE { cx: 1600, cy: 1200 }),
                ..Default::default()
            },
            DPI {
                dpi_x: 192,
                dpi_y: 192,
            }, // scale = 2.0
            Arrangement {
                offset: Offset { x: 0.0, y: 0.0 },
                scale: LayoutScale { x: 2.0, y: 2.0 },
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

    schedule.run(&mut world);

    let arrangement = world.get::<Arrangement>(entity).unwrap();
    assert_eq!(
        arrangement.offset.x, 200.0,
        "DPI 192: offset.x = 200.0 (物理px、レイアウト入力と同じ)"
    );
    assert_eq!(
        arrangement.offset.y, 400.0,
        "DPI 192: offset.y = 400.0 (物理px、レイアウト入力と同じ)"
    );
}

/// DPI 96 (100%) 環境で WindowPos.position がそのまま Arrangement.offset に設定されること
#[test]
fn test_sync_window_arrangement_dpi_96_identity() {
    let mut world = World::new();
    world.insert_resource(FrameCount(0));

    let entity = world
        .spawn((
            Window::default(),
            WindowPos {
                position: Some(POINT { x: 150, y: 250 }),
                size: Some(SIZE { cx: 800, cy: 600 }),
                ..Default::default()
            },
            DPI::default(), // 96 DPI → scale=1.0
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

    let mut schedule = Schedule::default();
    schedule.add_systems(sync_window_arrangement_from_window_pos);

    schedule.run(&mut world);

    let arrangement = world.get::<Arrangement>(entity).unwrap();
    assert_eq!(
        arrangement.offset.x, 150.0,
        "DPI 96: offset.x = 150.0 (物理px)"
    );
    assert_eq!(
        arrangement.offset.y, 250.0,
        "DPI 96: offset.y = 250.0 (物理px)"
    );
}

// =============================================================================
// Task 1.1 + R3: エッジケース検証
// =============================================================================

/// WindowPos.position が None の場合、Arrangement を更新しないこと
#[test]
fn test_sync_window_arrangement_skips_none_position() {
    let mut world = World::new();
    world.insert_resource(FrameCount(0));

    let entity = world
        .spawn((
            Window::default(),
            WindowPos {
                position: None,
                size: Some(SIZE { cx: 800, cy: 600 }),
                ..Default::default()
            },
            DPI::default(),
            Arrangement {
                offset: Offset { x: 50.0, y: 60.0 },
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

    schedule.run(&mut world);

    let arrangement = world.get::<Arrangement>(entity).unwrap();
    assert_eq!(
        arrangement.offset.x, 50.0,
        "None position: offset unchanged"
    );
    assert_eq!(
        arrangement.offset.y, 60.0,
        "None position: offset unchanged"
    );
}

/// WindowPos.position が CW_USEDEFAULT の場合、Arrangement を更新しないこと
#[test]
fn test_sync_window_arrangement_skips_cw_usedefault() {
    use windows::Win32::UI::WindowsAndMessaging::CW_USEDEFAULT;

    let mut world = World::new();
    world.insert_resource(FrameCount(0));

    let entity = world
        .spawn((
            Window::default(),
            WindowPos {
                position: Some(POINT {
                    x: CW_USEDEFAULT,
                    y: CW_USEDEFAULT,
                }),
                size: Some(SIZE { cx: 800, cy: 600 }),
                ..Default::default()
            },
            DPI::default(),
            Arrangement {
                offset: Offset { x: 50.0, y: 60.0 },
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

    schedule.run(&mut world);

    let arrangement = world.get::<Arrangement>(entity).unwrap();
    assert_eq!(
        arrangement.offset.x, 50.0,
        "CW_USEDEFAULT: offset unchanged"
    );
    assert_eq!(
        arrangement.offset.y, 60.0,
        "CW_USEDEFAULT: offset unchanged"
    );
}

/// 等値チェック: Arrangement.offset が変換後の値と同一の場合、更新しないこと
#[test]
fn test_sync_window_arrangement_skips_equal_offset() {
    let mut world = World::new();
    world.insert_resource(FrameCount(0));

    // 事前にオフセットが既に正しい値に設定されている
    let entity = world
        .spawn((
            Window::default(),
            WindowPos {
                position: Some(POINT { x: 100, y: 200 }),
                size: Some(SIZE { cx: 800, cy: 600 }),
                ..Default::default()
            },
            DPI::default(), // scale=1.0
            Arrangement {
                offset: Offset { x: 100.0, y: 200.0 }, // 既に正しい値
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

    // 初回: Added 扱いだが等値なので Arrangement は DerefMut されない
    schedule.run(&mut world);

    let arrangement = world.get::<Arrangement>(entity).unwrap();
    assert_eq!(arrangement.offset.x, 100.0);
    assert_eq!(arrangement.offset.y, 200.0);
}

// =============================================================================
// Task 3.1: フィードバックループ収束検証（R4-AC1）
// =============================================================================

/// ユーザーがウィンドウを移動した場合の収束テスト
/// WindowPos 更新 → sync_window_arrangement → propagate → window_pos_sync の連鎖で
/// 1フレーム内で収束し、次フレームで再更新が発生しないこと
#[test]
fn test_feedback_loop_converges_in_one_frame_dpi_96() {
    let mut world = World::new();
    world.insert_resource(FrameCount(0));

    let entity = world
        .spawn((
            Window::default(),
            WindowPos {
                position: Some(POINT { x: 0, y: 0 }),
                size: Some(SIZE { cx: 800, cy: 600 }),
                ..Default::default()
            },
            DPI::default(), // 96 DPI → scale=1.0
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

    // PostLayout スケジュール（本番と同じ順序）
    let mut schedule = Schedule::default();
    schedule.add_systems((
        sync_window_arrangement_from_window_pos,
        sync_simple_arrangements.after(sync_window_arrangement_from_window_pos),
        mark_dirty_arrangement_trees.after(sync_simple_arrangements),
        propagate_global_arrangements.after(mark_dirty_arrangement_trees),
        window_pos_sync_system.after(propagate_global_arrangements),
    ));

    // 初回実行で初期化（Added による初期処理）
    schedule.run(&mut world);

    // --- ユーザーがウィンドウをドラッグ: position を変更 ---
    {
        let mut wp = world.get_mut::<WindowPos>(entity).unwrap();
        wp.position = Some(POINT { x: 500, y: 300 });
    }

    // フレーム1: 変更が伝播
    schedule.run(&mut world);

    let arr1 = *world.get::<Arrangement>(entity).unwrap();
    let ga1 = *world.get::<GlobalArrangement>(entity).unwrap();
    let wp1 = *world.get::<WindowPos>(entity).unwrap();

    assert_eq!(arr1.offset.x, 500.0, "Frame1: Arrangement.offset.x = 500.0");
    assert_eq!(arr1.offset.y, 300.0, "Frame1: Arrangement.offset.y = 300.0");

    // フレーム2: 収束確認（変更なし）
    schedule.run(&mut world);

    let arr2 = *world.get::<Arrangement>(entity).unwrap();
    let ga2 = *world.get::<GlobalArrangement>(entity).unwrap();
    let wp2 = *world.get::<WindowPos>(entity).unwrap();

    assert_eq!(
        arr1, arr2,
        "Frame2: Arrangement should not change (converged)"
    );
    assert_eq!(
        ga1, ga2,
        "Frame2: GlobalArrangement should not change (converged)"
    );
    assert_eq!(wp1, wp2, "Frame2: WindowPos should not change (converged)");
}

/// DPI 192 (scale=2.0) でのフィードバックループ収束テスト
/// 実アプリと同じ構造: Window は LayoutRoot (scale=1.0) の子エンティティ
/// ChildOf(LayoutRoot) により propagate_global_arrangements で bounds 計算される
/// bounds.left = parent.bounds.left + offset × parent_scale = offset × 1.0 = offset
#[test]
fn test_feedback_loop_converges_dpi_192() {
    use bevy_ecs::hierarchy::ChildOf;
    use wintf::ecs::layout::LayoutRoot;

    let mut world = World::new();
    world.insert_resource(FrameCount(0));

    // LayoutRoot (scale=1.0) — 仮想デスクトップ原点
    let layout_root = world
        .spawn((
            LayoutRoot,
            Arrangement {
                offset: Offset { x: 0.0, y: 0.0 },
                scale: LayoutScale { x: 1.0, y: 1.0 },
                size: Size {
                    width: 3840.0,
                    height: 2160.0,
                },
            },
            GlobalArrangement::default(),
        ))
        .id();

    // Window entity — LayoutRoot の子
    let entity = world
        .spawn((
            ChildOf(layout_root),
            Window::default(),
            WindowPos {
                position: Some(POINT { x: 0, y: 0 }),
                size: Some(SIZE { cx: 1600, cy: 1200 }),
                ..Default::default()
            },
            DPI {
                dpi_x: 192,
                dpi_y: 192,
            }, // scale = 2.0
            Arrangement {
                offset: Offset { x: 0.0, y: 0.0 },
                scale: LayoutScale { x: 2.0, y: 2.0 },
                size: Size {
                    width: 800.0,
                    height: 600.0,
                },
            },
            GlobalArrangement::default(),
        ))
        .id();

    let mut schedule = Schedule::default();
    schedule.add_systems((
        sync_window_arrangement_from_window_pos,
        sync_simple_arrangements.after(sync_window_arrangement_from_window_pos),
        mark_dirty_arrangement_trees.after(sync_simple_arrangements),
        propagate_global_arrangements.after(mark_dirty_arrangement_trees),
        window_pos_sync_system.after(propagate_global_arrangements),
    ));

    // 初回実行
    schedule.run(&mut world);

    // ユーザーがウィンドウをドラッグ: (200, 400) 物理px
    {
        let mut wp = world.get_mut::<WindowPos>(entity).unwrap();
        wp.position = Some(POINT { x: 200, y: 400 });
    }

    // フレーム1
    schedule.run(&mut world);

    let arr1 = *world.get::<Arrangement>(entity).unwrap();
    assert_eq!(
        arr1.offset.x, 200.0,
        "DPI 192: offset.x = 200.0 (物理px、LayoutRoot子なのでDPI除算なし)"
    );
    assert_eq!(
        arr1.offset.y, 400.0,
        "DPI 192: offset.y = 400.0 (物理px、LayoutRoot子なのでDPI除算なし)"
    );

    // フレーム2: 収束確認
    schedule.run(&mut world);

    let arr2 = *world.get::<Arrangement>(entity).unwrap();
    assert_eq!(
        arr1, arr2,
        "DPI 192: Arrangement should converge in one frame"
    );
}
