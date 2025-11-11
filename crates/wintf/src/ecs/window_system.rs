use bevy_ecs::prelude::*;
use windows::Win32::UI::WindowsAndMessaging::*;

use crate::ecs::{Window, WindowHandle};

/// 未作成のWindowを検出して作成するシステム
pub fn create_windows(
    mut commands: Commands,
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
                    crate::dpi::Dpi::new(x_dpi as f32)
                } else {
                    crate::dpi::Dpi::new(96.0) // デフォルト
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

/// WindowHandleコンポーネントが追加されたときに反応するシステム
pub fn on_window_handle_added(
    query: Query<Entity, Added<WindowHandle>>,
    mut app: ResMut<crate::ecs::app::App>,
) {
    for entity in query.iter() {
        app.on_window_created(entity);
    }
}

/// WindowHandleコンポーネントが削除されたときに反応するシステム
pub fn on_window_handle_removed(
    mut removed: RemovedComponents<WindowHandle>,
    mut app: ResMut<crate::ecs::app::App>,
) {
    for entity in removed.read() {
        app.on_window_destroyed(entity);
    }
}
