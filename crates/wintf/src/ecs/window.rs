use bevy_ecs::prelude::*;
use windows::Win32::Foundation::*;

pub use crate::dpi::Dpi;

#[derive(Component, Debug)]
pub struct Window {
    pub hwnd: HWND,
}

unsafe impl Send for Window {}
unsafe impl Sync for Window {}
