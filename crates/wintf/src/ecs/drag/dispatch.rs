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
///
/// DraggingStateとPointerStateを持つエンティティを監視し、ドラッグイベントを配信する。
pub fn dispatch_drag_events(world: &mut World) {
    // thread_local DragStateを確認（JustStarted/JustEndedの検出用）
    let state_snapshot = read_drag_state(|state| state.clone());
    
    match state_snapshot {
        DragState::JustStarted { entity, start_pos, .. } => {
            // DragStartEvent送信
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
            
            world.resource_mut::<bevy_ecs::message::Messages<DragStartEvent>>().send(event.clone());
            
            let path = build_bubble_path(world, entity);
            crate::ecs::pointer::dispatch_event_for_handler::<DragStartEvent, OnDragStart>(
                world,
                entity,
                &path,
                &event,
                |h| h.0,
            );
            
            // JustStarted → Dragging遷移
            update_dragging(start_pos);
        }
        
        DragState::JustEnded { entity, position, cancelled } => {
            // DragEndEvent送信
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
    
    // DraggingStateを持つ全エンティティをクエリ（Dragging中のDragEvent配信）
    // PointerStateは持っていないかもしれないので、別途検索する
    let dragging_entities: Vec<(Entity, crate::ecs::drag::DraggingState)> = {
        let mut query = world.query::<(Entity, &crate::ecs::drag::DraggingState)>();
        query.iter(world)
            .map(|(e, ds)| (e, *ds))
            .collect()
    };
    
    tracing::info!("[dispatch_drag_events] Found {} dragging entities", dragging_entities.len());
    
    for (entity, mut dragging_state) in dragging_entities {
        // PointerStateを取得（同じエンティティにあるはず）
        let current_pos = if let Ok(entity_ref) = world.get_entity(entity) {
            if let Some(ps) = entity_ref.get::<crate::ecs::pointer::PointerState>() {
                ps.screen_point
            } else {
                // PointerStateがなければスキップ
                tracing::warn!(entity = ?entity, "[dispatch_drag_events] No PointerState found");
                continue;
            }
        } else {
            continue;
        };
        
        // 前回フレームとの差分を計算
        let delta = PhysicalPoint::new(
            current_pos.x - dragging_state.prev_frame_pos.x,
            current_pos.y - dragging_state.prev_frame_pos.y,
        );
        
        // デルタが0なら送信不要
        if delta.x == 0 && delta.y == 0 {
            continue;
        }
        
        let event = DragEvent {
            target: entity,
            delta,
            position: current_pos,
            is_primary: true,
            timestamp: Instant::now(),
        };
        
        tracing::info!(
            entity = ?entity,
            delta_x = delta.x,
            delta_y = delta.y,
            "[DragEvent] Dispatching"
        );
        
        world.resource_mut::<bevy_ecs::message::Messages<DragEvent>>().send(event.clone());
        
        let path = build_bubble_path(world, entity);
        crate::ecs::pointer::dispatch_event_for_handler::<DragEvent, OnDrag>(
            world,
            entity,
            &path,
            &event,
            |h| h.0,
        );
        
        // prev_frame_posを更新
        if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
            if let Some(mut ds) = entity_mut.get_mut::<crate::ecs::drag::DraggingState>() {
                ds.prev_frame_pos = current_pos;
            }
        }
    }
}
