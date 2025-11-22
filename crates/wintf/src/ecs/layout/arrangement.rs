use bevy_ecs::prelude::*;
use windows_numerics::Matrix3x2;

use super::{transform_rect_axis_aligned, D2DRectExt, LayoutScale, Offset, Rect, Size};

/// ローカルレイアウト配置（親からの相対位置とサイズ）
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
        Rect::from_offset_size(self.offset, self.size)
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
        use windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F;
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

/// Arrangementツリー変更マーカー（ダーティビット伝播用）
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ArrangementTreeChanged;

// ====== From/Mul trait implementations ======

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

/// GlobalArrangement同士の積算（親の累積変換 * 子のローカル変換）
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
