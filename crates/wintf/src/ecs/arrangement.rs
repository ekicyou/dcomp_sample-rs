use crate::ecs::layout::{Arrangement, ArrangementTreeChanged, GlobalArrangement};
use crate::ecs::common::tree_system::{
    mark_dirty_trees, propagate_parent_transforms, sync_simple_transforms, NodeQuery, WorkQueue,
};
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::prelude::*;

/// 階層に属していないEntity（ルートWindow）のGlobalArrangementを更新
pub fn sync_simple_arrangements(
    query: ParamSet<(
        Query<
            (&Arrangement, &mut GlobalArrangement),
            (
                Or<(Changed<Arrangement>, Added<GlobalArrangement>)>,
                Without<ChildOf>,
                Without<Children>,
            ),
        >,
        Query<(Ref<Arrangement>, &mut GlobalArrangement), (Without<ChildOf>, Without<Children>)>,
    )>,
    orphaned: RemovedComponents<ChildOf>,
) {
    sync_simple_transforms::<Arrangement, GlobalArrangement, ArrangementTreeChanged>(
        query, orphaned,
    );
}

/// 「ダーティビット」を階層の祖先に向かって伝播
pub fn mark_dirty_arrangement_trees(
    changed: Query<
        Entity,
        Or<(
            Changed<Arrangement>,
            Changed<ChildOf>,
            Added<GlobalArrangement>,
        )>,
    >,
    orphaned: RemovedComponents<ChildOf>,
    transforms: Query<(Option<&ChildOf>, &mut ArrangementTreeChanged)>,
) {
    mark_dirty_trees::<Arrangement, GlobalArrangement, ArrangementTreeChanged>(
        changed, orphaned, transforms,
    );
}

/// 親から子へGlobalArrangementを伝播
pub fn propagate_global_arrangements(
    queue: Local<WorkQueue>,
    roots: Query<
        (Entity, Ref<Arrangement>, &mut GlobalArrangement, &Children),
        (Without<ChildOf>, Changed<ArrangementTreeChanged>),
    >,
    nodes: NodeQuery<Arrangement, GlobalArrangement, ArrangementTreeChanged>,
) {
    propagate_parent_transforms::<Arrangement, GlobalArrangement, ArrangementTreeChanged>(
        queue, roots, nodes,
    );
}
