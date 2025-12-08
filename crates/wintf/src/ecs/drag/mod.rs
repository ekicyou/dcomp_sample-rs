//! ドラッグ操作モジュール
//!
//! ウィンドウドラッグ移動機能を提供する。
//! wndproc層でマウス入力を監視し、ドラッグ開始の検出（5px移動閾値）、
//! ドラッグ中の継続的な位置追跡、ドラッグ終了を検知する。

mod state;
mod dispatch;
mod systems;

pub use state::{DragState, update_drag_state, read_drag_state, reset_to_idle, 
                start_preparing, start_dragging, update_dragging, end_dragging, 
                cancel_dragging, check_threshold};
pub use dispatch::{
    dispatch_drag_events, OnDragStart, OnDrag, OnDragEnd,
    DragStartEvent, DragEvent, DragEndEvent,
};
pub use systems::{
    apply_window_drag_movement, cleanup_drag_state,
};

use bevy_ecs::prelude::*;
use crate::ecs::pointer::PhysicalPoint;

/// ドラッグ設定コンポーネント
///
/// エンティティごとのドラッグ設定を保持する。
#[derive(Component, Clone, Debug)]
#[component(storage = "SparseSet")]
pub struct DragConfig {
    /// ドラッグ開始閾値（物理ピクセル）、デフォルト5px
    pub threshold: i32,
    /// ドラッグ有効フラグ
    pub enabled: bool,
    /// 有効なボタン（左ボタンのみデフォルト）
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
}

impl Default for DragConfig {
    fn default() -> Self {
        Self {
            threshold: 5,
            enabled: true,
            left_button: true,
            right_button: false,
            middle_button: false,
        }
    }
}

/// ドラッグ状態コンポーネント
///
/// エンティティがドラッグ中であることを示す。
/// PointerStateと組み合わせて使用し、ECSフレーム間のデルタ計算に使う。
/// このコンポーネントの存在自体がドラッグ中であることを意味する。
#[derive(Component, Debug, Clone, Copy)]
#[component(storage = "SparseSet")]
pub struct DraggingState {
    /// ドラッグ開始位置（スクリーン座標）
    pub drag_start_pos: PhysicalPoint,
    /// 前回ECSフレームの位置（デルタ計算用）
    pub prev_frame_pos: PhysicalPoint,
}

/// ドラッグ制約コンポーネント
///
/// ドラッグ移動の範囲制約を定義する。
#[derive(Component, Clone, Copy, Debug)]
#[component(storage = "SparseSet")]
pub struct DragConstraint {
    /// X座標の最小値（物理ピクセル、Noneで制約なし）
    pub min_x: Option<i32>,
    /// X座標の最大値（物理ピクセル、Noneで制約なし）
    pub max_x: Option<i32>,
    /// Y座標の最小値（物理ピクセル、Noneで制約なし）
    pub min_y: Option<i32>,
    /// Y座標の最大値（物理ピクセル、Noneで制約なし）
    pub max_y: Option<i32>,
}

impl DragConstraint {
    /// 制約を適用した座標を返す
    pub fn apply(&self, x: i32, y: i32) -> (i32, i32) {
        let x = self.min_x.map_or(x, |min| x.max(min));
        let x = self.max_x.map_or(x, |max| x.min(max));
        let y = self.min_y.map_or(y, |min| y.max(min));
        let y = self.max_y.map_or(y, |max| y.min(max));
        (x, y)
    }
}
