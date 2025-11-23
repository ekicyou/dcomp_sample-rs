# Gap Analysis: taffy-layout-integration

## Analysis Summary

- **スコープ**: 既存の`BoxStyle`/`BoxComputedLayout`をtaffyレイアウトエンジンと完全統合し、高レベルECSコンポーネント（`BoxSize`、`BoxMargin`、`BoxPadding`、`FlexContainer`、`FlexItem`）による宣言的レイアウトを実現
- **既存資産の活用**: `Arrangement`伝播システム、Common Infrastructure（`sync_simple_transforms`、`propagate_parent_transforms`）、`ChildOf`階層管理は完全に再利用可能
- **主要な拡張領域**: 
  - 高レベルコンポーネント（5個）の新規作成
  - `TaffyStyle`への変換システム（ECS Changed<T>クエリベース）
  - taffyツリー構築と計算結果の`Arrangement`への反映システム
- **統合の複雑性**: 中程度 - 既存パターンを踏襲し、taffyをブラックボックス化することで境界を明確化

## 1. Current State Investigation

### 既存のレイアウト関連コンポーネント

**既に実装されている基盤**:

1. **`BoxStyle` / `BoxComputedLayout`** (`crates/wintf/src/ecs/layout/taffy.rs`)
   - `BoxStyle(pub Style)`: taffy::Styleの薄いラッパー（`#[derive(Component)]`）
   - `BoxComputedLayout(pub Layout)`: taffy::Layoutの薄いラッパー
   - 現状: 直接公開されているが、実質的に使用されていない
   - 今後: `TaffyStyle`/`TaffyComputedLayout`に名称変更し、内部実装として隠蔽

2. **`Arrangement` / `GlobalArrangement`** (`crates/wintf/src/ecs/layout/arrangement.rs`)
   - `Arrangement`: ローカル配置（`offset`, `scale`, `size`）
   - `GlobalArrangement`: 累積変換行列とバウンディングボックス
   - `ArrangementTreeChanged`: ダーティビット伝播マーカー
   - 現状: 手動設定（`simple_window.rs`で直接インスタンス化）
   - 今後: taffyの計算結果から自動生成される

3. **`Size` / `Offset` / `LayoutScale`** (`crates/wintf/src/ecs/layout/metrics.rs`)
   - 既存のメトリクス型（そのまま再利用可能）

### 既存の階層伝播システム

**完全に再利用可能なインフラ**:

1. **Common Infrastructure** (`crates/wintf/src/ecs/common/tree_system.rs`)
   - `sync_simple_transforms<L, G, M>()`: ルートエンティティの変換同期
   - `propagate_parent_transforms<L, G, M>()`: 親→子への伝播
   - `mark_dirty_trees<L, G, M>()`: ダーティビット伝播
   - 完全ジェネリック化されており、`Arrangement`型で既に活用中

2. **Layout Systems** (`crates/wintf/src/ecs/layout/systems.rs`)
   - `sync_simple_arrangements()`: Common Infrastructureを活用
   - `propagate_global_arrangements()`: Common Infrastructureを活用
   - `mark_dirty_arrangement_trees()`: Common Infrastructureを活用
   - これらはそのまま機能し続ける（taffyが`Arrangement`を生成するフローに変わるだけ）

3. **階層管理** (`bevy_ecs::hierarchy`)
   - `ChildOf`: 親子関係の定義
   - `Children`: 子エンティティリスト
   - taffyツリー構築時にこの情報を読み取る

### 既存のコンポーネントパターン

**統一されたECSコンポーネント設計**:

- `#[derive(Component, Debug, Clone, ...)]`属性の使用
- `#[component(on_add = ..., on_remove = ...)]`フックの活用例（`Arrangement::on_arrangement_add`）
- `Changed<T>`クエリによる増分更新パターン（`draw_rectangles`システム等）
- `Or<(Changed<X>, Added<Y>)>`による初回＋変更検知
- `Without<T>`による条件フィルタリング

### モジュール構成と公開API

**レイアウトモジュールの現状** (`crates/wintf/src/ecs/layout/mod.rs`):

```rust
pub mod arrangement;
pub mod metrics;
pub mod rect;
pub mod systems;
pub mod taffy;

pub use arrangement::*;
pub use metrics::*;
pub use rect::*;
pub use systems::*;
pub use taffy::*;  // ← 現在、BoxStyle/BoxComputedLayoutを直接公開
```

