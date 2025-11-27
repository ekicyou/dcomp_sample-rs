use bevy_ecs::hierarchy::ChildOf;
use bevy_ecs::lifecycle::HookContext;
use bevy_ecs::prelude::*;
use bevy_ecs::world::DeferredWorld;
use windows::Win32::Foundation::*;
use windows::Win32::UI::HiDpi::{AdjustWindowRectExForDpi, GetDpiForWindow};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows_numerics::*;

use crate::api::*;
use crate::ecs::layout::LayoutRoot;
use crate::ecs::Visual;

/// Windowコンポーネント - ウィンドウ作成に必要な基本パラメータを保持
/// スタイルや位置・サイズは WindowStyle, WindowPos コンポーネントで指定
#[derive(Component, Debug, Clone)]
#[component(on_add = on_window_add)]
pub struct Window {
    pub title: String,
    pub parent: Option<HWND>,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            title: "Window".to_string(),
            parent: None,
        }
    }
}

unsafe impl Send for Window {}
unsafe impl Sync for Window {}

/// 作成済みウィンドウのハンドル情報（システムが自動的に設定）
#[derive(Component, Debug, Copy, Clone)]
#[component(storage = "SparseSet", on_add = on_window_handle_add, on_remove = on_window_handle_remove)]
pub struct WindowHandle {
    pub hwnd: HWND,
    pub instance: HINSTANCE,
}

unsafe impl Send for WindowHandle {}
unsafe impl Sync for WindowHandle {}

impl WindowHandle {
    /// ウィンドウのDPI値を取得
    pub fn get_dpi(&self) -> u32 {
        unsafe { GetDpiForWindow(self.hwnd) }
    }

    /// ウィンドウスタイルと拡張スタイルを取得
    pub fn get_style(&self) -> Result<(WINDOW_STYLE, WINDOW_EX_STYLE), String> {
        let style = match get_window_long_ptr(self.hwnd, GWL_STYLE) {
            Ok(v) => WINDOW_STYLE(v as u32),
            Err(e) => {
                return Err(format!(
                    "GetWindowLongPtrW(GWL_STYLE) failed for HWND {:?}: {:?}",
                    self.hwnd, e
                ));
            }
        };

        let ex_style = match get_window_long_ptr(self.hwnd, GWL_EXSTYLE) {
            Ok(v) => WINDOW_EX_STYLE(v as u32),
            Err(e) => {
                return Err(format!(
                    "GetWindowLongPtrW(GWL_EXSTYLE) failed for HWND {:?}: {:?}",
                    self.hwnd, e
                ));
            }
        };

        Ok((style, ex_style))
    }

    /// クライアント領域RECTをウィンドウ全体RECTに変換
    ///
    /// # Arguments
    /// * `client_rect` - クライアント領域の矩形
    ///
    /// # Returns
    /// * `Ok(RECT)` - ウィンドウ全体の矩形
    /// * `Err(String)` - 変換失敗時のエラーメッセージ
    pub fn client_to_window_rect(&self, client_rect: RECT) -> Result<RECT, String> {
        let (style, ex_style) = self.get_style()?;
        let dpi = self.get_dpi();
        if dpi == 0 {
            return Err(format!(
                "GetDpiForWindow returned 0 for HWND {:?}",
                self.hwnd
            ));
        }

        let mut rect = client_rect;
        let result = unsafe { AdjustWindowRectExForDpi(&mut rect, style, false, ex_style, dpi) };

        if result.is_err() {
            return Err(format!(
                "AdjustWindowRectExForDpi failed for HWND {:?}: {:?}",
                self.hwnd, result
            ));
        }

        Ok(rect)
    }

