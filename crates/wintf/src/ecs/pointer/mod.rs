//! ポインター入力モジュール
//!
//! Win32マウスメッセージをECSコンポーネントとして正規化し、
//! ウィンドウレベルのポインター状態管理を提供する。
//!
//! WinUI3 スタイルの命名規則を採用し、将来のタッチ/ペン対応に備える。

mod dispatch;

pub use dispatch::{
    dispatch_pointer_events, dispatch_event_for_handler, build_bubble_path,
    EventHandler, OnPointerEntered, OnPointerExited, OnPointerMoved,
    OnPointerPressed, OnPointerReleased, Phase, PointerEventHandler,
};

use bevy_ecs::prelude::*;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::time::Instant;

// ============================================================================
// 基本型定義
// ============================================================================

/// 物理座標（ピクセル）
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PhysicalPoint {
    pub x: i32,
    pub y: i32,
}

impl PhysicalPoint {
    /// 新しいPhysicalPointを作成
    #[inline]
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }
}

/// ダブルクリック種別（1フレームのみ有効）
///
/// FrameFinalize で None にリセットされる。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DoubleClick {
    #[default]
    None,
    Left,
    Right,
    Middle,
    XButton1,
    XButton2,
}

/// ホイール回転情報（1フレームのみ有効）
///
/// WM_MOUSEWHEEL / WM_MOUSEHWHEEL から透過転送。
/// FrameFinalize でリセットされる。
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct WheelDelta {
    /// 垂直ホイール回転量（WHEEL_DELTA単位、正=上、負=下）
    pub vertical: i16,
    /// 水平ホイール回転量（WHEEL_DELTA単位、正=右、負=左）
    pub horizontal: i16,
}

/// カーソル移動速度（ピクセル/秒）
#[derive(Debug, Clone, Default, PartialEq)]
pub struct CursorVelocity {
    pub x: f32,
    pub y: f32,
    pub magnitude: f32,
}

impl CursorVelocity {
    /// 新しいCursorVelocityを作成
    #[inline]
    pub fn new(x: f32, y: f32) -> Self {
        Self {
            x,
            y,
            magnitude: (x * x + y * y).sqrt(),
        }
    }
}

/// ポインターボタン種別（マウスボタン）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PointerButton {
    Left,
    Right,
    Middle,
    XButton1,
    XButton2,
}

/// 後方互換性エイリアス
#[deprecated(since = "0.1.0", note = "Use PointerButton instead")]
pub type MouseButton = PointerButton;

// ============================================================================
// PointerState コンポーネント
// ============================================================================

/// ポインター状態コンポーネント（WinUI3 スタイル）
///
/// hit_test がヒットしたエンティティに付与される。
/// コンポーネントの存在 = ホバー中。
/// Added<PointerState> で Enter を検出。
///
/// Win32マウスメッセージの情報を透過的にECSに転送する。
/// 情報の解釈（Click判定等）はアプリ側の責務。
///
/// メモリ戦略: SparseSet - 頻繁な挿入/削除
#[derive(Component, Debug, Clone)]
#[component(storage = "SparseSet")]
pub struct PointerState {
    /// スクリーン座標（物理ピクセル）
    pub screen_point: PhysicalPoint,
    /// エンティティローカル座標（物理ピクセル）
    pub local_point: PhysicalPoint,

    // === ボタン押下状態（wParam のビットマスクを透過転送）===
    /// 左ボタン押下中 (MK_LBUTTON)
    pub left_down: bool,
    /// 右ボタン押下中 (MK_RBUTTON)
    pub right_down: bool,
    /// 中ボタン押下中 (MK_MBUTTON)
    pub middle_down: bool,
    /// XButton1 押下中 (MK_XBUTTON1) - 4thボタン
    pub xbutton1_down: bool,
    /// XButton2 押下中 (MK_XBUTTON2) - 5thボタン
    pub xbutton2_down: bool,

    // === 修飾キー状態（wParam から透過転送）===
    /// Shift押下中 (MK_SHIFT)
    pub shift_down: bool,
    /// Ctrl押下中 (MK_CONTROL)
    pub ctrl_down: bool,

    // === ダブルクリック（1フレームのみ有効）===
    /// ダブルクリック検出（FrameFinalizeでNoneにリセット）
    pub double_click: DoubleClick,

