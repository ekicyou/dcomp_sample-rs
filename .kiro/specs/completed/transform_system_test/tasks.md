# Implementation Plan: transform_system_test

## Overview
`transform_system.rs`の3つのシステム関数を統合したインテグレーションテストを`crates/wintf/tests/transform_test.rs`に実装する。素数スケール値による検算可能なテスト設計で、8つのシナリオをカバーする。

## Task Breakdown

### Phase 1: ヘルパー関数の実装

- [ ] 1. TestEntityTree構造体の実装 (P)
- [ ] 1.1 TestEntityTree構造体の定義
  - 18個のEntityフィールド（ツリーA: 12, ツリーB: 5, Standalone: 1）を定義
  - `new(world: &mut World) -> Self`メソッドのシグネチャ定義
  - _Requirements: FR-1_

- [ ] 1.2 ツリーA（12エンティティ）の構築
  - Root_A (素数2) をspawn
  - Branch_A1サブツリー（素数3, 5, 7）を構築し、ChildOf設定
  - Branch_A2サブツリー（素数11, 13, 17, 19, 23, 29, 31, 37）を構築し、ChildOf設定
  - 各エンティティに`Transform`, `GlobalTransform`, `TransformTreeChanged`を付与
  - _Requirements: FR-1_

- [ ] 1.3 ツリーB（5エンティティ）とStandaloneの構築
  - Root_B（素数43）をspawn
  - Child_B1, Child_B2サブツリー（素数47, 53, 59, 61）を構築し、ChildOf設定
  - Standalone（素数41）をspawn（ChildOfなし）
  - 各エンティティに`Transform`, `GlobalTransform`, `TransformTreeChanged`を付与
  - _Requirements: FR-1_

- [ ] 1.4 ヘルパーメソッドの実装
  - `all_entities() -> Vec<Entity>`メソッド実装
  - エンティティ名と素数値の対応をコメントで記述
  - _Requirements: FR-1_

- [ ] 2. スナップショットヘルパー関数の実装 (P)
- [ ] 2.1 capture_global_transforms関数の実装
  - `fn capture_global_transforms(world: &World) -> HashMap<Entity, Matrix3x2>`を実装
  - `world.query::<(Entity, &GlobalTransform)>()`でクエリ
  - HashMap にEntity → Matrix3x2のマッピングを保存
  - _Requirements: FR-2_

- [ ] 2.2 find_changed_entities関数の実装
  - `fn find_changed_entities(before: &HashMap<Entity, Matrix3x2>, after: &HashMap<Entity, Matrix3x2>) -> Vec<Entity>`を実装
  - Matrix3x2の完全一致比較（許容誤差なし）
  - 変更されたエンティティのリストを返す
  - _Requirements: FR-2, FR-3_

- [ ] 3. テストスケジュール作成関数の実装 (P)
- [ ] 3.1 create_test_schedule関数の実装
  - `fn create_test_schedule() -> Schedule`を実装
  - `mark_dirty_trees::<Transform, GlobalTransform, TransformTreeChanged>`を追加
  - `sync_simple_transforms::<Transform, GlobalTransform, TransformTreeChanged>`を追加
  - `propagate_parent_transforms::<Transform, GlobalTransform, TransformTreeChanged>`を追加
  - 正しい実行順序を確保
  - _Requirements: FR-1_

### Phase 2: テストシナリオの実装（並列可能）

- [ ] 4. Scenario 1: 深く広い階層での伝播 (P)
- [ ] 4.1 test_scenario_1_deep_wide_hierarchy_propagation関数の実装
  - TestEntityTreeのセットアップ
  - 初期スケジュール実行
  - 全12エンティティのGlobalTransformを検証
  - `Deep_Leaf_A.G = Scale(125774, 125774)` を確認
  - _Requirements: FR-3_

