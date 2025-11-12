use bevy_ecs::prelude::*;
use windows::Win32::Foundation::{POINT, SIZE};

// ===== 基本トランスフォーム =====

/// ローカル座標（親エンティティに対する相対位置）
#[derive(Component, Default, Clone, Copy, Debug, PartialEq)]
pub struct LocalPosition {
    pub x: f32,
    pub y: f32,
}

/// ローカルサイズ
#[derive(Component, Default, Clone, Copy, Debug, PartialEq)]
pub struct LocalSize {
    pub width: f32,
    pub height: f32,
}

/// ローカルスケール（倍率）
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct LocalScale {
    pub x: f32,
    pub y: f32,
}

impl Default for LocalScale {
    fn default() -> Self {
        Self { x: 1.0, y: 1.0 }
    }
}

/// ローカル回転（度数法、UI用なので0/90/180/270が主）
#[derive(Component, Default, Clone, Copy, Debug, PartialEq)]
pub struct LocalRotation(pub f32);

/// オフセット（LocalPositionに加算される追加オフセット）
#[derive(Component, Default, Clone, Copy, Debug, PartialEq)]
pub struct Offset {
    pub x: f32,
    pub y: f32,
}

// ===== グローバルトランスフォーム（計算結果） =====

/// グローバル座標（スクリーン座標系での絶対位置）
#[derive(Component, Default, Clone, Copy, Debug, PartialEq)]
pub struct GlobalPosition {
    pub x: f32,
    pub y: f32,
}

/// グローバルサイズ（スケール適用後のサイズ）
#[derive(Component, Default, Clone, Copy, Debug, PartialEq)]
pub struct GlobalSize {
    pub width: f32,
    pub height: f32,
}

/// グローバルスケール（親のスケールを乗算した結果）
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct GlobalScale {
    pub x: f32,
    pub y: f32,
}

impl Default for GlobalScale {
    fn default() -> Self {
        Self { x: 1.0, y: 1.0 }
    }
}

/// グローバル回転（親の回転を加算した結果）
#[derive(Component, Default, Clone, Copy, Debug, PartialEq)]
pub struct GlobalRotation(pub f32);

// ===== DPIスケーリング =====

/// DPI（dots per inch）
#[derive(Component, Default, Clone, Copy, Debug, PartialEq)]
pub struct Dpi(pub f32);

impl Dpi {
    pub const DEFAULT: f32 = 96.0;

    pub fn scale_factor(&self) -> f32 {
        self.0 / Self::DEFAULT
    }

    // POINT/SIZE変換用ヘルパー（整数型との相互変換）
    pub fn scale_point(&self, pt: POINT) -> POINT {
        let factor = self.scale_factor();
        POINT {
            x: (pt.x as f32 * factor) as i32,
            y: (pt.y as f32 * factor) as i32,
        }
    }

    pub fn scale_size(&self, sz: SIZE) -> SIZE {
        let factor = self.scale_factor();
        SIZE {
            cx: (sz.cx as f32 * factor) as i32,
            cy: (sz.cy as f32 * factor) as i32,
        }
    }

    pub fn unscale_point(&self, pt: POINT) -> POINT {
        let factor = self.scale_factor();
        POINT {
            x: (pt.x as f32 / factor) as i32,
            y: (pt.y as f32 / factor) as i32,
        }
    }

    pub fn unscale_size(&self, sz: SIZE) -> SIZE {
        let factor = self.scale_factor();
        SIZE {
            cx: (sz.cx as f32 / factor) as i32,
            cy: (sz.cy as f32 / factor) as i32,
        }
    }
}

// ===== ユーティリティ =====

impl LocalPosition {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn from_point(pt: POINT) -> Self {
        Self { x: pt.x as f32, y: pt.y as f32 }
    }

    pub fn to_point(&self) -> POINT {
        POINT { x: self.x as i32, y: self.y as i32 }
    }
}

impl LocalSize {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn from_size(sz: SIZE) -> Self {
        Self { width: sz.cx as f32, height: sz.cy as f32 }
    }

    pub fn to_size(&self) -> SIZE {
        SIZE { cx: self.width as i32, cy: self.height as i32 }
    }
}

impl LocalScale {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn uniform(scale: f32) -> Self {
        Self { x: scale, y: scale }
    }
}

impl GlobalPosition {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn from_point(pt: POINT) -> Self {
        Self { x: pt.x as f32, y: pt.y as f32 }
    }

    pub fn to_point(&self) -> POINT {
        POINT { x: self.x as i32, y: self.y as i32 }
    }
}

impl GlobalSize {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }

    pub fn from_size(sz: SIZE) -> Self {
        Self { width: sz.cx as f32, height: sz.cy as f32 }
    }

    pub fn to_size(&self) -> SIZE {
        SIZE { cx: self.width as i32, cy: self.height as i32 }
    }
}

impl GlobalScale {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub fn uniform(scale: f32) -> Self {
        Self { x: scale, y: scale }
    }
}
