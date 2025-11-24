#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy_ecs::prelude::*;
use std::sync::mpsc::channel;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use windows::core::Result;
use windows::Win32::Foundation::{POINT, SIZE};
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
use wintf::ecs::layout::{BoxMargin, BoxSize, Dimension, FlexContainer, FlexItem, LayoutRoot};
use wintf::ecs::widget::shapes::Rectangle;
use wintf::ecs::Window;
use wintf::ecs::WindowPos;
use wintf::*;

/// バックグラウンドスレッドから送信するコマンド
type WorldCommand = Box<dyn FnOnce(&mut World) + Send>;

#[derive(Debug, Clone, Copy, Component, PartialEq, Hash)]
pub struct FlexDemoWindow;

/// Flexコンテナを識別するマーカー
#[derive(Debug, Clone, Copy, Component, PartialEq, Hash)]
pub struct FlexDemoContainer;

/// 赤い矩形（固定サイズ）を識別するマーカー
#[derive(Debug, Clone, Copy, Component, PartialEq, Hash)]
pub struct RedBox;

/// 緑の矩形（grow=1）を識別するマーカー
#[derive(Debug, Clone, Copy, Component, PartialEq, Hash)]
pub struct GreenBox;

/// 青い矩形（grow=2）を識別するマーカー
#[derive(Debug, Clone, Copy, Component, PartialEq, Hash)]
pub struct BlueBox;

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
                    LayoutRoot, // Taffyレイアウト計算のルートマーカー
                    BoxSize {
                        width: Some(Dimension::Px(800.0)),
                        height: Some(Dimension::Px(600.0)),
                    },
                    Window {
                        title: "wintf - Taffy Flexbox Demo".to_string(),
                        ..Default::default()
                    },
                    WindowPos {
                        position: Some(POINT { x: 100, y: 100 }),
                        size: Some(SIZE { cx: 800, cy: 600 }),
                        ..Default::default()
                    },
                    wintf::ecs::Visual::default(), // Visual追加
                ))
                .id();

            // Flexコンテナ（横並び）
            let flex_container = world
                .spawn((
                    FlexDemoContainer, // マーカー追加
                    Rectangle {
                        color: D2D1_COLOR_F {
                            r: 0.9,
                            g: 0.9,
                            b: 0.9,
                            a: 1.0,
                        }, // 灰色（背景）
                    },
                    FlexContainer {
                        direction: taffy::FlexDirection::Row,
                        justify_content: Some(taffy::JustifyContent::SpaceEvenly),
                        align_items: Some(taffy::AlignItems::Center),
                    },
                    BoxSize {
                        width: None,  // Autoで親サイズから計算
                        height: None, // Autoで親サイズから計算
                    },
                    BoxMargin(wintf::ecs::layout::Rect {
                        left: wintf::ecs::layout::LengthPercentageAuto::Px(10.0),
                        right: wintf::ecs::layout::LengthPercentageAuto::Px(10.0),
                        top: wintf::ecs::layout::LengthPercentageAuto::Px(10.0),
                        bottom: wintf::ecs::layout::LengthPercentageAuto::Px(10.0),
                    }),
                    ChildOf(window_entity),
                ))
                .id(); // Flexアイテム1（赤、固定200px幅）
            world.spawn((
                RedBox, // マーカー追加
                Rectangle {
                    color: D2D1_COLOR_F {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }, // 赤
                },
                BoxSize {
                    width: Some(Dimension::Px(200.0)),
                    height: Some(Dimension::Px(100.0)), // 150 → 100に修正
                },
                FlexItem {
                    grow: 0.0,
                    shrink: 0.0,
                    basis: Dimension::Px(200.0),
                    align_self: None,
                },
                ChildOf(flex_container),
            ));

            // Flexアイテム2（緑、growで伸縮）
            world.spawn((
                GreenBox, // マーカー追加
                Rectangle {
                    color: D2D1_COLOR_F {
                        r: 0.0,
                        g: 1.0,
                        b: 0.0,
                        a: 1.0,
                    }, // 緑
                },
                BoxSize {
                    width: Some(Dimension::Px(100.0)),
                    height: Some(Dimension::Px(100.0)), // 200 → 100に修正
                },
                FlexItem {
                    grow: 1.0,
                    shrink: 1.0,
                    basis: Dimension::Auto,
                    align_self: None,
                },
                ChildOf(flex_container),
            ));

            // Flexアイテム3（青、growで伸縮、より大きなgrow値）
            world.spawn((
                BlueBox, // マーカー追加
                Rectangle {
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
            println!("  └─ FlexContainer (Row, SpaceEvenly, Center) - 灰色背景、10pxマージン");
            println!("     ├─ Rectangle (red, 200x100 fixed)");
            println!("     ├─ Rectangle (green, 100x100, grow=1)");
            println!("     └─ Rectangle (blue, 100x100, grow=2)");
        }));

        // 5秒後にレイアウトパラメーターを変更
        thread::sleep(Duration::from_secs(5));
        println!("[Timer Thread] 5s: Changing layout parameters");
        let _ = tx.send(Box::new(|world: &mut World| {
            // FlexContainerを縦並びに変更
            let mut container_query =
                world.query_filtered::<&mut FlexContainer, With<FlexDemoContainer>>();
            if let Some(mut flex_container) = container_query.iter_mut(world).next() {
                flex_container.direction = taffy::FlexDirection::Column;
                flex_container.justify_content = Some(taffy::JustifyContent::SpaceAround);
                println!("[Test] FlexContainer direction changed to Column");
            }

            // 赤い矩形のサイズを変更
            let mut red_query = world.query_filtered::<&mut BoxSize, With<RedBox>>();
            if let Some(mut box_size) = red_query.iter_mut(world).next() {
                box_size.width = Some(Dimension::Px(150.0)); // 200 → 150に変更
                box_size.height = Some(Dimension::Px(80.0)); // 100 → 80に変更
                println!("[Test] RedBox size changed to 150x80");
            }

            // 緑の矩形のgrowを変更
            let mut green_query = world.query_filtered::<&mut FlexItem, With<GreenBox>>();
            if let Some(mut flex_item) = green_query.iter_mut(world).next() {
                flex_item.grow = 2.0; // 1.0 → 2.0
                println!("[Test] GreenBox grow changed to 2.0");
            }

            // 青い矩形のgrowを変更
            let mut blue_query = world.query_filtered::<&mut FlexItem, With<BlueBox>>();
            if let Some(mut flex_item) = blue_query.iter_mut(world).next() {
                flex_item.grow = 1.0; // 2.0 → 1.0
                println!("[Test] BlueBox grow changed to 1.0");
            }

            println!("[Test] Layout parameters changed:");
            println!("  FlexContainer: Row → Column, SpaceEvenly → SpaceAround");
            println!("  RedBox: 200x100 → 150x80");
            println!("  GreenBox: grow 1.0 → 2.0");
            println!("  BlueBox: grow 2.0 → 1.0");
        }));

        // 10秒後にウィンドウを閉じる
        thread::sleep(Duration::from_secs(5));
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
    println!("  1. Window Entity (ルート) - 800x600");
    println!("  2. FlexContainer (横並び、均等配置、中央揃え) - 灰色背景");
    println!("  3. 赤い矩形 (固定200x100)");
    println!("  4. 緑の矩形 (100x100, grow=1.0、残りスペースの1/3)");
    println!("  5. 青い矩形 (100x100, grow=2.0、残りスペースの2/3)");
    println!("\n5秒後にレイアウトパラメーターを変更します。");
    println!("10秒後に自動的にWindowを閉じてアプリ終了します。");

    // メッセージループを開始
    mgr.run()?;

    Ok(())
}
