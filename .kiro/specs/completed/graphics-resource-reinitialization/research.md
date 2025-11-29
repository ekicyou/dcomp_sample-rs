# Research & Design Decisions

---
**Purpose**: GraphicsCore再初期化機能の設計判断と技術調査の記録

**Usage**:
- 要件フェーズで実施した検証テストの結果を設計に反映
- Bevy ECS 0.17.2のスケジューリング最適化パターンを文書化
- 2マーカー戦略の技術的根拠を記録
---

## Summary
- **Feature**: `graphics-resource-reinitialization`
- **Discovery Scope**: Extension（既存ECSシステムへの統合）
- **Key Findings**:
  - Bevy ECS 0.17.2のArchetype-levelクエリ最適化を活用した並列実行戦略
  - Option<T>ラップパターンによる状態管理の実装方式
  - 2マーカーコンポーネント（HasGraphicsResources + GraphicsNeedsInit）による拡張性とパフォーマンスの両立

## Research Log

### Bevy ECS Resource削除検出の制約
- **Context**: GraphicsCore破棄をどのように検出し、依存コンポーネントに伝播させるか
- **Sources Consulted**: 
  - Bevy ECS 0.17.2 公式ドキュメント
  - `tests/resource_removal_detection_test.rs`（6テスト合格）による実証
- **Findings**:
  - `RemovedComponents<T>`はComponentのみ対応、Resourceには使用不可
  - Resourceの削除検出にはポーリング方式（`Option<Res<GraphicsCore>>`チェック）が必要
  - 毎フレーム実行で1フレームの遅延を許容する設計が妥当
- **Implications**: 
  - GraphicsCore自体も`Option<T>`でラップし、破棄時はNone設定
  - 破棄検出システムは専用システムとして毎フレーム実行
  - 初期化システムがNoneを検出して自動再初期化

### 状態管理パターンの比較検証
- **Context**: コンポーネント無効化と再初期化をECSでどう表現するか
- **Sources Consulted**:
  - `tests/component_state_pattern_test.rs`（6テスト合格）
  - Bevy ECS Changed<T>検出機構の動作検証
- **Findings**:
  - **パターンA**: Optionラップ + 単一マーカー（GraphicsNeedsInit）
  - **パターンB**: 複数個別マーカー（WindowGraphicsNeedsInit, VisualNeedsInit, SurfaceNeedsInit）
  - **パターンC**: 2マーカー（HasGraphicsResources + GraphicsNeedsInit）
  - Changed<T>は同一フレーム内で複数システムが反応可能（検証済み）
  - Changed<T>はコンポーネント削除を検出しない（With/Without<T>必須）
- **Implications**: 
  - パターンCを採用：静的マーカー（リソース使用宣言）+ 動的マーカー（初期化状態）
  - With/Without<T>フィルタで初期化対象と描画対象を明確に分離
  - 世代番号（generation）フィールドで初期化回数を追跡

### 遅延初期化パターンとスケジューリング最適化
- **Context**: ECSスケジューリングでの並列実行を最大化したい
- **Sources Consulted**:
  - `tests/lazy_reinit_pattern_test.rs`（5テスト合格）
  - Bevy ECS Archetype-levelクエリフィルタリングの仕様
- **Findings**:
  - `get_or_init(&mut self)`による遅延初期化は全システムで&mut Tアクセス必須
  - &mut T要求により並列実行が阻害される（直列化）
  - With<GraphicsNeedsInit>による明示的マーキングで対象を限定
  - Query<&mut T, With<M>>とQuery<&T, Without<M>>は並列実行可能
  - Archetype-level最適化により、マーカー有無でクエリイテレーション回数が削減
- **Implications**:
  - 初期化システム：`Query<&mut T, With<GraphicsNeedsInit>>`（可変アクセス）
  - 参照システム：`Query<&T, Without<GraphicsNeedsInit>>`（読み取り専用）
  - クリーンアップシステムで初期化完了後にマーカー削除
  - Bevy ECSが自動的に並列スケジューリングを最適化

### 拡張性と新Widget対応
- **Context**: 将来的にButtonGraphics、TextGraphicsなど新しいWidgetグラフィックスコンポーネントが追加される
- **Sources Consulted**: 既存コードベースのWidget実装パターン分析
- **Findings**:
  - 現在: WindowGraphics、Visual、Surface（将来的にVisualGraphics、SurfaceGraphicsに改名予定）
  - 新Widget追加時、各初期化システムに個別対応を追加すると保守コストが増大
  - HasGraphicsResourcesマーカーによる一括検出で、システム変更不要
  - `Query<Entity, With<HasGraphicsResources>>`で全対象エンティティを自動検出
- **Implications**:
  - HasGraphicsResources: spawn時に付与、永続的（静的マーカー）
  - GraphicsNeedsInit: 初期化必要時に追加、完了時に削除（動的マーカー）
  - 新Widget追加時、spawn時にHasGraphicsResourcesマーカーを付与するだけで自動統合

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| 単一マーカー（GraphicsNeedsInit） | 初期化必要時のみマーカー追加 | シンプル、既存パターンに近い | Widget追加時にOr<>フィルタ更新必須 | 拡張性に課題 |
| 個別マーカー（XXXNeedsInit） | コンポーネント毎にマーカー | 細かい制御が可能 | マーカー数が増大、管理コスト高 | 却下：複雑性増大 |
| 2マーカー（HasGraphicsResources + GraphicsNeedsInit） | 静的宣言 + 動的状態 | 拡張性とパフォーマンス両立 | マーカー2つの運用が必要 | **採用**：Archetype最適化活用 |

## Design Decisions

