#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! # Taffy Flexbox Demo - Tunnel/Bubbleフェーズのイベント伝播デモ
//!
//! このサンプルは、wintfフレームワークのポインターイベントシステムにおける
//! Tunnel（親→子）とBubble（子→親）の2フェーズイベント伝播を実演します。
//!
//! ## イベントフェーズの概念
//!
//! wintfのイベントシステムは、WinUI3/WPF/DOMイベントモデルと同様の2フェーズを実装:
//!
//! - **Tunnelフェーズ** (親→子): イベント発生前に親が介入可能
//! - **Bubbleフェーズ** (子→親): イベント発生後に親が処理可能
//!
//! ## 他のフレームワークとの対応表
//!
//! | wintf             | WinUI3           | WPF              | DOM Level 3      |
//! |-------------------|------------------|------------------|------------------|
//! | Phase::Tunnel     | PreviewMouseDown | PreviewMouseDown | Capture Phase    |
//! | Phase::Bubble     | MouseDown        | MouseDown        | Bubble Phase     |
//! | handler return    | e.Handled = true | e.Handled = true | stopPropagation()|
//! | sender引数        | e.OriginalSource | e.OriginalSource | event.target     |
//! | entity引数        | sender引数       | sender引数       | currentTarget    |
//!
//! ## デモの操作例
//!
//! 1. **GreenBoxChild（黄色矩形）を左クリック**
//!    - 期待: `[Tunnel] GreenBox: Captured event` のみ出力
//!    - GreenBoxChildのログは出ない（親がTunnelでキャプチャ）
//!
//! 2. **GreenBoxChild（黄色矩形）を右クリック**
//!    - 期待: `[Tunnel] GreenBox` → `[Tunnel] GreenBoxChild` → `[Bubble] GreenBoxChild`
//!    - 両エンティティがログ出力（親がキャプチャしない）
//!
//! 3. **Ctrl+左クリックでRedBox**
//!    - 期待: `[Tunnel] FlexContainer: Event stopped` のみ出力
//!    - RedBoxのログは出ない（Containerで停止）
//!
//! ## 実装パターン
//!
//! ```rust
//! fn handler(world: &mut World, sender: Entity, entity: Entity, ev: &Phase<PointerState>) -> bool {
//!     match ev {
//!         Phase::Tunnel(state) => {
//!             if state.ctrl_down && state.left_down {
//!                 // 親で事前処理してイベントを停止
//!                 return true; // stopPropagation相当
//!             }
//!             false
//!         }
//!         Phase::Bubble(state) => {
//!             // 通常のイベント処理
//!             if state.right_down {
//!                 // 処理...
//!                 return true;
//!             }
//!             false
//!         }
//!     }
//! }
//! ```

use bevy_ecs::name::Name;
use bevy_ecs::prelude::*;
use std::time::Duration;
use tracing::info;
use tracing_subscriber::EnvFilter;
use windows::core::Result;
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
use wintf::ecs::layout::{hit_test, GlobalArrangement, PhysicalPoint};
use wintf::ecs::layout::{
    BoxInset, BoxMargin, BoxPosition, BoxSize, BoxStyle, Dimension, LengthPercentageAuto, Opacity,
};
use wintf::ecs::pointer::{OnPointerMoved, OnPointerPressed, Phase, PointerState};
use wintf::ecs::widget::bitmap_source::{BitmapSource, CommandSender};
use wintf::ecs::widget::brushes::Brushes;
use wintf::ecs::widget::shapes::Rectangle;
use wintf::ecs::{Window, WindowPos};
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

