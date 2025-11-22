use bevy_ecs::prelude::*;
use taffy::prelude::*;
use windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F;
use windows_numerics::{Matrix3x2, Vector2};

/// taffyのStyle
#[derive(Component, Debug, Clone, PartialEq, Default)]
pub struct BoxStyle(pub Style);

unsafe impl Send for BoxStyle {}
unsafe impl Sync for BoxStyle {}

impl BoxStyle {
    pub fn new(style: Style) -> Self {
        Self(style)
    }
}

/// レイアウト計算結果
#[derive(Component, Debug, Clone, PartialEq, Default)]
pub struct BoxComputedLayout(pub Layout);

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

/// D2D_RECT_Fの型エイリアス
///
/// Direct2DのD2D_RECT_F構造体をwintfフレームワーク内で使いやすくするための型エイリアスです。
/// UIフレームワーク利用者が`use wintf::ecs::layout::*`だけで完結できるようにしています。
pub type Rect = D2D_RECT_F;

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
/// use wintf::ecs::{Rect, D2DRectExt, Offset, Size};
///
/// let rect = Rect::from_offset_size(
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

/// ローカルレイアウト配置（オフセット + スケール + サイズ）
#[derive(Component, Debug, Clone, Copy, PartialEq)]
#[component(on_add = on_arrangement_add)]
pub struct Arrangement {
    pub offset: Offset,
    pub scale: LayoutScale,
    pub size: Size,
}

impl Arrangement {
    /// ローカル座標系でのバウンディングボックスを返す
    pub fn local_bounds(&self) -> Rect {
        D2D_RECT_F::from_offset_size(self.offset, self.size)
    }
}

impl Default for Arrangement {
    fn default() -> Self {
        Self {
            offset: Offset::default(),
            scale: LayoutScale::default(),
            size: Size::default(),
        }
    }
}

fn on_arrangement_add(
    mut world: bevy_ecs::world::DeferredWorld,
    hook: bevy_ecs::lifecycle::HookContext,
) {
    world
        .commands()
        .entity(hook.entity)
        .insert((GlobalArrangement::default(), ArrangementTreeChanged));
}

/// グローバルレイアウト変換（親からの累積変換とバウンディングボックス）
///
/// ワールド座標系での累積変換行列とバウンディングボックスを保持します。
/// ECS階層システムの`propagate_parent_transforms`により自動的に伝播されます。
///
/// # フィールド
/// - `transform`: 親からの累積変換行列（Matrix3x2）
/// - `bounds`: ワールド座標系でのバウンディングボックス（軸平行矩形）
///
/// # 座標系
/// - ワールド座標系: ルートWindowを基準とした絶対座標
/// - ローカル座標系: 親エンティティを基準とした相対座標
///
/// # Surface生成との関連
/// `bounds`はDirect2D Surfaceの必要サイズを決定する際に使用されます。
/// Surface生成最適化では、子孫の`bounds`を集約して最小限のSurfaceサイズを計算します。
///
/// # 使用例
/// ```
/// use wintf::ecs::{Arrangement, GlobalArrangement};
///
/// let arrangement = Arrangement::default();
/// let global: GlobalArrangement = arrangement.into();
/// // propagate_global_arrangementsシステムにより自動的に更新されます
/// ```
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct GlobalArrangement {
    pub transform: Matrix3x2,
    pub bounds: Rect,
}

impl Default for GlobalArrangement {
    fn default() -> Self {
        Self {
            transform: Matrix3x2::identity(),
            bounds: D2D_RECT_F {
                left: 0.0,
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
            },
        }
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
/// use wintf::ecs::{transform_rect_axis_aligned, Rect};
/// use windows_numerics::Matrix3x2;
///
/// let rect = Rect { left: 0.0, top: 0.0, right: 10.0, bottom: 10.0 };
/// let matrix = Matrix3x2::translation(5.0, 5.0);
/// let transformed = transform_rect_axis_aligned(&rect, &matrix);
/// assert_eq!(transformed.left, 5.0);
/// assert_eq!(transformed.top, 5.0);
/// ```
pub fn transform_rect_axis_aligned(rect: &Rect, matrix: &Matrix3x2) -> Rect {
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

/// Arrangementツリー変更マーカー（ダーティビット伝播用）
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ArrangementTreeChanged;

/// OffsetからMatrix3x2への変換（平行移動）
impl From<Offset> for Matrix3x2 {
    fn from(offset: Offset) -> Self {
        Matrix3x2::translation(offset.x, offset.y)
    }
}

/// LayoutScaleからMatrix3x2への変換（スケール）
impl From<LayoutScale> for Matrix3x2 {
    fn from(scale: LayoutScale) -> Self {
        Matrix3x2::scale(scale.x, scale.y)
    }
}

/// ArrangementからMatrix3x2への変換（スケール + 平行移動）
impl From<Arrangement> for Matrix3x2 {
    fn from(arr: Arrangement) -> Self {
        let scale: Matrix3x2 = arr.scale.into();
        let translation: Matrix3x2 = arr.offset.into();
        scale * translation
    }
}

/// ArrangementからGlobalArrangementへの変換
impl From<Arrangement> for GlobalArrangement {
    fn from(arrangement: Arrangement) -> Self {
        Self {
            transform: arrangement.into(),
            bounds: arrangement.local_bounds(),
        }
    }
}

/// GlobalArrangement同士の乗算（親の累積変換 * 子のローカル変換）
impl std::ops::Mul<Arrangement> for GlobalArrangement {
    type Output = GlobalArrangement;

    fn mul(self, rhs: Arrangement) -> Self::Output {
        // transform計算
        let child_matrix: Matrix3x2 = rhs.into();
        let result_transform = self.transform * child_matrix;

        // bounds計算
        let child_bounds = rhs.local_bounds();
        let result_bounds = transform_rect_axis_aligned(&child_bounds, &result_transform);

        GlobalArrangement {
            transform: result_transform,
            bounds: result_bounds,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_from() {
        let offset = Offset { x: 10.0, y: 20.0 };
        let matrix: Matrix3x2 = offset.into();
        assert_eq!(matrix.M31, 10.0);
        assert_eq!(matrix.M32, 20.0);
    }

    #[test]
    fn test_scale_from() {
        let scale = LayoutScale { x: 2.0, y: 3.0 };
        let matrix: Matrix3x2 = scale.into();
        assert_eq!(matrix.M11, 2.0);
        assert_eq!(matrix.M22, 3.0);
    }

    #[test]
    fn test_arrangement_from() {
        let arr = Arrangement {
            offset: Offset { x: 10.0, y: 20.0 },
            scale: LayoutScale { x: 2.0, y: 3.0 },
            size: Size {
                width: 0.0,
                height: 0.0,
            },
        };

        let matrix: Matrix3x2 = arr.into();
        assert_eq!(matrix.M11, 2.0);
        assert_eq!(matrix.M22, 3.0);
        assert_eq!(matrix.M31, 10.0);
        assert_eq!(matrix.M32, 20.0);
    }

    #[test]
    fn test_global_arrangement_mul() {
        let parent = GlobalArrangement {
            transform: Matrix3x2::translation(10.0, 20.0),
            bounds: D2D_RECT_F {
                left: 0.0,
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
            },
        };

        let child = Arrangement {
            offset: Offset { x: 5.0, y: 7.0 },
            scale: LayoutScale { x: 1.0, y: 1.0 },
            size: Size {
                width: 0.0,
                height: 0.0,
            },
        };

        let result = parent * child;
        assert_eq!(result.transform.M31, 15.0); // 10 + 5
        assert_eq!(result.transform.M32, 27.0); // 20 + 7
    }
}
