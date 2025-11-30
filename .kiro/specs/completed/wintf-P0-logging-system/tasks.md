# Implementation Plan

## Task Summary
- **Feature**: wintf-P0-logging-system
- **Total**: 4 major tasks, 10 sub-tasks
- **Requirements Coverage**: 1, 2, 3, 4, 5 (all 5 requirements)

---

## Tasks

- [x] 1. 依存関係の設定
- [x] 1.1 ワークスペースにtracing依存を追加
  - workspace Cargo.tomlに`tracing`と`tracing-subscriber`を追加
  - workspace.dependenciesセクションでバージョン統一管理
  - env-filter featureを有効化
  - _Requirements: 1.1, 1.4_

- [x] 1.2 wintfクレートにtracing依存を追加
  - crates/wintf/Cargo.tomlにworkspace参照を追加
  - lib.rsにtracingマクロのuse文を追加（必要に応じて）
  - ビルド確認
  - _Requirements: 1.1, 1.2_

- [x] 2. ライブラリ内のeprintln!置換（コア部分）
- [x] 2.1 (P) スレッド管理・プロセス関連のログ置換
  - win_thread_mgr.rs（6箇所）のeprintln!をtracing マクロに変換
  - winproc.rs（4箇所）のログを置換
  - process_singleton.rs（2箇所）のログを置換
  - ログレベル: 初期化=info、詳細=debug、エラー=warn/error
  - _Requirements: 1.2, 2.1, 4.2_

- [x] 2.2 (P) ECSコア部分のログ置換
  - ecs/world.rs（1箇所）のログを置換
  - ecs/window_system.rs（5箇所）のログを置換
  - 構造化フィールドの追加（必要に応じてEntity ID等）
  - _Requirements: 1.2, 2.1, 3.3, 4.2_

- [x] 3. ライブラリ内のeprintln!置換（ウィジェット部分）
- [x] 3.1 (P) ウィンドウ関連のログ置換
  - ecs/window_proc.rs（10+箇所）のWMメッセージ処理ログを置換
  - ecs/window.rs（10+箇所）のウィンドウ状態ログを置換
  - WMメッセージ詳細はtraceレベル、重要イベントはinfo/debug
  - HWNDフィールドの構造化出力を追加
  - _Requirements: 1.2, 2.1, 3.1, 3.2, 3.3, 4.2_

- [x] 3.2 (P) ウィジェット描画関連のログ置換
  - ecs/widget/text/draw_labels.rs（10+箇所）のログを置換
  - ecs/widget/shapes/rectangle.rs（3+箇所）のログを置換
  - 描画エラーはwarn、詳細計算はtrace
  - 追加: monitor.rs, visual_manager.rs, graphics/systems.rs, layout/systems.rs, layout/metrics.rs, graphics/core.rs, app.rs, label.rsも置換
  - _Requirements: 1.2, 2.1, 4.2_

- [x] 4. 参考実装と検証
- [x] 4.1 taffy_flex_demoにSubscriber初期化を追加
  - main関数先頭にtracing-subscriber初期化コードを追加
  - EnvFilterによるRUST_LOG環境変数対応
  - デフォルトフィルタレベルをinfoに設定
  - _Requirements: 4.1, 4.3, 4.4, 2.2_

- [x] 4.2 ログレベルフィルタリングの動作確認
  - デフォルト実行でinfoレベル以上が出力されることを確認
  - RUST_LOG=debug で詳細ログが出力されることを確認
  - RUST_LOG=wintf=trace でwintf限定のtraceログを確認
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [x] 4.3 パフォーマンス検証
  - ログシステム導入前後でGUI動作の滑らかさを確認
  - メッセージループ内のログがフレームレートに影響しないことを確認
  - Subscriber未設定時にゼロコストで動作することを確認
  - _Requirements: 5.1, 5.2, 5.3_

---

## Requirements Coverage Matrix

| Requirement | Tasks |
|-------------|-------|
| 1.1 | 1.1, 1.2 |
| 1.2 | 1.2, 2.1, 2.2, 3.1, 3.2 |
| 1.3 | (設計方針による自動達成) |
| 1.4 | 1.1 |
| 2.1 | 2.1, 2.2, 3.1, 3.2, 4.2 |
| 2.2 | 4.1, 4.2 |
| 2.3 | 4.2 |
| 2.4 | 4.2 |
| 3.1 | 3.1 |
| 3.2 | 3.1 |
| 3.3 | 2.2, 3.1 |
| 4.1 | 4.1 |
| 4.2 | 2.1, 2.2, 3.1, 3.2 |
| 4.3 | 4.1 |
| 4.4 | 4.1 |
| 5.1 | 4.3 |
| 5.2 | 4.3 |
| 5.3 | 4.3 |

## Parallel Execution Notes
- タスク 2.1, 2.2 は並列実行可能（異なるファイル群を対象）
- タスク 3.1, 3.2 は並列実行可能（異なるファイル群を対象）
- タスク 1.x は 2.x, 3.x の前提条件（依存関係設定が必須）
- タスク 4.x は 2.x, 3.x 完了後に実行（ログ出力の検証のため）