    // === ホイール（1フレームのみ有効）===
    /// ホイール回転情報（FrameFinalizeでリセット）
    pub wheel: WheelDelta,

    // === その他 ===
    /// カーソル移動速度
    pub velocity: CursorVelocity,
    /// タイムスタンプ
    pub timestamp: Instant,
}

impl Default for PointerState {
    fn default() -> Self {
        Self {
            screen_point: PhysicalPoint::default(),
            local_point: PhysicalPoint::default(),
            left_down: false,
            right_down: false,
            middle_down: false,
            xbutton1_down: false,
            xbutton2_down: false,
            shift_down: false,
            ctrl_down: false,
            double_click: DoubleClick::None,
            wheel: WheelDelta::default(),
            velocity: CursorVelocity::default(),
            timestamp: Instant::now(),
        }
    }
}

/// 後方互換性エイリアス
#[deprecated(since = "0.1.0", note = "Use PointerState instead")]
pub type MouseState = PointerState;

// ============================================================================
// PointerLeave マーカー
// ============================================================================

/// ポインター離脱マーカー（1フレーム限り）
///
/// PointerState が削除されたフレームに付与される。
/// FrameFinalize で削除されるため、1フレームのみ存在。
///
/// メモリ戦略: SparseSet - 一時的マーカー
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
#[component(storage = "SparseSet")]
pub struct PointerLeave;

/// 後方互換性エイリアス
#[deprecated(since = "0.1.0", note = "Use PointerLeave instead")]
pub type MouseLeave = PointerLeave;

// ============================================================================
// WindowPointerTracking コンポーネント
// ============================================================================

/// TrackMouseEvent 状態追跡
///
/// ウィンドウエンティティに自動付与される。
/// `true` = TrackMouseEvent(TME_LEAVE) が有効
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct WindowPointerTracking(pub bool);

/// 後方互換性エイリアス
#[deprecated(since = "0.1.0", note = "Use WindowPointerTracking instead")]
pub type WindowMouseTracking = WindowPointerTracking;

// ============================================================================
// PointerBuffer
// ============================================================================

/// 位置サンプル
#[derive(Debug, Clone, Copy)]
pub struct PositionSample {
    pub x: f32,
    pub y: f32,
    pub timestamp: Instant,
}

/// ポインターバッファ（thread_local! で管理）
///
/// WndProc内で複数のWM_MOUSEMOVEが発生する可能性があるため、
/// バッファに蓄積してInputスケジュールで処理する。
#[derive(Debug, Default)]
pub struct PointerBuffer {
    samples: VecDeque<PositionSample>,
}

impl PointerBuffer {
    /// 最大サンプル数（速度計算用）
    const MAX_SAMPLES: usize = 5;

    /// 新しいPointerBufferを作成
    pub fn new() -> Self {
        Self {
            samples: VecDeque::with_capacity(Self::MAX_SAMPLES),
        }
    }

    /// サンプルを追加
    pub fn push(&mut self, sample: PositionSample) {
        if self.samples.len() >= Self::MAX_SAMPLES {
            self.samples.pop_front();
        }
        self.samples.push_back(sample);
    }

    /// 最新のサンプルを取得
    pub fn latest(&self) -> Option<&PositionSample> {
        self.samples.back()
    }

    /// サンプル数を取得
    pub fn len(&self) -> usize {
        self.samples.len()
    }

    /// バッファが空かどうか
    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    /// サンプルをクリア（速度計算用履歴は保持しない）
    pub fn clear(&mut self) {
        self.samples.clear()
    }

    /// 速度計算（最新2サンプル間）
    pub fn calculate_velocity(&self) -> (f32, f32) {
        if self.samples.len() < 2 {
            return (0.0, 0.0);
        }
        let newest = self.samples.back().unwrap();
        let prev = &self.samples[self.samples.len() - 2];
        let dt = newest.timestamp.duration_since(prev.timestamp).as_secs_f32();
        if dt < 0.0001 {
            return (0.0, 0.0);
        }
        ((newest.x - prev.x) / dt, (newest.y - prev.y) / dt)
    }
}

