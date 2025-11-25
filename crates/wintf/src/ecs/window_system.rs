use crate::ecs::*;
use crate::process_singleton::*;
use bevy_ecs::prelude::*;
use bevy_ecs::system::SystemState;
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

/// 未作成のWindowを検出して作成する排他システム
///
/// 排他システムにすることで、WindowHandleの追加が即時反映され、
/// 同じフレーム内の後続スケジュールでWindowHandleが参照可能になる。
pub fn create_windows(world: &mut World) {
    // SystemStateを使ってクエリとリソースにアクセス
    let mut system_state: SystemState<(
        Query<(Entity, &Window, Option<&WindowStyle>, Option<&WindowPos>), Without<WindowHandle>>,
        Res<crate::ecs::world::FrameCount>,
    )> = SystemState::new(world);

    // クエリ結果を先に収集（borrowの問題を回避）
    let (query, frame_count) = system_state.get(world);
    let frame = frame_count.0;
    let entities_to_create: Vec<_> = query
        .iter()
        .map(|(entity, window, opt_style, opt_pos)| {
            (
                entity,
                window.title.clone(),
                window.parent,
                opt_style.copied(),
                opt_pos.copied(),
            )
        })
        .collect();

    // 収集したエンティティに対してウィンドウを作成
    let singleton = WinProcessSingleton::get_or_init();

    for (entity, title, parent, opt_style, opt_pos) in entities_to_create {
        eprintln!(
            "[Frame {}] [create_windows] ウィンドウ作成開始 (Entity: {:?}, title: {})",
            frame, entity, title
        );

        let title_hstring = HSTRING::from(&title);
        let style_comp = opt_style.unwrap_or_default();
        let pos_comp = opt_pos.unwrap_or_default();
        let system_dpi = unsafe { GetDpiForSystem() };

        let (x, y, width, height) = pos_comp.to_window_coords_for_creation(
            style_comp.style,
            style_comp.ex_style,
            system_dpi,
        );

        let entity_bits = entity.to_bits() as *mut std::ffi::c_void;

        let result = unsafe {
            CreateWindowExW(
                style_comp.ex_style,
                singleton.ecs_window_class_name(),
                &title_hstring,
                style_comp.style,
                x,
                y,
                width,
                height,
                parent,
                None,
                Some(singleton.instance()),
                Some(entity_bits),
            )
        };

        match result {
            Ok(hwnd) => {
                eprintln!(
                    "[Frame {}] [create_windows] HWND作成成功 (Entity: {:?}, hwnd: {:?})",
                    frame, entity, hwnd
                );

                // 即時にWindowHandleを追加（排他システムなので即時反映）
                world.entity_mut(entity).insert((
                    WindowHandle {
                        hwnd,
                        instance: singleton.instance(),
                    },
                    crate::ecs::graphics::HasGraphicsResources,
                ));

                eprintln!(
                    "[Frame {}] [create_windows] WindowHandle即時追加完了 (Entity: {:?})",
                    frame, entity
                );

                unsafe {
                    let _ = ShowWindow(hwnd, SW_SHOW);
                }

                eprintln!(
                    "[Frame {}] [create_windows] ShowWindow完了 (Entity: {:?})",
                    frame, entity
                );
            }
            Err(e) => {
                eprintln!(
                    "[Frame {}] [create_windows] エラー: Entity {:?}, {:?}",
                    frame, entity, e
                );
            }
        }
    }
}