**今後の変更**:
- `taffy`モジュール: `TaffyStyle`/`TaffyComputedLayout`を内部実装とし、pub useから除外
- 新規`components`モジュール: 高レベルコンポーネント（`BoxSize`等）を公開

### テスト環境

**サンプルアプリケーション** (`simple_window.rs`):
- 現状: 手動で`Arrangement`を設定（階層4レベル、6個のRectangle + 2個のLabel）
- 今後: 高レベルコンポーネントによる宣言的記述に移行
- 検証に最適な複雑度（階層構造、複数ウィジェット、縦書きテキスト）

### 依存関係

**taffyクレート** (`Cargo.toml`):
- バージョン: 0.9.1（ワークスペース依存関係で定義）
- 既に`Cargo.toml`に含まれており、追加インストール不要
- 主要API: `Taffy`、`Style`、`Layout`、`Dimension`、`Rect`、`FlexDirection`等

## 2. Requirements Feasibility Analysis

### 技術的要求事項の整理

**Requirement 1: コンポーネント名称変更**
- 実装: 単純なリファクタリング（ファイル編集、grepによる置換）
- 依存: `ecs/layout/taffy.rs`、`ecs/layout/mod.rs`、`ecs/transform/mod.rs`のドキュメンテーション
- 複雑度: 低

**Requirement 2: TaffyStyleコンポーネント構造**
- 実装: `#[repr(transparent)]`属性の追加、トレイト実装（`Deref`、`DerefMut`等）
- 隠蔽: `pub use taffy::*`を削除し、内部実装として扱う
- 複雑度: 低

**Requirement 3: 高レベルレイアウトコンポーネント**
- 実装: 5つの新規structを定義（`BoxSize`、`BoxMargin`、`BoxPadding`、`FlexContainer`、`FlexItem`）
- re-export: taffy型（`Dimension`、`Rect<T>`、`FlexDirection`、`JustifyContent`、`AlignItems`、`AlignSelf`）をwintf公開APIでre-export
- 複雑度: 中（適切なフィールド設計、デフォルト値の定義、ドキュメンテーション）

**Requirement 4: Taffyレイアウト計算システム**
- 実装: 
  - ECS階層（`ChildOf`）→taffyツリー（`add_child`）の同期システム
  - `set_style()`による変更通知
  - `compute_layout()`による計算実行
  - `layout()`による結果取得と`TaffyComputedLayout`更新
- ブラックボックス化: taffyの内部実装（`Cache`、`mark_dirty`）に依存しない設計
- 複雑度: 高（ツリー同期ロジック、エンティティ↔ノードIDマッピング）

**Requirement 5: Arrangement更新システム**
- 実装: `TaffyComputedLayout` → `Arrangement`変換システム
- 統合: 既存の`propagate_global_arrangements`と連携
- 複雑度: 低（座標系変換のみ）

**Requirement 6: 増分レイアウト計算**
- 実装: 
  - `Changed<BoxSize>`、`Changed<BoxMargin>`等のクエリによる変更検知
  - 変更されたエンティティのみ`set_style()`を呼び出す
  - 変更がない場合、`compute_layout()`をスキップ
- taffy依存: taffyの`mark_dirty()`と`Cache`機構に全面的に依存
- 複雑度: 中（ECS Changed<T>パターンは既存だが、taffy統合が必要）

**Requirement 7: Taffyレイアウトインフラストラクチャ**
- 実装:
  - `Taffy`インスタンスをECSリソース（`Resource`）として管理
  - エンティティ↔taffyノードIDのマッピング（実装方法は設計フェーズで決定）
  - システムセット定義（実行順序制御）
  - エンティティ削除時のtaffyノード削除（`on_remove`フック）
- 複雑度: 高（リソース管理、ライフサイクル同期、デバッグ機能）

**Requirement 8: ビルドおよび動作検証**
- 実装: `simple_window.rs`を高レベルコンポーネントに移行
- 検証: ビジュアル結果の一致、動的変更の反映
- 複雑度: 低（検証作業のみ）

### ギャップと制約

