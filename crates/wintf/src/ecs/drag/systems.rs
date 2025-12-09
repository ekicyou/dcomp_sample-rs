//! ドラッグシステム
//!
//! ドラッグ状態クリーンアップを提供する。

use bevy_ecs::prelude::*;
use bevy_ecs::message::MessageReader;
use super::DragEndEvent;

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
