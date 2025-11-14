use bevy_ecs::entity::*;
use bevy_ecs::prelude::*;
use bevy_ecs::system::lifetimeless::*;
use bevy_tasks::*;
use bevy_utils::*;
use std::sync::atomic::*;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::*;

use super::transform::{GlobalTransform, Transform, TransformTreeChanged};

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

/// エンティティ階層と[`Transform`]コンポーネントに基づいて、
/// エンティティの[`GlobalTransform`]コンポーネントを更新する。
///
/// サードパーティプラグインは、このシステムを
/// [`sync_simple_transforms`](super::sync_simple_transforms)および
/// [`mark_dirty_trees`](super::mark_dirty_trees)と組み合わせて使用する必要がある。
pub fn propagate_parent_transforms(
    mut queue: Local<WorkQueue>,
    mut roots: Query<
        (Entity, Ref<Transform>, &mut GlobalTransform, &Children),
        (Without<ChildOf>, Changed<TransformTreeChanged>),
    >,
    nodes: NodeQuery,
) {
    // ルートを並列処理し、ワークキューを準備する
    roots.par_iter_mut().for_each_init(
        || queue.local_queue.borrow_local_mut(),
        |outbox, (parent, transform, mut parent_transform, children)| {
            *parent_transform = GlobalTransform((*transform).into());

            // SAFETY: この関数に渡される親エンティティは、ルートエンティティクエリの
            // イテレーションから取得される。クエリは互いに素なエンティティをイテレートするため、
            // ミュータブルなエイリアシングを防ぎ、この呼び出しを安全にする。
            #[expect(unsafe_code, reason = "Mutating disjoint entities in parallel")]
            unsafe {
                propagate_descendants_unchecked(
                    parent,
                    parent_transform,
                    children,
                    &nodes,
                    outbox,
                    &queue,
                    // より代表的なシーンでプロファイリングして、この単一最大深度を再検討する必要がある。
                    // ワーカーを開始する前に、階層の深部に進んで良好なタスクキューを構築することが
                    // 実際に有益である可能性がある。しかし、現時点では、単一のスレッドが階層の深部に
                    // 進む間、他のスレッドがアイドル状態になるケースを防ぐために、これを避けている。
                    // これは、タスク共有ワーカーが既に解決している問題である。
                    1,
                );
            }
        },
    );
    // ルート処理後にスレッドローカルアウトボックス内のすべてのタスクを送信し、
    // 部分的なバッチの送信を避けることでチャネル送信の総数を削減する。
    queue.send_batches();

    if let Ok(rx) = queue.receiver.try_lock() {
        if let Some(task) = rx.try_iter().next() {
            // 少し馬鹿げているが、作業があるかどうかを確認する唯一の方法はタスクを取得することである。
            // peekはnextを呼び出さなくてもタスクを削除してしまい、タスクがドロップされる結果になる。
            // ここで行っているのは、最初のタスクがあればそれを取得し、すぐにキューの後ろに送り返すことである。
            queue.sender.send(task).ok();
        } else {
            return; // 作業がない場合、タスクを生成する必要はない
        }
    }

    // タスクプールにワーカーを生成し、階層を並列に再帰的に伝播する。
    let task_pool = ComputeTaskPool::get_or_init(TaskPool::default);
    task_pool.scope(|s| {
        (1..task_pool.thread_num()) // 最初のワーカーはタスクプールではなくローカルで実行される
            .for_each(|_| s.spawn(async { propagation_worker(&queue, &nodes) }));
        propagation_worker(&queue, &nodes);
    });
}

