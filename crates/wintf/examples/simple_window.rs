#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Instant;
use windows::core::*;
use windows::Win32::Foundation::{POINT, SIZE};
use wintf::ecs::{Window, WindowHandle, WindowPos};
use wintf::*;

/// テスト用: 一定時間後にウィンドウを自動的に閉じるシステム
fn auto_close_window_system(world: &mut bevy_ecs::world::World) {
    use bevy_ecs::prelude::*;

    // 初回実行時に開始時刻を記録
    if !world.contains_resource::<AutoCloseTimer>() {
        println!("[Test] Auto-close timer started. Will close first window after 5 seconds.");
        world.insert_resource(AutoCloseTimer {
            start: Instant::now(),
            target_closed: false,
        });
    }

    // タイマーの状態を先にチェック
    let should_close = {
        let timer = world.resource::<AutoCloseTimer>();
        !timer.target_closed && timer.start.elapsed().as_secs() >= 5
    };

    // 5秒経過したら最初のウィンドウを閉じる
    if should_close {
        println!("[Test] 5 seconds elapsed. Closing first window...");

        // WindowHandleを持つ最初のエンティティを取得
        let mut query = world.query::<(Entity, &WindowHandle)>();
        if let Some((entity, handle)) = query.iter(world).next() {
            println!(
                "[Test] Despawning entity {:?} with hwnd {:?}",
                entity, handle.hwnd
            );
            world.despawn(entity);

            // タイマーの状態を更新
            let mut timer = world.resource_mut::<AutoCloseTimer>();
            timer.target_closed = true;
        }
    }
}

#[derive(bevy_ecs::prelude::Resource)]
struct AutoCloseTimer {
    start: Instant,
    target_closed: bool,
}

fn main() -> Result<()> {
    human_panic::setup_panic!();

    let mgr = WinThreadMgr::new()?;
    let world = mgr.world();

    // テスト用システムを登録（Updateスケジュールに追加）
    world
        .borrow_mut()
        .add_systems(wintf::ecs::world::Update, auto_close_window_system);

    // 1つ目のWindowコンポーネントを持つEntityを作成
    world.borrow_mut().world_mut().spawn((
        Window {
            title: "wintf - ECS Window 1 (will close after 5s)".to_string(),
            ..Default::default()
        },
        WindowPos {
            position: Some(POINT { x: 100, y: 100 }),
            size: Some(SIZE { cx: 800, cy: 600 }),
            ..Default::default()
        },
    ));

    // 2つ目のWindowコンポーネントを持つEntityを作成
    world.borrow_mut().world_mut().spawn((
        Window {
            title: "wintf - ECS Window 2".to_string(),
            ..Default::default()
        },
        WindowPos {
            position: Some(POINT { x: 950, y: 150 }),
            size: Some(SIZE { cx: 600, cy: 400 }),
            ..Default::default()
        },
    ));

    println!("[Test] Two windows created. First window will auto-close after 5 seconds.");

    // メッセージループを開始（システムが自動的にウィンドウを作成）
    mgr.run()?;

    Ok(())
}
