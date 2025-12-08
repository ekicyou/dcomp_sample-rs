//! ドラッグイベントディスパッチ
//!
//! DragStateからイベントを生成し、Phase<T>でTunnel/Bubble配信する。

use bevy_ecs::prelude::*;
use bevy_ecs::message::Message;
use std::time::Instant;
use crate::ecs::pointer::{PhysicalPoint, build_bubble_path, EventHandler};
use super::DragState;
use super::state::{read_drag_state, reset_to_idle, update_dragging};

/// ドラッグ開始イベント
#[derive(Message, Clone, Debug)]
pub struct DragStartEvent {
    /// ドラッグ対象エンティティ
    pub target: Entity,
    /// ドラッグ開始位置（物理ピクセル、スクリーン座標）
    pub position: PhysicalPoint,
    /// 左ボタンドラッグかどうか
    pub is_primary: bool,
    /// イベント発生時刻
    pub timestamp: Instant,
}

/// ドラッグ中イベント
#[derive(Message, Clone, Debug)]
pub struct DragEvent {
    /// ドラッグ対象エンティティ
    pub target: Entity,
    /// 前回位置からの移動量（物理ピクセル）
    pub delta: PhysicalPoint,
    /// 現在位置（物理ピクセル、スクリーン座標）
    pub position: PhysicalPoint,
    /// 左ボタンドラッグかどうか
    pub is_primary: bool,
    /// イベント発生時刻
    pub timestamp: Instant,
}

/// ドラッグ終了イベント
#[derive(Message, Clone, Debug)]
pub struct DragEndEvent {
    /// ドラッグ対象エンティティ
    pub target: Entity,
    /// 終了位置（物理ピクセル、スクリーン座標）
    pub position: PhysicalPoint,
    /// キャンセルされたかどうか
    pub cancelled: bool,
    /// 左ボタンドラッグかどうか
    pub is_primary: bool,
    /// イベント発生時刻
    pub timestamp: Instant,
}

/// ドラッグ開始ハンドラコンポーネント
#[derive(Component, Clone, Copy)]
#[component(storage = "SparseSet")]
pub struct OnDragStart(pub EventHandler<DragStartEvent>);

/// ドラッグ中ハンドラコンポーネント
#[derive(Component, Clone, Copy)]
#[component(storage = "SparseSet")]
pub struct OnDrag(pub EventHandler<DragEvent>);

/// ドラッグ終了ハンドラコンポーネント
#[derive(Component, Clone, Copy)]
#[component(storage = "SparseSet")]
pub struct OnDragEnd(pub EventHandler<DragEndEvent>);

/// ドラッグイベントディスパッチシステム
pub fn dispatch_drag_events(world: &mut World) {
    let state_snapshot = read_drag_state(|state| state.clone());
    
    match state_snapshot {
        DragState::JustStarted { entity, start_pos, current_pos, .. } => {
            let event = DragStartEvent {
                target: entity,
                position: start_pos,
                is_primary: true,
                timestamp: Instant::now(),
            };
            
            tracing::debug!(
                entity = ?entity,
                x = start_pos.x,
                y = start_pos.y,
                "[DragStartEvent] Dispatching"
            );
            
            // Messagesに送信
            world.resource_mut::<bevy_ecs::message::Messages<DragStartEvent>>().send(event.clone());
            
            // Phase<T>配信（pointer::dispatch_event_for_handlerを使用）
            let path = build_bubble_path(world, entity);
            crate::ecs::pointer::dispatch_event_for_handler::<DragStartEvent, OnDragStart>(
                world,
                entity,
                &path,
                &event,
                |h| h.0,
            );
            
            // JustStarted → Dragging遷移
            update_dragging(current_pos);
        }
        
        DragState::Dragging { entity, current_pos, prev_pos, .. } => {
            let delta = PhysicalPoint::new(
                current_pos.x - prev_pos.x,
                current_pos.y - prev_pos.y,
            );
            
            let event = DragEvent {
                target: entity,
                delta,
                position: current_pos,
                is_primary: true,
                timestamp: Instant::now(),
            };
            
            // Messagesに送信
            world.resource_mut::<bevy_ecs::message::Messages<DragEvent>>().send(event.clone());
            
            let path = build_bubble_path(world, entity);
            crate::ecs::pointer::dispatch_event_for_handler::<DragEvent, OnDrag>(
                world,
                entity,
                &path,
                &event,
                |h| h.0,
            );
        }
        
        DragState::JustEnded { entity, position, cancelled } => {
            let event = DragEndEvent {
                target: entity,
                position,
                cancelled,
                is_primary: true,
                timestamp: Instant::now(),
            };
            
            tracing::debug!(
                entity = ?entity,
                x = position.x,
                y = position.y,
                cancelled,
                "[DragEndEvent] Dispatching"
            );
            
            // Messagesに送信
            world.resource_mut::<bevy_ecs::message::Messages<DragEndEvent>>().send(event.clone());
            
            let path = build_bubble_path(world, entity);
            crate::ecs::pointer::dispatch_event_for_handler::<DragEndEvent, OnDragEnd>(
                world,
                entity,
                &path,
                &event,
                |h| h.0,
            );
            
            // JustEnded → Idle遷移
            reset_to_idle();
        }
        
        _ => {}
    }
}