/// キューから処理された親エンティティを消費し、その[`GlobalTransform`]を
/// 伝播したら子をキューにプッシュする並列ワーカー。
#[inline]
fn propagation_worker(queue: &WorkQueue, nodes: &NodeQuery) {
    let mut outbox = queue.local_queue.borrow_local_mut();
    loop {
        // タイトループでワークキューのロックを取得しようとする。プロファイリングによると、
        // これは`.lock()`に依存するよりもはるかに効率的で、タスク間にギャップが生じるのを防ぐ。
        let Ok(rx) = queue.receiver.try_lock() else {
            core::hint::spin_loop(); // プロファイルに明らかな影響はないが、ベストプラクティス
            continue;
        };
        // キューが空で、他のスレッドが作業を処理していない場合、もう作業がないと結論付け、
        // ループを終了してタスクを終了する。
        let Some(mut tasks) = rx.try_iter().next() else {
            if queue.busy_threads.load(Ordering::Relaxed) == 0 {
                break; // すべての作業が完了、ワーカーを終了
            }
            continue; // 今は作業がないが、別のスレッドがさらに作業を作成している
        };
        if tasks.is_empty() {
            continue; // これは起こらないはずだが、起こった場合は早期に停止する
        }

        // タスクキューが非常に短い場合、この非常に短いタスクが完了した後に必要な
        // スレッド同期の量を減らすために、さらにいくつかのタスクを収集する価値がある。
        while tasks.len() < WorkQueue::CHUNK_SIZE / 2 {
            let Some(mut extra_task) = rx.try_iter().next() else {
                break;
            };
            tasks.append(&mut extra_task);
        }

        // この時点で、作業があることがわかっているので、ビジースレッドカウンターを
        // インクリメントし、カウンターをインクリメントした*後*にミューテックスガードを
        // ドロップする。これにより、別のスレッドがロックを取得できる場合、ビジースレッド
        // カウンターは既にインクリメントされていることが保証される。
        queue.busy_threads.fetch_add(1, Ordering::Relaxed);
        drop(rx); // 重要: アトミック操作の後、作業開始前にドロップ

        for parent in tasks.drain(..) {
            // SAFETY: ワーカーキューにプッシュされた各タスクは、階層の未処理のサブツリーを
            // 表しており、一意のアクセスが保証されている。
            #[expect(unsafe_code, reason = "Mutating disjoint entities in parallel")]
            unsafe {
                let (_, (_, p_global_transform, _), (p_children, _)) =
                    nodes.get_unchecked(parent).unwrap();
                propagate_descendants_unchecked(
                    parent,
                    p_global_transform,
                    p_children.unwrap(), // キュー内のすべてのエンティティは子を持つべき
                    nodes,
                    &mut outbox,
                    queue,
                    // パフォーマンスにのみ影響する。これより深いツリーも完全に伝播されるが、
                    // 作業は複数のタスクに分割される。この数値は、合理的なツリーの深さよりも
                    // 大きく選択されているが、関数が深い階層でハングするほど大きくはない。
                    10_000,
                );
            }
        }
        WorkQueue::send_batches_with(&queue.sender, &mut outbox);
        queue.busy_threads.fetch_add(-1, Ordering::Relaxed);
    }
}