- [ ] 5. Scenario 2: 部分的なサブツリーの変更 (P)
- [ ] 5.1 test_scenario_2_partial_subtree_change関数の実装
  - セットアップと初期実行
  - before スナップショット取得
  - Branch_A2を`Scale(67, 67)`に変更
  - スケジュール実行とafter スナップショット取得
  - 変更リスト（8エンティティ）を検証
  - Root_A、Branch_A1サブツリーが変更されていないことを確認
  - `Deep_Leaf_A.G = Scale(766178, 766178)` を確認
  - _Requirements: FR-3_

- [ ] 6. Scenario 3: 深い中間ノードの変更 (P)
- [ ] 6.1 test_scenario_3_deep_intermediate_node_change関数の実装
  - セットアップと初期実行
  - before スナップショット取得
  - Branch_A2a2を`Scale(71, 71)`に変更
  - スケジュール実行とafter スナップショット取得
  - 変更リスト（2エンティティ: Branch_A2a2, Deep_Leaf_A）を検証
  - Root_A、Branch_A1、Branch_A2、Branch_A2a、Branch_A2bサブツリーが変更されていないことを確認
  - `Deep_Leaf_A.G = Scale(470414, 470414)` を確認
  - _Requirements: FR-3_

- [ ] 7. Scenario 4: 独立エンティティの更新 (P)
- [ ] 7.1 test_scenario_4_standalone_entity_update関数の実装
  - セットアップと初期実行
  - before スナップショット取得
  - Standaloneを`Scale(73, 73)`に変更
  - スケジュール実行とafter スナップショット取得
  - 変更リスト（1エンティティ: Standalone）を検証
  - ツリーA、ツリーBの全エンティティが変更されていないことを確認
  - `Standalone.G = Scale(73, 73)` を確認
  - _Requirements: FR-3_

- [ ] 8. Scenario 5: 複数子への並列伝播 (P)
- [ ] 8.1 test_scenario_5_parallel_propagation_to_multiple_children関数の実装
  - セットアップと初期実行
  - before スナップショット取得
  - Root_Bを`Scale(79, 79)`に変更
  - スケジュール実行とafter スナップショット取得
  - 変更リスト（5エンティティ: ツリーB全体）を検証
  - ツリーA、Standaloneが変更されていないことを確認
  - `GrandChild_B1.G = Scale(196789, 196789)` を確認
  - _Requirements: FR-3_

- [ ] 9. Scenario 6: 複数ツリーの同時処理 (P)
- [ ] 9.1 test_scenario_6_concurrent_multiple_tree_processing関数の実装
  - セットアップと初期実行
  - before スナップショット取得
  - Root_A、Root_B、Standaloneを異なる未使用の素数に同時変更
  - スケジュール実行とafter スナップショット取得
  - 変更リスト（18エンティティ全て）を検証
  - 各ツリーが独立して正しく計算されていることを確認
  - _Requirements: FR-3_

- [ ] 10. Scenario 7: 孤立化とツリー再構築 (P)
- [ ] 10.1 test_scenario_7_isolation_and_tree_reconstruction関数の実装（孤立化フェーズ）
  - セットアップと初期実行
  - before スナップショット取得
  - Branch_A2aから`ChildOf`を削除
  - スケジュール実行とafter スナップショット取得
  - 変更リスト（4エンティティ: Branch_A2aサブツリー）を検証
  - `Branch_A2a.G = Scale(13, 13)` を確認（親の影響なし）
  - Root_A、Branch_A1サブツリー、Branch_A2、Branch_A2bサブツリーが変更されていないことを確認
  - _Requirements: FR-3_

- [ ] 10.2 test_scenario_7_isolation_and_tree_reconstruction関数の実装（再構築フェーズ）
  - 孤立化後の状態から継続
  - before2 スナップショット取得
  - Branch_A2aに`ChildOf(Root_B)`を追加
  - スケジュール実行とafter2 スナップショット取得
  - 変更リスト（4エンティティ: Branch_A2aサブツリー）を検証
  - `Branch_A2a.G = Scale(559, 559)` を確認（Root_Bの影響）
  - Root_B、Child_B1、Child_B2サブツリーが変更されていないことを確認
  - _Requirements: FR-3_

