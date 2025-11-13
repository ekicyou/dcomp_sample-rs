use bevy_ecs::prelude::*;
use windows::Win32::Foundation::*;
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
                    // Entity IDをGWLP_USERDATAに保存（ID 0も有効）
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, entity_bits);
                }
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
            WM_NCDESTROY => {
                // クリーンアップ（WM_NCCREATEに対応する最後のメッセージ）
                // Entity IDを取得してエンティティを削除
                if let Some(entity) = get_entity_from_hwnd(hwnd) {
                    if let Some(world) = try_get_ecs_world() {
                        let mut world = world.borrow_mut();

                        // エンティティを削除（関連する全コンポーネントも削除される）
                        // on_window_handle_removedシステムが自動的にApp通知を行う
                        world.world_mut().despawn(entity);
                    }
                }

                // GWLP_USERDATAをクリア
                SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);

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
            _ => DefWindowProcW(hwnd, message, wparam, lparam),
        }
    }
}

/// hwndからEntity IDを取得するヘルパー関数
pub fn get_entity_from_hwnd(hwnd: HWND) -> Option<Entity> {
    unsafe {
        let entity_bits = GetWindowLongPtrW(hwnd, GWLP_USERDATA);
        Entity::try_from_bits(entity_bits as u64)
    }
}