    /// ウィンドウ全体RECTをクライアント領域RECTに変換（逆変換）
    ///
    /// AdjustWindowRectExForDpiの逆変換を行う。
    /// 原点(0,0)での差分を計算し、その差分を使って変換する。
    ///
    /// # Arguments
    /// * `window_rect` - ウィンドウ全体の矩形
    ///
    /// # Returns
    /// * `Ok(RECT)` - クライアント領域の矩形
    /// * `Err(String)` - 変換失敗時のエラーメッセージ
    pub fn window_to_client_rect(&self, window_rect: RECT) -> Result<RECT, String> {
        // 原点でクライアント→ウィンドウ変換を行い、差分を計算
        let origin_client = RECT {
            left: 0,
            top: 0,
            right: 100, // サイズは差分計算には影響しない
            bottom: 100,
        };
        let origin_window = self.client_to_window_rect(origin_client)?;

        // 差分: ウィンドウ座標 - クライアント座標
        let left_diff = origin_window.left - origin_client.left;
        let top_diff = origin_window.top - origin_client.top;
        let right_diff = origin_window.right - origin_client.right;
        let bottom_diff = origin_window.bottom - origin_client.bottom;

        // 逆変換: クライアント座標 = ウィンドウ座標 - 差分
        Ok(RECT {
            left: window_rect.left - left_diff,
            top: window_rect.top - top_diff,
            right: window_rect.right - right_diff,
            bottom: window_rect.bottom - bottom_diff,
        })
    }

    /// クライアント領域の座標・サイズをウィンドウ全体の座標・サイズに変換
    ///
    /// # Arguments
    /// * `position` - クライアント領域の左上座標
    /// * `size` - クライアント領域のサイズ
    ///
    /// # Returns
    /// * `Ok((x, y, width, height))` - ウィンドウ全体座標
    /// * `Err(String)` - 変換失敗時のエラーメッセージ
    pub fn client_to_window_coords(
        &self,
        position: POINT,
        size: SIZE,
    ) -> Result<(i32, i32, i32, i32), String> {
        let client_rect = RECT {
            left: position.x,
            top: position.y,
            right: position.x + size.cx,
            bottom: position.y + size.cy,
        };
        let window_rect = self.client_to_window_rect(client_rect)?;
        Ok((
            window_rect.left,
            window_rect.top,
            window_rect.right - window_rect.left,
            window_rect.bottom - window_rect.top,
        ))
    }

    /// ウィンドウ全体の座標・サイズをクライアント領域の座標・サイズに変換
    ///
    /// # Arguments
    /// * `x` - ウィンドウ左上X座標
    /// * `y` - ウィンドウ左上Y座標
    /// * `width` - ウィンドウ幅
    /// * `height` - ウィンドウ高さ
    ///
    /// # Returns
    /// * `Ok((POINT, SIZE))` - クライアント領域の座標とサイズ
    /// * `Err(String)` - 変換失敗時のエラーメッセージ
    pub fn window_to_client_coords(
        &self,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) -> Result<(POINT, SIZE), String> {
        let window_rect = RECT {
            left: x,
            top: y,
            right: x + width,
            bottom: y + height,
        };
        let client_rect = self.window_to_client_rect(window_rect)?;
        Ok((
            POINT {
                x: client_rect.left,
                y: client_rect.top,
            },
            SIZE {
                cx: client_rect.right - client_rect.left,
                cy: client_rect.bottom - client_rect.top,
            },
        ))
    }
}

/// WindowHandleコンポーネントが追加された直後に呼ばれるフック
fn on_window_handle_add(
    mut world: bevy_ecs::world::DeferredWorld,
    hook: bevy_ecs::lifecycle::HookContext,
) {
    let entity = hook.entity;
    if let Some(handle) = world.get::<WindowHandle>(entity) {
        println!(
            "[Hook] WindowHandle added to entity {:?}, hwnd {:?}",
            entity, handle.hwnd
        );

        // アプリに通知
        if let Some(mut app) = world.get_resource_mut::<crate::ecs::app::App>() {
            app.on_window_created(entity);
        }
    }
}

/// WindowHandleコンポーネントが削除される直前に呼ばれるフック
fn on_window_handle_remove(
    mut world: bevy_ecs::world::DeferredWorld,
    hook: bevy_ecs::lifecycle::HookContext,
) {
    let entity = hook.entity;
    // このタイミングではまだWindowHandleにアクセスできる
    if let Some(handle) = world.get::<WindowHandle>(entity) {
        let hwnd = handle.hwnd;
        println!(
            "[Hook] Entity {:?} being removed, sending WM_CLOSE to hwnd {:?}",
            entity, hwnd
        );

        // アプリに通知（ウィンドウカウント更新 & 必要ならメッセージ送信）
        if let Some(mut app) = world.get_resource_mut::<crate::ecs::app::App>() {
            app.on_window_destroyed(entity);
        }

        // ウィンドウクローズを非同期で要求
        unsafe {
            let _ = PostMessageW(Some(hwnd), WM_CLOSE, WPARAM(0), LPARAM(0));
        }
    }
}