**Missing Capabilities**:
1. **高レベルコンポーネントの定義**: 5つのコンポーネント（`BoxSize`等）は新規作成が必要
2. **TaffyStyle変換システム**: 高レベルコンポーネント→`TaffyStyle`の変換ロジック（新規ECSシステム）
3. **Taffyツリー構築システム**: ECS階層→taffyツリーの同期（新規ECSシステム）
4. **エンティティ↔ノードIDマッピング**: 効率的な双方向検索構造（実装方法は設計フェーズで決定）
5. **ルートウィンドウサイズ変更検知**: ウィンドウリサイズ→taffy再計算トリガー（新規システム）

**Unknowns (Research Needed)**:
1. **エンティティ↔ノードIDマッピングの最適実装**: 
   - Option A: `HashMap<Entity, NodeId>`をリソースとして保持
   - Option B: エンティティにノードIDをコンポーネントとして追加（`TaffyNode(NodeId)`）
   - 決定基準: 検索効率、メモリ使用量、デバッグ可能性
2. **ルートノードの決定方法**: 
   - `Window`コンポーネントを持つエンティティをルートとして扱う？
   - 複数ウィンドウの場合、それぞれ独立したtaffyツリーを構築？
3. **エンティティ削除時のクリーンアップ**: 
   - `on_remove`フックで`Taffy.remove(node_id)`を呼び出す？
   - 孤立ノードの検出と削除タイミング？

**Constraints**:
1. **軸平行変換のみ**: taffyは回転・スキューをサポートしないため、既存の制約を継承
2. **Flexboxレイアウトのみ**: Grid、Tableレイアウトは将来の拡張に備えて設計を拡張可能にするが、今回は実装しない
3. **既存のArrangement伝播システムとの互換性**: `propagate_global_arrangements`等は変更せず、taffyが`Arrangement`を生成するフローに変わるだけ

## 3. Implementation Approach Options

### Option A: 段階的統合（Extend + New）

**戦略**: 既存の`Arrangement`システムを維持しながら、taffyレイアウトを段階的に統合

**Phase 1: 基盤構築**
- `BoxStyle`→`TaffyStyle`の名称変更
- 高レベルコンポーネント（5個）の定義
- taffy型のre-export
- **拡張するファイル**: `ecs/layout/taffy.rs`
- **新規ファイル**: `ecs/layout/components.rs`（高レベルコンポーネント）

**Phase 2: 変換システム**
- 高レベルコンポーネント→`TaffyStyle`変換システム
- `Changed<T>`クエリによる増分更新
- **新規ファイル**: `ecs/layout/taffy_conversion.rs`

**Phase 3: ツリー統合**
- ECS階層→taffyツリー同期システム
- `Taffy`リソース管理
- エンティティ↔ノードIDマッピング
- **新規ファイル**: `ecs/layout/taffy_tree.rs`

**Phase 4: Arrangement更新**
- `TaffyComputedLayout`→`Arrangement`変換システム
- 既存の伝播システムと連携
- **拡張するファイル**: `ecs/layout/systems.rs`

**Phase 5: 検証と移行**
- `simple_window.rs`の移行
- テスト実行と調整
- **拡張するファイル**: `examples/simple_window.rs`

**Trade-offs**:
- ✅ 段階的な実装と検証が可能（リスク低減）
- ✅ 既存システムを壊さずに並行開発
- ✅ 各フェーズで部分的な動作確認が可能
- ❌ 移行期間中、手動`Arrangement`とtaffy自動計算が混在
- ❌ 完全統合まで時間がかかる

### Option B: 一括リファクタリング（New + Replace）

**戦略**: taffyレイアウトを完全実装してから、既存の手動`Arrangement`設定を一括置換

**Phase 1: 完全実装**
- 名称変更、高レベルコンポーネント、変換システム、ツリー同期をすべて実装
- **新規ファイル**: `ecs/layout/components.rs`、`ecs/layout/taffy_conversion.rs`、`ecs/layout/taffy_tree.rs`
- **拡張するファイル**: `ecs/layout/taffy.rs`、`ecs/layout/systems.rs`、`ecs/layout/mod.rs`

**Phase 2: 一括移行**
- `simple_window.rs`を高レベルコンポーネントに移行
- すべてのテストを実行
- **拡張するファイル**: `examples/simple_window.rs`

**Trade-offs**:
- ✅ 実装期間が短い（並行開発なし）
- ✅ 移行後、システム全体が統一された状態
- ❌ 実装完了までテストができない（高リスク）
- ❌ デバッグが困難（複数システムを同時に検証）

