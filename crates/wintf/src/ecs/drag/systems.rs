//! ドラッグシステム
//!
//! ウィンドウ移動とドラッグ状態クリーンアップを提供する。

use bevy_ecs::prelude::*;
use bevy_ecs::message::{Messages, MessageReader};
use super::{DragEvent, DragEndEvent};
use crate::ecs::drag::{DragConstraint};
use crate::ecs::window::Window;

/// ウィンドウドラッグ移動システム
///
/// DragEventを監視し、BoxStyle.insetを更新してウィンドウ位置を変更する。
pub fn apply_window_drag_movement(
    mut drag_events: bevy_ecs::message::MessageReader<DragEvent>,
    mut query: Query<(&mut crate::ecs::layout::BoxStyle, Option<&DragConstraint>), With<Window>>,
    child_query: Query<&bevy_ecs::hierarchy::ChildOf>,
) {
    let events: Vec<DragEvent> = drag_events.read().cloned().collect();
    
    tracing::info!("[apply_window_drag_movement] Processing {} drag events", events.len());
    
    for event in events {
        // イベント対象エンティティの親階層からWindowコンポーネントを探索
        let mut current = event.target;
        let mut window_entity = None;
        
        loop {
            if query.get(current).is_ok() {
                window_entity = Some(current);
                break;
            }
            
            // 親へ
            if let Ok(child_of) = child_query.get(current) {
                current = child_of.parent();
            } else {
                break;
            }
        }
        
        if let Some(window_entity) = window_entity {
            if let Ok((mut box_style, constraint)) = query.get_mut(window_entity) {
                if let Some(inset) = &mut box_style.inset {
                    // 現在のinset値を取得
                    let current_left = match inset.0.left {
                        crate::ecs::layout::LengthPercentageAuto::Px(val) => val,
                        _ => 0.0,
                    };
                    let current_top = match inset.0.top {
                        crate::ecs::layout::LengthPercentageAuto::Px(val) => val,
                        _ => 0.0,
                    };
                    
                    // 新しい位置を計算
                    let new_left = current_left + event.delta.x as f32;
                    let new_top = current_top + event.delta.y as f32;
                    
                    // 制約を適用
                    let (constrained_left, constrained_top) = if let Some(c) = constraint {
                        let (x, y) = c.apply(new_left as i32, new_top as i32);
                        (x as f32, y as f32)
                    } else {
                        (new_left, new_top)
                    };
                    
                    // insetを更新
                    inset.0.left = crate::ecs::layout::LengthPercentageAuto::Px(constrained_left);
                    inset.0.top = crate::ecs::layout::LengthPercentageAuto::Px(constrained_top);
                    
                    tracing::info!(
                        window = ?window_entity,
                        dx = event.delta.x,
                        dy = event.delta.y,
                        new_left = constrained_left,
                        new_top = constrained_top,
                        "[apply_window_drag_movement] BoxStyle.inset updated"
                    );
                }
            }
        }
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
        commands.entity(event.target).remove::<crate::ecs::drag::DraggingState>();
        
        tracing::debug!(
            target = ?event.target,
            "[cleanup_drag_state] DraggingState removed"
        );
    }
}
