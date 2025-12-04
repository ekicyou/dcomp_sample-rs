//! Typewriter システム実装
//!
//! - init_typewriter_layout: TypewriterTalk追加時にLayoutCache生成（Draw スケジュール）
//! - update_typewriters: 毎フレームの状態更新（Update スケジュール）
//! - draw_typewriters: 描画コマンド生成（Draw スケジュール）

use crate::com::d2d::{D2D1CommandListExt, D2D1DeviceContextExt};
use crate::com::dwrite::{DWriteFactoryExt, DWriteTextLayoutExt};
use crate::ecs::graphics::{FrameTime, GraphicsCommandList, GraphicsCore};
use crate::ecs::layout::Arrangement;
use crate::ecs::widget::shapes::rectangle::colors;
use crate::ecs::widget::text::label::TextDirection;
use crate::ecs::widget::text::typewriter::{
    Typewriter, TypewriterLayoutCache, TypewriterState, TypewriterTalk,
};
use crate::ecs::widget::text::typewriter_ir::{
    TimelineItem, TypewriterEvent, TypewriterTimeline, TypewriterToken,
};
use crate::ecs::TextLayoutMetrics;
use bevy_ecs::prelude::*;
use tracing::{debug, trace, warn};
use windows::Win32::Graphics::Direct2D::D2D1_DRAW_TEXT_OPTIONS_NONE;
use windows::Win32::Graphics::DirectWrite::*;
use windows_numerics::Vector2;

/// TypewriterLayoutCache 無効化システム（Draw スケジュール）
///
/// Arrangementが変更されたらLayoutCacheを削除し、再生成をトリガーする。
pub fn invalidate_typewriter_layout_on_arrangement_change(
    mut commands: Commands,
    query: Query<Entity, (With<TypewriterLayoutCache>, Changed<Arrangement>)>,
) {
    for entity in query.iter() {
        debug!(entity = ?entity, "[invalidate_typewriter_layout] Arrangement changed, removing LayoutCache");
        commands.entity(entity).remove::<TypewriterLayoutCache>();
    }
}

