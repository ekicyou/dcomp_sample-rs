#![allow(non_snake_case)]

use crate::dpi::Dpi;
use crate::ecs::world::*;
use crate::ecs::*;
use crate::win_state::*;
use crate::WinMessageHandler;
use bevy_ecs::prelude::*;
use std::cell::*;
use std::rc::*;
use windows::Win32::Foundation::*;

#[derive(Debug, Default)]
pub struct EcsWindow {
    world: Rc<RefCell<EcsWorld>>,
    entity: Option<Entity>,

    hwnd: HWND,
    mouse_tracking: bool,
    dpi: Dpi,
}

impl Drop for EcsWindow {
    fn drop(&mut self) {
        if let Some(entity) = self.entity {
            let mut world = self.world.borrow_mut();
            let ecs = world.world_mut();
            ecs.despawn(entity);
        }
    }
}

impl EcsWindow {
    pub fn new(world: Rc<RefCell<EcsWorld>>) -> Self {
        Self {
            world,
            ..Default::default()
        }
    }
}

impl WinState for EcsWindow {
    fn hwnd(&self) -> HWND {
        self.hwnd
    }

    fn set_hwnd(&mut self, hwnd: HWND) {
        self.hwnd = hwnd;
        let mut world = self.world.borrow_mut();
        let ecs = world.world_mut();
        if None == self.entity {
            let entity = ecs.spawn_empty().id();
            self.entity = Some(entity);
        }
        if let Some(entity) = self.entity {
            let mut entity = ecs.entity_mut(entity);
            // WindowHandleコンポーネントを追加
            use crate::process_singleton::WinProcessSingleton;
            let singleton = WinProcessSingleton::get_or_init();
            entity.insert(WindowHandle {
                hwnd,
                instance: singleton.instance(),
                initial_dpi: self.dpi,
            });
        }
    }

    fn mouse_tracking(&self) -> bool {
        self.mouse_tracking
    }

    fn set_mouse_tracking(&mut self, tracking: bool) {
        self.mouse_tracking = tracking;
    }

    fn dpi(&self) -> Dpi {
        self.dpi
    }

    fn set_dpi(&mut self, dpi: Dpi) {
        self.dpi = dpi;
        if let Some(entity) = self.entity {
            let mut world = self.world.borrow_mut();
            let ecs = world.world_mut();
            let mut entity = ecs.entity_mut(entity);
            entity.insert(dpi);
        }
    }
}

impl WinMessageHandler for EcsWindow {
    fn WM_CLOSE(&mut self, _wparam: WPARAM, _lparam: LPARAM) -> Option<LRESULT> {
        unsafe {
            let _ = windows::Win32::UI::WindowsAndMessaging::DestroyWindow(self.hwnd);
        }
        Some(LRESULT(0))
    }

    fn WM_DESTROY(&mut self, _wparam: WPARAM, _lparam: LPARAM) -> Option<LRESULT> {
        self.post_quit_message(0)
    }
}
