/// Layout-to-Graphics同期システムのテスト
///
/// このテストは、レイアウト計算結果が正しくグラフィックスコンポーネントに
/// 伝播することを検証します。
use bevy_ecs::prelude::*;
use bevy_ecs::system::IntoSystem;
use windows::Win32::Foundation::{POINT, SIZE};
use wintf::ecs::*;

// sync_visual_from_layout_rootは廃止され、Arrangementから直接Surfaceサイズを取得するようになりました。
// Visual.sizeは削除され、Arrangement.sizeがSingle Source of Truthです。

#[test]
fn test_sync_window_pos() {
    use bevy_ecs::schedule::Schedule;

    // WorldとScheduleを準備
    let mut world = World::new();
    world.insert_resource(wintf::ecs::world::FrameCount(1)); // window_pos_sync_systemに必要
    let mut schedule = Schedule::default();
    schedule.add_systems(wintf::ecs::window_pos_sync_system);

    // Window、Visual、WindowPosを持つエンティティを作成
    // Note: Visual::on_addがArrangement::default()を自動挿入し、
    //       Arrangement::on_addがGlobalArrangement::default()を自動挿入する
    let entity = world
        .spawn((Window::default(), Visual::default(), WindowPos::default()))
        .id();

    // Commands適用のためにflush
    world.flush();

    // 自動挿入されたArrangementとGlobalArrangementを上書き
    world.entity_mut(entity).insert((
        layout::Arrangement {
            offset: layout::Offset { x: 100.0, y: 50.0 },
            scale: layout::LayoutScale::default(),
            size: layout::Size {
                width: 800.0,
                height: 600.0,
            },
        },
        layout::GlobalArrangement {
            transform: windows_numerics::Matrix3x2::identity(),
            bounds: layout::D2DRect {
                left: 100.0,
                top: 50.0,
                right: 900.0,
                bottom: 650.0,
            },
        },
    ));

    // スケジュールを実行（GlobalArrangementがChangedとして検出される）
    schedule.run(&mut world);

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
    assert!(window_pos.is_echo(POINT { x: 100, y: 50 }, SIZE { cx: 800, cy: 600 }));

    // 異なる値を受信した場合、エコーバックではない
    assert!(!window_pos.is_echo(POINT { x: 200, y: 100 }, SIZE { cx: 1024, cy: 768 }));
}

#[test]
fn test_skip_invalid_bounds() {
    // WorldとScheduleを準備
    let mut world = World::new();

    // 無効なbounds (0,0,0,0)を持つエンティティを作成
    let entity = world
        .spawn((
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
        ))
        .id();

    // window_pos_sync_systemシステムを実行
    let mut system = IntoSystem::into_system(wintf::ecs::window_pos_sync_system);
    system.initialize(&mut world);
    let _ = system.run((), &mut world);
    system.apply_deferred(&mut world);

    // 無効なboundsの場合、WindowPosは更新されないべき
    let window_pos = world.entity(entity).get::<WindowPos>().unwrap();
    assert_eq!(window_pos.position, Some(POINT { x: 100, y: 100 }));
    assert_eq!(window_pos.size, Some(SIZE { cx: 800, cy: 600 }));
}

