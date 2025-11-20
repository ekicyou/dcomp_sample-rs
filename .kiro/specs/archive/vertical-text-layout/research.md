# Research & Design Decisions: Vertical Text Layout

---
**Purpose**: Capture discovery findings, architectural investigations, and rationale that inform the technical design.
---

## Summary
- **Feature**: `vertical-text-layout`
- **Discovery Scope**: Extension (Light Discovery)
- **Key Findings**:
  - DirectWrite supports vertical text via `SetReadingDirection` and `SetFlowDirection` on `IDWriteTextFormat`.
  - Vertical text requires orthogonal directions: `ReadingDirection::TopToBottom` and `FlowDirection::RightToLeft` (for Japanese).
  - `DWRITE_TEXT_METRICS` interpretation likely flips for vertical text (Width = Vertical Advance, Height = Horizontal Flow).

## Research Log

### DirectWrite Vertical Text APIs
- **Context**: Need to implement vertical text rendering using DirectWrite.
- **Sources Consulted**: Microsoft Learn (IDWriteTextFormat::SetReadingDirection, SetFlowDirection, DWRITE_TEXT_METRICS)
- **Findings**:
  - `IDWriteTextFormat::SetReadingDirection(DWRITE_READING_DIRECTION_TOP_TO_BOTTOM)` sets vertical reading.
  - `IDWriteTextFormat::SetFlowDirection(DWRITE_FLOW_DIRECTION_RIGHT_TO_LEFT)` sets lines to stack from right to left.
  - These must be set on the `IDWriteTextFormat` used to create the `IDWriteTextLayout`.
  - `DWRITE_TEXT_METRICS` fields (`width`, `height`) are relative to the reading/flow directions.
    - `width`: Dimension in reading direction (Vertical Height).
    - `height`: Dimension in flow direction (Horizontal Width).
- **Implications**:
  - When extracting metrics for Taffy/Layout, we must swap width and height if the text direction is vertical.
  - `TextLayoutMetrics` component should store "Physical" width/height to avoid confusion in the rest of the system.

### `windows` Crate Compatibility
- **Context**: Verify if `windows` crate (0.62.1) supports these APIs.
- **Findings**:
  - `windows` crate generally maps all COM interfaces.
  - `IDWriteTextFormat` is already used in `draw_labels.rs`.
  - `SetReadingDirection` and `SetFlowDirection` are standard methods of `IDWriteTextFormat`.
- **Implications**: No special bindings needed; just call the methods.

## Design Decisions

### Decision: Metric Swapping for Vertical Text
- **Context**: `DWRITE_TEXT_METRICS` returns logical dimensions based on reading direction.
- **Selected Approach**: The `draw_labels` system will swap `width` and `height` from `DWRITE_TEXT_METRICS` when storing them into `TextLayoutMetrics` if the direction is vertical.
- **Rationale**: The rest of the layout system (Taffy, Windowing) expects physical X/Y dimensions (Width/Height). Normalizing at the source prevents confusion downstream.
- **Trade-offs**: `TextLayoutMetrics` will always represent physical dimensions, losing the "logical" text metrics context, but this is preferred for layout integration.

### Decision: `TextLayoutMetrics` Component
- **Context**: Need to expose calculated text size to the ECS world.
- **Selected Approach**: Create a new component `TextLayoutMetrics { width: f32, height: f32 }`.
- **Rationale**: Decouples rendering (DirectWrite) from Layout (Taffy). `draw_labels` calculates and populates this; a future `layout_system` can consume it.
- **Trade-offs**: Adds an extra component, but improves separation of concerns.

## Risks & Mitigations
- **Risk 1**: `DWRITE_TEXT_METRICS` interpretation might be subtle (e.g., `layoutWidth` vs `width`).
  - **Mitigation**: Verify visually with the `simple_window.rs` update. If the bounding box looks wrong (e.g., too wide/short), adjust the mapping.
- **Risk 2**: Font support for vertical text.
  - **Mitigation**: Use standard fonts like "Meiryo" or "MS Gothic" which are known to support vertical glyph variants.

## References
- [IDWriteTextFormat::SetReadingDirection](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/nf-dwrite-idwritetextformat-setreadingdirection)
- [DWRITE_TEXT_METRICS](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/ns-dwrite-dwrite_text_metrics)

## CSS Writing Modes and Direction
ユーザーからの要望により、CSSにおけるテキスト方向の仕様を調査。

### CSS Properties
*   `writing-mode`: ブロックのフロー方向を指定。
    *   `horizontal-tb`: 横書き、行は上から下へ。
    *   `vertical-rl`: 縦書き、行は右から左へ（日本語など）。
    *   `vertical-lr`: 縦書き、行は左から右へ（モンゴル語など）。
*   `direction`: インラインコンテンツの方向を指定。
    *   `ltr`: 左から右へ。
    *   `rtl`: 右から左へ（アラビア語など）。

### DirectWrite Mapping
DirectWriteの `DWRITE_READING_DIRECTION` と `DWRITE_FLOW_DIRECTION` は直交する必要がある。
CSSのプロパティとの対応は以下の通り。

| CSS Combination | DirectWrite Reading | DirectWrite Flow | Description |
| :--- | :--- | :--- | :--- |
| `writing-mode: horizontal-tb`<br>`direction: ltr` | `LEFT_TO_RIGHT` | `TOP_TO_BOTTOM` | 一般的な横書き (英語、日本語横書き) |
| `writing-mode: horizontal-tb`<br>`direction: rtl` | `RIGHT_TO_LEFT` | `TOP_TO_BOTTOM` | アラビア語などの横書き |
| `writing-mode: vertical-rl` | `TOP_TO_BOTTOM` | `RIGHT_TO_LEFT` | 日本語などの縦書き |
| `writing-mode: vertical-lr` | `TOP_TO_BOTTOM` | `LEFT_TO_RIGHT` | モンゴル語などの縦書き |

この4パターンを `TextDirection` Enum でサポートする方針とする。
