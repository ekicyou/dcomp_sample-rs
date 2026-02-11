# Implementation Plan

## Overview

`PointerState.screen_point` フィールドを `client_point` にリネームする。全参照箇所・コメント・tracing ログキーを一貫性を保ちながら更新し、ビルド成功・テスト全パスを確認する。

## Task Breakdown

### 1. PointerState 構造体定義の修正

- [x] 1.1 PointerState 構造体フィールド・doc コメント・Default 実装のリネーム
  - `crates/wintf/src/ecs/pointer/mod.rs` L116-118 の `screen_point: PhysicalPoint` フィールドを `client_point: PhysicalPoint` にリネーム
  - L117 の doc コメント「スクリーン座標（物理ピクセル）」を「クライアント座標（物理ピクセル）」に修正
  - L158 の Default 実装内の `screen_point: PhysicalPoint::default()` を `client_point: PhysicalPoint::default()` にリネーム
  - _Requirements: 1.1, 1.2, 1.3, 2.2, 4.1_

### 2. PointerState 参照箇所のリネーム

- [x] 2.1 (P) handlers.rs 内のポインター状態初期化のリネーム
  - `crates/wintf/src/ecs/window_proc/handlers.rs` L673, L736, L918, L1258 の 4箇所において、構造体リテラルのフィールド名 `screen_point:` を `client_point:` にリネーム
  - _Requirements: 2.1, 3.3_

- [x] 2.2 pointer/mod.rs 内のフィールドアクセス・コメント・tracing ログキーの修正
  - L506-510: `pointer.screen_point` → `pointer.client_point` のアクセス（5箇所）、L509 コメント「Phase 1ではscreen_pointと同じ」を「Phase 1ではclient_pointと同じ」に修正
  - L515-516: `new_x = pointer.screen_point.x` → `new_x = pointer.client_point.x`（ログ出力内、キー名は変更なし）
  - L618-619 tracing ログの構造化フィールド名 `screen_x = pointer.screen_point.x` → `client_x = pointer.client_point.x` に変更（座標キー名も更新）
  - L878: ユニットテスト内の `state.screen_point` → `state.client_point` にリネーム
  - L969-971: `pointer_state.screen_point` アクセスの 2箇所をリネーム
  - _Requirements: 2.3, 2.6, 4.2, 6.1, 6.2_

- [x] 2.3 (P) taffy_flex_demo.rs サンプルコード内のフィールドアクセスのリネーム
  - `crates/wintf/examples/taffy_flex_demo.rs` の全行（L779, L808, L999, L1141, L1188, L1231, L1243, L1288-1289, L1322）において、`state.screen_point.x` / `state.screen_point.y` をそれぞれ `state.client_point.x` / `state.client_point.y` にリネーム
  - _Requirements: 2.4, 3.3_

### 3. 検証・ビルド・テスト実行

- [x] 3.1 ビルド・テスト検証
  - `cargo build` を実行し、コンパイルがすべてのカスタマイズなしで成功（警告なし）することを確認
  - `cargo test` を実行し、既存テスト全てがパスすることを確認（pointer/mod.rs L878 のユニットテスト含む）
  - リネーム前後で `client_point` フィールド に格納される値（座標値）が変更されていないことを目視確認
  - _Requirements: 5.1, 5.2, 5.3_

- [x] 3.2 リネーム対象外の確認
  - `nchittest_cache.rs` 内の `screen_point` (型 `(i32, i32)`、WM_NCHITTEST 用）が未変更であることを確認
  - `hit_test.rs` 内の `screen_point` パラメータ（関数引数）が未変更であることを確認
  - _Requirements: 3.1, 3.2, 3.3_

## Requirement Coverage

| Requirement ID | Task | Notes |
|---|---|---|
| 1.1, 1.2, 1.3 | 1.1 | PointerState 構造体定義 |
| 2.1 | 2.1 | handlers.rs 初期化 |
| 2.2 | 1.1 | Default 実装 |
| 2.3 | 2.2 | pointer/mod.rs アクセス |
| 2.4 | 2.3 | taffy_flex_demo.rs |
| 2.6 | 2.2 | tracing ログキー |
| 3.1, 3.2, 3.3 | 3.2 | リネーム対象外の確認 |
| 4.1, 4.2 | 1.1, 2.2 | ドキュメント・コメント |
| 5.1, 5.2, 5.3 | 3.1 | ビルド・テスト検証 |
| 6.1, 6.2 | 2.2 | tracing ログ整合性 |

## Parallelization

- **2.1 (P)**: handlers.rs リネーム — 他のタスクへの依存なし、独立して実施可能
- **2.3 (P)**: taffy_flex_demo.rs リネーム — サンプルコード、独立して実施可能
- **2.2**: pointer/mod.rs — tracing ログキー名の更新含む、単独で集中すべき

## Notes

- 全 6つの Requirement が網羅されている
- 平均タスク所要時間: 1～3時間/サブタスク
- 並行実行可能タスク: 2.1, 2.3（handlers.rs と taffy_flex_demo.rs は独立）