#[test]
fn test_echo_back_flow() {
    use bevy_ecs::schedule::Schedule;

    // WorldとScheduleを準備
    let mut world = World::new();
    world.insert_resource(wintf::ecs::world::FrameCount(1)); // window_pos_sync_systemに必要
    let mut schedule = Schedule::default();
    schedule.add_systems(wintf::ecs::window_pos_sync_system);

    // Window、Visual、WindowPosを持つエンティティを作成
    // Note: Visual::on_addがArrangement::default()を自動挿入し、
    //       Arrangement::on_addがGlobalArrangement::default()を自動挿入する
    let entity = world
        .spawn((Window::default(), Visual::default(), WindowPos::default()))
        .id();

    // Commands適用のためにflush
    world.flush();

    // 自動挿入されたArrangementとGlobalArrangementを上書き
    world.entity_mut(entity).insert((
        layout::Arrangement {
            offset: layout::Offset { x: 100.0, y: 50.0 },
            scale: layout::LayoutScale::default(),
            size: layout::Size {
                width: 800.0,
                height: 600.0,
            },
        },
        layout::GlobalArrangement {
            transform: windows_numerics::Matrix3x2::identity(),
            bounds: layout::D2DRect {
                left: 100.0,
                top: 50.0,
                right: 900.0,
                bottom: 650.0,
            },
        },
    ));

    // スケジュールを実行してWindowPosを更新
    schedule.run(&mut world);

    // WindowPosが更新されたことを確認
    let window_pos = world.entity(entity).get::<WindowPos>().unwrap();
    assert_eq!(window_pos.position, Some(POINT { x: 100, y: 50 }));
    assert_eq!(window_pos.size, Some(SIZE { cx: 800, cy: 600 }));

    // apply_window_pos_changesをシミュレート（last_sentを記録）
    {
        let mut entity_mut = world.entity_mut(entity);
        let mut window_pos = entity_mut.get_mut::<WindowPos>().unwrap();
        let bypass = window_pos.bypass_change_detection();
        bypass.last_sent_position = Some((100, 50));
        bypass.last_sent_size = Some((800, 600));
    }

    // エコーバック：同じ値を受信した場合、is_echoがtrueを返すべき
    let window_pos = world.entity(entity).get::<WindowPos>().unwrap();
    assert!(window_pos.is_echo(POINT { x: 100, y: 50 }, SIZE { cx: 800, cy: 600 }));

    // 外部変更：異なる値を受信した場合、is_echoがfalseを返すべき
    assert!(!window_pos.is_echo(POINT { x: 150, y: 100 }, SIZE { cx: 1024, cy: 768 }));
}

#[test]
fn test_reverse_flow_simulation() {
    // WorldとScheduleを準備
    let mut world = World::new();

    // Window、WindowPosを持つエンティティを作成
    let entity = world
        .spawn((
            Window::default(),
            WindowPos {
                position: Some(POINT { x: 100, y: 50 }),
                size: Some(SIZE { cx: 800, cy: 600 }),
                last_sent_position: Some((100, 50)),
                last_sent_size: Some((800, 600)),
                ..Default::default()
            },
        ))
        .id();

    // ユーザーがウィンドウをリサイズ（WM_WINDOWPOSCHANGEDシミュレーション）
    // 新しい値（外部変更）を受信
    let new_position = POINT { x: 150, y: 100 };
    let new_size = SIZE { cx: 1024, cy: 768 };

    {
        let window_pos = world.entity(entity).get::<WindowPos>().unwrap();

        // エコーバックではないことを確認
        assert!(!window_pos.is_echo(new_position, new_size));

        // 外部変更時の処理：WindowPosを更新（bypass_change_detectionで）
        let mut entity_mut = world.entity_mut(entity);
        let mut window_pos = entity_mut.get_mut::<WindowPos>().unwrap();
        let bypass = window_pos.bypass_change_detection();
        bypass.position = Some(new_position);
        bypass.size = Some(new_size);
        // last_sentをクリア
        bypass.last_sent_position = None;
        bypass.last_sent_size = None;
    }

    // WindowPosが更新されたことを確認
    let window_pos = world.entity(entity).get::<WindowPos>().unwrap();
    assert_eq!(window_pos.position, Some(new_position));
    assert_eq!(window_pos.size, Some(new_size));
    assert_eq!(window_pos.last_sent_position, None);
    assert_eq!(window_pos.last_sent_size, None);
}

#[test]
fn test_visual_partial_eq_optimization() {
    // PartialEqが正しく実装されているかを確認
    let visual1 = Visual {
        opacity: 0.5,
        ..Default::default()
    };

    let visual2 = Visual {
        opacity: 0.5,
        ..Default::default()
    };

    let visual3 = Visual {
        opacity: 0.8,
        ..Default::default()
    };

    // 同じ値を持つVisualは等しいべき
    assert_eq!(
        visual1, visual2,
        "同じ値を持つVisualはPartialEqでtrueを返すべき"
    );

    // 異なる値を持つVisualは等しくないべき
    assert_ne!(
        visual1, visual3,
        "異なる値を持つVisualはPartialEqでfalseを返すべき"
    );

    // PartialEqは変更検知最適化の基礎となる
    // Bevy ECSはMut<T>のDrop時にPartialEqで比較し、
    // 値が変わっていなければChangedフラグを立てない仕組みを提供する
}