/// GreenBoxの子矩形を識別するマーカー
#[derive(Debug, Clone, Copy, Component, PartialEq, Hash)]
pub struct GreenBoxChild;

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

    // === 1秒待機 ===
    async_io::Timer::after(Duration::from_secs(1)).await;

    // === 1秒: ヒットテスト検証 ===
    println!("[Async] 1s: Running hit test verification");
    let _ = tx.send(Box::new(test_hit_test_1s));

    // === 長時間待機（ポインターイベントデモ用） ===
    println!("[Async] Waiting 60 seconds for pointer event demo...");
    println!("  Try: Left-click on RedBox, BlueBox, Right-click on Container");
    async_io::Timer::after(Duration::from_secs(60)).await;

    // === 61秒: ウィンドウ終了 ===
    println!("[Async] 61s: Closing window");
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

    // Flexコンテナ（横並び）- 右クリックで色変更
    let flex_container = world
        .spawn((
            Name::new("FlexDemo-Container"),
            FlexDemoContainer,
            Rectangle::new(),
            Brushes::with_foreground(D2D1_COLOR_F {
                r: 0.9,
                g: 0.9,
                b: 0.9,
                a: 1.0,
            }),
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
            // イベントハンドラ: 右クリックで色変更
            OnPointerPressed(on_container_pressed),
            ChildOf(window_entity),
        ))
        .id();

    // Flexアイテム1（赤、固定200px幅）- 左クリックで色トグル（αマスクデモ用）
    // SeikatuImage（子）の透明部分をクリックするとRedBoxにイベントが伝播し色が変わる
    let red_box = world
        .spawn((
            Name::new("RedBox"),
            RedBox,
            Opacity(1.0),
            Rectangle::new(),
            Brushes::with_foreground(D2D1_COLOR_F {
                r: 1.0,
                g: 0.0,
                b: 0.0,
                a: 1.0,
            }),
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
            // イベントハンドラ: 左クリックで色トグル（赤 ⇔ 黄）
            OnPointerPressed(on_red_box_pressed),
            ChildOf(flex_container),
        ))
        .id();

    // 赤ボックスの子として画像を追加（αマスクヒットテストデモ）
    // 透明部分クリック → 親(RedBox)に伝播して色が変わる
    // 不透明部分クリック → 画像がイベント消費して親に伝播しない
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
        // イベントハンドラ: 不透明部分クリックでイベント消費（親に伝播しない）
        OnPointerPressed(on_image_pressed),
        ChildOf(red_box),
    ));

    // Flexアイテム2（緑、growで伸縮）- マウス移動でログ、左クリックでTunnelキャプチャ
    let green_box = world
        .spawn((
            Name::new("GreenBox"),
            GreenBox,
            Opacity(0.5),
            Rectangle::new(),
            Brushes::with_foreground(D2D1_COLOR_F {
                r: 0.0,
                g: 1.0,
                b: 0.0,
                a: 1.0,
            }),
            BoxStyle {
                flex_direction: Some(taffy::FlexDirection::Column),
                size: Some(BoxSize {
                    width: Some(Dimension::Px(100.0)),
                    height: Some(Dimension::Px(100.0)),
                }),
                flex_grow: Some(1.0),
                flex_shrink: Some(1.0),
                flex_basis: Some(Dimension::Auto),
                ..Default::default()
            },
            // イベントハンドラ: ポインター移動でログ出力
            OnPointerMoved(on_green_box_moved),
            // イベントハンドラ: ポインター押下でTunnelキャプチャ
            OnPointerPressed(on_green_box_pressed),
            ChildOf(flex_container),
        ))
        .id();

    // GreenBoxの子エンティティ（黄色矩形、半透明）
    world.spawn((
        Name::new("GreenBoxChild"),
        GreenBoxChild,
        Opacity(0.5),
        Rectangle::new(),
        Brushes::with_foreground(D2D1_COLOR_F {
            r: 1.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        }),
        BoxStyle {
            size: Some(BoxSize {
                width: Some(Dimension::Px(50.0)),
                height: Some(Dimension::Px(50.0)),
            }),
            ..Default::default()
        },
        // イベントハンドラ: Tunnelキャプチャ検証用
        OnPointerPressed(on_green_child_pressed),
        ChildOf(green_box),
    ));

    // Flexアイテム3（青、growで伸縮、より大きなgrow値）- 左クリックでサイズ変更
    world.spawn((
        Name::new("BlueBox"),
        BlueBox,
        Opacity(0.5),
        Rectangle::new(),
        Brushes::with_foreground(D2D1_COLOR_F {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 1.0,
        }),
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
        // イベントハンドラ: 左クリックでサイズトグル
        OnPointerPressed(on_blue_box_pressed),
        ChildOf(flex_container),
    ));

    println!("[Test] Flexbox demo window created:");
    println!("  Window (root)");
    println!("  └─ FlexContainer (Row, SpaceEvenly, Center) - 灰色背景、10pxマージン、右クリック/Ctrl+左クリックでTunnelデモ");
    println!("     ├─ Rectangle (red, 200x100 fixed) - 左クリックで色トグル");
    println!("     │   └─ BitmapSource (seikatu_0_0.webp) - αマスクヒットテスト有効、透明部分は親に透過");
    println!("     ├─ Rectangle (green, 100x100, grow=1, Column) - マウス移動でログ、左クリックでTunnelキャプチャ");
    println!("     │   └─ Rectangle (yellow, 50x50) - Tunnelキャプチャ検証用子エンティティ");
    println!("     └─ Rectangle (blue, 100x100, grow=2) - 左クリックでサイズトグル");
    println!("\n[PointerEvent Demo]");
    println!("  - 灰色コンテナを右クリック → 色がピンクに変化");
    println!("  - 灰色コンテナをCtrl+左クリック → Tunnelで停止、子にイベント到達せず");
    println!("  - 黄色矩形(GreenBoxChild)を左クリック → 親(GreenBox)がTunnelキャプチャ、子は到達しない");
    println!("  - 黄色矩形(GreenBoxChild)を右クリック → 親がキャプチャせず、Tunnel/Bubble両フェーズ実行");
    println!("  - 赤い矩形を左クリック → 色が赤⇔黄トグル");
    println!("  - 画像の透明部分を左クリック → 背景(RedBox)の色が変わる（αマスクヒットテスト）");
    println!("  - 画像の不透明部分を左クリック → 画像がクリックされ背景は変わらない");
    println!("  - 緑の矩形でマウス移動 → ログ出力（デバッグ）");
    println!("  - 青い矩形を左クリック → サイズが100⇔150トグル");
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

