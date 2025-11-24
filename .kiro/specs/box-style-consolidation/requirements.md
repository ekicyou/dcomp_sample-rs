# Requirements Document

## Project Description (Input)

`BoxSize`、`BoxMargin`、`BoxPadding`など、`build_taffy_styles_system`に関わるクエリが巨大になってきて性能不安がある。本質的にレイアウト入力の論理コンポーネントは分離している意義があまりない。そのためコンポーネントを1つにまとめて`BoxStyle`にしてしまうほうがよいのではないか？各フィールドを`Option<BoxSize>`などにして、従来コンポーネントだった`BoxSize`などはコンポーネントでなくする。実装可否判断を含め検討せよ。

## Introduction

本仕様は、wintfライブラリのレイアウトシステムにおけるECSコンポーネント構造を最適化し、`build_taffy_styles_system`のクエリパフォーマンスを改善することを目的とする。現在、レイアウト入力用コンポーネント（`BoxSize`、`BoxMargin`、`BoxPadding`、`BoxPosition`、`BoxInset`、`FlexContainer`、`FlexItem`）が個別に定義されており、これらすべてを参照するクエリが肥大化している。本プロジェクトでは、これらを統合した単一の`BoxStyle`コンポーネントへの移行可否を検討・実装する。

## Requirements

### Requirement 1: コンポーネント統合設計

**Objective:** 開発者として、レイアウト入力コンポーネントを単一の`BoxStyle`コンポーネントに統合したい。これにより、システムクエリの複雑度を削減し、コードの保守性を向上させる。

#### Acceptance Criteria

1. レイアウトシステムは、`BoxSize`、`BoxMargin`、`BoxPadding`、`BoxPosition`、`BoxInset`の各フィールドを`Option`型として持つ単一の`BoxStyle`構造体を提供すること
2. `BoxStyle`構造体は、従来の個別コンポーネントと同等のデフォルト値を持つこと
3. `BoxStyle`を使用する場合、レイアウトシステムは1つのコンポーネントへのクエリのみで全レイアウト入力を取得できること

### Requirement 2: Flexレイアウト統合設計

**Objective:** 開発者として、Flexレイアウト関連コンポーネント（`FlexContainer`、`FlexItem`）の統合方針を決定したい。これにより、一貫したコンポーネント設計を実現する。

#### Acceptance Criteria

1. `FlexContainer`と`FlexItem`が`BoxStyle`に統合されるべきか、独立を維持すべきかの判断基準が文書化されること
2. When `FlexContainer`と`FlexItem`を統合する場合、レイアウトシステムは`Option<FlexContainer>`および`Option<FlexItem>`フィールドとして`BoxStyle`に含めること
3. When `FlexContainer`と`FlexItem`を独立維持する場合、レイアウトシステムはこれらを別コンポーネントとして参照する合理的理由を持つこと

### Requirement 3: クエリパフォーマンス改善

**Objective:** 開発者として、`build_taffy_styles_system`のクエリパフォーマンスを改善したい。これにより、多数のエンティティを持つシーンでのレイアウト計算が高速化される。

#### Acceptance Criteria

1. 統合後の`build_taffy_styles_system`は、現行の8コンポーネントクエリより少ないコンポーネント参照数でスタイル構築を実行できること
2. While 大量のエンティティ（1000以上）が存在する場合、レイアウトシステムはアーキタイプ断片化の影響を最小化すること
3. If 変更検出（`Changed<T>`）が必要な場合、レイアウトシステムは統合コンポーネント全体ではなく実際に変更されたフィールドのみを検出する手段を提供すること

### Requirement 4: API互換性とマイグレーション

**Objective:** ライブラリ利用者として、既存コードからの移行パスを提供してほしい。これにより、破壊的変更の影響を最小化できる。

#### Acceptance Criteria

1. レイアウトシステムは、従来の個別コンポーネント（`BoxSize`、`BoxMargin`等）を非コンポーネント型（通常の構造体）として維持すること
2. `BoxStyle`は、従来型からの変換（`From`/`Into`トレイト）を実装すること
3. When 旧APIから新APIへ移行する場合、レイアウトシステムはコンパイルエラーにより移行必要箇所を明示すること
4. レイアウトシステムは、テスト・サンプルコードを新APIに更新した移行例を提供すること

## Implementation Feasibility Assessment (実装可否判断)

**判定: ✅ 実装を推奨**

### 1. bevy_ecsにおけるコンポーネント統合のメリット・デメリット