### Decision: Option<T>ラップによる無効化表現
- **Context**: GraphicsCoreやコンポーネントが無効化されたことをどう表現するか
- **Alternatives Considered**:
  1. boolフラグ（`is_valid: bool`） — 冗長、Rustのidiomatic patternから外れる
  2. Option<T>ラップ — Rust標準、型安全
- **Selected Approach**: 全グラフィックスリソース（GraphicsCore、WindowGraphics、Visual、Surface）の内部データをOption<T>でラップ
- **Rationale**: 
  - Rust言語のOption<T>は「値がない」を型安全に表現
  - `invalidate()`メソッドで内部をNoneに設定
  - `is_valid()`メソッドでSome/Noneチェック
- **Trade-offs**: 
  - 利点：型安全、メモリ効率（None時はスマートポインタ解放）
  - 欠点：アクセス時にunwrap/matchが必要（ただし、is_valid()で事前チェック）
- **Follow-up**: 実装時にunwrap()ではなくmatch/if-letパターンでエラーハンドリング

### Decision: 2マーカー戦略（HasGraphicsResources + GraphicsNeedsInit）
- **Context**: 新Widget追加時にシステム変更なしで自動統合したい
- **Alternatives Considered**:
  1. 単一マーカー（GraphicsNeedsInit）のみ — Widget追加時にOr<>フィルタ更新必須
  2. 個別マーカー（XXXNeedsInit） — マーカー数が増大
  3. 2マーカー（HasGraphicsResources + GraphicsNeedsInit） — 静的宣言 + 動的状態
- **Selected Approach**: 2マーカー戦略
  - HasGraphicsResources: グラフィックスリソース使用を宣言（永続的）
  - GraphicsNeedsInit: 初期化が必要な状態を示す（一時的）
- **Rationale**:
  - HasGraphicsResourcesで対象エンティティを一括検出
  - GraphicsNeedsInitで初期化状態を管理
  - Bevy ECS Archetype-levelクエリ最適化を活用
  - Widget追加時、spawn時にHasGraphicsResourcesを付与するだけ
- **Trade-offs**:
  - 利点：拡張性、パフォーマンス（Archetype最適化）、保守性
  - 欠点：マーカー2種類の運用が必要
- **Follow-up**: 実装時にマーカー付与タイミングを明確化（HasGraphicsResources: spawn時、GraphicsNeedsInit: 初期化必要時）

### Decision: GraphicsCore初期化システムによる一括マーキング
- **Context**: GraphicsCore初期化/再初期化時に全依存エンティティを効率的にマークしたい
- **Alternatives Considered**:
  1. 個別無効化システム — 各コンポーネントを個別に無効化
  2. GraphicsCore初期化システムで一括マーキング — 初期化完了時に全HasGraphicsResourcesへ一括追加
- **Selected Approach**: init_graphics_coreシステムが初期化完了時に`Query<Entity, With<HasGraphicsResources>>`で全エンティティを取得し、`Commands::entity(entity).insert(GraphicsNeedsInit)`で一括マーキング
- **Rationale**:
  - 初期化トリガーを一箇所に集約
  - HasGraphicsResourcesマーカーで対象を自動検出
  - 新Widget追加時もシステム変更不要
- **Trade-offs**:
  - 利点：一元管理、拡張性、シンプル
  - 欠点：全エンティティに一括マーキング（パフォーマンス懸念は小）
- **Follow-up**: 実装時にapply_deferred()のタイミングを明確化

### Decision: クリーンアップシステムによるマーカー削除
- **Context**: GraphicsNeedsInitマーカーをいつ削除するか
- **Alternatives Considered**:
  1. 各初期化システムで自己削除 — 個別判断が必要
  2. 専用クリーンアップシステム — 一元管理
- **Selected Approach**: cleanup_graphics_needs_initシステムが全初期化システム後に実行され、WindowGraphics・Visual・Surfaceすべてが有効な場合のみマーカー削除
- **Rationale**:
  - 初期化完了判定を一箇所に集約
  - 各初期化システムは自己の初期化のみに専念
  - 依存関係チェーンの完了を確実に判定
- **Trade-offs**:
  - 利点：一元管理、依存関係保証
  - 欠点：クリーンアップシステムが全コンポーネントをチェック（拡張時に更新必要）
- **Follow-up**: 将来的にOption<&T>パターンで拡張性向上を検討

## Risks & Mitigations
- **リスク1**: 新Widget追加時にcleanup_graphics_needs_initシステムの更新を忘れる
  - **緩和策**: テストケースで新Widgetのマーカー削除を検証、ドキュメントに明記
- **リスク2**: apply_deferred()のタイミングミスでマーカー反映が遅延
  - **緩和策**: システム実行順序を明確化（.after()依存関係）、統合テストで検証
- **リスク3**: GraphicsCore初期化失敗時の無限ループ
  - **緩和策**: エラーログ出力、内部None維持で次フレーム再試行（回数制限は将来検討）

## References
- [Bevy ECS 0.17.2 公式ドキュメント](https://docs.rs/bevy_ecs/0.17.2/bevy_ecs/) — ECSスケジューリングとクエリフィルタリング
- `tests/resource_removal_detection_test.rs` — Resource削除検出パターン検証（6テスト合格）
- `tests/component_state_pattern_test.rs` — 状態管理パターン比較（6テスト合格）
- `tests/lazy_reinit_pattern_test.rs` — 遅延初期化パターン検証（5テスト合格）
- 既存実装: `crates/wintf/src/ecs/graphics/` — WindowGraphics、Visual、Surfaceの現在の実装
- 既存実装: `crates/wintf/src/ecs/world.rs` — PostLayoutスケジュールとシステム実行順序