/// 後方互換性エイリアス
#[deprecated(since = "0.1.0", note = "Use PointerBuffer instead")]
pub type MouseBuffer = PointerBuffer;

// ============================================================================
// ButtonBuffer
// ============================================================================

/// ボタンバッファ
///
/// 1 tick 内に複数のボタンイベントが発生する可能性があるため、
/// down/up の発生を記録する。
#[derive(Debug, Clone, Copy, Default)]
pub struct ButtonBuffer {
    /// tick中にDownが発生したか
    pub down_received: bool,
    /// tick中にUpが発生したか
    pub up_received: bool,
}

impl ButtonBuffer {
    /// ボタン押下を記録
    pub fn record_down(&mut self) {
        self.down_received = true;
    }

    /// ボタン解放を記録
    pub fn record_up(&mut self) {
        self.up_received = true;
    }

    /// バッファをリセット
    pub fn reset(&mut self) {
        self.down_received = false;
        self.up_received = false;
    }
}

// ============================================================================
// ホイールバッファ（tick 内累積用）
// ============================================================================

/// ホイールバッファ
///
/// 1 tick 内に複数の WM_MOUSEWHEEL/WM_MOUSEHWHEEL が発生する可能性があるため、
/// デルタを累積する。
#[derive(Debug, Clone, Copy, Default)]
pub struct WheelBuffer {
    /// 垂直ホイール累積
    pub vertical: i16,
    /// 水平ホイール累積
    pub horizontal: i16,
}

impl WheelBuffer {
    /// 垂直ホイール回転を累積
    pub fn add_vertical(&mut self, delta: i16) {
        self.vertical = self.vertical.saturating_add(delta);
    }

    /// 水平ホイール回転を累積
    pub fn add_horizontal(&mut self, delta: i16) {
        self.horizontal = self.horizontal.saturating_add(delta);
    }

    /// バッファをリセット
    pub fn reset(&mut self) {
        self.vertical = 0;
        self.horizontal = 0;
    }
}

// ============================================================================
// thread_local! バッファ
// ============================================================================

thread_local! {
    /// Entity ごとの PointerBuffer
    pub(crate) static POINTER_BUFFERS: RefCell<HashMap<Entity, PointerBuffer>> = RefCell::new(HashMap::new());

    /// Entity × Button ごとの ButtonBuffer
    pub(crate) static BUTTON_BUFFERS: RefCell<HashMap<(Entity, PointerButton), ButtonBuffer>> = RefCell::new(HashMap::new());

    /// Entity ごとの WheelBuffer
    pub(crate) static WHEEL_BUFFERS: RefCell<HashMap<Entity, WheelBuffer>> = RefCell::new(HashMap::new());

    /// Entity ごとの DoubleClick（tick 内で最初に検出されたもの）
    pub(crate) static DOUBLE_CLICK_BUFFERS: RefCell<HashMap<Entity, DoubleClick>> = RefCell::new(HashMap::new());

    /// Entity ごとの修飾キー状態（最新値）
    pub(crate) static MODIFIER_STATE: RefCell<HashMap<Entity, (bool, bool)>> = RefCell::new(HashMap::new());
}

// 後方互換性エイリアス（コンパイル時参照のため関数ではなくマクロは使えない）
// MOUSE_BUFFERS は POINTER_BUFFERS を内部で使用

// ============================================================================
// hit_test 仮スタブ
// ============================================================================

/// hit_test プレースホルダー（Phase 1）
///
/// event-hit-test 完了後に実際の実装に差し替え。
/// Phase 1では常にウィンドウエンティティを返す。
///
/// # Returns
/// - `Some(Entity)`: ヒットしたエンティティ
/// - `None`: ヒットなし（透過）
#[inline]
pub fn hit_test_placeholder(
    _world: &bevy_ecs::world::World,
    window_entity: Entity,
    _position: (f32, f32),
) -> Option<Entity> {
    // Phase 1: 常にウィンドウエンティティを返す
    Some(window_entity)
}

/// hit_test プレースホルダー（ローカル座標変換付き）
///
/// Phase 1ではスクリーン座標をそのまま返す。
#[inline]
pub fn hit_test_with_local_coords(
    _world: &bevy_ecs::world::World,
    window_entity: Entity,
    screen_x: i32,
    screen_y: i32,
) -> Option<(Entity, PhysicalPoint)> {
    // Phase 1: 常にウィンドウエンティティを返し、ローカル座標＝スクリーン座標
    Some((window_entity, PhysicalPoint::new(screen_x, screen_y)))
}

