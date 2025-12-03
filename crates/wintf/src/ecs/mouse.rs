//! マウス入力モジュール
//!
//! Win32マウスメッセージをECSコンポーネントとして正規化し、
//! ウィンドウレベルのマウス状態管理を提供する。

use bevy_ecs::prelude::*;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::time::Instant;

// ============================================================================
// 基本型定義 (Task 1.1)
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

/// マウスボタン種別
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
    XButton1,
    XButton2,
}

// ============================================================================
// MouseState コンポーネント (Task 1.1)
// ============================================================================

/// マウス状態コンポーネント
///
/// hit_test がヒットしたエンティティに付与される。
/// コンポーネントの存在 = ホバー中。
/// Added<MouseState> で Enter を検出。
///
/// Win32マウスメッセージの情報を透過的にECSに転送する。
/// 情報の解釈（Click判定等）はアプリ側の責務。
///
/// メモリ戦略: SparseSet - 頻繁な挿入/削除
#[derive(Component, Debug, Clone)]
#[component(storage = "SparseSet")]
pub struct MouseState {
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

impl Default for MouseState {
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

// ============================================================================
// MouseLeave マーカー (Task 2.1)
// ============================================================================

/// マウス離脱マーカー（1フレーム限り）
///
/// MouseState が削除されたフレームに付与される。
/// FrameFinalize で削除されるため、1フレームのみ存在。
///
/// メモリ戦略: SparseSet - 一時的マーカー
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
#[component(storage = "SparseSet")]
pub struct MouseLeave;

// ============================================================================
// WindowMouseTracking コンポーネント (Task 2.1)
// ============================================================================

/// TrackMouseEvent 状態追跡
///
/// ウィンドウエンティティに自動付与される。
/// `true` = TrackMouseEvent(TME_LEAVE) が有効
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct WindowMouseTracking(pub bool);

// ============================================================================
// MouseBuffer (Task 1.2, 5A)
// ============================================================================

/// 位置サンプル
#[derive(Debug, Clone, Copy)]
pub struct PositionSample {
    pub x: f32,
    pub y: f32,
    pub timestamp: Instant,
}

/// マウスバッファ（thread_local! で管理）
///
/// WndProc内で複数のWM_MOUSEMOVEが発生する可能性があるため、
/// バッファに蓄積してInputスケジュールで処理する。
#[derive(Debug, Default)]
pub struct MouseBuffer {
    samples: VecDeque<PositionSample>,
}

impl MouseBuffer {
    /// 最大サンプル数（速度計算用）
    const MAX_SAMPLES: usize = 5;

    /// 新しいMouseBufferを作成
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
        self.samples.clear();
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

// ============================================================================
// ButtonBuffer (Task 1.2, 5A)
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
    /// Entity ごとの MouseBuffer
    pub(crate) static MOUSE_BUFFERS: RefCell<HashMap<Entity, MouseBuffer>> = RefCell::new(HashMap::new());

    /// Entity × Button ごとの ButtonBuffer
    pub(crate) static BUTTON_BUFFERS: RefCell<HashMap<(Entity, MouseButton), ButtonBuffer>> = RefCell::new(HashMap::new());

    /// Entity ごとの WheelBuffer
    pub(crate) static WHEEL_BUFFERS: RefCell<HashMap<Entity, WheelBuffer>> = RefCell::new(HashMap::new());

    /// Entity ごとの DoubleClick（tick 内で最初に検出されたもの）
    pub(crate) static DOUBLE_CLICK_BUFFERS: RefCell<HashMap<Entity, DoubleClick>> = RefCell::new(HashMap::new());

