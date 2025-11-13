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

/// グローバル変換行列コンポーネント
#[derive(Component, Clone, Copy, Debug, Default, PartialEq)]
#[repr(transparent)]
pub struct GlobalTransform(pub Matrix3x2);

impl From<GlobalTransform> for Matrix3x2 {
    fn from(gt: GlobalTransform) -> Self {
        gt.0
    }
}

/// 階層に属していないエンティティの[`GlobalTransform`]コンポーネントを更新する。
///
/// サードパーティプラグインは、このシステムを
/// [`propagate_parent_transforms`]および[`mark_dirty_trees`]と組み合わせて使用する必要がある。
pub fn sync_simple_transforms(
    mut query: ParamSet<(
        Query<
            (&Transform, &mut GlobalTransform),
            (
                Or<(Changed<Transform>, Added<GlobalTransform>)>,
                Without<ChildOf>,
                Without<Children>,
            ),
        >,
        Query<(Ref<Transform>, &mut GlobalTransform), (Without<ChildOf>, Without<Children>)>,
    )>,
    mut orphaned: RemovedComponents<ChildOf>,
) {
    // 変更されたエンティティを更新
    query
        .p0()
        .par_iter_mut()
        .for_each(|(transform, mut global_transform)| {
            *global_transform = GlobalTransform((*transform).into());
        });
    // 孤立したエンティティを更新
    let mut query = query.p1();
    let mut iter = query.iter_many_mut(orphaned.read());
    while let Some((transform, mut global_transform)) = iter.fetch_next() {
        if !transform.is_changed() && !global_transform.is_added() {
            *global_transform = GlobalTransform((*transform).into());
        }
    }
}

/// 変換伝播の最適化のためのマーカーコンポーネント。
/// このゼロサイズ型（ZST）のマーカーコンポーネントは、変更検出を使用して
/// 階層内の全てのエンティティを「ダーティ」としてマークする。これは、子孫の
/// いずれかが変更された`Transform`を持つ場合に発生する。
/// このコンポーネントが`is_changed()`でマークされて*いない*場合、伝播は停止する。
#[derive(Component, Clone, Copy, Default, PartialEq, Debug)]
pub struct TransformTreeChanged;

/// 静的シーン向けの最適化。「ダーティビット」を階層の祖先に向かって伝播させる。
/// 変換の伝播は、ダーティビットを持たないエンティティに遭遇した場合、
/// 階層のサブツリー全体を無視できる。
pub fn mark_dirty_trees(
    changed_transforms: Query<
        Entity,
        Or<(Changed<Transform>, Changed<ChildOf>, Added<GlobalTransform>)>,
    >,
    mut orphaned: RemovedComponents<ChildOf>,
    mut transforms: Query<(Option<&ChildOf>, &mut TransformTreeChanged)>,
) {
    for entity in changed_transforms.iter().chain(orphaned.read()) {
        let mut next = entity;
        while let Ok((child_of, mut tree)) = transforms.get_mut(next) {
            if tree.is_changed() && !tree.is_added() {
                // コンポーネントが変更されていた場合、このツリーの部分は既に処理済み。
                // ただし、変更がコンポーネントの追加によって引き起こされた場合は無視する。
                break;
            }
            tree.set_changed();
            if let Some(parent) = child_of.map(ChildOf::parent) {
                next = parent;
            } else {
                break;
            };
        }
    }
}
