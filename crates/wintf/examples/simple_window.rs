#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::time::Instant;
use windows::core::*;
use windows::Win32::Foundation::{POINT, SIZE};
use wintf::ecs::{Window, WindowHandle, WindowPos};
use wintf::*;

#[derive(bevy_ecs::prelude::Resource)]
struct AutoCloseTimer {
    start: Instant,
    last_close_time: Option<Instant>,
}

/// テスト用: 5秒ごとに1つずつウィンドウを閉じるシステム
fn auto_close_window_system(world: &mut bevy_ecs::world::World) {
    use bevy_ecs::prelude::*;

    // 初回実行時に開始時刻を記録
    if !world.contains_resource::<AutoCloseTimer>() {
        println!("[Test] Auto-close timer started. Will close windows every 5 seconds.");
        world.insert_resource(AutoCloseTimer {
            start: Instant::now(),
            last_close_time: None,
        });
    }

    // タイマーの状態を先にチェック
    let should_close = {
        let timer = world.resource::<AutoCloseTimer>();
        let elapsed = timer.start.elapsed().as_secs();

        if let Some(last_close) = timer.last_close_time {
            // 前回閉じてから5秒以上経過
            last_close.elapsed().as_secs() >= 5
        } else {
            // 初回は5秒後
            elapsed >= 5
        }
    };

    // 5秒経過したらウィンドウを1つ閉じる
    if should_close {
        // WindowHandleを持つエンティティの数を確認
        let window_count = {
            let mut query = world.query::<&WindowHandle>();
            query.iter(world).count()
        };

        if window_count > 0 {
            println!("[Test] Closing one window (remaining: {})...", window_count);

            // タイマーの状態を更新
            let mut timer = world.resource_mut::<AutoCloseTimer>();
            timer.last_close_time = Some(Instant::now());

            // WindowHandleを持つ最初のエンティティを取得
            let mut query = world.query::<(Entity, &WindowHandle)>();
            if let Some((entity, handle)) = query.iter(world).next() {
                println!(
                    "[Test] Despawning entity {:?} with hwnd {:?}",
                    entity, handle.hwnd
                );
                world.despawn(entity);
            }
        }
    }
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

    println!("[Test] Two windows created. Windows will close every 5 seconds.");

    // メッセージループを開始（システムが自動的にウィンドウを作成）
    mgr.run()?;

    Ok(())
}