// ============================================================================
// システム
// ============================================================================

/// ポインターバッファ処理システム
///
/// Inputスケジュールで実行され、バッファ内容をPointerStateコンポーネントに反映する。
pub fn process_pointer_buffers(mut query: Query<(Entity, &mut PointerState)>) {
    tracing::trace!("[process_pointer_buffers] Called");
    
    // ButtonBufferの内容をPointerStateに反映（エンティティIDで照合）
    // Note: BUTTON_BUFFERSのリセットはdispatch_pointer_eventsで行われる
    BUTTON_BUFFERS.with(|buffers| {
        let buffers = buffers.borrow();
        
        for (entity, mut pointer) in query.iter_mut() {
            // 各ボタンの処理（DOWN優先ルール）
            for button in [
                PointerButton::Left,
                PointerButton::Right,
                PointerButton::Middle,
                PointerButton::XButton1,
                PointerButton::XButton2,
            ] {
                if let Some(buf) = buffers.get(&(entity, button)) {
                    let is_down = if buf.down_received {
                        true
                    } else if buf.up_received {
                        false
                    } else {
                        // イベントなし - 現在の状態を維持
                        match button {
                            PointerButton::Left => pointer.left_down,
                            PointerButton::Right => pointer.right_down,
                            PointerButton::Middle => pointer.middle_down,
                            PointerButton::XButton1 => pointer.xbutton1_down,
                            PointerButton::XButton2 => pointer.xbutton2_down,
                        }
                    };

                    match button {
                        PointerButton::Left => pointer.left_down = is_down,
                        PointerButton::Right => pointer.right_down = is_down,
                        PointerButton::Middle => pointer.middle_down = is_down,
                        PointerButton::XButton1 => pointer.xbutton1_down = is_down,
                        PointerButton::XButton2 => pointer.xbutton2_down = is_down,
                    }

                    // ログ出力（ボタン状態が変化した場合）
                    if buf.down_received || buf.up_received {
                        tracing::trace!(
                            entity = ?entity,
                            button = ?button,
                            is_down,
                            "[process_pointer_buffers] Button state updated"
                        );
                    }
                }
            }
        }
    });

    for (entity, mut pointer) in query.iter_mut() {
        tracing::trace!(
            entity = ?entity,
            thread_id = ?std::thread::current().id(),
            "[process_pointer_buffers] Checking POINTER_BUFFERS"
        );
        
        // PointerBuffer から位置と速度を取得
        POINTER_BUFFERS.with(|buffers| {
            let mut buffers = buffers.borrow_mut();
            if let Some(buffer) = buffers.get_mut(&entity) {
                tracing::trace!(
                    entity = ?entity,
                    "[process_pointer_buffers] Buffer found"
                );
                
                // 速度計算
                let (vx, vy) = buffer.calculate_velocity();
                pointer.velocity = CursorVelocity::new(vx, vy);

                // 最新位置取得
                if let Some(sample) = buffer.latest() {
                    let old_x = pointer.screen_point.x;
                    let old_y = pointer.screen_point.y;
                    pointer.screen_point = PhysicalPoint::new(sample.x as i32, sample.y as i32);
                    // Note: local_point は hit_test 結果から設定（Phase 1ではscreen_pointと同じ）
                    pointer.local_point = pointer.screen_point;
                    
                    tracing::trace!(
                        entity = ?entity,
                        old_x, old_y,
                        new_x = pointer.screen_point.x,
                        new_y = pointer.screen_point.y,
                        "[process_pointer_buffers] Position updated"
                    );
                }

                // バッファクリア
                buffer.clear();
            } else {
                tracing::trace!(
                    entity = ?entity,
                    "[process_pointer_buffers] No buffer found"
                );
            }
        });

        // WheelBuffer からホイール情報を取得
        WHEEL_BUFFERS.with(|buffers| {
            let mut buffers = buffers.borrow_mut();
            if let Some(buf) = buffers.get_mut(&entity) {
                pointer.wheel = WheelDelta {
                    vertical: buf.vertical,
                    horizontal: buf.horizontal,
                };
                buf.reset();
            }
        });

        // DoubleClick を取得
        DOUBLE_CLICK_BUFFERS.with(|buffers| {
            let mut buffers = buffers.borrow_mut();
            if let Some(dc) = buffers.remove(&entity) {
                pointer.double_click = dc;
            }
        });

        // 修飾キー状態を取得
        MODIFIER_STATE.with(|state| {
            let state = state.borrow();
            if let Some(&(shift, ctrl)) = state.get(&entity) {
                pointer.shift_down = shift;
                pointer.ctrl_down = ctrl;
            }
        });

        pointer.timestamp = Instant::now();
    }
}