/// 1秒後のヒットテスト検証
fn test_hit_test_1s(world: &mut World) {
    println!("[HitTest @1s] === Running hit test verification ===");

    // ウィンドウエンティティを取得
    let mut window_query = world.query_filtered::<Entity, With<FlexDemoWindow>>();
    let Some(window_entity) = window_query.iter(world).next() else {
        println!("[HitTest @1s] Window entity not found");
        return;
    };

    // ウィンドウの GlobalArrangement からスケールと原点を取得
    let Some(window_global) = world.get::<GlobalArrangement>(window_entity) else {
        println!("[HitTest @1s] Window has no GlobalArrangement");
        return;
    };
    let (scale_x, scale_y) = window_global.scale();
    let origin_x = window_global.bounds.left;
    let origin_y = window_global.bounds.top;

    println!(
        "[HitTest @1s] Window scale: ({:.2}, {:.2}), origin: ({:.0}, {:.0})",
        scale_x, scale_y, origin_x, origin_y
    );

    // DIP座標からスクリーン座標（物理ピクセル）に変換するヘルパー
    let to_screen = |dip_x: f32, dip_y: f32| -> PhysicalPoint {
        PhysicalPoint::new(origin_x + dip_x * scale_x, origin_y + dip_y * scale_y)
    };

    // ウィンドウの WindowPos をログ出力（基準座標）
    println!("[HitTest @1s] --- Window reference coordinates ---");
    dump_window_pos(world, window_entity);

    // 各エンティティの GlobalArrangement.bounds をログ出力
    println!("[HitTest @1s] --- Entity bounds (GlobalArrangement) ---");
    dump_entity_bounds(world, "FlexDemo-Window", window_entity);

    // FlexContainerを検索
    let mut container_query = world.query_filtered::<Entity, With<FlexDemoContainer>>();
    if let Some(container) = container_query.iter(world).next() {
        dump_entity_bounds(world, "FlexDemo-Container", container);
    }

    // 各Boxを検索
    let mut red_query = world.query_filtered::<Entity, With<RedBox>>();
    if let Some(red) = red_query.iter(world).next() {
        dump_entity_bounds(world, "RedBox", red);
    }

    let mut green_query = world.query_filtered::<Entity, With<GreenBox>>();
    if let Some(green) = green_query.iter(world).next() {
        dump_entity_bounds(world, "GreenBox", green);
    }

    let mut blue_query = world.query_filtered::<Entity, With<BlueBox>>();
    if let Some(blue) = blue_query.iter(world).next() {
        dump_entity_bounds(world, "BlueBox", blue);
    }
    println!("[HitTest @1s] --- End of entity bounds ---");

    // テストポイント（DIP座標で指定、to_screen で物理ピクセルに変換）
    // 実際のレイアウト結果（物理ピクセル、スケール1.25、原点125,125）:
    // - GreenBox: (135,375)→(260,500) → DIP (8,200)→(108,300)
    // - RedBox: (235,375)→(485,500) → DIP (88,200)→(288,300)
    //   - RedBox内に子要素 SeikatuImage があり、中心テストでは子がヒット
    // - BlueBox: (435,375)→(560,500) → DIP (248,200)→(348,300)
    let test_points = [
        (
            to_screen(50.0, 250.0),
            "GreenBox center (DIP 50,250)",
            "GreenBox",
        ),
        (
            to_screen(150.0, 250.0),
            "RedBox child (SeikatuImage) (DIP 150,250)",
            "SeikatuImage",
        ),
        (
            to_screen(320.0, 250.0),
            "BlueBox center (DIP 320,250)",
            "BlueBox",
        ),
        (
            to_screen(15.0, 15.0),
            "Container area (DIP 15,15)",
            "FlexDemo-Container",
        ),
        (
            to_screen(700.0, 300.0),
            "Outside Container (DIP 700,300)",
            "FlexDemo-Window",
        ),
    ];

    println!("[HitTest @1s] --- Hit test results ---");
    for (point, description, expected) in test_points {
        match hit_test(world, window_entity, point) {
            Some(entity) => {
                if let Some(name) = world.get::<Name>(entity) {
                    println!(
                        "[HitTest @1s] {} at DIP->Screen ({:.0}, {:.0}): Hit {:?} (expected: {})",
                        description,
                        point.x,
                        point.y,
                        name.as_str(),
                        expected
                    );
                } else {
                    println!(
                        "[HitTest @1s] {} at ({:.0}, {:.0}): Hit Entity {:?} (no name)",
                        description, point.x, point.y, entity
                    );
                }
            }
            None => {
                println!(
                    "[HitTest @1s] {} at ({:.0}, {:.0}): No hit (expected: {})",
                    description, point.x, point.y, expected
                );
            }
        }
    }
}