| 観点 | メリット | デメリット |
|------|---------|-----------|
| クエリ複雑度 | 8コンポーネント参照 → 1〜3コンポーネント参照に削減 | - |
| コード保守性 | スタイル構築ロジックの重複削減（現在2箇所で同一処理） | 既存テスト・サンプルの移行コスト |
| API設計 | 一貫した構造体ベースの設計 | `BoxStyle { size: Some(BoxSize {...}), ... }` の冗長性 |
| 変更検出 | - | `Changed<BoxStyle>` が全フィールド変更で発火 |

### 2. アーキタイプシステムへの影響評価

**現状の問題**: エンティティごとに異なるコンポーネント組み合わせが存在し、アーキタイプが断片化

**統合後の改善**: `BoxStyle` 1つに統一することで、レイアウト対象エンティティは同一アーキタイプに収束

**評価**: ✅ 統合はアーキタイプ断片化を**改善**する方向に働く

### 3. 変更検出（`Changed<T>`）の粒度トレードオフ

**現行実装の確認**: `build_taffy_styles_system` は変更検出時に**全フィールドを再構築**している

**結論**: 変更検出の粒度低下による実質的なパフォーマンス悪化は**ほぼない**

### 4. 推奨アプローチ

**部分統合**: Box系プロパティ（5種）を `BoxStyle` に統合、Flex系（2種）は独立維持

```rust
#[derive(Component, Default)]
pub struct BoxStyle {
    pub size: Option<BoxSize>,      // 非コンポーネント化
    pub margin: Option<BoxMargin>,  // 非コンポーネント化
    pub padding: Option<BoxPadding>,// 非コンポーネント化
    pub position: Option<BoxPosition>,
    pub inset: Option<BoxInset>,
}
// FlexContainer, FlexItem は独立コンポーネントとして維持
```

**理由**: Flexコンテナーと子アイテムは異なるエンティティに付与される設計上の分離が明確

### 5. 追加検討事項（ギャップ分析結果）

#### 影響範囲の詳細

| カテゴリ | ファイル数 | 主な変更内容 |
|---------|-----------|-------------|
| ソースコード | 2 | `high_level.rs`（型定義）、`systems.rs`（クエリ変更） |
| テスト | 5+ | `taffy_layout_integration_test.rs`、`taffy_flex_layout_pure_test.rs`、`taffy_advanced_test.rs`等 |
| サンプル | 1 | `taffy_flex_demo.rs` |
| ドキュメント | 1 | `layout/mod.rs`（docコメント更新） |

#### 未検討項目（設計フェーズで確定）

1. **`LayoutRoot`マーカーとの関係**: `BoxStyle`導入後も`LayoutRoot`は独立マーカーとして維持するか
2. **ビルダーパターン**: `BoxStyle::new().size(...).margin(...)`のようなビルダーAPI提供の是非
3. **デバッグ表示**: `Debug`トレイト実装の読みやすさ（ネストした`Option`の表示）
4. **シリアライズ**: 将来的な`serde`対応を考慮した構造設計

#### リスク評価

| リスク | 影響度 | 発生確率 | 対策 |
|--------|--------|----------|------|
| テスト移行漏れ | 中 | 低 | コンパイルエラーで検出可能 |
| API冗長性による利便性低下 | 低 | 中 | ビルダーパターンで軽減可能 |
| 将来のレイアウトプロパティ追加 | 低 | 中 | `BoxStyle`にフィールド追加で対応 |

### 6. 判断確定

上記検討の結果、以下を確定事項とする：

- ✅ **実装を推奨**
- ✅ **部分統合アプローチ**を採用（Box系5種統合、Flex系2種独立維持）
- ✅ **移行コスト**は許容範囲内（コンパイルエラーによる移行箇所明示）
- ⚠️ **未検討項目**は設計フェーズで詳細化

## Technical Notes

### 現行アーキテクチャ

```
現在のクエリ構造（build_taffy_styles_system）:
- Option<&BoxSize>
- Option<&BoxMargin>
- Option<&BoxPadding>
- Option<&BoxPosition>
- Option<&BoxInset>
- Option<&FlexContainer>
- Option<&FlexItem>
- &mut TaffyStyle

フィルタ条件: Or<(Changed<BoxSize>, Changed<BoxMargin>, ...)>
```

### 検討すべき代替案

1. **コンポーネントバンドル**: `#[derive(Bundle)]`を使用した論理グループ化
2. **階層的コンポーネント**: `BoxStyle`を親として、詳細を子コンポーネントに分離
3. **クエリキャッシュ最適化**: bevy_ecsの内部最適化に依存
4. **現状維持**: パフォーマンス影響が許容範囲内であれば変更不要
