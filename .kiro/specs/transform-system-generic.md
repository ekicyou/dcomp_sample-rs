# 仕様: transform_system.rs実装のジェネリック化

**機能名**: `transform-system-generic`  
**作成日**: 2025-11-14  
**ステータス**: Phase 1完了、Phase 2実装準備中

## 概要

既存の3つのシステム関数に型パラメータを追加し、任意の変換型で使用可能にする。

## 目標

`crates/wintf/src/ecs/transform_system.rs`の3つの関数を型パラメータ化：
- `sync_simple_transforms` → `sync_simple_transforms<L, G, M>`
- `mark_dirty_trees` → `mark_dirty_trees<L, G, M>`
- `propagate_parent_transforms` → `propagate_parent_transforms<L, G, M>`

## 要件定義

### 型パラメータ（3つ）

#### L: ローカル変換型
```rust
L: Component + Copy + Into<G>
```
- `Component`: bevy_ecsコンポーネント
- `Copy`: 値渡しで効率的
- `Into<G>`: グローバル変換型Gへ変換可能

**使用箇所**: `sync_simple_transforms`で`(*transform).into()`

#### G: グローバル変換型
```rust
G: Component + Copy + PartialEq + Mul<L, Output = G>
```
- `Component`: bevy_ecsコンポーネント
- `Copy`: 値渡しで効率的
- `PartialEq`: `set_if_neq`最適化
- `Mul<L, Output = G>`: 変換合成（`parent * child`）

**使用箇所**: `propagate_descendants_unchecked`で`a * b`

#### M: ダーティマーカー型
```rust
M: Component
```
- `Component`: bevy_ecsコンポーネント

**使用箇所**: `mark_dirty_trees`で変更検出

### スコープ外

- `ChildOf`, `Children` → 具体型のまま維持（bevy_ecs標準）
- `WorkQueue` → 変更なし（型に依存しない）
- 新しいトレイト定義 → 不要（既存の`Into`と`Mul`で十分）

### 受け入れ基準

- [ ] 既存の型（`Transform`, `GlobalTransform`, `TransformTreeChanged`）で動作
- [ ] `cargo build`が通る
- [ ] 型推論が機能する

## 実装タスク

### タスク1: 型パラメータの追加（2-3時間）

**ファイル**: `crates/wintf/src/ecs/transform_system.rs`

#### 1. 関数シグネチャの変更

**sync_simple_transforms**:
```rust
// Before
pub fn sync_simple_transforms(
    mut query: ParamSet<(
        Query<(&Transform, &mut GlobalTransform), (Or<(Changed<Transform>, Added<GlobalTransform>)>, Without<ChildOf>, Without<Children>)>,
        Query<(Ref<Transform>, &mut GlobalTransform), (Without<ChildOf>, Without<Children>)>,
    )>,
    mut orphaned: RemovedComponents<ChildOf>,
)

// After
pub fn sync_simple_transforms<L, G, M>(
    mut query: ParamSet<(
        Query<(&L, &mut G), (Or<(Changed<L>, Added<G>)>, Without<ChildOf>, Without<Children>)>,
        Query<(Ref<L>, &mut G), (Without<ChildOf>, Without<Children>)>,
    )>,
    mut orphaned: RemovedComponents<ChildOf>,
) where
    L: Component + Copy + Into<G>,
    G: Component + Copy + PartialEq + Mul<L, Output = G>,
    M: Component,
```

**mark_dirty_trees**:
```rust
// Before
pub fn mark_dirty_trees(
    changed_transforms: Query<Entity, Or<(Changed<Transform>, Changed<ChildOf>, Added<GlobalTransform>)>>,
    mut orphaned: RemovedComponents<ChildOf>,
    mut transforms: Query<(Option<&ChildOf>, &mut TransformTreeChanged)>,
)

// After
pub fn mark_dirty_trees<L, G, M>(
    changed_transforms: Query<Entity, Or<(Changed<L>, Changed<ChildOf>, Added<G>)>>,
    mut orphaned: RemovedComponents<ChildOf>,
    mut transforms: Query<(Option<&ChildOf>, &mut M)>,
) where
    L: Component + Copy + Into<G>,
    G: Component + Copy + PartialEq + Mul<L, Output = G>,
    M: Component,
```

