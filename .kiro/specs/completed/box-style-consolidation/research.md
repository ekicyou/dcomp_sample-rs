# Research & Design Decisions

## Summary

- **Feature**: `box-style-consolidation`
- **Discovery Scope**: Extension（既存レイアウトシステムの内部リファクタリング）
- **Key Findings**:
  - taffyの`Style`構造体はコンテナー/アイテムを区別せず全プロパティをフラットに持つ設計
  - 現行の`build_taffy_styles_system`は8コンポーネントクエリ + 7条件Orフィルタで肥大化
  - bevy_ecsのアーキタイプ断片化は統合により改善される

## Research Log

### taffy::Style構造体の設計調査

- **Context**: `FlexContainer`と`FlexItem`を分離すべきか、統合すべきかの判断材料
- **Sources Consulted**: 
  - `target/doc/taffy/struct.Style.html`（ローカルドキュメント）
  - taffy 0.9.1 ソースコード
- **Findings**:
  - `Style`は39フィールドを持つフラットな構造体
  - コンテナー用プロパティ（`flex_direction`, `justify_content`, `align_items`）とアイテム用プロパティ（`flex_grow`, `flex_shrink`, `align_self`）が同一構造体に共存
  - CSSのFlexboxモデルと同様、同じ要素がコンテナーかつアイテムになりうる
- **Implications**: wintfの`BoxStyle`も同様にフラット構造で全プロパティを統合すべき

### bevy_ecsクエリパフォーマンス

- **Context**: 8コンポーネントクエリの性能影響評価
- **Sources Consulted**: bevy_ecs 0.17.2ドキュメント、既存コードベース分析
- **Findings**:
  - クエリは参照するコンポーネント数に比例してアーキタイプ走査コストが増加
  - 現行実装は`Or<(Changed<BoxSize>, ...)>`で7条件フィルタ
  - 統合により1コンポーネント + `Changed<BoxStyle>`のシンプルなクエリに削減可能
- **Implications**: クエリ複雑度削減は性能改善に直結

### 既存コードパターン分析

- **Context**: 統合後の設計が既存パターンと整合するか確認
- **Sources Consulted**: `high_level.rs`, `systems.rs`, `taffy_flex_demo.rs`
- **Findings**:
  - 既存型（`BoxSize`, `BoxMargin`等）は`#[derive(Component)]`で定義
  - `Dimension`, `LengthPercentageAuto`, `LengthPercentage`, `Rect<T>`は共通型として再利用
  - `From`/`Into`トレイトによるtaffy型への変換パターンが確立
  - サンプルコードでは`BoxSize`と`FlexContainer`/`FlexItem`を個別に付与
- **Implications**: 既存の共通型を維持し、統合後の`BoxStyle`でも再利用

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| 完全統合 | 全7コンポーネントを`BoxStyle`に統合 | クエリ最大簡略化（8→1）、taffy設計と整合 | 構造体サイズ増加 | **採用** |
| 部分統合 | Box系5種統合、Flex系2種独立維持 | 概念的分離維持 | クエリ削減効果が限定的（8→3） | taffyが区別しないため不採用 |
| 現状維持 | 変更なし | 移行コストゼロ | 問題継続 | 不採用 |

## Design Decisions

### Decision: 完全統合アプローチ

- **Context**: `build_taffy_styles_system`のクエリ肥大化を解消したい
- **Alternatives Considered**:
  1. 部分統合 — Box系5種のみ統合、Flex系は独立維持
  2. 完全統合 — 全7種を`BoxStyle`に統合
  3. 現状維持 — 変更なし
- **Selected Approach**: 完全統合
- **Rationale**: taffyの`Style`がコンテナー/アイテムを区別しないため、wintfも同様の設計とする
- **Trade-offs**: 
  - ✅ クエリ最大簡略化
  - ✅ taffy設計との1:1対応
  - ⚠️ 構造体サイズ増加（許容範囲）
- **Follow-up**: ビルダーパターンAPI提供の是非は実装フェーズで判断

### Decision: Flex系プロパティのフラット化

- **Context**: `FlexContainer`と`FlexItem`を`BoxStyle`にどう統合するか
- **Alternatives Considered**:
  1. ネスト維持 — `Option<FlexContainer>`, `Option<FlexItem>`フィールド
  2. フラット化 — 各プロパティを直接フィールドとして展開
- **Selected Approach**: フラット化
- **Rationale**: 
  - taffyの`Style`がフラット構造であるため変換が単純化
  - `FlexContainer`/`FlexItem`の概念的区別は使用パターンから判断可能
- **Trade-offs**:
  - ✅ taffy変換ロジック簡略化
  - ✅ フィールドアクセスが直接的
  - ⚠️ コンテナー/アイテム概念の明示性低下（ドキュメントで補完）

### Decision: 従来型の非コンポーネント化

- **Context**: `BoxSize`等の従来型をどう扱うか
- **Alternatives Considered**:
  1. 完全削除
  2. 非コンポーネント型として維持
- **Selected Approach**: 非コンポーネント型として維持
- **Rationale**: 
  - `From`/`Into`トレイトで移行パスを提供
  - 既存コードでの構造体リテラル記法を維持
  - コンパイルエラーで移行箇所を明示
- **Trade-offs**:
  - ✅ 移行が段階的に可能
  - ⚠️ 型が残存することによる混乱リスク（ドキュメントで明確化）

## Risks & Mitigations

- **テスト移行漏れ** — コンパイルエラーで検出可能（Component削除により自動検出）
- **API冗長性** — ビルダーパターンで軽減可能（将来対応）
- **将来のプロパティ追加** — `BoxStyle`にフィールド追加で対応

## References

- [taffy 0.9.1 Style documentation](target/doc/taffy/struct.Style.html) — taffyのStyle構造体設計
- [bevy_ecs 0.17.2](https://docs.rs/bevy_ecs/0.17.2) — ECSクエリパフォーマンス特性
- `crates/wintf/src/ecs/layout/high_level.rs` — 現行型定義
- `crates/wintf/src/ecs/layout/systems.rs` — 現行クエリ実装
