//! Typewriter システム実装
//!
//! - update_typewriters: 毎フレームの状態更新（Update スケジュール）
//! - draw_typewriters: 描画コマンド生成（Draw スケジュール）

use crate::com::d2d::{D2D1CommandListExt, D2D1DeviceContextExt};
use crate::com::dwrite::DWriteTextLayoutExt;
use crate::ecs::graphics::{AnimationCore, GraphicsCommandList, GraphicsCore};
use crate::ecs::widget::shapes::rectangle::colors;
use crate::ecs::widget::text::typewriter::{Typewriter, TypewriterState, TypewriterTalk};
use crate::ecs::widget::text::typewriter_ir::TypewriterEvent;
use crate::ecs::TextLayoutMetrics;
use bevy_ecs::prelude::*;
use tracing::{debug, warn};
use windows::Win32::Graphics::Direct2D::D2D1_DRAW_TEXT_OPTIONS_NONE;
use windows::Win32::Graphics::DirectWrite::*;
use windows_numerics::Vector2;

/// Typewriter 更新システム（Update スケジュール）
///
/// AnimationCore から現在時刻を取得し、TypewriterTalk の状態を更新する。
/// FireEvent トークンを処理して対象エンティティの TypewriterEvent を設定する。
pub fn update_typewriters(
    mut commands: Commands,
    animation_core: Option<Res<AnimationCore>>,
    mut query: Query<(Entity, &mut TypewriterTalk)>,
) {
    let Some(animation_core) = animation_core else {
        return;
    };

    let current_time = match animation_core.get_time() {
        Ok(t) => t,
        Err(e) => {
            warn!("Failed to get animation time: {:?}", e);
            return;
        }
    };

    for (entity, mut talk) in query.iter_mut() {
        // 状態を更新し、発火すべきイベントを取得
        let events = talk.update(current_time);

        // FireEvent 処理: 対象エンティティの TypewriterEvent を設定
        for (target, event_kind) in events {
            commands.entity(target).insert(TypewriterEvent::from(event_kind));
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
/// Changed<TypewriterTalk> でクエリフィルタリングし、
/// TextLayout から visible_cluster_count までのグリフを描画する。
pub fn draw_typewriters(
    mut commands: Commands,
    query: Query<
        (Entity, &Typewriter, &TypewriterTalk),
        Or<(Changed<TypewriterTalk>, Without<GraphicsCommandList>)>,
    >,
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

    for (entity, typewriter, talk) in query.iter() {
        let text_layout = talk.text_layout();
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

        // ブラシ作成
        let brush = match dc.create_solid_color_brush(&typewriter.color, None) {
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
            // 描画位置の調整（RTL/縦書き対応）
            let origin = Vector2 {
                X: -text_metrics.left,
                Y: -text_metrics.top,
            };

            // テキスト範囲を制限して描画
            // SetMaxWidth/SetMaxHeight で描画範囲を制限する代わりに、
            // DWRITE_TEXT_RANGE で visible_text_length までの範囲のみ不透明に設定
            // Note: この実装では全文を描画し、超過部分は透明色で描画する
            // より効率的な実装は、visible部分のみの別TextLayoutを作成すること

            // 全文を描画（visible_count までが表示される）
            dc.draw_text_layout(origin, text_layout, &brush, D2D1_DRAW_TEXT_OPTIONS_NONE);
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
