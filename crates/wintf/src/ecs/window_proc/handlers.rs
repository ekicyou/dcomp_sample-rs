//! Windowsメッセージハンドラ関数
//!
//! 各関数はWindowsメッセージ定数と同じ名前を持ち、
//! 統一されたシグネチャ `fn(HWND, u32, WPARAM, LPARAM) -> Option<LRESULT>` を使用する。
//!
//! - `Some(LRESULT)`: 処理完了、この値を返す
//! - `None`: DefWindowProcWに委譲

#![allow(non_snake_case)]

use tracing::{debug, trace, warn};
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;

/// メッセージハンドラの戻り値型
/// - Some(LRESULT): 処理完了、この値を返す
/// - None: DefWindowProcWに委譲
type HandlerResult = Option<LRESULT>;

/// WM_NCCREATE: ウィンドウ作成時の非クライアント領域初期化
///
/// Entity IDをGWLP_USERDATAに保存する
#[inline]
pub(super) unsafe fn WM_NCCREATE(
    hwnd: HWND,
    _message: u32,
    _wparam: WPARAM,
    lparam: LPARAM,
) -> HandlerResult {
    let cs = lparam.0 as *const CREATESTRUCTW;
    if !cs.is_null() {
        let entity_bits = (*cs).lpCreateParams as isize;
        // Entity IDをGWLP_USERDATAに保存（ID 0も有効）
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, entity_bits);
    }
    None // DefWindowProcWに委譲
}

/// WM_NCDESTROY: ウィンドウ破棄時のクリーンアップ
///
/// Entity IDを取得してエンティティを削除し、GWLP_USERDATAをクリアする
#[inline]
pub(super) unsafe fn WM_NCDESTROY(
    hwnd: HWND,
    _message: u32,
    _wparam: WPARAM,
    _lparam: LPARAM,
) -> HandlerResult {
    // Entity IDを取得してエンティティを削除
    if let Some(entity) = super::get_entity_from_hwnd(hwnd) {
        if let Some(world) = super::try_get_ecs_world() {
            let mut world = world.borrow_mut();

            // エンティティを削除（関連する全コンポーネントも削除される）
            // on_window_handle_removedシステムが自動的にApp通知を行う
            world.world_mut().despawn(entity);
        }
    }

    // GWLP_USERDATAをクリア
    SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);

    None // DefWindowProcWに委譲
}

/// WM_ERASEBKGND: 背景消去要求
///
/// DirectCompositionで描画するため、背景消去をスキップする
#[inline]
pub(super) unsafe fn WM_ERASEBKGND(
    _hwnd: HWND,
    _message: u32,
    _wparam: WPARAM,
    _lparam: LPARAM,
) -> HandlerResult {
    Some(LRESULT(1)) // 背景消去をスキップ
}

/// WM_PAINT: 再描画要求
///
/// DirectCompositionで描画するため、領域を無効化解除するだけ
#[inline]
pub(super) unsafe fn WM_PAINT(
    hwnd: HWND,
    _message: u32,
    _wparam: WPARAM,
    _lparam: LPARAM,
) -> HandlerResult {
    use windows::Win32::Graphics::Gdi::ValidateRect;
    let _ = ValidateRect(Some(hwnd), None);
    Some(LRESULT(0))
}

/// WM_CLOSE: ウィンドウクローズ要求
///
/// DestroyWindowを呼び出してウィンドウを破棄する
#[inline]
pub(super) unsafe fn WM_CLOSE(
    hwnd: HWND,
    _message: u32,
    _wparam: WPARAM,
    _lparam: LPARAM,
) -> HandlerResult {
    let _ = DestroyWindow(hwnd);
    Some(LRESULT(0))
}

/// WM_WINDOWPOSCHANGED: ウィンドウ位置/サイズ変更通知
///
/// World借用区切り方式による処理:
/// ① World借用 → DPI更新, WindowPosChanged=true, WindowPos更新, BoxStyle更新 → 借用解放
/// ② try_tick_on_vsync() (内部で借用→解放)
/// ③ flush_window_pos_commands() (SetWindowPos実行)
/// ④ World借用 → WindowPosChanged=false → 借用解放
#[inline]
pub(super) unsafe fn WM_WINDOWPOSCHANGED(
    hwnd: HWND,
    _message: u32,
    _wparam: WPARAM,
    lparam: LPARAM,
) -> HandlerResult {
    // ------------------------------------------------------------------
    // ① 第1借用セクション: DPI更新, WindowPosChanged=true, WindowPos/BoxStyle更新
    // ------------------------------------------------------------------
    if let Some(entity) = super::get_entity_from_hwnd(hwnd) {
        if let Some(world) = super::try_get_ecs_world() {
            // DpiChangeContextを先に取得（try_tick_on_vsync前に消費する必要がある）
            let dpi_context = crate::ecs::window::DpiChangeContext::take();

            // RefCellが既に借用されている場合はスキップ（再入時）
            if let Ok(mut world_borrow) = world.try_borrow_mut() {
                let windowpos = lparam.0 as *const WINDOWPOS;
                if !windowpos.is_null() {
                    let wp = &*windowpos;

                    if let Ok(mut entity_ref) = world_borrow.world_mut().get_entity_mut(entity) {
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
                                window_pos.last_sent_position = Some((client_pos.x, client_pos.y));
                                window_pos.last_sent_size = Some((client_size.cx, client_size.cy));

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
                                    BoxInset, BoxSize, Dimension, LengthPercentageAuto, Rect,
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
                if let Ok(mut entity_ref) = world_borrow.world_mut().get_entity_mut(entity) {
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
    None // DefWindowProcWに委譲
}

/// WM_DISPLAYCHANGE: ディスプレイ構成変更通知
///
/// Appリソースのmark_display_changeを呼び出す
#[inline]
pub(super) unsafe fn WM_DISPLAYCHANGE(
    _hwnd: HWND,
    _message: u32,
    _wparam: WPARAM,
    _lparam: LPARAM,
) -> HandlerResult {
    if let Some(world) = super::try_get_ecs_world() {
        if let Ok(mut world_borrow) = world.try_borrow_mut() {
            if let Some(mut app) = world_borrow
                .world_mut()
                .get_resource_mut::<crate::ecs::App>()
            {
                app.mark_display_change();
            }
        }
    }
    None // DefWindowProcWに委譲
}

/// WM_DPICHANGED: DPI変更通知（モニター間移動など）
///
/// Per-Monitor DPI Aware (v2)では、アプリケーションが明示的にSetWindowPosを呼ぶ必要がある
/// DpiChangeContextを設定し、SetWindowPosを呼び出す
#[inline]
pub(super) unsafe fn WM_DPICHANGED(
    hwnd: HWND,
    _message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> HandlerResult {
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
    crate::ecs::window::DpiChangeContext::set(crate::ecs::window::DpiChangeContext::new(
        new_dpi,
        suggested_rect,
    ));

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

    Some(LRESULT(0))
}