**propagate_parent_transforms**:
```rust
// Before
pub fn propagate_parent_transforms(
    mut queue: Local<WorkQueue>,
    mut roots: Query<(Entity, Ref<Transform>, &mut GlobalTransform, &Children), (Without<ChildOf>, Changed<TransformTreeChanged>)>,
    nodes: NodeQuery,
)

// After
pub fn propagate_parent_transforms<L, G, M>(
    mut queue: Local<WorkQueue>,
    mut roots: Query<(Entity, Ref<L>, &mut G, &Children), (Without<ChildOf>, Changed<M>)>,
    nodes: NodeQuery<L, G, M>,
) where
    L: Component + Copy + Into<G>,
    G: Component + Copy + PartialEq + Mul<L, Output = G>,
    M: Component,
```

#### 2. 内部関数の変更

**propagation_worker**:
```rust
// Before
fn propagation_worker(queue: &WorkQueue, nodes: &NodeQuery)

// After
fn propagation_worker<L, G, M>(queue: &WorkQueue, nodes: &NodeQuery<L, G, M>)
where
    L: Component + Copy + Into<G>,
    G: Component + Copy + PartialEq + Mul<L, Output = G>,
    M: Component,
```

**propagate_descendants_unchecked**:
```rust
// Before
unsafe fn propagate_descendants_unchecked(
    parent: Entity,
    p_global_transform: Mut<GlobalTransform>,
    p_children: &Children,
    nodes: &NodeQuery,
    outbox: &mut Vec<Entity>,
    queue: &WorkQueue,
    max_depth: usize,
)

// After
unsafe fn propagate_descendants_unchecked<L, G, M>(
    parent: Entity,
    p_global_transform: Mut<G>,
    p_children: &Children,
    nodes: &NodeQuery<L, G, M>,
    outbox: &mut Vec<Entity>,
    queue: &WorkQueue,
    max_depth: usize,
) where
    L: Component + Copy + Into<G>,
    G: Component + Copy + PartialEq + Mul<L, Output = G>,
    M: Component,
```

#### 3. 型エイリアスの変更

**NodeQuery**:
```rust
// Before
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

// After
type NodeQuery<'w, 's, L, G, M> = Query<
    'w,
    's,
    (
        Entity,
        (
            Ref<'static, L>,
            Mut<'static, G>,
            Ref<'static, M>,
        ),
        (Option<Read<Children>>, Read<ChildOf>),
    ),
>;
```

#### 4. 変換処理の微調整

**sync_simple_transformsとpropagation_worker内**:
```rust
// Before
*global_transform = GlobalTransform((*transform).into());

// After
*global_transform = (*transform).into();
```

理由: `Into<G>`トレイトで直接変換可能、newtypeラッパーは不要

### 検証

1. **ビルド確認**:
   ```bash
   cargo build --package wintf
   ```

2. **型推論テスト**:
   既存の型で関数を呼び出して、型パラメータが正しく推論されることを確認

3. **動作確認**:
   既存のサンプル（`areka.rs`, `dcomp_demo.rs`）がビルドできることを確認
   （現状これらのサンプルは変換システムを使用していないが、将来的に使用する可能性）

## 設計決定の理由

### なぜTransformOpsトレイトを作らないのか？

既存のRust標準トレイト（`Into`, `Mul`）で十分対応可能：
- `Into<G>` → ローカルからグローバルへの変換
- `Mul<L, Output = G>` → 変換の合成

新しいトレイトは抽象化レイヤーを増やすだけで、利点がない。

### なぜジェネリック版と既存版を分けないのか？

型パラメータを追加しても、既存の使用方法はそのまま動作：
```rust
// 既存の使用方法（型推論で動作）
world.add_systems(Update, sync_simple_transforms);

// ジェネリック版の明示的使用
world.add_systems(Update, sync_simple_transforms::<MyLocal, MyGlobal, MyMarker>);
```

分ける理由がない。

### なぜWorkQueueを変更しないのか？

`WorkQueue`は`Entity`ベースで動作し、変換型に依存しない。
変更すると複雑になるだけでメリットがない。

## フェーズ

### Phase 1: 仕様策定 ✅
- [x] 初期化完了
- [x] 要件定義完了
- [x] ギャップ分析完了
- [x] 設計完了
- [x] タスク分解完了

### Phase 2: 実装
- [ ] 型パラメータの追加（2-3時間）

## 次のステップ

実装を開始:
1. `sync_simple_transforms`に型パラメータを追加
2. `mark_dirty_trees`に型パラメータを追加
3. `propagate_parent_transforms`と内部関数に型パラメータを追加
4. `NodeQuery`型エイリアスを更新
5. ビルド確認

---
_最終更新: 2025-11-14_