/// `parent`からその`children`に変換を伝播し、更新された子エンティティを
/// `outbox`にプッシュする。この関数は深さ優先探索で子孫に変換を伝播し続けながら、
/// 同時に未訪問のブランチをアウトボックスにプッシュして、他のスレッドがアイドル時に
/// 取得できるようにする。
///
/// # Safety
///
/// 呼び出し側は、この関数への同時呼び出しに一意の`parent`エンティティが与えられることを
/// 保証する必要がある。同じ`parent`でこの関数を同時に呼び出すことは健全ではない。
/// この関数は、伝播中のミュータブルエイリアシングを防ぐために、エンティティ階層に
/// サイクルが含まれていないことを検証するが、同じエンティティをミュータブルに
/// エイリアスするために使用されていないことを検証することはできない。
///
/// ## Panics
///
/// 子ノードの親が指定された`parent`と同じでない場合にパニックする。
/// このアサーションは、階層が非循環であることを保証し、呼び出し側が指定された
/// 安全性ルールに従っている場合、マルチスレッド伝播が健全であることを保証する。
#[inline]
#[expect(unsafe_code, reason = "Mutating disjoint entities in parallel")]
unsafe fn propagate_descendants_unchecked(
    parent: Entity,
    p_global_transform: Mut<GlobalTransform>,
    p_children: &Children,
    nodes: &NodeQuery,
    outbox: &mut Vec<Entity>,
    queue: &WorkQueue,
    max_depth: usize,
) {
    // 反復的な深さ優先探索に使用する入力変数のミュータブルコピーを作成
    let (mut parent, mut p_global_transform, mut p_children) =
        (parent, p_global_transform, p_children);

    // このループがここにある理由を理解するには、最後の最適化ノートを参照
    for depth in 1..=max_depth {
        // Safety: ルートからエンティティツリーをトラバースする際、childofと
        // childrenポインタが双方向で一致することをアサート（下記のassertを参照）して、
        // 階層にサイクルがないことを保証する。階層にサイクルがないため、並列で
        // 互いに素なエンティティを訪問していることがわかり、これは安全である。
        #[expect(unsafe_code, reason = "Mutating disjoint entities in parallel")]
        let children_iter = unsafe {
            nodes.iter_many_unique_unsafe(UniqueEntityIter::from_iterator_unchecked(
                p_children.iter(),
            ))
        };

        let mut last_child = None;
        let new_children = children_iter.filter_map(
            |(child, (transform, mut global_transform, tree), (children, child_of))| {
                if !tree.is_changed() && !p_global_transform.is_changed() {
                    // 静的シーンの最適化
                    return None;
                }
                assert_eq!(child_of.parent(), parent);

                // 変換の伝播はコストが高い - これは、GlobalTransformが変更されていない場合、
                // 追加の等価性チェックのコストで、サブツリー全体の更新を回避するのに役立つ。
                let a = *p_global_transform;
                let b = *transform;
                global_transform.set_if_neq(a * b);

                children.map(|children| {
                    // エンティティが子を持つ場合にのみ伝播を続ける
                    last_child = Some((child, global_transform, children));
                    child
                })
            },
        );
        outbox.extend(new_children);

        if depth >= max_depth || last_child.is_none() {
            break; // アウトボックスから何も削除せず、チャンクも送信せず、単に終了
        }

        // 最適化: タスクは可能な限りスレッド同期を避けるために、できる限りローカルで作業を消費する
        if let Some(last_child) = last_child {
            // 親データを子で上書きし、ループして子孫を反復処理する
            (parent, p_global_transform, p_children) = last_child;
            outbox.pop();

            // トラバース中にチャンクを送信。これにより、トラバースを完全に完了する前に
            // 他のスレッドとタスクを共有できる。
            if outbox.len() >= WorkQueue::CHUNK_SIZE {
                WorkQueue::send_batches_with(&queue.sender, outbox);
            }
        }
    }
}

/// 大きく繰り返し使用されるクエリのエイリアス。親と場合によっては子の両方を持つ
/// 変換エンティティをクエリするため、これらはルートではない。
type NodeQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        (
            Ref<'static, Transform>,
            Mut<'static, GlobalTransform>,
            Ref<'static, TransformTreeChanged>,
        ),
        (Option<Read<Children>>, Read<ChildOf>),
    ),
>;

/// 変換伝播のためにスレッド間で共有されるキュー。
pub struct WorkQueue {
    /// 作業中のスレッド数を追跡するセマフォ。もう作業がないことを判断するために使用される。
    busy_threads: AtomicI32,
    sender: Sender<Vec<Entity>>,
    receiver: Arc<Mutex<Receiver<Vec<Entity>>>>,
    local_queue: Parallel<Vec<Entity>>,
}
impl Default for WorkQueue {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        Self {
            busy_threads: AtomicI32::default(),
            sender: tx,
            receiver: Arc::new(Mutex::new(rx)),
            local_queue: Default::default(),
        }
    }
}
impl WorkQueue {
    const CHUNK_SIZE: usize = 512;

    #[inline]
    fn send_batches_with(sender: &Sender<Vec<Entity>>, outbox: &mut Vec<Entity>) {
        for chunk in outbox
            .chunks(WorkQueue::CHUNK_SIZE)
            .filter(|c| !c.is_empty())
        {
            sender.send(chunk.to_vec()).ok();
        }
        outbox.clear();
    }

    #[inline]
    fn send_batches(&mut self) {
        let Self {
            sender,
            local_queue,
            ..
        } = self;
        // バッチ化されたタスクを送信するためにローカルをイテレートし、ローカルを
        // より大きな割り当てにドレインする必要を回避する。
        local_queue
            .iter_mut()
            .for_each(|outbox| Self::send_batches_with(sender, outbox));
    }
}