/// 6秒後のヒットテスト検証（レイアウト変更後）
fn test_hit_test_6s(world: &mut World) {
    println!("[HitTest @6s] === Running hit test after layout change ===");

    // ウィンドウエンティティを取得
    let mut window_query = world.query_filtered::<Entity, With<FlexDemoWindow>>();
    let Some(window_entity) = window_query.iter(world).next() else {
        println!("[HitTest @6s] Window entity not found");
        return;
    };

    // ウィンドウの GlobalArrangement からスケールと原点を取得
    let Some(window_global) = world.get::<GlobalArrangement>(window_entity) else {
        println!("[HitTest @6s] Window has no GlobalArrangement");
        return;
    };
    let (scale_x, scale_y) = window_global.scale();
    let origin_x = window_global.bounds.left;
    let origin_y = window_global.bounds.top;

    println!(
        "[HitTest @6s] Window scale: ({:.2}, {:.2}), origin: ({:.0}, {:.0})",
        scale_x, scale_y, origin_x, origin_y
    );

    // DIP座標からスクリーン座標（物理ピクセル）に変換するヘルパー
    let to_screen = |dip_x: f32, dip_y: f32| -> PhysicalPoint {
        PhysicalPoint::new(origin_x + dip_x * scale_x, origin_y + dip_y * scale_y)
    };

    // 各エンティティの GlobalArrangement.bounds をログ出力（デバッグ用）
    println!("[HitTest @6s] --- Entity bounds (GlobalArrangement) ---");
    dump_entity_bounds(world, "FlexDemo-Window", window_entity);

    let mut container_query = world.query_filtered::<Entity, With<FlexDemoContainer>>();
    if let Some(container) = container_query.iter(world).next() {
        dump_entity_bounds(world, "FlexDemo-Container", container);
    }

    let mut red_query = world.query_filtered::<Entity, With<RedBox>>();
    if let Some(red) = red_query.iter(world).next() {
        dump_entity_bounds(world, "RedBox", red);
    }

    let mut green_query = world.query_filtered::<Entity, With<GreenBox>>();
    if let Some(green) = green_query.iter(world).next() {
        dump_entity_bounds(world, "GreenBox", green);
    }

    let mut blue_query = world.query_filtered::<Entity, With<BlueBox>>();
    if let Some(blue) = blue_query.iter(world).next() {
        dump_entity_bounds(world, "BlueBox", blue);
    }
    println!("[HitTest @6s] --- End of entity bounds ---");

    // テストポイント（DIP座標で指定）
    // 6秒時点: ウィンドウサイズ 600x400 DIP、Column レイアウト
    // 実際のレイアウト結果に基づく（Containerは幅150DIP程度、左寄せ）
    // GreenBox, RedBox, BlueBox は Container内で縦並び
    let test_points = [
        (
            to_screen(20.0, 50.0),
            "GreenBox area (DIP 20,50)",
            "GreenBox",
        ),
        (to_screen(20.0, 150.0), "RedBox area (DIP 20,150)", "RedBox"),
        (
            to_screen(20.0, 200.0),
            "BlueBox area (DIP 20,200)",
            "BlueBox",
        ),
        (
            to_screen(5.0, 5.0),
            "Top-left corner (DIP 5,5)",
            "FlexDemo-Container",
        ),
        (
            to_screen(400.0, 200.0),
            "Right side - outside Container (DIP 400,200)",
            "FlexDemo-Window",
        ),
    ];

    println!("[HitTest @6s] --- Hit test results ---");
    for (point, description, expected) in test_points {
        match hit_test(world, window_entity, point) {
            Some(entity) => {
                if let Some(name) = world.get::<Name>(entity) {
                    let result = if name.as_str() == expected {
                        "✓"
                    } else {
                        "✗"
                    };
                    println!(
                        "[HitTest @6s] {} {} -> ({:.0}, {:.0}): Hit {:?} (expected: {})",
                        result,
                        description,
                        point.x,
                        point.y,
                        name.as_str(),
                        expected
                    );
                } else {
                    println!(
                        "[HitTest @6s] ✗ {} -> ({:.0}, {:.0}): Hit Entity {:?} (no name, expected: {})",
                        description, point.x, point.y, entity, expected
                    );
                }
            }
            None => {
                println!(
                    "[HitTest @6s] ✗ {} -> ({:.0}, {:.0}): No hit (expected: {})",
                    description, point.x, point.y, expected
                );
            }
        }
    }
}