/// 後方互換性エイリアス
#[deprecated(since = "0.1.0", note = "Use process_pointer_buffers instead")]
pub fn process_mouse_buffers(query: Query<(Entity, &mut PointerState)>) {
    process_pointer_buffers(query);
}

/// 一時的ポインター状態クリアシステム（FrameFinalize）
///
/// CommitComposition 後に実行され、1フレームのみ有効な状態をリセットする。
pub fn clear_transient_pointer_state(
    mut query: Query<&mut PointerState>,
    mut commands: Commands,
    leave_query: Query<Entity, With<PointerLeave>>,
) {
    // double_click, wheel をリセット（1フレームのみ有効）
    for mut pointer in query.iter_mut() {
        pointer.double_click = DoubleClick::None;
        pointer.wheel = WheelDelta::default();
    }

    // PointerLeave マーカー除去
    for entity in leave_query.iter() {
        commands.entity(entity).remove::<PointerLeave>();
    }
}

/// 後方互換性エイリアス
#[deprecated(since = "0.1.0", note = "Use clear_transient_pointer_state instead")]
pub fn clear_transient_mouse_state(
    query: Query<&mut PointerState>,
    commands: Commands,
    leave_query: Query<Entity, With<PointerLeave>>,
) {
    clear_transient_pointer_state(query, commands, leave_query);
}

// ============================================================================
// デバッグ用監視システム
// ============================================================================

/// PointerState の Added/Changed を監視するデバッグシステム
///
/// Inputスケジュールで実行し、PointerStateの変化をログ出力する。
/// デバッグ用途のため、リリースビルドでは使用しないこと。
pub fn debug_pointer_state_changes(
    added_query: Query<(Entity, &PointerState), Added<PointerState>>,
    changed_query: Query<(Entity, &PointerState), Changed<PointerState>>,
) {
    use tracing::debug;

    // 新規追加（Enter）
    for (entity, pointer) in added_query.iter() {
        debug!(
            entity = ?entity,
            screen_x = pointer.screen_point.x,
            screen_y = pointer.screen_point.y,
            left = pointer.left_down,
            right = pointer.right_down,
            middle = pointer.middle_down,
            shift = pointer.shift_down,
            ctrl = pointer.ctrl_down,
            "[PointerState Added] Enter detected"
        );
    }

    // 変更（移動・ボタン・ホイール等）
    for (entity, pointer) in changed_query.iter() {
        // Added も Changed に含まれるのでスキップ
        // Note: bevy_ecs では Added は Changed のサブセット
        // ここでは移動・ボタン変化のみログ出力したい場合、
        // 別途フラグ管理が必要だが、デバッグ用なので許容

        // ダブルクリック検出時のみログ
        if pointer.double_click != DoubleClick::None {
            debug!(
                entity = ?entity,
                double_click = ?pointer.double_click,
                "[PointerState Changed] DoubleClick detected"
            );
        }

        // ホイール回転時のみログ
        if pointer.wheel.vertical != 0 || pointer.wheel.horizontal != 0 {
            debug!(
                entity = ?entity,
                vertical = pointer.wheel.vertical,
                horizontal = pointer.wheel.horizontal,
                "[PointerState Changed] Wheel detected"
            );
        }
    }
}

/// 後方互換性エイリアス
#[deprecated(since = "0.1.0", note = "Use debug_pointer_state_changes instead")]
pub fn debug_mouse_state_changes(
    added_query: Query<(Entity, &PointerState), Added<PointerState>>,
    changed_query: Query<(Entity, &PointerState), Changed<PointerState>>,
) {
    debug_pointer_state_changes(added_query, changed_query);
}

