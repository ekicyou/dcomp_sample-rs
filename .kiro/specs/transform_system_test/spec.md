# 仕様: transform_system.rsのテストを「transform_test.rs」に追加

## メタデータ
- **作成日**: 2025-11-14
- **機能名**: transform_system_test
- **ステータス**: ✅ 実装完了
- **テスト種類**: インテグレーションテスト

## 概要
`crates/wintf/src/ecs/transform_system.rs`の3つのシステム関数を統合した**インテグレーションテスト**を`crates/wintf/tests/transform_test.rs`に追加する。

ツリー構造のエンティティ群（`bevy_ecs::hierarchy`使用）を用意し、`mark_dirty_trees`、`sync_simple_transforms`、`propagate_parent_transforms`を登録したスケジュールに対してツリーを更新していき、スケジュール実行ごとに`GlobalTransform`の値が正しく伝搬することを検証する。

## 背景

### 現在のファイル構成
- **対象ソースコード**: `crates/wintf/src/ecs/transform_system.rs`
  - `sync_simple_transforms<L, G, M>()` - 階層に属していないエンティティのGlobalTransformを更新
  - `mark_dirty_trees<L, G, M>()` - 静的シーン向け最適化、ダーティビットを階層の祖先に伝播
  - `propagate_parent_transforms<L, G, M>()` - 階層とTransformに基づいてGlobalTransformを更新
  - その他ヘルパー関数

- **既存のテストファイル**: `crates/wintf/tests/transform_test.rs`
  - 現在は`Transform`から`Matrix3x2`への変換テストのみ
  - `test_sync_simple_transforms()`が1つあるが、カバレッジは限定的

### 目的
- `transform_system.rs`の各関数に対する包括的なテストを追加
- テストの構造化とメンテナンス性の向上
- 既存の`transform_test.rs`との整合性維持

## 要件定義

### テストの種類
**インテグレーションテスト** - 3つのシステム関数を統合したエンドツーエンドテスト

### テスト対象システム
1. `mark_dirty_trees<L, G, M>` - 変更検出とダーティビット伝播
2. `sync_simple_transforms<L, G, M>` - 階層なしエンティティの更新
3. `propagate_parent_transforms<L, G, M>` - 階層的変換伝播

### 機能要件

#### FR-1: ツリー構造のセットアップ
**優先度**: 必須  
**説明**: テスト用の階層構造を持つエンティティ群を準備

**要件**:
1. **階層構造の作成**
   - `bevy_ecs::hierarchy::ChildOf`と`Children`を使用した親子関係
   - 各エンティティに`Transform`、`GlobalTransform`、`TransformTreeChanged`を付与
   
   **具体的なツリー構成**:
   - **ツリーA（深く広い階層）**: 5階層で複数の分岐を持つツリー
     ```
     Root_A (Level 0)          : Transform = Scale(2, 2)
       ├─ Branch_A1 (Level 1)  : Transform = Scale(3, 3)
       │    ├─ Leaf_A1a (Level 2)  : Transform = Scale(5, 5)
       │    └─ Leaf_A1b (Level 2)  : Transform = Scale(7, 7)
       └─ Branch_A2 (Level 1)  : Transform = Scale(11, 11)
            ├─ Branch_A2a (Level 2)  : Transform = Scale(13, 13)
            │    ├─ Leaf_A2a1 (Level 3)  : Transform = Scale(17, 17)
            │    └─ Branch_A2a2 (Level 3)  : Transform = Scale(19, 19)
            │         └─ Deep_Leaf_A (Level 4) ← 最深部 : Transform = Scale(23, 23)
            └─ Branch_A2b (Level 2)  : Transform = Scale(29, 29)
                 ├─ Leaf_A2b1 (Level 3)  : Transform = Scale(31, 31)
                 └─ Leaf_A2b2 (Level 3)  : Transform = Scale(37, 37)
     ```
     - 合計: 12エンティティ（素数 2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37）
     - 各エンティティに一意の素数スケール値を割り当て
     - 検証例: `Deep_Leaf_A.G = 2 * 11 * 13 * 19 * 23 = 125,774`
     - 検証ポイント: Branch_A2の変更がBranch_A2a/A2bの全子孫に伝播するが、Branch_A1には影響しない
   
   - **独立エンティティ**: 親子関係を持たない単独エンティティ
     ```
     Standalone : Transform = Scale(41, 41)
     ```
   
   - **ツリーB（中程度の階層）**: 3階層のツリー
     ```
     Root_B               : Transform = Scale(43, 43)
       ├─ Child_B1        : Transform = Scale(47, 47)
       │    └─ GrandChild_B1  : Transform = Scale(53, 53)
       └─ Child_B2        : Transform = Scale(59, 59)
            └─ GrandChild_B2  : Transform = Scale(61, 61)
     ```
     - 合計: 5エンティティ（素数 43, 47, 53, 59, 61）
     - 検証例: `GrandChild_B1.G = 43 * 47 * 53 = 107,189`