/// エンティティの GlobalArrangement.bounds をログ出力
fn dump_entity_bounds(world: &World, name: &str, entity: Entity) {
    if let Some(global) = world.get::<GlobalArrangement>(entity) {
        let b = &global.bounds;
        println!(
            "[HitTest] {} bounds: left={:.1}, top={:.1}, right={:.1}, bottom={:.1} (size: {:.1}x{:.1})",
            name, b.left, b.top, b.right, b.bottom,
            b.right - b.left, b.bottom - b.top
        );
    } else {
        println!("[HitTest] {} has no GlobalArrangement", name);
    }
}

/// ウィンドウの WindowPos をログ出力
fn dump_window_pos(world: &World, entity: Entity) {
    if let Some(window_pos) = world.get::<WindowPos>(entity) {
        if let Some(pos) = window_pos.position {
            println!("[HitTest] WindowPos.position: x={}, y={}", pos.x, pos.y);
        } else {
            println!("[HitTest] WindowPos.position: None");
        }
        if let Some(size) = window_pos.size {
            println!("[HitTest] WindowPos.size: cx={}, cy={}", size.cx, size.cy);
        } else {
            println!("[HitTest] WindowPos.size: None");
        }
    } else {
        println!("[HitTest] Window has no WindowPos");
    }
}

// ============================================================================
// ポインターイベントハンドラ
// ============================================================================

