# Requirements Document

## Project Description (Input)

`BoxSize`、`BoxMargin`、`BoxPadding`など、`build_taffy_styles_system`に関わるクエリが巨大になってきて性能不安がある。本質的にレイアウト入力の論理コンポーネントは分離している意義があまりない。そのためコンポーネントを1つにまとめて`BoxStyle`にしてしまうほうがよいのではないか？各フィールドを`Option<BoxSize>`などにして、従来コンポーネントだった`BoxSize`などはコンポーネントでなくする。実装可否判断を含め検討せよ。

## Introduction

本仕様は、wintfライブラリのレイアウトシステムにおけるECSコンポーネント構造を最適化し、`build_taffy_styles_system`のクエリパフォーマンスを改善することを目的とする。現在、レイアウト入力用コンポーネント（`BoxSize`、`BoxMargin`、`BoxPadding`、`BoxPosition`、`BoxInset`、`FlexContainer`、`FlexItem`）が個別に定義されており、これらすべてを参照するクエリが肥大化している。

**設計方針**: `BoxStyle`は内部の`TaffyStyle`に対応するユーザー向け高レベルAPIとして位置づける。taffyの`Style`構造体がコンテナー/アイテムを区別せずフラットに全プロパティを持つ設計であることから、`BoxStyle`も同様に全レイアウトプロパティを統合する。

## Requirements

### Requirement 1: コンポーネント完全統合設計

**Objective:** 開発者として、全レイアウト入力コンポーネントを単一の`BoxStyle`コンポーネントに統合したい。これにより、システムクエリの複雑度を削減し、`TaffyStyle`との1:1対応を実現する。

#### Acceptance Criteria

1. レイアウトシステムは、Box系（`BoxSize`、`BoxMargin`、`BoxPadding`、`BoxPosition`、`BoxInset`）およびFlex系（`FlexContainer`、`FlexItem`相当）の全フィールドを持つ単一の`BoxStyle`構造体を提供すること
2. `BoxStyle`構造体は`TaffyStyle`と1:1で対応し、相互変換が可能であること
3. `BoxStyle`を使用する場合、レイアウトシステムは1つのコンポーネントへのクエリのみで全レイアウト入力を取得できること
4. Flex関連プロパティ（`flex_direction`、`justify_content`、`align_items`、`flex_grow`、`flex_shrink`、`flex_basis`、`align_self`）は`BoxStyle`にフラットなフィールドとして含めること

### Requirement 2: クエリパフォーマンス改善

**Objective:** 開発者として、`build_taffy_styles_system`のクエリパフォーマンスを改善したい。これにより、多数のエンティティを持つシーンでのレイアウト計算が高速化される。

#### Acceptance Criteria (Req2)

1. 統合後の`build_taffy_styles_system`は、現行の8コンポーネントクエリから1コンポーネント参照に削減すること
2. 大量のエンティティ（1000以上）が存在する場合、レイアウトシステムはアーキタイプ断片化の影響を最小化すること
3. 変更検出は`Changed<BoxStyle>`で統一し、現行実装と同様に全フィールド再構築を行うこと（粒度低下による実質的影響なし）
4. `LayoutRoot`は仮想デスクトップの正確な矩形情報（座標・サイズ）を`BoxStyle`として保持すること

### Requirement 3: API互換性とマイグレーション

**Objective:** ライブラリ利用者として、既存コードからの移行パスを提供してほしい。これにより、破壊的変更の影響を最小化できる。

#### Acceptance Criteria (Req3)

1. レイアウトシステムは、従来の個別コンポーネント（`BoxSize`、`BoxMargin`等）を非コンポーネント型（通常の構造体）として維持すること
2. `BoxStyle`は、従来型からの変換（`From`/`Into`トレイト）を実装すること
3. 旧APIから新APIへ移行する場合、レイアウトシステムはコンパイルエラーにより移行必要箇所を明示すること
4. レイアウトシステムは、テスト・サンプルコードを新APIに更新した移行例を提供すること

## Implementation Feasibility Assessment (実装可否判断)

### 判定

✅ **実装を推奨** - 完全統合アプローチ

### 1. taffyの設計調査結果

