#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy_ecs::prelude::*;
use std::sync::mpsc::channel;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use windows::core::Result;
use windows::Win32::Foundation::{POINT, SIZE};
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
use wintf::ecs::layout::{BoxSize, Dimension, FlexContainer, FlexItem};
use wintf::ecs::widget::shapes::Rectangle;
use wintf::ecs::Window;
use wintf::ecs::WindowPos;
use wintf::*;

/// バックグラウンドスレッドから送信するコマンド
type WorldCommand = Box<dyn FnOnce(&mut World) + Send>;

#[derive(Debug, Clone, Copy, Component, PartialEq, Hash)]
pub struct FlexDemoWindow;

fn main() -> Result<()> {
    human_panic::setup_panic!();

    let mgr = WinThreadMgr::new()?;
    let world = mgr.world();

    // クロージャチャンネル
    let (tx, rx) = channel::<WorldCommand>();
    let rx = Mutex::new(rx);

    // タイマースレッド: Flexboxレイアウトのデモ
    thread::spawn(move || {
        println!("[Timer Thread] 0s: Creating Flexbox demo window");
        let _ = tx.send(Box::new(|world: &mut World| {
            // Window Entity (ルート)
            let window_entity = world
                .spawn((
                    FlexDemoWindow,
                    Window {
                        title: "wintf - Taffy Flexbox Demo".to_string(),
                        ..Default::default()
                    },
                    WindowPos {
                        position: Some(POINT { x: 100, y: 100 }),
                        size: Some(SIZE { cx: 800, cy: 600 }),
                        ..Default::default()
                    },
                ))
                .id();

            // Flexコンテナ（横並び）
            let flex_container = world
                .spawn((
                    FlexContainer {
                        direction: taffy::FlexDirection::Row,
                        justify_content: Some(taffy::JustifyContent::SpaceEvenly),
                        align_items: Some(taffy::AlignItems::Center),
                    },
                    BoxSize {
                        width: Some(Dimension::Percent(100.0)),
                        height: Some(Dimension::Percent(100.0)),
                    },
                    ChildOf(window_entity),
                ))
                .id();

            // Flexアイテム1（赤、固定200px幅）
            world.spawn((
                Rectangle {
                    width: 200.0,
                    height: 150.0,
                    color: D2D1_COLOR_F {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }, // 赤
                },
                BoxSize {
                    width: Some(Dimension::Px(200.0)),
                    height: Some(Dimension::Px(150.0)),
                },
                FlexItem {
                    grow: 0.0,
                    shrink: 0.0,
                    basis: Dimension::Px(200.0),
                    align_self: None,
                },
                ChildOf(flex_container),
            ));

            // Flexアイテム2（緑、growで残りスペースの1/3）
            world.spawn((
                Rectangle {
                    width: 100.0,
                    height: 200.0,
                    color: D2D1_COLOR_F {
                        r: 0.0,
                        g: 1.0,
                        b: 0.0,
                        a: 1.0,
                    }, // 緑
                },
                BoxSize {
                    width: Some(Dimension::Px(100.0)),
                    height: Some(Dimension::Px(200.0)),
                },
                FlexItem {
                    grow: 1.0,
                    shrink: 1.0,
                    basis: Dimension::Auto,
                    align_self: None,
                },
                ChildOf(flex_container),
            ));

            // Flexアイテム3（青、growで残りスペースの2/3）
            world.spawn((
                Rectangle {
                    width: 100.0,
                    height: 100.0,
                    color: D2D1_COLOR_F {
                        r: 0.0,
                        g: 0.0,
                        b: 1.0,
                        a: 1.0,
                    }, // 青
                },
                BoxSize {
                    width: Some(Dimension::Px(100.0)),
                    height: Some(Dimension::Px(100.0)),
                },
                FlexItem {
                    grow: 2.0,
                    shrink: 1.0,
                    basis: Dimension::Auto,
                    align_self: None,
                },
                ChildOf(flex_container),
            ));

            println!("[Test] Flexbox demo window created:");
            println!("  Window (root)");
            println!("  └─ FlexContainer (Row, SpaceEvenly, Center)");
            println!("     ├─ Rectangle (red, 200px fixed)");
            println!("     ├─ Rectangle (green, grow=1)");
            println!("     └─ Rectangle (blue, grow=2)");
        }));

        // 10秒後にウィンドウを閉じる
        thread::sleep(Duration::from_secs(10));
        println!("[Timer Thread] 10s: Closing window");
        let _ = tx.send(Box::new(|world: &mut World| {
            let mut query = world.query_filtered::<Entity, With<FlexDemoWindow>>();
            if let Some(window) = query.iter(world).next() {
                println!("[Test] Removing Window entity {:?}", window);
                world.despawn(window);
            }
        }));
    });

    // システム登録: コマンド処理
    world
        .borrow_mut()
        .add_systems(wintf::ecs::world::Update, move |world: &mut World| {
            let Ok(rx_guard) = rx.lock() else {
                return;
            };

            for command in rx_guard.try_iter() {
                command(world);
            }
        });

    println!("\nTaffy Flexboxレイアウトのデモ:");
    println!("  1. Window Entity (ルート)");
    println!("  2. FlexContainer (横並び、均等配置、中央揃え)");
    println!("  3. 赤い矩形 (固定200px幅)");
    println!("  4. 緑の矩形 (grow=1.0、残りスペースの1/3)");
    println!("  5. 青い矩形 (grow=2.0、残りスペースの2/3)");
    println!("\n10秒後に自動的にWindowを閉じてアプリ終了します。");

    // メッセージループを開始
    mgr.run()?;

    Ok(())
}