/// DPI変換行列を保持するコンポーネント
#[derive(Component, Debug, Clone, Copy, PartialEq)]
#[component(storage = "SparseSet")]
pub struct DpiTransform {
    pub transform: Matrix3x2,
    pub global_transform: Matrix3x2,
}

impl DpiTransform {
    /// WM_DPICHANGED イベントのパラメーターから作成
    #[allow(non_snake_case)]
    pub fn from_WM_DPICHANGED(wparam: WPARAM, _lparam: LPARAM) -> Self {
        let (x_dpi, y_dpi) = (wparam.0 as u16, (wparam.0 >> 16) as u16);
        Self::from_dpi(x_dpi, y_dpi)
    }

    pub fn from_dpi(x_dpi: u16, y_dpi: u16) -> Self {
        let scale_x = x_dpi as f32 / 96.0;
        let scale_y = y_dpi as f32 / 96.0;
        let transform = Matrix3x2::scale(scale_x, scale_y);
        let global_transform = transform;
        Self {
            transform,
            global_transform,
        }
    }

    pub fn push(self, transform: Matrix3x2) -> Self {
        let global_transform = self.global_transform * transform;
        Self {
            transform,
            global_transform,
        }
    }
}

/// Window Style / Ex Style
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct WindowStyle {
    pub style: WINDOW_STYLE,
    pub ex_style: WINDOW_EX_STYLE,
}

impl Default for WindowStyle {
    fn default() -> Self {
        Self {
            style: WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            ex_style: WS_EX_NOREDIRECTIONBITMAP,
        }
    }
}

impl WindowStyle {
    /// 新しい WindowStyle を作成
    pub fn from_hwnd(hwnd: HWND) -> windows::core::Result<Self> {
        let style = WINDOW_STYLE(get_window_long_ptr(hwnd, GWL_STYLE)? as u32);
        let ex_style = WINDOW_EX_STYLE(get_window_long_ptr(hwnd, GWL_EXSTYLE)? as u32);
        Ok(Self { style, ex_style })
    }
}

/// Z-order の設定方法を表す列挙型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZOrder {
    /// Z-orderを変更しない
    NoChange,
    /// 最前面（常に最前面）
    TopMost,
    /// 最前面ではない（通常のウィンドウ）
    NoTopMost,
    /// 最前面に配置
    Top,
    /// 最背面に配置
    Bottom,
    /// 指定したウィンドウの後ろに配置
    InsertAfter(HWND),
}

impl Default for ZOrder {
    fn default() -> Self {
        ZOrder::NoChange
    }
}

unsafe impl Send for ZOrder {}
unsafe impl Sync for ZOrder {}

/// ウィンドウの位置・サイズ・表示オプション
/// ウィンドウ作成時の初期位置・サイズにも使用される
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct WindowPos {
    pub zorder: ZOrder,
    pub position: Option<POINT>,
    pub size: Option<SIZE>,

    pub no_redraw: bool,        // SWP_NOREDRAW: 再描画しない
    pub no_activate: bool,      // SWP_NOACTIVATE: ウィンドウをアクティブにしない
    pub frame_changed: bool,    // SWP_FRAMECHANGED: フレーム変更を通知
    pub show_window: bool,      // SWP_SHOWWINDOW: ウィンドウを表示
    pub hide_window: bool,      // SWP_HIDEWINDOW: ウィンドウを非表示
    pub no_copy_bits: bool,     // SWP_NOCOPYBITS: クライアント領域をコピーしない
    pub no_owner_zorder: bool,  // SWP_NOOWNERZORDER: オーナーウィンドウのZ-orderを変更しない
    pub no_send_changing: bool, // SWP_NOSENDCHANGING: WM_WINDOWPOSCHANGINGを送信しない
    pub defer_erase: bool,      // SWP_DEFERERASE: WM_SYNCPAINTを送信しない
    pub async_window_pos: bool, // SWP_ASYNCWINDOWPOS: 非同期で処理

    // エコーバック検知用フィールド
    pub last_sent_position: Option<(i32, i32)>,
    pub last_sent_size: Option<(i32, i32)>,
}

