use bevy_ecs::prelude::*;

/// テキストレイアウトの物理サイズ
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct TextLayoutMetrics {
    pub width: f32,  // Physical width (pixels)
    pub height: f32, // Physical height (pixels)
}

/// レイアウトサイズ（幅と高さ）
///
/// レイアウト計算後の確定サイズを保持する値オブジェクトです。
/// 将来的にtaffyレイアウトエンジンによって自動設定されます。
///
/// # フィールド
/// - `width`: 幅（ピクセル単位、物理ピクセル）
/// - `height`: 高さ（ピクセル単位、物理ピクセル）
///
/// # 使用例
/// ```
/// use wintf::ecs::{Size};
///
/// let size = Size { width: 100.0, height: 50.0 };
/// assert_eq!(size.width, 100.0);
/// ```
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct Size {
    pub width: f32,  // 幅（ピクセル単位）
    pub height: f32, // 高さ（ピクセル単位）
}

impl Size {
    /// サイズのバリデーション（負の値チェック）
    ///
    /// 負の値が検出された場合は警告ログを出力します。
    /// これはmayレベルの推奨事項であり、実行時エラーとはしません。
    pub fn validate(&self) {
        if self.width < 0.0 {
            eprintln!("Warning: Size.width is negative: {}", self.width);
        }
        if self.height < 0.0 {
            eprintln!("Warning: Size.height is negative: {}", self.height);
        }
    }
}

/// オフセット（親からの相対位置）
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Offset {
    pub x: f32,
    pub y: f32,
}

impl Default for Offset {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// レイアウトスケール（DPIスケール、ViewBox等）
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct LayoutScale {
    pub x: f32,
    pub y: f32,
}

impl LayoutScale {
    /// スケールのバリデーション（ゼロ値チェック）
    ///
    /// ゼロ値が検出された場合は警告ログを出力します。
    /// これはmayレベルの推奨事項であり、実行時エラーとはしません。
    pub fn validate(&self) {
        if self.x == 0.0 {
            eprintln!("Warning: LayoutScale.x is zero, which may cause layout issues");
        }
        if self.y == 0.0 {
            eprintln!("Warning: LayoutScale.y is zero, which may cause layout issues");
        }
    }
}

impl Default for LayoutScale {
    fn default() -> Self {
        Self { x: 1.0, y: 1.0 }
    }
}
