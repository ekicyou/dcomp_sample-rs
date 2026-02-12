# Implementation Plan: wintf-fix3-sync-arrangement-enable

## Overview
逆方向同期システム `sync_window_arrangement_from_window_pos` を有効化し、Win32 ウィンドウ位置変更を ECS `Arrangement` に反映する。Option B' (Changed<WindowPos> フィルタ + set_if_neq パターン) を実装。

---

## Tasks

### Phase 1: コア実装

- [ ] 1. 逆方向同期システムの有効化
- [ ] 1.1 (P) Changed<WindowPos> フィルタの追加
  - `sync_window_arrangement_from_window_pos` のクエリフィルタに `Changed<WindowPos>` を追加
  - クエリ定義を `Query<..., (With<Window>, Changed<WindowPos>)>` に変更
  - 既存のロジック（DPI変換、エッジケース処理、等値チェック、ログ出力）は変更なし
  - _Requirements: 1.1_

- [ ] 1.2 PostLayout スケジュール登録
  - `world.rs` の PostLayout スケジュール登録で、`sync_window_arrangement_from_window_pos` のコメントアウト（4行: 3行コメント + 1行コード）を解除
  - レガシーコメント（「一旦無効化」「二重になる」等）を削除
  - `.chain()` タプルの先頭に配置して実行順序を保証: `sync_window_arrangement_from_window_pos` → `sync_simple_arrangements` → `mark_dirty_arrangement_trees` → `propagate_global_arrangements` → `window_pos_sync_system`
  - _Requirements: 1.2, 1.3_

- [ ] 2. エコーバック抑制の実装
- [ ] 2.1 (P) WM_WINDOWPOSCHANGED ハンドラに set_if_neq パターンを適用
  - `handlers.rs` の `WM_WINDOWPOSCHANGED` ハンドラで `WindowPos` 更新時に条件分岐を追加
  - 新しい `position` / `size` を現在値と比較（`WindowPos` は `PartialEq` derive 済み）
  - 変更ありの場合: 通常代入（`DerefMut` → `Changed` 発火）+ `last_sent` 更新
  - 変更なしの場合（エコーバック等）: `bypass_change_detection()` 経由で `last_sent` のみ更新（`Changed` 抑制）
  - 既存の `apply_window_pos_changes` (systems.rs L807) と同一パターン
  - _Requirements: 1.4_

### Phase 2: テストと検証

- [ ] 3. フィードバックループ検証の自動化
- [ ] 3.1 統合テストの実装
  - `tests/feedback_loop_convergence_test.rs` を新規作成
  - `EntityWorldMut` 経由で `WindowPos.position` を変更 → `try_tick_world()` 実行
  - `Changed<WindowPos>` / `Changed<Arrangement>` / `Changed<GlobalArrangement>` の発火回数を change detection API で計測
  - R4-AC1「1フレーム内収束」を assert で検証（変更回数 ≤ 1 回）
  - DPI 96/192 の両環境でテスト（scale=1.0, 2.0）
  - _Requirements: 4.1_

- [ ] 4. 検証とテスト
- [ ] 4.1 回帰テストの実行
  - `cargo test` で全テストがパスすることを確認
  - 特に `tests/feedback_loop_convergence_test.rs` が成功することを確認
  - 既存の arrangement/layout 関連テストの非破壊を確認
  - _Requirements: 5.1_

- [ ] 4.2* 手動検証の実施
  - `RUST_LOG=debug cargo run --example taffy_flex_demo` でウィンドウをドラッグ移動
  - ログに `[sync_window_arrangement_from_window_pos]` 出力があることを確認（R6-AC1, AC3）
  - 変更なしの場合にログ出力がないことを確認（R6-AC2）
  - DPI 125%（scale=1.25）環境でウィンドウ移動の視覚的確認（R2-AC1～AC4）
  - ウィンドウの表示・移動・リサイズが正常に機能することを確認（R5-AC2）
  - `RUST_LOG=trace` でエコーバック時に `Changed` が抑制されることを確認（R1-AC4）
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 3.1, 3.2, 3.3, 4.2, 4.3, 5.2, 6.1, 6.2, 6.3_

---

## Requirements Coverage

| 要件 | タスク | 備考 |
|------|--------|------|
| R1.1 | 1.1 | Changed<WindowPos> フィルタ |
| R1.2 | 1.2 | スケジュール順序 |
| R1.3 | 1.2 | レガシーコメント削除 |
| R1.4 | 2.1 | set_if_neq パターン |
| R2.1～R2.4 | 4.2 | DPI変換（既存実装の検証） |
| R3.1～R3.3 | 4.2 | エッジケース（既存実装の検証） |
| R4.1 | 3.1 | 1フレーム内収束（自動テスト） |
| R4.2 | 4.2 | Changed フィルタ（既存確認） |
| R4.3 | 4.2 | 等値チェック（既存確認） |
| R5.1 | 4.1 | cargo test パス |
| R5.2 | 4.2 | taffy_flex_demo 動作確認 |
| R6.1～R6.3 | 4.2 | ログ出力（既存実装の検証） |

**全 19 個の AC（Acceptance Criteria）をカバー**
