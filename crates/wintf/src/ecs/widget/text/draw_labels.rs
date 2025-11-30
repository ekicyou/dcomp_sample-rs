use crate::com::d2d::{D2D1CommandListExt, D2D1DeviceContextExt};
use crate::com::dwrite::DWriteFactoryExt;
use crate::ecs::graphics::{GraphicsCommandList, GraphicsCore};
use crate::ecs::widget::shapes::rectangle::colors;
use crate::ecs::widget::text::label::TextDirection;
use crate::ecs::widget::text::{Label, TextLayoutResource};
use crate::ecs::TextLayoutMetrics;
use bevy_ecs::prelude::*;
use tracing::{debug, warn};
use windows::Win32::Graphics::Direct2D::D2D1_DRAW_TEXT_OPTIONS_NONE;
use windows::Win32::Graphics::DirectWrite::*;
use windows_numerics::Vector2;

/// Labelコンポーネントから GraphicsCommandList を生成
///
/// Changed<Label> または GraphicsCommandList がないエンティティを対象に、
/// DirectWrite を使用してテキストレイアウトを生成し、
/// Direct2D の CommandList に描画命令を記録する。
pub fn draw_labels(
    mut commands: Commands,
    query: Query<(Entity, &Label), Or<(Changed<Label>, Without<GraphicsCommandList>)>>,
    graphics_core: Option<Res<GraphicsCore>>,
) {
    let Some(graphics_core) = graphics_core else {
        warn!("GraphicsCore not available, skipping draw_labels");
        return;
    };

    let Some(dwrite_factory) = graphics_core.dwrite_factory() else {
        warn!("DirectWrite factory not available");
        return;
    };

    // グローバル共有DeviceContextを取得
    let Some(dc) = graphics_core.device_context() else {
        warn!("DeviceContext not available");
        return;
    };

    for (entity, label) in query.iter() {
        #[cfg(debug_assertions)]
        debug!(
            entity = ?entity,
            text = %label.text,
            font = %label.font_family,
            size_pt = label.font_size,
            "Drawing label"
        );

        // TextFormat作成
        let font_family_hstring = windows::core::HSTRING::from(&label.font_family);
        let locale_hstring = windows::core::HSTRING::from("ja-JP");
        let text_format = match dwrite_factory.create_text_format(
            &font_family_hstring,
            None::<&windows::Win32::Graphics::DirectWrite::IDWriteFontCollection>,
            DWRITE_FONT_WEIGHT_NORMAL,
            DWRITE_FONT_STYLE_NORMAL,
            DWRITE_FONT_STRETCH_NORMAL,
            label.font_size,
            &locale_hstring,
        ) {
            Ok(fmt) => fmt,
            Err(err) => {
                warn!(
                    entity = ?entity,
                    error = ?err,
                    "Failed to create TextFormat"
                );
                continue;
            }
        };

        // 縦書き・横書きの設定
        unsafe {
            match label.direction {
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

        // TextLayout作成
        // maxWidth/maxHeightを0にしてNO_WRAPを指定することで、
        // コンテンツサイズに合わせてレイアウトさせる。
        // RTLの場合、maxWidthが無限大だと右端が無限遠になってしまうため。
        let text_hstring = windows::core::HSTRING::from(&label.text);
        let text_layout =
            match dwrite_factory.create_text_layout(&text_hstring, &text_format, 0.0, 0.0) {
                Ok(layout) => layout,
                Err(err) => {
                    warn!(
                        entity = ?entity,
                        error = ?err,
                        "Failed to create TextLayout"
                    );
                    continue;
                }
            }; // 折り返しなしに設定
        unsafe {
            let _ = text_layout.SetWordWrapping(DWRITE_WORD_WRAPPING_NO_WRAP);
        }

        // メトリクス取得（描画位置調整のために先に取得）
        let mut metrics = DWRITE_TEXT_METRICS::default();
        unsafe {
            let _ = text_layout.GetMetrics(&mut metrics);
        }

        // CommandList生成
        let command_list = match unsafe { dc.CreateCommandList() } {
            Ok(cl) => cl,
            Err(err) => {
                warn!(
                    entity = ?entity,
                    error = ?err,
                    "Failed to create CommandList"
                );
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
        let brush = match dc.create_solid_color_brush(&label.color, None) {
            Ok(b) => b,
            Err(err) => {
                warn!(
                    entity = ?entity,
                    error = ?err,
                    "Failed to create brush"
                );
                unsafe {
                    let _ = dc.EndDraw(None, None);
                }
                continue;
            }
        };

        // テキスト描画
        // RTL（縦書き含む）の場合、レイアウト幅0でNO_WRAPにすると、
        // テキストは原点から左側（負のX方向）にはみ出して描画されるため、metrics.leftは負の値になる。
        // これを正の領域（0,0〜）に持ってくるため、origin.Xを -metrics.left だけずらす。
        // LTRの場合は metrics.left は通常0なので影響しない。
        let origin = Vector2 {
            X: -metrics.left,
            Y: -metrics.top, // topも念のため補正
        };

        dc.draw_text_layout(origin, &text_layout, &brush, D2D1_DRAW_TEXT_OPTIONS_NONE);

        // 描画終了
        if let Err(err) = unsafe { dc.EndDraw(None, None) } {
            warn!(
                entity = ?entity,
                error = ?err,
                "EndDraw failed"
            );
            continue;
        }

        // CommandListを閉じる
        if let Err(err) = command_list.close() {
            warn!(
                entity = ?entity,
                error = ?err,
                "Failed to close CommandList"
            );
            continue;
        }

        // GraphicsCommandListとTextLayoutResource、TextLayoutMetricsをエンティティに挿入
        // metrics.width/heightは物理サイズ（スクリーン座標系）として扱われるため、そのまま使用する。
        commands.entity(entity).insert((
            GraphicsCommandList::new(command_list),
            TextLayoutResource::new(text_layout),
            TextLayoutMetrics {
                width: metrics.width,
                height: metrics.height,
            },
        ));
    }
}
