use crate::com::d2d::{D2D1CommandListExt, D2D1DeviceContextExt, D2D1DeviceExt};
use crate::ecs::graphics::{GraphicsCommandList, GraphicsCore};
use bevy_ecs::component::Component;
use bevy_ecs::prelude::*;
use windows::Win32::Graphics::Direct2D::Common::{D2D1_COLOR_F, D2D_RECT_F};

/// 色の型エイリアス（D2D1_COLOR_Fをそのまま使用）
pub type Color = D2D1_COLOR_F;

/// 基本色定義
pub mod colors {
    use super::Color;

    /// 透明色
    pub const TRANSPARENT: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    };
    /// 黒
    pub const BLACK: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    /// 白
    pub const WHITE: Color = Color {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    };
    /// 赤
    pub const RED: Color = Color {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    };
    /// 緑
    pub const GREEN: Color = Color {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    };
    /// 青
    pub const BLUE: Color = Color {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    };
}

/// 四角形ウィジット
#[derive(Component, Debug, Clone)]
#[component(on_remove = on_rectangle_remove)]
pub struct Rectangle {
    /// X座標（ピクセル単位）
    pub x: f32,
    /// Y座標（ピクセル単位）
    pub y: f32,
    /// 幅（ピクセル単位）
    pub width: f32,
    /// 高さ（ピクセル単位）
    pub height: f32,
    /// 塗りつぶし色
    pub color: Color,
}

/// Rectangleコンポーネントが削除される時に呼ばれるフック
fn on_rectangle_remove(
    mut world: bevy_ecs::world::DeferredWorld,
    hook: bevy_ecs::lifecycle::HookContext,
) {
    let entity = hook.entity;
    // GraphicsCommandListを取得して中身をクリア(Changed検出のため)
    if let Some(mut cmd_list) = world.get_mut::<GraphicsCommandList>(entity) {
        cmd_list.set_if_neq(GraphicsCommandList::empty());
    }
}
/// RectangleコンポーネントからGraphicsCommandListを生成
pub fn draw_rectangles(
    mut commands: Commands,
    query: Query<(Entity, &Rectangle), Changed<Rectangle>>,
    graphics_core: Option<Res<GraphicsCore>>,
) {
    let Some(graphics_core) = graphics_core else {
        eprintln!("[draw_rectangles] GraphicsCore not available, skipping");
        return;
    };

    for (entity, rectangle) in query.iter() {
        eprintln!("[draw_rectangles] Entity={:?}", entity);
        eprintln!(
            "[draw_rectangles] Rectangle: x={}, y={}, width={}, height={}, color=({},{},{},{})",
            rectangle.x,
            rectangle.y,
            rectangle.width,
            rectangle.height,
            rectangle.color.r,
            rectangle.color.g,
            rectangle.color.b,
            rectangle.color.a
        );

        // DeviceContextとCommandList生成
        let dc = match graphics_core.d2d_device().create_device_context(
            windows::Win32::Graphics::Direct2D::D2D1_DEVICE_CONTEXT_OPTIONS_NONE,
        ) {
            Ok(dc) => dc,
            Err(err) => {
                eprintln!(
                    "[draw_rectangles] Failed to create DeviceContext for Entity={:?}: {:?}",
                    entity, err
                );
                continue;
            }
        };

        let command_list = match unsafe { dc.CreateCommandList() } {
            Ok(cl) => cl,
            Err(err) => {
                eprintln!(
                    "[draw_rectangles] Failed to create CommandList for Entity={:?}: {:?}",
                    entity, err
                );
                continue;
            }
        };

        // DeviceContextのターゲットをCommandListに設定
        unsafe {
            dc.SetTarget(&command_list);
        }

        // 描画命令を記録
        unsafe {
            dc.BeginDraw();

            // 透明色クリア
            dc.clear(Some(&colors::TRANSPARENT));

            // 四角形描画
            let rect = D2D_RECT_F {
                left: rectangle.x,
                top: rectangle.y,
                right: rectangle.x + rectangle.width,
                bottom: rectangle.y + rectangle.height,
            };

            // ソリッドカラーブラシ作成
            let brush = match dc.create_solid_color_brush(&rectangle.color, None) {
                Ok(b) => b,
                Err(err) => {
                    eprintln!(
                        "[draw_rectangles] Failed to create brush for Entity={:?}: {:?}",
                        entity, err
                    );
                    let _ = dc.EndDraw(None, None);
                    continue;
                }
            };

            dc.fill_rectangle(&rect, &brush);

            if let Err(err) = dc.EndDraw(None, None) {
                eprintln!(
                    "[draw_rectangles] EndDraw failed for Entity={:?}: {:?}",
                    entity, err
                );
                continue;
            }
        }

        // CommandListを閉じる
        if let Err(err) = command_list.close() {
            eprintln!(
                "[draw_rectangles] Failed to close CommandList for Entity={:?}: {:?}",
                entity, err
            );
            continue;
        }

        // GraphicsCommandListコンポーネントを挿入
        commands
            .entity(entity)
            .insert(GraphicsCommandList::new(command_list));
        eprintln!(
            "[draw_rectangles] CommandList created for Entity={:?}",
            entity
        );
    }
}
