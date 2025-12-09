//! ドラッグイベントディスパッチ
//!
//! DragStateからイベントを生成し、Phase<T>でTunnel/Bubble配信する。

use bevy_ecs::prelude::*;
use bevy_ecs::message::Message;
use std::time::Instant;
use crate::ecs::pointer::{PhysicalPoint, build_bubble_path, EventHandler};

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
    /// ドラッグ開始位置（物理ピクセル、スクリーン座標）
    pub start_position: PhysicalPoint,
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
/// DragAccumulatorResourceから累積量をflushしてドラッグイベントを配信する。
pub fn dispatch_drag_events(world: &mut World) {
    // DragAccumulatorResourceをflush
    let flush_result = world.resource::<crate::ecs::drag::DragAccumulatorResource>()
        .flush();
    
    let Some(flush_result) = flush_result else {
        return;
    };
    
    // 状態遷移イベントを処理
    if let Some(transition) = flush_result.transition {
        match transition {
            crate::ecs::drag::DragTransition::Started { entity, start_pos, timestamp } => {
                // Windowエンティティを探索してBoxStyle.insetを取得
                let mut current = entity;
                let mut initial_inset = (0.0, 0.0);
                loop {
                    if world.get::<crate::ecs::window::Window>(current).is_some() {
                        // Windowが見つかった、BoxStyle.insetを取得
                        if let Some(box_style) = world.get::<crate::ecs::layout::BoxStyle>(current) {
                            if let Some(inset) = &box_style.inset {
                                initial_inset.0 = match inset.0.left {
                                    crate::ecs::layout::LengthPercentageAuto::Px(val) => val,
                                    _ => 0.0,
                                };
                                initial_inset.1 = match inset.0.top {
                                    crate::ecs::layout::LengthPercentageAuto::Px(val) => val,
                                    _ => 0.0,
                                };
                            }
                        }
                        break;
                    }
                    if let Some(child_of) = world.get::<bevy_ecs::hierarchy::ChildOf>(current) {
                        current = child_of.parent();
                    } else {
                        break;
                    }
                }
                
                // DraggingStateコンポーネント挿入
                if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                    entity_mut.insert(crate::ecs::drag::DraggingState {
                        drag_start_pos: start_pos,
                        initial_inset,
                        prev_frame_pos: start_pos,
                    });
                    
                    tracing::debug!(
                        entity = ?entity,
                        initial_inset_left = initial_inset.0,
                        initial_inset_top = initial_inset.1,
                        "[dispatch_drag_events] DraggingState inserted"
                    );
                }
                
                // DragStartEvent送信
                let event = DragStartEvent {
                    target: entity,
                    position: start_pos,
                    is_primary: true,
                    timestamp,
                };
                
                tracing::info!(
                    entity = ?entity,
                    x = start_pos.x,
                    y = start_pos.y,
                    "[DragStartEvent] Dispatching"
                );
                
                world.resource_mut::<bevy_ecs::message::Messages<DragStartEvent>>().write(event.clone());
                
                let path = build_bubble_path(world, entity);
                crate::ecs::pointer::dispatch_event_for_handler::<DragStartEvent, OnDragStart>(
                    world,
                    entity,
                    &path,
                    &event,
                    |h| h.0,
                );
                
                // JustStarted→Dragging遷移（次のWM_MOUSEMOVEでDragEventが発火できるように）
                super::state::update_dragging(start_pos);
            }
            
            crate::ecs::drag::DragTransition::Ended { entity, end_pos, cancelled } => {
                // DragEndEvent送信
                let event = DragEndEvent {
                    target: entity,
                    position: end_pos,
                    cancelled,
                    is_primary: true,
                    timestamp: Instant::now(),
                };
                
                tracing::info!(
                    entity = ?entity,
                    x = end_pos.x,
                    y = end_pos.y,
                    cancelled,
                    "[DragEndEvent] Dispatching"
                );
                
                world.resource_mut::<bevy_ecs::message::Messages<DragEndEvent>>().write(event.clone());
                
                let path = build_bubble_path(world, entity);
                crate::ecs::pointer::dispatch_event_for_handler::<DragEndEvent, OnDragEnd>(
                    world,
                    entity,
                    &path,
                    &event,
                    |h| h.0,
                );
                
                // DraggingStateコンポーネント削除
                if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                    entity_mut.remove::<crate::ecs::drag::DraggingState>();
                    
                    tracing::debug!(
                        entity = ?entity,
                        "[dispatch_drag_events] DraggingState removed"
                    );
                }
            }
        }
    }
    
    // 累積デルタが非ゼロなら DragEvent 配信
    if let Some(entity) = flush_result.current_dragging_entity {
        if flush_result.delta.x != 0 || flush_result.delta.y != 0 {
            // DraggingStateからstart_posを取得
            let start_pos = world.get::<crate::ecs::drag::DraggingState>(entity)
                .map(|ds| ds.drag_start_pos)
                .unwrap_or(flush_result.current_position);
            
            // DragEvent送信
            let event = DragEvent {
                target: entity,
                start_position: start_pos,
                position: flush_result.current_position,
                is_primary: true,
                timestamp: Instant::now(),
            };
            
            tracing::trace!(
                entity = ?entity,
                start_x = start_pos.x,
                start_y = start_pos.y,
                current_x = flush_result.current_position.x,
                current_y = flush_result.current_position.y,
                delta_x = flush_result.current_position.x - start_pos.x,
                delta_y = flush_result.current_position.y - start_pos.y,
                "[DragEvent] Dispatching"
            );
            
            world.resource_mut::<bevy_ecs::message::Messages<DragEvent>>().write(event.clone());
            
            let path = build_bubble_path(world, entity);
            crate::ecs::pointer::dispatch_event_for_handler::<DragEvent, OnDrag>(
                world,
                entity,
                &path,
                &event,
                |h| h.0,
            );
            
            // DraggingState.prev_frame_posを更新
            if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                if let Some(mut dragging_state) = entity_mut.get_mut::<crate::ecs::drag::DraggingState>() {
                    dragging_state.prev_frame_pos = flush_result.current_position;
                }
            }
        }
    }
}