/// TypewriterLayoutCache 初期化システム（Draw スケジュール）
///
/// TypewriterTalk が追加されたが TypewriterLayoutCache がないエンティティに対して、
/// TextLayout を作成し、Stage 2 IR に変換して LayoutCache を生成する。
pub fn init_typewriter_layout(
    mut commands: Commands,
    query: Query<
        (Entity, &Typewriter, &TypewriterTalk, &Arrangement),
        Without<TypewriterLayoutCache>,
    >,
    graphics_core: Option<Res<GraphicsCore>>,
) {
    debug!("[init_typewriter_layout] Running, query count: {}", query.iter().count());
    
    let Some(graphics_core) = graphics_core else {
        debug!("[init_typewriter_layout] No GraphicsCore");
        return;
    };

    let Some(dwrite_factory) = graphics_core.dwrite_factory() else {
        warn!("DirectWrite factory not available");
        return;
    };

    for (entity, typewriter, talk, arrangement) in query.iter() {
        // トークン列から全文テキストを結合
        let full_text: String = talk
            .tokens()
            .iter()
            .filter_map(|t| match t {
                TypewriterToken::Text(s) => Some(s.as_str()),
                _ => None,
            })
            .collect();

        if full_text.is_empty() {
            continue;
        }

        // Arrangementからサイズを取得
        let layout_width = arrangement.size.width;
        let layout_height = arrangement.size.height;

        // Arrangementがまだ計算されていない場合（極小値）は次フレームに委ねる
        // レイアウト計算はフレーム境界で適用されるため、初回フレームでは初期値の可能性がある
        const MIN_LAYOUT_SIZE: f32 = 10.0;
        if layout_width < MIN_LAYOUT_SIZE || layout_height < MIN_LAYOUT_SIZE {
            trace!(
                entity = ?entity,
                layout_width = layout_width,
                layout_height = layout_height,
                "[init_typewriter_layout] Arrangement too small, deferring to next frame"
            );
            continue;
        }

        debug!(
            entity = ?entity,
            layout_width = layout_width,
            layout_height = layout_height,
            "[init_typewriter_layout] Arrangement size"
        );

        // TextFormat 作成
        let font_family_hstring = windows::core::HSTRING::from(&typewriter.font_family);
        let locale_hstring = windows::core::HSTRING::from("ja-JP");
        let text_format = match dwrite_factory.create_text_format(
            &font_family_hstring,
            None::<&IDWriteFontCollection>,
            DWRITE_FONT_WEIGHT_NORMAL,
            DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_STRETCH_NORMAL,
            typewriter.font_size,
            &locale_hstring,
        ) {
            Ok(fmt) => fmt,
            Err(e) => {
                warn!(entity = ?entity, error = ?e, "Failed to create TextFormat");
                continue;
            }
        };

        // 縦書き・横書きの設定
        unsafe {
            match typewriter.direction {
                TextDirection::HorizontalLeftToRight => {
                    let _ = text_format.SetReadingDirection(DWRITE_READING_DIRECTION_LEFT_TO_RIGHT);
                    let _ = text_format.SetFlowDirection(DWRITE_FLOW_DIRECTION_TOP_TO_BOTTOM);
                }
                TextDirection::HorizontalRightToLeft => {
                    let _ = text_format.SetReadingDirection(DWRITE_READING_DIRECTION_RIGHT_TO_LEFT);
                    let _ = text_format.SetFlowDirection(DWRITE_FLOW_DIRECTION_TOP_TO_BOTTOM);
                }
                TextDirection::VerticalRightToLeft => {
                    let _ = text_format.SetReadingDirection(DWRITE_READING_DIRECTION_TOP_TO_BOTTOM);
                    let _ = text_format.SetFlowDirection(DWRITE_FLOW_DIRECTION_RIGHT_TO_LEFT);
                }
                TextDirection::VerticalLeftToRight => {
                    let _ = text_format.SetReadingDirection(DWRITE_READING_DIRECTION_TOP_TO_BOTTOM);
                    let _ = text_format.SetFlowDirection(DWRITE_FLOW_DIRECTION_LEFT_TO_RIGHT);
                }
            }
        }

        // TextLayout 作成（Arrangementのサイズを使用）
        let text_hstring = windows::core::HSTRING::from(&full_text);
        let text_layout = match dwrite_factory.create_text_layout(
            &text_hstring,
            &text_format,
            layout_width,
            layout_height,
        ) {
            Ok(layout) => layout,
            Err(e) => {
                warn!(entity = ?entity, error = ?e, "Failed to create TextLayout");
                continue;
            }
        };

        // Stage 1 → Stage 2 IR 変換
        let timeline = match convert_to_timeline(talk.tokens(), typewriter, &text_layout) {
            Ok(tl) => tl,
            Err(e) => {
                warn!(entity = ?entity, error = ?e, "Failed to convert timeline");
                continue;
            }
        };

        trace!(
            entity = ?entity,
            total_cluster_count = timeline.total_cluster_count,
            total_duration = timeline.total_duration,
            "[init_typewriter_layout] LayoutCache created"
        );

        // TypewriterLayoutCache を挿入
        commands
            .entity(entity)
            .insert(TypewriterLayoutCache::new(text_layout, timeline));
    }
}

/// Stage 1 → Stage 2 IR 変換
fn convert_to_timeline(
    tokens: &[TypewriterToken],
    typewriter: &Typewriter,
    text_layout: &IDWriteTextLayout,
) -> windows::core::Result<TypewriterTimeline> {
    let cluster_metrics = text_layout.get_cluster_metrics()?;
    let total_cluster_count = cluster_metrics.len() as u32;

    let mut full_text = String::new();
    let mut items = Vec::new();
    let mut current_time = 0.0;
    let mut cluster_index = 0u32;

    for token in tokens {
        match token {
            TypewriterToken::Text(text) => {
                full_text.push_str(text);

                let char_count = text.chars().count();
                for _ in 0..char_count {
                    if cluster_index < total_cluster_count {
                        current_time += typewriter.default_char_wait;
                        items.push(TimelineItem::Glyph {
                            cluster_index,
                            show_at: current_time,
                        });
                        cluster_index += 1;
                    }
                }
            }
            TypewriterToken::Wait(duration) => {
                items.push(TimelineItem::Wait {
                    duration: *duration,
                    start_at: current_time,
                });
                current_time += duration;
            }
            TypewriterToken::FireEvent { target, event } => {
                items.push(TimelineItem::FireEvent {
                    target: *target,
                    event: event.clone(),
                    fire_at: current_time,
                });
            }
        }
    }

    Ok(TypewriterTimeline {
        full_text,
        items,
        total_duration: current_time,
        total_cluster_count,
    })
}

