#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

//! Typewriter デモ
//!
//! タイプライター効果のデモンストレーション:
//! - 文字を一文字ずつ表示
//! - ウェイト制御
//! - pause/resume/skip 操作
//! - FireEvent による完了イベント

use bevy_ecs::name::Name;
use bevy_ecs::prelude::*;
use std::time::Duration;
use tracing_subscriber::EnvFilter;
use windows::core::Result;
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;
use wintf::ecs::layout::{
    BoxInset, BoxMargin, BoxPosition, BoxSize, BoxStyle, Dimension, LengthPercentageAuto,
};
use wintf::ecs::widget::bitmap_source::CommandSender;
use wintf::ecs::widget::shapes::Rectangle;
use wintf::ecs::widget::text::{
    TextDirection, Typewriter, TypewriterEvent, TypewriterEventKind, TypewriterLayoutCache,
    TypewriterTalk, TypewriterToken,
};
use wintf::ecs::{FrameTime, Window};
use wintf::*;

#[derive(Debug, Clone, Copy, Component, PartialEq, Hash)]
pub struct TypewriterDemoWindow;

/// 横書き Typewriter Entity を識別するマーカー
#[derive(Debug, Clone, Copy, Component, PartialEq, Hash)]
pub struct HorizontalTypewriter;

/// 縦書き Typewriter Entity を識別するマーカー
#[derive(Debug, Clone, Copy, Component, PartialEq, Hash)]
pub struct VerticalTypewriter;

/// 完了イベント受信用エンティティ
#[derive(Debug, Clone, Copy, Component, PartialEq, Hash)]
pub struct CompletionReceiver;

fn main() -> Result<()> {
    human_panic::setup_panic!();

    // tracing-subscriber 初期化
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let mgr = WinThreadMgr::new()?;
    let world = mgr.world();

    // 非同期タスクでデモを実行
    world.borrow().spawn(|tx| async move {
        run_demo(tx).await;
    });

    println!("\nTypewriter デモ:");
    println!("  1. ウィンドウ作成とTypewriterエンティティ生成");
    println!("  2. Stage 1 IR でテキストを設定");
    println!("  3. 一文字ずつ表示（タイプライター効果）");
    println!("  4. 5秒後: pause()");
    println!("  5. 7秒後: resume()");
    println!("  6. 9秒後: skip()（全文即時表示）");
    println!("  7. 15秒後: ウィンドウ終了");

    // メッセージループを開始
    mgr.run()?;

    Ok(())
}

/// 非同期デモ実行
async fn run_demo(tx: CommandSender) {
    // === 0秒: ウィンドウ作成 ===
    println!("[Async] 0s: Creating typewriter demo window");
    let _ = tx.send(Box::new(create_typewriter_demo_window));

    // === 1秒待機 ===
    async_io::Timer::after(Duration::from_secs(1)).await;

    // === 1秒: Typewriter トークを開始 ===
    println!("[Async] 1s: Starting typewriter talk");
    let _ = tx.send(Box::new(start_typewriter_talk));

    // === 4秒待機 ===
    async_io::Timer::after(Duration::from_secs(4)).await;

    // === 5秒: 一時停止 ===
    println!("[Async] 5s: Pausing typewriter");
    let _ = tx.send(Box::new(pause_typewriter));

    // === 2秒待機 ===
    async_io::Timer::after(Duration::from_secs(2)).await;

    // === 7秒: 再開 ===
    println!("[Async] 7s: Resuming typewriter");
    let _ = tx.send(Box::new(resume_typewriter));

    // === 2秒待機 ===
    async_io::Timer::after(Duration::from_secs(2)).await;

    // === 9秒: スキップ ===
    println!("[Async] 9s: Skipping to end");
    let _ = tx.send(Box::new(skip_typewriter));

    // === 6秒待機 ===
    async_io::Timer::after(Duration::from_secs(6)).await;

    // === 15秒: ウィンドウ終了 ===
    println!("[Async] 15s: Closing window");
    let _ = tx.send(Box::new(close_window));
}

/// ウィンドウを閉じる
fn close_window(world: &mut World) {
    let mut query = world.query_filtered::<Entity, With<TypewriterDemoWindow>>();
    if let Some(window) = query.iter(world).next() {
        println!("[Test] Removing Window entity {:?}", window);
        world.despawn(window);
    }
}

