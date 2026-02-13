//! ドラッグ状態管理
//!
//! thread_local! + RefCellパターンでwndproc層のドラッグ状態を管理する。

use crate::ecs::drag::DragConstraint;
use crate::ecs::pointer::PhysicalPoint;
use bevy_ecs::entity::Entity;
use std::cell::RefCell;
use std::time::Instant;
use windows::Win32::Foundation::{HWND, POINT};

/// ドラッグ状態（thread_local!で管理）
#[derive(Debug, Clone)]
pub enum DragState {
    /// アイドル状態、ドラッグなし
    Idle,

    /// マウス押下済み、閾値未到達
    Preparing {
        /// ドラッグ対象エンティティ
        entity: Entity,
        /// マウス押下位置（物理ピクセル、スクリーン座標）
        start_pos: PhysicalPoint,
        /// 押下時刻
        start_time: Instant,
    },

    /// ドラッグ開始直後（1フレームのみ）
    JustStarted {
        /// ドラッグ対象エンティティ
        entity: Entity,
        /// ドラッグ開始位置
        start_pos: PhysicalPoint,
        /// 現在位置
        current_pos: PhysicalPoint,
        /// 開始時刻
        start_time: Instant,
    },

    /// ドラッグ中、閾値到達済み
    Dragging {
        /// ドラッグ対象エンティティ
        entity: Entity,
        /// ドラッグ開始位置
        start_pos: PhysicalPoint,
        /// 現在位置
        current_pos: PhysicalPoint,
        /// 前回位置
        prev_pos: PhysicalPoint,
        /// 開始時刻
        start_time: Instant,
        // --- WndProc レベルドラッグ用の新規フィールド ---
        /// Window の Win32 ハンドル
        hwnd: HWND,
        /// ドラッグ開始時のウィンドウ位置（クライアント領域スクリーン座標）
        initial_window_pos: POINT,
        /// DragConfig.move_window のキャッシュ
        move_window: bool,
        /// DragConstraint のキャッシュ
        constraint: Option<DragConstraint>,
    },

    /// ドラッグ終了直後（1フレームのみ）
    JustEnded {
        /// ドラッグ対象エンティティ
        entity: Entity,
        /// 終了位置
        position: PhysicalPoint,
        /// キャンセルされたか
        cancelled: bool,
    },
}

thread_local! {
    /// グローバルドラッグ状態（単一ドラッグのみ）
    static DRAG_STATE: RefCell<DragState> = RefCell::new(DragState::Idle);
}

/// ドラッグ状態を更新する（wndprocハンドラから呼ばれる）
pub fn update_drag_state<F, R>(f: F) -> R
where
    F: FnOnce(&mut DragState) -> R,
{
    DRAG_STATE.with(|state| {
        let mut state = state.borrow_mut();
        f(&mut *state)
    })
}

/// ドラッグ状態を取得する（読み取り専用）
pub fn read_drag_state<F, R>(f: F) -> R
where
    F: FnOnce(&DragState) -> R,
{
    DRAG_STATE.with(|state| {
        let state = state.borrow();
        f(&*state)
    })
}

/// ドラッグ開始準備（WM_LBUTTONDOWN時）
#[inline]
pub fn start_preparing(entity: Entity, pos: PhysicalPoint) {
    update_drag_state(|state| {
        // 既にドラッグ中の場合は無視（複数ボタン同時ドラッグ禁止）
        // JustEndedは許可（前回のドラッグが終了した後の新しいドラッグ）
        if matches!(
            state,
            DragState::Preparing { .. }
                | DragState::JustStarted { .. }
                | DragState::Dragging { .. }
        ) {
            tracing::debug!("[drag] Already dragging, ignoring new button press");
            return;
        }

        *state = DragState::Preparing {
            entity,
            start_pos: pos,
            start_time: Instant::now(),
        };

        tracing::debug!(
            entity = ?entity,
            x = pos.x,
            y = pos.y,
            "[start_preparing] DragState -> Preparing"
        );
    });
}

/// ドラッグ開始（閾値到達時）
#[inline]
pub fn start_dragging(current_pos: PhysicalPoint) {
    update_drag_state(|state| {
        if let DragState::Preparing {
            entity,
            start_pos,
            start_time,
        } = *state
        {
            *state = DragState::JustStarted {
                entity,
                start_pos,
                current_pos,
                start_time,
            };

            tracing::debug!(
                entity = ?entity,
                start_x = start_pos.x,
                start_y = start_pos.y,
                current_x = current_pos.x,
                current_y = current_pos.y,
                "[drag] Dragging started"
            );
        }
    });
}