/// PointerLeave マーカーを監視するデバッグシステム
///
/// Inputスケジュールで実行し、PointerLeaveの付与をログ出力する。
pub fn debug_pointer_leave(leave_query: Query<Entity, Added<PointerLeave>>) {
    use tracing::debug;

    for entity in leave_query.iter() {
        debug!(
            entity = ?entity,
            "[PointerLeave Added] Leave detected"
        );
    }
}

/// 後方互換性エイリアス
#[deprecated(since = "0.1.0", note = "Use debug_pointer_leave instead")]
pub fn debug_mouse_leave(leave_query: Query<Entity, Added<PointerLeave>>) {
    debug_pointer_leave(leave_query);
}

// ============================================================================
// バッファ操作ヘルパー（handlers.rs から使用）
// ============================================================================

/// PointerBufferにサンプルを追加
#[inline]
pub(crate) fn push_pointer_sample(entity: Entity, x: f32, y: f32, timestamp: Instant) {
    tracing::trace!(
        entity = ?entity,
        x, y,
        thread_id = ?std::thread::current().id(),
        "[push_pointer_sample] Sample added"
    );
    POINTER_BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        let buffer = buffers.entry(entity).or_insert_with(PointerBuffer::new);
        buffer.push(PositionSample { x, y, timestamp });
    });
}

/// 後方互換性エイリアス
#[allow(dead_code)]
#[inline]
pub(crate) fn push_mouse_sample(entity: Entity, x: f32, y: f32, timestamp: Instant) {
    push_pointer_sample(entity, x, y, timestamp);
}

/// ButtonBufferにボタン押下を記録
#[inline]
pub(crate) fn record_button_down(entity: Entity, button: PointerButton) {
    BUTTON_BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        let buffer = buffers.entry((entity, button)).or_default();
        buffer.record_down();
        // デバッグ用に info レベルで出力
        tracing::info!(
            entity = ?entity,
            button = ?button,
            "[ButtonBuffer] record_button_down"
        );
    });
}

/// ButtonBufferにボタン解放を記録
#[inline]
pub(crate) fn record_button_up(entity: Entity, button: PointerButton) {
    BUTTON_BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        let buffer = buffers.entry((entity, button)).or_default();
        buffer.record_up();
        // デバッグ用に info レベルで出力
        tracing::info!(
            entity = ?entity,
            button = ?button,
            "[ButtonBuffer] record_button_up"
        );
    });
}

/// WheelBufferに垂直ホイール回転を累積
#[inline]
pub(crate) fn add_wheel_vertical(entity: Entity, delta: i16) {
    WHEEL_BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        let buffer = buffers.entry(entity).or_default();
        buffer.add_vertical(delta);
    });
}

/// WheelBufferに水平ホイール回転を累積
#[inline]
pub(crate) fn add_wheel_horizontal(entity: Entity, delta: i16) {
    WHEEL_BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        let buffer = buffers.entry(entity).or_default();
        buffer.add_horizontal(delta);
    });
}

/// DoubleClickを設定
#[inline]
pub(crate) fn set_double_click(entity: Entity, double_click: DoubleClick) {
    DOUBLE_CLICK_BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        // 最初のダブルクリックのみ記録
        buffers.entry(entity).or_insert(double_click);
    });
}

/// 修飾キー状態を設定
#[inline]
pub(crate) fn set_modifier_state(entity: Entity, shift: bool, ctrl: bool) {
    MODIFIER_STATE.with(|state| {
        let mut state = state.borrow_mut();
        state.insert(entity, (shift, ctrl));
    });
}

