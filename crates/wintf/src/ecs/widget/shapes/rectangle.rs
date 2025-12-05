use crate::com::d2d::{D2D1CommandListExt, D2D1DeviceContextExt};
use crate::ecs::graphics::{format_entity_name, GraphicsCommandList, GraphicsCore};
use crate::ecs::layout::Arrangement;
use crate::ecs::widget::brushes::{Brushes, DEFAULT_FOREGROUND};
use crate::ecs::Visual;
use bevy_ecs::component::Component;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::name::Name;
use bevy_ecs::prelude::*;
use bevy_ecs::world::DeferredWorld;
use tracing::{trace, warn};
use windows::Win32::Graphics::Direct2D::Common::{D2D1_COLOR_F, D2D_RECT_F};

/// 色の型エイリアス（D2D1_COLOR_Fをそのまま使用）
pub type Color = D2D1_COLOR_F;

/// 基本色定数（Brush::XXXを使用する新しい方式を推奨）
#[deprecated(since = "0.1.0", note = "Use Brush::XXX constants instead")]
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

/// 透明色定数（内部使用）
const TRANSPARENT_COLOR: Color = Color {
    r: 0.0,
    g: 0.0,
    b: 0.0,
    a: 0.0,
};

/// 四角形ウィジット
///
/// サイズは`Arrangement`コンポーネントで管理されます。
/// レイアウトシステム（`BoxSize` → Taffy → `Arrangement`）との統合により、
/// 動的なサイズ変更が自動的に反映されます。
///
/// 色は`Brushes`コンポーネントで指定します。
/// ```ignore
/// world.spawn((
///     Rectangle::new(),
///     Brushes::with_foreground(Brush::RED.as_color().unwrap()),
/// ));
/// ```
#[derive(Component, Debug, Clone, Default)]
// NOTE: on_add フックは既存の描画フロー（draw_recursive方式）と競合する
// Phase 4（自己描画方式への移行）完了後に正常表示される
#[component(on_add = on_rectangle_add, on_remove = on_rectangle_remove)]
pub struct Rectangle;

impl Rectangle {
    /// 新しいRectangleを作成
    pub fn new() -> Self {
        Self
    }
}

/// Rectangle追加時のフック: Visualコンポーネントを自動挿入 (R4)
fn on_rectangle_add(mut world: DeferredWorld, hook: HookContext) {
    // 既にVisualを持っている場合はスキップ
    if world.get::<Visual>(hook.entity).is_some() {
        return;
    }
    world
        .commands()
        .entity(hook.entity)
        .insert(Visual::default());
}

/// Rectangleコンポーネントが削除される時に呼ばれるフック
fn on_rectangle_remove(mut world: DeferredWorld, hook: HookContext) {
    let entity = hook.entity;
    // GraphicsCommandListを取得して中身をクリア(Changed検出のため)
    if let Some(mut cmd_list) = world.get_mut::<GraphicsCommandList>(entity) {
        cmd_list.set_if_neq(GraphicsCommandList::empty());
    }
}

/// RectangleコンポーネントからGraphicsCommandListを生成
///
/// Rectangleのサイズは`Arrangement`コンポーネントから取得されます。
/// 色は`Brushes`コンポーネントのforegroundから取得されます。
pub fn draw_rectangles(
    mut commands: Commands,
    query: Query<
        (
            Entity,
            &Rectangle,
            &Arrangement,
            &Brushes,
            Option<&GraphicsCommandList>,
            Option<&Name>,
        ),
        Or<(Changed<Rectangle>, Changed<Arrangement>, Changed<Brushes>)>,
    >,
    graphics_core: Option<Res<GraphicsCore>>,
) {
    let Some(graphics_core) = graphics_core else {
        warn!("GraphicsCore not available, skipping draw_rectangles");
        return;
    };

    for (entity, _rectangle, arrangement, brushes, cmd_list_opt, name) in query.iter() {
        let entity_name = format_entity_name(entity, name);
        trace!(
            entity = %entity_name,
            width = arrangement.size.width,
            height = arrangement.size.height,
            "Drawing rectangle"
        );

        // Brushes.foregroundから色を取得
        let color = brushes
            .foreground
            .as_color()
            .unwrap_or_else(|| DEFAULT_FOREGROUND.as_color().unwrap());

        // DeviceContextとCommandList生成
        // グローバル共有DeviceContextを取得
        let dc = match graphics_core.device_context() {
            Some(dc) => dc,
            None => {
                warn!(
                    entity = %entity_name,
                    "DeviceContext not available"
                );
                continue;
            }
        };

        let command_list = match unsafe { dc.CreateCommandList() } {
            Ok(cl) => cl,
            Err(err) => {
                warn!(
                    entity = %entity_name,
                    error = ?err,
                    "Failed to create CommandList"
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
            dc.clear(Some(&TRANSPARENT_COLOR));

            // 四角形描画（原点0,0から描画）
            let rect = D2D_RECT_F {
                left: 0.0,
                top: 0.0,
                right: arrangement.size.width,
                bottom: arrangement.size.height,
            };

            trace!(
                entity = %entity_name,
                left = rect.left,
                top = rect.top,
                right = rect.right,
                bottom = rect.bottom,
                r = format_args!("{:.2}", color.r),
                g = format_args!("{:.2}", color.g),
                b = format_args!("{:.2}", color.b),
                a = format_args!("{:.2}", color.a),
                "Rectangle details"
            );

            // ソリッドカラーブラシ作成
            let brush = match dc.create_solid_color_brush(&color, None) {
                Ok(b) => b,
                Err(err) => {
                    warn!(
                        entity = %entity_name,
                        error = ?err,
                        "Failed to create brush"
                    );
                    let _ = dc.EndDraw(None, None);
                    continue;
                }
            };

            dc.fill_rectangle(&rect, &brush);

            if let Err(err) = dc.EndDraw(None, None) {
                warn!(
                    entity = %entity_name,
                    error = ?err,
                    "EndDraw failed"
                );
                continue;
            }
        }

        // CommandListを閉じる
        if let Err(err) = command_list.close() {
            warn!(
                entity = %entity_name,
                error = ?err,
                "Failed to close CommandList"
            );
            continue;
        }

        // GraphicsCommandListコンポーネントを挿入または更新
        let new_cmd_list = GraphicsCommandList::new(command_list);
        match cmd_list_opt {
            Some(existing) if *existing != new_cmd_list => {
                commands.entity(entity).insert(new_cmd_list);
                trace!(
                    entity = %entity_name,
                    "CommandList updated"
                );
            }
            None => {
                commands.entity(entity).insert(new_cmd_list);
                trace!(
                    entity = %entity_name,
                    "CommandList created"
                );
            }
            _ => {
                trace!(
                    entity = %entity_name,
                    "CommandList unchanged"
                );
            }
        }
    }
}
