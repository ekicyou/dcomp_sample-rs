use crate::com::d2d::{D2D1CommandListExt, D2D1DeviceContextExt};
use crate::ecs::graphics::{GraphicsCommandList, GraphicsCore};
use crate::ecs::layout::Arrangement;
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
///
/// サイズは`Arrangement`コンポーネントで管理されます。
/// レイアウトシステム（`BoxSize` → Taffy → `Arrangement`）との統合により、
/// 動的なサイズ変更が自動的に反映されます。
#[derive(Component, Debug, Clone)]
#[component(on_remove = on_rectangle_remove)]
pub struct Rectangle {
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
///
/// Rectangleのサイズは`Arrangement`コンポーネントから取得されます。
pub fn draw_rectangles(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &Rectangle,
            &Arrangement,
            Option<&GraphicsCommandList>,
        ),
        Or<(Changed<Rectangle>, Changed<Arrangement>)>,
    >,
    graphics_core: Option<Res<GraphicsCore>>,
) {
    let Some(graphics_core) = graphics_core else {
        eprintln!("[draw_rectangles] GraphicsCore not available, skipping");
        return;
    };

    for (entity, rectangle, arrangement, cmd_list_opt) in query.iter() {
        eprintln!(
            "[draw_rectangles] Entity={:?}, size=({}, {})",
            entity, arrangement.size.width, arrangement.size.height
        );

        // DeviceContextとCommandList生成
        // グローバル共有DeviceContextを取得
        let dc = match graphics_core.device_context() {
            Some(dc) => dc,
            None => {
                eprintln!(
                    "[draw_rectangles] DeviceContext not available for Entity={:?}",
                    entity
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

            // 四角形描画（原点0,0から描画）
            let rect = D2D_RECT_F {
                left: 0.0,
                top: 0.0,
                right: arrangement.size.width,
                bottom: arrangement.size.height,
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

        // GraphicsCommandListコンポーネントを挿入または更新
        let new_cmd_list = GraphicsCommandList::new(command_list);
        match cmd_list_opt {
            Some(existing) if *existing != new_cmd_list => {
                commands.entity(entity).insert(new_cmd_list);
                eprintln!(
                    "[draw_rectangles] CommandList updated for Entity={:?}",
                    entity
                );
            }
            None => {
                commands.entity(entity).insert(new_cmd_list);
                eprintln!(
                    "[draw_rectangles] CommandList created for Entity={:?}",
                    entity
                );
            }
            _ => {
                eprintln!(
                    "[draw_rectangles] CommandList unchanged for Entity={:?}",
                    entity
                );
            }
        }
    }
}