/// FlexContainer の OnPointerPressed ハンドラ（拡張版）
///
/// **Tunnelフェーズ**: Ctrl+左クリックでキャプチャ（条件付き前処理の例）
/// **Bubbleフェーズ**: 右クリックで色変更（既存）
///
/// # パラメータ
/// - `sender`: イベント発生元エンティティ（e.OriginalSource相当）
/// - `entity`: 現在処理中のエンティティ（e.currentTarget相当）
/// - `ev`: Tunnel/Bubbleフェーズを含むイベント情報
///
/// # 戻り値
/// - `true`: イベント伝播を停止（stopPropagation相当）
/// - `false`: イベント伝播を継続
fn on_container_pressed(
    world: &mut World,
    sender: Entity,
    entity: Entity,
    ev: &Phase<PointerState>,
) -> bool {
    match ev {
        Phase::Tunnel(state) => {
            // Ctrl+左クリックでイベントを停止
            if state.ctrl_down && state.left_down {
                info!(
                    "[Tunnel] FlexContainer: Event stopped at Container (Ctrl+Left), sender={:?}, entity={:?}, screen=({:.1},{:.1}), local=({:.1},{:.1})",
                    sender, entity,
                    state.screen_point.x, state.screen_point.y,
                    state.local_point.x, state.local_point.y,
                );

                // コンテナの色をピンクに変更
                if let Some(mut brushes) = world.get_mut::<Brushes>(entity) {
                    brushes.foreground = wintf::ecs::widget::brushes::Brush::Solid(D2D1_COLOR_F {
                        r: 1.0,
                        g: 0.4,
                        b: 0.8,
                        a: 1.0,
                    });
                }

                return true; // イベント停止、子に到達しない
            }

            info!(
                "[Tunnel] FlexContainer: Passing through, sender={:?}, entity={:?}",
                sender, entity,
            );
            false
        }
        Phase::Bubble(state) => {
            // 右クリック検出
            if state.right_down {
                info!(
                    "[Bubble] FlexContainer: Right-click detected! sender={:?}, entity={:?}, screen=({:.1},{:.1}), local=({:.1},{:.1})",
                    sender, entity,
                    state.screen_point.x, state.screen_point.y,
                    state.local_point.x, state.local_point.y,
                );

                // コンテナの色をピンクに変更
                if let Some(mut brushes) = world.get_mut::<Brushes>(entity) {
                    brushes.foreground = wintf::ecs::widget::brushes::Brush::Solid(D2D1_COLOR_F {
                        r: 1.0,
                        g: 0.7,
                        b: 0.8,
                        a: 1.0,
                    });
                }

                return true; // イベント処理済み
            }

            false
        }
    }
}

/// RedBox の OnPointerPressed ハンドラ
///
/// 左クリックで色をトグル（赤 ⇔ 黄）する。
/// αマスクヒットテストのデモ: 画像の透明部分をクリックすると
/// イベントが親(RedBox)に伝播してこのハンドラが呼ばれる。
fn on_red_box_pressed(
    world: &mut World,
    sender: Entity,
    entity: Entity,
    ev: &Phase<PointerState>,
) -> bool {
    // Bubble フェーズでのみ処理
    if !ev.is_bubble() {
        info!(
            "[Tunnel] RedBox: Passing through, sender={:?}, entity={:?}",
            sender, entity,
        );
        return false;
    }

    let state = ev.value();

    // 左クリック検出
    if state.left_down {
        info!(
            "[Bubble] RedBox: Left-click, sender={:?}, entity={:?}, screen=({:.1},{:.1}), local=({:.1},{:.1}), L={}, R={}, Ctrl={}",
            sender, entity,
            state.screen_point.x, state.screen_point.y,
            state.local_point.x, state.local_point.y,
            state.left_down, state.right_down, state.ctrl_down,
        );

        // 色をトグル（赤 ⇔ 黄）
        if let Some(mut brushes) = world.get_mut::<Brushes>(entity) {
            let is_red = match brushes.foreground.as_color() {
                Some(c) => c.r > 0.9 && c.g < 0.1,
                None => false,
            };
            if is_red {
                // 黄色に変更
                brushes.foreground = wintf::ecs::widget::brushes::Brush::Solid(D2D1_COLOR_F {
                    r: 1.0,
                    g: 1.0,
                    b: 0.0,
                    a: 1.0,
                });
                info!("[AlphaMask Demo] BACKGROUND clicked (transparent area) - color: RED -> YELLOW");
            } else {
                // 赤に戻す
                brushes.foreground = wintf::ecs::widget::brushes::Brush::Solid(D2D1_COLOR_F {
                    r: 1.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                });
                info!("[AlphaMask Demo] BACKGROUND clicked (transparent area) - color: YELLOW -> RED");
            }
        }

        return true; // イベント処理済み、親に伝播しない
    }

    false
}