### Option C: Hybrid - 段階的実装 + 機能フラグ（推奨）

**戦略**: Option Aの段階的実装に、オプション機能フラグを組み合わせる

**Phase 1: 基盤構築（手動Arrangementと並行動作）**
- 名称変更、高レベルコンポーネント、taffy型re-export
- **新規ファイル**: `ecs/layout/components.rs`
- **拡張するファイル**: `ecs/layout/taffy.rs`、`ecs/layout/mod.rs`

**Phase 2: 変換システム（デバッグ出力あり）**
- 高レベルコンポーネント→`TaffyStyle`変換
- 変換結果をログ出力（検証用）
- **新規ファイル**: `ecs/layout/taffy_conversion.rs`

**Phase 3: ツリー統合（並行実行）**
- taffyツリー構築と計算
- `TaffyComputedLayout`を更新するが、まだ`Arrangement`に反映しない
- 計算結果をログ出力して手動設定と比較
- **新規ファイル**: `ecs/layout/taffy_tree.rs`

**Phase 4: Arrangement自動更新（フラグ制御）**
- `TaffyComputedLayout`→`Arrangement`変換を有効化
- 環境変数またはコンポーネントマーカーで手動/自動を切り替え
- **拡張するファイル**: `ecs/layout/systems.rs`

**Phase 5: 完全移行**
- 手動`Arrangement`設定を削除
- `simple_window.rs`を高レベルコンポーネントに移行
- **拡張するファイル**: `examples/simple_window.rs`

**Trade-offs**:
- ✅ 段階的な検証が可能（各フェーズで部分的に動作確認）
- ✅ デバッグ出力により、計算結果を既存の手動設定と比較可能
- ✅ 問題発見時、手動モードに戻せる（リスク低減）
- ✅ 移行期間中も既存機能が安定動作
- ❌ フラグ制御のロジックが必要（実装コスト増）
- ❌ 完全移行まで一定の期間が必要

## 4. Implementation Complexity & Risk

### Effort Estimation

**S (1-3 days)**:
- Requirement 1: コンポーネント名称変更（リファクタリングのみ）
- Requirement 2: `TaffyStyle`構造の整備（`#[repr(transparent)]`、トレイト実装）
- Requirement 8: ビルドおよび動作検証（`simple_window.rs`移行）

**M (3-7 days)**:
- Requirement 3: 高レベルコンポーネント定義（5個）+ re-export設定
- Requirement 5: `Arrangement`更新システム（座標変換ロジック）
- Requirement 6: 増分レイアウト計算（`Changed<T>`クエリ統合）

**L (1-2 weeks)**:
- Requirement 4: Taffyレイアウト計算システム（ツリー同期、`compute_layout`統合）
- Requirement 7: Taffyレイアウトインフラストラクチャ（リソース管理、マッピング、システムセット）

**Total Effort**: L (1-2 weeks)
- 理由: 複数の中規模タスクと1つの大規模タスクを含むが、既存パターンを活用できるため、実装自体は明確

### Risk Assessment

**High Risk**:
- **エンティティ↔ノードIDマッピングの設計**: 効率的な実装方法が確定していない（設計フェーズで決定）
- **taffyツリー同期の正確性**: ECS階層の追加・削除・移動を正しくtaffyに反映する必要がある
- **デバッグの困難性**: taffyの内部状態が不可視のため、レイアウト計算の問題を特定しにくい

**Medium Risk**:
- **taffy APIの理解**: `set_style()`、`compute_layout()`、`layout()`の正しい使用順序とタイミング
- **ルートウィンドウサイズ変更の検知**: ウィンドウリサイズイベント→taffy再計算のトリガー実装
- **既存の`Arrangement`システムとの統合**: `propagate_global_arrangements`との連携で予期しない動作が発生する可能性

**Low Risk**:
- **高レベルコンポーネントの定義**: 明確な仕様があり、既存パターンを踏襲
- **名称変更**: 単純なリファクタリング作業
- **`simple_window.rs`の移行**: サンプルコードの書き換えのみ

### Risk Mitigation Strategies

1. **段階的実装とログ出力**: 各フェーズで計算結果をログ出力し、既存の手動設定と比較
2. **デバッグ可視化**: taffy計算結果を画面に表示する機能（Requirement 7のオプション機能）
3. **マッピング実装の早期決定**: 設計フェーズでエンティティ↔ノードIDマッピングの実装方法を確定
4. **ユニットテスト**: ツリー同期ロジックのユニットテスト作成（小規模な階層構造で検証）

