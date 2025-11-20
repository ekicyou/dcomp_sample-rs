#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemState;
use std::sync::mpsc::channel;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use windows::core::Result;
use windows::Win32::Foundation::{POINT, SIZE};
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
use wintf::ecs::widget::shapes::{colors, Rectangle};
use wintf::ecs::widget::text::label::TextDirection;
use wintf::ecs::widget::text::Label;
use wintf::ecs::Window;
use wintf::ecs::{Arrangement, LayoutScale, Offset};
use wintf::ecs::{GraphicsCore, WindowHandle, WindowPos};
use wintf::*;

/// バックグラウンドスレッドから送信するコマンド
/// &mut World に直接アクセスできる（排他システムで実行）
type WorldCommand = Box<dyn FnOnce(&mut World) + Send>;

#[derive(Debug, Clone, Copy, Component, PartialEq, Hash)]
pub struct HierarchyWindow;

fn main() -> Result<()> {
    human_panic::setup_panic!();

    let mgr = WinThreadMgr::new()?;
    let world = mgr.world();

    // クロージャチャンネル
    let (tx, rx) = channel::<WorldCommand>();
    let rx = Mutex::new(rx);

    // タイマースレッド: 階層構造のデモ
    thread::spawn(move || {
        // 0秒: 4階層構造のWindowを作成
        println!("[Timer Thread] 0s: Creating hierarchical window with 6 Rectangles + 2 Labels");
        let _ = tx.send(Box::new(|world: &mut World| {
            // Window Entity (ルート)
            let window_entity = world
                .spawn((
                    HierarchyWindow,
                    Window {
                        title: "wintf - Visual Tree Demo (4-Level Hierarchy)".to_string(),
                        ..Default::default()
                    },
                    WindowPos {
                        position: Some(POINT { x: 100, y: 100 }),
                        size: Some(SIZE { cx: 800, cy: 600 }),
                        ..Default::default()
                    },
                ))
                .id();

            // Rectangle1 (青、200x150、offset: 20,20)
            let rect1 = world
                .spawn((
                    Rectangle {
                        width: 200.0,
                        height: 150.0,
                        color: colors::BLUE,
                    },
                    Arrangement {
                        offset: Offset { x: 20.0, y: 20.0 },
                        scale: LayoutScale::default(),
                    },
                    ChildOf(window_entity),
                ))
                .id();

            // Rectangle1-1 (緑、80x60、offset: 10,10)
            let rect1_1 = world
                .spawn((
                    Rectangle {
                        width: 80.0,
                        height: 60.0,
                        color: colors::GREEN,
                    },
                    Arrangement {
                        offset: Offset { x: 10.0, y: 10.0 },
                        scale: LayoutScale::default(),
                    },
                    ChildOf(rect1),
                ))
                .id();

            // Label1 (赤「Hello」、offset: 5,5) - Rectangle1-1の子
            world.spawn((
                Label {
                    text: "Hello".to_string(),
                    font_family: "MS Gothic".to_string(),
                    font_size: 16.0,
                    color: colors::RED,
                    ..Default::default()
                },
                Arrangement {
                    offset: Offset { x: 5.0, y: 5.0 },
                    scale: LayoutScale::default(),
                },
                ChildOf(rect1_1),
            ));

            // Rectangle1-2 (黄、80x60、offset: 10,80)
            let rect1_2 = world
                .spawn((
                    Rectangle {
                        width: 80.0,
                        height: 60.0,
                        color: D2D1_COLOR_F {
                            r: 1.0,
                            g: 1.0,
                            b: 0.0,
                            a: 1.0,
                        }, // 黄色
                    },
                    Arrangement {
                        offset: Offset { x: 10.0, y: 80.0 },
                        scale: LayoutScale::default(),
                    },
                    ChildOf(rect1),
                ))
                .id();

            // Rectangle1-2-1 (紫、60x40、offset: 10,10)
            let rect1_2_1 = world
                .spawn((
                    Rectangle {
                        width: 60.0,
                        height: 40.0,
                        color: D2D1_COLOR_F {
                            r: 0.5,
                            g: 0.0,
                            b: 0.5,
                            a: 1.0,
                        }, // 紫色
                    },
                    Arrangement {
                        offset: Offset { x: 10.0, y: 10.0 },
                        scale: LayoutScale::default(),
                    },
                    ChildOf(rect1_2),
                ))
                .id();

            // Label2 (白「World」、offset: 5,5) - Rectangle1-2-1の子
            world.spawn((
                Label {
                    text: "World".to_string(),
                    font_family: "MS Gothic".to_string(),
                    font_size: 16.0,
                    color: colors::WHITE,
                    ..Default::default()
                },
                Arrangement {
                    offset: Offset { x: 5.0, y: 5.0 },
                    scale: LayoutScale::default(),
                },
                ChildOf(rect1_2_1),
            ));

            println!("[Test] Hierarchical window created with:");
            println!("  Window (root)");
            println!("  └─ Rectangle1 (blue, 200x150 @ 20,20)");
            println!("     ├─ Rectangle1-1 (green, 80x60 @ 10,10)");
            println!("     │  └─ Label1 (red 'Hello' @ 5,5)");
            println!("     └─ Rectangle1-2 (yellow, 80x60 @ 10,80)");
            println!("        └─ Rectangle1-2-1 (purple, 60x40 @ 10,10)");
            println!("           └─ Label2 (white 'World' @ 5,5)");

            // --- Vertical Text Verification ---
            // Container for Vertical Text (Gray, 200x300, offset: 300, 20)
            let v_container = world
                .spawn((
                    Rectangle {
                        width: 300.0,
                        height: 200.0,
                        color: D2D1_COLOR_F {
                            r: 0.8,
                            g: 0.8,
                            b: 0.8,
                            a: 1.0,
                        }, // Gray
                    },
                    Arrangement {
                        offset: Offset { x: 300.0, y: 20.0 },
                        scale: LayoutScale::default(),
                    },
                    ChildOf(window_entity),
                ))
                .id();

            // Vertical Label (Black, "縦書き\nテスト", offset: 10, 10)
            world.spawn((
                Label {
                    text: "縦書き\nテスト".to_string(),
                    font_family: "メイリオ".to_string(),
                    font_size: 24.0,
                    color: colors::BLACK,
                    direction: TextDirection::VerticalRightToLeft,
                },
                Arrangement {
                    offset: Offset { x: 10.0, y: 10.0 },
                    scale: LayoutScale::default(),
                },
                ChildOf(v_container),
            ));

            // Horizontal RTL Label (Black, "RTL Test", offset: 100, 10)
            world.spawn((
                Label {
                    text: "RTL Test \u{05E9}\u{05DC}\u{05D5}\u{05DD}".to_string(),
                    font_family: "Arial".to_string(),
                    font_size: 24.0,
                    color: colors::BLACK,
                    direction: TextDirection::HorizontalRightToLeft,
                    ..Default::default()
                },
                Arrangement {
                    offset: Offset { x: 100.0, y: 10.0 },
                    scale: LayoutScale::default(),
                },
                ChildOf(v_container),
            ));

            println!("  └─ Vertical Container (gray, 200x300 @ 300,20)");
            println!("     ├─ Vertical Label (black '縦書き\\nテスト' @ 10,10)");
            println!("     └─ RTL Label (black 'RTL Test (Shalom)' @ 100,10)");
        }));

        // 10秒: Windowを削除（アプリ終了）
        thread::sleep(Duration::from_secs(10));
        println!("[Timer Thread] 10s: Closing window");
        let _ = tx.send(Box::new(|world: &mut World| {
            let mut query = world.query_filtered::<Entity, With<HierarchyWindow>>();
            if let Some(entity) = query.iter(world).next() {
                println!("[Test] Removing Window entity {entity:?})");

                // world.despawn(entity)を直接呼ぶと、保留中のコマンド（コンポーネント追加など）と競合してパニックになる可能性がある。
                // 代わりにCommandsを使って削除をキューイングし、順序正しく処理させる。

                // 再帰的に削除対象を収集（despawn_recursiveの代用）
                let mut to_despawn = Vec::new();
                let mut stack = vec![entity];
                while let Some(curr) = stack.pop() {
                    to_despawn.push(curr);
                    if let Some(children) = world.get::<Children>(curr) {
                        for child in children.iter() {
                            stack.push(child);
                        }
                    }
                }

                let mut system_state: SystemState<Commands> = SystemState::new(world);
                let mut commands = system_state.get_mut(world);

                for e in to_despawn {
                    commands.entity(e).despawn();
                }

                // コマンドを適用（保留中のコマンド -> 削除コマンド の順で実行される）
                system_state.apply(world);
            }
        }));
    });

    println!("[Test] Timer thread started. Hierarchical visual tree demo for 10 seconds.");

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

    println!("\nビジュアルツリー階層描画の例:");
    println!("  1. Window Entity (ルート)");
    println!("  2. Rectangle1 (青) → Rectangle1-1 (緑) → Label1 (赤 'Hello')");
    println!(
        "  3. Rectangle1 (青) → Rectangle1-2 (黄) → Rectangle1-2-1 (紫) → Label2 (白 'World')"
    );
    println!("  4. 各EntityにArrangementコンポーネントで座標を指定");
    println!("  5. GlobalArrangementが自動的に親から子へ伝播");
    println!("  6. render_surfaceが階層的に描画（深さ優先）");
    println!("\n10秒後に自動的にWindowを閉じてアプリ終了します。");

    // メッセージループを開始（システムが自動的にウィンドウを作成）
    mgr.run()?;

    Ok(())
}
