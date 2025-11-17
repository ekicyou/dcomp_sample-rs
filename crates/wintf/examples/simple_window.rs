#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy_ecs::prelude::*;
use std::sync::mpsc::channel;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use windows::core::Result;
use windows::Win32::Foundation::{POINT, SIZE};
use wintf::ecs::widget::shapes::{colors, Rectangle};
use wintf::ecs::widget::text::Label;
use wintf::ecs::Window;
use wintf::ecs::{GraphicsCore, WindowHandle, WindowPos};
use wintf::*;

/// バックグラウンドスレッドから送信するコマンド
/// &mut World に直接アクセスできる（排他システムで実行）
type WorldCommand = Box<dyn FnOnce(&mut World) + Send>;

#[derive(Debug, Clone, Copy, Component, PartialEq, Hash)]
pub struct Window1;

#[derive(Debug, Clone, Copy, Component, PartialEq, Hash)]
pub struct Window2;

fn main() -> Result<()> {
    human_panic::setup_panic!();

    let mgr = WinThreadMgr::new()?;
    let world = mgr.world();

    // クロージャチャンネル
    let (tx, rx) = channel::<WorldCommand>();
    let rx = Mutex::new(rx);

    // タイマースレッド: シナリオベースでコマンドを送信
    thread::spawn(move || {
        // 0秒: すぐにWindowを2つ作成
        println!("[Timer Thread] 0s: Creating two windows");
        let _ = tx.send(Box::new(|world: &mut World| {
            // 1つ目のWindow
            world.spawn((
                Window1,
                Window {
                    title: "wintf - ECS Window 1 (Red Rectangle + Label)".to_string(),
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
                Label {
                    text: "こんにちは、世界！".to_string(),
                    font_family: "メイリオ".to_string(),
                    font_size: 24.0,
                    color: colors::BLACK,
                    x: 120.0,
                    y: 280.0,
                },
            ));

            // 2つ目のWindow
            world.spawn((
                Window2,
                Window {
                    title: "wintf - ECS Window 2 (Blue Rectangle + Multi Label)".to_string(),
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
                Label {
                    text: "Hello, DirectWrite!".to_string(),
                    font_family: "Arial".to_string(),
                    font_size: 20.0,
                    color: colors::GREEN,
                    x: 50.0,
                    y: 50.0,
                },
            ));

            println!("[Test] Two windows created");
        }));

        // 2秒: Window1からRectangleコンポーネントを削除
        thread::sleep(Duration::from_secs(2));
        println!("[Timer Thread] 2s: Removing Rectangle from first window");
        let _ = tx.send(Box::new(|world: &mut World| {
            let mut query = world.query_filtered::<Entity, With<Window1>>();
            if let Some(entity) = query.iter(world).next() {
                println!("[Test] Removing Rectangle component from entity {entity:?})");
                world.entity_mut(entity).remove::<Rectangle>();
            }
        }));

        // 4秒: Window2からRectangleコンポーネントを削除
        thread::sleep(Duration::from_secs(2));
        println!("[Timer Thread] 4s: Removing Rectangle from second window");
        let _ = tx.send(Box::new(|world: &mut World| {
            let mut query = world.query_filtered::<Entity, With<Window2>>();
            if let Some(entity) = query.iter(world).next() {
                println!("[Test] Removing Rectangle component from entity {entity:?})");
                world.entity_mut(entity).remove::<Rectangle>();
            }
        }));

        // 6秒: Window2を削除
        thread::sleep(Duration::from_secs(2));
        println!("[Timer Thread] 6s: Closing second window");
        let _ = tx.send(Box::new(|world: &mut World| {
            let mut query = world.query_filtered::<Entity, With<Window2>>();
            if let Some(entity) = query.iter(world).next() {
                println!("[Test] Removing Window2 entity {entity:?})");
                world.despawn(entity);
            }
        }));

        // 8秒: Window1を削除（これでアプリ終了）
        thread::sleep(Duration::from_secs(2));
        println!("[Timer Thread] 8s: Closing last window");
        let _ = tx.send(Box::new(|world: &mut World| {
            let mut query = world.query_filtered::<Entity, With<Window1>>();
            if let Some(entity) = query.iter(world).next() {
                println!("[Test] Removing Window1 entity {entity:?})");
                world.despawn(entity);
            }
        }));
    });

    println!("[Test] Timer thread started. Scenario: Create windows at 0s, remove Rectangles at 2s and 4s, close at 6s and 8s.");

    // 排他システム: 受信したクロージャを実行（&mut World に直接アクセス）
    world
        .borrow_mut()
        .add_systems(wintf::ecs::world::Update, move |world: &mut World| {
            // 初回実行時にGraphicsCoreの検証
            static VERIFIED: std::sync::atomic::AtomicBool =
                std::sync::atomic::AtomicBool::new(false);
            if !VERIFIED.load(std::sync::atomic::Ordering::Relaxed) {
                if world.get_resource::<GraphicsCore>().is_some() {
                    println!("\n========== Graphics Initialization Test ==========");
                    println!("[TEST PASS] GraphicsCore resource exists");

                    let mut query = world.query::<(Entity, &WindowHandle)>();
                    let window_count = query.iter(world).count();
                    println!("[TEST] Found {} window entities", window_count);

                    if window_count > 0 {
                        println!("[TEST PASS] Windows with graphics components found");
                    }
                    println!("========== Test Complete ==========\n");

                    VERIFIED.store(true, std::sync::atomic::Ordering::Relaxed);
                }
            }

            // try_iter()で受信した全クロージャを実行
            let Ok(rx_guard) = rx.lock() else {
                return;
            };

            for command in rx_guard.try_iter() {
                command(world);
            }
        });

    println!("\nWidget描画の例:");
    println!("  1. WindowエンティティにRectangleコンポーネントを追加");
    println!("  2. draw_rectanglesシステムが自動的にGraphicsCommandListを生成");
    println!("  3. render_surfaceシステムがSurfaceに描画");
    println!("  4. commit_compositionで画面に表示");
    println!("\nバックグラウンドスレッドでのシナリオテスト:");
    println!("  0s: タイマースレッドがWindowを2つ作成するコマンドを送信");
    println!("  2s: 1つ目のWindowからRectangleコンポーネントを削除");
    println!("  4s: 2つ目のWindowからRectangleコンポーネントを削除");
    println!("  6s: 2つ目のWindowを削除するコマンドを送信");
    println!("  8s: 最後のWindowを削除してアプリ終了");

    // メッセージループを開始（システムが自動的にウィンドウを作成）
    mgr.run()?;

    Ok(())
}
