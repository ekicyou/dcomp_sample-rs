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

/// Z順序（描画順序、大きいほど手前）
#[derive(Component, Default, Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ZOrder(pub i32);

// ===== オフセット・マージン =====

/// オフセット（LocalPositionに加算される追加オフセット）
#[derive(Component, Default, Clone, Copy, Debug, PartialEq)]
pub struct Offset {
    pub x: f32,
    pub y: f32,
}

/// マージン（レイアウト計算時の外側余白）
#[derive(Component, Default, Clone, Copy, Debug, PartialEq)]
pub struct Margin {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Margin {
    pub fn uniform(value: i32) -> Self {
        Self {
            left: value,
            top: value,
            right: value,
            bottom: value,
        }
    }

    pub fn symmetric(horizontal: i32, vertical: i32) -> Self {
        Self {
            left: horizontal,
            top: vertical,
            right: horizontal,
            bottom: vertical,
        }
    }
}

/// パディング（レイアウト計算時の内側余白）
#[derive(Component, Default, Clone, Copy, Debug, PartialEq)]
pub struct Padding {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Padding {
    pub fn uniform(value: i32) -> Self {
        Self {
            left: value,
            top: value,
            right: value,
            bottom: value,
        }
    }

    pub fn symmetric(horizontal: i32, vertical: i32) -> Self {
        Self {
            left: horizontal,
            top: vertical,
            right: horizontal,
            bottom: vertical,
        }
    }
}

// ===== アンカー・ピボット =====

/// アンカー（親に対する配置基準点、0.0～1.0）
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct Anchor {
    pub x: f32,
    pub y: f32,
}

impl Default for Anchor {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 } // 左上
    }
}

impl Anchor {
    pub const TOP_LEFT: Self = Self { x: 0.0, y: 0.0 };
    pub const TOP_CENTER: Self = Self { x: 0.5, y: 0.0 };
    pub const TOP_RIGHT: Self = Self { x: 1.0, y: 0.0 };
    pub const CENTER_LEFT: Self = Self { x: 0.0, y: 0.5 };
    pub const CENTER: Self = Self { x: 0.5, y: 0.5 };
    pub const CENTER_RIGHT: Self = Self { x: 1.0, y: 0.5 };
    pub const BOTTOM_LEFT: Self = Self { x: 0.0, y: 1.0 };
    pub const BOTTOM_CENTER: Self = Self { x: 0.5, y: 1.0 };
    pub const BOTTOM_RIGHT: Self = Self { x: 1.0, y: 1.0 };
}

/// ピボット（自身の回転・スケールの中心点、0.0～1.0）
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct Pivot {
    pub x: f32,
    pub y: f32,
}

impl Default for Pivot {
    fn default() -> Self {
        Self { x: 0.5, y: 0.5 } // 中心
    }
}

impl Pivot {
    pub const TOP_LEFT: Self = Self { x: 0.0, y: 0.0 };
    pub const TOP_CENTER: Self = Self { x: 0.5, y: 0.0 };
    pub const TOP_RIGHT: Self = Self { x: 1.0, y: 0.0 };
    pub const CENTER_LEFT: Self = Self { x: 0.0, y: 0.5 };
    pub const CENTER: Self = Self { x: 0.5, y: 0.5 };
    pub const CENTER_RIGHT: Self = Self { x: 1.0, y: 0.5 };
    pub const BOTTOM_LEFT: Self = Self { x: 0.0, y: 1.0 };
    pub const BOTTOM_CENTER: Self = Self { x: 0.5, y: 1.0 };
    pub const BOTTOM_RIGHT: Self = Self { x: 1.0, y: 1.0 };
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

// ===== 変更追跡 =====

/// トランスフォーム変更マーカー（再計算が必要）
#[derive(Component, Default, Clone, Copy, Debug)]
pub struct TransformDirty;

// ===== 階層構造 =====

/// 親エンティティ
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Parent(pub Entity);

/// 子エンティティリスト（動的に管理される）
#[derive(Component, Default, Clone, Debug)]
pub struct Children(pub Vec<Entity>);

impl Children {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, child: Entity) {
        if !self.0.contains(&child) {
            self.0.push(child);
        }
    }

    pub fn remove(&mut self, child: Entity) {
        self.0.retain(|&e| e != child);
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn iter(&self) -> impl Iterator<Item = &Entity> {
        self.0.iter()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

// ===== 可視性 =====

/// ローカル可視性（このエンティティ自身の可視性）
#[derive(Component, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Visible(pub bool);

/// グローバル可視性（親の可視性を考慮した結果）
#[derive(Component, Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlobalVisible(pub bool);

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

impl Visible {
    pub fn visible() -> Self {
        Self(true)
    }

    pub fn hidden() -> Self {
        Self(false)
    }
}

impl GlobalVisible {
    pub fn visible() -> Self {
        Self(true)
    }

    pub fn hidden() -> Self {
        Self(false)
    }
}
