# Requirements Document

## Project Description (Input)
`sync_window_arrangement_from_window_pos`（現在コメントアウト中）の有効化条件と副作用を評価し、必要に応じて有効化する。このシステムは、ユーザーがウィンドウを移動した際に Win32 側の位置変更を ECS の `Arrangement` に逆反映する役割を持つ。

## 親仕様からの参照情報

### 調査レポート参照
- **親仕様**: `dpi-coordinate-transform-survey`
- **根拠セクション**: report.md §1.2 項目3, §5.5, §6.4.3, §8.2 Step 3, §7.1 G5

### 調査結果サマリ
- `ecs/layout/systems.rs` L405-455 に `sync_window_arrangement_from_window_pos` が定義されている
- `ecs/world.rs` L360-363 でコメントアウトにより無効化されている
- 無効化理由は「DIP 座標への二重変換」のリスク
- 調査の結果、現在の座標系設計（`WindowPos.position` = 物理px、`Arrangement.offset` = 物理px）は **同一座標系** であり、座標変換は不要 — 二重変換リスクは当該システムの有効化で発生しない
- ただし、重複システムが存在する状態（`sync_window_pos` + `update_window_pos_system`）で有効化すると **同期方向の競合リスク** がある → wintf-fix1 完了が前提

### 関連するコード箇所
| ファイル | 行番号 | 内容 |
|---------|--------|------|
| `crates/wintf/src/ecs/layout/systems.rs` | L405-455 | `sync_window_arrangement_from_window_pos` 本体 |
| `crates/wintf/src/ecs/world.rs` | L360-363 | コメントアウト箇所（有効化対象） |
| `crates/wintf/src/ecs/layout/systems.rs` | L369-393 | `update_window_pos_system`（順方向同期、fix1 で統合予定） |

### 依存関係
- **前提条件**: `wintf-fix1-sync-window-pos-consolidation` 完了（重複システムが統合済みであること）
- **後続仕様**: `wintf-fix4-feedback-loop-simplify`（本仕様の完了が前提）

### コスト見積もり
- Medium（2-3日）

### 検証基準（調査レポートより）
- ユーザーがウィンドウを移動した際に ECS 側の `Arrangement` が更新されること
- フィードバックループが発生しないこと

## Requirements
<!-- Will be generated in /kiro:spec-requirements phase -->
