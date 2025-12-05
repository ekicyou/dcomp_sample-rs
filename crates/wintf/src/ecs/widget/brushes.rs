//! Brush/Brushes コンポーネント
//!
//! ウィジェットの色・ブラシ管理を統一するECSコンポーネント。
//! 親からの継承（Inherit）と単色（Solid）をサポート。

use bevy_ecs::prelude::*;
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;

// ============================================================
// Brush enum
// ============================================================

/// ブラシ値
///
/// 継承マーカー（Inherit）と単色（Solid）の2バリアントを持つ。
/// 将来的にグラデーションブラシへの拡張が可能な設計。
#[derive(Clone, Debug, PartialEq)]
pub enum Brush {
    /// 親から継承（描画前に解決される）
    Inherit,
    /// 単色
    Solid(D2D1_COLOR_F),
}

impl Brush {
    // === 色定数 ===

    /// 透明色
    pub const TRANSPARENT: Self = Self::Solid(D2D1_COLOR_F {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 0.0,
    });
    /// 黒
    pub const BLACK: Self = Self::Solid(D2D1_COLOR_F {
        r: 0.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    });
    /// 白
    pub const WHITE: Self = Self::Solid(D2D1_COLOR_F {
        r: 1.0,
        g: 1.0,
        b: 1.0,
        a: 1.0,
    });
    /// 赤
    pub const RED: Self = Self::Solid(D2D1_COLOR_F {
        r: 1.0,
        g: 0.0,
        b: 0.0,
        a: 1.0,
    });
    /// 緑
    pub const GREEN: Self = Self::Solid(D2D1_COLOR_F {
        r: 0.0,
        g: 1.0,
        b: 0.0,
        a: 1.0,
    });
    /// 青
    pub const BLUE: Self = Self::Solid(D2D1_COLOR_F {
        r: 0.0,
        g: 0.0,
        b: 1.0,
        a: 1.0,
    });

    /// D2D1_COLOR_Fを取得（Inheritの場合はNone）
    pub fn as_color(&self) -> Option<D2D1_COLOR_F> {
        match self {
            Brush::Inherit => None,
            Brush::Solid(color) => Some(*color),
        }
    }

    /// Inheritかどうか
    pub fn is_inherit(&self) -> bool {
        matches!(self, Brush::Inherit)
    }
}

impl Default for Brush {
    fn default() -> Self {
        Brush::Inherit
    }
}

// ============================================================
// Brushes コンポーネント
// ============================================================

/// Brushesコンポーネント
///
/// foreground/backgroundブラシを保持するECSコンポーネント。
/// デフォルト値は両方ともBrush::Inherit。
#[derive(Component, Clone, Debug, PartialEq)]
#[component(storage = "SparseSet")]
pub struct Brushes {
    /// 前景色（テキスト、図形塗りつぶし）
    pub foreground: Brush,
    /// 背景色
    pub background: Brush,
}

impl Default for Brushes {
    fn default() -> Self {
        Self {
            foreground: Brush::Inherit,
            background: Brush::Inherit,
        }
    }
}

impl Brushes {
    /// 前景色を指定して作成（背景はInherit）
    pub fn with_foreground(color: D2D1_COLOR_F) -> Self {
        Self {
            foreground: Brush::Solid(color),
            background: Brush::Inherit,
        }
    }

    /// 背景色を指定して作成（前景はInherit）
    pub fn with_background(color: D2D1_COLOR_F) -> Self {
        Self {
            foreground: Brush::Inherit,
            background: Brush::Solid(color),
        }
    }

    /// 前景色・背景色を指定して作成
    pub fn with_colors(foreground: D2D1_COLOR_F, background: D2D1_COLOR_F) -> Self {
        Self {
            foreground: Brush::Solid(foreground),
            background: Brush::Solid(background),
        }
    }
}

// ============================================================
// BrushInherit マーカーコンポーネント
// ============================================================

/// 未解決状態を示すマーカーコンポーネント
///
/// Visual on_addで自動挿入され、resolve_inherited_brushesシステムで解決後に除去される。
/// SparseSetストレージ: 一時的マーカーに最適（挿入/削除O(1)、archetype変更なし）
#[derive(Component, Default, Debug, Clone, Copy, PartialEq, Eq)]
#[component(storage = "SparseSet")]
pub struct BrushInherit;