## 5. Recommendations for Design Phase

### 推奨アプローチ

**Option C: Hybrid - 段階的実装 + 機能フラグ**を推奨

**理由**:
- 既存の`Arrangement`システムを壊さずに開発可能
- 各フェーズで部分的な検証ができる（リスク低減）
- デバッグ出力により、taffy計算結果と手動設定を比較可能
- 問題発見時、手動モードに戻せる

### 設計フェーズで決定すべき事項

1. **エンティティ↔ノードIDマッピングの実装方法**
   - Option A: `HashMap<Entity, NodeId>`をリソースとして保持
   - Option B: エンティティにノードIDをコンポーネントとして追加（`TaffyNode(NodeId)`）
   - 評価基準: 検索効率（O(1) vs O(log n)）、メモリ使用量、デバッグ可能性、bevy_ecsのベストプラクティス

2. **ルートノードの決定方法**
   - `Window`コンポーネントを持つエンティティをルートとして扱う
   - 複数ウィンドウの場合、それぞれ独立したtaffyツリーを構築
   - `Taffy`インスタンスを各ウィンドウごとに保持するか、グローバルに1つだけ保持するか

3. **システム実行順序**
   - 高レベルコンポーネント変更検知 → `TaffyStyle`更新 → ツリー同期 → `compute_layout()` → `TaffyComputedLayout`更新 → `Arrangement`更新 → 既存の伝播システム
   - bevy_ecsの`SystemSet`を使用して実行順序を明示的に定義

4. **エンティティ削除時のクリーンアップ**
   - `on_remove`フックで`Taffy.remove(node_id)`を呼び出す
   - 孤立ノード（親が削除された子ノード）の検出と削除
   - `RemovedComponents`イテレータを使用した遅延削除

5. **デバッグ機能の設計**
   - taffy計算結果の可視化（バウンディングボックスの描画）
   - ログ出力の詳細度（通常モード vs デバッグモード）
   - 環境変数またはビルド時フィーチャーフラグによる制御

### 研究項目（Research Items）

1. **taffy v0.9.1のAPIベストプラクティス**
   - `set_style()`の呼び出しタイミング（毎フレーム vs 変更時のみ）
   - `compute_layout()`のパフォーマンス特性（大規模ツリーでの挙動）
   - エラーハンドリング（`Result`型の扱い）

2. **bevy_ecsの階層管理のベストプラクティス**
   - `ChildOf`の追加・削除・移動パターン
   - `Children`イテレータの効率的な使用方法
   - 階層構造の変更検知（`Changed<ChildOf>`の動作）

3. **他のtaffy統合例の調査**
   - bevy_uiやTauriのtaffy統合方法
   - エンティティ↔ノードIDマッピングの実装パターン
   - パフォーマンス最適化のテクニック

## Document Status

このギャップ分析は、`.kiro/settings/rules/gap-analysis.md`のフレームワークに従って実施されました。

**分析手法**:
- 既存コードベースの調査（grep検索、ファイル読み取り）
- 要件定義（`requirements.md`）との照合
- プロジェクト構造（`steering/`）とのアライメント確認

**信頼性**:
- 既存パターンと命名規則を完全に把握
- taffy v0.9.1の依存関係を確認
- Common Infrastructureの再利用可能性を検証

**未確定事項**:
- エンティティ↔ノードIDマッピングの実装方法（設計フェーズで決定）
- ルートノードの決定ロジック（設計フェーズで決定）
- デバッグ機能の詳細設計（設計フェーズで決定）

## Next Steps

ギャップ分析が完了しました。次のステップに進んでください：

```bash
# 設計フェーズへ進む（手動承認）
/kiro-spec-design taffy-layout-integration

# または、要件を自動承認して設計フェーズへ（高速トラック）
/kiro-spec-design taffy-layout-integration -y
```

設計フェーズでは、以下を決定します：
1. エンティティ↔ノードIDマッピングの実装方法
2. システム実行順序とシステムセット定義
3. ルートノード決定ロジックと複数ウィンドウ対応
4. エンティティ削除時のクリーンアップ戦略
5. デバッグ機能の詳細設計
