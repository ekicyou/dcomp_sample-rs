# Design: transform_system.rs → tree_system.rs への変更

**Status**: design_complete  
**Updated**: 2025-11-14  
**Feature**: transform-to-tree-refactor

## 1. アーキテクチャ設計

### 1.1 変更の性質
このリファクタリングは**純粋なファイル名変更**であり、以下は変更しない:
- モジュール内部の実装ロジック
- 公開API（関数シグネチャ、ジェネリック制約）
- テスト動作

### 1.2 モジュール構造の変更

#### 変更前
```
crates/wintf/src/ecs/
├── mod.rs
│   ├── pub mod transform_system;
│   └── pub use transform_system::*;
├── transform.rs          (コンポーネント定義)
└── transform_system.rs   (階層伝播システム)
```

#### 変更後
```
crates/wintf/src/ecs/
├── mod.rs
│   ├── pub mod tree_system;
│   └── pub use tree_system::*;
├── transform.rs          (コンポーネント定義) ← 変更なし
└── tree_system.rs        (階層伝播システム)   ← リネーム
```

### 1.3 責務の明確化

| モジュール | 責務 | 主要な型 |
|-----------|------|---------|
| `transform.rs` | 変換コンポーネントの定義 | `Transform`, `GlobalTransform`, `TransformTreeChanged` |
| `tree_system.rs` | 階層ツリーにおける変換の伝播システム | `sync_simple_transforms`, `mark_dirty_trees`, `propagate_parent_transforms` |

## 2. 変更対象ファイルの詳細設計

### 2.1 ファイル: `crates/wintf/src/ecs/transform_system.rs`

#### 操作
**ファイル名変更**: `transform_system.rs` → `tree_system.rs`

#### 実装内容
ファイル内容は**完全に保持**する（コピー）:
- 381行のコード
- 全てのコメント（日本語ドキュメント）
- 全てのインポート文
- 全ての関数実装（unsafe含む）
- `WorkQueue`構造体の実装

#### 技術的詳細
Gitでのファイル名変更を検出可能にするため:
1. ファイル内容を変更せずに移動
2. Git履歴で `git mv` 相当の操作として記録

### 2.2 ファイル: `crates/wintf/src/ecs/mod.rs`

#### 変更箇所1: モジュール宣言
```rust
// 変更前
pub mod transform_system;

// 変更後
pub mod tree_system;
```

**行番号**: 5行目

#### 変更箇所2: 再エクスポート
```rust
// 変更前
pub use transform_system::*;

// 変更後
pub use tree_system::*;
```

**行番号**: 15行目

#### その他の行
変更なし（`transform.rs`のモジュール宣言・再エクスポートはそのまま）

### 2.3 ファイル: `crates/wintf/tests/transform_test.rs`

#### 変更箇所: インポート文
```rust
// 変更前
use wintf::ecs::transform_system::*;

// 変更後
use wintf::ecs::tree_system::*;
```

**行番号**: 4行目

#### その他の行
変更なし（テストコード本体は668行全て保持）

## 3. データフロー設計

### 3.1 変更前後の比較

#### 変更前のインポートフロー
```
transform_test.rs
  └─> use wintf::ecs::transform_system::*
        └─> ecs/mod.rs: pub use transform_system::*
              └─> ecs/transform_system.rs
```

#### 変更後のインポートフロー
```
transform_test.rs
  └─> use wintf::ecs::tree_system::*
        └─> ecs/mod.rs: pub use tree_system::*
              └─> ecs/tree_system.rs
```

### 3.2 API互換性の保証

再エクスポート（`pub use tree_system::*`）により、以下は変更不要:
- システム関数の呼び出し
- ジェネリック型パラメータの使用
- `WorkQueue`などの内部型の利用（テスト内では直接参照なし）

## 4. 実装戦略

### 4.1 実装順序

```
Phase 1: ファイル移動
  1. transform_system.rs → tree_system.rs (内容コピー)
  2. 元のtransform_system.rsを削除
  
Phase 2: インポート更新
  3. ecs/mod.rsの2箇所を更新
  4. tests/transform_test.rsの1箇所を更新
  
Phase 3: 検証
  5. cargo buildで構文エラー確認
  6. cargo testで動作確認
```

### 4.2 Git操作戦略

#### 推奨: 単一コミット
```bash
# ファイル名変更とインポート更新を1つのコミットにまとめる
git add -A
git commit -m "Refactor: Rename transform_system.rs to tree_system.rs

- Rename crates/wintf/src/ecs/transform_system.rs to tree_system.rs
- Update module declarations in ecs/mod.rs
- Update import in tests/transform_test.rs
- No functional changes, API remains identical"
```

#### 理由
- ファイル名変更とインポート更新は不可分の変更
- ビルドが通る状態でコミット履歴を保つ

### 4.3 エラーハンドリング

| エラーシナリオ | 検出方法 | 対処法 |
|--------------|---------|--------|
| ファイル移動失敗 | ファイルシステムエラー | 手動で再実行 |
| インポート文の更新漏れ | `cargo build` でコンパイルエラー | 該当箇所を修正 |
| テスト失敗 | `cargo test` で失敗 | ファイル内容の差分を確認、必要なら戻す |
| Git履歴の不整合 | `git status` で確認 | コミット前に状態確認 |

## 5. インターフェース設計

### 5.1 公開APIの保証

以下の関数シグネチャは**完全に保持**:

