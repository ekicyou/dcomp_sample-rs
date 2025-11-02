use bevy_ecs::prelude::*;
use windows::Win32::Foundation::*;

#[derive(Component, Debug)]
pub struct Window {
    pub hwnd: HWND,
}

unsafe impl Send for Window {}
unsafe impl Sync for Window {}
