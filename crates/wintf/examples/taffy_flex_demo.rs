#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy_ecs::name::Name;
use bevy_ecs::prelude::*;
use std::sync::mpsc::channel;
use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use windows::core::Result;
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
use wintf::ecs::layout::{
    BoxInset, BoxMargin, BoxPosition, BoxSize, BoxStyle, Dimension, LengthPercentageAuto, Opacity,
};
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
            // BoxPosition::Absolute + BoxInset でクライアント領域の位置を指定
            // Note: LayoutRootマーカーは不要 - Window追加時に自動的にLayoutRootの子になる
            let window_entity = world
                .spawn((
                    Name::new("FlexDemo-Window"), // R1.1: Windowエンティティに名前を付与
                    FlexDemoWindow,
                    // LayoutRoot削除: on_window_addフックでChildOf(layout_root)が自動設定される
                    BoxStyle {
                        position: Some(BoxPosition::Absolute),
                        inset: Some(BoxInset(wintf::ecs::layout::Rect {
                            left: LengthPercentageAuto::Px(100.0),
                            top: LengthPercentageAuto::Px(100.0),
                            right: LengthPercentageAuto::Auto,
                            bottom: LengthPercentageAuto::Auto,
                        })),
                        size: Some(BoxSize {
                            width: Some(Dimension::Px(800.0)),
                            height: Some(Dimension::Px(600.0)),
                        }),
                        ..Default::default()
                    },
                    Window {
                        title: "wintf - Taffy Flexbox Demo".to_string(),
                        ..Default::default()
                    },
                    WindowPos::default(), // レイアウトシステムが更新
                                          // Visual は Window の on_add フックで自動挿入される
                ))
                .id();

            // Flexコンテナ（横並び）
            let flex_container = world
                .spawn((
                    Name::new("FlexDemo-Container"), // R1.2: FlexContainerエンティティに名前を付与
                    FlexDemoContainer,               // マーカー追加
                    Rectangle {
                        color: D2D1_COLOR_F {
                            r: 0.9,
                            g: 0.9,
                            b: 0.9,
                            a: 1.0,
                        }, // 灰色（背景）
                    },
                    BoxStyle {
                        flex_direction: Some(taffy::FlexDirection::Row),
                        justify_content: Some(taffy::JustifyContent::SpaceEvenly),
                        align_items: Some(taffy::AlignItems::Center),
                        size: Some(BoxSize {
                            width: None,  // Autoで親サイズから計算
                            height: None, // Autoで親サイズから計算
                        }),
                        margin: Some(BoxMargin(wintf::ecs::layout::Rect {
                            left: wintf::ecs::layout::LengthPercentageAuto::Px(10.0),
                            right: wintf::ecs::layout::LengthPercentageAuto::Px(10.0),
                            top: wintf::ecs::layout::LengthPercentageAuto::Px(10.0),
                            bottom: wintf::ecs::layout::LengthPercentageAuto::Px(10.0),
                        })),
                        ..Default::default()
                    },
                    ChildOf(window_entity),
                ))
                .id(); // Flexアイテム1（赤、固定200px幅）
            world.spawn((
                Name::new("RedBox"), // R1.3: RedBoxエンティティに名前を付与
                RedBox,              // マーカー追加
                Opacity(0.5),        // 50%透明度
                Rectangle {
                    color: D2D1_COLOR_F {
                        r: 1.0,
                        g: 0.0,
                        b: 0.0,
                        a: 1.0,
                    }, // 赤
                },
                BoxStyle {
                    size: Some(BoxSize {
                        width: Some(Dimension::Px(200.0)),
                        height: Some(Dimension::Px(100.0)), // 150 → 100に修正
                    }),
                    flex_grow: Some(0.0),
                    flex_shrink: Some(0.0),
                    flex_basis: Some(Dimension::Px(200.0)),
                    ..Default::default()
                },
                ChildOf(flex_container),
            ));

            // Flexアイテム2（緑、growで伸縮）
            world.spawn((
                Name::new("GreenBox"), // R1.4: GreenBoxエンティティに名前を付与
                GreenBox,              // マーカー追加
                Opacity(0.5),          // 50%透明度
                Rectangle {
                    color: D2D1_COLOR_F {
                        r: 0.0,
                        g: 1.0,
                        b: 0.0,
                        a: 1.0,
                    }, // 緑
                },
                BoxStyle {
                    size: Some(BoxSize {
                        width: Some(Dimension::Px(100.0)),
                        height: Some(Dimension::Px(100.0)), // 200 → 100に修正
                    }),
                    flex_grow: Some(1.0),
                    flex_shrink: Some(1.0),
                    flex_basis: Some(Dimension::Auto),
                    ..Default::default()
                },
                ChildOf(flex_container),
            ));

            // Flexアイテム3（青、growで伸縮、より大きなgrow値）
            world.spawn((
                Name::new("BlueBox"), // R1.5: BlueBoxエンティティに名前を付与
                BlueBox,              // マーカー追加
                Opacity(0.5),         // 50%透明度
                Rectangle {
                    color: D2D1_COLOR_F {
                        r: 0.0,
                        g: 0.0,
                        b: 1.0,
                        a: 1.0,
                    }, // 青
                },
                BoxStyle {
                    size: Some(BoxSize {
                        width: Some(Dimension::Px(100.0)),
                        height: Some(Dimension::Px(100.0)),
                    }),
                    flex_grow: Some(2.0),
                    flex_shrink: Some(1.0),
                    flex_basis: Some(Dimension::Auto),
                    ..Default::default()
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
            // *** WindowエンティティのBoxStyleを変更してウィンドウを移動・リサイズ ***
            let mut window_query = world.query_filtered::<&mut BoxStyle, With<FlexDemoWindow>>();
            if let Some(mut style) = window_query.iter_mut(world).next() {
                // ウィンドウを左モニター (-1500, 500) に移動し、サイズを 600x400 に変更
                // 左モニターは x=-2880〜0 の範囲、DPI=192 (200%スケール)
                // 右モニターは x=0〜3840, DPI=120 (125%スケール)
                // これによりWM_DPICHANGEDが発火するはず
                style.size = Some(BoxSize {
                    width: Some(Dimension::Px(600.0)),
                    height: Some(Dimension::Px(400.0)),
                });
                style.inset = Some(BoxInset(wintf::ecs::layout::Rect {
                    left: LengthPercentageAuto::Px(-1500.0), // 左モニター領域へ移動
                    top: LengthPercentageAuto::Px(500.0),
                    right: LengthPercentageAuto::Auto,
                    bottom: LengthPercentageAuto::Auto,
                }));
                println!("[Test] Window BoxStyle changed: position=(-1500,500), size=(600,400)");
                println!("[Test] Moving to left monitor (DPI=192) to trigger WM_DPICHANGED");
            }

            // FlexContainerを縦並びに変更
            let mut container_query =
                world.query_filtered::<&mut BoxStyle, With<FlexDemoContainer>>();
            if let Some(mut style) = container_query.iter_mut(world).next() {
                style.flex_direction = Some(taffy::FlexDirection::Column);
                style.justify_content = Some(taffy::JustifyContent::SpaceAround);
                println!("[Test] FlexContainer direction changed to Column");
            }

            // 赤い矩形のサイズを変更
            let mut red_query = world.query_filtered::<&mut BoxStyle, With<RedBox>>();
            if let Some(mut style) = red_query.iter_mut(world).next() {
                if let Some(ref mut size) = style.size {
                    size.width = Some(Dimension::Px(150.0)); // 200 → 150に変更
                    size.height = Some(Dimension::Px(80.0)); // 100 → 80に変更
                }
                println!("[Test] RedBox size changed to 150x80");
            }

            // 緑の矩形のgrowを変更
            let mut green_query = world.query_filtered::<&mut BoxStyle, With<GreenBox>>();
            if let Some(mut style) = green_query.iter_mut(world).next() {
                style.flex_grow = Some(2.0); // 1.0 → 2.0
                println!("[Test] GreenBox grow changed to 2.0");
            }

            // 青い矩形のgrowを変更
            let mut blue_query = world.query_filtered::<&mut BoxStyle, With<BlueBox>>();
            if let Some(mut style) = blue_query.iter_mut(world).next() {
                style.flex_grow = Some(1.0); // 2.0 → 1.0
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