/// Typewriter 更新システム（Update スケジュール）
///
/// FrameTime から現在時刻を取得し、TypewriterTalk の状態を更新する。
/// FireEvent トークンを処理して対象エンティティの TypewriterEvent を設定する。
pub fn update_typewriters(
    mut commands: Commands,
    frame_time: Option<Res<FrameTime>>,
    mut query: Query<(Entity, &mut TypewriterTalk, &TypewriterLayoutCache)>,
) {
    let Some(frame_time) = frame_time else {
        return;
    };

    let current_time = frame_time.elapsed_secs();

    for (entity, mut talk, layout_cache) in query.iter_mut() {
        // 状態を更新し、発火すべきイベントを取得
        let events = talk.update(current_time, layout_cache.timeline());

        // FireEvent 処理: 対象エンティティの TypewriterEvent を設定
        for (target, event_kind) in events {
            commands
                .entity(target)
                .insert(TypewriterEvent::from(event_kind));
        }

        // 完了した場合のログ
        if talk.state() == TypewriterState::Completed {
            #[cfg(debug_assertions)]
            debug!(entity = ?entity, "Typewriter talk completed");
        }
    }
}

/// Typewriter 描画システム（Draw スケジュール）
///
/// TypewriterLayoutCache から TextLayout を取得し、
/// visible_cluster_count までのグリフを描画する。
pub fn draw_typewriters(
    mut commands: Commands,
    query: Query<(Entity, &Typewriter, &TypewriterTalk, &TypewriterLayoutCache)>,
    graphics_core: Option<Res<GraphicsCore>>,
) {
    let Some(graphics_core) = graphics_core else {
        warn!("GraphicsCore not available, skipping draw_typewriters");
        return;
    };

    let Some(dc) = graphics_core.device_context() else {
        warn!("DeviceContext not available");
        return;
    };

    for (entity, typewriter, talk, layout_cache) in query.iter() {
        let text_layout = layout_cache.text_layout();
        let timeline = layout_cache.timeline();
        let visible_count = talk.visible_cluster_count();

        #[cfg(debug_assertions)]
        debug!(
            entity = ?entity,
            visible_count = visible_count,
            progress = talk.progress(),
            "Drawing typewriter"
        );

        // クラスタメトリクス取得（描画範囲決定用）
        let cluster_metrics = match text_layout.get_cluster_metrics() {
            Ok(m) => m,
            Err(e) => {
                warn!(entity = ?entity, error = ?e, "Failed to get cluster metrics");
                continue;
            }
        };

        // visible_count までのテキスト位置を計算
        let visible_text_length: u32 = cluster_metrics
            .iter()
            .take(visible_count as usize)
            .map(|m| m.length as u32)
            .sum();

        // テキストメトリクス取得
        let mut text_metrics = DWRITE_TEXT_METRICS::default();
        unsafe {
            let _ = text_layout.GetMetrics(&mut text_metrics);
        }

        // CommandList生成
        let command_list = match unsafe { dc.CreateCommandList() } {
            Ok(cl) => cl,
            Err(err) => {
                warn!(entity = ?entity, error = ?err, "Failed to create CommandList");
                continue;
            }
        };

        // CommandListをターゲットに設定
        unsafe {
            dc.SetTarget(&command_list);
        }

        // 描画開始
        unsafe {
            dc.BeginDraw();
        }

        // 透明でクリア
        dc.clear(Some(&colors::TRANSPARENT));

        // バックグラウンド描画（指定されている場合）
        if let Some(ref bg_color) = typewriter.background {
            if let Ok(bg_brush) = dc.create_solid_color_brush(bg_color, None) {
                let bg_rect = windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F {
                    left: 0.0,
                    top: 0.0,
                    right: text_metrics.layoutWidth,
                    bottom: text_metrics.layoutHeight,
                };
                unsafe {
                    dc.FillRectangle(&bg_rect, &bg_brush);
                }
            }
        }

        // フォアグラウンドブラシ作成
        let brush = match dc.create_solid_color_brush(&typewriter.foreground, None) {
            Ok(b) => b,
            Err(err) => {
                warn!(entity = ?entity, error = ?err, "Failed to create brush");
                unsafe {
                    let _ = dc.EndDraw(None, None);
                }
                continue;
            }
        };

        // visible_count > 0 の場合のみ描画
        if visible_count > 0 && visible_text_length > 0 {
            // 描画位置の調整
            // DirectWriteはレイアウトボックス(layoutWidth x layoutHeight)を基準に描画する
            //
            // 横書きLTR: 原点はレイアウトボックス左上、テキストは右へ流れる
            //   → left >= 0, origin.X = -left で左端を合わせる
            //
            // 縦書きRTL: 原点はレイアウトボックス左上、1行目はレイアウトボックスの右端から始まり左へ流れる
            //   → テキストがはみ出すとleft < 0になる
            //   → Surfaceはtext_metrics.width幅で作成される
            //   → レイアウトボックスの右端をSurfaceの右端に合わせるには:
            //     origin.X = text_metrics.width - layoutWidth
            let origin = match typewriter.direction {
                TextDirection::VerticalRightToLeft => {
                    // 縦書きRTL: レイアウトボックスの右端をSurface右端に合わせる
                    Vector2 {
                        X: text_metrics.width - text_metrics.layoutWidth,
                        Y: -text_metrics.top,
                    }
                }
                TextDirection::VerticalLeftToRight => {
                    // 縦書きLTR: 1行目は左端から始まる
                    Vector2 {
                        X: -text_metrics.left,
                        Y: -text_metrics.top,
                    }
                }
                _ => {
                    // 横書き: 従来通り
                    Vector2 {
                        X: -text_metrics.left,
                        Y: -text_metrics.top,
                    }
                }
            };

            // 非表示部分を透明にするため、visible_text_length以降に透明ブラシを設定
            let total_text_length = timeline.full_text.chars().count() as u32;

            if visible_text_length < total_text_length {
                // 透明ブラシ作成
                let transparent_brush =
                    match dc.create_solid_color_brush(&colors::TRANSPARENT, None) {
                        Ok(b) => b,
                        Err(_) => {
                            // フォールバック: 全文描画
                            dc.draw_text_layout(
                                origin,
                                text_layout,
                                &brush,
                                D2D1_DRAW_TEXT_OPTIONS_NONE,
                            );
                            unsafe {
                                let _ = dc.EndDraw(None, None);
                            }
                            continue;
                        }
                    };

                // 非表示範囲に透明ブラシを設定
                let hidden_range = DWRITE_TEXT_RANGE {
                    startPosition: visible_text_length,
                    length: total_text_length - visible_text_length,
                };
                unsafe {
                    let _ = text_layout.SetDrawingEffect(&transparent_brush, hidden_range);
                }
            }

            // 全文を描画（非表示部分は透明ブラシで描画される）
            dc.draw_text_layout(origin, text_layout, &brush, D2D1_DRAW_TEXT_OPTIONS_NONE);

            // 設定をリセット（次回描画のため）
            if visible_text_length < total_text_length {
                let hidden_range = DWRITE_TEXT_RANGE {
                    startPosition: visible_text_length,
                    length: total_text_length - visible_text_length,
                };
                unsafe {
                    let _ = text_layout.SetDrawingEffect(None, hidden_range);
                }
            }
        }

        // 描画終了
        if let Err(err) = unsafe { dc.EndDraw(None, None) } {
            warn!(entity = ?entity, error = ?err, "EndDraw failed");
            continue;
        }

        // CommandListを閉じる
        if let Err(err) = command_list.close() {
            warn!(entity = ?entity, error = ?err, "Failed to close CommandList");
            continue;
        }

        // GraphicsCommandList と TextLayoutMetrics をエンティティに挿入
        commands.entity(entity).insert((
            GraphicsCommandList::new(command_list),
            TextLayoutMetrics {
                width: text_metrics.width,
                height: text_metrics.height,
            },
        ));
    }
}