- [ ] 11. Scenario 8: ダーティマーク最適化 (P)
- [ ] 11.1 test_scenario_8_dirty_mark_optimization関数の実装
  - セットアップと初期実行
  - before スナップショット取得
  - Branch_A1を`Scale(83, 83)`に変更
  - スケジュール実行とafter スナップショット取得
  - 変更リスト（3エンティティ: Branch_A1サブツリー）を検証
  - Root_A、Branch_A2サブツリー全体、ツリーB全体、Standaloneが変更されていないことを確認
  - `Leaf_A1a.G = Scale(830, 830)` を確認
  - Branch_A2の`TransformTreeChanged.is_changed() == false` を確認
  - _Requirements: FR-3_

### Phase 3: テスト検証とドキュメント

- [ ] 12. ヘルパー関数のユニットテスト実装
- [ ] 12.1 test_capture_global_transforms関数の実装
  - 簡単なエンティティでスナップショット取得をテスト
  - HashMapに正しく保存されることを確認
  - _Requirements: NFR-2_

- [ ] 12.2 test_find_changed_entities関数の実装
  - 変更あり/なしのケースをテスト
  - 正しいエンティティリストが返されることを確認
  - _Requirements: NFR-2_

- [ ] 12.3 test_prime_scale_calculation関数の実装
  - 素数の積計算が正しいことを確認
  - 期待値と実際の値の一致を検証
  - _Requirements: NFR-2_

- [ ] 13. 既存テストの動作確認と統合
- [ ] 13.1 既存テストの実行確認
  - `cargo test transform_test`を実行
  - 既存の7つのテストが引き続き動作することを確認
  - 新規追加の11テストも全てpassすることを確認
  - _Requirements: TC-4_

- [ ] 13.2 テストコードの最終レビュー
  - コメントでテストの意図が明確に記述されていることを確認
  - 素数値の対応がドキュメント化されていることを確認
  - エラーメッセージが適切に設定されていることを確認
  - _Requirements: NFR-2, NFR-4_

- [ ] 14. 最終検証
- [ ] 14.1 cargo testの実行
  - `cargo test` で全テストがpassすることを確認
  - テストカバレッジの確認
  - _Requirements: すべて_

## Task Dependencies

```
Phase 1 (並列可能)
├─ Task 1 (TestEntityTree)
├─ Task 2 (Snapshot/Change Detection)
└─ Task 3 (Test Schedule)

↓ (Phase 1完了後)

Phase 2 (並列可能: Task 4-11)
├─ Task 4 (Scenario 1)
├─ Task 5 (Scenario 2)
├─ Task 6 (Scenario 3)
├─ Task 7 (Scenario 4)
├─ Task 8 (Scenario 5)
├─ Task 9 (Scenario 6)
├─ Task 10 (Scenario 7)
└─ Task 11 (Scenario 8)

↓ (Phase 2完了後)

Phase 3 (順次実行)
├─ Task 12 (Helper Tests)
├─ Task 13 (Integration Check)
└─ Task 14 (Final Verification)
```

## Estimated Effort

| Phase | Tasks | Estimated Time | Priority |
|-------|-------|----------------|----------|
| Phase 1 | 1-3 | 2-3 hours | P0 (必須) |
| Phase 2 | 4-11 | 4-6 hours | P0 (必須) |
| Phase 3 | 12-14 | 1-2 hours | P0 (必須) |
| **Total** | **14** | **7-11 hours** | |

## Notes

- Phase 1のタスクは並列実装可能だが、依存関係はなし
- Phase 2の8つのシナリオテストは完全に独立しており、並列実装可能
- Phase 3は順次実行が望ましい（統合確認のため）
- 各タスクは独立してテスト可能
- すべてのテストで素数スケール値を使用し、検算可能性を確保
