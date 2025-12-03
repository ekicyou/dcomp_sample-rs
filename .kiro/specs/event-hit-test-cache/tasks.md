# Implementation Plan: event-hit-test-cache

| 項目 | 内容 |
|------|------|
| **Feature** | event-hit-test-cache |
| **Generated** | 2025-12-03 |
| **Total Tasks** | 3 major, 6 sub-tasks |

---

## Tasks

- [x] 1. キャッシュモジュールの作成
- [x] 1.1 キャッシュストレージの実装
  - スレッドローカル変数でウィンドウごとのキャッシュエントリを管理
  - キャッシュエントリにスクリーン座標と LRESULT を保持
  - HashMap を使用して HWND をキーとしたエントリ管理
  - thread_local! + RefCell パターンで内部可変性を提供
  - _Requirements: 1_

- [x] 1.2 キャッシュ操作APIの実装
  - キャッシュルックアップ機能（HWND + 座標でエントリ検索）
  - キャッシュ挿入機能（新規エントリまたは更新）
  - キャッシュクリア機能（全エントリ削除）
  - _Requirements: 1, 3_

- [x] 2. 公開APIの実装
- [x] 2.1 cached_nchittest 関数の実装
  - キャッシュヒット判定（HWND + 座標の一致確認）
  - キャッシュヒット時は World 借用なしで LRESULT を返却
  - キャッシュミス時は hit_test_in_window を呼び出し
  - ヒットテスト結果を LRESULT に変換しキャッシュに格納
  - スクリーン座標からクライアント座標への変換を実行
  - _Requirements: 2, 4_

- [x] 2.2 clear_nchittest_cache 関数の実装
  - 全ウィンドウのキャッシュエントリをクリア
  - 外部から呼び出し可能な公開関数として定義
  - _Requirements: 3, 4_

- [x] 3. システム統合
- [x] 3.1 WM_NCHITTEST ハンドラへの統合
  - 既存の WM_NCHITTEST ハンドラを cached_nchittest 呼び出しに変更
  - スクリーン座標の抽出ロジックは維持
  - エラー時は DefWindowProcW に委譲する既存動作を維持
  - _Requirements: 2, 4_

- [x] 3.2 try_tick_world へのキャッシュクリア統合
  - try_tick_world 終了時に clear_nchittest_cache を呼び出し
  - Layout スケジュール実行後のタイミングでクリア
  - _Requirements: 3_

---

## Requirements Coverage

| Requirement | Tasks |
|-------------|-------|
| 1 (スレッドローカルキャッシュ) | 1.1, 1.2 |
| 2 (キャッシュヒット判定) | 2.1, 3.1 |
| 3 (キャッシュクリア) | 1.2, 2.2, 3.2 |
| 4 (キャッシュ公開API) | 2.1, 2.2, 3.1 |

---

## Execution Notes

- タスク 1.x は並行実行不可（1.2 は 1.1 の構造体に依存）
- タスク 2.x は 1.x 完了後に実行可能
- タスク 3.x は 2.x 完了後に実行（統合テストとして機能確認）
- 全タスクが小規模（各1-2時間）のため、順次実行を推奨
