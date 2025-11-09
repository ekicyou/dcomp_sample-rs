use bevy_ecs::prelude::*;
use windows::Win32::Foundation::*;
use windows_numerics::*;

#[derive(Component, Debug)]
pub struct Window {
    pub hwnd: HWND,
}

unsafe impl Send for Window {}
unsafe impl Sync for Window {}

#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct DpiTransform {
    pub transform: Matrix3x2,
    pub global_transform: Matrix3x2,
}
