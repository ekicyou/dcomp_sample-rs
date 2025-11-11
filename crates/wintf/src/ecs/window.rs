use bevy_ecs::prelude::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows_numerics::*;

use crate::api::*;
pub use crate::dpi::Dpi;
use crate::{RawPoint, RawSize};

/// Windowコンポーネント - ウィンドウ作成に必要なパラメータを保持
#[derive(Component, Debug, Clone)]
pub struct Window {
    pub title: String,
    pub style: WINDOW_STYLE,
    pub ex_style: WINDOW_EX_STYLE,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub parent: Option<HWND>,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            title: "Window".to_string(),
            style: WS_OVERLAPPEDWINDOW | WS_VISIBLE,
            ex_style: WINDOW_EX_STYLE(0),
            x: CW_USEDEFAULT,
            y: CW_USEDEFAULT,
            width: CW_USEDEFAULT,
            height: CW_USEDEFAULT,
            parent: None,
        }
    }
}

unsafe impl Send for Window {}
unsafe impl Sync for Window {}

/// 作成済みウィンドウのハンドル情報（システムが自動的に設定）
#[derive(Component, Debug)]
#[component(storage = "SparseSet")]
pub struct WindowHandle {
    pub hwnd: HWND,
    pub instance: HINSTANCE,
    pub initial_dpi: Dpi,
}

unsafe impl Send for WindowHandle {}
unsafe impl Sync for WindowHandle {}

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
#[component(storage = "SparseSet")]
pub struct WindowStyle {
    pub style: WINDOW_STYLE,
    pub ex_style: WINDOW_EX_STYLE,
}

impl Default for WindowStyle {
    fn default() -> Self {
        Self {
            style: WS_OVERLAPPEDWINDOW,
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

#[derive(Component, Debug, Default, Clone, Copy, PartialEq)]
#[component(storage = "SparseSet")]
pub struct WindowPos {
    pub zorder: ZOrder,
    pub position: Option<RawPoint>,
    pub size: Option<RawSize>,

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
    pub fn with_position(mut self, position: RawPoint) -> Self {
        self.position = Some(position);
        self
    }

    /// サイズを設定
    pub fn with_size(mut self, size: RawSize) -> Self {
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
            (size.width, size.height)
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

//================================================================================
// ECS Window Message Handler
//================================================================================

use std::sync::OnceLock;
use std::cell::RefCell;
use std::rc::{Rc, Weak};

// SAFETY: EcsWorldはメインスレッドでのみアクセスされる
// wndprocもメインスレッドから呼ばれるため安全
struct SendWeak(Weak<RefCell<crate::ecs::world::EcsWorld>>);
unsafe impl Send for SendWeak {}
unsafe impl Sync for SendWeak {}

static ECS_WORLD: OnceLock<SendWeak> = OnceLock::new();

/// EcsWorldへの弱参照を登録（WinThreadMgr初期化時に呼ばれる）
pub fn set_ecs_world(world: Weak<RefCell<crate::ecs::world::EcsWorld>>) {
    let _ = ECS_WORLD.set(SendWeak(world));
}

/// EcsWorldへの参照を取得
fn try_get_ecs_world() -> Option<Rc<RefCell<crate::ecs::world::EcsWorld>>> {
    ECS_WORLD.get().and_then(|weak| weak.0.upgrade())
}

/// ECS専用のウィンドウプロシージャ
pub extern "system" fn ecs_wndproc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    use windows::Win32::Graphics::Gdi::ValidateRect;

    unsafe {
        match message {
            WM_NCCREATE => {
                let cs = lparam.0 as *const CREATESTRUCTW;
                if !cs.is_null() {
                    let entity_bits = (*cs).lpCreateParams as isize;
                    if entity_bits != 0 {
                        // Entity IDをGWLP_USERDATAに保存
                        SetWindowLongPtrW(hwnd, GWLP_USERDATA, entity_bits);
                    }
                }
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
            WM_NCHITTEST => DefWindowProcW(hwnd, message, wparam, lparam),
            WM_ERASEBKGND => {
                // 背景を消去しない（DirectCompositionで描画するため）
                LRESULT(1)
            }
            WM_PAINT => {
                // DirectCompositionで描画するため、ここでは領域を無効化解除するだけ
                let _ = ValidateRect(Some(hwnd), None);
                LRESULT(0)
            }
            WM_CLOSE => {
                // DestroyWindowを呼ぶと、WM_DESTROYが送られる
                let _ = DestroyWindow(hwnd);
                LRESULT(0)
            }
            WM_DESTROY => {
                // クリーンアップ
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
                
                // Appリソースに通知
                if let Some(world) = try_get_ecs_world() {
                    let mut world = world.borrow_mut();
                    if let Some(mut app) = world.world_mut().get_resource_mut::<crate::ecs::app::App>() {
                        app.on_window_destroyed();
                    }
                }
                
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, message, wparam, lparam),
        }
    }
}

/// hwndからEntity IDを取得するヘルパー関数
pub fn get_entity_from_hwnd(hwnd: HWND) -> Option<Entity> {
    unsafe {
        let entity_bits = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
        if entity_bits != 0 {
            Some(Entity::from_bits(entity_bits as u64))
        } else {
            None
        }
    }
}

//================================================================================
// Systems
//================================================================================

/// 未作成のWindowを検出して作成するシステム
pub fn create_windows(
    mut commands: Commands,
    mut app: ResMut<crate::ecs::app::App>,
    query: Query<(Entity, &Window), Without<WindowHandle>>,
) {
    use crate::process_singleton::WinProcessSingleton;
    use windows::core::HSTRING;

    let singleton = WinProcessSingleton::get_or_init();

    for (entity, window) in query.iter() {
        // Win32ウィンドウを作成
        let title = HSTRING::from(&window.title);

        // EntityのIDをlpCreateParamsとして渡す
        let entity_bits = entity.to_bits() as *mut std::ffi::c_void;

        let result = unsafe {
            CreateWindowExW(
                window.ex_style,
                singleton.ecs_window_class_name(), // ECS用のウィンドウクラスを使用
                &title,
                window.style,
                window.x,
                window.y,
                window.width,
                window.height,
                window.parent,
                None,
                Some(singleton.instance()),
                Some(entity_bits), // EntityのIDを渡す
            )
        };

        match result {
            Ok(hwnd) => {
                // 初期DPIを取得
                use windows::Win32::Graphics::Gdi::*;
                use windows::Win32::UI::HiDpi::*;

                let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
                let mut x_dpi = 0u32;
                let mut y_dpi = 0u32;
                let dpi_result =
                    unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut x_dpi, &mut y_dpi) };

                let initial_dpi = if dpi_result.is_ok() {
                    Dpi::new(x_dpi as f32)
                } else {
                    Dpi::new(96.0) // デフォルト
                };

                // WindowHandleコンポーネントを追加
                commands.entity(entity).insert(WindowHandle {
                    hwnd,
                    instance: singleton.instance(),
                    initial_dpi,
                });

                // ウィンドウを表示
                unsafe {
                    let _ = ShowWindow(hwnd, SW_SHOW);
                }

                // Appリソースに通知
                app.on_window_created();

                eprintln!("Window created: hwnd={:?}, entity={:?}", hwnd, entity);
            }
            Err(e) => {
                eprintln!("Failed to create window for entity {:?}: {:?}", entity, e);
            }
        }
    }
}
