use bevy_ecs::prelude::*;
use tracing::warn;

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
            warn!(width = self.width, "Size.width is negative");
        }
        if self.height < 0.0 {
            warn!(height = self.height, "Size.height is negative");
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
            warn!("LayoutScale.x is zero, which may cause layout issues");
        }
        if self.y == 0.0 {
            warn!("LayoutScale.y is zero, which may cause layout issues");
        }
    }
}

impl Default for LayoutScale {
    fn default() -> Self {
        Self { x: 1.0, y: 1.0 }
    }
}

/// 不透明度（Visualの透明度制御）
///
/// DirectCompositionのVisualに対して不透明度を設定するためのコンポーネントです。
/// 値は0.0（完全透明）から1.0（完全不透明）の範囲で指定します。
///
/// # フィールド
/// - `0`: 不透明度（0.0 = 完全透明, 1.0 = 完全不透明）
///
/// # 使用例
/// ```
/// use wintf::ecs::layout::Opacity;
///
/// let opacity = Opacity(0.5);  // 50%の不透明度
/// let fully_opaque = Opacity::default();  // 1.0（完全不透明）
/// ```
///
/// # 設計意図
/// - UIレベルのパラメータとしてlayoutモジュールに配置
/// - 内部的にはgraphicsシステムがVisualGraphicsに反映
/// - アニメーション対象として将来的に拡張可能
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Opacity(pub f32);

impl Opacity {
    /// 不透明度のバリデーション（範囲チェック）
    ///
    /// 範囲外の値が検出された場合は警告ログを出力します。
    pub fn validate(&self) {
        if self.0 < 0.0 || self.0 > 1.0 {
            warn!(
                value = self.0,
                "Opacity value is outside valid range [0.0, 1.0]"
            );
        }
    }

    /// クランプされた値を取得（0.0〜1.0の範囲に制限）
    pub fn clamped(&self) -> f32 {
        self.0.clamp(0.0, 1.0)
    }
}

impl Default for Opacity {
    fn default() -> Self {
        Self(1.0) // 完全不透明がデフォルト
    }
}
