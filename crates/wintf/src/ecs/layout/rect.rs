use windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F;
use windows_numerics::{Matrix3x2, Vector2};

use super::{Offset, Size};

/// D2D_RECT_Fの型エイリアス
pub type D2DRect = D2D_RECT_F;

/// D2D_RECT_Fに対する拡張トレイト
///
/// Direct2D APIの`D2D_RECT_F`に便利メソッドを追加するトレイトです。
/// 矩形の構築、取得、設定、判定、演算メソッドを提供します。
///
/// # パフォーマンス特性
/// - すべてのメソッドはO(1)の計算量
/// - インライン化により実質的なオーバーヘッドなし
///
/// # 使用例
/// ```
/// use wintf::ecs::{D2DRect, D2DRectExt, Offset, Size};
///
/// let rect = D2DRect::from_offset_size(
///     Offset { x: 10.0, y: 20.0 },
///     Size { width: 100.0, height: 50.0 }
/// );
/// assert_eq!(rect.width(), 100.0);
/// assert!(rect.contains(50.0, 40.0));
/// ```
pub trait D2DRectExt {
    /// offsetとsizeから矩形を構築
    ///
    /// # パラメータ
    /// - `offset`: 左上座標
    /// - `size`: 幅と高さ
    fn from_offset_size(offset: Offset, size: Size) -> Self;

    /// 幅を取得（right - left）
    fn width(&self) -> f32;

    /// 高さを取得（bottom - top）
    fn height(&self) -> f32;

    /// 左上座標を取得
    fn offset(&self) -> Vector2;

    /// サイズを取得
    fn size(&self) -> Vector2;

    /// 左上座標を設定（幅・高さは維持）
    fn set_offset(&mut self, offset: Vector2);

    /// サイズを設定（左上座標は維持）
    fn set_size(&mut self, size: Vector2);

    /// 左座標を設定
    fn set_left(&mut self, left: f32);

    /// 上座標を設定
    fn set_top(&mut self, top: f32);

    /// 右座標を設定
    fn set_right(&mut self, right: f32);

    /// 下座標を設定
    fn set_bottom(&mut self, bottom: f32);

    /// 点が矩形内に含まれるか判定
    fn contains(&self, x: f32, y: f32) -> bool;

    /// 2つの矩形の最小外接矩形を返す
    fn union(&self, other: &Self) -> Self;

    /// 矩形の一貫性を検証（デバッグビルドのみ）
    #[cfg(debug_assertions)]
    fn validate(&self);
}

impl D2DRectExt for D2D_RECT_F {
    fn from_offset_size(offset: Offset, size: Size) -> Self {
        D2D_RECT_F {
            left: offset.x,
            top: offset.y,
            right: offset.x + size.width,
            bottom: offset.y + size.height,
        }
    }

    fn width(&self) -> f32 {
        self.right - self.left
    }

    fn height(&self) -> f32 {
        self.bottom - self.top
    }

    fn offset(&self) -> Vector2 {
        Vector2 {
            X: self.left,
            Y: self.top,
        }
    }

    fn size(&self) -> Vector2 {
        Vector2 {
            X: self.width(),
            Y: self.height(),
        }
    }

    fn set_offset(&mut self, offset: Vector2) {
        let w = self.width();
        let h = self.height();
        self.left = offset.X;
        self.top = offset.Y;
        self.right = offset.X + w;
        self.bottom = offset.Y + h;
    }

    fn set_size(&mut self, size: Vector2) {
        self.right = self.left + size.X;
        self.bottom = self.top + size.Y;
    }

    fn set_left(&mut self, left: f32) {
        self.left = left;
    }

    fn set_top(&mut self, top: f32) {
        self.top = top;
    }

    fn set_right(&mut self, right: f32) {
        self.right = right;
    }

    fn set_bottom(&mut self, bottom: f32) {
        self.bottom = bottom;
    }

    fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.left && x <= self.right && y >= self.top && y <= self.bottom
    }

    fn union(&self, other: &Self) -> Self {
        D2D_RECT_F {
            left: self.left.min(other.left),
            top: self.top.min(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }

    #[cfg(debug_assertions)]
    fn validate(&self) {
        debug_assert!(self.left <= self.right, "Invalid rect: left > right");
        debug_assert!(self.top <= self.bottom, "Invalid rect: top > bottom");
    }
}

/// 軸平行変換専用の矩形変換（2点変換）
///
/// 平行移動とスケールのみを含む軸平行変換に最適化された矩形変換関数です。
/// 4点すべてを変換する代わりに、左上と右下の2点のみを変換します。
///
/// # パラメータ
/// - `rect`: 変換対象の矩形（ローカル座標系）
/// - `matrix`: 変換行列（Matrix3x2、軸平行変換を想定）
///
/// # 戻り値
/// 変換後の軸平行矩形（ワールド座標系）
///
/// # パフォーマンス
/// - 2点変換: O(2) の計算量（通常の4点変換はO(4)）
/// - 軸平行変換の場合、左上と右下の2点だけで完全な矩形を再構築可能
///
/// # 制約
/// - **軸平行変換のみサポート**: 回転・スキュー変換には対応していません
/// - 回転・スキュー変換が含まれる場合、正しくない結果を返す可能性があります
/// - 将来的にDirectComposition Visual層で回転・スキュー変換をサポート予定
///
/// # 使用例
/// ```
/// use wintf::ecs::{transform_rect_axis_aligned, D2DRect};
/// use windows_numerics::Matrix3x2;
///
/// let rect = D2DRect { left: 0.0, top: 0.0, right: 10.0, bottom: 10.0 };
/// let matrix = Matrix3x2::translation(5.0, 5.0);
/// let transformed = transform_rect_axis_aligned(&rect, &matrix);
/// assert_eq!(transformed.left, 5.0);
/// assert_eq!(transformed.top, 5.0);
/// ```
pub fn transform_rect_axis_aligned(rect: &D2DRect, matrix: &Matrix3x2) -> D2DRect {
    // 左上と右下の2点を変換
    // Matrix3x2での点変換: x' = M11*x + M21*y + M31, y' = M12*x + M22*y + M32
    let top_left_x = matrix.M11 * rect.left + matrix.M21 * rect.top + matrix.M31;
    let top_left_y = matrix.M12 * rect.left + matrix.M22 * rect.top + matrix.M32;

    let bottom_right_x = matrix.M11 * rect.right + matrix.M21 * rect.bottom + matrix.M31;
    let bottom_right_y = matrix.M12 * rect.right + matrix.M22 * rect.bottom + matrix.M32;

    // min/maxで新しい軸平行矩形を構築
    D2D_RECT_F {
        left: top_left_x.min(bottom_right_x),
        top: top_left_y.min(bottom_right_y),
        right: top_left_x.max(bottom_right_x),
        bottom: top_left_y.max(bottom_right_y),
    }
}
