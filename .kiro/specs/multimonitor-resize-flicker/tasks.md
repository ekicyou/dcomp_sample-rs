# Implementation Plan

## Task Overview

マルチモニターDPI変更時のちらつき問題を解決するための実装タスク。

**有効な要件**: 1, 6, 7, 8, 9, 10, 11（計7件）

---

## Tasks

### Phase 1: 基盤コンポーネントの実装

- [ ] 1. DpiChangeContextスレッドローカル基盤の実装
- [ ] 1.1 (P) DpiChangeContext構造体とスレッドローカルストレージを定義する
  - DPI値と推奨RECTを保持する構造体を作成
  - thread_local!マクロでOption型のスレッドローカル変数を定義
  - set()とtake()の静的メソッドを実装
  - デバッグログ出力を追加（コンテキスト設定・消費時）
  - _Requirements: 8_

- [ ] 1.2 (P) SetWindowPosCommandキュー基盤を実装する
  - コマンド構造体（hwnd, 座標, フラグ）を定義
  - thread_local!マクロでVec型のスレッドローカルキューを定義
  - enqueue()とflush()の関数を実装
  - flush()内でSetWindowPos APIを順次呼び出し
  - デバッグログ出力を追加（キュー追加・実行時）
  - _Requirements: 11_

- [ ] 1.3 (P) WindowPosChanged ECSコンポーネントを定義する
  - bool型の単純なコンポーネントを定義
  - SparseSetストレージ属性を指定
  - Windowエンティティ作成時に初期化するよう既存処理を修正
  - デバッグログ出力を追加（フラグ変更時）
  - _Requirements: 9_

### Phase 2: メッセージ処理の改修

- [ ] 2. WM_DPICHANGED処理を同期型に変更する
  - WM_DPICHANGED受信時にDpiChangeContextを保存
  - DefWindowProcWを呼び出して推奨サイズを適用
  - PostMessageによる非同期処理を削除
  - デバッグログで変更前後のDPI、推奨RECTを出力
  - _Requirements: 1, 8, 10_

- [ ] 3. WM_WINDOWPOSCHANGED処理をWorld借用区切り方式に改修する
- [ ] 3.1 第1借用セクションを実装する
  - DpiChangeContextの消費とDPI更新を実装
  - WindowPosChangedフラグをtrueに設定
  - WindowPosとBoxStyleを更新
  - 借用を明示的に解放（スコープ終了）
  - _Requirements: 1, 8, 9_

- [ ] 3.2 tick実行とflush処理を組み込む
  - try_tick_on_vsync()を呼び出し（内部で借用→解放）
  - flush_window_pos_commands()を呼び出し
  - _Requirements: 11_

- [ ] 3.3 第2借用セクションを実装する
  - WindowPosChangedフラグをfalseにリセット
  - 借用を明示的に解放
  - _Requirements: 9_

### Phase 3: ECSシステムの修正

- [ ] 4. apply_window_pos_changesシステムを修正する
  - クエリにWindowPosChangedの不変参照を追加
  - フラグがtrueの場合はSetWindowPosCommand生成を抑制
  - フラグがfalseの場合は従来通りコマンドをキューに追加
  - SetWindowPos直接呼び出しをenqueue()に置換
  - デバッグログで抑制発生を出力
  - _Requirements: 9, 11_

- [ ] 5. VsyncTickトレイト実装を拡張する
  - try_tick_on_vsync()の戻り後にflush_window_pos_commands()を呼び出し
  - World借用解放後の安全なタイミングで実行
  - _Requirements: 11_

### Phase 4: レガシーコード削除

- [ ] 6. WM_DPICHANGED_DEFERRED関連コードを削除する
  - カスタムメッセージ定義を削除
  - post_dpi_change()関数を削除
  - process_deferred_dpi_change()関数を削除
  - 関連する呼び出し箇所をすべて削除
  - _Requirements: 10_

### Phase 5: 統合テストと検証

- [ ] 7. 手動統合テストを実施する
- [ ] 7.1 DPI変更シナリオを検証する
  - 異なるDPIモニター間でウィンドウを移動
  - 論理サイズ（DIP）が維持されることを確認
  - ちらつきが発生しないことを確認
  - デバッグログで処理フローを確認
  - _Requirements: 1_

- [ ] 7.2 既存動作の回帰テストを実施する
  - ウィンドウリサイズ操作（端のドラッグ）が正常に動作
  - 最大化・最小化操作が正常に動作
  - BoxStyle更新によるプログラムからのサイズ変更が正常に動作
  - 単一モニター環境で問題がないことを確認
  - _Requirements: 6_

- [ ]\* 7.3 デバッグログ出力を検証する
  - DPI変更イベントのログが出力される
  - WindowPosChangedフラグ抑制のログが出力される
  - SetWindowPosCommandキュー操作のログが出力される
  - _Requirements: 7_

---

## Requirements Coverage

| Requirement | Tasks |
|-------------|-------|
| 1 | 1.1, 2, 3.1, 7.1 |
| 6 | 1.2, 1.3, 4, 7.2 |
| 7 | 1.1, 1.2, 1.3, 7.3 |
| 8 | 1.1, 2, 3.1 |
| 9 | 1.3, 3.1, 3.3, 4 |
| 10 | 2, 6 |
| 11 | 1.2, 3.2, 4, 5 |

**廃止済み要件**: 2, 3, 4, 5（REQ-009, REQ-011に統合）