impl Default for WindowPos {
    fn default() -> Self {
        Self {
            zorder: ZOrder::NoChange,
            position: Some(POINT {
                x: CW_USEDEFAULT,
                y: CW_USEDEFAULT,
            }),
            size: Some(SIZE {
                cx: CW_USEDEFAULT,
                cy: CW_USEDEFAULT,
            }),
            no_redraw: false,
            no_activate: false,
            frame_changed: false,
            show_window: false,
            hide_window: false,
            no_copy_bits: false,
            no_owner_zorder: false,
            no_send_changing: false,
            defer_erase: false,
            async_window_pos: false,
            last_sent_position: None,
            last_sent_size: None,
        }
    }
}

unsafe impl Send for WindowPos {}
unsafe impl Sync for WindowPos {}

impl WindowPos {
    /// 新しい WindowPos を作成
    pub fn new() -> Self {
        Self::default()
    }

    /// Z-order を設定
    pub fn with_zorder(mut self, zorder: ZOrder) -> Self {
        self.zorder = zorder;
        self
    }

    /// Z-orderを変更しない（デフォルト）
    pub fn zorder_no_change(mut self) -> Self {
        self.zorder = ZOrder::NoChange;
        self
    }

    /// 最前面（常に最前面）に配置
    pub fn zorder_topmost(mut self) -> Self {
        self.zorder = ZOrder::TopMost;
        self
    }

    /// 最前面ではない（通常のウィンドウ）に変更
    pub fn zorder_notopmost(mut self) -> Self {
        self.zorder = ZOrder::NoTopMost;
        self
    }

    /// 最前面に配置
    pub fn zorder_top(mut self) -> Self {
        self.zorder = ZOrder::Top;
        self
    }

    /// 最背面に配置
    pub fn zorder_bottom(mut self) -> Self {
        self.zorder = ZOrder::Bottom;
        self
    }

    /// 指定したウィンドウの後ろに配置
    pub fn zorder_insert_after(mut self, hwnd: HWND) -> Self {
        self.zorder = ZOrder::InsertAfter(hwnd);
        self
    }

    /// 位置を設定
    pub fn with_position(mut self, position: POINT) -> Self {
        self.position = Some(position);
        self
    }

    /// サイズを設定
    pub fn with_size(mut self, size: SIZE) -> Self {
        self.size = Some(size);
        self
    }

    /// SWP_NOREDRAW フラグを設定
    pub fn no_redraw(mut self, value: bool) -> Self {
        self.no_redraw = value;
        self
    }

    /// SWP_NOACTIVATE フラグを設定
    pub fn no_activate(mut self, value: bool) -> Self {
        self.no_activate = value;
        self
    }

    /// SWP_FRAMECHANGED フラグを設定
    pub fn frame_changed(mut self, value: bool) -> Self {
        self.frame_changed = value;
        self
    }

    /// SWP_SHOWWINDOW フラグを設定
    pub fn show_window(mut self, value: bool) -> Self {
        self.show_window = value;
        self
    }

    /// SWP_HIDEWINDOW フラグを設定
    pub fn hide_window(mut self, value: bool) -> Self {
        self.hide_window = value;
        self
    }

    /// SWP_NOCOPYBITS フラグを設定
    pub fn no_copy_bits(mut self, value: bool) -> Self {
        self.no_copy_bits = value;
        self
    }

    /// SWP_NOOWNERZORDER フラグを設定
    pub fn no_owner_zorder(mut self, value: bool) -> Self {
        self.no_owner_zorder = value;
        self
    }

    /// SWP_NOSENDCHANGING フラグを設定
    pub fn no_send_changing(mut self, value: bool) -> Self {
        self.no_send_changing = value;
        self
    }

    /// SWP_DEFERERASE フラグを設定
    pub fn defer_erase(mut self, value: bool) -> Self {
        self.defer_erase = value;
        self
    }

