use crate::ecs::*;
use crate::process_singleton::*;
use bevy_ecs::prelude::*;
use windows::core::*;
use windows::Win32::Foundation::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows_numerics::Vector2;

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
                // 実際のクライアント領域のサイズを取得
                let mut rect = RECT::default();
                unsafe {
                    let _ = GetClientRect(hwnd, &mut rect);
                }
                let width = (rect.right - rect.left) as f32;
                let height = (rect.bottom - rect.top) as f32;

                // WindowHandleコンポーネントを追加
                commands.entity(entity).insert((
                    WindowHandle {
                        hwnd,
                        instance: singleton.instance(),
                    },
                    crate::ecs::graphics::HasGraphicsResources,
                    crate::ecs::graphics::Visual {
                        size: Vector2 {
                            X: width,
                            Y: height,
                        },
                        ..Default::default()
                    },
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
