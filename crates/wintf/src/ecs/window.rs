use bevy_ecs::prelude::*;
use windows::Win32::Foundation::*;

pub use crate::dpi::Dpi;
use crate::{RawPoint, RawSize};

#[derive(Component, Debug)]
pub struct Window {
    pub hwnd: HWND,
}

unsafe impl Send for Window {}
unsafe impl Sync for Window {}

#[derive(Component, Debug)]
pub struct WindowPos {
    pub position: Option<RawPoint>,
    pub size: Option<RawSize>,
}
