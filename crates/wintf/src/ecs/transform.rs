use bevy_ecs::prelude::*;
use windows_numerics::Matrix3x2;

/// 平行移動（CSS transform: translate に相当）
#[derive(Default, Clone, Copy, Debug, PartialEq)]
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
#[derive(Clone, Copy, Debug, PartialEq)]
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
        Matrix3x2::scale(s.x, s.y)
    }
}

/// 回転（CSS transform: rotate に相当）
/// 角度は度数法で指定（UI用なので0/90/180/270が主）
#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Rotate(pub f32);

impl From<Rotate> for Matrix3x2 {
    fn from(r: Rotate) -> Self {
        Matrix3x2::rotation(r.0.to_radians())
    }
}

/// 傾斜変換（CSS transform: skew に相当）
#[derive(Default, Clone, Copy, Debug, PartialEq)]
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
#[derive(Clone, Copy, Debug, PartialEq)]
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

/// 2D変換を表すコンポーネント
/// Translate、Scale、Rotate、Skew、TransformOriginをまとめて管理
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
pub struct Transform {
    pub translate: Translate,
    pub scale: Scale,
    pub rotate: Rotate,
    pub skew: Skew,
    pub origin: TransformOrigin,
}

impl From<Transform> for Matrix3x2 {
    fn from(transform: Transform) -> Self {
        let origin_offset = Matrix3x2::translation(-transform.origin.x, -transform.origin.y);
        let origin_restore = Matrix3x2::translation(transform.origin.x, transform.origin.y);

        let scale_matrix: Matrix3x2 = transform.scale.into();
        let rotate_matrix: Matrix3x2 = transform.rotate.into();
        let skew_matrix: Matrix3x2 = transform.skew.into();
        let translate_matrix: Matrix3x2 = transform.translate.into();

        origin_offset
            * scale_matrix
            * rotate_matrix
            * skew_matrix
            * origin_restore
            * translate_matrix
    }
}

/// エンティティローカルの変換行列。
/// TransformOrigin → Scale → Rotate → Skew → Translateを計算した値。
/// 親子関係は考慮しない。
#[derive(Component, Clone, Copy, Debug, PartialEq)]
#[repr(transparent)]
pub struct LocalTransform(pub Matrix3x2);

impl From<Matrix3x2> for LocalTransform {
    fn from(matrix: Matrix3x2) -> Self {
        LocalTransform(matrix)
    }
}

/// LocalTransformの変更を追跡するためのマーカーコンポーネント。
/// Transform関連のコンポーネント（Translate, Scale, Rotate, Skew, TransformOrigin）が
/// 変更されたときに自動的に追加される。
/// LocalTransformの更新が完了したら削除される。
#[derive(Component, Default)]
#[component(storage = "SparseSet")]
pub struct LocalTransformChanged;

/// Transformコンポーネントが変更されたときにLocalTransformを更新または作成し、
/// LocalTransformChangedマーカーを追加するシステム
pub fn update_local_transform(
    mut commands: Commands,
    query: Query<(Entity, &Transform), Changed<Transform>>,
) {
    for (entity, transform) in query.iter() {
        let matrix: Matrix3x2 = (*transform).into();
        let new_local_transform: LocalTransform = matrix.into();

        // LocalTransformを更新または新規作成
        commands.entity(entity).insert(new_local_transform);

        // LocalTransformが更新されたことを示すマーカーを追加
        commands.entity(entity).insert(LocalTransformChanged);
    }
}
