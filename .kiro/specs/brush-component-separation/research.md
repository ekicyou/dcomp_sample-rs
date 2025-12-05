# Research & Design Decisions: brush-component-separation

## Summary
- **Feature**: `brush-component-separation`
- **Discovery Scope**: Extension（既存ECSパターンの拡張）
- **Key Findings**:
  1. Visual on_addフックが拡張ポイントとして適切（L247-280, components.rs）
  2. 既存ウィジェット（Rectangle/Label/Typewriter）のon_addパターンが統一されている
  3. スケジュール構造: Draw前にresolve_inherited_brushes配置が可能（PreRenderSurface推奨）

## Research Log

### 既存コンポーネントフックパターン
- **Context**: Brushes自動挿入のためのon_addフック実装方式の調査
- **Sources Consulted**: 
  - `ecs/graphics/components.rs` L247-280: on_visual_add実装
  - `ecs/widget/shapes/rectangle.rs` L79+: on_rectangle_add
  - `ecs/widget/text/label.rs` L45+: on_label_add
- **Findings**:
  - Visual on_addで他コンポーネント（Arrangement, VisualGraphics, SurfaceGraphics）を挿入済み
  - `world.get::<T>(entity).is_none()`パターンで重複挿入を防止
  - `world.commands().entity(entity).insert()`でdeferred insertion
- **Implications**: Brushes挿入も同様のパターンで実装可能

### スケジュール構造分析
- **Context**: resolve_inherited_brushesシステムの配置位置決定
- **Sources Consulted**: `ecs/world.rs` L70-280
- **Findings**:
  - 実行順序: Input → Update → PreLayout → Layout → PostLayout → UISetup → GraphicsSetup → Draw → PreRenderSurface → RenderSurface → Composition → CommitComposition
  - Draw直前のPreRenderSurfaceが継承解決に適切
  - 親子関係はLayoutフェーズで確定済み
- **Implications**: PreRenderSurfaceスケジュールでresolve_inherited_brushesを実行

### 親階層の参照方法
- **Context**: Brush::Inherit解決時の親エンティティ取得方法
- **Sources Consulted**: `tests/visual_hierarchy_sync_test.rs`, bevy_ecs hierarchy
- **Findings**:
  - `bevy_ecs::hierarchy::ChildOf`コンポーネントで親参照
  - `Query<&ChildOf>`で親エンティティ取得
  - 親がない場合（ルート）はデフォルト値を適用
- **Implications**: ChildOfを使用した親辿りロジックを実装

### 色定数の現状
- **Context**: 既存colors moduleの統合方針
- **Sources Consulted**: `shapes/rectangle.rs` L16-56
- **Findings**:
  - 6色定数定義: TRANSPARENT, BLACK, WHITE, RED, GREEN, BLUE
  - `Color`型エイリアスは`D2D1_COLOR_F`
  - rectangle.rs内のサブモジュールとして実装
- **Implications**: brushes.rsに移動し、Brush::XXXとして再定義

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| Visual Hook拡張 | on_visual_addでBrushes挿入 | 既存パターン踏襲、一貫性 | フック実行順序の考慮 | 採用 |
| Widget Hook個別 | 各ウィジェットon_addでBrushes挿入 | 柔軟性 | コード重複 | 不採用 |
| システム挿入 | 専用システムでBrushes配布 | 明示的 | 実行タイミング複雑 | 不採用 |

## Design Decisions

### Decision: Brush::Inherit解決タイミング
- **Context**: 静的解決（初回のみ）vs 動的解決（毎フレーム）
- **Alternatives Considered**:
  1. 毎フレーム親を辿る（動的）
  2. 初回描画時のみ解決し、Solid値に置換（静的）
- **Selected Approach**: 静的解決（Option 2）
- **Rationale**: 親変更追従は別仕様スコープ。シンプル実装優先
- **Trade-offs**: 親変更時の自動更新なし、明示的な再設定が必要
- **Follow-up**: 動的継承は需要が出てから別仕様で検討

### Decision: モジュール配置
- **Context**: brushes.rsの配置場所
- **Alternatives Considered**:
  1. `ecs/widget/brushes.rs`（論理コンポーネント）
  2. `ecs/graphics/brushes.rs`（GPUリソース的位置づけ）
- **Selected Approach**: `ecs/widget/brushes.rs`
- **Rationale**: Brush/BrushesはGPUリソースではなく論理コンポーネント。widgetレイヤーが適切
- **Trade-offs**: graphicsモジュールからのimportが必要
- **Follow-up**: なし

### Decision: SparseSetストレージ
- **Context**: Brushesコンポーネントのストレージ戦略
- **Alternatives Considered**:
  1. Table（デフォルト）
  2. SparseSet
- **Selected Approach**: SparseSet
- **Rationale**: 動的追加/削除が頻繁、Changed検出効率
- **Trade-offs**: メモリオーバーヘッドやや増
- **Follow-up**: 性能問題発生時にTable検討

## Risks & Mitigations
- 描画システム修正漏れ → Rustコンパイラが参照エラーで検出
- 親未確定時のInherit解決失敗 → デフォルト色（BLACK/TRANSPARENT）でフォールバック
- テスト不足 → 既存テストパターンを踏襲し網羅

## References
- [bevy_ecs Component Hooks](https://docs.rs/bevy_ecs/latest/bevy_ecs/component/trait.Component.html) - on_add/on_remove実装
- `doc/spec/01-ecs-components.md` - ECSコンポーネント設計原則
- `doc/spec/06-visual-directcomp.md` - Visual階層とDirectComposition統合