taffyの`Style`構造体はコンテナー/アイテムを区別せず、全プロパティをフラットに持つ設計：

```rust
// taffy::Style（抜粋）
pub struct Style {
    pub display: Display,
    pub flex_direction: FlexDirection,  // コンテナー用
    pub justify_content: Option<JustifyContent>,  // コンテナー用
    pub align_items: Option<AlignItems>,  // コンテナー用
    pub flex_grow: f32,   // アイテム用
    pub flex_shrink: f32, // アイテム用
    pub flex_basis: Dimension,  // アイテム用
    // ... 全プロパティがフラット
}
```

**結論**: `BoxStyle`も`TaffyStyle`と1:1対応させ、全プロパティを統合すべき

### 2. bevy_ecsにおけるコンポーネント統合のメリット・デメリット

| 観点 | メリット | デメリット |
|------|---------|-----------|
| クエリ複雑度 | 8コンポーネント参照 → 1コンポーネント参照に削減 | - |
| コード保守性 | スタイル構築ロジックの重複削減（現在2箇所で同一処理） | 既存テスト・サンプルの移行コスト |
| API設計 | `TaffyStyle`と1:1対応、一貫した構造体ベースの設計 | - |
| 変更検出 | 現行と同等（全フィールド再構築） | - |

### 3. アーキタイプシステムへの影響評価

**現状の問題**: エンティティごとに異なるコンポーネント組み合わせが存在し、アーキタイプが断片化

**統合後の改善**: `BoxStyle` 1つに統一することで、レイアウト対象エンティティは同一アーキタイプに収束

**評価**: ✅ 統合はアーキタイプ断片化を**改善**する方向に働く

### 4. 推奨アプローチ

**完全統合**: 全レイアウトプロパティ（Box系5種 + Flex系）を`BoxStyle`に統合

```rust
#[derive(Component, Default)]
pub struct BoxStyle {
    // Box系（ネスト構造維持）
    pub size: Option<BoxSize>,
    pub margin: Option<BoxMargin>,
    pub padding: Option<BoxPadding>,
    pub position: Option<BoxPosition>,
    pub inset: Option<BoxInset>,
    // Flex系（フラット化）
    pub flex_direction: Option<FlexDirection>,
    pub justify_content: Option<JustifyContent>,
    pub align_items: Option<AlignItems>,
    pub flex_grow: Option<f32>,
    pub flex_shrink: Option<f32>,
    pub flex_basis: Option<Dimension>,
    pub align_self: Option<AlignSelf>,
}
```

**理由**: taffyの`Style`がコンテナー/アイテムを区別しないため、wintfも同様の設計とする

### 5. 追加検討事項（ギャップ分析結果）

#### 影響範囲の詳細

| カテゴリ | ファイル数 | 主な変更内容 |
|---------|-----------|-------------|
| ソースコード | 2 | `high_level.rs`（型定義）、`systems.rs`（クエリ変更） |
| テスト | 5+ | `taffy_layout_integration_test.rs`、`taffy_flex_layout_pure_test.rs`、`taffy_advanced_test.rs`等 |
| サンプル | 1 | `taffy_flex_demo.rs` |
| ドキュメント | 1 | `layout/mod.rs`（docコメント更新） |
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
- ✅ **完全統合アプローチ**を採用（Box系5種 + Flex系を`BoxStyle`に統合）
- ✅ **移行コスト**は許容範囲内（コンパイルエラーによる移行箇所明示）
- ⚠️ **未検討項目**は設計フェーズで詳細化

## Technical Notes

### 現行アーキテクチャ

```text
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

### 統合後のアーキテクチャ

```text
統合後のクエリ構造（build_taffy_styles_system）:
- &BoxStyle
- &mut TaffyStyle

フィルタ条件: Changed<BoxStyle>
```

### 廃止される代替案

1. ~~**部分統合**~~: taffyがコンテナー/アイテムを区別しないため不採用
2. **コンポーネントバンドル**: `#[derive(Bundle)]`は論理グループ化のみでクエリ簡略化に寄与しない
3. **現状維持**: クエリ複雑度・コード重複の問題が継続
