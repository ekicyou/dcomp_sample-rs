use bevy_ecs::prelude::*;
use bevy_ecs::relationship::Relationship;
use windows_numerics::Matrix3x2;

/// 平行移動（CSS transform: translate に相当）
#[derive(Component, Default, Clone, Copy, Debug, PartialEq)]
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
#[derive(Component, Clone, Copy, Debug, PartialEq)]
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
#[derive(Component, Default, Clone, Copy, Debug, PartialEq)]
pub struct Rotate(pub f32);

impl From<Rotate> for Matrix3x2 {
    fn from(r: Rotate) -> Self {
        Matrix3x2::rotation(r.0.to_radians())
    }
}

/// 傾斜変換（CSS transform: skew に相当）
#[derive(Component, Default, Clone, Copy, Debug, PartialEq)]
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
#[derive(Component, Clone, Copy, Debug, PartialEq)]
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

/// Globalな変換行列。
/// エンティティツリーの親子関係から計算する。
#[repr(transparent)]
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub struct GlobalTransform(pub Matrix3x2);

impl From<Matrix3x2> for GlobalTransform {
    fn from(matrix: Matrix3x2) -> Self {
        GlobalTransform(matrix)
    }
}

/// すべての変換コンポーネントから最終的なMatrix3x2を計算
/// 適用順序: TransformOrigin → Scale → Rotate → Skew → Translate
pub fn compute_transform_matrix(
    translate: Option<Translate>,
    scale: Option<Scale>,
    rotate: Option<Rotate>,
    skew: Option<Skew>,
    origin: Option<TransformOrigin>,
) -> Matrix3x2 {
    let translate = translate.unwrap_or_default();
    let scale = scale.unwrap_or_default();
    let rotate = rotate.unwrap_or_default();
    let skew = skew.unwrap_or_default();
    let origin = origin.unwrap_or_default();

    let origin_offset = Matrix3x2::translation(-origin.x, -origin.y);
    let origin_restore = Matrix3x2::translation(origin.x, origin.y);

    let scale_matrix: Matrix3x2 = scale.into();
    let rotate_matrix: Matrix3x2 = rotate.into();
    let skew_matrix: Matrix3x2 = skew.into();
    let translate_matrix: Matrix3x2 = translate.into();

    origin_offset * scale_matrix * rotate_matrix * skew_matrix * origin_restore * translate_matrix
}

/// Transformコンポーネントが変更されたときにLocalTransformを更新するシステム
pub fn update_local_transform(
    mut query: Query<
        (
            Option<&Translate>,
            Option<&Scale>,
            Option<&Rotate>,
            Option<&Skew>,
            Option<&TransformOrigin>,
            &mut LocalTransform,
        ),
        Or<(
            Changed<Translate>,
            Changed<Scale>,
            Changed<Rotate>,
            Changed<Skew>,
            Changed<TransformOrigin>,
        )>,
    >,
) {
    for (translate, scale, rotate, skew, origin, mut local_transform) in query.iter_mut() {
        let matrix = compute_transform_matrix(
            translate.copied(),
            scale.copied(),
            rotate.copied(),
            skew.copied(),
            origin.copied(),
        );
        *local_transform = matrix.into();
    }
}

/// LocalTransformまたは親のGlobalTransformが変更されたときにGlobalTransformを更新するシステム
/// 階層構造を正しく処理するため、World排他アクセスを使用
pub fn update_global_transform(world: &mut World) {
    let mut changed_entities = Vec::new();
    
    // 変更されたエンティティを収集
    {
        let mut query = world.query_filtered::<Entity, Or<(Changed<LocalTransform>, Changed<GlobalTransform>)>>();
        for entity in query.iter(world) {
            changed_entities.push(entity);
        }
    }
    
    // 変更されたエンティティとその子孫を更新
    for entity in changed_entities {
        propagate_global_transform(world, entity);
    }
}

fn propagate_global_transform(world: &mut World, entity: Entity) {
    let local_transform = world.get::<LocalTransform>(entity).copied();
    
    // 親エンティティを取得
    let parent_entity = if let Some(child_of) = world.get::<ChildOf>(entity) {
        Some(child_of.get())
    } else {
        None
    };
    
    if let Some(local) = local_transform {
        let global_matrix = if let Some(parent) = parent_entity {
            if let Some(parent_global) = world.get::<GlobalTransform>(parent) {
                parent_global.0 * local.0
            } else {
                local.0
            }
        } else {
            local.0
        };
        
        if let Some(mut global) = world.get_mut::<GlobalTransform>(entity) {
            global.0 = global_matrix;
        }
    }
    
    // 子エンティティを探して伝播
    let mut query = world.query::<(Entity, &ChildOf)>();
    let children_entities: Vec<Entity> = query
        .iter(world)
        .filter(|(_, child_of)| child_of.get() == entity)
        .map(|(e, _)| e)
        .collect();
    
    for child in children_entities {
        propagate_global_transform(world, child);
    }
}
