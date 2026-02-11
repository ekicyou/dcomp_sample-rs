# Requirements Document

## Project Description (Input)
`WindowPosCommandBuffer` + `SuppressExternalSync` + `DpiChangeContext` の 3層フィードバックループ防止機構を整理し、簡素化する。`WindowPosCommandBuffer` と `SuppressExternalSync` を単一ゲートシステムに統合し、`DpiChangeContext`（TLS）は WndProc コンテキスト固有のため維持する。

## 親仕様からの参照情報

### 調査レポート参照
- **親仕様**: `dpi-coordinate-transform-survey`
- **根拠セクション**: report.md §1.2 項目4, §5（フィードバック防止関連セクション全体）, §6.4.4, §8.2 Step 4, §7.1 G9

### 調査結果サマリ
- 現在、ECS ↔ Win32 間のフィードバックループ防止が **3層** で実装されている：
  1. `WindowPosCommandBuffer` — ECS → Win32 の同期バッファ
  2. `SuppressExternalSync` — Win32 → ECS の逆反映を一時抑制
  3. `DpiChangeContext` (TLS) — DPI 変更チェーンの追跡
- 3層は冗長であり、`WindowPosCommandBuffer` と `SuppressExternalSync` は **統一可能**
- `DpiChangeContext` は `WndProc` のコールスタック内でのみ有効な TLS であり、性質が異なるため **維持** する
- 逆同期（`sync_window_arrangement_from_window_pos`）が有効化された状態でフィードバックループ防止が正しく動作することが必要

### 関連するコード箇所
| ファイル | 行番号 | 内容 |
|---------|--------|------|
| （要調査） | — | `WindowPosCommandBuffer` の定義と使用箇所 |
| （要調査） | — | `SuppressExternalSync` の定義と使用箇所 |
| （要調査） | — | `DpiChangeContext` の定義と使用箇所（TLS） |
| `crates/wintf/src/ecs/world.rs` | — | フィードバック防止のシステム登録箇所 |

### 依存関係
- **前提条件**: `wintf-fix3-sync-arrangement-enable` 完了（逆同期が有効化されていること）
- **後続仕様**: なし（本系列の最終ステップ）

### コスト見積もり
- Medium（2-3日）

### 検証基準（調査レポートより）
- フィードバックループが発生しないこと
- DPI 変更時の処理が正しく動作すること
- ウィンドウ移動・リサイズがスムーズであること

## Requirements
<!-- Will be generated in /kiro:spec-requirements phase -->