// ============================================================
// デフォルト色定数
// ============================================================

/// デフォルト前景色（黒）
pub const DEFAULT_FOREGROUND: Brush = Brush::BLACK;
/// デフォルト背景色（透明）
pub const DEFAULT_BACKGROUND: Brush = Brush::TRANSPARENT;

// ============================================================
// Tests
// ============================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brush_as_color_inherit() {
        let brush = Brush::Inherit;
        assert!(brush.as_color().is_none());
    }

    #[test]
    fn test_brush_as_color_solid() {
        let brush = Brush::Solid(D2D1_COLOR_F {
            r: 1.0,
            g: 0.5,
            b: 0.25,
            a: 0.75,
        });
        let color = brush.as_color().unwrap();
        assert!((color.r - 1.0).abs() < f32::EPSILON);
        assert!((color.g - 0.5).abs() < f32::EPSILON);
        assert!((color.b - 0.25).abs() < f32::EPSILON);
        assert!((color.a - 0.75).abs() < f32::EPSILON);
    }

    #[test]
    fn test_brush_is_inherit() {
        assert!(Brush::Inherit.is_inherit());
        assert!(!Brush::BLACK.is_inherit());
        assert!(!Brush::TRANSPARENT.is_inherit());
    }

    #[test]
    fn test_brush_constants() {
        // TRANSPARENT
        let color = Brush::TRANSPARENT.as_color().unwrap();
        assert!((color.a - 0.0).abs() < f32::EPSILON);

        // BLACK
        let color = Brush::BLACK.as_color().unwrap();
        assert!((color.r - 0.0).abs() < f32::EPSILON);
        assert!((color.g - 0.0).abs() < f32::EPSILON);
        assert!((color.b - 0.0).abs() < f32::EPSILON);
        assert!((color.a - 1.0).abs() < f32::EPSILON);

        // WHITE
        let color = Brush::WHITE.as_color().unwrap();
        assert!((color.r - 1.0).abs() < f32::EPSILON);
        assert!((color.g - 1.0).abs() < f32::EPSILON);
        assert!((color.b - 1.0).abs() < f32::EPSILON);

        // RED
        let color = Brush::RED.as_color().unwrap();
        assert!((color.r - 1.0).abs() < f32::EPSILON);
        assert!((color.g - 0.0).abs() < f32::EPSILON);
        assert!((color.b - 0.0).abs() < f32::EPSILON);

        // GREEN
        let color = Brush::GREEN.as_color().unwrap();
        assert!((color.r - 0.0).abs() < f32::EPSILON);
        assert!((color.g - 1.0).abs() < f32::EPSILON);
        assert!((color.b - 0.0).abs() < f32::EPSILON);

        // BLUE
        let color = Brush::BLUE.as_color().unwrap();
        assert!((color.r - 0.0).abs() < f32::EPSILON);
        assert!((color.g - 0.0).abs() < f32::EPSILON);
        assert!((color.b - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn test_brushes_default() {
        let brushes = Brushes::default();
        assert!(brushes.foreground.is_inherit());
        assert!(brushes.background.is_inherit());
    }

    #[test]
    fn test_brushes_with_foreground() {
        let color = D2D1_COLOR_F {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let brushes = Brushes::with_foreground(color);
        assert_eq!(brushes.foreground, Brush::Solid(color));
        assert!(brushes.background.is_inherit());
    }

    #[test]
    fn test_brushes_with_background() {
        let color = D2D1_COLOR_F {
            r: 0.0,
            g: 1.0,
            b: 0.0,
            a: 1.0,
        };
        let brushes = Brushes::with_background(color);
        assert!(brushes.foreground.is_inherit());
        assert_eq!(brushes.background, Brush::Solid(color));
    }

    #[test]
    fn test_brushes_with_colors() {
        let fg = D2D1_COLOR_F {
            r: 1.0,
            g: 0.0,
            b: 0.0,
            a: 1.0,
        };
        let bg = D2D1_COLOR_F {
            r: 0.0,
            g: 0.0,
            b: 1.0,
            a: 0.5,
        };
        let brushes = Brushes::with_colors(fg, bg);
        assert_eq!(brushes.foreground, Brush::Solid(fg));
        assert_eq!(brushes.background, Brush::Solid(bg));
    }
}
