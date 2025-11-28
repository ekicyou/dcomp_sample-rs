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
            WM_WINDOWPOSCHANGED => {
                // ウィンドウの位置・サイズが変更された
                if let Some(entity) = get_entity_from_hwnd(hwnd) {
                    if let Some(world) = try_get_ecs_world() {
                        // RefCellが既に借用されている場合はスキップ（ウィンドウ作成中など）
                        if let Ok(mut world_borrow) = world.try_borrow_mut() {
                            let windowpos = lparam.0 as *const WINDOWPOS;
                            if !windowpos.is_null() {
                                let wp = &*windowpos;

                                // WindowPosコンポーネントとWindowHandleを更新
                                if let Ok(mut entity_ref) =
                                    world_borrow.world_mut().get_entity_mut(entity)
                                {
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
                                            // エコーバックチェック（クライアント座標で比較）
                                            if !window_pos.is_echo(client_pos, client_size) {
                                                // エコーバックでない場合のみ更新（ユーザー操作による変更）
                                                // 変更検知を発火させるため、通常の代入を使用
                                                window_pos.position = Some(client_pos);
                                                window_pos.size = Some(client_size);
                                                // last_sentに現在値を設定（apply_window_pos_changesでのSetWindowPos呼び出しを抑制）
                                                // これにより、ユーザー操作による変更はECS内で完結し、Win32への再通知を防ぐ
                                                window_pos.last_sent_position =
                                                    Some((client_pos.x, client_pos.y));
                                                window_pos.last_sent_size =
                                                    Some((client_size.cx, client_size.cy));

                                                eprintln!(
                                                    "[WM_WINDOWPOSCHANGED] Entity {:?}: User operation detected. window=({},{},{},{}) -> client=({},{},{},{})",
                                                    entity, wp.x, wp.y, wp.cx, wp.cy,
                                                    client_pos.x, client_pos.y, client_size.cx, client_size.cy
                                                );
                                            }
                                        }

                                        // BoxStyleがあれば更新（なければスキップ）
                                        if let Some(mut box_style) =
                                            entity_ref.get_mut::<crate::ecs::layout::BoxStyle>()
                                        {
                                            use crate::ecs::layout::{
                                                BoxInset, BoxSize, Dimension, LengthPercentageAuto,
                                                Rect,
                                            };

                                            // サイズを更新
                                            box_style.size = Some(BoxSize {
                                                width: Some(Dimension::Px(client_size.cx as f32)),
                                                height: Some(Dimension::Px(client_size.cy as f32)),
                                            });

                                            // 位置を更新（絶対配置のinset）
                                            box_style.inset = Some(BoxInset(Rect {
                                                left: LengthPercentageAuto::Px(client_pos.x as f32),
                                                top: LengthPercentageAuto::Px(client_pos.y as f32),
                                                right: LengthPercentageAuto::Auto,
                                                bottom: LengthPercentageAuto::Auto,
                                            }));

                                            eprintln!(
                                                "[WM_WINDOWPOSCHANGED] Entity {:?}: BoxStyle updated. size=({},{}), inset=({},{})",
                                                entity, client_size.cx, client_size.cy, client_pos.x, client_pos.y
                                            );
                                        }
                                    }
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
                let dpi_x = (wparam.0 & 0xFFFF) as u16;
                let dpi_y = ((wparam.0 >> 16) & 0xFFFF) as u16;
                let scale_x = dpi_x as f32 / 96.0;
                let scale_y = dpi_y as f32 / 96.0;

                if let Some(entity) = get_entity_from_hwnd(hwnd) {
                    eprintln!(
                        "[WM_DPICHANGED] Entity {:?}: dpi=({}, {}), scale=({:.2}, {:.2})",
                        entity, dpi_x, dpi_y, scale_x, scale_y
                    );
                } else {
                    eprintln!(
                        "[WM_DPICHANGED] hwnd {:?}: dpi=({}, {}), scale=({:.2}, {:.2})",
                        hwnd, dpi_x, dpi_y, scale_x, scale_y
                    );
                }

                // TODO: DPIコンポーネントを更新する処理を追加予定
                // 現在はログ出力のみ

                DefWindowProcW(hwnd, message, wparam, lparam)
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
