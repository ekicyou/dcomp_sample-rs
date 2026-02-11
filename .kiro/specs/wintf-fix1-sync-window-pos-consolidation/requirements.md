# Requirements Document

## Introduction

本仕様は、wintf フレームワークにおける `GlobalArrangement.bounds` → `WindowPos` 変換ロジックの重複解消を定義する。現在、`sync_window_pos`（graphics/systems.rs）と `update_window_pos_system`（layout/systems.rs）の2つのシステムが同一の変換を実装しており、保守性・可読性およびバグ混入リスクの観点から1つのシステム `window_pos_sync_system` に統合する。この統合は後続仕様（wintf-fix3: 逆同期有効化、wintf-fix4: フィードバック防止簡素化）の前提条件である。

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

### Requirement 1: 重複システムの統合
**Objective:** 開発者として、`GlobalArrangement.bounds` → `WindowPos` 変換を単一のシステムで実行したい。これにより、変換ロジックの一元管理が可能になり、保守性が向上する。

#### Acceptance Criteria
1. The wintf framework shall provide a single ECS system `window_pos_sync_system` that converts `GlobalArrangement.bounds` to `WindowPos`（position と size）for all `Window` entities.
2. When `GlobalArrangement` が変更された場合, the `window_pos_sync_system` shall update the corresponding `WindowPos.position` and `WindowPos.size` based on `GlobalArrangement.bounds`.
3. The `window_pos_sync_system` shall be located in the `layout` module（`crates/wintf/src/ecs/layout/systems.rs`）as the canonical owner of this transformation.

### Requirement 2: 変換ロジックの正確性
**Objective:** 開発者として、統合後のシステムが既存の動作と完全に等価であることを保証したい。これにより、統合による機能リグレッションを防止する。

#### Acceptance Criteria
1. The `window_pos_sync_system` shall convert `GlobalArrangement.bounds.left` and `GlobalArrangement.bounds.top` to `WindowPos.position`（`POINT { x, y }`）by truncating to `i32`.
2. The `window_pos_sync_system` shall convert `GlobalArrangement.bounds` の幅と高さ to `WindowPos.size`（`SIZE { cx, cy }`）by ceiling to `i32`（`width.ceil() as i32`, `height.ceil() as i32`）.
3. If `GlobalArrangement.bounds` の幅または高さが 0 以下である場合, the `window_pos_sync_system` shall skip the `WindowPos` update for that entity.
4. The `window_pos_sync_system` shall only update `WindowPos` when the computed position or size differs from the current values（差分検出による不要な変更通知の防止）.

### Requirement 3: 旧システムの除去
**Objective:** 開発者として、統合により不要になった重複コードを完全に除去したい。これにより、コードベースの明確性を維持する。

#### Acceptance Criteria
1. When 統合が完了した場合, the wintf framework shall no longer contain the `sync_window_pos` function in `graphics/systems.rs`.
2. When 統合が完了した場合, the wintf framework shall no longer contain the `update_window_pos_system` function in `layout/systems.rs`.
3. The `world.rs` system registration shall register only `window_pos_sync_system` in the `PostLayout` schedule, replacing the previous two-system chain.

### Requirement 4: スケジューリング順序の維持
**Objective:** 開発者として、統合後もシステム実行順序が正しく保たれることを保証したい。これにより、レイアウト伝播後のウィンドウ位置更新が確実に行われる。

#### Acceptance Criteria
1. The `window_pos_sync_system` shall execute after `propagate_global_arrangements` in the `PostLayout` schedule.
2. The `window_pos_sync_system` shall be the final system in the `PostLayout` schedule chain（`propagate_global_arrangements` の直後、かつ PostLayout の末尾）.

> **Note:** `apply_window_pos_changes`（`WindowPos` → `SetWindowPos` 変換）は `UISetup` スケジュールに登録されている。フレーム実行順（`PostLayout` → `UISetup`）により、`window_pos_sync_system` → `apply_window_pos_changes` の順序はスケジュール間で暗黙的に保証される。

### Requirement 5: 構造化ログの維持
**Objective:** 開発者として、統合後もデバッグに必要な tracing ログが維持されることを保証したい。これにより、ウィンドウ位置同期の問題診断能力を維持する。

#### Acceptance Criteria
1. The `window_pos_sync_system` shall emit a `debug!` level log with structured fields（`frame`, `entity`, bounds 座標, 幅, 高さ）when processing each entity.
2. If bounds が無効な値（幅 ≤ 0 or 高さ ≤ 0）である場合, the `window_pos_sync_system` shall emit a `debug!` level log indicating the skip reason.
3. When `WindowPos` が実際に更新された場合, the `window_pos_sync_system` shall emit a `debug!` level log with old and new values.
4. When `WindowPos` に変更がない場合, the `window_pos_sync_system` shall emit a `trace!` level log indicating no change was needed.
5. The `window_pos_sync_system` shall use the function name prefix `[window_pos_sync_system]` in all log messages（logging.md の書式パターン規約に準拠）.

### Requirement 6: 既存テストとの互換性
**Objective:** 開発者として、統合後にすべての既存テストがパスすることを保証したい。これにより、リファクタリングの安全性を確認する。

#### Acceptance Criteria
1. The wintf framework shall pass all existing tests（`cargo test`）without modification to test expectations after consolidation.
2. The wintf framework shall maintain correct window positioning across all DPI settings（100%/125%/150%）after consolidation.
3. If 既存テストが旧システム名（`sync_window_pos` or `update_window_pos_system`）を直接参照している場合, the wintf framework shall update those references to `window_pos_sync_system`.
