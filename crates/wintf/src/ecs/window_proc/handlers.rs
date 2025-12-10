//! Windowsメッセージハンドラ関数
//!
//! 各関数はWindowsメッセージ定数と同じ名前を持ち、
//! 統一されたシグネチャ `fn(HWND, u32, WPARAM, LPARAM) -> Option<LRESULT>` を使用する。
//!
//! - `Some(LRESULT)`: 処理完了、この値を返す
//! - `None`: DefWindowProcWに委譲

#![allow(non_snake_case)]

use tracing::{debug, info, trace, warn};
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
// Note: SetCapture/ReleaseCapture might require additional features or might not be exposed
// For MVP, we'll implement drag without explicit capture (Windows provides implicit capture during drag)

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

                                info!(
                                    "[WM_WINDOWPOSCHANGED] client_x={}, client_y={}",
                                    client_pos.x, client_pos.y
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

                                // Window の offset は物理ピクセル単位（LayoutRoot は scale=1.0）
                                // Window の size は論理ピクセル単位（Taffy が DIP で計算するため）
                                let physical_x = client_pos.x as f32;
                                let physical_y = client_pos.y as f32;
                                let physical_width = client_size.cx as f32;
                                let physical_height = client_size.cy as f32;

                                // DPI スケールで割って論理サイズに変換
                                let scale_x = dpi.scale_x();
                                let scale_y = dpi.scale_y();
                                let logical_width = physical_width / scale_x;
                                let logical_height = physical_height / scale_y;

                                // サイズを更新（論理ピクセル単位）
                                box_style.size = Some(BoxSize {
                                    width: Some(Dimension::Px(logical_width)),
                                    height: Some(Dimension::Px(logical_height)),
                                });

                                // 位置を更新（絶対配置のinset、物理ピクセル単位）
                                box_style.inset = Some(BoxInset(Rect {
                                    left: LengthPercentageAuto::Px(physical_x),
                                    top: LengthPercentageAuto::Px(physical_y),
                                    right: LengthPercentageAuto::Auto,
                                    bottom: LengthPercentageAuto::Auto,
                                }));

                                trace!(
                                    entity = ?entity,
                                    physical_x = physical_x,
                                    physical_y = physical_y,
                                    physical_width = physical_width,
                                    physical_height = physical_height,
                                    logical_width = logical_width,
                                    logical_height = logical_height,
                                    scale_x = scale_x,
                                    scale_y = scale_y,
                                    "BoxStyle updated: position (physical pixels), size (logical pixels)"
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

// ============================================================================
// マウスメッセージハンドラ (Task 3.1-3.4, 4.1)
// ============================================================================

/// WM_NCHITTEST: 非クライアント領域ヒットテスト
///
/// クライアント領域判定を実装し、キャッシュ付きhit_testを呼び出す。
/// HTCLIENT / HTTRANSPARENT を返す（クリックスルー対応）。
#[inline]
pub(super) unsafe fn WM_NCHITTEST(
    hwnd: HWND,
    _message: u32,
    _wparam: WPARAM,
    lparam: LPARAM,
) -> HandlerResult {
    use windows::Win32::UI::WindowsAndMessaging::GetClientRect;

    // スクリーン座標を取得（lparam から）
    let x = (lparam.0 & 0xFFFF) as i16 as i32;
    let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

    // クライアント領域判定用に座標変換（クライアント領域外はDefWindowProcWに委譲）
    {
        use windows::Win32::Graphics::Gdi::ScreenToClient;
        let mut pt = POINT { x, y };
        if !ScreenToClient(hwnd, &mut pt).as_bool() {
            return None; // DefWindowProcW に委譲
        }

        // クライアント領域外の場合は DefWindowProcW に委譲
        let mut rect = RECT::default();
        if GetClientRect(hwnd, &mut rect).is_err() {
            return None;
        }
        if pt.x < rect.left || pt.x >= rect.right || pt.y < rect.top || pt.y >= rect.bottom {
            return None; // DefWindowProcW に委譲（非クライアント領域処理）
        }
    }

    // Entity 取得
    let Some(entity) = super::get_entity_from_hwnd(hwnd) else {
        return None;
    };

    // World 取得
    let Some(world) = super::try_get_ecs_world() else {
        return None;
    };

    // キャッシュ付きヒットテスト実行
    crate::ecs::nchittest_cache::cached_nchittest(hwnd, (x, y), entity, &world)
}

/// WM_MOUSEMOVE: マウス移動メッセージ
///
/// 位置をPointerBufferに蓄積し、hit_testでヒットしたエンティティにPointerStateを付与。
/// 初回移動時にTrackMouseEventを設定。
#[inline]
pub(super) unsafe fn WM_MOUSEMOVE(
    hwnd: HWND,
    _message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> HandlerResult {
    use crate::ecs::layout::hit_test::{hit_test_in_window, PhysicalPoint as HitTestPoint};
    use crate::ecs::pointer::{
        push_pointer_sample, set_modifier_state, PointerLeave, PointerState, WindowPointerTracking,
    };
    use std::time::Instant;
    use windows::Win32::UI::Input::KeyboardAndMouse::{
        TrackMouseEvent, TME_LEAVE, TRACKMOUSEEVENT,
    };

    let Some(window_entity) = super::get_entity_from_hwnd(hwnd) else {
        return None;
    };

    // 位置取得（物理ピクセル、クライアント座標）
    let x = (lparam.0 & 0xFFFF) as i16 as i32;
    let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

    tracing::trace!("[WM_MOUSEMOVE] Received: x={}, y={}", x, y);

    // 修飾キー状態を抽出
    let wparam_val = wparam.0 as u32;
    let shift = (wparam_val & 0x04) != 0; // MK_SHIFT
    let ctrl = (wparam_val & 0x08) != 0; // MK_CONTROL

    // World借用してhit_testとPointerState管理
    if let Some(world) = super::try_get_ecs_world() {
        if let Ok(mut world_borrow) = world.try_borrow_mut() {
            // TrackMouseEvent 設定（ウィンドウに対して）
            if let Ok(mut entity_ref) = world_borrow.world_mut().get_entity_mut(window_entity) {
                let needs_tracking = entity_ref
                    .get::<WindowPointerTracking>()
                    .is_none_or(|t| !t.0);

                if needs_tracking {
                    let mut tme = TRACKMOUSEEVENT {
                        cbSize: std::mem::size_of::<TRACKMOUSEEVENT>() as u32,
                        dwFlags: TME_LEAVE,
                        hwndTrack: hwnd,
                        dwHoverTime: 0,
                    };
                    let _ = TrackMouseEvent(&mut tme);

                    if entity_ref.get::<WindowPointerTracking>().is_some() {
                        if let Some(mut tracking) = entity_ref.get_mut::<WindowPointerTracking>() {
                            tracking.0 = true;
                        }
                    } else {
                        drop(entity_ref);
                        world_borrow
                            .world_mut()
                            .entity_mut(window_entity)
                            .insert(WindowPointerTracking(true));
                    }

                    trace!(
                        entity = ?window_entity,
                        hwnd = ?hwnd,
                        "TrackMouseEvent(TME_LEAVE) set"
                    );
                }
            }

            // hit_test でヒットしたエンティティを取得
            let hit_entity = hit_test_in_window(
                world_borrow.world(),
                window_entity,
                HitTestPoint::new(x as f32, y as f32),
            );

            // WindowPosを取得してスクリーン座標を計算
            let window_pos = world_borrow
                .world()
                .get::<crate::ecs::window::WindowPos>(window_entity);
            let (screen_x, screen_y) = if let Some(wp) = window_pos {
                if let Some(pos) = wp.position {
                    (x + pos.x, y + pos.y)
                } else {
                    (x, y)
                }
            } else {
                (x, y)
            };

            // ドラッグ処理（thread_local DragState + DragAccumulatorResource）
            // ヒットテスト結果に関わらず、Preparing/Dragging状態なら処理を続ける
            let state_snapshot = crate::ecs::drag::read_drag_state(|state| state.clone());

            match state_snapshot {
                crate::ecs::drag::DragState::Preparing {
                    entity,
                    start_pos,
                    start_time,
                } => {
                    if let Some(drag_config) = world_borrow
                        .world()
                        .get::<crate::ecs::drag::DragConfig>(entity)
                    {
                        if drag_config.enabled {
                            let current_pos =
                                crate::ecs::pointer::PhysicalPoint::new(screen_x, screen_y);

                            // 閾値チェック
                            let dx = current_pos.x - start_pos.x;
                            let dy = current_pos.y - start_pos.y;
                            let distance_sq = dx * dx + dy * dy;
                            let threshold_sq = drag_config.threshold * drag_config.threshold;

                            if distance_sq >= threshold_sq {
                                // 閾値到達：Dragging状態に遷移
                                crate::ecs::drag::start_dragging(current_pos);

                                // DragAccumulatorResourceにStarted遷移を記録
                                if let Some(accumulator) = world_borrow
                                    .world()
                                    .get_resource::<crate::ecs::drag::DragAccumulatorResource>(
                                ) {
                                    accumulator.set_transition(
                                        crate::ecs::drag::DragTransition::Started {
                                            entity,
                                            start_pos,
                                            timestamp: start_time,
                                        },
                                    );
                                    accumulator.update_position(current_pos);
                                }
                            }
                        }
                    }
                }
                crate::ecs::drag::DragState::Dragging { prev_pos, .. } => {
                    let current_pos = crate::ecs::pointer::PhysicalPoint::new(screen_x, screen_y);

                    // ドラッグ中：デルタを累積
                    let delta = crate::ecs::pointer::PhysicalPoint::new(
                        current_pos.x - prev_pos.x,
                        current_pos.y - prev_pos.y,
                    );

                    tracing::trace!(
                        "[WM_MOUSEMOVE] Dragging: delta=({}, {}), current=({}, {})",
                        delta.x,
                        delta.y,
                        current_pos.x,
                        current_pos.y
                    );

                    // DragAccumulatorResourceにデルタを累積
                    if let Some(accumulator) = world_borrow
                        .world()
                        .get_resource::<crate::ecs::drag::DragAccumulatorResource>(
                    ) {
                        accumulator.accumulate_delta(delta);
                        accumulator.update_position(current_pos);
                    }

                    // thread_local DragStateのprev_posを更新
                    crate::ecs::drag::update_dragging(current_pos);
                }
                _ => {}
            }

            // ヒットしたエンティティが存在する場合
            if let Some(target_entity) = hit_entity {
                // 現在PointerStateを持っている全エンティティを探す
                // 異なるエンティティにPointerStateがある場合はLeave処理
                let mut entities_to_leave = Vec::new();
                {
                    let world_mut = world_borrow.world_mut();
                    let mut query = world_mut.query::<(bevy_ecs::prelude::Entity, &PointerState)>();
                    for (e, _) in query.iter(world_mut) {
                        if e != target_entity {
                            entities_to_leave.push(e);
                        }
                    }
                }

                // 古いエンティティからPointerStateを削除し、PointerLeaveを付与
                for old_entity in entities_to_leave {
                    if let Ok(mut entity_ref) = world_borrow.world_mut().get_entity_mut(old_entity)
                    {
                        entity_ref.remove::<PointerState>();
                        entity_ref.insert(PointerLeave);
                        debug!(
                            old_entity = ?old_entity,
                            new_entity = ?target_entity,
                            "PointerState moved, Leave marker inserted"
                        );
                    }
                }

                // 新しいエンティティにPointerStateを挿入または維持
                if let Ok(entity_ref) = world_borrow.world_mut().get_entity_mut(target_entity) {
                    let needs_insert = entity_ref.get::<PointerState>().is_none();
                    tracing::trace!(
                        entity = ?target_entity,
                        needs_insert,
                        "[WM_MOUSEMOVE] PointerState check"
                    );
                    drop(entity_ref);

                    if needs_insert {
                        world_borrow
                            .world_mut()
                            .entity_mut(target_entity)
                            .insert(PointerState {
                                screen_point: crate::ecs::pointer::PhysicalPoint::new(x, y),
                                local_point: crate::ecs::pointer::PhysicalPoint::new(x, y),
                                shift_down: shift,
                                ctrl_down: ctrl,
                                ..Default::default()
                            });
                        debug!(
                            entity = ?target_entity,
                            x, y,
                            "PointerState inserted (Enter)"
                        );
                    }
                } else {
                    tracing::warn!(
                        entity = ?target_entity,
                        "[WM_MOUSEMOVE] Failed to get entity_mut"
                    );
                }

                // PointerStateが存在することを保証した上で、バッファに蓄積
                push_pointer_sample(target_entity, x as f32, y as f32, Instant::now());
                set_modifier_state(target_entity, shift, ctrl);
            } else {
                // hit_test失敗（Windowの空白部分）
                // Windowエンティティに対して処理
                let target_entity = window_entity;

                // 現在PointerStateを持っている全エンティティを探す
                let mut entities_to_leave = Vec::new();
                {
                    let world_mut = world_borrow.world_mut();
                    let mut query = world_mut.query::<(bevy_ecs::prelude::Entity, &PointerState)>();
                    for (e, _) in query.iter(world_mut) {
                        if e != target_entity {
                            entities_to_leave.push(e);
                        }
                    }
                }

                // 古いエンティティからPointerStateを削除し、PointerLeaveを付与
                for old_entity in entities_to_leave {
                    if let Ok(mut entity_ref) = world_borrow.world_mut().get_entity_mut(old_entity)
                    {
                        entity_ref.remove::<PointerState>();
                        entity_ref.insert(PointerLeave);
                        debug!(
                            old_entity = ?old_entity,
                            new_entity = ?target_entity,
                            "PointerState moved to Window, Leave marker inserted"
                        );
                    }
                }

                // PointerStateを挿入または維持
                if let Ok(entity_ref) = world_borrow.world_mut().get_entity_mut(target_entity) {
                    let needs_insert = entity_ref.get::<PointerState>().is_none();
                    drop(entity_ref);

                    if needs_insert {
                        world_borrow
                            .world_mut()
                            .entity_mut(target_entity)
                            .insert(PointerState {
                                screen_point: crate::ecs::pointer::PhysicalPoint::new(x, y),
                                local_point: crate::ecs::pointer::PhysicalPoint::new(x, y),
                                shift_down: shift,
                                ctrl_down: ctrl,
                                ..Default::default()
                            });
                        debug!(
                            entity = ?target_entity,
                            x, y,
                            "PointerState inserted on Window (no hit)"
                        );
                    }
                }

                // PointerStateが存在することを保証した上で、バッファに蓄積
                push_pointer_sample(target_entity, x as f32, y as f32, Instant::now());
                set_modifier_state(target_entity, shift, ctrl);
            }
        }
    }

    Some(LRESULT(0))
}

/// WM_MOUSELEAVE: マウス離脱メッセージ
///
/// 全エンティティのPointerStateを削除し、PointerLeaveマーカーを付与する。
#[inline]
pub(super) unsafe fn WM_MOUSELEAVE(
    hwnd: HWND,
    _message: u32,
    _wparam: WPARAM,
    _lparam: LPARAM,
) -> HandlerResult {
    use crate::ecs::pointer::{PointerLeave, PointerState, WindowPointerTracking};

    let Some(window_entity) = super::get_entity_from_hwnd(hwnd) else {
        return None;
    };

    if let Some(world) = super::try_get_ecs_world() {
        if let Ok(mut world_borrow) = world.try_borrow_mut() {
            // PointerStateを持つ全エンティティを収集
            let mut entities_with_pointer_state = Vec::new();
            {
                let world_mut = world_borrow.world_mut();
                let mut query = world_mut.query::<(bevy_ecs::prelude::Entity, &PointerState)>();
                for (e, _) in query.iter(world_mut) {
                    entities_with_pointer_state.push(e);
                }
            }

            // 各エンティティからPointerStateを削除し、PointerLeaveを付与
            for entity in entities_with_pointer_state {
                if let Ok(mut entity_ref) = world_borrow.world_mut().get_entity_mut(entity) {
                    entity_ref.remove::<PointerState>();
                    entity_ref.insert(PointerLeave);
                    debug!(
                        entity = ?entity,
                        hwnd = ?hwnd,
                        "PointerLeave marker inserted"
                    );
                }
            }

            // WindowPointerTrackingを無効化
            if let Ok(mut entity_ref) = world_borrow.world_mut().get_entity_mut(window_entity) {
                if let Some(mut tracking) = entity_ref.get_mut::<WindowPointerTracking>() {
                    tracking.0 = false;
                }
            }
        }
    }

    Some(LRESULT(0))
}

/// ボタンメッセージハンドラ共通処理
///
/// hit_test でヒット対象エンティティを特定し、ButtonBuffer に記録する。
/// PointerState がない場合は付与する。
#[inline]
unsafe fn handle_button_message(
    hwnd: HWND,
    wparam: WPARAM,
    lparam: LPARAM,
    button: crate::ecs::pointer::PointerButton,
    is_down: bool,
) -> HandlerResult {
    use crate::ecs::layout::hit_test::{hit_test_in_window, PhysicalPoint as HitTestPoint};
    use crate::ecs::pointer::{PhysicalPoint, PointerState};

    let Some(window_entity) = super::get_entity_from_hwnd(hwnd) else {
        return None;
    };

    // クリック位置を取得
    let x = (lparam.0 & 0xFFFF) as i16 as i32;
    let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

    tracing::debug!(
        "[WM_BUTTON] button={:?}, is_down={}, x={}, y={}",
        button,
        is_down,
        x,
        y
    );

    // 修飾キー状態を抽出
    let wparam_val = wparam.0 as u32;
    let shift = (wparam_val & 0x04) != 0;
    let ctrl = (wparam_val & 0x08) != 0;

    // スクリーン座標を事前計算（フォールバック処理でも使用）
    let (screen_x, screen_y) = if let Some(world) = super::try_get_ecs_world() {
        if let Ok(world_borrow) = world.try_borrow() {
            let window_pos = world_borrow
                .world()
                .get::<crate::ecs::window::WindowPos>(window_entity);
            if let Some(wp) = window_pos {
                if let Some(pos) = wp.position {
                    (x + pos.x, y + pos.y)
                } else {
                    (x, y)
                }
            } else {
                (x, y)
            }
        } else {
            (x, y)
        }
    } else {
        (x, y)
    };

    // hit_test でターゲットエンティティを特定し、PointerState を確保
    if let Some(world) = super::try_get_ecs_world() {
        if let Ok(mut world_borrow) = world.try_borrow_mut() {
            if let Some(target_entity) = hit_test_in_window(
                world_borrow.world(),
                window_entity,
                HitTestPoint::new(x as f32, y as f32),
            ) {
                // ヒットしたエンティティの名前とGlobalArrangement.boundsをログ出力
                let entity_name = world_borrow
                    .world()
                    .get::<bevy_ecs::name::Name>(target_entity)
                    .map(|n| n.as_str().to_string())
                    .unwrap_or_else(|| format!("{:?}", target_entity));
                let bounds_info = world_borrow
                    .world()
                    .get::<crate::ecs::layout::GlobalArrangement>(target_entity)
                    .map(|g| {
                        format!(
                            "({:.0},{:.0})-({:.0},{:.0})",
                            g.bounds.left, g.bounds.top, g.bounds.right, g.bounds.bottom
                        )
                    })
                    .unwrap_or_else(|| "no bounds".to_string());
                info!(
                    target_entity = ?target_entity,
                    entity_name = %entity_name,
                    client_x = x,
                    client_y = y,
                    screen_x,
                    screen_y,
                    bounds = %bounds_info,
                    button = ?button,
                    is_down,
                    "[handle_button_message] hit_test result"
                );

                // PointerState がない場合は付与
                if world_borrow
                    .world()
                    .get::<PointerState>(target_entity)
                    .is_none()
                {
                    world_borrow
                        .world_mut()
                        .entity_mut(target_entity)
                        .insert(PointerState {
                            screen_point: PhysicalPoint::new(x, y),
                            local_point: PhysicalPoint::new(x, y),
                            left_down: button == crate::ecs::pointer::PointerButton::Left
                                && is_down,
                            right_down: button == crate::ecs::pointer::PointerButton::Right
                                && is_down,
                            middle_down: button == crate::ecs::pointer::PointerButton::Middle
                                && is_down,
                            shift_down: shift,
                            ctrl_down: ctrl,
                            ..Default::default()
                        });
                    debug!(
                        entity = ?target_entity,
                        button = ?button,
                        is_down,
                        "PointerState inserted on button event"
                    );
                }

                // 修飾キー状態を記録
                crate::ecs::pointer::set_modifier_state(target_entity, shift, ctrl);

                // ボタン状態をバッファに記録
                if is_down {
                    crate::ecs::pointer::record_button_down(target_entity, button);

                    // ドラッグ準備開始（DragConfigがあり、有効な場合）
                    if button == crate::ecs::pointer::PointerButton::Left {
                        if let Some(drag_config) = world_borrow
                            .world()
                            .get::<crate::ecs::drag::DragConfig>(target_entity)
                        {
                            if drag_config.enabled && drag_config.left_button {
                                tracing::info!(
                                    entity = ?target_entity,
                                    x = screen_x,
                                    y = screen_y,
                                    "[handle_button_message] Calling start_preparing"
                                );
                                crate::ecs::drag::start_preparing(
                                    target_entity,
                                    PhysicalPoint::new(screen_x, screen_y),
                                );
                                // TODO: SetCapture for proper mouse capture (not available in current windows crate version)
                                // let _ = unsafe { SetCapture(hwnd) };
                            }
                        }
                    }
                } else {
                    crate::ecs::pointer::record_button_up(target_entity, button);

                    // ドラッグ終了
                    if button == crate::ecs::pointer::PointerButton::Left {
                        // thread_local DragStateをクローンして取得
                        let state_snapshot =
                            crate::ecs::drag::read_drag_state(|state| state.clone());

                        if let crate::ecs::drag::DragState::Dragging { entity, .. } = state_snapshot
                        {
                            // DragAccumulatorResourceにEnded遷移を記録
                            if let Some(accumulator) = world_borrow
                                .world()
                                .get_resource::<crate::ecs::drag::DragAccumulatorResource>(
                            ) {
                                accumulator.set_transition(
                                    crate::ecs::drag::DragTransition::Ended {
                                        entity,
                                        end_pos: PhysicalPoint::new(screen_x, screen_y),
                                        cancelled: false,
                                    },
                                );
                            }
                        }

                        // thread_local DragStateをIdleに戻す
                        crate::ecs::drag::end_dragging(
                            PhysicalPoint::new(screen_x, screen_y),
                            false,
                        );
                        // TODO: ReleaseCapture (not available in current windows crate version)
                        // let _ = unsafe { ReleaseCapture() };
                    }
                }

                return Some(LRESULT(0));
            }
        }
    }

    // フォールバック: ウィンドウエンティティに記録
    if is_down {
        crate::ecs::pointer::record_button_down(window_entity, button);
    } else {
        crate::ecs::pointer::record_button_up(window_entity, button);

        // ドラッグ終了（hit_test失敗時でもドラッグ中なら終了処理）
        if button == crate::ecs::pointer::PointerButton::Left {
            if let Some(world) = super::try_get_ecs_world() {
                if let Ok(world_borrow) = world.try_borrow() {
                    // thread_local DragStateをクローンして取得
                    let state_snapshot = crate::ecs::drag::read_drag_state(|state| state.clone());

                    if let crate::ecs::drag::DragState::Dragging { entity, .. }
                    | crate::ecs::drag::DragState::Preparing { entity, .. }
                    | crate::ecs::drag::DragState::JustStarted { entity, .. } = state_snapshot
                    {
                        // DragAccumulatorResourceにEnded遷移を記録
                        if let Some(accumulator) = world_borrow
                            .world()
                            .get_resource::<crate::ecs::drag::DragAccumulatorResource>(
                        ) {
                            accumulator.set_transition(crate::ecs::drag::DragTransition::Ended {
                                entity,
                                end_pos: PhysicalPoint::new(screen_x, screen_y),
                                cancelled: false,
                            });
                        }
                    }
                }
            }

            // thread_local DragStateをIdleに戻す
            crate::ecs::drag::end_dragging(PhysicalPoint::new(screen_x, screen_y), false);
            // TODO: ReleaseCapture (not available in current windows crate version)
            // let _ = unsafe { ReleaseCapture() };
        }
    }

    Some(LRESULT(0))
}

/// WM_LBUTTONDOWN: 左ボタン押下
#[inline]
pub(super) unsafe fn WM_LBUTTONDOWN(
    hwnd: HWND,
    _message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> HandlerResult {
    handle_button_message(
        hwnd,
        wparam,
        lparam,
        crate::ecs::pointer::PointerButton::Left,
        true,
    )
}

/// WM_LBUTTONUP: 左ボタン解放
#[inline]
pub(super) unsafe fn WM_LBUTTONUP(
    hwnd: HWND,
    _message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> HandlerResult {
    handle_button_message(
        hwnd,
        wparam,
        lparam,
        crate::ecs::pointer::PointerButton::Left,
        false,
    )
}

/// WM_RBUTTONDOWN: 右ボタン押下
#[inline]
pub(super) unsafe fn WM_RBUTTONDOWN(
    hwnd: HWND,
    _message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> HandlerResult {
    handle_button_message(
        hwnd,
        wparam,
        lparam,
        crate::ecs::pointer::PointerButton::Right,
        true,
    )
}

/// WM_RBUTTONUP: 右ボタン解放
#[inline]
pub(super) unsafe fn WM_RBUTTONUP(
    hwnd: HWND,
    _message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> HandlerResult {
    handle_button_message(
        hwnd,
        wparam,
        lparam,
        crate::ecs::pointer::PointerButton::Right,
        false,
    )
}

/// WM_MBUTTONDOWN: 中ボタン押下
#[inline]
pub(super) unsafe fn WM_MBUTTONDOWN(
    hwnd: HWND,
    _message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> HandlerResult {
    handle_button_message(
        hwnd,
        wparam,
        lparam,
        crate::ecs::pointer::PointerButton::Middle,
        true,
    )
}

/// WM_MBUTTONUP: 中ボタン解放
#[inline]
pub(super) unsafe fn WM_MBUTTONUP(
    hwnd: HWND,
    _message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> HandlerResult {
    handle_button_message(
        hwnd,
        wparam,
        lparam,
        crate::ecs::pointer::PointerButton::Middle,
        false,
    )
}

/// WM_XBUTTONDOWN: 拡張ボタン押下
#[inline]
pub(super) unsafe fn WM_XBUTTONDOWN(
    hwnd: HWND,
    _message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> HandlerResult {
    // GET_XBUTTON_WPARAM: HIWORD of wParam
    let xbutton = ((wparam.0 >> 16) & 0xFFFF) as u16;
    let button = if xbutton == 1 {
        crate::ecs::pointer::PointerButton::XButton1
    } else {
        crate::ecs::pointer::PointerButton::XButton2
    };
    handle_button_message(hwnd, wparam, lparam, button, true)
}

/// WM_XBUTTONUP: 拡張ボタン解放
#[inline]
pub(super) unsafe fn WM_XBUTTONUP(
    hwnd: HWND,
    _message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> HandlerResult {
    let xbutton = ((wparam.0 >> 16) & 0xFFFF) as u16;
    let button = if xbutton == 1 {
        crate::ecs::pointer::PointerButton::XButton1
    } else {
        crate::ecs::pointer::PointerButton::XButton2
    };
    handle_button_message(hwnd, wparam, lparam, button, false)
}

/// ダブルクリックメッセージハンドラ共通処理
#[inline]
/// WM_*BUTTONDBLCLK ハンドラ共通処理
///
/// ダブルクリックイベントを処理し、hit_testでターゲットエンティティを特定して
/// PointerStateを付与する。WM_LBUTTONDOWNの代わりにWM_LBUTTONDBLCLKが来るため、
/// ボタン押下記録も同時に行う。
///
/// # Arguments
/// - `hwnd`: ウィンドウハンドル
/// - `wparam`: 修飾キー状態とXBUTTON情報
/// - `lparam`: クリック座標（クライアント座標）
/// - `double_click`: ダブルクリック種別
unsafe fn handle_double_click_message(
    hwnd: HWND,
    wparam: WPARAM,
    lparam: LPARAM,
    double_click: crate::ecs::pointer::DoubleClick,
) -> HandlerResult {
    use crate::ecs::layout::hit_test::{hit_test_in_window, PhysicalPoint as HitTestPoint};
    use crate::ecs::pointer::{PhysicalPoint, PointerState};

    let Some(window_entity) = super::get_entity_from_hwnd(hwnd) else {
        return None;
    };

    // クリック位置を取得
    let x = (lparam.0 & 0xFFFF) as i16 as i32;
    let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;

    // 修飾キー状態を抽出
    let wparam_val = wparam.0 as u32;
    let shift = (wparam_val & 0x04) != 0;
    let ctrl = (wparam_val & 0x08) != 0;

    // ダブルクリックに対応するボタンを取得
    let button = match double_click {
        crate::ecs::pointer::DoubleClick::Left => crate::ecs::pointer::PointerButton::Left,
        crate::ecs::pointer::DoubleClick::Right => crate::ecs::pointer::PointerButton::Right,
        crate::ecs::pointer::DoubleClick::Middle => crate::ecs::pointer::PointerButton::Middle,
        crate::ecs::pointer::DoubleClick::XButton1 => crate::ecs::pointer::PointerButton::XButton1,
        crate::ecs::pointer::DoubleClick::XButton2 => crate::ecs::pointer::PointerButton::XButton2,
        crate::ecs::pointer::DoubleClick::None => return Some(LRESULT(0)),
    };

    // hit_test でターゲットエンティティを特定し、PointerState を確保
    if let Some(world) = super::try_get_ecs_world() {
        if let Ok(mut world_borrow) = world.try_borrow_mut() {
            if let Some(target_entity) = hit_test_in_window(
                world_borrow.world(),
                window_entity,
                HitTestPoint::new(x as f32, y as f32),
            ) {
                tracing::info!(
                    window_entity = ?window_entity,
                    target_entity = ?target_entity,
                    double_click = ?double_click,
                    x, y,
                    "[handle_double_click_message] Double-click detected"
                );

                // PointerState がない場合は付与
                if world_borrow
                    .world()
                    .get::<PointerState>(target_entity)
                    .is_none()
                {
                    world_borrow
                        .world_mut()
                        .entity_mut(target_entity)
                        .insert(PointerState {
                            screen_point: PhysicalPoint::new(x, y),
                            local_point: PhysicalPoint::new(x, y),
                            left_down: button == crate::ecs::pointer::PointerButton::Left,
                            right_down: button == crate::ecs::pointer::PointerButton::Right,
                            middle_down: button == crate::ecs::pointer::PointerButton::Middle,
                            xbutton1_down: button == crate::ecs::pointer::PointerButton::XButton1,
                            xbutton2_down: button == crate::ecs::pointer::PointerButton::XButton2,
                            shift_down: shift,
                            ctrl_down: ctrl,
                            double_click,
                            ..Default::default()
                        });
                    debug!(
                        entity = ?target_entity,
                        button = ?button,
                        double_click = ?double_click,
                        "PointerState inserted on double-click event"
                    );
                } else {
                    // 既存の PointerState に double_click を設定
                    if let Some(mut ps) = world_borrow
                        .world_mut()
                        .get_mut::<PointerState>(target_entity)
                    {
                        ps.double_click = double_click;
                        ps.shift_down = shift;
                        ps.ctrl_down = ctrl;
                    }
                }

                // 修飾キー状態を記録
                crate::ecs::pointer::set_modifier_state(target_entity, shift, ctrl);

                // ボタン状態をバッファに記録
                crate::ecs::pointer::record_button_down(target_entity, button);
            }
        }
    }

    Some(LRESULT(0))
}

/// WM_LBUTTONDBLCLK: 左ボタンダブルクリック
#[inline]
pub(super) unsafe fn WM_LBUTTONDBLCLK(
    hwnd: HWND,
    _message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> HandlerResult {
    handle_double_click_message(hwnd, wparam, lparam, crate::ecs::pointer::DoubleClick::Left)
}

/// WM_RBUTTONDBLCLK: 右ボタンダブルクリック
#[inline]
pub(super) unsafe fn WM_RBUTTONDBLCLK(
    hwnd: HWND,
    _message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> HandlerResult {
    handle_double_click_message(
        hwnd,
        wparam,
        lparam,
        crate::ecs::pointer::DoubleClick::Right,
    )
}

/// WM_MBUTTONDBLCLK: 中ボタンダブルクリック
#[inline]
pub(super) unsafe fn WM_MBUTTONDBLCLK(
    hwnd: HWND,
    _message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> HandlerResult {
    handle_double_click_message(
        hwnd,
        wparam,
        lparam,
        crate::ecs::pointer::DoubleClick::Middle,
    )
}

/// WM_XBUTTONDBLCLK: 拡張ボタンダブルクリック
#[inline]
pub(super) unsafe fn WM_XBUTTONDBLCLK(
    hwnd: HWND,
    _message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> HandlerResult {
    let xbutton = ((wparam.0 >> 16) & 0xFFFF) as u16;
    let double_click = if xbutton == 1 {
        crate::ecs::pointer::DoubleClick::XButton1
    } else {
        crate::ecs::pointer::DoubleClick::XButton2
    };
    handle_double_click_message(hwnd, wparam, lparam, double_click)
}

/// WM_MOUSEWHEEL: 垂直ホイール回転
#[inline]
pub(super) unsafe fn WM_MOUSEWHEEL(
    hwnd: HWND,
    _message: u32,
    wparam: WPARAM,
    _lparam: LPARAM,
) -> HandlerResult {
    let Some(entity) = super::get_entity_from_hwnd(hwnd) else {
        return None;
    };

    // GET_WHEEL_DELTA_WPARAM: HIWORD of wParam (signed)
    let delta = ((wparam.0 >> 16) & 0xFFFF) as i16;
    crate::ecs::pointer::add_wheel_vertical(entity, delta);

    Some(LRESULT(0))
}

/// WM_MOUSEHWHEEL: 水平ホイール回転
#[inline]
pub(super) unsafe fn WM_MOUSEHWHEEL(
    hwnd: HWND,
    _message: u32,
    wparam: WPARAM,
    _lparam: LPARAM,
) -> HandlerResult {
    let Some(entity) = super::get_entity_from_hwnd(hwnd) else {
        return None;
    };

    let delta = ((wparam.0 >> 16) & 0xFFFF) as i16;
    crate::ecs::pointer::add_wheel_horizontal(entity, delta);

    Some(LRESULT(0))
}

/// WM_KEYDOWN: キー押下
#[inline]
pub(super) unsafe fn WM_KEYDOWN(
    _hwnd: HWND,
    _message: u32,
    wparam: WPARAM,
    _lparam: LPARAM,
) -> HandlerResult {
    use windows::Win32::UI::Input::KeyboardAndMouse::VK_ESCAPE;

    // ESCキーでドラッグキャンセル
    if wparam.0 == VK_ESCAPE.0 as usize {
        // thread_local DragStateをクローンして取得
        let state_snapshot = crate::ecs::drag::read_drag_state(|state| state.clone());

        if let crate::ecs::drag::DragState::Dragging {
            entity, start_pos, ..
        }
        | crate::ecs::drag::DragState::Preparing {
            entity, start_pos, ..
        }
        | crate::ecs::drag::DragState::JustStarted {
            entity, start_pos, ..
        } = state_snapshot
        {
            // DragAccumulatorResourceにEnded遷移を記録
            if let Some(world) = super::try_get_ecs_world() {
                if let Ok(world_borrow) = world.try_borrow() {
                    if let Some(accumulator) = world_borrow
                        .world()
                        .get_resource::<crate::ecs::drag::DragAccumulatorResource>(
                    ) {
                        accumulator.set_transition(crate::ecs::drag::DragTransition::Ended {
                            entity,
                            end_pos: start_pos,
                            cancelled: true,
                        });
                    }
                }
            }
        }

        crate::ecs::drag::cancel_dragging();
        // ReleaseCapture
        // TODO: ReleaseCapture (not available in current windows crate version)
        // let _ = unsafe { ReleaseCapture() };

        tracing::debug!("[WM_KEYDOWN] ESC key pressed, drag cancelled");
    }

    None // DefWindowProcWに委譲
}

/// WM_CANCELMODE: システムキャンセル
#[inline]
pub(super) unsafe fn WM_CANCELMODE(
    _hwnd: HWND,
    _message: u32,
    _wparam: WPARAM,
    _lparam: LPARAM,
) -> HandlerResult {
    // thread_local DragStateをクローンして取得
    let state_snapshot = crate::ecs::drag::read_drag_state(|state| state.clone());

    if let crate::ecs::drag::DragState::Dragging {
        entity, start_pos, ..
    }
    | crate::ecs::drag::DragState::Preparing {
        entity, start_pos, ..
    }
    | crate::ecs::drag::DragState::JustStarted {
        entity, start_pos, ..
    } = state_snapshot
    {
        // DragAccumulatorResourceにEnded遷移を記録
        if let Some(world) = super::try_get_ecs_world() {
            if let Ok(world_borrow) = world.try_borrow() {
                if let Some(accumulator) = world_borrow
                    .world()
                    .get_resource::<crate::ecs::drag::DragAccumulatorResource>(
                ) {
                    accumulator.set_transition(crate::ecs::drag::DragTransition::Ended {
                        entity,
                        end_pos: start_pos,
                        cancelled: true,
                    });
                }
            }
        }
    }

    // ドラッグキャンセル
    crate::ecs::drag::cancel_dragging();

    tracing::debug!("[WM_CANCELMODE] System cancel, drag cancelled");

    None // DefWindowProcWに委譲（ReleaseCapture自動実行）
}
