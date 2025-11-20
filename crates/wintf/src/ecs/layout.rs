use bevy_ecs::prelude::*;
use taffy::prelude::*;
use windows_numerics::Matrix3x2;

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

impl Default for LayoutScale {
    fn default() -> Self {
        Self { x: 1.0, y: 1.0 }
    }
}

/// ローカルレイアウト配置（オフセット + スケール）
#[derive(Component, Debug, Clone, Copy, PartialEq)]
#[component(on_add = on_arrangement_add)]
pub struct Arrangement {
    pub offset: Offset,
    pub scale: LayoutScale,
}

impl Default for Arrangement {
    fn default() -> Self {
        Self {
            offset: Offset::default(),
            scale: LayoutScale::default(),
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

/// グローバルレイアウト変換（親からの累積変換）
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct GlobalArrangement(pub Matrix3x2);

impl Default for GlobalArrangement {
    fn default() -> Self {
        Self(Matrix3x2::identity())
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
        Self(arrangement.into())
    }
}

/// GlobalArrangement同士の乗算（親の累積変換 * 子のローカル変換）
impl std::ops::Mul<Arrangement> for GlobalArrangement {
    type Output = GlobalArrangement;

    fn mul(self, rhs: Arrangement) -> Self::Output {
        let child_matrix: Matrix3x2 = rhs.into();
        GlobalArrangement(self.0 * child_matrix)
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
        };

        let matrix: Matrix3x2 = arr.into();
        assert_eq!(matrix.M11, 2.0);
        assert_eq!(matrix.M22, 3.0);
        assert_eq!(matrix.M31, 10.0);
        assert_eq!(matrix.M32, 20.0);
    }

    #[test]
    fn test_global_arrangement_mul() {
        let parent = GlobalArrangement(Matrix3x2::translation(10.0, 20.0));

        let child = Arrangement {
            offset: Offset { x: 5.0, y: 7.0 },
            scale: LayoutScale { x: 1.0, y: 1.0 },
        };

        let result = parent * child;
        assert_eq!(result.0.M31, 15.0); // 10 + 5
        assert_eq!(result.0.M32, 27.0); // 20 + 7
    }
}
