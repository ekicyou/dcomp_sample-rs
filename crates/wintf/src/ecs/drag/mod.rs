//! ドラッグ操作モジュール
//!
//! ウィンドウドラッグ移動機能を提供する。
//! wndproc層でマウス入力を監視し、ドラッグ開始の検出（5px移動閾値）、
//! ドラッグ中の継続的な位置追跡、ドラッグ終了を検知する。

mod accumulator;
mod context;
mod dispatch;
mod state;
mod systems;

pub use accumulator::{DragAccumulator, DragAccumulatorResource, DragTransition, FlushResult};
pub use context::{WindowDragContext, WindowDragContextResource};
pub use dispatch::{
    DragEndEvent, DragEvent, DragStartEvent, OnDrag, OnDragEnd, OnDragStart, dispatch_drag_events,
};
pub use state::{
    DragState, cancel_dragging, check_threshold, end_dragging, read_drag_state, reset_to_idle,
    start_dragging, start_preparing, update_drag_state, update_dragging,
};
pub use systems::cleanup_drag_state;

use crate::ecs::pointer::PhysicalPoint;
use bevy_ecs::prelude::*;

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
    /// ドラッグ時にウィンドウを自動移動するか
    ///
    /// `true`の場合、wndprocレベルで直接`SetWindowPos`を呼び出し、
    /// ネイティブ同等のドラッグ性能を実現する。
    /// `false`の場合、アプリケーション側のOnDragハンドラで移動処理を実装する必要がある。
    pub move_window: bool,
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
            move_window: true,
            left_button: true,
            right_button: false,
            middle_button: false,
        }
    }
}

/// ドラッグ状態コンポーネント
///
/// エンティティがドラッグ中であることを示す。
/// PointerStateと組み合わせて使用し、ドラッグ開始時の情報を保持する。
/// このコンポーネントの存在自体がドラッグ中であることを意味する。
#[derive(Component, Debug, Clone, Copy)]
#[component(storage = "SparseSet")]
pub struct DraggingState {
    /// ドラッグ開始位置（スクリーン座標、物理ピクセル）
    pub drag_start_pos: PhysicalPoint,
    /// ドラッグ開始時のBoxStyle.inset (left, top)（物理ピクセル）
    pub initial_inset: (f32, f32),
    /// 前回ECSフレームの位置（デルタ計算用、現在は未使用）
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

/// Windowエンティティがドラッグ中であることを示すマーカーコンポーネント。
///
/// `Added<WindowDragging>` でドラッグ開始、`RemovedComponents<WindowDragging>` でドラッグ終了を検知。
/// `dispatch_drag_events` の Started/Ended で insert/remove される。
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct WindowDragging;