// ============================================================================
// テスト
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pointer_buffer_push() {
        let mut buffer = PointerBuffer::new();
        assert!(buffer.is_empty());

        buffer.push(PositionSample {
            x: 10.0,
            y: 20.0,
            timestamp: Instant::now(),
        });
        assert_eq!(buffer.len(), 1);
        assert!(!buffer.is_empty());
    }

    #[test]
    fn test_pointer_buffer_max_samples() {
        let mut buffer = PointerBuffer::new();
        for i in 0..10 {
            buffer.push(PositionSample {
                x: i as f32,
                y: i as f32,
                timestamp: Instant::now(),
            });
        }
        // MAX_SAMPLES (5) を超えないことを確認
        assert_eq!(buffer.len(), PointerBuffer::MAX_SAMPLES);

        // 最新の値が最後に追加されたものであることを確認
        let latest = buffer.latest().unwrap();
        assert_eq!(latest.x, 9.0);
    }

    #[test]
    fn test_velocity_calculation() {
        let mut buffer = PointerBuffer::new();
        let t1 = Instant::now();
        buffer.push(PositionSample {
            x: 0.0,
            y: 0.0,
            timestamp: t1,
        });

        // 1サンプルでは速度は0
        let (vx, vy) = buffer.calculate_velocity();
        assert_eq!(vx, 0.0);
        assert_eq!(vy, 0.0);
    }

    #[test]
    fn test_button_buffer_state() {
        let mut buffer = ButtonBuffer::default();
        assert!(!buffer.down_received);
        assert!(!buffer.up_received);

        buffer.record_down();
        assert!(buffer.down_received);
        assert!(!buffer.up_received);

        buffer.record_up();
        assert!(buffer.down_received);
        assert!(buffer.up_received);

        buffer.reset();
        assert!(!buffer.down_received);
        assert!(!buffer.up_received);
    }

    #[test]
    fn test_wheel_buffer() {
        let mut buffer = WheelBuffer::default();
        assert_eq!(buffer.vertical, 0);
        assert_eq!(buffer.horizontal, 0);

        buffer.add_vertical(120);
        buffer.add_vertical(120);
        assert_eq!(buffer.vertical, 240);

        buffer.add_horizontal(-60);
        assert_eq!(buffer.horizontal, -60);

        buffer.reset();
        assert_eq!(buffer.vertical, 0);
        assert_eq!(buffer.horizontal, 0);
    }

    #[test]
    fn test_cursor_velocity_new() {
        let v = CursorVelocity::new(3.0, 4.0);
        assert_eq!(v.x, 3.0);
        assert_eq!(v.y, 4.0);
        assert_eq!(v.magnitude, 5.0); // 3-4-5 直角三角形
    }

    #[test]
    fn test_pointer_state_default() {
        let state = PointerState::default();
        assert_eq!(state.screen_point, PhysicalPoint::default());
        assert_eq!(state.local_point, PhysicalPoint::default());
        assert!(!state.left_down);
        assert!(!state.right_down);
        assert!(!state.middle_down);
        assert!(!state.xbutton1_down);
        assert!(!state.xbutton2_down);
        assert!(!state.shift_down);
        assert!(!state.ctrl_down);
        assert_eq!(state.double_click, DoubleClick::None);
        assert_eq!(state.wheel, WheelDelta::default());
    }

    #[test]
    fn test_pointer_leave_marker() {
        // PointerLeaveはunitスタイルのマーカーコンポーネント
        let leave1 = PointerLeave;
        let leave2 = PointerLeave;
        assert_eq!(leave1, leave2);
    }

    #[test]
    fn test_window_pointer_tracking_default() {
        let tracking = WindowPointerTracking::default();
        assert!(!tracking.0);

        let tracking_enabled = WindowPointerTracking(true);
        assert!(tracking_enabled.0);
    }

    #[test]
    fn test_physical_point_new() {
        let pt = PhysicalPoint::new(100, 200);
        assert_eq!(pt.x, 100);
        assert_eq!(pt.y, 200);
    }

    #[test]
    fn test_double_click_variants() {
        assert_eq!(DoubleClick::default(), DoubleClick::None);

        // 各バリアントが異なることを確認
        assert_ne!(DoubleClick::Left, DoubleClick::Right);
        assert_ne!(DoubleClick::Middle, DoubleClick::XButton1);
        assert_ne!(DoubleClick::XButton1, DoubleClick::XButton2);
    }

    #[test]
    fn test_wheel_delta_default() {
        let delta = WheelDelta::default();
        assert_eq!(delta.vertical, 0);
        assert_eq!(delta.horizontal, 0);
    }

    #[test]
    fn test_pointer_button_enum() {
        // PointerButtonのHashトレイト実装を確認
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(PointerButton::Left);
        set.insert(PointerButton::Right);
        set.insert(PointerButton::Middle);
        set.insert(PointerButton::XButton1);
        set.insert(PointerButton::XButton2);
        assert_eq!(set.len(), 5);
    }
}

