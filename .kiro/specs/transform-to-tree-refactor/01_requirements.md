# Requirements: transform_system.rs → tree_system.rs への変更

**Status**: requirements_defined  
**Updated**: 2025-11-14  
**Feature**: transform-to-tree-refactor

## 1. 変更の目的

### 1.1 主要目標
`transform_system.rs` を `tree_system.rs` にリファクタリングし、ファイル名とモジュール名をその責務により適合させる。

### 1.2 背景
現在のファイル名 `transform_system.rs` は誤解を招く可能性がある:
- 実際の実装は**階層ツリー伝播システム**（tree propagation）であり、変換（transform）そのものの定義ではない
- `transform.rs` が `Transform`, `GlobalTransform` 等のコンポーネントを定義
- `transform_system.rs` は階層構造における変換の伝播を担当

より適切な命名により、責務を明確化し、コードベースの可読性を向上させる。

## 2. 機能要件

### 2.1 保持すべき機能
以下の全ての公開APIと機能は**変更なし**で保持する必要がある:

#### 公開関数
1. `sync_simple_transforms<L, G, M>` - 階層に属さないエンティティのGlobalTransform更新
2. `mark_dirty_trees<L, G, M>` - ダーティビットの階層伝播（静的シーン最適化）
3. `propagate_parent_transforms<L, G, M>` - 親から子への変換伝播（並列処理）

#### 内部実装
- `propagation_worker<L, G, M>` - ワーカー関数
- `propagate_descendants_unchecked<L, G, M>` - unsafeな子孫伝播（深さ優先探索）
- `WorkQueue` - 並列処理用ワークキュー構造体
- `NodeQuery<L, G, M>` - 型エイリアス

#### ジェネリック型パラメータ
- `L`: ローカル変換コンポーネント（`Transform`）
- `G`: グローバル変換コンポーネント（`GlobalTransform`）
- `M`: ツリー変更マーカーコンポーネント（`TransformTreeChanged`）

#### 依存関係
- `bevy_ecs` の `Children`, `ChildOf` コンポーネント
- `bevy_tasks::ComputeTaskPool` - 並列タスクプール
- `bevy_utils::Parallel` - スレッドローカルストレージ

### 2.2 変更すべき要素

#### ファイル名
- **現在**: `crates/wintf/src/ecs/transform_system.rs`
- **変更後**: `crates/wintf/src/ecs/tree_system.rs`

#### モジュール宣言
- **現在**: `pub mod transform_system;` (in `ecs/mod.rs`)
- **変更後**: `pub mod tree_system;`

#### 再エクスポート
- **現在**: `pub use transform_system::*;` (in `ecs/mod.rs`)
- **変更後**: `pub use tree_system::*;`

### 2.3 影響を受けるコード

#### 外部利用箇所
1. **テストコード**: `crates/wintf/tests/transform_test.rs`
   - `use wintf::ecs::transform_system::*;` → `use wintf::ecs::tree_system::*;`
   - システム関数の呼び出し（変更不要、再エクスポートにより透過的）

2. **ECSモジュール**: `crates/wintf/src/ecs/mod.rs`
   - モジュール宣言と再エクスポートの更新

## 3. 非機能要件

### 3.1 互換性
- **API互換性**: 全ての公開API（関数シグネチャ、ジェネリック制約）は変更しない
- **動作互換性**: 既存のテストコード（8シナリオ）が全てパスする必要がある

### 3.2 パフォーマンス
- 並列処理性能を維持（`WorkQueue`, `propagation_worker` の実装は変更しない）
- `unsafe` ブロックの安全性保証を維持

### 3.3 コード品質
- コメントとドキュメント（日本語）を全て保持
- コード構造（関数順序、行数）を可能な限り維持
- Rust標準の命名規則（`snake_case.rs`）に従う

## 4. 制約条件

### 4.1 技術的制約
- ファイル名変更のみ（実装ロジックは変更しない）
- `bevy_ecs` v0.17.2 の階層関連機能（`Children`, `ChildOf`）に依存
- Windows環境でのビルドとテスト実行

### 4.2 プロジェクト制約
- Kiroワークフローに従った段階的変更
- Git履歴の明確性（ファイル名変更のコミットは独立させる）
- ステアリングドキュメント（`.kiro/steering/`）との整合性

## 5. 受け入れ基準

### 5.1 必須条件
- [ ] ファイルが `tree_system.rs` に正しくリネームされている
- [ ] `ecs/mod.rs` のモジュール宣言が更新されている
- [ ] テストコードのインポート文が更新されている
- [ ] `cargo build` が成功する
- [ ] `cargo test` が全てパスする（特に8つのシナリオテスト）

### 5.2 検証方法
```bash
# ビルド確認
cargo build

# テスト実行（重要）
cargo test --package wintf --test transform_test

# 特定のシナリオテスト
cargo test test_scenario_ --package wintf
```

### 5.3 ドキュメント更新
- [ ] この仕様ドキュメント自体が最新状態である
- [ ] 必要に応じてREADME.mdに補足説明を追加（オプション）

## 6. リスクと軽減策

### 6.1 識別されたリスク

| リスク | 影響度 | 軽減策 |
|--------|--------|--------|
| インポート文の更新漏れ | 高 | `cargo build` でコンパイルエラーを検出 |
| テスト失敗 | 中 | 全テスト実行で検証 |
| Git履歴の不明瞭化 | 低 | ファイル名変更のみのコミットを作成 |

### 6.2 ロールバック計画
- Git でコミット前の状態に戻す
- ファイル名を `transform_system.rs` に戻し、インポート文を元に戻す

## 7. 依存関係

### 7.1 前提条件
- 既存のコードベースが正常にビルド・テストできる状態
- `.kiro/specs/transform-to-tree-refactor/00_init.md` が作成済み

### 7.2 後続フェーズ
- **設計フェーズ**: `/kiro-spec-design transform-to-tree-refactor`
- **タスク分解**: `/kiro-spec-tasks transform-to-tree-refactor`
- **実装フェーズ**: `/kiro-spec-impl transform-to-tree-refactor`

## 8. 追加コンテキスト

### 8.1 関連ファイル
- `crates/wintf/src/ecs/transform.rs` - Transform/GlobalTransformコンポーネント定義
- `crates/wintf/src/ecs/mod.rs` - ECSモジュールの統合ポイント
- `crates/wintf/tests/transform_test.rs` - 包括的なシステムテスト

### 8.2 参考情報
- bevy_ecs階層システム: `Children`, `ChildOf` の使用パターン
- 並列処理: `bevy_tasks::ComputeTaskPool` を使用したマルチスレッド伝播
- 静的シーン最適化: `mark_dirty_trees` による変更検出の最適化

---

**次のステップ**: `/kiro-spec-design transform-to-tree-refactor`