    /// SWP_ASYNCWINDOWPOS フラグを設定
    pub fn async_window_pos(mut self, value: bool) -> Self {
        self.async_window_pos = value;
        self
    }

    /// bool値からフラグを生成
    fn build_flags(&self) -> SET_WINDOW_POS_FLAGS {
        let mut flags = SET_WINDOW_POS_FLAGS(0);

        // 自動判定: position が None なら SWP_NOMOVE
        if self.position.is_none() {
            flags |= SWP_NOMOVE;
        }

        // 自動判定: size が None なら SWP_NOSIZE
        if self.size.is_none() {
            flags |= SWP_NOSIZE;
        }

        // 自動判定: zorder が NoChange なら SWP_NOZORDER
        if self.zorder == ZOrder::NoChange {
            flags |= SWP_NOZORDER;
        }

        if self.no_redraw {
            flags |= SWP_NOREDRAW;
        }
        if self.no_activate {
            flags |= SWP_NOACTIVATE;
        }
        if self.frame_changed {
            flags |= SWP_FRAMECHANGED;
        }
        if self.show_window {
            flags |= SWP_SHOWWINDOW;
        }
        if self.hide_window {
            flags |= SWP_HIDEWINDOW;
        }
        if self.no_copy_bits {
            flags |= SWP_NOCOPYBITS;
        }
        if self.no_owner_zorder {
            flags |= SWP_NOOWNERZORDER;
        }
        if self.no_send_changing {
            flags |= SWP_NOSENDCHANGING;
        }
        if self.defer_erase {
            flags |= SWP_DEFERERASE;
        }
        if self.async_window_pos {
            flags |= SWP_ASYNCWINDOWPOS;
        }

        flags
    }

    /// システム用フラグビルダー（公開用）
    /// apply_window_pos_changesシステムから利用
    pub fn build_flags_for_system(&self) -> SET_WINDOW_POS_FLAGS {
        self.build_flags()
    }

    /// ZOrderに基づくhwnd_insert_afterを取得（公開用）
    /// apply_window_pos_changesシステムから利用
    pub fn get_hwnd_insert_after(&self) -> Option<HWND> {
        match self.zorder {
            ZOrder::NoChange => None,
            ZOrder::TopMost => Some(HWND_TOPMOST),
            ZOrder::NoTopMost => Some(HWND_NOTOPMOST),
            ZOrder::Top => Some(HWND_TOP),
            ZOrder::Bottom => Some(HWND_BOTTOM),
            ZOrder::InsertAfter(hwnd) => Some(hwnd),
        }
    }

    /// SetWindowPos を呼び出す
    pub fn set_window_pos(&self, hwnd: HWND) -> windows::core::Result<()> {
        let (x, y) = if let Some(pos) = self.position {
            (pos.x, pos.y)
        } else {
            (0, 0)
        };

        let (width, height) = if let Some(size) = self.size {
            (size.cx, size.cy)
        } else {
            (0, 0)
        };

        let flags = self.build_flags();
        let hwnd_insert_after = self.get_hwnd_insert_after();

        unsafe { SetWindowPos(hwnd, hwnd_insert_after, x, y, width, height, flags) }
    }

    /// エコーバック判定メソッド
    /// SetWindowPosで送信した値とWM_WINDOWPOSCHANGEDで受信した値が一致するかチェック
    pub fn is_echo(&self, position: POINT, size: SIZE) -> bool {
        self.last_sent_position == Some((position.x, position.y))
            && self.last_sent_size == Some((size.cx, size.cy))
    }

    /// クライアント領域の座標・サイズをウィンドウ全体の座標・サイズに変換する。
    ///
    /// # Arguments
    /// * `window_handle` - 変換対象のウィンドウハンドル
    ///
    /// # Returns
    /// * `Ok((x, y, width, height))` - 変換後のウィンドウ全体座標（左上x, 左上y, 幅, 高さ）
    /// * `Err(String)` - Win32 API呼び出し失敗時のエラーメッセージ
    ///
    /// # Notes
    /// - `WindowHandle::client_to_window_coords`に委譲
    /// - Windows 11専用実装（DPIフォールバック不要）
    pub fn to_window_coords(
        &self,
        window_handle: &WindowHandle,
    ) -> Result<(i32, i32, i32, i32), String> {
        // position/sizeがNoneの場合はデフォルト値(0, 0)を使用
        let position = self.position.unwrap_or(POINT { x: 0, y: 0 });
        let size = self.size.unwrap_or(SIZE { cx: 0, cy: 0 });

        window_handle.client_to_window_coords(position, size)
    }

