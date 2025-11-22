# Implementation Tasks

## Overview
ECSコンポーネントを5つの機能グループに再編成するリファクタリングを3つのPhaseに分けて実施します。各Phase完了時にビルドとテストを検証し、API互換性を維持しながら段階的に進めます。

## Task Breakdown

### Phase 1: Common Infrastructure分離

- [x] 1. Common Infrastructureサブフォルダーの作成と汎用階層伝播システムの移動
- [x] 1.1 (P) 汎用階層伝播システムの配置準備
  - `crates/wintf/src/ecs/common/`ディレクトリを作成
  - 既存の`tree_system.rs`を`ecs/common/tree_system.rs`に移動
  - _Requirements: 6_

- [x] 1.2 (P) Common Infrastructureモジュール定義の作成
  - `ecs/common/mod.rs`を作成し、`tree_system`モジュールを宣言
  - `pub mod tree_system;`および`pub use tree_system::*;`で再エクスポート
  - _Requirements: 6_

- [x] 1.3 既存コードのインポートパス更新
  - `arrangement.rs`のインポート文を`use crate::ecs::tree_system::*;`から`use crate::ecs::common::tree_system::*;`に変更
  - 他のモジュールで`tree_system`を使用している箇所があれば同様に更新
  - _Requirements: 6_

- [x] 1.4 ルートモジュールでのCommon Infrastructure再エクスポート
  - `ecs/mod.rs`に`pub mod common;`を追加
  - `pub use common::tree_system::*;`で汎用関数を再エクスポート
  - API互換性を維持（外部利用者のコード変更不要）
  - _Requirements: 1, 6, 9_

- [x] 1.5 Phase 1検証：Common Infrastructure分離の動作確認
  - `cargo check --all-targets`でコンパイル成功を確認
  - `cargo test`で全テスト成功を確認
  - Gitコミット作成（"refactor: Common Infrastructureをecs/common/に分離"）
  - _Requirements: 9_

### Phase 2: Layout System統合

- [ ] 2. Layout Systemサブフォルダの作成と大型ファイルの分割
- [ ] 2.1 (P) Layout Systemサブフォルダの準備
  - `crates/wintf/src/ecs/layout/`ディレクトリを作成
  - 既存の`layout.rs`の内容を確認し、分割計画を立てる（taffy, metrics, arrangement, rect, systemsの5モジュール）
  - _Requirements: 4_

- [x] 2.2 (P) taffyレイアウトエンジン連携モジュールの抽出
  - `layout.rs`の行1-20周辺から`BoxStyle`, `BoxComputedLayout`定義を抽出
  - `layout/taffy.rs`を作成し、taffyクレートの`Style`, `Layout`をラップするコンポーネントを配置
  - bevy_ecsの`Component`トレイト実装を維持
  - _Requirements: 4_

- [x] 2.3 (P) レイアウトメトリクスモジュールの抽出
  - `layout.rs`の行30-80周辺から`Size`, `Offset`, `LayoutScale`, `TextLayoutMetrics`定義を抽出
  - `layout/metrics.rs`を作成し、レイアウトメトリクスコンポーネントを配置
  - `Size`構造体の使用例doctestを含める（1個）
  - _Requirements: 4_

- [x] 2.4 (P) 配置情報コンポーネントモジュールの抽出
  - `layout.rs`の行260-400周辺から`Arrangement`, `GlobalArrangement`, `ArrangementTreeChanged`定義を抽出
  - `layout/arrangement.rs`を作成し、配置情報コンポーネントを配置
  - `GlobalArrangement`構造体の使用例doctestを含める（1個）
  - _Requirements: 4_

- [x] 2.5 (P) 矩形操作ユーティリティモジュールの抽出
  - `layout.rs`の行70-230周辺から`Rect`型エイリアス、`D2DRectExt`トレイト、`transform_rect_axis_aligned`関数を抽出
  - `layout/rect.rs`を作成し、矩形操作機能を配置
  - `D2DRectExt`トレイトのdoctest（1個）と`transform_rect_axis_aligned`関数のdoctest（1個）を含める
  - Direct2Dの`D2D_RECT_F`型へのトレイト実装を維持
  - _Requirements: 4_

