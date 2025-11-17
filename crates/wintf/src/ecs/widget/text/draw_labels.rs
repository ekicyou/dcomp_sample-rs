use crate::com::d2d::{D2D1CommandListExt, D2D1DeviceContextExt};
use crate::com::dwrite::DWriteFactoryExt;
use crate::ecs::graphics::{GraphicsCommandList, GraphicsCore};
use crate::ecs::widget::shapes::rectangle::colors;
use crate::ecs::widget::text::{Label, TextLayoutResource};
use bevy_ecs::prelude::*;
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
    query: Query<
        (Entity, &Label),
        Or<(Changed<Label>, Without<GraphicsCommandList>)>,
    >,
    graphics_core: Option<Res<GraphicsCore>>,
) {
    let Some(graphics_core) = graphics_core else {
        eprintln!("[draw_labels] GraphicsCore not available, skipping");
        return;
    };

    let Some(dwrite_factory) = graphics_core.dwrite_factory() else {
        eprintln!("[draw_labels] DirectWrite factory not available");
        return;
    };

    // グローバル共有DeviceContextを取得
    let Some(dc) = graphics_core.device_context() else {
        eprintln!("[draw_labels] DeviceContext not available");
        return;
    };

    for (entity, label) in query.iter() {
        #[cfg(debug_assertions)]
        eprintln!(
            "[draw_labels] Entity={:?}, text='{}', font='{}', size={}pt",
            entity, label.text, label.font_family, label.font_size
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
                eprintln!(
                    "[draw_labels] Failed to create TextFormat for Entity={:?}: {:?}",
                    entity, err
                );
                continue;
            }
        };

        // TextLayout作成
        let text_hstring = windows::core::HSTRING::from(&label.text);
        let text_layout = match dwrite_factory.create_text_layout(
            &text_hstring,
            &text_format,
            f32::MAX,
            f32::MAX,
        ) {
            Ok(layout) => layout,
            Err(err) => {
                eprintln!(
                    "[draw_labels] Failed to create TextLayout for Entity={:?}: {:?}",
                    entity, err
                );
                continue;
            }
        };

        // CommandList生成
        let command_list = match unsafe { dc.CreateCommandList() } {
            Ok(cl) => cl,
            Err(err) => {
                eprintln!(
                    "[draw_labels] Failed to create CommandList for Entity={:?}: {:?}",
                    entity, err
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
                eprintln!(
                    "[draw_labels] Failed to create brush for Entity={:?}: {:?}",
                    entity, err
                );
                unsafe {
                    let _ = dc.EndDraw(None, None);
                }
                continue;
            }
        };

        // テキスト描画（原点0,0から描画）
        let origin = Vector2 { X: 0.0, Y: 0.0 };
        dc.draw_text_layout(origin, &text_layout, &brush, D2D1_DRAW_TEXT_OPTIONS_NONE);

        // 描画終了
        if let Err(err) = unsafe { dc.EndDraw(None, None) } {
            eprintln!(
                "[draw_labels] EndDraw failed for Entity={:?}: {:?}",
                entity, err
            );
            continue;
        }

        // CommandListを閉じる
        if let Err(err) = command_list.close() {
            eprintln!(
                "[draw_labels] Failed to close CommandList for Entity={:?}: {:?}",
                entity, err
            );
            continue;
        }

        // GraphicsCommandListとTextLayoutResourceをエンティティに挿入
        commands.entity(entity).insert((
            GraphicsCommandList::new(command_list),
            TextLayoutResource::new(text_layout),
        ));
    }
}