/// SeikatuImage の OnPointerPressed ハンドラ
///
/// αマスクヒットテストのデモ用。
/// 不透明部分がクリックされた場合のみこのハンドラが呼ばれる。
/// イベントを消費して親(RedBox)に伝播させない。
fn on_image_pressed(
    _world: &mut World,
    _sender: Entity,
    _entity: Entity,
    ev: &Phase<PointerState>,
) -> bool {
    // Bubble フェーズでのみ処理
    if !ev.is_bubble() {
        return false;
    }

    let state = ev.value();

    // 左クリック検出
    if state.left_down {
        info!(
            "[AlphaMask Demo] IMAGE clicked (opaque area) - event consumed, background unchanged"
        );
        return true; // イベント処理済み、親(RedBox)に伝播しない
    }

    false
}

/// GreenBox の OnPointerPressed ハンドラ
///
/// **Tunnelフェーズ**: 左クリックでキャプチャし、子（GreenBoxChild）に到達させない
/// **Bubbleフェーズ**: 右クリックで色を変更
///
/// # stopPropagation使用例
/// Tunnelフェーズでtrueを返すことで、親エンティティが子のイベント処理前に
/// 介入できます。これはWinUI3/WPFの`PreviewMouseDown`やDOMの`Capture Phase`と
/// 同じ動作です。
///
/// # sender vs entity
/// - `sender`: 常にイベント発生元（例: GreenBoxChild）
/// - `entity`: 現在処理中のエンティティ（この場合はGreenBox）
fn on_green_box_pressed(
    world: &mut World,
    sender: Entity,
    entity: Entity,
    ev: &Phase<PointerState>,
) -> bool {
    match ev {
        Phase::Tunnel(state) => {
            // 左クリックでキャプチャ
            if state.left_down {
                info!(
                    "[Tunnel] GreenBox: Captured event, stopping propagation (Left), sender={:?}, entity={:?}, screen=({:.1},{:.1}), local=({:.1},{:.1})",
                    sender, entity,
                    state.screen_point.x, state.screen_point.y,
                    state.local_point.x, state.local_point.y,
                );

                // 色をトグル（緑 ⇔ 黄緑）
                if let Some(mut brushes) = world.get_mut::<Brushes>(entity) {
                    let is_green = match brushes.foreground.as_color() {
                        Some(c) => c.r < 0.1 && c.g > 0.9,
                        None => false,
                    };
                    if is_green {
                        // 黄緑に変更
                        brushes.foreground = wintf::ecs::widget::brushes::Brush::Solid(D2D1_COLOR_F {
                            r: 0.5,
                            g: 1.0,
                            b: 0.0,
                            a: 1.0,
                        });
                        info!("[Tunnel] GreenBox: Color changed GREEN -> YELLOW-GREEN");
                    } else {
                        // 緑に戻す
                        brushes.foreground = wintf::ecs::widget::brushes::Brush::Solid(D2D1_COLOR_F {
                            r: 0.0,
                            g: 1.0,
                            b: 0.0,
                            a: 1.0,
                        });
                        info!("[Tunnel] GreenBox: Color changed YELLOW-GREEN -> GREEN");
                    }
                }

                return true; // イベント停止、子に到達しない
            }

            info!(
                "[Tunnel] GreenBox: Passing through, sender={:?}, entity={:?}",
                sender, entity,
            );
            false
        }
        Phase::Bubble(state) => {
            // 右クリック処理
            if state.right_down {
                info!(
                    "[Bubble] GreenBox: Right-click, sender={:?}, entity={:?}, screen=({:.1},{:.1}), local=({:.1},{:.1})",
                    sender, entity,
                    state.screen_point.x, state.screen_point.y,
                    state.local_point.x, state.local_point.y,
                );

                // 色を変更
                if let Some(mut brushes) = world.get_mut::<Brushes>(entity) {
                    brushes.foreground = wintf::ecs::widget::brushes::Brush::Solid(D2D1_COLOR_F {
                        r: 0.0,
                        g: 0.8,
                        b: 0.8,
                        a: 1.0,
                    });
                }

                return true;
            }

            false
        }
    }
}