```rust
pub fn sync_simple_transforms<L, G, M>(
    mut query: ParamSet<(...)>,
    mut orphaned: RemovedComponents<ChildOf>,
) where
    L: Component + Copy + Into<G>,
    G: Component<Mutability = Mutable> + Copy + PartialEq + Mul<L, Output = G>,
    M: Component<Mutability = Mutable>,
{ /* 実装変更なし */ }

pub fn mark_dirty_trees<L, G, M>(
    changed_transforms: Query<Entity, Or<(...)>>,
    mut orphaned: RemovedComponents<ChildOf>,
    mut transforms: Query<(Option<&ChildOf>, &mut M)>,
) where
    L: Component + Copy + Into<G>,
    G: Component<Mutability = Mutable> + Copy + PartialEq + Mul<L, Output = G>,
    M: Component<Mutability = Mutable>,
{ /* 実装変更なし */ }

pub fn propagate_parent_transforms<L, G, M>(
    mut queue: Local<WorkQueue>,
    mut roots: Query<(Entity, Ref<L>, &mut G, &Children), (Without<ChildOf>, Changed<M>)>,
    nodes: NodeQuery<L, G, M>,
) where
    L: Component + Copy + Into<G>,
    G: Component<Mutability = Mutable> + Copy + PartialEq + Mul<L, Output = G>,
    M: Component<Mutability = Mutable>,
{ /* 実装変更なし */ }
```

### 5.2 内部APIの保証

型エイリアスと構造体も保持:

```rust
type NodeQuery<'w, 's, L, G, M> = Query<
    'w,
    's,
    (
        Entity,
        (Ref<'static, L>, Mut<'static, G>, Ref<'static, M>),
        (Option<Read<Children>>, Read<ChildOf>),
    ),
>;

pub struct WorkQueue {
    busy_threads: AtomicI32,
    sender: Sender<Vec<Entity>>,
    receiver: Arc<Mutex<Receiver<Vec<Entity>>>>,
    local_queue: Parallel<Vec<Entity>>,
}
```

## 6. テスト設計

### 6.1 既存テストの保証

`transform_test.rs`の以下のテストが**全てパス**する必要がある:

#### 単体テスト (6個)
- `test_transform_to_matrix3x2_identity`
- `test_transform_to_matrix3x2_translate`
- `test_transform_to_matrix3x2_scale`
- `test_transform_to_matrix3x2_rotate_90`
- `test_transform_to_matrix3x2_combined`
- `test_transform_to_matrix3x2_with_origin`
- `test_sync_simple_transforms`

#### シナリオテスト (8個)
1. `test_scenario_1_deep_wide_hierarchy_propagation`
2. `test_scenario_2_partial_subtree_change`
3. `test_scenario_3_deep_intermediate_node_change`
4. `test_scenario_4_standalone_entity_update`
5. `test_scenario_5_parallel_propagation_to_multiple_children`
6. `test_scenario_6_concurrent_multiple_tree_processing`
7. `test_scenario_7_isolation_and_tree_reconstruction`
8. `test_scenario_8_dirty_mark_optimization`

### 6.2 検証コマンド

```bash
# 全テスト実行
cargo test --package wintf --test transform_test

# シナリオテストのみ
cargo test test_scenario_ --package wintf

# ビルド確認
cargo build --package wintf
```

## 7. パフォーマンス設計

### 7.1 パフォーマンス影響

**影響なし** - 理由:
- 実装コードが完全に同一
- コンパイル後のバイナリは変わらない
- モジュール名はコンパイル時に解決される

### 7.2 並列処理の保証

以下の並列処理実装は変更なし:
- `ComputeTaskPool`を使用したマルチスレッド処理
- `WorkQueue`による作業分散
- `propagation_worker`のスピンループ最適化
- `unsafe`ブロックによる並列アクセス最適化

## 8. セキュリティとエラーハンドリング

### 8.1 Unsafe コードの保持

以下の`unsafe`ブロックは完全に保持:
- `propagate_descendants_unchecked` 関数内
- `propagation_worker` 関数内
- 安全性保証のドキュメント（Safety コメント）も保持

### 8.2 エラー処理の保持

以下のエラーハンドリングは変更なし:
- `assert_eq!(child_of.parent(), parent)` - 階層サイクル検出
- `WorkQueue`のチャネルエラー処理（`.ok()`による無視）

## 9. 制約と前提条件

### 9.1 技術的制約
- Rust 2021 Edition
- bevy_ecs 0.17.2 のAPI互換性
- Windows環境でのビルド

### 9.2 プロジェクト制約
- ステアリングドキュメントとの整合性（structure.md参照）
- 既存のECSアーキテクチャパターンを維持

## 10. 検証計画

### 10.1 検証チェックリスト

```
[ ] Phase 1: 構文検証
    [ ] cargo build が成功
    [ ] 警告が増えていない
    
[ ] Phase 2: 動作検証
    [ ] cargo test --package wintf が全テストパス
    [ ] 特にtransform_testの8シナリオテスト
    
[ ] Phase 3: Git履歴検証
    [ ] git status で意図しないファイル変更がない
    [ ] git diff で変更箇所が3ファイルのみ
    
[ ] Phase 4: ドキュメント検証
    [ ] 仕様ドキュメントと実装が一致
```

### 10.2 ロールバック手順

問題が発生した場合:
```bash
# コミット前
git restore .

# コミット後
git revert HEAD
```

## 11. 次のステップ

### 11.1 タスク分解フェーズ
`/kiro-spec-tasks transform-to-tree-refactor`

期待されるタスク:
1. ファイル名変更（transform_system.rs → tree_system.rs）
2. ecs/mod.rsのモジュール宣言更新
3. ecs/mod.rsの再エクスポート更新
4. tests/transform_test.rsのインポート文更新
5. ビルド検証
6. テスト実行

### 11.2 実装フェーズ
`/kiro-spec-impl transform-to-tree-refactor`

---

**設計承認待ち** - 次のコマンド: `/kiro-spec-tasks transform-to-tree-refactor [-y]`
