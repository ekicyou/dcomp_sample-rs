# Specification: transform_system.rs → tree_system.rs への変更

**Status**: implementation_complete  
**Created**: 2025-11-14  
**Updated**: 2025-11-14  
**Feature**: transform-to-tree-refactor

## 概要

`crates/wintf/src/ecs/transform_system.rs` を `tree_system.rs` にリファクタリングする。

## 目的

ファイル名と内部実装を、より適切な命名規則とアーキテクチャに変更する。

## スコープ

- **対象ファイル**: `crates/wintf/src/ecs/transform_system.rs`
- **新ファイル名**: `tree_system.rs` (予定)

## 初期状態

- 現在のファイル: `transform_system.rs` (381行)
- 主要な公開関数:
  - `sync_simple_transforms<L, G, M>`
  - `mark_dirty_trees<L, G, M>`
  - `propagate_parent_transforms<L, G, M>`
- 内部実装:
  - `propagation_worker<L, G, M>`
  - `propagate_descendants_unchecked<L, G, M>` (unsafe)
  - `WorkQueue` 構造体
  - `NodeQuery<L, G, M>` 型エイリアス

## 次のステップ

1. 要件定義フェーズ: `/kiro-spec-requirements transform-to-tree-refactor`
2. 既存コードとのギャップ分析: `/kiro-validate-gap transform-to-tree-refactor` (オプション)
3. 設計フェーズ: `/kiro-spec-design transform-to-tree-refactor`
4. タスク分解: `/kiro-spec-tasks transform-to-tree-refactor`
5. 実装: `/kiro-spec-impl transform-to-tree-refactor`

## メモ

- 既存の実装は並列処理とECS階層管理の複雑なロジックを含む
- 外部依存: bevy_ecs, bevy_tasks, bevy_utils
- ジェネリック型パラメータ L, G, M を多用