// ============================================================================
// WndProcスレッドからWorldへのデータ転送
// ============================================================================

/// WndProcスレッドのthread_localバッファからWorldのPointerStateに直接データを転送
/// 
/// この関数は`try_tick_world()`の冒頭（Inputスケジュール実行前）で呼ばれ、
/// WndProcスレッド（メインスレッド）で収集したポインター情報を、
/// マルチスレッドで実行されるシステムがアクセスできるように転送する。
pub(crate) fn transfer_buffers_to_world(world: &mut World) {
    // POINTER_BUFFERSからPointerStateへ位置情報を転送
    POINTER_BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        
        for (entity, buffer) in buffers.iter_mut() {
            // 最新位置を取得
            if let Some(sample) = buffer.latest() {
                // 速度計算
                let (vx, vy) = buffer.calculate_velocity();
                
                // Worldから該当エンティティのPointerStateを取得または作成
                if let Some(mut pointer_state) = world.get_mut::<PointerState>(*entity) {
                    // 既存のPointerStateを更新
                    pointer_state.screen_point = PhysicalPoint::new(sample.x as i32, sample.y as i32);
                    pointer_state.local_point = pointer_state.screen_point;
                    pointer_state.velocity = CursorVelocity::new(vx, vy);
                    
                    tracing::trace!(
                        entity = ?entity,
                        x = sample.x,
                        y = sample.y,
                        "[transfer_buffers_to_world] PointerState updated"
                    );
                }
            }
            
            // バッファをクリア
            buffer.clear();
        }
    });
    
    // BUTTON_BUFFERSからPointerStateへボタン状態を転送
    // down_receivedがtrueの場合のみ、ボタンが押されたとしてtrue設定
    // up_receivedがtrueの場合のみ、ボタンが離されたとしてfalse設定
    // どちらでもない場合は既存の状態を維持（エッジ検出）
    BUTTON_BUFFERS.with(|buffers| {
        let buffers = buffers.borrow();
        
        for ((entity, button), buf) in buffers.iter() {
            if buf.down_received {
                // ボタンが押された瞬間
                if let Some(mut pointer_state) = world.get_mut::<PointerState>(*entity) {
                    match button {
                        PointerButton::Left => pointer_state.left_down = true,
                        PointerButton::Right => pointer_state.right_down = true,
                        PointerButton::Middle => pointer_state.middle_down = true,
                        PointerButton::XButton1 => pointer_state.xbutton1_down = true,
                        PointerButton::XButton2 => pointer_state.xbutton2_down = true,
                    }
                    
                    tracing::trace!(
                        entity = ?entity,
                        button = ?button,
                        "[transfer_buffers_to_world] Button pressed"
                    );
                }
            } else if buf.up_received {
                // ボタンが離された瞬間
                if let Some(mut pointer_state) = world.get_mut::<PointerState>(*entity) {
                    match button {
                        PointerButton::Left => pointer_state.left_down = false,
                        PointerButton::Right => pointer_state.right_down = false,
                        PointerButton::Middle => pointer_state.middle_down = false,
                        PointerButton::XButton1 => pointer_state.xbutton1_down = false,
                        PointerButton::XButton2 => pointer_state.xbutton2_down = false,
                    }
                    
                    tracing::trace!(
                        entity = ?entity,
                        button = ?button,
                        "[transfer_buffers_to_world] Button released"
                    );
                }
            }
        }
    });
    
    // BUTTON_BUFFERSをリセット（転送完了後）
    BUTTON_BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        for buf in buffers.values_mut() {
            buf.reset();
        }
    });
    
    // MODIFIER_STATEからPointerStateへ修飾キー状態を転送
    MODIFIER_STATE.with(|state| {
        let state = state.borrow();
        
        for (entity, (shift_down, ctrl_down)) in state.iter() {
            if let Some(mut pointer_state) = world.get_mut::<PointerState>(*entity) {
                pointer_state.shift_down = *shift_down;
                pointer_state.ctrl_down = *ctrl_down;
            }
        }
    });
}
