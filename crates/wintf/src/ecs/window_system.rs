use bevy_ecs::prelude::*;
use windows::core::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::HiDpi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::ecs::*;
use crate::process_singleton::*;

/// 未作成のWindowを検出して作成するシステム
pub fn create_windows(
    mut commands: Commands,
    query: Query<
        (Entity, &Window, Option<&WindowStyle>, Option<&WindowPos>),
        Without<WindowHandle>,
    >,
) {
    let singleton = WinProcessSingleton::get_or_init();

    for (entity, window, opt_style, opt_pos) in query.iter() {
        // Win32ウィンドウを作成
        let title = HSTRING::from(&window.title);

        // WindowStyleが指定されていなければデフォルトを使用
        let style_comp = opt_style.copied().unwrap_or_default();

        // WindowPosが指定されていなければデフォルトを使用
        let pos_comp = opt_pos.copied().unwrap_or_default();

        // 位置とサイズを取得
        let (x, y) = if let Some(pos) = pos_comp.position {
            (pos.x, pos.y)
        } else {
            (CW_USEDEFAULT, CW_USEDEFAULT)
        };

        let (width, height) = if let Some(size) = pos_comp.size {
            (size.cx, size.cy)
        } else {
            (CW_USEDEFAULT, CW_USEDEFAULT)
        };

        // EntityのIDをlpCreateParamsとして渡す
        let entity_bits = entity.to_bits() as *mut std::ffi::c_void;

        let result = unsafe {
            CreateWindowExW(
                style_comp.ex_style,
                singleton.ecs_window_class_name(), // ECS用のウィンドウクラスを使用
                &title,
                style_comp.style,
                x,
                y,
                width,
                height,
                window.parent,
                None,
                Some(singleton.instance()),
                Some(entity_bits), // EntityのIDを渡す
            )
        };

        match result {
            Ok(hwnd) => {
                // 初期DPIを取得
                let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
                let mut x_dpi = 0u32;
                let mut y_dpi = 0u32;
                let dpi_result =
                    unsafe { GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut x_dpi, &mut y_dpi) };

                let initial_dpi = if dpi_result.is_ok() {
                    x_dpi as f32
                } else {
                    96.0 // デフォルト
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

                eprintln!("Window created: hwnd={:?}, entity={:?}", hwnd, entity);
            }
            Err(e) => {
                eprintln!("Failed to create window for entity {:?}: {:?}", entity, e);
            }
        }
    }
}


