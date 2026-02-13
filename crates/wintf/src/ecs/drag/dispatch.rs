//! ドラッグイベントディスパッチ
//!
//! DragStateからイベントを生成し、Phase<T>でTunnel/Bubble配信する。

use crate::ecs::pointer::{EventHandler, PhysicalPoint, build_bubble_path};
use bevy_ecs::message::Message;
use bevy_ecs::prelude::*;
use std::time::Instant;

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
    let flush_result = world
        .resource::<crate::ecs::drag::DragAccumulatorResource>()
        .flush();

    let Some(flush_result) = flush_result else {
        return;
    };

    // 状態遷移イベントを処理
    if let Some(transition) = flush_result.transition {
        match transition {
            crate::ecs::drag::DragTransition::Started {
                entity,
                start_pos,
                timestamp,
            } => {
                // 親階層からWindowエンティティを探索し、HWND・位置・DragConfig情報を取得
                let mut current = entity;
                let mut window_entity: Option<bevy_ecs::entity::Entity> = None;
                let mut initial_window_pos = windows::Win32::Foundation::POINT { x: 0, y: 0 };
                let mut move_window = false;
                let mut constraint: Option<crate::ecs::drag::DragConstraint> = None;
                let mut hwnd: Option<windows::Win32::Foundation::HWND> = None;

                loop {
                    if world.get::<crate::ecs::window::Window>(current).is_some() {
                        window_entity = Some(current);

                        // WindowHandle.hwnd を取得
                        if let Some(wh) = world.get::<crate::ecs::window::WindowHandle>(current) {
                            hwnd = Some(wh.hwnd);

                            // WindowPos.position（クライアント領域座標）をウィンドウ座標に変換
                            // SetWindowPos はウィンドウ枠を含むウィンドウ座標を期待するため
                            if let Some(wp) = world.get::<crate::ecs::window::WindowPos>(current) {
                                if let Some(pos) = wp.position {
                                    let size = wp.size.unwrap_or_default();
                                    if let Ok((wx, wy, _, _)) =
                                        wh.client_to_window_coords(pos, size)
                                    {
                                        initial_window_pos =
                                            windows::Win32::Foundation::POINT { x: wx, y: wy };
                                    }
                                }
                            }
                        } else {
                            // WindowHandle がない場合はフォールバック
                            if let Some(wp) = world.get::<crate::ecs::window::WindowPos>(current) {
                                if let Some(pos) = wp.position {
                                    initial_window_pos = pos;
                                }
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

                // DragConfig.move_window と DragConstraint を取得（ドラッグ対象エンティティから）
                if let Some(dc) = world.get::<crate::ecs::drag::DragConfig>(entity) {
                    move_window = dc.move_window;
                }
                if let Some(dc) = world.get::<crate::ecs::drag::DragConstraint>(entity) {
                    constraint = Some(*dc);
                }

                // WindowDragContextResource に書き込み（wndprocスレッドでの読み取り用）
                if let Some(ctx_res) =
                    world.get_resource::<crate::ecs::drag::WindowDragContextResource>()
                {
                    ctx_res.set(crate::ecs::drag::WindowDragContext {
                        hwnd,
                        initial_window_pos: Some(initial_window_pos),
                        move_window,
                        constraint,
                    });
                }

                // DraggingStateコンポーネント挿入（initial_insetはinitial_window_posに置き換え）
                if let Ok(mut entity_mut) = world.get_entity_mut(entity) {
                    entity_mut.insert(crate::ecs::drag::DraggingState {
                        drag_start_pos: start_pos,
                        initial_inset: (initial_window_pos.x as f32, initial_window_pos.y as f32),
                        prev_frame_pos: start_pos,
                    });

                    tracing::debug!(
                        entity = ?entity,
                        initial_window_x = initial_window_pos.x,
                        initial_window_y = initial_window_pos.y,
                        move_window = move_window,
                        "[dispatch_drag_events] DraggingState inserted"
                    );
                }

                // WindowDragging マーカーを Window entity に insert
                if let Some(we) = window_entity {
                    if let Ok(mut window_mut) = world.get_entity_mut(we) {
                        window_mut.insert(crate::ecs::drag::WindowDragging);
                        tracing::debug!(
                            window_entity = ?we,
                            "[dispatch_drag_events] WindowDragging marker inserted"
                        );
                    }
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

                world
                    .resource_mut::<bevy_ecs::message::Messages<DragStartEvent>>()
                    .write(event.clone());

                let path = build_bubble_path(world, entity);
                crate::ecs::pointer::dispatch_event_for_handler::<DragStartEvent, OnDragStart>(
                    world,
                    entity,
                    &path,
                    &event,
                    |h| h.0,
                );

                // JustStarted→Dragging遷移（次のWM_MOUSEMOVEでDragEventが発火できるように）
                let ctx_res = world.get_resource::<crate::ecs::drag::WindowDragContextResource>();
                super::state::update_dragging(start_pos, ctx_res);
            }

            crate::ecs::drag::DragTransition::Ended {
                entity,
                end_pos,
                cancelled,
            } => {
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

                world
                    .resource_mut::<bevy_ecs::message::Messages<DragEndEvent>>()
                    .write(event.clone());

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

                // Window entity を探索して WindowDragging を remove + WindowPos を最終位置で更新
                {
                    let mut current = entity;
                    loop {
                        if world.get::<crate::ecs::window::Window>(current).is_some() {
                            // WindowDragging マーカーを remove
                            if let Ok(mut window_mut) = world.get_entity_mut(current) {
                                window_mut.remove::<crate::ecs::drag::WindowDragging>();
                                tracing::debug!(
                                    window_entity = ?current,
                                    "[dispatch_drag_events] WindowDragging marker removed"
                                );
                            }

                            // WindowPos.position を DerefMut で更新（Changed<WindowPos> 発火）
                            // これにより PostLayout の sync_window_arrangement_from_window_pos が
                            // Arrangement.offset を正しく更新する
                            if let Ok(mut window_mut) = world.get_entity_mut(current) {
                                if let Some(wp) = window_mut.get::<crate::ecs::window::WindowPos>()
                                {
                                    let current_pos = wp.position;
                                    tracing::debug!(
                                        window_entity = ?current,
                                        current_pos = ?current_pos,
                                        "[dispatch_drag_events] Syncing final WindowPos"
                                    );
                                }
                                // DerefMut アクセスで Changed<WindowPos> を明示的に発火
                                if let Some(mut wp) =
                                    window_mut.get_mut::<crate::ecs::window::WindowPos>()
                                {
                                    // 現在のWindowPosは WM_WINDOWPOSCHANGED で既に更新されているため
                                    // 値自体は変更しないが、DerefMut を通じて Changed を発火させる
                                    wp.set_changed();
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
                }

                // WindowDragContextResource をクリア
                if let Some(ctx_res) =
                    world.get_resource::<crate::ecs::drag::WindowDragContextResource>()
                {
                    ctx_res.clear();
                }
            }
        }
    }

    // 累積デルタが非ゼロなら DragEvent 配信
    if let Some(entity) = flush_result.current_dragging_entity {
        if flush_result.delta.x != 0 || flush_result.delta.y != 0 {
            // DraggingStateからstart_posを取得
            let start_pos = world
                .get::<crate::ecs::drag::DraggingState>(entity)
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

            world
                .resource_mut::<bevy_ecs::message::Messages<DragEvent>>()
                .write(event.clone());

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
                if let Some(mut dragging_state) =
                    entity_mut.get_mut::<crate::ecs::drag::DraggingState>()
                {
                    dragging_state.prev_frame_pos = flush_result.current_position;
                }
            }
        }
    }
}