    /// Entity ごとの修飾キー状態（最新値）
    pub(crate) static MODIFIER_STATE: RefCell<HashMap<Entity, (bool, bool)>> = RefCell::new(HashMap::new());
}

// ============================================================================
// hit_test 仮スタブ (Task 1.3)
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
// システム (Task 4.2, 2.2)
// ============================================================================

/// マウスバッファ処理システム
///
/// Inputスケジュールで実行され、バッファ内容をMouseStateコンポーネントに反映する。
pub fn process_mouse_buffers(mut query: Query<(Entity, &mut MouseState)>) {
    for (entity, mut mouse) in query.iter_mut() {
        // MouseBuffer から位置と速度を取得
        MOUSE_BUFFERS.with(|buffers| {
            let mut buffers = buffers.borrow_mut();
            if let Some(buffer) = buffers.get_mut(&entity) {
                // 速度計算
                let (vx, vy) = buffer.calculate_velocity();
                mouse.velocity = CursorVelocity::new(vx, vy);

                // 最新位置取得
                if let Some(sample) = buffer.latest() {
                    mouse.screen_point = PhysicalPoint::new(sample.x as i32, sample.y as i32);
                    // Note: local_point は hit_test 結果から設定（Phase 1ではscreen_pointと同じ）
                    mouse.local_point = mouse.screen_point;
                }

                // バッファクリア
                buffer.clear();
            }
        });

        // ButtonBuffer から各ボタン状態を取得
        BUTTON_BUFFERS.with(|buffers| {
            let mut buffers = buffers.borrow_mut();

            // 各ボタンの処理（DOWN優先ルール）
            for button in [
                MouseButton::Left,
                MouseButton::Right,
                MouseButton::Middle,
                MouseButton::XButton1,
                MouseButton::XButton2,
            ] {
                if let Some(buf) = buffers.get_mut(&(entity, button)) {
                    let is_down = if buf.down_received {
                        true
                    } else if buf.up_received {
                        false
                    } else {
                        // イベントなし - 現在の状態を維持
                        match button {
                            MouseButton::Left => mouse.left_down,
                            MouseButton::Right => mouse.right_down,
                            MouseButton::Middle => mouse.middle_down,
                            MouseButton::XButton1 => mouse.xbutton1_down,
                            MouseButton::XButton2 => mouse.xbutton2_down,
                        }
                    };

                    match button {
                        MouseButton::Left => mouse.left_down = is_down,
                        MouseButton::Right => mouse.right_down = is_down,
                        MouseButton::Middle => mouse.middle_down = is_down,
                        MouseButton::XButton1 => mouse.xbutton1_down = is_down,
                        MouseButton::XButton2 => mouse.xbutton2_down = is_down,
                    }

                    buf.reset();
                }
            }
        });

        // WheelBuffer からホイール情報を取得
        WHEEL_BUFFERS.with(|buffers| {
            let mut buffers = buffers.borrow_mut();
            if let Some(buf) = buffers.get_mut(&entity) {
                mouse.wheel = WheelDelta {
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
                mouse.double_click = dc;
            }
        });

        // 修飾キー状態を取得
        MODIFIER_STATE.with(|state| {
            let state = state.borrow();
            if let Some(&(shift, ctrl)) = state.get(&entity) {
                mouse.shift_down = shift;
                mouse.ctrl_down = ctrl;
            }
        });

        mouse.timestamp = Instant::now();
    }
}

/// 一時的マウス状態クリアシステム（FrameFinalize）
///
/// CommitComposition 後に実行され、1フレームのみ有効な状態をリセットする。
pub fn clear_transient_mouse_state(
    mut query: Query<&mut MouseState>,
    mut commands: Commands,
    leave_query: Query<Entity, With<MouseLeave>>,
) {
    // double_click, wheel をリセット（1フレームのみ有効）
    for mut mouse in query.iter_mut() {
        mouse.double_click = DoubleClick::None;
        mouse.wheel = WheelDelta::default();
    }

    // MouseLeave マーカー除去
    for entity in leave_query.iter() {
        commands.entity(entity).remove::<MouseLeave>();
    }
}

// ============================================================================
// デバッグ用監視システム
// ============================================================================

/// MouseState の Added/Changed を監視するデバッグシステム
///
/// Inputスケジュールで実行し、MouseStateの変化をログ出力する。
/// デバッグ用途のため、リリースビルドでは使用しないこと。
pub fn debug_mouse_state_changes(
    added_query: Query<(Entity, &MouseState), Added<MouseState>>,
    changed_query: Query<(Entity, &MouseState), Changed<MouseState>>,
) {
    use tracing::info;

    // 新規追加（Enter）
    for (entity, mouse) in added_query.iter() {
        info!(
            entity = ?entity,
            screen_x = mouse.screen_point.x,
            screen_y = mouse.screen_point.y,
            left = mouse.left_down,
            right = mouse.right_down,
            middle = mouse.middle_down,
            shift = mouse.shift_down,
            ctrl = mouse.ctrl_down,
            "[MouseState Added] Enter detected"
        );
    }

    // 変更（移動・ボタン・ホイール等）
    for (entity, mouse) in changed_query.iter() {
        // Added も Changed に含まれるのでスキップ
        // Note: bevy_ecs では Added は Changed のサブセット
        // ここでは移動・ボタン変化のみログ出力したい場合、
        // 別途フラグ管理が必要だが、デバッグ用なので許容

        // ダブルクリック検出時のみログ
        if mouse.double_click != DoubleClick::None {
            info!(
                entity = ?entity,
                double_click = ?mouse.double_click,
                "[MouseState Changed] DoubleClick detected"
            );
        }

        // ホイール回転時のみログ
        if mouse.wheel.vertical != 0 || mouse.wheel.horizontal != 0 {
            info!(
                entity = ?entity,
                vertical = mouse.wheel.vertical,
                horizontal = mouse.wheel.horizontal,
                "[MouseState Changed] Wheel detected"
            );
        }
    }
}

/// MouseLeave マーカーを監視するデバッグシステム
///
/// Inputスケジュールで実行し、MouseLeaveの付与をログ出力する。
pub fn debug_mouse_leave(leave_query: Query<Entity, Added<MouseLeave>>) {
    use tracing::info;

    for entity in leave_query.iter() {
        info!(
            entity = ?entity,
            "[MouseLeave Added] Leave detected"
        );
    }
}

// ============================================================================
// バッファ操作ヘルパー（handlers.rs から使用）
// ============================================================================

/// MouseBufferにサンプルを追加
#[inline]
pub(crate) fn push_mouse_sample(entity: Entity, x: f32, y: f32, timestamp: Instant) {
    MOUSE_BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        let buffer = buffers.entry(entity).or_insert_with(MouseBuffer::new);
        buffer.push(PositionSample { x, y, timestamp });
    });
}

/// ButtonBufferにボタン押下を記録
#[inline]
pub(crate) fn record_button_down(entity: Entity, button: MouseButton) {
    BUTTON_BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        let buffer = buffers.entry((entity, button)).or_default();
        buffer.record_down();
    });
}