2. **World/Scheduleの準備**
   - `World`に3つのシステムを登録した`Schedule`を作成
   - システム実行順序: `mark_dirty_trees` → `sync_simple_transforms` → `propagate_parent_transforms`

#### FR-2: Transform更新とスケジュール実行
**優先度**: 必須  
**説明**: ツリー内のエンティティを更新し、スケジュールを実行

**要件**:
1. **エンティティの更新操作**
   - ルートエンティティの`Transform`を変更
   - 子エンティティの`Transform`を変更
   - `ChildOf`を変更（親子関係の変更）
   - `ChildOf`を削除（孤立化）

2. **スケジュール実行とスナップショット**
   - 各更新前に全エンティティの`GlobalTransform`をスナップショット保存
   - `schedule.run(&mut world)`を実行
   - 実行後に全エンティティの`GlobalTransform`を取得
   - 変更前後を比較し、変更されたエンティティのリストを生成

3. **変更検出ヘルパー関数**
   - `capture_global_transforms(world: &World) -> HashMap<Entity, Matrix3x2>` - スナップショット取得
     - テスト内のローカル変数として`HashMap`にG値を保存
     - コンポーネントは追加せず、テストコードのみで完結
   - `find_changed_entities(before: &HashMap<Entity, Matrix3x2>, after: &HashMap<Entity, Matrix3x2>) -> Vec<Entity>` - 変更エンティティ列挙
   - 各テストシナリオで期待される変更エンティティリストと実際の変更を比較検証

#### FR-3: GlobalTransformの伝播検証
**優先度**: 必須  
**説明**: 各スケジュール実行後、GlobalTransformが正しく伝播していることを検証

**テストシナリオ**:

1. **シナリオ1: 深く広い階層（5層）での伝播**
   - 対象: ツリーA全体
   - 初期状態でスケジュール実行
   - 全エンティティの`GlobalTransform`が素数の積で正しく計算されていることを検証
   - 検証例:
     - `Root_A.G = Scale(2, 2)`
     - `Branch_A1.G = Scale(2*3, 2*3) = Scale(6, 6)`
     - `Leaf_A1a.G = Scale(2*3*5, 2*3*5) = Scale(30, 30)`
     - `Deep_Leaf_A.G = Scale(2*11*13*19*23, 2*11*13*19*23) = Scale(125774, 125774)`

2. **シナリオ2: 部分的なサブツリーの変更**
   - 対象: ツリーA
   - Branch_A2のスケールを`Scale(11, 11)`から`Scale(67, 67)`に変更（未使用の素数に）
   - **検証ポイント**:
     - Branch_A2以下のみに伝播:
       - `Branch_A2.G = Scale(2*67, 2*67) = Scale(134, 134)`
       - `Branch_A2a.G = Scale(2*67*13, 2*67*13) = Scale(1742, 1742)`
       - `Deep_Leaf_A.G = Scale(2*67*13*19*23, 2*67*13*19*23) = Scale(766178, 766178)`
     - **Root_Aは変更されない**: `Root_A.G = Scale(2, 2)` のまま
     - **Branch_A1サブツリーは変更されない**:
       - `Branch_A1.G = Scale(6, 6)`, `Leaf_A1a.G = Scale(30, 30)`, `Leaf_A1b.G = Scale(42, 42)` のまま
     - 変更前後のG値を比較し、変更されたエンティティのリストを取得
     - 変更リスト = [Branch_A2, Branch_A2a, Leaf_A2a1, Branch_A2a2, Deep_Leaf_A, Branch_A2b, Leaf_A2b1, Leaf_A2b2]

