//! ドラッグシステム
//!
//! ウィンドウドラッグ移動とドラッグ状態クリーンアップを提供する。

use super::{DragConfig, DragConstraint, DragEndEvent, DragEvent, DraggingState};
use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::message::MessageReader;
use bevy_ecs::prelude::*;

/// ウィンドウドラッグ移動システム
///
/// DragEventメッセージを読み取り、ドラッグ対象エンティティの親階層にある
/// Windowエンティティの`BoxStyle.inset`を更新する。
///
/// `DragConfig.move_window`が`true`のエンティティのみが対象。
/// Window が見つからない場合はスキップする（将来の非ウィンドウドラッグ対応）。
///
/// # パイプライン
///
/// BoxStyle.inset 更新 → taffy レイアウト → Arrangement → GlobalArrangement
/// → window_pos_sync_system → WindowPos → apply_window_pos_changes → SetWindowPosCommand
pub fn apply_window_drag_movement(
    mut drag_events: MessageReader<DragEvent>,
    dragging_query: Query<(&DraggingState, &DragConfig, Option<&DragConstraint>)>,
    child_of_query: Query<&ChildOf>,
    window_query: Query<(), With<crate::ecs::window::Window>>,
    mut box_style_query: Query<&mut crate::ecs::layout::BoxStyle>,
) {
    for event in drag_events.read() {
        // ターゲットエンティティから DraggingState と DragConfig を取得
        let Ok((dragging_state, drag_config, constraint)) = dragging_query.get(event.target) else {
            continue;
        };

        // ウィンドウ移動が無効なら処理しない
        if !drag_config.move_window {
            continue;
        }

        // ターゲットの親階層からWindowエンティティを探索
        let mut current = event.target;
        let window_entity = loop {
            if window_query.get(current).is_ok() {
                break Some(current);
            }
            if let Ok(child_of) = child_of_query.get(current) {
                current = child_of.parent();
            } else {
                break None;
            }
        };

        let Some(window_entity) = window_entity else {
            tracing::trace!(
                target = ?event.target,
                "[apply_window_drag_movement] No Window found in hierarchy, skipping"
            );
            continue;
        };

        // 新しいinsetを計算: initial_inset + (current_pos - start_pos)
        let delta_x = event.position.x - event.start_position.x;
        let delta_y = event.position.y - event.start_position.y;
        let mut new_left = dragging_state.initial_inset.0 + delta_x as f32;
        let mut new_top = dragging_state.initial_inset.1 + delta_y as f32;

        // DragConstraint があれば制約を適用
        if let Some(constraint) = constraint {
            let (cx, cy) = constraint.apply(new_left as i32, new_top as i32);
            new_left = cx as f32;
            new_top = cy as f32;
        }

        // BoxStyle.inset を更新
        if let Ok(mut box_style) = box_style_query.get_mut(window_entity) {
            if let Some(inset) = &mut box_style.inset {
                inset.0.left = crate::ecs::layout::LengthPercentageAuto::Px(new_left);
                inset.0.top = crate::ecs::layout::LengthPercentageAuto::Px(new_top);
            }
        }

        tracing::debug!(
            target = ?event.target,
            window = ?window_entity,
            delta_x,
            delta_y,
            new_left,
            new_top,
            "[apply_window_drag_movement] Window position updated"
        );
    }
}

/// ドラッグ状態クリーンアップシステム
///
/// DragEndEventを監視し、DraggingStateを削除する。
pub fn cleanup_drag_state(
    mut commands: Commands,
    mut drag_end_events: MessageReader<DragEndEvent>,
) {
    for event in drag_end_events.read() {
        // DraggingStateを削除
        commands
            .entity(event.target)
            .remove::<crate::ecs::drag::DraggingState>();

        tracing::debug!(
            target = ?event.target,
            "[cleanup_drag_state] DraggingState removed"
        );
    }
}