/// ButtonBufferにボタン解放を記録
#[inline]
pub(crate) fn record_button_up(entity: Entity, button: MouseButton) {
    BUTTON_BUFFERS.with(|buffers| {
        let mut buffers = buffers.borrow_mut();
        let buffer = buffers.entry((entity, button)).or_default();
        buffer.record_up();
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
    fn test_mouse_buffer_push() {
        let mut buffer = MouseBuffer::new();
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
    fn test_mouse_buffer_max_samples() {
        let mut buffer = MouseBuffer::new();
        for i in 0..10 {
            buffer.push(PositionSample {
                x: i as f32,
                y: i as f32,
                timestamp: Instant::now(),
            });
        }
        // MAX_SAMPLES (5) を超えないことを確認
        assert_eq!(buffer.len(), MouseBuffer::MAX_SAMPLES);

        // 最新の値が最後に追加されたものであることを確認
        let latest = buffer.latest().unwrap();
        assert_eq!(latest.x, 9.0);
    }

    #[test]
    fn test_velocity_calculation() {
        let mut buffer = MouseBuffer::new();
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
    fn test_mouse_state_default() {
        let state = MouseState::default();
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
    fn test_mouse_leave_marker() {
        // MouseLeaveはunitスタイルのマーカーコンポーネント
        let leave1 = MouseLeave;
        let leave2 = MouseLeave;
        assert_eq!(leave1, leave2);
    }

    #[test]
    fn test_window_mouse_tracking_default() {
        let tracking = WindowMouseTracking::default();
        assert!(!tracking.0);

        let tracking_enabled = WindowMouseTracking(true);
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
    fn test_mouse_button_enum() {
        // MouseButtonのHashトレイト実装を確認
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(MouseButton::Left);
        set.insert(MouseButton::Right);
        set.insert(MouseButton::Middle);
        set.insert(MouseButton::XButton1);
        set.insert(MouseButton::XButton2);
        assert_eq!(set.len(), 5);
    }
}