3. **シナリオ3: 深い中間ノードの変更**
   - 対象: ツリーA
   - Branch_A2a2（Level 3）のスケールを`Scale(19, 19)`から`Scale(71, 71)`に変更
   - **検証ポイント**:
     - Deep_Leaf_Aのみに伝播:
       - `Deep_Leaf_A.G = Scale(2*11*13*71*23, 2*11*13*71*23) = Scale(470414, 470414)`
     - **Root_A, Branch_A1, Branch_A2, Branch_A2aは変更されない**: 元の値のまま
     - **Branch_A2bサブツリーは影響を受けない**: 元の値のまま
     - 変更前後のG値を比較し、変更されたエンティティのリストを取得
     - 変更リスト = [Branch_A2a2, Deep_Leaf_A]

4. **シナリオ4: 独立エンティティの更新**
   - 対象: Standalone
   - 初期状態: `Standalone.G = Scale(41, 41)`
   - スケールを`Scale(73, 73)`に変更
   - **検証ポイント**:
     - `sync_simple_transforms`により`GlobalTransform = Scale(73, 73)`に更新
     - **ツリーA、ツリーBの全エンティティは変更されない**: 元の値のまま
     - 変更前後のG値を比較し、変更されたエンティティのリストを取得
     - 変更リスト = [Standalone]

5. **シナリオ5: 複数子への並列伝播**
   - 対象: ツリーB
   - Root_Bのスケールを`Scale(43, 43)`から`Scale(79, 79)`に変更
   - **検証ポイント**:
     - 両方の子孫に伝播:
       - `Child_B1.G = Scale(79*47, 79*47) = Scale(3713, 3713)`
       - `GrandChild_B1.G = Scale(79*47*53, 79*47*53) = Scale(196789, 196789)`
       - `Child_B2.G = Scale(79*59, 79*59) = Scale(4661, 4661)`
       - `GrandChild_B2.G = Scale(79*59*61, 79*59*61) = Scale(284321, 284321)`
     - **ツリーA、Standaloneの全エンティティは変更されない**: 元の値のまま
     - 変更前後のG値を比較し、変更されたエンティティのリストを取得
     - 変更リスト = [Root_B, Child_B1, GrandChild_B1, Child_B2, GrandChild_B2]

6. **シナリオ6: 複数ツリーの同時処理**
   - 対象: ツリーA、ツリーB、Standalone
   - Root_A、Root_B、Standaloneを同時に異なる未使用の素数に変更
   - **検証ポイント**:
     - 単一のスケジュール実行で全てが正しく更新されること
     - 素数の積により各ツリーが独立して計算されていることを検証
     - 変更前後のG値を比較し、全エンティティ（18個）の変更を確認

7. **シナリオ7: 孤立化とツリー再構築**
   - 対象: ツリーA
   - Branch_A2aから`ChildOf`を削除 → Branch_A2a以下が孤立
   - **孤立後のスケジュール実行**:
     - Branch_A2aサブツリーが親の影響を受けない:
       - `Branch_A2a.G = Scale(13, 13)` （親の影響を受けない）
       - `Leaf_A2a1.G = Scale(13*17, 13*17) = Scale(221, 221)`
       - `Deep_Leaf_A.G = Scale(13*19*23, 13*19*23) = Scale(5681, 5681)`
     - **Root_A, Branch_A1サブツリー, Branch_A2, Branch_A2bサブツリーは変更されない**: 元の値のまま
     - 変更リスト = [Branch_A2a, Leaf_A2a1, Branch_A2a2, Deep_Leaf_A]
   - **Branch_A2aに新しい`ChildOf`（Root_B: Scale(43,43)へ）を追加**:
     - Branch_A2aサブツリーがツリーBに統合:
       - `Branch_A2a.G = Scale(43*13, 43*13) = Scale(559, 559)`
       - `Deep_Leaf_A.G = Scale(43*13*19*23, 43*13*19*23) = Scale(244277, 244277)`
     - **Root_B, Child_B1, Child_B2, GrandChild_B1, GrandChild_B2は変更されない**: 元の値のまま
     - 変更リスト = [Branch_A2a, Leaf_A2a1, Branch_A2a2, Deep_Leaf_A]

