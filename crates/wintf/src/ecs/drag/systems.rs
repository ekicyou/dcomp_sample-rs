//! ドラッグシステム
//!
//! ウィンドウ移動とドラッグ状態クリーンアップを提供する。

use bevy_ecs::prelude::*;
use bevy_ecs::message::{Messages, MessageReader};
use super::{DragEvent, DragEndEvent, DragConstraint, DraggingMarker};
use crate::ecs::window::Window;

/// ウィンドウドラッグ移動システム
///
/// DragEventを監視し、ウィンドウ位置を更新する。
pub fn apply_window_drag_movement(world: &mut World) {
    // DragEventを取得
    let events: Vec<DragEvent> = {
        let events_reader = world.resource::<Messages<DragEvent>>();
        events_reader.iter_current_update_messages().cloned().collect()
    };
    
    for event in events {
        // イベント対象エンティティの親階層からWindowコンポーネントを探索
        let mut current = event.target;
        let mut window_entity = None;
        
        loop {
            if let Ok(entity_ref) = world.get_entity(current) {
                if entity_ref.contains::<Window>() {
                    window_entity = Some(current);
                    break;
                }
                
                // 親へ
                if let Some(child_of) = entity_ref.get::<bevy_ecs::hierarchy::ChildOf>() {
                    current = child_of.parent();
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        
        if let Some(window_entity) = window_entity {
            // WindowPosコンポーネントを取得
            if let Ok(entity_ref) = world.get_entity(window_entity) {
                // まず制約を取得
                let constraint = entity_ref.get::<DragConstraint>().copied();
                
                // 次にWindowPosを更新
                drop(entity_ref);
                
                if let Ok(mut entity_mut) = world.get_entity_mut(window_entity) {
                    if let Some(mut window_pos) = entity_mut.get_mut::<crate::ecs::window::WindowPos>() {
                        if let Some(pos) = &mut window_pos.position {
                            // 新しい位置を計算
                            let new_x = pos.x + event.delta.x;
                            let new_y = pos.y + event.delta.y;
                            
                            // 制約を適用
                            let (constrained_x, constrained_y) = if let Some(c) = constraint {
                                c.apply(new_x, new_y)
                            } else {
                                (new_x, new_y)
                            };
                            
                            // 位置を更新
                            pos.x = constrained_x;
                            pos.y = constrained_y;
                            
                            tracing::trace!(
                                window = ?window_entity,
                                dx = event.delta.x,
                                dy = event.delta.y,
                                new_x = constrained_x,
                                new_y = constrained_y,
                                "[apply_window_drag_movement] Window position updated"
                            );
                        }
                    }
                }
            }
        }
    }
}

/// ドラッグ状態クリーンアップシステム
///
/// DragEndEventを監視し、DraggingMarkerを削除する。
pub fn cleanup_drag_state(
    mut commands: Commands,
    mut drag_end_events: MessageReader<DragEndEvent>,
    dragging_query: Query<Entity, With<DraggingMarker>>,
) {
    for event in drag_end_events.read() {
        // DraggingMarkerを持つエンティティを全て削除
        for entity in dragging_query.iter() {
            commands.entity(entity).remove::<DraggingMarker>();
            
            tracing::debug!(
                entity = ?entity,
                target = ?event.target,
                "[cleanup_drag_state] DraggingMarker removed"
            );
        }
    }
}
