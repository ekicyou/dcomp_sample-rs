/// Layout-to-Graphics同期システムのテスト
/// 
/// このテストは、レイアウト計算結果が正しくグラフィックスコンポーネントに
/// 伝播することを検証します。

use bevy_ecs::prelude::*;
use bevy_ecs::system::IntoSystem;
use wintf::ecs::*;
use windows::Win32::Foundation::{POINT, SIZE};

#[test]
fn test_sync_visual_from_layout_root() {
    // WorldとScheduleを準備
    let mut world = World::new();
    
    // LayoutRootとGlobalArrangement、Visualを持つエンティティを作成
    let entity = world.spawn((
        layout::LayoutRoot,
        layout::GlobalArrangement {
            transform: windows_numerics::Matrix3x2::identity(),
            bounds: layout::D2DRect {
                left: 0.0,
                top: 0.0,
                right: 800.0,
                bottom: 600.0,
            },
        },
        Visual::default(),
    )).id();

    // sync_visual_from_layout_rootシステムを実行
    let mut system = IntoSystem::into_system(wintf::ecs::sync_visual_from_layout_root);
    system.initialize(&mut world);
    system.run((), &mut world);
    system.apply_deferred(&mut world);

    // Visualのサイズが更新されていることを確認
    let visual = world.entity(entity).get::<Visual>().unwrap();
    assert_eq!(visual.size.X, 800.0);
    assert_eq!(visual.size.Y, 600.0);
}

#[test]
fn test_sync_window_pos() {
    // WorldとScheduleを準備
    let mut world = World::new();
    
    // Window、LayoutRoot、GlobalArrangement、Visual、WindowPosを持つエンティティを作成
    let entity = world.spawn((
        Window::default(),
        layout::LayoutRoot,
        layout::GlobalArrangement {
            transform: windows_numerics::Matrix3x2::identity(),
            bounds: layout::D2DRect {
                left: 100.0,
                top: 50.0,
                right: 900.0,
                bottom: 650.0,
            },
        },
        Visual {
            size: windows_numerics::Vector2 { X: 800.0, Y: 600.0 },
            ..Default::default()
        },
        WindowPos::default(),
    )).id();

    // sync_window_posシステムを実行
    let mut system = IntoSystem::into_system(wintf::ecs::sync_window_pos);
    system.initialize(&mut world);
    system.run((), &mut world);
    system.apply_deferred(&mut world);

    // WindowPosが更新されていることを確認
    let window_pos = world.entity(entity).get::<WindowPos>().unwrap();
    assert_eq!(window_pos.position, Some(POINT { x: 100, y: 50 }));
    assert_eq!(window_pos.size, Some(SIZE { cx: 800, cy: 600 }));
}

#[test]
fn test_echo_detection() {
    let mut window_pos = WindowPos::default();
    
    // last_sentを設定
    window_pos.last_sent_position = Some((100, 50));
    window_pos.last_sent_size = Some((800, 600));
    
    // 同じ値を受信した場合、エコーバックと判定されるべき
    assert!(window_pos.is_echo(
        POINT { x: 100, y: 50 },
        SIZE { cx: 800, cy: 600 }
    ));
    
    // 異なる値を受信した場合、エコーバックではない
    assert!(!window_pos.is_echo(
        POINT { x: 200, y: 100 },
        SIZE { cx: 1024, cy: 768 }
    ));
}

#[test]
fn test_skip_invalid_bounds() {
    // WorldとScheduleを準備
    let mut world = World::new();
    
    // 無効なbounds (0,0,0,0)を持つエンティティを作成
    let entity = world.spawn((
        Window::default(),
        layout::LayoutRoot,
        layout::GlobalArrangement {
            transform: windows_numerics::Matrix3x2::identity(),
            bounds: layout::D2DRect {
                left: 0.0,
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
            },
        },
        Visual::default(),
        WindowPos {
            position: Some(POINT { x: 100, y: 100 }),
            size: Some(SIZE { cx: 800, cy: 600 }),
            ..Default::default()
        },
    )).id();

    // sync_window_posシステムを実行
    let mut system = IntoSystem::into_system(wintf::ecs::sync_window_pos);
    system.initialize(&mut world);
    system.run((), &mut world);
    system.apply_deferred(&mut world);

    // 無効なboundsの場合、WindowPosは更新されないべき
    let window_pos = world.entity(entity).get::<WindowPos>().unwrap();
    assert_eq!(window_pos.position, Some(POINT { x: 100, y: 100 }));
    assert_eq!(window_pos.size, Some(SIZE { cx: 800, cy: 600 }));
}
