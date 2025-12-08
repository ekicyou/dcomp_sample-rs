//! ドラッグイベントディスパッチ
//!
//! DragStateからイベントを生成し、Phase<T>でTunnel/Bubble配信する。

use bevy_ecs::prelude::*;
use bevy_ecs::message::Message;
use std::time::Instant;
use crate::ecs::pointer::{Phase, PhysicalPoint};
use super::{DragState, DraggingMarker};
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
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct OnDragStart(pub Box<dyn Fn(Phase<DragStartEvent>, &mut World) + Send + Sync>);

/// ドラッグ中ハンドラコンポーネント
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct OnDrag(pub Box<dyn Fn(Phase<DragEvent>, &mut World) + Send + Sync>);

/// ドラッグ終了ハンドラコンポーネント
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct OnDragEnd(pub Box<dyn Fn(Phase<DragEndEvent>, &mut World) + Send + Sync>);

/// Phase<T>でイベントを配信する汎用関数
fn dispatch_event<E: Message + Clone, H: Component>(
    world: &mut World,
    event: E,
    target: Entity,
    call_handler: impl Fn(&H, Phase<E>, &mut World),
) -> Option<Entity> {
    use bevy_ecs::hierarchy::ChildOf;
    
    // 親チェーン構築
    let mut path = vec![target];
    let mut current = target;
    while let Some(child_of) = world.get::<ChildOf>(current) {
        let parent = child_of.parent();
        path.push(parent);
        current = parent;
    }
    
    let mut sender_entity: Option<Entity> = None;
    
    // Tunnelフェーズ（親→子）
    for &entity in path.iter().rev() {
        if world.get_entity(entity).is_err() {
            return sender_entity;
        }
        
        // ハンドラがあるかチェック
        let has_handler = world.get::<H>(entity).is_some();
        if has_handler {
            // Worldから一時的に借用を外してハンドラを呼び出す
            // SAFETY: この関数は排他的アクセスを保証している
            let world_ptr = world as *mut World;
            unsafe {
                if let Some(handler_comp) = (*world_ptr).get::<H>(entity) {
                    call_handler(handler_comp, Phase::Tunnel(event.clone()), &mut *world_ptr);
                }
            }
            
            if sender_entity.is_none() {
                sender_entity = Some(entity);
            }
        }
    }
    
    // Bubbleフェーズ（子→親）
    for &entity in path.iter() {
        if world.get_entity(entity).is_err() {
            return sender_entity;
        }
        
        let has_handler = world.get::<H>(entity).is_some();
        if has_handler {
            let world_ptr = world as *mut World;
            unsafe {
                if let Some(handler_comp) = (*world_ptr).get::<H>(entity) {
                    call_handler(handler_comp, Phase::Bubble(event.clone()), &mut *world_ptr);
                }
            }
            
            if sender_entity.is_none() {
                sender_entity = Some(entity);
            }
        }
    }
    
    sender_entity
}

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
            
            // Phase<T>配信し、最初にハンドラを実行したエンティティを取得
            let sender = dispatch_event::<DragStartEvent, OnDragStart>(
                world,
                event,
                entity,
                |h, phase, w| (h.0)(phase, w),
            );
            
            // DraggingMarker挿入
            if let Some(sender_entity) = sender {
                if let Ok(mut entity_mut) = world.get_entity_mut(sender_entity) {
                    entity_mut.insert(DraggingMarker { sender: sender_entity });
                    
                    tracing::debug!(
                        sender = ?sender_entity,
                        "[DraggingMarker] Inserted"
                    );
                }
            }
            
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
            
            dispatch_event::<DragEvent, OnDrag>(
                world,
                event,
                entity,
                |h, phase, w| (h.0)(phase, w),
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
            
            dispatch_event::<DragEndEvent, OnDragEnd>(
                world,
                event,
                entity,
                |h, phase, w| (h.0)(phase, w),
            );
            
            // JustEnded → Idle遷移
            reset_to_idle();
        }
        
        _ => {}
    }
}