8. **シナリオ8: ダーティマーク最適化**
   - 対象: ツリーA、ツリーB
   - Branch_A1のスケールを`Scale(3, 3)`から`Scale(83, 83)`に変更
   - **検証ポイント**:
     - Branch_A1サブツリーのみ更新:
       - `Branch_A1.G = Scale(2*83, 2*83) = Scale(166, 166)`
       - `Leaf_A1a.G = Scale(2*83*5, 2*83*5) = Scale(830, 830)`
       - `Leaf_A1b.G = Scale(2*83*7, 2*83*7) = Scale(1162, 1162)`
     - **Root_Aは変更されない**: `Root_A.G = Scale(2, 2)` のまま
     - **Branch_A2サブツリー全体は変更されない**: 元の値のまま
     - **ツリーB全体は変更されない**: 元の値のまま
     - **Standaloneは変更されない**: 元の値のまま
     - Branch_A2サブツリーの`TransformTreeChanged`は`is_changed() == false`
     - ツリーBは完全に静的で再計算されない
     - 変更リスト = [Branch_A1, Leaf_A1a, Leaf_A1b]

### 非機能要件

#### NFR-1: テストコードの品質
- テストコードは`tests/`ディレクトリに配置（プロジェクトルール）
- 既存の`transform_test.rs`のスタイルと一貫性を保つこと
- 各テスト関数は独立して実行可能であること

#### NFR-2: テストの可読性
- テスト関数名は`test_`プレフィックスで始まり、テスト対象を明確に示すこと
- **素数スケール値による検証**: 各エンティティのGlobalTransformが素数の積として表現される
  - 初期構成で素数 2-61 を使用（重複なし、小さい値から順に割り当て）
  - テスト変更時は未使用の素数（67, 71, 73, 79, 83, ...）を使用
  - 検算が容易: 結果のスケール値を素因数分解すれば、どのエンティティを通過したか追跡可能
  - 例: `Scale(830, 830) = 2 * 5 * 83` → Root_A(2) * Leaf_A1a(5) * Branch_A1(83に変更済み)
- アサーションには期待値（素数の積）を明記すること
- 複雑なセットアップはヘルパー関数で抽象化すること

#### NFR-3: テストの実行速度
- 各テストは単体で1秒以内に完了すること
- 並列処理のテストでも過度な待機時間を避けること

#### NFR-4: メンテナンス性
- コメントでテストの意図と期待される素数の積を明確に記述すること
- ツリー構築のヘルパー関数にエンティティ名と素数値の対応を記述すること
- **変更検出ヘルパー関数**: スナップショット比較により変更されたエンティティを列挙
  - テストの検証漏れを防ぐ
  - アルゴリズム的にルートから再計算が走る場合でも、実際に値が変更されたエンティティのみを特定

### 技術的制約

#### TC-1: ECSフレームワークの制約
- `bevy_ecs 0.17.2`を使用
- `World`、`Schedule`、`Query`などのECS APIに準拠
- **階層コンポーネント**: `bevy_ecs::hierarchy::{ChildOf, Children}` を使用
  - `ChildOf`: 親エンティティへの参照（Relationshipコンポーネント）
  - `Children`: 子エンティティのリスト（自動管理される）
  - Component hookにより自動的に同期される

#### TC-2: 型パラメータの汎用性
- テスト対象関数は`<L, G, M>`のジェネリック型パラメータを持つ
- テストでは具体型として以下を使用:
  - `L` = `Transform` (ローカル変換)
  - `G` = `GlobalTransform` (グローバル変換)
  - `M` = `TransformTreeChanged` (ダーティマーク)

#### TC-3: システム実行順序
- 正しい順序でシステムを実行する必要がある:
  1. `mark_dirty_trees` - 変更を検出してツリーをマーク
  2. `sync_simple_transforms` - 階層なしエンティティを更新
  3. `propagate_parent_transforms` - 階層的に伝播

#### TC-4: 既存コードへの影響
- `transform_system.rs`のソースコードは変更しない
- 既存の`transform_test.rs`の既存テストは維持する（追加のみ）
- **テスト専用コンポーネントは追加しない**: 変更検出は`HashMap`スナップショットで実装

### 受け入れ基準

- [ ] 8つのテストシナリオすべてが実装され、passすること
- [ ] 各シナリオで`GlobalTransform`の伝播が素数の積として正しく計算されていること
- [ ] 素数スケール値により検算が容易であること（素因数分解で伝播経路を追跡可能）
- [ ] 5階層広いツリー（12エンティティ、素数2-37）、3階層ツリー（5エンティティ、素数43-61）、独立エンティティ（素数41）が全て正しく動作すること
- [ ] 部分的なサブツリー変更で、**影響を受けない兄弟サブツリーおよびルートの値が変更されていない**ことが検証されていること
- [ ] **変更検出ヘルパー関数**により、変更前後のスナップショット比較が実装されていること
- [ ] 各シナリオで期待される変更エンティティリストと実際の変更が一致すること
- [ ] `Matrix3x2`レベルでのスケール値検証が行われていること（許容誤差なし、整数値の完全一致）
- [ ] 孤立化とツリー再構築のシナリオで素数の積が正しく再計算されていること
- [ ] ダーティマーク最適化により不要な再計算が行われていないこと（変更リストで確認）
- [ ] `cargo test`で全てのテストがpassすること
- [ ] テストコードがRustのベストプラクティスに従っていること
- [ ] 既存の`transform_test.rs`の構造と一貫性があること
- [ ] ヘルパー関数でツリー構築とアサーションを抽象化していること

