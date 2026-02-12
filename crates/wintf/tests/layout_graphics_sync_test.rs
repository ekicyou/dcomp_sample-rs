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
fn test_is_self_initiated_default_false() {
    // TLS フラグの初期値は false であること
    assert!(
        !wintf::ecs::is_self_initiated(),
        "is_self_initiated() の初期値は false であるべき"
    );
}

#[test]
fn test_is_self_initiated_flag_lifecycle() {
    // guarded_set_window_pos 呼び出し外では false
    assert!(!wintf::ecs::is_self_initiated());

    // 注: 実際の guarded_set_window_pos は有効な HWND が必要なため、
    // ここでは TLS フラグの直接操作はテストできない。
    // ラッパー関数は E2E テスト（taffy_flex_demo）で検証する。
    // ここでは初期状態の確認のみ行う。
    assert!(!wintf::ecs::is_self_initiated());
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

    // TLS フラグ方式: echo 時は bypass_change_detection で値を更新する（Changed 非発火）
    // これは WM_WINDOWPOSCHANGED ハンドラが is_self_initiated() == true の場合に行う処理
    {
        let mut entity_mut = world.entity_mut(entity);
        let mut window_pos = entity_mut.get_mut::<WindowPos>().unwrap();
        // echo シミュレーション: bypass で同一値を書き込み（Changed 非発火）
        let bypass = window_pos.bypass_change_detection();
        bypass.position = Some(POINT { x: 100, y: 50 });
        bypass.size = Some(SIZE { cx: 800, cy: 600 });
    }

    // WindowPos が変わっていないことを確認
    let window_pos = world.entity(entity).get::<WindowPos>().unwrap();
    assert_eq!(window_pos.position, Some(POINT { x: 100, y: 50 }));
    assert_eq!(window_pos.size, Some(SIZE { cx: 800, cy: 600 }));
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
                ..Default::default()
            },
        ))
        .id();

    // ユーザーがウィンドウをリサイズ（WM_WINDOWPOSCHANGEDシミュレーション）
    // 新しい値（外部変更）を受信
    // TLS フラグ方式: is_self_initiated() == false の場合、DerefMut で更新
    let new_position = POINT { x: 150, y: 100 };
    let new_size = SIZE { cx: 1024, cy: 768 };

    {
        // 外部変更時の処理：WindowPosを通常代入で更新（Changed 発火）
        let mut entity_mut = world.entity_mut(entity);
        let mut window_pos = entity_mut.get_mut::<WindowPos>().unwrap();
        window_pos.position = Some(new_position);
        window_pos.size = Some(new_size);
    }

    // WindowPosが更新されたことを確認
    let window_pos = world.entity(entity).get::<WindowPos>().unwrap();
    assert_eq!(window_pos.position, Some(new_position));
    assert_eq!(window_pos.size, Some(new_size));
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