    /// ウィンドウ作成時に使用する座標変換（HWNDなしでスタイル情報から変換）
    ///
    /// # Arguments
    /// * `style` - ウィンドウスタイル
    /// * `ex_style` - 拡張ウィンドウスタイル
    /// * `dpi` - DPI値（0の場合はデフォルト96を使用）
    ///
    /// # Returns
    /// * `(x, y, width, height)` - 変換後のウィンドウ全体座標
    ///
    /// # Notes
    /// - CW_USEDEFAULTが含まれる場合は変換せずそのまま返す
    pub fn to_window_coords_for_creation(
        &self,
        style: WINDOW_STYLE,
        ex_style: WINDOW_EX_STYLE,
        dpi: u32,
    ) -> (i32, i32, i32, i32) {
        let position = self.position.unwrap_or(POINT {
            x: CW_USEDEFAULT,
            y: CW_USEDEFAULT,
        });
        let size = self.size.unwrap_or(SIZE {
            cx: CW_USEDEFAULT,
            cy: CW_USEDEFAULT,
        });

        // CW_USEDEFAULTが含まれる場合は変換をスキップ
        if position.x == CW_USEDEFAULT || size.cx == CW_USEDEFAULT {
            return (position.x, position.y, size.cx, size.cy);
        }

        // DPIが0の場合はデフォルト値を使用
        let dpi = if dpi == 0 { 96 } else { dpi };

        // クライアント領域からRECT構造体を構築
        let mut rect = RECT {
            left: position.x,
            top: position.y,
            right: position.x + size.cx,
            bottom: position.y + size.cy,
        };

        // AdjustWindowRectExForDpiでウィンドウ全体の矩形を計算
        let result = unsafe { AdjustWindowRectExForDpi(&mut rect, style, false, ex_style, dpi) };

        if result.is_err() {
            // 変換失敗時は元の座標を返す
            eprintln!(
                "AdjustWindowRectExForDpi failed during window creation: {:?}. Using original values.",
                result
            );
            return (position.x, position.y, size.cx, size.cy);
        }

        (
            rect.left,
            rect.top,
            rect.right - rect.left,
            rect.bottom - rect.top,
        )
    }
}

/// Window追加時にLayoutRootの子として設定するためのコマンド
struct SetWindowParentToLayoutRoot {
    entity: Entity,
}

impl Command for SetWindowParentToLayoutRoot {
    fn apply(self, world: &mut World) {
        // LayoutRootを検索
        let mut query = world.query_filtered::<Entity, With<LayoutRoot>>();
        let layout_root = query.iter(world).next();

        if let Some(root) = layout_root {
            // Windowエンティティがまだ親を持っていない場合のみChildOfを設定
            if let Ok(mut entity_mut) = world.get_entity_mut(self.entity) {
                if !entity_mut.contains::<ChildOf>() {
                    eprintln!(
                        "[on_window_add] Setting ChildOf({:?}) for Window entity {:?}",
                        root, self.entity
                    );
                    entity_mut.insert(ChildOf(root));
                }
            }
        }
        // LayoutRootが見つからない場合は何もしない
        // 後のフレームで ensure_window_parent_system が処理する
    }
}

/// Windowコンポーネントが追加されたときに呼ばれるフック
/// WindowをLayoutRootの子として自動的に設定し、Visualコンポーネントを自動挿入する
fn on_window_add(mut world: DeferredWorld, context: HookContext) {
    let entity = context.entity;

    // LayoutRootの子として設定
    world
        .commands()
        .queue(SetWindowParentToLayoutRoot { entity });

    // Visual自動挿入（既に存在する場合はスキップ）
    if world.get::<Visual>(entity).is_none() {
        world.commands().entity(entity).insert(Visual::default());
    }
}
