# Implementation Plan

## Task Summary
- **Total**: 4 major tasks, 8 sub-tasks
- **Requirements Coverage**: 7 requirements (1-7) fully covered
- **Average Task Size**: 1-2 hours per sub-task

---

## Tasks

- [ ] 1. VSYNCカウンターの実装
  - Staticなアトミックカウンター変数を定義し、VSYNCスレッドからのシグナル通知基盤を構築する

- [ ] 1.1 (P) アトミックカウンター変数の定義
  - `win_thread_mgr.rs`にVSYNC到来回数を追跡するためのstatic変数を追加
  - 前回処理したtick値を記録するためのstatic変数を追加
  - 初期値は0、メモリオーダリングはRelaxedを使用
  - _Requirements: 1.2, 1.4, 1.5_

- [ ] 1.2 VSYNCスレッドでのカウンターインクリメント
  - VSYNCスレッドがDwmFlush()から復帰した直後にカウンターをインクリメント
  - インクリメント処理はWM_VSYNC送信より前に実行
  - fetch_add操作でアトミックにインクリメント
  - _Requirements: 1.1, 1.3_

- [ ] 2. EcsWorldにVSYNC駆動tick関数を追加
  - カウンター変化を検知してworld tickを実行する関数をEcsWorldに実装する

- [ ] 2.1 EcsWorld::try_tick_on_vsync()メソッドの実装
  - 現在のVSYNCカウンターと前回処理値を比較する
  - 値が異なる場合は前回処理値を更新してからtry_tick_world()を呼び出す
  - 前回処理値の更新はtry_tick_world()呼び出しより前に行う（再入時の重複防止）
  - tick実行有無をbool値で返す
  - _Requirements: 2.1, 2.3_

- [ ] 2.2 (P) VsyncTickトレイトの定義と実装
  - world.rsにVsyncTickトレイトを定義
  - Rc<RefCell<EcsWorld>>に対してトレイトを実装
  - try_borrow_mut()で借用を試み、成功時のみtry_tick_on_vsync()を呼び出す
  - 借用失敗時（再入時）は安全にスキップしてfalseを返す
  - _Requirements: 2.2, 2.4, 2.5_

- [ ] 3. WndProcとメッセージループへの統合
  - VSYNC駆動tick関数を実際の処理フローに組み込む

- [ ] 3.1 WM_WINDOWPOSCHANGED処理へのtick呼び出し追加
  - ecs_wndprocのWM_WINDOWPOSCHANGED処理の冒頭でtry_tick_on_vsync()を呼び出す
  - 既存のWindowPos/BoxStyle更新処理より前に実行
  - モーダルループ中（ウィンドウドラッグ中）の描画継続を実現
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [ ] 3.2 run()のWM_VSYNC処理の統一
  - run()メソッドのWM_VSYNC処理をtry_tick_on_vsync()呼び出しに変更
  - 既存のtry_tick_world()直接呼び出しを置き換える
  - WndProcで既に処理済みの場合はカウンター比較によりスキップ
  - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [ ] 4. 動作検証とデバッグサポート
  - 既存動作との互換性確認と診断機能の整備

- [ ] 4.1 taffy_flex_demoでの動作検証
  - ウィンドウドラッグ中の描画継続を確認
  - 通常動作時のフレームレート維持を確認
  - 外部API（WinThreadMgr::run()等）の互換性を確認
  - _Requirements: 5.1, 5.2, 5.3, 5.4_

- [ ] 4.2* (P) デバッグログ出力の追加（オプション）
  - tick実行時のログ出力機能を追加（デバッグビルド時）
  - WndProc経由とrun()経由のtick回数を区別して計測可能にする
  - 既存のフレームレート計測機能を維持
  - 設計意図と拡張ポイントをコメントで明記
  - _Requirements: 6.1, 6.2, 6.3, 7.1, 7.2, 7.3_

---

## Requirements Traceability Matrix

| Requirement | Tasks |
|-------------|-------|
| 1.1, 1.3 | 1.2 |
| 1.2, 1.4, 1.5 | 1.1 |
| 2.1, 2.3 | 2.1 |
| 2.2, 2.4, 2.5 | 2.2 |
| 3.1, 3.2, 3.3, 3.4 | 3.1 |
| 4.1, 4.2, 4.3, 4.4 | 3.2 |
| 5.1, 5.2, 5.3, 5.4 | 4.1 |
| 6.1, 6.2, 6.3, 7.1, 7.2, 7.3 | 4.2 |

## Notes

- タスク1.1と2.2は並列実行可能（ファイル・リソース競合なし）
- タスク4.2はオプションのデバッグ機能で、MVP後に実装可能
- 前回処理値の更新タイミングは再入時の重複tick防止のため重要（research.md参照）
