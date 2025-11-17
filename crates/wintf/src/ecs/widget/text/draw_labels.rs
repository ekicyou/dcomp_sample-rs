use crate::com::d2d::{D2D1CommandListExt, D2D1DeviceContextExt, D2D1DeviceExt};
use crate::com::dwrite::DWriteFactoryExt;
use crate::ecs::graphics::{GraphicsCommandList, GraphicsCore, WindowGraphics};
use crate::ecs::widget::shapes::rectangle::colors;
use crate::ecs::widget::text::{Label, TextLayout};
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
        (Entity, &Label, &WindowGraphics),
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

    let Some(d2d_device) = graphics_core.d2d_device() else {
        eprintln!("[draw_labels] Direct2D device not available");
        return;
    };

    for (entity, label, window_graphics) in query.iter() {
        // WindowGraphicsが無効なら後回し
        if !window_graphics.is_valid() {
            continue;
        }

        #[cfg(debug_assertions)]
        eprintln!(
            "[draw_labels] Entity={:?}, text='{}', font='{}', size={}pt, pos=({}, {})",
            entity, label.text, label.font_family, label.font_size, label.x, label.y
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

        // DeviceContextとCommandList生成
        let dc = match d2d_device.create_device_context(
            windows::Win32::Graphics::Direct2D::D2D1_DEVICE_CONTEXT_OPTIONS_NONE,
        ) {
            Ok(dc) => dc,
            Err(err) => {
                eprintln!(
                    "[draw_labels] Failed to create DeviceContext for Entity={:?}: {:?}",
                    entity, err
                );
                continue;
            }
        };

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

        // テキスト描画
        let origin = Vector2 {
            X: label.x,
            Y: label.y,
        };
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

        // GraphicsCommandListとTextLayoutをエンティティに挿入
        commands.entity(entity).insert((
            GraphicsCommandList::new(command_list),
            TextLayout::new(text_layout),
        ));
    }
}
