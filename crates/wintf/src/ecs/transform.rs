use bevy_ecs::prelude::*;

/// オフセット（追加オフセット）
#[derive(Component, Default, Clone, Copy, Debug, PartialEq)]
pub struct Offset {
    pub x: f32,
    pub y: f32,
}

impl Offset {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// 回転（度数法、UI用なので0/90/180/270が主）
#[derive(Component, Default, Clone, Copy, Debug, PartialEq)]
pub struct Rotation(pub f32);

/// スケール（倍率）
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct Scale {
    pub x: f32,
    pub y: f32,
}

impl Default for Scale {
    fn default() -> Self {
        Self { x: 1.0, y: 1.0 }
    }
}

impl Scale {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn uniform(scale: f32) -> Self {
        Self { x: scale, y: scale }
    }
}
