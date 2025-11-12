use bevy_ecs::prelude::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows_numerics::*;

use crate::api::*;

/// Windowコンポーネント - ウィンドウ作成に必要な基本パラメータを保持
/// スタイルや位置・サイズは WindowStyle, WindowPos コンポーネントで指定
#[derive(Component, Debug, Clone)]
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
    pub initial_dpi: f32,
}

unsafe impl Send for WindowHandle {}
unsafe impl Sync for WindowHandle {}

/// WindowHandleコンポーネントが追加された直後に呼ばれるフック
fn on_window_handle_add(mut world: bevy_ecs::world::DeferredWorld, hook: bevy_ecs::lifecycle::HookContext) {
    let entity = hook.entity;
    if let Some(handle) = world.get::<WindowHandle>(entity) {
        println!("[Hook] WindowHandle added to entity {:?}, hwnd {:?}, dpi {}", 
            entity, handle.hwnd, handle.initial_dpi);
        
        // アプリに通知
        if let Some(mut app) = world.get_resource_mut::<crate::ecs::app::App>() {
            app.on_window_created(entity);
        }
    }
}

/// 独自メッセージ: 最後のウィンドウが破棄されたことを通知
pub const WM_LAST_WINDOW_DESTROYED: u32 = WM_USER + 100;

/// WindowHandleコンポーネントが削除される直前に呼ばれるフック
fn on_window_handle_remove(mut world: bevy_ecs::world::DeferredWorld, hook: bevy_ecs::lifecycle::HookContext) {
    let entity = hook.entity;
    // このタイミングではまだWindowHandleにアクセスできる
    if let Some(handle) = world.get::<WindowHandle>(entity) {
        let hwnd = handle.hwnd;
        println!("[Hook] Entity {:?} being removed, sending WM_CLOSE to hwnd {:?}", entity, hwnd);
        
        // まず、アプリに通知（ウィンドウカウント更新のため）
        if let Some(mut app) = world.get_resource_mut::<crate::ecs::app::App>() {
            let should_quit = app.on_window_destroyed_no_quit(entity);
            
            // 最後のウィンドウの場合、メッセージウィンドウに通知を投げる
            // メッセージウィンドウはEcsWorldに保持されているので、
            // window_proc.rsのECS_WORLDから取得する必要がある
            if should_quit {
                use crate::ecs::window_proc::try_post_last_window_destroyed;
                try_post_last_window_destroyed();
            }
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
            ex_style: WINDOW_EX_STYLE(0),
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
}

impl Default for WindowPos {
    fn default() -> Self {
        Self {
            zorder: ZOrder::NoChange,
            position: Some(POINT { x: CW_USEDEFAULT, y: CW_USEDEFAULT }),
            size: Some(SIZE { cx: CW_USEDEFAULT, cy: CW_USEDEFAULT }),
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

        // ZOrder から hwnd_insert_after を決定
        let hwnd_insert_after = match self.zorder {
            ZOrder::NoChange => None,
            ZOrder::TopMost => Some(HWND_TOPMOST),
            ZOrder::NoTopMost => Some(HWND_NOTOPMOST),
            ZOrder::Top => Some(HWND_TOP),
            ZOrder::Bottom => Some(HWND_BOTTOM),
            ZOrder::InsertAfter(hwnd) => Some(hwnd),
        };

        unsafe { SetWindowPos(hwnd, hwnd_insert_after, x, y, width, height, flags) }
    }
}
