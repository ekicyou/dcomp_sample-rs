# Requirements Document

## Project Description (Input)
`PointerState.screen_point` フィールドの命名を `client_point` に修正する。現在のフィールド名は `screen_point` だが、実際に保持している値はクライアント座標（物理 px）であり、名前と内容が不一致。値の変更は行わず、純粋なリネーム作業。

## 親仕様からの参照情報

### 調査レポート参照
- **親仕様**: `dpi-coordinate-transform-survey`
- **根拠セクション**: report.md §1.2 項目2, §2.1（PointerState行）, §6.4.2, §8.2 Step 2, §7.1 G6

### 調査結果サマリ
- `ecs/pointer/mod.rs` L116-145 の `PointerState.screen_point` は、名前が `screen_point`（スクリーン座標を示唆）だが、実際にはクライアント座標（物理 px）を保持している
- ドラッグ処理は別経路で正しいスクリーン座標を使用しているため、現在の命名不整合による **実害は限定的** だが、可読性・保守性の観点から修正すべき
- 値そのものは変更しない（クライアント座標・物理 px のまま）

### 関連するコード箇所
| ファイル | 行番号 | 内容 |
|---------|--------|------|
| `crates/wintf/src/ecs/pointer/mod.rs` | L116-145 | `PointerState` 構造体定義 — `screen_point` フィールド |
| （参照箇所） | — | `screen_point` を参照する全コード（リネーム対象） |

### 依存関係
- **前提条件**: なし（Step 1 と独立して実施可能）
- **後続仕様**: なし

### コスト見積もり
- Low（数時間）

### 検証基準（調査レポートより）
- コンパイル通過
- 既存動作に変更なし

## Requirements
<!-- Will be generated in /kiro:spec-requirements phase -->
