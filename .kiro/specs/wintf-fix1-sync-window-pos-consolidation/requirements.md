# Requirements Document

## Project Description (Input)
`sync_window_pos`（graphics/systems.rs）と `update_window_pos_system`（layout/systems.rs）の2つの重複システムを1つに統合する。両システムは同じ変換（`GlobalArrangement.bounds` → `WindowPos` → `SetWindowPos`）を行っており、保守性・可読性の観点から統合が必要。

## 親仕様からの参照情報

### 調査レポート参照
- **親仕様**: `dpi-coordinate-transform-survey`
- **根拠セクション**: report.md §1.2 項目1, §6.4.1, §8.2 Step 1, §7.1 G3

### 調査結果サマリ
- `graphics/systems.rs` L699-720 の `sync_window_pos` と `layout/systems.rs` L369-393 の `update_window_pos_system` が **同一の変換ロジック**（`GlobalArrangement.bounds` → `WindowPos`）を重複して実装している
- 統合先は `window_pos_sync_system` とし、`GlobalArrangement.bounds` → `WindowPos` → `SetWindowPos` を一元管理する
- この統合は他の改善（wintf-fix3: 逆同期有効化、wintf-fix4: FB防止簡素化）の **前提条件** である

### 関連するコード箇所
| ファイル | 行番号 | 内容 |
|---------|--------|------|
| `crates/wintf/src/ecs/graphics/systems.rs` | L699-720 | `sync_window_pos` — 重複システム（廃止対象） |
| `crates/wintf/src/ecs/layout/systems.rs` | L369-393 | `update_window_pos_system` — 統合先 |
| `crates/wintf/src/ecs/world.rs` | — | システム登録箇所 |

### 依存関係
- **前提条件**: なし（最初に着手可能）
- **後続仕様**: `wintf-fix3-sync-arrangement-enable`（本仕様の完了が前提）

### コスト見積もり
- Low（1-2日）

### 検証基準（調査レポートより）
- ウィンドウ配置が全 DPI（100%/125%/150%）で正しく動作すること
- テスト期待値に変更がないこと

## Requirements
<!-- Will be generated in /kiro:spec-requirements phase -->
