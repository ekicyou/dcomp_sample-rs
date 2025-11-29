# Design Document: Vertical Text Layout

---
**Purpose**: Provide sufficient detail to ensure implementation consistency across different implementers, preventing interpretation drift.
---

## Overview
本機能は、`wintf` ライブラリの `Label` コンポーネントを拡張し、日本語の縦書き表示をサポートします。DirectWrite の機能を活用して縦書きレンダリングを実現し、そのレイアウトサイズをシステム内で利用可能にします。

### Goals
- `Label` コンポーネントで縦書き・横書きを指定可能にする。
- DirectWrite を使用して正しく縦書きレンダリングを行う（グリフ回転、行送り）。
- 計算されたテキストの物理サイズ（幅・高さ）を ECS コンポーネントとして出力する。
- `simple_window.rs` サンプルで動作を検証する。

### Non-Goals
- Taffy レイアウトエンジンへの完全な自動統合（今回はサイズ出力まで）。
- 縦中横（Tate-Chu-Yoko）などの高度な組版機能。
- 複雑な書式設定（リッチテキスト）。

## Architecture

### Architecture Pattern & Boundary Map
既存の ECS + COM Wrapper パターンに従います。

*   **ECS Layer**:
    *   `Label` (Modified): 方向プロパティを保持。
    *   `TextLayoutMetrics` (New): 計算された物理サイズを保持。
*   **System Layer**:
    *   `draw_labels` (Modified): 縦書き設定を行い、Metrics を出力。
*   **COM Layer**:
    *   `DirectWrite`: 実際のレンダリングとメトリクス計算を担当。

### Technology Stack
| Layer | Choice / Version | Role in Feature | Notes |
|-------|------------------|-----------------|-------|
| Graphics | DirectWrite | Text Rendering | `SetReadingDirection`, `SetFlowDirection` を使用 |
| ECS | bevy_ecs | State Management | コンポーネントによるデータ保持 |

## Requirements Traceability

| Requirement | Summary | Components | Interfaces | Flows |
|-------------|---------|------------|------------|-------|
| 1.1, 1.2 | Text Direction Config | `Label` | `direction` field | N/A |
| 1.3 | Trigger Recalc | `Label` | Change detection | `draw_labels` detects change |
| 2.1, 2.2 | Vertical Layout Calc | `draw_labels` | `IDWriteTextLayout` | Metrics extraction |
| 3.1, 3.2 | Vertical Rendering | `draw_labels` | `IDWriteTextFormat` | `SetReadingDirection` |
| 4.1, 4.2 | Layout Size Output | `TextLayoutMetrics` | `width`, `height` | Output from `draw_labels` |
| 5.1, 5.2 | Verification Sample | `simple_window.rs` | N/A | Visual verification |

## Components and Interfaces

### Component: Label (Extension)
**Domain**: ECS / Widget
**Intent**: テキスト表示の設定を保持する。

*   **Changes**:
    *   `direction: TextDirection` フィールドを追加。
    *   `TextDirection` enum を定義（CSSの `writing-mode` と `direction` の組み合わせに対応）。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TextDirection {
    #[default]
    HorizontalLeftToRight, // writing-mode: horizontal-tb, direction: ltr
    HorizontalRightToLeft, // writing-mode: horizontal-tb, direction: rtl
    VerticalRightToLeft,   // writing-mode: vertical-rl (Japanese)
    VerticalLeftToRight,   // writing-mode: vertical-lr
}

pub struct Label {
    // ... existing fields ...
    pub direction: TextDirection,
}
```

### Component: TextLayoutMetrics (New)
**Domain**: ECS / Layout
**Intent**: 計算されたテキストの物理サイズを保持する。

```rust
#[derive(Component, Debug, Clone, Copy)]
pub struct TextLayoutMetrics {
    pub width: f32,  // Physical width (pixels)
    pub height: f32, // Physical height (pixels)
}
```

### System: draw_labels (Extension)
**Domain**: ECS / System
**Intent**: DirectWrite リソースを生成し、描画コマンドを発行する。

*   **Logic Update**:
    1.  `Label.direction` をチェック。
    2.  `IDWriteTextFormat` 生成後、方向に応じて以下を設定：
        *   **HorizontalLeftToRight**:
            *   Reading: `DWRITE_READING_DIRECTION_LEFT_TO_RIGHT`
            *   Flow: `DWRITE_FLOW_DIRECTION_TOP_TO_BOTTOM`
        *   **HorizontalRightToLeft**:
            *   Reading: `DWRITE_READING_DIRECTION_RIGHT_TO_LEFT`
            *   Flow: `DWRITE_FLOW_DIRECTION_TOP_TO_BOTTOM`
        *   **VerticalRightToLeft**:
            *   Reading: `DWRITE_READING_DIRECTION_TOP_TO_BOTTOM`
            *   Flow: `DWRITE_FLOW_DIRECTION_RIGHT_TO_LEFT`
        *   **VerticalLeftToRight**:
            *   Reading: `DWRITE_READING_DIRECTION_TOP_TO_BOTTOM`
            *   Flow: `DWRITE_FLOW_DIRECTION_LEFT_TO_RIGHT`
    3.  `IDWriteTextLayout` 生成後、`GetMetrics` を呼び出す。
    4.  Metrics を `TextLayoutMetrics` に変換。
        *   縦書き（Vertical*）の場合は、DirectWriteが返す width/height の意味が入れ替わっているため、物理的な幅・高さに合わせてスワップする（またはDWの仕様に従い適切に解釈する）。
            *   *Note*: DirectWriteの縦書きでは、Layout Widthは「行の高さ（文字の進行方向）」、Layout Heightは「行の積み重ね方向の幅」になる場合があるため、`DWRITE_TEXT_METRICS` の値を物理座標系（スクリーン上の幅・高さ）に正しくマッピングする。
    5.  Entity に `TextLayoutMetrics` を挿入（または更新）。

## Data Models
N/A (Component definitions cover this)

## System Flows
N/A (Single system update)

## Verification Plan

### Sample Update: `simple_window.rs`
既存の `simple_window.rs` に以下の検証用 UI ツリーを追加します。

*   **Location**: Root Window の直下、既存のツリーの右側 (Offset x: 300.0, y: 20.0)。
*   **Structure**:
    *   **Container (Rectangle)**:
        *   Size: 200x400
        *   Color: Light Gray (visual separation)
    *   **Vertical Label (Child of Container)**:
        *   Text: "縦書き\nテスト\n(Vertical)"
        *   Direction: `TextDirection::VerticalRightToLeft`
        *   Offset: x: 150.0, y: 10.0 (右から左へ流れるため、右側に配置)
    *   **Horizontal RTL Label (Child of Container)**:
        *   Text: "RTL Test"
        *   Direction: `TextDirection::HorizontalRightToLeft`
        *   Offset: x: 10.0, y: 200.0

### Verification Steps
1.  `cargo run --example simple_window` を実行。
2.  ウィンドウ右側にグレーの領域が表示されることを確認。
3.  "縦書きテスト" が縦方向に描画され、行が右から左へ進むことを確認。
4.  "RTL Test" が右寄せ（またはRTLの挙動）で描画されることを確認。