/// GreenBoxChild の OnPointerPressed ハンドラ
///
/// 親（GreenBox）がTunnelでキャプチャした場合、このハンドラは呼ばれない。
/// 右クリック時は親がキャプチャしないため、Tunnel/Bubble両方で実行される。
///
/// # ev.value()の使用例
/// `Phase::Tunnel(state)` や `Phase::Bubble(state)` でパターンマッチする代わりに、
/// `ev.value()` で `PointerState` を直接取得できます。
fn on_green_child_pressed(
    world: &mut World,
    sender: Entity,
    entity: Entity,
    ev: &Phase<PointerState>,
) -> bool {
    let state = ev.value();

    match ev {
        Phase::Tunnel(_) => {
            info!(
                "[Tunnel] GreenBoxChild: This should NOT be called if parent captured (Left), sender={:?}, entity={:?}, screen=({:.1},{:.1}), local=({:.1},{:.1}), L={}, R={}, Ctrl={}",
                sender, entity,
                state.screen_point.x, state.screen_point.y,
                state.local_point.x, state.local_point.y,
                state.left_down, state.right_down, state.ctrl_down,
            );
            false
        }
        Phase::Bubble(_) => {
            // 右クリック処理
            if state.right_down {
                info!(
                    "[Bubble] GreenBoxChild: Right-click detected, changing to orange, sender={:?}, entity={:?}, screen=({:.1},{:.1}), local=({:.1},{:.1})",
                    sender, entity,
                    state.screen_point.x, state.screen_point.y,
                    state.local_point.x, state.local_point.y,
                );

                // 色をオレンジに変更
                if let Some(mut brushes) = world.get_mut::<Brushes>(entity) {
                    brushes.foreground = wintf::ecs::widget::brushes::Brush::Solid(D2D1_COLOR_F {
                        r: 1.0,
                        g: 0.5,
                        b: 0.0,
                        a: 1.0,
                    });
                }

                return true;
            }

            false
        }
    }
}

/// GreenBox の OnPointerMoved ハンドラ
///
/// マウス移動時にログを出力する（デバッグ用）。
fn on_green_box_moved(
    _world: &mut World,
    sender: Entity,
    entity: Entity,
    ev: &Phase<PointerState>,
) -> bool {
    // Bubble フェーズでのみ処理（Tunnel でログ出力すると冗長）
    if !ev.is_bubble() {
        return false;
    }

    let state = ev.value();

    // 10フレームに1回程度ログ出力（頻繁すぎないように）
    static MOVE_COUNT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
    let count = MOVE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    if count % 30 == 0 {
        info!(
            sender = ?sender,
            entity = ?entity,
            x = state.screen_point.x,
            y = state.screen_point.y,
            "[Bubble] GreenBox: Pointer moved"
        );
    }

    false // 伝播続行（親にも通知）
}

/// BlueBox の OnPointerPressed ハンドラ
///
/// 左クリックでサイズをトグル（100 ⇔ 150）する。
fn on_blue_box_pressed(
    world: &mut World,
    sender: Entity,
    entity: Entity,
    ev: &Phase<PointerState>,
) -> bool {
    // Bubble フェーズでのみ処理
    if !ev.is_bubble() {
        info!(
            "[Tunnel] BlueBox: Passing through, sender={:?}, entity={:?}",
            sender, entity,
        );
        return false;
    }

    let state = ev.value();

    // 左クリック検出
    if state.left_down {
        info!(
            "[Bubble] BlueBox: Left-click detected! Toggling size, sender={:?}, entity={:?}, screen=({:.1},{:.1}), local=({:.1},{:.1}), L={}, R={}, Ctrl={}",
            sender, entity,
            state.screen_point.x, state.screen_point.y,
            state.local_point.x, state.local_point.y,
            state.left_down, state.right_down, state.ctrl_down,
        );

        // サイズをトグル
        if let Some(mut style) = world.get_mut::<BoxStyle>(entity) {
            if let Some(ref mut size) = style.size {
                let new_size = if size.width == Some(Dimension::Px(100.0)) {
                    150.0
                } else {
                    100.0
                };
                size.width = Some(Dimension::Px(new_size));
                size.height = Some(Dimension::Px(new_size));
                info!(new_size = new_size, "[PointerEvent] BlueBox: New size");
            }
        }

        return true; // イベント処理済み
    }

    false
}
