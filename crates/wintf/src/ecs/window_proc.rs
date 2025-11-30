use bevy_ecs::prelude::*;
use tracing::{debug, trace, warn};
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

/// EcsWorldへの参照を取得（try_borrow_mut可能な状態で）
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
            WM_WINDOWPOSCHANGED => {
                // ======================================================================
                // World借用区切り方式によるWM_WINDOWPOSCHANGED処理
                //
                // WM_WINDOWPOSCHANGED処理内でWorldを借用する際は、短く区切って都度解放する。
                // これにより、try_tick_on_vsync()やflush_window_pos_commands()が
                // 安全にWorldを借用できる。
                //
                // 処理フロー:
                // ① World借用 → DPI更新, WindowPosChanged=true, WindowPos更新, BoxStyle更新 → 借用解放
                // ② try_tick_on_vsync() (内部で借用→解放)
                // ③ flush_window_pos_commands() (SetWindowPos実行)
                // ④ World借用 → WindowPosChanged=false → 借用解放
                // ======================================================================

                // ------------------------------------------------------------------
                // ① 第1借用セクション: DPI更新, WindowPosChanged=true, WindowPos/BoxStyle更新
                // ------------------------------------------------------------------
                if let Some(entity) = get_entity_from_hwnd(hwnd) {
                    if let Some(world) = try_get_ecs_world() {
                        // DpiChangeContextを先に取得（try_tick_on_vsync前に消費する必要がある）
                        let dpi_context = crate::ecs::window::DpiChangeContext::take();

                        // RefCellが既に借用されている場合はスキップ（再入時）
                        if let Ok(mut world_borrow) = world.try_borrow_mut() {
                            let windowpos = lparam.0 as *const WINDOWPOS;
                            if !windowpos.is_null() {
                                let wp = &*windowpos;

                                if let Ok(mut entity_ref) =
                                    world_borrow.world_mut().get_entity_mut(entity)
                                {
                                    // DpiChangeContextが存在する場合、DPIコンポーネントを即時更新
                                    // これにより、以降の物理→論理座標変換で新DPIが使用される
                                    let dpi = if let Some(ref ctx) = dpi_context {
                                        // 新DPIでDPIコンポーネントを更新
                                        if let Some(mut dpi_comp) =
                                            entity_ref.get_mut::<crate::ecs::window::DPI>()
                                        {
                                            let old_dpi = *dpi_comp;
                                            *dpi_comp = ctx.new_dpi;
                                            trace!(
                                                entity = ?entity,
                                                old_dpi_x = old_dpi.dpi_x,
                                                old_dpi_y = old_dpi.dpi_y,
                                                new_dpi_x = ctx.new_dpi.dpi_x,
                                                new_dpi_y = ctx.new_dpi.dpi_y,
                                                "DPI updated via DpiChangeContext"
                                            );
                                        }
                                        ctx.new_dpi
                                    } else {
                                        // コンテキストがない場合は既存のDPIを使用
                                        entity_ref
                                            .get::<crate::ecs::window::DPI>()
                                            .copied()
                                            .unwrap_or_default()
                                    };

                                    // WindowPosChangedフラグをtrueに設定
                                    // apply_window_pos_changesでのSetWindowPos生成を抑制する
                                    if let Some(mut wpc) =
                                        entity_ref.get_mut::<crate::ecs::window::WindowPosChanged>()
                                    {
                                        wpc.0 = true;
                                        trace!(
                                            entity = ?entity,
                                            "WindowPosChanged set to true"
                                        );
                                    }

                                    // WindowHandleを取得してウィンドウ座標→クライアント座標に変換
                                    let client_coords = entity_ref
                                        .get::<crate::ecs::window::WindowHandle>()
                                        .and_then(|handle| {
                                            handle
                                                .window_to_client_coords(wp.x, wp.y, wp.cx, wp.cy)
                                                .ok()
                                        });

                                    // クライアント座標が取得できた場合のみ処理
                                    if let Some((client_pos, client_size)) = client_coords {
                                        if let Some(mut window_pos) =
                                            entity_ref.get_mut::<crate::ecs::window::WindowPos>()
                                        {
                                            // WindowPosを更新
                                            window_pos.position = Some(client_pos);
                                            window_pos.size = Some(client_size);
                                            // last_sentに現在値を設定（エコーバック判定用）
                                            window_pos.last_sent_position =
                                                Some((client_pos.x, client_pos.y));
                                            window_pos.last_sent_size =
                                                Some((client_size.cx, client_size.cy));

                                            trace!(
                                                entity = ?entity,
                                                window_x = wp.x,
                                                window_y = wp.y,
                                                window_cx = wp.cx,
                                                window_cy = wp.cy,
                                                client_x = client_pos.x,
                                                client_y = client_pos.y,
                                                client_cx = client_size.cx,
                                                client_cy = client_size.cy,
                                                "WindowPos updated"
                                            );
                                        }

                                        // BoxStyleがあれば更新（なければスキップ）
                                        // 注: BoxStyleは論理座標（DIP）を使用するため、物理ピクセルから変換が必要
                                        if let Some(mut box_style) =
                                            entity_ref.get_mut::<crate::ecs::layout::BoxStyle>()
                                        {
                                            use crate::ecs::layout::{
                                                BoxInset, BoxSize, Dimension, LengthPercentageAuto,
                                                Rect,
                                            };

                                            // 物理ピクセルサイズ→DIPサイズに変換（新DPIを使用）
                                            let (logical_width, logical_height) =
                                                dpi.to_logical_size(client_size.cx, client_size.cy);
                                            // 物理ピクセル位置→DIP位置に変換（新DPIを使用）
                                            let (logical_x, logical_y) =
                                                dpi.to_logical_point(client_pos.x, client_pos.y);

                                            // サイズを更新（DIP単位）
                                            box_style.size = Some(BoxSize {
                                                width: Some(Dimension::Px(logical_width)),
                                                height: Some(Dimension::Px(logical_height)),
                                            });

                                            // 位置を更新（絶対配置のinset、DIP単位）
                                            box_style.inset = Some(BoxInset(Rect {
                                                left: LengthPercentageAuto::Px(logical_x),
                                                top: LengthPercentageAuto::Px(logical_y),
                                                right: LengthPercentageAuto::Auto,
                                                bottom: LengthPercentageAuto::Auto,
                                            }));

                                            trace!(
                                                entity = ?entity,
                                                physical_x = client_pos.x,
                                                physical_y = client_pos.y,
                                                physical_cx = client_size.cx,
                                                physical_cy = client_size.cy,
                                                logical_x = format_args!("{:.1}", logical_x),
                                                logical_y = format_args!("{:.1}", logical_y),
                                                logical_width = format_args!("{:.1}", logical_width),
                                                logical_height = format_args!("{:.1}", logical_height),
                                                scale_x = format_args!("{:.2}", dpi.scale_x()),
                                                scale_y = format_args!("{:.2}", dpi.scale_y()),
                                                "BoxStyle updated (DIP)"
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        // world_borrowスコープ終了: 借用解放

                        // ------------------------------------------------------------------
                        // ② try_tick_on_vsync() (内部で借用→解放)
                        // ------------------------------------------------------------------
                        {
                            use crate::ecs::world::VsyncTick;
                            let _ = world.try_tick_on_vsync();
                        }

                        // ------------------------------------------------------------------
                        // ③ flush_window_pos_commands() (SetWindowPos実行)
                        // World借用解放後なので安全
                        // ------------------------------------------------------------------
                        crate::ecs::window::flush_window_pos_commands();

                        // ------------------------------------------------------------------
                        // ④ 第2借用セクション: WindowPosChanged=false
                        // ------------------------------------------------------------------
                        if let Ok(mut world_borrow) = world.try_borrow_mut() {
                            if let Ok(mut entity_ref) =
                                world_borrow.world_mut().get_entity_mut(entity)
                            {
                                if let Some(mut wpc) =
                                    entity_ref.get_mut::<crate::ecs::window::WindowPosChanged>()
                                {
                                    wpc.0 = false;
                                    trace!(
                                        entity = ?entity,
                                        "WindowPosChanged reset to false"
                                    );
                                }
                            }
                        }
                    }
                }
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
            WM_DISPLAYCHANGE => {
                // ディスプレイ構成が変更された
                if let Some(world) = try_get_ecs_world() {
                    if let Ok(mut world_borrow) = world.try_borrow_mut() {
                        if let Some(mut app) = world_borrow
                            .world_mut()
                            .get_resource_mut::<crate::ecs::App>()
                        {
                            app.mark_display_change();
                        }
                    }
                }
                DefWindowProcW(hwnd, message, wparam, lparam)
            }
            WM_DPICHANGED => {
                // DPIが変更された（モニター間移動など）
                // Per-Monitor DPI Aware (v2)では、アプリケーションが明示的にSetWindowPosを呼ぶ必要がある
                // DefWindowProcWは自動的にSetWindowPosを呼ばない

                let new_dpi = crate::ecs::window::DPI::from_WM_DPICHANGED(wparam, lparam);

                // lparam から suggested_rect を取得
                let suggested_rect_ptr = lparam.0 as *const RECT;
                let suggested_rect = if !suggested_rect_ptr.is_null() {
                    *suggested_rect_ptr
                } else {
                    RECT::default()
                };

                debug!(
                    hwnd = ?hwnd,
                    dpi_x = new_dpi.dpi_x,
                    dpi_y = new_dpi.dpi_y,
                    scale_x = format_args!("{:.2}", new_dpi.scale_x()),
                    scale_y = format_args!("{:.2}", new_dpi.scale_y()),
                    suggested_left = suggested_rect.left,
                    suggested_top = suggested_rect.top,
                    suggested_right = suggested_rect.right,
                    suggested_bottom = suggested_rect.bottom,
                    "WM_DPICHANGED"
                );

                // DpiChangeContextをスレッドローカルに保存
                // SetWindowPos → WM_WINDOWPOSCHANGED の流れで
                // WM_WINDOWPOSCHANGEDがこのコンテキストを消費する
                crate::ecs::window::DpiChangeContext::set(
                    crate::ecs::window::DpiChangeContext::new(new_dpi, suggested_rect),
                );

                // 明示的にSetWindowPosを呼び出してsuggested_rectを適用
                // これによりWM_WINDOWPOSCHANGEDが同期的に発火し、DpiChangeContextを消費する
                let width = suggested_rect.right - suggested_rect.left;
                let height = suggested_rect.bottom - suggested_rect.top;
                trace!(
                    hwnd = ?hwnd,
                    x = suggested_rect.left,
                    y = suggested_rect.top,
                    width,
                    height,
                    "Calling SetWindowPos with suggested_rect"
                );
                let result = SetWindowPos(
                    hwnd,
                    None,
                    suggested_rect.left,
                    suggested_rect.top,
                    width,
                    height,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                );
                if let Err(e) = result {
                    warn!(hwnd = ?hwnd, error = ?e, "SetWindowPos failed in WM_DPICHANGED");
                }

                LRESULT(0)
            }
            crate::win_thread_mgr::WM_DPICHANGED_DEFERRED => {
                // 非推奨: REQ-010により廃止予定
                // 互換性のため残しているが、新しい同期型処理では使用されない
                trace!(
                    hwnd = ?hwnd,
                    wparam = ?wparam,
                    "WM_DPICHANGED_DEFERRED (deprecated, ignored)"
                );
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