- [x] 2.6 配置伝播システム関数の統合
  - 既存の`arrangement.rs`（60行）の内容を`layout/systems.rs`に移動
  - `sync_simple_arrangements`, `mark_dirty_arrangement_trees`, `propagate_global_arrangements`システム関数を配置
  - `use crate::ecs::common::tree_system::*;`でCommon Infrastructureの汎用関数をインポート
  - _Requirements: 4, 6_

- [x] 2.7 Layout Systemモジュール定義の作成
  - `ecs/layout/mod.rs`を作成
  - 5つのサブモジュール（taffy, metrics, arrangement, rect, systems）を宣言
  - `pub use`で各サブモジュールの全型・関数を再エクスポート
  - _Requirements: 4_

- [x] 2.8 ルートモジュールでのLayout System統合
  - `ecs/mod.rs`から`mod arrangement;`および`pub use arrangement::*;`を削除
  - `pub mod layout;`を追加
  - `pub use layout::*;`で全Layout Systemコンポーネントを再エクスポート
  - API互換性を維持（`use wintf::ecs::*;`で従来通りアクセス可能）
  - _Requirements: 1, 4, 9_

- [x] 2.9 Phase 2検証：Layout System統合の動作確認
  - `cargo check --all-targets`でコンパイル成功を確認
  - `cargo test`で全テスト成功を確認（4つのdoctestを含む）
  - doctestの内訳検証：Size（1個）、D2DRectExt（1個）、transform_rect_axis_aligned（1個）、GlobalArrangement（1個）
  - Gitコミット作成（"refactor: Layout Systemをecs/layout/に統合"）
  - _Requirements: 4, 9_

### Phase 3: Transform非推奨化

- [ ] 3. Transform実験的コンポーネントの隔離と非推奨化
- [x] 3.1 (P) Transformサブフォルダの作成と移動
  - `crates/wintf/src/ecs/transform/`ディレクトリを作成
  - 既存の`transform.rs`を`transform/components.rs`に改名移動
  - コンポーネント定義（`Translate`, `Scale`, `Rotate`, `Skew`, `Transform`, `GlobalTransform`等）をそのまま維持
  - _Requirements: 5_

- [x] 3.2 (P) Transform非推奨警告モジュールの作成
  - `ecs/transform/mod.rs`を作成
  - モジュールレベルdocコメント（`//!`）で非推奨警告とMigration Guideを記載
  - WinUI3模倣としての位置付けと`Arrangement`ベースレイアウトへの移行推奨を明記
  - `pub mod components;`および`pub use components::*;`で再エクスポート
  - _Requirements: 5_

- [x] 3.3 ルートモジュールでのTransform再エクスポート維持
  - `ecs/mod.rs`で`mod transform;`を`pub mod transform;`に変更
  - `pub use transform::*;`を維持（API互換性のため）
  - _Requirements: 5, 9_

- [x] 3.4 Phase 3検証：Transform非推奨化の動作確認
  - `cargo check --all-targets`でコンパイル成功を確認
  - `cargo test`で全テスト成功を確認
  - `cargo doc`で非推奨警告が正しく表示されることを確認
  - Gitコミット作成（"refactor: Transformをecs/transform/に隔離し非推奨化"）
  - _Requirements: 5, 9_

### 最終検証と統合

- [x] 4. リファクタリング完了の総合検証
- [x] 4.1 サンプルアプリケーションの実行確認
  - `cargo run --example simple_window`でシンプルなウィンドウ生成が正常動作することを確認 (✓)
  - `cargo run --example dcomp_demo`はECS未使用のためコンパイル確認のみ (✓)
  - 全サンプルが正常にビルド・実行可能
  - _Requirements: 9_

