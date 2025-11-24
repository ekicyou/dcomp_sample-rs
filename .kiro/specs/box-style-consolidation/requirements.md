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

### Requirement 5: 実装可否判断

**Objective:** プロジェクト管理者として、本統合の実装可否を判断するための情報を得たい。これにより、適切なリソース配分と計画立案が可能になる。

#### Acceptance Criteria

1. 設計フェーズで、bevy_ecsにおけるコンポーネント統合のメリット・デメリットが文書化されること
2. 設計フェーズで、アーキタイプシステムへの影響評価が実施されること
3. 設計フェーズで、変更検出（`Changed<T>`）の粒度トレードオフが分析されること
4. If 実装が推奨されない場合、レイアウトシステムは代替案（クエリ最適化、コンポーネントバンドル等）を提示すること

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
