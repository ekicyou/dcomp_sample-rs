use bevy_ecs::prelude::*;
use windows_numerics::Matrix3x2;

/// 平行移動（CSS transform: translate に相当）
#[derive(Component, Default, Clone, Copy, Debug, PartialEq)]
pub struct Translate {
    pub x: f32,
    pub y: f32,
}

impl Translate {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl From<Translate> for Matrix3x2 {
    fn from(t: Translate) -> Self {
        Matrix3x2::translation(t.x, t.y)
    }
}

/// スケール（CSS transform: scale に相当）
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

    pub fn from_dpi(x_dpi: f32, y_dpi: f32) -> Self {
        Self {
            x: x_dpi / 96.0,
            y: y_dpi / 96.0,
        }
    }
}

impl From<Scale> for Matrix3x2 {
    fn from(s: Scale) -> Self {
        Matrix3x2 {
            M11: s.x,
            M12: 0.0,
            M21: 0.0,
            M22: s.y,
            M31: 0.0,
            M32: 0.0,
        }
    }
}

/// 回転（CSS transform: rotate に相当）
/// 角度は度数法で指定（UI用なので0/90/180/270が主）
#[derive(Component, Default, Clone, Copy, Debug, PartialEq)]
pub struct Rotate(pub f32);

impl From<Rotate> for Matrix3x2 {
    fn from(r: Rotate) -> Self {
        let radians = r.0.to_radians();
        let cos = radians.cos();
        let sin = radians.sin();
        Matrix3x2 {
            M11: cos,
            M12: sin,
            M21: -sin,
            M22: cos,
            M31: 0.0,
            M32: 0.0,
        }
    }
}

/// 傾斜変換（CSS transform: skew に相当）
#[derive(Component, Default, Clone, Copy, Debug, PartialEq)]
pub struct Skew {
    pub x: f32,
    pub y: f32,
}

impl Skew {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl From<Skew> for Matrix3x2 {
    fn from(s: Skew) -> Self {
        let tan_x = s.x.to_radians().tan();
        let tan_y = s.y.to_radians().tan();
        Matrix3x2 {
            M11: 1.0,
            M12: tan_y,
            M21: tan_x,
            M22: 1.0,
            M31: 0.0,
            M32: 0.0,
        }
    }
}

/// 変換の基準点（CSS transform-origin に相当）
/// デフォルトは中心(0.5, 0.5)
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct TransformOrigin {
    pub x: f32,
    pub y: f32,
}

impl Default for TransformOrigin {
    fn default() -> Self {
        Self { x: 0.5, y: 0.5 }
    }
}

impl TransformOrigin {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn center() -> Self {
        Self { x: 0.5, y: 0.5 }
    }

    pub fn top_left() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}