- [x] 4.2 API互換性の回帰テスト
  - 外部利用者が`use wintf::ecs::*;`で従来通りアクセス可能なことを確認 (✓)
  - `Rect`, `NodeQuery<L, G, M>`等の型エイリアスが正しく再エクスポートされていることを確認 (✓)
  - `BoxStyle`, `Arrangement`等のコンポーネントがbevy_ecsで正しく認識されることを確認 (✓)
  - 全テスト成功 (93 passed, 4 doctests) で互換性維持確認
  - _Requirements: 9_

- [x] 4.3 (P) 機能グループ分類のドキュメント更新
  - `.kiro/steering/structure.md`に5つの機能グループの責務と代表的なコンポーネント例を記載 (✓)
  - 各グループの目的、含まれるコンポーネントタイプ、命名規則（`XxxGraphics`, `XxxResource`）を明記 (✓)
  - _Requirements: 1, 10_

- [x] 4.4 (P) モジュールレベルドキュメントの追加
  - `layout/mod.rs`, `common/mod.rs`, `transform/mod.rs`にモジュールレベルdocコメント（`//!`）を追加 (✓)
  - 各モジュールの責務、含まれるコンポーネント概要、使用例を記載 (✓)
  - _Requirements: 10_

- [x] 4.5 最終コミットとリファクタリング完了
  - 全変更内容を確認し、Gitコミット作成（"docs: ECSコンポーネント機能グループのドキュメント更新"）
  - リファクタリング完了の確認（全10要件カバー、全テスト成功、サンプルアプリ動作確認完了）
  - _Requirements: 1, 2, 3, 4, 5, 6, 7, 8, 9, 10_

## Requirements Coverage Matrix

| Requirement | Covered by Tasks |
|-------------|------------------|
| 1. コンポーネント機能グループの定義 | 1.4, 2.8, 4.3, 4.5 |
| 2. ウィンドウ管理グループの分類 | 4.5（変更なし、現状維持の確認） |
| 3. グラフィックスリソースグループの分類 | 4.5（変更なし、現状維持の確認） |
| 4. レイアウトシステムグループの統合 | 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.7, 2.8, 2.9, 4.5 |
| 5. 変換行列の実験的配置と非推奨化 | 3.1, 3.2, 3.3, 3.4, 4.5 |
| 6. 共通インフラグループの分離 | 1.1, 1.2, 1.3, 1.4, 2.6, 4.5 |
| 7. ウィジェットグループの現状維持 | 4.5（変更なし、現状維持の確認） |
| 8. モジュール構造の一貫性 | 4.5（全Phase通して検証） |
| 9. リファクタリングの安全性保証 | 1.4, 1.5, 2.8, 2.9, 3.3, 3.4, 4.1, 4.2, 4.5 |
| 10. ドキュメント更新 | 4.3, 4.4, 4.5 |

## Implementation Notes

### Parallel Execution (P) マーカー
- `(P)`マークが付いたタスクは、データ依存や共有リソースの競合がなく、並列実行可能です
- Phase内の並列タスクは同時に着手できますが、Phaseをまたぐ並列実行は避けてください
- Phase 2のファイル分割タスク（2.2, 2.3, 2.4, 2.5）は並列実行可能ですが、2.6（systems.rs統合）は2.4（arrangement.rs）完了後に実施してください

### Task Sizing
- Phase 1: 0.5日（5タスク、各1-2時間）
- Phase 2: 3-4日（9タスク、ファイル分割が中心で各2-4時間）
- Phase 3: 0.5日（4タスク、各1-2時間）
- 最終検証: 0.5日（5タスク、各1時間）
- **合計**: 4.5-5.5日（23タスク）

### Risk Mitigation
- 各Phase完了時に`cargo check --all-targets`と`cargo test`で検証
- 各PhaseをGitコミットとして記録し、問題時はロールバック可能
- Phase 2が最もリスクが高いため、慎重な検証を実施