## 次のステップ
- [x] `/kiro-spec-requirements transform_system_test` - 詳細な要件を定義
- [x] `/kiro-spec-design transform_system_test` - 設計を作成
- [x] `/kiro-spec-tasks transform_system_test` - タスクに分解
- [x] `/kiro-spec-impl transform_system_test` - 実装を実行

## 実装結果

### 実装されたコンポーネント
✅ **Phase 1: ヘルパー関数** (完了)
- `TestEntityTree` - 18エンティティの階層構造（素数スケール値2-61）
- `capture_global_transforms()` - スナップショット取得
- `find_changed_entities()` - 変更検出
- `create_test_schedule()` - 3システム登録

✅ **Phase 2: 8つのテストシナリオ** (完了)
1. `test_scenario_1_deep_wide_hierarchy_propagation` - 深く広い階層での伝播
2. `test_scenario_2_partial_subtree_change` - 部分的なサブツリーの変更
3. `test_scenario_3_deep_intermediate_node_change` - 深い中間ノードの変更
4. `test_scenario_4_standalone_entity_update` - 独立エンティティの更新
5. `test_scenario_5_parallel_propagation_to_multiple_children` - 複数子への並列伝播
6. `test_scenario_6_concurrent_multiple_tree_processing` - 複数ツリーの同時処理
7. `test_scenario_7_isolation_and_tree_reconstruction` - 孤立化とツリー再構築
8. `test_scenario_8_dirty_mark_optimization` - ダーティマーク最適化

### テスト結果
```
running 15 tests
test test_transform_to_matrix3x2_combined ... ok
test test_transform_to_matrix3x2_identity ... ok
test test_transform_to_matrix3x2_rotate_90 ... ok
test test_transform_to_matrix3x2_scale ... ok
test test_transform_to_matrix3x2_translate ... ok
test test_transform_to_matrix3x2_with_origin ... ok
test test_sync_simple_transforms ... ok (既存)
test test_scenario_1_deep_wide_hierarchy_propagation ... ok (新規)
test test_scenario_2_partial_subtree_change ... ok (新規)
test test_scenario_3_deep_intermediate_node_change ... ok (新規)
test test_scenario_4_standalone_entity_update ... ok (新規)
test test_scenario_5_parallel_propagation_to_multiple_children ... ok (新規)
test test_scenario_6_concurrent_multiple_tree_processing ... ok (新規)
test test_scenario_7_isolation_and_tree_reconstruction ... ok (新規)
test test_scenario_8_dirty_mark_optimization ... ok (新規)

test result: ok. 15 passed; 0 failed; 0 ignored
```

### 実装のハイライト
- **素数スケール値**: 各エンティティに一意の素数（2-61）を割り当て、素因数分解で伝播経路を追跡可能
- **変更検出**: HashMap スナップショットで変更前後を比較し、不要な再計算がないことを確認
- **包括的カバレッジ**: 8シナリオで深い階層、部分更新、孤立化、最適化をテスト
- **既存テスト保持**: 既存7テストは影響を受けず、すべて動作

### 受け入れ基準の達成状況
- ✅ 8つのテストシナリオすべてが実装され、pass
- ✅ GlobalTransformの伝播が素数の積として正しく計算
- ✅ 5階層広いツリー、3階層ツリー、独立エンティティが正しく動作
- ✅ 部分的なサブツリー変更で影響範囲を正確に検証
- ✅ 変更検出ヘルパー関数によるスナップショット比較を実装
- ✅ Matrix3x2レベルでのスケール値検証（完全一致）
- ✅ 孤立化とツリー再構築のシナリオが正しく動作
- ✅ ダーティマーク最適化により不要な再計算がないことを確認
- ✅ cargo testで全テストpass
- ✅ 既存テストとの一貫性を保持

---
*この仕様は Kiro ワークフローに従って管理されています*