/// Typewriterデモウィンドウを作成
fn create_typewriter_demo_window(world: &mut World) {
    // Window Entity (ルート)
    let window_entity = world
        .spawn((
            Name::new("TypewriterDemo-Window"),
            TypewriterDemoWindow,
            BoxStyle {
                position: Some(BoxPosition::Absolute),
                inset: Some(BoxInset(wintf::ecs::layout::Rect {
                    left: LengthPercentageAuto::Px(100.0),
                    top: LengthPercentageAuto::Px(100.0),
                    right: LengthPercentageAuto::Auto,
                    bottom: LengthPercentageAuto::Auto,
                })),
                size: Some(BoxSize {
                    width: Some(Dimension::Px(500.0)),
                    height: Some(Dimension::Px(500.0)),
                }),
                ..Default::default()
            },
            Window {
                title: "wintf - Typewriter Demo".to_string(),
                ..Default::default()
            },
        ))
        .id();

    // 背景矩形（灰色）
    let background = world
        .spawn((
            Name::new("Background"),
            Rectangle {
                color: D2D1_COLOR_F {
                    r: 0.95,
                    g: 0.95,
                    b: 0.95,
                    a: 1.0,
                },
            },
            BoxStyle {
                // size 100% + margin は親をはみ出すため、flex_grow を使用
                flex_grow: Some(1.0),
                margin: Some(BoxMargin(wintf::ecs::layout::Rect {
                    left: LengthPercentageAuto::Px(20.0),
                    right: LengthPercentageAuto::Px(20.0),
                    top: LengthPercentageAuto::Px(20.0),
                    bottom: LengthPercentageAuto::Px(20.0),
                })),
                flex_direction: Some(wintf::ecs::layout::FlexDirection::Column),
                ..Default::default()
            },
            ChildOf(window_entity),
        ))
        .id();

    // 横書き用の内側ボックス（薄い青）
    let horizontal_box = world
        .spawn((
            Name::new("HorizontalBox"),
            Rectangle {
                color: D2D1_COLOR_F {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
            },
            BoxStyle {
                size: Some(BoxSize {
                    width: None,
                    height: Some(Dimension::Px(80.0)),
                }),
                margin: Some(BoxMargin(wintf::ecs::layout::Rect {
                    left: LengthPercentageAuto::Px(10.0),
                    right: LengthPercentageAuto::Px(10.0),
                    top: LengthPercentageAuto::Px(10.0),
                    bottom: LengthPercentageAuto::Px(10.0),
                })),
                ..Default::default()
            },
            ChildOf(background),
        ))
        .id();

    // 横書き Typewriter エンティティ
    // NOTE: Typewriterは描画コンテンツサイズを自己申告しないため、
    // 親のサイズに合わせて固定サイズで指定する必要がある
    // NOTE: 空のTypewriterTalkを初期設定することで、会話開始前も背景が描画される
    world.spawn((
        Name::new("HorizontalTypewriter"),
        HorizontalTypewriter,
        Typewriter {
            font_family: "メイリオ".to_string(),
            font_size: 18.0,
            foreground: D2D1_COLOR_F {
                r: 0.1,
                g: 0.1,
                b: 0.1,
                a: 1.0,
            },
            background: Some(D2D1_COLOR_F {
                r: 0.9,
                g: 0.9,
                b: 1.0,
                a: 1.0,
            }),
            direction: TextDirection::HorizontalLeftToRight,
            default_char_wait: 0.08,
            ..Default::default()
        },
        BoxStyle {
            // size 100% + margin は親をはみ出すため、flex_grow を使用
            flex_grow: Some(1.0),
            margin: Some(BoxMargin(wintf::ecs::layout::Rect {
                left: LengthPercentageAuto::Px(1.0),
                right: LengthPercentageAuto::Px(1.0),
                top: LengthPercentageAuto::Px(1.0),
                bottom: LengthPercentageAuto::Px(1.0),
            })),
            ..Default::default()
        },
        ChildOf(horizontal_box),
    ));

    // 縦書き用の内側ボックス（薄い緑）
    let vertical_box = world
        .spawn((
            Name::new("VerticalBox"),
            Rectangle {
                color: D2D1_COLOR_F {
                    r: 0.0,
                    g: 0.0,
                    b: 0.0,
                    a: 1.0,
                },
            },
            BoxStyle {
                size: Some(BoxSize {
                    width: Some(Dimension::Px(80.0)),
                    height: None, // 高さは flex_grow で自動調整
                }),
                flex_grow: Some(1.0), // 残りの空間を埋める
                margin: Some(BoxMargin(wintf::ecs::layout::Rect {
                    left: LengthPercentageAuto::Px(10.0),
                    right: LengthPercentageAuto::Px(10.0),
                    top: LengthPercentageAuto::Px(10.0),
                    bottom: LengthPercentageAuto::Px(10.0),
                })),
                align_self: Some(wintf::ecs::layout::AlignSelf::FlexEnd), // 右寄せ
                ..Default::default()
            },
            ChildOf(background),
        ))
        .id();

    // 縦書き Typewriter エンティティ
    // NOTE: Typewriterは描画コンテンツサイズを自己申告しないため、
    // 親のサイズに合わせて固定サイズで指定する必要がある
    // NOTE: 空のTypewriterTalkを初期設定することで、会話開始前も背景が描画される
    world.spawn((
        Name::new("VerticalTypewriter"),
        VerticalTypewriter,
        Typewriter {
            font_family: "メイリオ".to_string(),
            font_size: 18.0,
            foreground: D2D1_COLOR_F {
                r: 0.1,
                g: 0.1,
                b: 0.5,
                a: 1.0,
            },
            background: Some(D2D1_COLOR_F {
                r: 1.0,
                g: 1.0,
                b: 0.9,
                a: 1.0,
            }),
            direction: TextDirection::VerticalRightToLeft,
            default_char_wait: 0.08,
            ..Default::default()
        },
        TypewriterTalk::new(vec![], 0.0), // 空のトークで初期化（背景描画用）
        BoxStyle {
            // size 100% + margin は親をはみ出すため、flex_grow を使用
            size: Some(BoxSize {
                width: Some(Dimension::Px(85.0)), // 3行分（テキスト幅81px + 余白）
                height: None,
            }),
            flex_grow: Some(1.0),
            margin: Some(BoxMargin(wintf::ecs::layout::Rect {
                left: LengthPercentageAuto::Px(1.0),
                right: LengthPercentageAuto::Px(1.0),
                top: LengthPercentageAuto::Px(1.0),
                bottom: LengthPercentageAuto::Px(1.0),
            })),
            ..Default::default()
        },
        ChildOf(vertical_box),
    ));

    // 完了イベント受信用エンティティ
    world.spawn((
        Name::new("CompletionReceiver"),
        CompletionReceiver,
        TypewriterEvent::None,
    ));

    println!("[Test] Typewriter demo window created:");
    println!("  Window (root)");
    println!("  └─ Background (gray)");
    println!("     ├─ HorizontalTypewriter (横書き)");
    println!("     └─ VerticalTypewriter (縦書き)");
}

/// 両方のTypewriterにトークを開始
fn start_typewriter_talk(world: &mut World) {
    // 完了イベント受信用エンティティを取得
    let mut receiver_query = world.query_filtered::<Entity, With<CompletionReceiver>>();
    let Some(receiver_entity) = receiver_query.iter(world).next() else {
        println!("[Error] CompletionReceiver entity not found");
        return;
    };

    // FrameTime から現在時刻を取得
    let current_time = {
        let Some(frame_time) = world.get_resource::<FrameTime>() else {
            println!("[Error] FrameTime not found");
            return;
        };
        frame_time.elapsed_secs()
    };

    // Stage 1 IR トークン列を作成
    let tokens = vec![
        TypewriterToken::Text("こんにちは、".to_string()),
        TypewriterToken::Wait(0.3),
        TypewriterToken::Text("今日もいい天気ですね。".to_string()),
        TypewriterToken::Wait(0.5),
        TypewriterToken::Text("タイプライター効果のデモです！".to_string()),
        TypewriterToken::FireEvent {
            target: receiver_entity,
            event: TypewriterEventKind::Complete,
        },
    ];

    // 横書きTypewriterにTalkを挿入
    let mut h_query = world.query_filtered::<Entity, With<HorizontalTypewriter>>();
    if let Some(entity) = h_query.iter(world).next() {
        let talk = TypewriterTalk::new(tokens.clone(), current_time);
        world.entity_mut(entity).insert(talk);
        println!("[Test] HorizontalTypewriter talk started");
    }

    // 縦書きTypewriterにTalkを挿入
    let mut v_query = world.query_filtered::<Entity, With<VerticalTypewriter>>();
    if let Some(entity) = v_query.iter(world).next() {
        let talk = TypewriterTalk::new(tokens, current_time);
        world.entity_mut(entity).insert(talk);
        println!("[Test] VerticalTypewriter talk started");
    }

    println!("[Test] TypewriterTalk started with text:");
    println!("  \"こんにちは、（wait 300ms）今日もいい天気ですね。（wait 500ms）タイプライター効果のデモです！\"");
}

/// Typewriter を一時停止
fn pause_typewriter(world: &mut World) {
    let Some(frame_time) = world.get_resource::<FrameTime>() else {
        println!("[Error] FrameTime not found");
        return;
    };
    let current_time = frame_time.elapsed_secs();

    let mut query = world.query::<&mut TypewriterTalk>();
    for mut talk in query.iter_mut(world) {
        let progress_before = talk.progress();
        talk.pause(current_time);
        println!(
            "[Test] Typewriter paused at progress: {:.1}%",
            progress_before * 100.0
        );
    }
}

/// Typewriter を再開
fn resume_typewriter(world: &mut World) {
    let Some(frame_time) = world.get_resource::<FrameTime>() else {
        println!("[Error] FrameTime not found");
        return;
    };
    let current_time = frame_time.elapsed_secs();

    let mut query = world.query::<&mut TypewriterTalk>();
    for mut talk in query.iter_mut(world) {
        talk.resume(current_time);
        println!(
            "[Test] Typewriter resumed at progress: {:.1}%",
            talk.progress() * 100.0
        );
    }
}

/// Typewriter をスキップ
fn skip_typewriter(world: &mut World) {
    // LayoutCache から total_cluster_count を取得して skip に渡す
    let mut query = world.query::<(&mut TypewriterTalk, &TypewriterLayoutCache)>();
    for (mut talk, layout_cache) in query.iter_mut(world) {
        let total = layout_cache.timeline().total_cluster_count;
        talk.skip(total);
        println!("[Test] Typewriter skipped to end (progress: 100%)");
    }
}