/// ドラッグ移動（WM_MOUSEMOVE時）
///
/// JustStarted → Dragging 遷移時に WindowDragContextResource から
/// HWND・初期位置・制約情報を読み取り、DragState::Dragging にセットする。
#[inline]
pub fn update_dragging(
    current_pos: PhysicalPoint,
    drag_context: Option<&crate::ecs::drag::WindowDragContextResource>,
) {
    update_drag_state(|state| match state {
        DragState::JustStarted {
            entity,
            start_pos,
            start_time,
            ..
        } => {
            // WindowDragContextResource から Window 情報を読み取り
            let (hwnd, initial_window_pos, move_window, constraint) =
                if let Some(ctx_res) = drag_context {
                    if let Some(ctx) = ctx_res.get() {
                        (
                            ctx.hwnd.unwrap_or(HWND::default()),
                            ctx.initial_window_pos.unwrap_or(POINT { x: 0, y: 0 }),
                            ctx.move_window,
                            ctx.constraint,
                        )
                    } else {
                        (HWND::default(), POINT { x: 0, y: 0 }, false, None)
                    }
                } else {
                    (HWND::default(), POINT { x: 0, y: 0 }, false, None)
                };

            tracing::debug!(
                entity = ?*entity,
                hwnd = format!("0x{:X}", hwnd.0 as usize),
                initial_x = initial_window_pos.x,
                initial_y = initial_window_pos.y,
                move_window = move_window,
                "[update_dragging] JustStarted -> Dragging with WindowDragContext"
            );

            *state = DragState::Dragging {
                entity: *entity,
                start_pos: *start_pos,
                current_pos,
                prev_pos: current_pos,
                start_time: *start_time,
                hwnd,
                initial_window_pos,
                move_window,
                constraint,
            };
        }
        DragState::Dragging {
            current_pos: old_pos,
            ..
        } => {
            let prev_pos = *old_pos;
            if let DragState::Dragging {
                entity,
                start_pos,
                start_time,
                hwnd,
                initial_window_pos,
                move_window,
                constraint,
                ..
            } = state.clone()
            {
                *state = DragState::Dragging {
                    entity,
                    start_pos,
                    current_pos,
                    prev_pos,
                    start_time,
                    hwnd,
                    initial_window_pos,
                    move_window,
                    constraint,
                };
            }
        }
        _ => {}
    });
}

/// ドラッグ終了（WM_LBUTTONUP時）
#[inline]
pub fn end_dragging(position: PhysicalPoint, cancelled: bool) {
    update_drag_state(|state| match state {
        DragState::Preparing { entity, .. }
        | DragState::JustStarted { entity, .. }
        | DragState::Dragging { entity, .. } => {
            let entity = *entity;
            *state = DragState::JustEnded {
                entity,
                position,
                cancelled,
            };

            tracing::debug!(
                entity = ?entity,
                x = position.x,
                y = position.y,
                cancelled,
                "[drag] Dragging ended"
            );
        }
        _ => {}
    });
}

/// ドラッグキャンセル（ESCキー、WM_CANCELMODE時）
#[inline]
pub fn cancel_dragging() {
    update_drag_state(|state| match state {
        DragState::Preparing {
            entity, start_pos, ..
        }
        | DragState::JustStarted {
            entity, start_pos, ..
        }
        | DragState::Dragging {
            entity, start_pos, ..
        } => {
            let entity = *entity;
            let position = *start_pos;
            *state = DragState::JustEnded {
                entity,
                position,
                cancelled: true,
            };

            tracing::debug!(
                entity = ?entity,
                "[drag] Dragging cancelled"
            );
        }
        _ => {}
    });
}

/// ドラッグ状態をIdleにリセット（dispatch_drag_events後）
#[inline]
pub fn reset_to_idle() {
    update_drag_state(|state| {
        if matches!(state, DragState::JustEnded { .. }) {
            *state = DragState::Idle;
        }
    });
}

/// ドラッグ準備中をDraggingに遷移させるか判定する
#[inline]
pub fn check_threshold(current_pos: PhysicalPoint, threshold: i32) -> bool {
    read_drag_state(|state| {
        if let DragState::Preparing { start_pos, .. } = state {
            let dx = current_pos.x - start_pos.x;
            let dy = current_pos.y - start_pos.y;
            let distance_sq = dx * dx + dy * dy;
            let threshold_sq = threshold * threshold;
            let result = distance_sq >= threshold_sq;

            tracing::debug!(
                dx,
                dy,
                distance_sq,
                threshold_sq,
                result,
                "[check_threshold]"
            );

            result
        } else {
            tracing::warn!(state = ?state, "[check_threshold] Not in Preparing state");
            false
        }
    })
}
