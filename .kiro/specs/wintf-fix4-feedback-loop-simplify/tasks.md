# Implementation Plan

## Task Overview

本実装計画は、`SetWindowPos` ラッパーと TLS フラグによるフィードバックループ防止メカニズムの簡素化を段階的に実現する。コア機能（TLS + ラッパー）を先に実装し、既存ハンドラ・システムを順次移行、最後にクリーンアップとテストを行う。

## Tasks

- [ ] 1. SetWindowPos ラッパーと TLS フラグの実装
- [ ] 1.1 (P) IS_SELF_INITIATED TLS フラグと is_self_initiated() ヘルパーを追加
  - `ecs/window.rs` に `IS_SELF_INITIATED: Cell<bool>` を `thread_local!` で宣言
  - 初期値は `false`、TLS のため `Cell` を使用
  - `pub fn is_self_initiated() -> bool` ヘルパー関数を実装し、TLS フラグの現在値を返す
  - doc comment でフラグのライフサイクル（ラッパー関数のスコープ内でのみ `true`）を明記
  - ロギング: `trace!` レベルでフラグ状態変更を記録（構造化フィールド: `is_initiated` bool）
  - _Requirements: 1.1, 1.2, 4.1_

- [ ] 1.2 (P) SetWindowPosGuard 構造体と guarded_set_window_pos() 関数を実装
  - `SetWindowPosGuard` 構造体を定義し、Drop trait で `IS_SELF_INITIATED.set(false)` を実装
  - `pub unsafe fn guarded_set_window_pos(...)` を実装、関数冒頭で `IS_SELF_INITIATED.set(true)` 実行
  - `let _guard = SetWindowPosGuard;` で RAII ガード作成、スコープ終了時に自動リセット保証
  - `SetWindowPos` Win32 API を呼び出し、`?` で Result をそのまま返す
  - doc comment で RAII パターン、`?` early return 時の安全性、パニック時の動作を説明
  - ロギング: `trace!` レベル、構造化フィールド（hwnd 16進数、x, y, cx, cy, flags）
  - _Requirements: 1.1, 1.2, 4.2_

- [ ] 1.3 SetWindowPosCommand の flush() をラッパー経由に変更
  - `ecs/window.rs` の `flush_window_pos_commands()` 内の `SetWindowPos` 直接呼び出しを `guarded_set_window_pos()` に置換
  - エラーハンドリングは既存の `warn!` ログ出力を維持
  - flush() の呼び出し箇所（3箇所: VsyncTick, WM_VSYNC, WM_WINDOWPOSCHANGED）は変更不要
  - _Requirements: 1.3_

- [ ] 2. WM_WINDOWPOSCHANGED ハンドラの簡素化
- [ ] 2.1 4ステップ→3ステップへの変更と echo 判定追加
  - `ecs/window_proc/handlers.rs` の `WM_WINDOWPOSCHANGED` ハンドラを変更
  - ステップ① 冒頭で `is_self_initiated()` を呼び出し、ローカル変数 `is_echo` に保存
  - ステップ① から `WindowPosChanged` コンポーネントへの `true` 設定を削除
  - ステップ① で `is_echo` が `true` の場合、`bypass_change_detection()` で `WindowPos` を更新（`Changed` 抑制）
  - ステップ① で `is_echo` が `false` の場合、`DerefMut` で `WindowPos` を更新（`Changed` 発火）
  - ステップ④（World 第2借用による `WindowPosChanged=false` リセット）を削除
  - `DpiChangeContext::take()` は `is_echo` にかかわらず常に実行
  - ロギング: `debug!` レベル、構造化フィールド（is_echo, entity, x, y, cx, cy）
  - _Requirements: 1.2, 1.4, 3.1, 3.2, 4.3_

- [ ] 3. WM_DPICHANGED ハンドラのラッパー統一
- [ ] 3.1 (P) SetWindowPos 直接呼び出しをラッパー経由に変更
  - `ecs/window_proc/handlers.rs` の `WM_DPICHANGED` ハンドラ内（L410 付近）の `SetWindowPos` 呼び出しを `guarded_set_window_pos()` に置換
  - `suggested_rect` パラメータ展開は既存ロジックを維持
  - `DpiChangeContext::set()` のライフサイクルは変更なし
  - エラーハンドリング（`warn!` ログ）は既存実装を維持
  - _Requirements: 1.5, 2.1, 2.2, 2.3, 2.4, 3.3_

- [ ] 4. apply_window_pos_changes システムのガード削除
- [ ] 4.1 (P) WindowPosChanged ガードと is_echo() ガード削除、last_sent_* bypass 削除
  - `ecs/graphics/systems.rs` の `apply_window_pos_changes` システムから以下を削除:
    - Query パラメータの `&WindowPosChanged` 削除
    - G1 ガード: `wpc.0 == true` チェック削除
    - G2 ガード: `window_pos.is_echo(position, size)` チェック削除
    - L808 付近の `last_sent_position` / `last_sent_size` への `bypass_change_detection()` 書き込み削除
  - G3 ガード（`CW_USEDEFAULT` チェック）は維持
  - `SetWindowPosCommand::enqueue()` 呼び出しは維持
  - _Requirements: 1.4, 3.2, 4.1_

