# Gap Analysis: Vertical Text Layout

## Analysis Summary
`vertical-text-layout` 機能の実装に向けたギャップ分析を実施しました。
現状の `wintf` ライブラリは DirectWrite を使用したテキスト描画の基盤を持っていますが、縦書きに関する設定や処理は完全に欠落しています。
主なギャップは、`Label` コンポーネントへの方向プロパティの追加、`draw_labels` システムにおける DirectWrite の縦書き設定 (`SetReadingDirection`, `SetFlowDirection`) の実装、および計算されたレイアウトサイズを出力する新しいコンポーネントの導入です。
既存のアーキテクチャ（ECS + COMラッパー）を拡張することで、比較的スムーズに実装可能と判断しました。

## 1. Current State Investigation

### Domain Assets
*   **Label Component**: `crates/wintf/src/ecs/widget/text/label.rs`
    *   現状: `text`, `font_family`, `font_size`, `color` のみを保持。
    *   制約: テキスト方向に関する情報は持っていない。
*   **Drawing System**: `crates/wintf/src/ecs/widget/text/draw_labels.rs`
    *   現状: `IDWriteFactory::CreateTextFormat` と `CreateTextLayout` を使用して描画リソースを生成。
    *   制約: 生成された `IDWriteTextFormat` / `IDWriteTextLayout` に対して、方向設定を行うロジックが存在しない。デフォルト（横書き）で動作している。
*   **COM Wrapper**: `crates/wintf/src/com/dwrite.rs`
    *   現状: 基本的なファクトリーメソッドのラッパーは存在するが、縦書き設定に必要なメソッド（`SetReadingDirection`, `SetFlowDirection`）へのアクセスが確認できない（`windows` クレートの生インターフェースを直接呼ぶ必要があるかもしれない）。

### Conventions
*   **ECS Pattern**: コンポーネント (`Label`) でデータを保持し、システム (`draw_labels`) で処理を行う。
*   **Resource Management**: `TextLayoutResource` で `IDWriteTextLayout` をキャッシュする仕組みがある（`label.rs` 内）。
*   **DirectWrite Usage**: `IDWriteFactory` から `TextFormat` -> `TextLayout` の順に生成。

## 2. Requirements Feasibility Analysis

### Requirement 1: テキスト方向の設定
*   **Gap**: `Label` 構造体に `direction` フィールドがない。
*   **Feasibility**: `Label` 構造体に `TextDirection` enum (Horizontal/Vertical) を追加することで容易に実現可能。

### Requirement 2 & 3: 縦書きレイアウト計算 & レンダリング
*   **Gap**: `draw_labels` システム内で `IDWriteTextFormat` または `IDWriteTextLayout` に対して縦書き設定を行っていない。
*   **Feasibility**:
    *   DirectWrite API (`SetReadingDirection`, `SetFlowDirection`) を使用して設定可能。
    *   縦書きの場合:
        *   `ReadingDirection`: `TopToBottom`
        *   `FlowDirection`: `RightToLeft` (一般的な日本語縦書き)
    *   これらを `draw_labels.rs` の `TextFormat` または `TextLayout` 生成後に追加設定するロジックが必要。

### Requirement 4: レイアウトサイズの出力
*   **Gap**: 計算されたサイズ（`TextLayout` のメトリクス）を外部（Taffy等）が利用できる形で出力していない。現状は `TextLayoutResource` に `IDWriteTextLayout` を保持しているだけ。
*   **Feasibility**:
    *   新しいコンポーネント `TextLayoutMetrics` (幅・高さを持つ) を定義。
    *   `draw_labels` システム内で `TextLayout::GetMetrics()` を呼び出し、その結果（`width`, `height`）を `TextLayoutMetrics` コンポーネントとして Entity に追加/更新する。

## 3. Implementation Approach Options

### Option A: Extend Existing Components (Recommended)
既存の `Label` と `draw_labels` を拡張するアプローチ。

*   **Changes**:
    1.  `Label` 構造体に `pub direction: TextDirection` を追加。
    2.  `TextDirection` enum を定義 (Horizontal, Vertical)。
    3.  `draw_labels.rs` を修正:
        *   `Label.direction` を読み取る。
        *   `IDWriteTextFormat` (または `TextLayout`) に対して `SetReadingDirection` / `SetFlowDirection` を呼び出す。
        *   `TextLayout` 生成後、`GetMetrics` でサイズを取得し、新設する `TextLayoutMetrics` コンポーネントに書き込む。

*   **Trade-offs**:
    *   ✅ 既存の構造を維持しつつ機能追加できる。
    *   ✅ 変更範囲が `label.rs` と `draw_labels.rs` に集中する。
    *   ❌ `Label` コンポーネントのフィールドが増える（許容範囲）。

### Option B: New Component for Vertical Text
縦書き専用の `VerticalLabel` コンポーネントを作成するアプローチ。

*   **Changes**:
    1.  `VerticalLabel` コンポーネントを新規作成。
    2.  専用の描画システム `draw_vertical_labels` を作成。

*   **Trade-offs**:
    *   ✅ 既存の `Label` に影響を与えない。
    *   ❌ コードの重複が多くなる（`draw_labels` とほぼ同じロジックが必要）。
    *   ❌ ユーザーが使い分けるのが面倒。

### Conclusion
**Option A** が最適です。`Label` コンポーネントは汎用的なテキスト表示要素であるべきで、プロパティで方向を切り替えられるのが自然です。また、DirectWrite の API 設計とも合致します。

## 4. Technical Research Needs
*   **DirectWrite API**: `SetReadingDirection` と `SetFlowDirection` が `IDWriteTextFormat` と `IDWriteTextLayout` のどちらで設定すべきか、または両方かを確認する。（通常は `TextFormat` で設定し `TextLayout` に継承させるか、`TextLayout` で上書きする）
*   **Metrics**: 縦書き時の `GetMetrics` が返す `width` / `height` の意味（論理的な幅・高さか、物理的な幅・高さか）を確認し、`TextLayoutMetrics` に格納する際に適切にマッピングする。

