# Implementation Plan

## Overview
`GlobalArrangement.bounds` → `WindowPos` 変換の重複システムを単一の `window_pos_sync_system` に統合する。`sync_window_pos` のロジックを layout モジュールに移動し、正しい丸めロジック（ceil）を適用する。

## 変更ファイル
- `crates/wintf/src/ecs/layout/systems.rs` - 新システム実装
- `crates/wintf/src/ecs/graphics/systems.rs` - 旧システム削除
- `crates/wintf/src/ecs/world.rs` - スケジュール登録更新
- `crates/wintf/tests/layout_graphics_sync_test.rs` - テスト更新

## Tasks

- [ ] 1. window_pos_sync_system の実装
- [ ] 1.1 layout/systems.rs に新システムを実装
  - `sync_window_pos` のロジックをベースに移植（graphics/systems.rs L699-784）
  - クエリ簡素化: `Arrangement` 除去、`Changed<GlobalArrangement>` のみのフィルタ
  - 変換ロジック: `position`（left/top を truncate）、`size`（幅/高さを ceil）
  - 無効 bounds ガード（幅 ≤ 0 or 高さ ≤ 0 でスキップ）
  - 差分検出による不要な `Changed` フラグ発行防止
  - 構造化ログ: `[window_pos_sync_system]` プレフィックス、debug/trace レベル
  - `format_entity_name` の既存インポート活用（L14 で既に `use crate::ecs::graphics::format_entity_name`）
  - _Requirements: 1.1, 1.2, 1.3, 2.1, 2.2, 2.3, 2.4, 5.1, 5.2, 5.3, 5.4, 5.5_

- [ ] 2. 旧システムの削除
- [ ] 2.1 (P) graphics/systems.rs から sync_window_pos を削除
  - `pub fn sync_window_pos` の関数定義全体を削除（L699-784）
  - _Requirements: 3.1_

- [ ] 2.2 (P) layout/systems.rs から update_window_pos_system を削除
  - `pub fn update_window_pos_system` の関数定義全体を削除（L369-393）
  - _Requirements: 3.2_

- [ ] 3. スケジュール登録の統合
- [ ] 3.1 world.rs の PostLayout スケジュール更新
  - `sync_window_pos` と `update_window_pos_system` の2つの登録行を削除
  - `window_pos_sync_system` を `propagate_global_arrangements` の後に登録
  - `.chain()` による順序保証を継続
  - _Requirements: 3.3, 4.1, 4.2_

- [ ] 4. テストの更新と検証
- [ ] 4.1 (P) layout_graphics_sync_test.rs の旧名参照を更新
  - `test_sync_window_pos`, `test_skip_invalid_bounds`, `test_echo_back_flow` の3テスト
  - `wintf::ecs::sync_window_pos` → `wintf::ecs::window_pos_sync_system` に更新
  - `FrameCount` リソース挿入は継続（新システムでも必要）
  - _Requirements: 6.3_

- [ ] 4.2 統合テストの実行と検証
  - `cargo test` で全テストパスを確認
  - サンプルアプリ（`cargo run --example areka`）で全 DPI（100%/125%/150%）動作確認
  - _Requirements: 6.1, 6.2_

## Requirements Coverage

| Requirement | Tasks |
|-------------|-------|
| 1.1, 1.2, 1.3 | 1.1 |
| 2.1, 2.2, 2.3, 2.4 | 1.1 |
| 3.1 | 2.1 |
| 3.2 | 2.2 |
| 3.3 | 3.1 |
| 4.1, 4.2 | 3.1 |
| 5.1, 5.2, 5.3, 5.4, 5.5 | 1.1 |
| 6.1, 6.2 | 4.2 |
| 6.3 | 4.1 |

**全19要件カバー済み**
