//! ウィンドウプロシージャモジュール
//!
//! Windowsメッセージのディスパッチとハンドラ管理

mod handlers;

use bevy_ecs::prelude::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::Controls::WM_MOUSELEAVE;
use windows::Win32::UI::WindowsAndMessaging::*;

use std::cell::RefCell;
use std::rc::{Rc, Weak};
use std::sync::OnceLock;

// SAFETY: EcsWorldはメインスレッドでのみアクセスされる
// wndprocもメインスレッドから呼ばれるため安全
struct SendWeak(Weak<RefCell<crate::ecs::world::EcsWorld>>);
unsafe impl Send for SendWeak {}
unsafe impl Sync for SendWeak {}

static ECS_WORLD: OnceLock<SendWeak> = OnceLock::new();

/// EcsWorldへの弱参照を登録（WinThreadMgr初期化時に呼ばれる）
#[inline]
pub(crate) fn set_ecs_world(world: Weak<RefCell<crate::ecs::world::EcsWorld>>) {
    let _ = ECS_WORLD.set(SendWeak(world));
}

/// EcsWorldへの参照を取得（try_borrow_mut可能な状態で）
pub(super) fn try_get_ecs_world() -> Option<Rc<RefCell<crate::ecs::world::EcsWorld>>> {
    ECS_WORLD.get().and_then(|weak| weak.0.upgrade())
}

/// ECS専用のウィンドウプロシージャ
pub(crate) extern "system" fn ecs_wndproc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let result = unsafe {
        match message {
            WM_NCCREATE => handlers::WM_NCCREATE(hwnd, message, wparam, lparam),
            WM_NCDESTROY => handlers::WM_NCDESTROY(hwnd, message, wparam, lparam),
            WM_ERASEBKGND => handlers::WM_ERASEBKGND(hwnd, message, wparam, lparam),
            WM_PAINT => handlers::WM_PAINT(hwnd, message, wparam, lparam),
            WM_CLOSE => handlers::WM_CLOSE(hwnd, message, wparam, lparam),
            WM_WINDOWPOSCHANGED => handlers::WM_WINDOWPOSCHANGED(hwnd, message, wparam, lparam),
            WM_DISPLAYCHANGE => handlers::WM_DISPLAYCHANGE(hwnd, message, wparam, lparam),
            WM_DPICHANGED => handlers::WM_DPICHANGED(hwnd, message, wparam, lparam),
            // マウスメッセージ
            WM_NCHITTEST => handlers::WM_NCHITTEST(hwnd, message, wparam, lparam),
            WM_MOUSEMOVE => handlers::WM_MOUSEMOVE(hwnd, message, wparam, lparam),
            WM_MOUSELEAVE => handlers::WM_MOUSELEAVE(hwnd, message, wparam, lparam),
            WM_LBUTTONDOWN => handlers::WM_LBUTTONDOWN(hwnd, message, wparam, lparam),
            WM_LBUTTONUP => handlers::WM_LBUTTONUP(hwnd, message, wparam, lparam),
            WM_RBUTTONDOWN => handlers::WM_RBUTTONDOWN(hwnd, message, wparam, lparam),
            WM_RBUTTONUP => handlers::WM_RBUTTONUP(hwnd, message, wparam, lparam),
            WM_MBUTTONDOWN => handlers::WM_MBUTTONDOWN(hwnd, message, wparam, lparam),
            WM_MBUTTONUP => handlers::WM_MBUTTONUP(hwnd, message, wparam, lparam),
            WM_XBUTTONDOWN => handlers::WM_XBUTTONDOWN(hwnd, message, wparam, lparam),
            WM_XBUTTONUP => handlers::WM_XBUTTONUP(hwnd, message, wparam, lparam),
            WM_LBUTTONDBLCLK => handlers::WM_LBUTTONDBLCLK(hwnd, message, wparam, lparam),
            WM_RBUTTONDBLCLK => handlers::WM_RBUTTONDBLCLK(hwnd, message, wparam, lparam),
            WM_MBUTTONDBLCLK => handlers::WM_MBUTTONDBLCLK(hwnd, message, wparam, lparam),
            WM_XBUTTONDBLCLK => handlers::WM_XBUTTONDBLCLK(hwnd, message, wparam, lparam),
            WM_MOUSEWHEEL => handlers::WM_MOUSEWHEEL(hwnd, message, wparam, lparam),
            WM_MOUSEHWHEEL => handlers::WM_MOUSEHWHEEL(hwnd, message, wparam, lparam),
            WM_KEYDOWN => handlers::WM_KEYDOWN(hwnd, message, wparam, lparam),
            WM_CANCELMODE => handlers::WM_CANCELMODE(hwnd, message, wparam, lparam),
            _ => None,
        }
    };

    result.unwrap_or_else(|| unsafe { DefWindowProcW(hwnd, message, wparam, lparam) })
}

/// hwndからEntity IDを取得するヘルパー関数
#[inline]
pub(crate) fn get_entity_from_hwnd(hwnd: HWND) -> Option<Entity> {
    unsafe {
        let entity_bits = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
        Entity::try_from_bits(entity_bits as u64)
    }
}
