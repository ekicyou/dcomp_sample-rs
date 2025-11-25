use crate::ecs::*;
use crate::process_singleton::*;
use bevy_ecs::prelude::*;
use windows::core::*;
use windows::Win32::UI::HiDpi::GetDpiForSystem;
use windows::Win32::UI::WindowsAndMessaging::*;

/// Window EntityにArrangementコンポーネントを自動追加するシステム
pub fn init_window_arrangement(
    mut commands: Commands,
    query: Query<Entity, (With<Window>, Without<Arrangement>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert((
            Arrangement::default(),
            GlobalArrangement::default(),
            ArrangementTreeChanged,
        ));
    }
}

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

        // システムDPIを取得（ウィンドウ作成前なのでGetDpiForSystemを使用）
        let system_dpi = unsafe { GetDpiForSystem() };

        // クライアント領域座標をウィンドウ全体座標に変換
        let (x, y, width, height) = pos_comp.to_window_coords_for_creation(
            style_comp.style,
            style_comp.ex_style,
            system_dpi,
        );

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
                // WindowHandleコンポーネントを追加
                // Visual.sizeは sync_visual_from_layout_root で設定される
                // Visualコンポーネントが既に存在する場合は上書きしない
                commands.entity(entity).insert((
                    WindowHandle {
                        hwnd,
                        instance: singleton.instance(),
                    },
                    crate::ecs::graphics::HasGraphicsResources,
                ));

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
