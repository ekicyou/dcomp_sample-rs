#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy_ecs::name::Name;
use bevy_ecs::prelude::*;
use std::time::Duration;
use tracing_subscriber::EnvFilter;
use windows::core::Result;
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
use wintf::ecs::layout::{
    BoxInset, BoxMargin, BoxPosition, BoxSize, BoxStyle, Dimension, LengthPercentageAuto, Opacity,
};
use wintf::ecs::widget::bitmap_source::{BitmapSource, CommandSender};
use wintf::ecs::widget::shapes::Rectangle;
use wintf::ecs::Window;
use wintf::*;

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

    // tracing-subscriber 初期化
    // RUST_LOG環境変数で制御: 例 RUST_LOG=wintf=debug,info
    // 環境変数未設定時はデフォルトでinfoレベル
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let mgr = WinThreadMgr::new()?;
    let world = mgr.world();

    // 非同期タスクでFlexboxデモを実行
    world.borrow().spawn(|tx| async move {
        run_demo(tx).await;
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

/// 非同期デモ実行
async fn run_demo(tx: CommandSender) {
    // === 0秒: ウィンドウ作成 ===
    println!("[Async] 0s: Creating Flexbox demo window");
    let _ = tx.send(Box::new(create_flexbox_window));

    // === 5秒待機 ===
    async_io::Timer::after(Duration::from_secs(5)).await;

    // === 5秒: レイアウト変更 ===
    println!("[Async] 5s: Changing layout parameters");
    let _ = tx.send(Box::new(change_layout_parameters));

    // === 5秒待機 ===
    async_io::Timer::after(Duration::from_secs(5)).await;

    // === 10秒: ウィンドウ終了 ===
    println!("[Async] 10s: Closing window");
    let _ = tx.send(Box::new(close_window));
}

/// Flexboxデモウィンドウを作成
fn create_flexbox_window(world: &mut World) {
    // Window Entity (ルート)
    // BoxPosition::Absolute + BoxInset でクライアント領域の位置を指定
    let window_entity = world
        .spawn((
            Name::new("FlexDemo-Window"),
            FlexDemoWindow,
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
        ))
        .id();

    // Flexコンテナ（横並び）
    let flex_container = world
        .spawn((
            Name::new("FlexDemo-Container"),
            FlexDemoContainer,
            Rectangle {
                color: D2D1_COLOR_F {
                    r: 0.9,
                    g: 0.9,
                    b: 0.9,
                    a: 1.0,
                },
            },
            BoxStyle {
                flex_direction: Some(taffy::FlexDirection::Row),
                justify_content: Some(taffy::JustifyContent::SpaceEvenly),
                align_items: Some(taffy::AlignItems::Center),
                size: Some(BoxSize {
                    width: None,
                    height: None,
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
        .id();

    // Flexアイテム1（赤、固定200px幅）
    let red_box = world
        .spawn((
            Name::new("RedBox"),
            RedBox,
            Opacity(0.5),
            Rectangle {
                color: D2D1_COLOR_F {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
            },
            BoxStyle {
                size: Some(BoxSize {
                    width: Some(Dimension::Px(200.0)),
                    height: Some(Dimension::Px(100.0)),
                }),
                flex_grow: Some(0.0),
                flex_shrink: Some(0.0),
                flex_basis: Some(Dimension::Px(200.0)),
                ..Default::default()
            },
            ChildOf(flex_container),
        ))
        .id();

    // 赤ボックスの子として画像を追加（BitmapSourceデモ）
    const SEIKATU_IMAGE_PATH: &str =
        concat!(env!("CARGO_MANIFEST_DIR"), "/tests/assets/seikatu_0_0.webp");
    world.spawn((
        Name::new("SeikatuImage"),
        BitmapSource::new(SEIKATU_IMAGE_PATH),
        BoxStyle {
            size: Some(BoxSize {
                width: Some(Dimension::Px(64.0)),
                height: Some(Dimension::Px(64.0)),
            }),
            margin: Some(BoxMargin(wintf::ecs::layout::Rect {
                left: wintf::ecs::layout::LengthPercentageAuto::Px(68.0),
                right: wintf::ecs::layout::LengthPercentageAuto::Auto,
                top: wintf::ecs::layout::LengthPercentageAuto::Px(18.0),
                bottom: wintf::ecs::layout::LengthPercentageAuto::Px(18.0),
            })),
            ..Default::default()
        },
        ChildOf(red_box),
    ));

    // Flexアイテム2（緑、growで伸縮）
    world.spawn((
        Name::new("GreenBox"),
        GreenBox,
        Opacity(0.5),
        Rectangle {
            color: D2D1_COLOR_F {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            },
        },
        BoxStyle {
            size: Some(BoxSize {
                width: Some(Dimension::Px(100.0)),
                height: Some(Dimension::Px(100.0)),
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
        Name::new("BlueBox"),
        BlueBox,
        Opacity(0.5),
        Rectangle {
            color: D2D1_COLOR_F {
                r: 0.0,
                g: 0.0,
                b: 1.0,
                a: 1.0,
            },
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
}

/// レイアウトパラメーターを変更
fn change_layout_parameters(world: &mut World) {
    // WindowエンティティのBoxStyleを変更してウィンドウを移動・リサイズ
    let mut window_query = world.query_filtered::<&mut BoxStyle, With<FlexDemoWindow>>();
    if let Some(mut style) = window_query.iter_mut(world).next() {
        style.size = Some(BoxSize {
            width: Some(Dimension::Px(600.0)),
            height: Some(Dimension::Px(400.0)),
        });
        style.inset = Some(BoxInset(wintf::ecs::layout::Rect {
            left: LengthPercentageAuto::Px(-500.0),
            top: LengthPercentageAuto::Px(400.0),
            right: LengthPercentageAuto::Auto,
            bottom: LengthPercentageAuto::Auto,
        }));
        println!("[Test] Window BoxStyle changed: position=(-500,400), size=(600,400) in DIP");
    }

    // FlexContainerを縦並びに変更
    let mut container_query = world.query_filtered::<&mut BoxStyle, With<FlexDemoContainer>>();
    if let Some(mut style) = container_query.iter_mut(world).next() {
        style.flex_direction = Some(taffy::FlexDirection::Column);
        style.justify_content = Some(taffy::JustifyContent::SpaceAround);
        println!("[Test] FlexContainer direction changed to Column");
    }

    // 赤い矩形のサイズを変更
    let mut red_query = world.query_filtered::<&mut BoxStyle, With<RedBox>>();
    if let Some(mut style) = red_query.iter_mut(world).next() {
        if let Some(ref mut size) = style.size {
            size.width = Some(Dimension::Px(150.0));
            size.height = Some(Dimension::Px(80.0));
        }
        println!("[Test] RedBox size changed to 150x80");
    }

    // 緑の矩形のgrowを変更
    let mut green_query = world.query_filtered::<&mut BoxStyle, With<GreenBox>>();
    if let Some(mut style) = green_query.iter_mut(world).next() {
        style.flex_grow = Some(2.0);
        println!("[Test] GreenBox grow changed to 2.0");
    }

    // 青い矩形のgrowを変更
    let mut blue_query = world.query_filtered::<&mut BoxStyle, With<BlueBox>>();
    if let Some(mut style) = blue_query.iter_mut(world).next() {
        style.flex_grow = Some(1.0);
        println!("[Test] BlueBox grow changed to 1.0");
    }

    println!("[Test] Layout parameters changed:");
    println!("  FlexContainer: Row → Column, SpaceEvenly → SpaceAround");
    println!("  RedBox: 200x100 → 150x80");
    println!("  GreenBox: grow 1.0 → 2.0");
    println!("  BlueBox: grow 2.0 → 1.0");
}

/// ウィンドウを閉じる
fn close_window(world: &mut World) {
    let mut query = world.query_filtered::<Entity, With<FlexDemoWindow>>();
    if let Some(window) = query.iter(world).next() {
        println!("[Test] Removing Window entity {:?}", window);
        world.despawn(window);
    }
}