- [ ] 5. WindowPos コンポーネントのクリーンアップ
- [ ] 5.1 (P) last_sent_* フィールドと is_echo() メソッドを削除
  - `ecs/window.rs` の `WindowPos` 構造体から以下を削除:
    - `pub last_sent_position: Option<(i32, i32)>` フィールド
    - `pub last_sent_size: Option<(i32, i32)>` フィールド
    - `pub fn is_echo(&self, position: (i32, i32), size: (i32, i32)) -> bool` メソッド
  - `Default` 実装から `last_sent_position: None, last_sent_size: None` を削除
  - `WindowPosChanged` コンポーネント定義（`pub struct WindowPosChanged(pub bool)`）を削除
  - `ecs/world.rs` の `WindowPosChanged` 参照コメントを更新（該当する場合）
  - _Requirements: 1.4, 4.1_

- [ ] 6. テスト更新と検証
- [ ] 6.1 layout_graphics_sync_test.rs の修正
  - `tests/layout_graphics_sync_test.rs` で `is_echo()` / `last_sent_*` / `bypass_change_detection` を使用している箇所を特定
  - 削除されたフィールド・メソッドへの参照を削除または代替実装に置換
  - テストロジックが TLS フラグ方式でも正しく動作するよう調整
  - テストが引き続き意図した検証を行うことを確認
  - _Requirements: 5.1, 5.2_

- [ ] 6.2 (P) TLS フラグ動作検証の単体テストを追加
  - `tests/` または `ecs/window.rs` 内の `#[cfg(test)]` モジュールに単体テストを追加
  - テストケース: `is_self_initiated()` の初期値が `false` であること
  - テストケース: `guarded_set_window_pos()` 呼び出し中に `is_self_initiated()` が `true` になること
  - テストケース: `guarded_set_window_pos()` 完了後に `is_self_initiated()` が `false` に戻ること
  - テストケース: `SetWindowPos` 失敗時（`Err` 返却）も TLS フラグが確実にリセットされること（モック可能な場合）
  - _Requirements: 5.4_

- [ ] 6.3 全テストスイートの実行と機能検証
  - `cargo test` を実行し、全テストがパスすることを確認
  - `cargo test --test feedback_loop_convergence_test` が全8テストをパスすることを確認
  - `cargo run --example taffy_flex_demo` でウィンドウ移動・リサイズのスムーズな動作を確認
  - マルチモニタ環境で DPI 変更（モニタ間移動）が正しく処理されることを確認（可能な場合）
  - フィードバックループの発生がないことを目視確認
  - _Requirements: 5.1, 5.2, 5.3_

## Requirements Coverage

| Requirement | Summary | Covered by Tasks |
|-------------|---------|------------------|
| 1.1, 1.2 | ラッパー + TLS フラグ | 1.1, 1.2 |
| 1.3 | キュー維持、flush ラッパー化 | 1.3 |
| 1.4 | 旧メカニズム削除 | 2.1, 4.1, 5.1 |
| 1.5 | WM_DPICHANGED ラッパー統一 | 3.1 |
| 2.1, 2.2, 2.3, 2.4 | DpiChangeContext 維持 | 3.1 |
| 3.1, 3.2, 3.3 | フィードバック防止正確性 | 2.1, 3.1, 4.1 |
| 4.1, 4.2, 4.3 | コード簡素化 | 1.1, 1.2, 2.1, 4.1, 5.1 |
| 5.1, 5.2, 5.3, 5.4 | 後方互換とテスト | 6.1, 6.2, 6.3 |

## Implementation Notes

### Parallel Execution Strategy
- Task 1.1 と 1.2 は並列実行可能（異なるコード要素）
- Task 3.1 は Task 2 完了後、Task 4.1 と並列実行可能（異なるハンドラ）
- Task 4.1 は Task 2 完了後、Task 3.1 と並列実行可能（異なるシステム）
- Task 5.1 は Task 4.1 完了後に実行（`is_echo()` 参照削除が前提）
- Task 6.2 は Task 1 完了後、Task 6.1 と並列実行可能（独立したテスト）

### Critical Path
1 → 2 → (3 || 4) → 5 → 6

### Design References
- Architecture Pattern: TLS ラッパーフラグ、既存 `DpiChangeContext` パターン踏襲
- Boundary Map: COM → ECS → Message Handling 責務分離維持
- Interface Contracts: `guarded_set_window_pos()` サービス契約、`IS_SELF_INITIATED` 状態管理契約
