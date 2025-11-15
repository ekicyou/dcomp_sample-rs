#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy_ecs::prelude::*;
use std::sync::mpsc::channel;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use windows::core::Result;
use windows::Win32::Foundation::{POINT, SIZE};
use wintf::ecs::widget::shapes::{colors, Rectangle};
use wintf::ecs::Window;
use wintf::ecs::{GraphicsCore, WindowHandle, WindowPos};
use wintf::*;

fn main() -> Result<()> {
    human_panic::setup_panic!();

    let mgr = WinThreadMgr::new()?;
    let world = mgr.world();

    // チャンネル作成とタイマースレッド起動
    let (tx, rx) = channel();
    let rx = Mutex::new(rx);

    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(5));
        if tx.send(()).is_err() {
            break;
        }
    });

    println!(
        "[Test] Auto-close timer started. Will close windows every 5 seconds using background thread."
    );

    // クロージャシステム: Receiverをキャプチャしてシステムとして登録
    world.borrow_mut().add_systems(
        wintf::ecs::world::Update,
        move |mut commands: Commands,
              windows: Query<(Entity, &WindowHandle)>,
              graphics_core: Option<Res<GraphicsCore>>| {
            // 初回実行時にGraphicsCoreの検証
            static VERIFIED: std::sync::atomic::AtomicBool =
                std::sync::atomic::AtomicBool::new(false);
            if !VERIFIED.load(std::sync::atomic::Ordering::Relaxed) && graphics_core.is_some() {
                println!("\n========== Graphics Initialization Test ==========");
                println!("[TEST PASS] GraphicsCore resource exists");

                let window_count = windows.iter().count();
                println!("[TEST] Found {} window entities", window_count);

                if window_count > 0 {
                    println!("[TEST PASS] Windows with graphics components found");
                }
                println!("========== Test Complete ==========\n");

                VERIFIED.store(true, std::sync::atomic::Ordering::Relaxed);
            }

            // try_iter()で利用可能な全通知を処理
            let Ok(rx_guard) = rx.lock() else {
                return;
            };

            for () in rx_guard.try_iter() {
                let window_count = windows.iter().count();

                if window_count > 0 {
                    println!("[Test] Closing one window (remaining: {})...", window_count);

                    if let Some((entity, handle)) = windows.iter().next() {
                        println!(
                            "[Test] Despawning entity {:?} with hwnd {:?}",
                            entity, handle.hwnd
                        );
                        commands.entity(entity).despawn();
                    }
                }
            }
        },
    );

    // 1つ目のWindowコンポーネントを持つEntityを作成
    let window1_entity = world
        .borrow_mut()
        .world_mut()
        .spawn((
            Window {
                title: "wintf - ECS Window 1 (Red Rectangle)".to_string(),
                ..Default::default()
            },
            WindowPos {
                position: Some(POINT { x: 100, y: 100 }),
                size: Some(SIZE { cx: 800, cy: 600 }),
                ..Default::default()
            },
            Rectangle {
                x: 100.0,
                y: 100.0,
                width: 200.0,
                height: 150.0,
                color: colors::RED,
            },
        ))
        .id();

    // 2つ目のWindowコンポーネントを持つEntityを作成
    let window2_entity = world
        .borrow_mut()
        .world_mut()
        .spawn((
            Window {
                title: "wintf - ECS Window 2 (Blue Rectangle)".to_string(),
                ..Default::default()
            },
            WindowPos {
                position: Some(POINT { x: 950, y: 150 }),
                size: Some(SIZE { cx: 600, cy: 400 }),
                ..Default::default()
            },
            Rectangle {
                x: 150.0,
                y: 150.0,
                width: 180.0,
                height: 120.0,
                color: colors::BLUE,
            },
        ))
        .id();

    println!("[Test] Two windows created with rectangles:");
    println!(
        "  Window 1 (Entity {:?}): Red rectangle at (100, 100), size 200x150",
        window1_entity
    );
    println!(
        "  Window 2 (Entity {:?}): Blue rectangle at (150, 150), size 180x120",
        window2_entity
    );
    println!("\nWidget描画の例:");
    println!("  1. WindowエンティティにRectangleコンポーネントを追加");
    println!("  2. draw_rectanglesシステムが自動的にGraphicsCommandListを生成");
    println!("  3. render_surfaceシステムがSurfaceに描画");
    println!("  4. commit_compositionで画面に表示");
    println!("\nバックグラウンドスレッドとの通信例:");
    println!("  1. 別スレッドで5秒ごとにstd::thread::sleep");
    println!("  2. std::mpsc::channelでメインスレッドに通知");
    println!("  3. システムがtry_iter()でノンブロッキングに全通知を処理");

    // メッセージループを開始（システムが自動的にウィンドウを作成）
    mgr.run()?;

    Ok(())
}
