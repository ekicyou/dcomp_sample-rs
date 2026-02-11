# Implementation Plan

## Overview

本仕様は調査レポート作成が成果物であり、コード実装は含まれません。既存の調査成果（research.md, gap-analysis.md）を基盤として、構造化された report.md を執筆します。

## Tasks

- [ ] 1. (P) 座標系インベントリセクション執筆
  - 全 ECS コンポーネントの座標フィールドを Markdown table で一覧化（コンポーネント名 | フィールド | 座標系 | ファイルパス:行番号 | 意図的/不整合）
  - `BoxStyle.inset` 物理px と `BoxStyle.size` DIP の混在箇所を特定し、意図的設計であることを判定結果に記載
  - レガシー `WinState` trait と ECS `DPI` コンポーネントの使用箇所・責務の違いを文書化し、「ECS 移行完了後に廃止」の統合方針を提示
  - Win32 API 全座標系（`SetWindowPos`, `GetCursorPos`, `AdjustWindowRectExForDpi` 等）の入出力座標系を Per-Monitor v2 下での挙動と共に一覧化
  - gap-analysis.md の座標系コンポーネント一覧を基盤とし、Win32 API マトリクスを追加
  - _Requirements: 1.1, 1.2, 1.3, 1.4_

- [ ] 2. (P) WM メッセージ伝搬マトリクスセクション執筆
  - DPI/座標に影響する全 WM メッセージ（16種）を Markdown table で一覧化（メッセージ | 入力座標系 | ECS出力先 | Win32 API呼出 | トリガーWM | 循環リスク）
  - メッセージカスケードチェーン 5 種類を Mermaid sequence diagram で図示（WM_DPICHANGED → SetWindowPos → WM_WINDOWPOSCHANGED 等）
  - フィードバックループ防止機構 3 層（WindowPosChanged フラグ / エコーバック検知 / RefCell 再入保護）を Markdown table で評価（層 | メカニズム | 有効範囲 | 潜在リスク）
  - ハンドルされていない注目メッセージ（`WM_GETDPISCALEDSIZE`, `WM_NCCALCSIZE` 等 6 種類）の影響を評価
  - 座標劣化パス（f32→i32 丸め、PointerState.screen_point 命名不整合等）を数値トレース付きで特定
  - research.md のトピック 1・2 をレポート形式に整形
  - _Requirements: 1.4, 2.1, 2.2_

- [ ] 3. (P) DPI データフローセクション執筆
  - `GetDpiForWindow` → `DPI` → `Arrangement.scale` → `GlobalArrangement` → `Visual`/`Surface`/`SetWindowPos` の全経路を Mermaid flowchart で図示（取得元 → 保持 → 伝播 → 消費の 4 段階）
  - `DpiChangeContext` スレッドローカルによる `WM_DPICHANGED` → `WM_WINDOWPOSCHANGED` 間の同期伝達パスを検証し、DPI 値の正しい受け渡しを確認
  - 二重スケーリング検証結果を記載（Window の `Mul` パスで `parent_scale=1.0` のため現状は問題なし）
  - `Monitor.dpi` が `Arrangement.scale` に未反映である影響を評価
  - スケール値追跡表を Markdown table で作成（パイプラインステージ | scale値 | 座標系）
  - _Requirements: 2.1, 2.2, 2.3, 2.4_

- [ ] 4. (P) ドラッグ座標変換チェーンセクション執筆
  - As-Is チェーン: `WM_MOUSEMOVE` → `DragEvent` → ハンドラ → `BoxStyle` → `SetWindowPos` の全 10 ステップを Mermaid sequence diagram で図示（物理px フロー）
  - To-Be チェーン: DIP 統一時の理想的な変換チェーンを併記（DIP 内部フロー）
  - As-Is / To-Be 並列 Mermaid sequence diagram を作成し、コンフリクト箇所を指摘（`BoxStyle.inset` 物理px → DIP 変更、ドラッグデルタの DIP 変換必要性）
  - DPI=120 (1.25x) 条件での数値トレース表を作成（各ステップでの座標値を追跡）
  - `sync_window_arrangement_from_window_pos`（無効化中）の設計意図を文書化し、To-Be における有効化条件（BoxStyle.inset が DIP 統一後）を評価
  - gap-analysis.md のドラッグチェーン 10 ステップ分析を基盤として整形
  - _Requirements: 3.1, 3.2, 3.3_

- [ ] 5. (P) To-Be アーキテクチャセクション執筆
  - WPF の「デバイス独立ユニット（DIP）統一モデル」および WinUI3 の DPI スケーリングモデルを要約し、wintf への適用パターンを提示
  - 「全内部座標を DIP で統一し、物理px 変換は出力層のみ」方針の妥当性を評価（DIP Layer Rule, Output Conversion Rule, Input Conversion Rule, Single Authority Rule として 4 原則を定義）
  - As-Is vs To-Be の座標フロー比較を Mermaid graph で図示
  - `BoxStyle` 座標系統一方針と影響範囲を Markdown table で列挙（コンポーネント | 現在の座標系 | To-Be 座標系 | 変更内容 | 影響範囲）
  - `Arrangement.offset` と `WindowPos.position` の関係を再定義し、`sync_window_arrangement_from_window_pos` 有効化の前提条件を明確化
  - Per-Monitor DPI v2 環境でのモニタ間移動時の DPI 変更再計算フローを Mermaid sequence diagram で定義
  - research.md のアーキテクチャ評価（4 オプション）と設計判断（3 項目）を基盤として整形
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

- [ ] 6. ギャップ分析マトリクスセクション執筆
  - As-Is / To-Be ギャップを Markdown table で一覧化（# | ギャップ | 影響度 | コスト | ブロック仕様 | 優先度 | クロスリファレンス）
  - 各ギャップに「影響度（High/Medium/Low）」「修正コスト（High/Medium/Low）」「ブロックしている仕様」を評価
  - `dpi-propagation`（完了済）と `wintf-P1-dpi-scaling`（バックログ）の差分を Markdown table で明確化
  - 各ギャップに既存仕様（`dpi-propagation`, `wintf-P1-dpi-scaling`, `event-drag-system`）へのクロスリファレンスを付与
  - タスク 1-5 の成果物（座標系インベントリ、WM マトリクス、DPI フロー、ドラッグチェーン、To-Be アーキテクチャ）を統合してギャップを抽出
  - _Requirements: 5.1, 5.3_

- [ ] 7. 改善ロードマップセクション執筆
  - 段階的移行と一括移行のアプローチを Markdown table で比較（アプローチ | メリット | リスク | 所要期間）
  - フェーズ分割案を提示（Phase 1: Window DIP 化 → Phase 2: Widget DIP 化 → Phase 3: LayoutRoot/Monitor DIP 化）
  - 各フェーズの前提条件・検証基準・ロールバック条件を Markdown table で定義
  - タスク 6 のギャップマトリクスの優先度情報を基盤としてロードマップを策定
  - _Requirements: 5.2, 5.3_

- [ ] 8. エグゼクティブサマリーセクション執筆
  - 調査の背景・目的・主要発見事項・推奨事項を 1 ページ以内で要約
  - 主要発見事項: BoxStyle.inset/size 混在、WM メッセージ 3 層防御の冗長性、sync_window_pos 重複、WPF DIP 統一モデル採用推奨
  - 推奨事項: 段階的 DIP 統一移行、フィードバック防止機構の簡素化、WM メッセージ伝搬マトリクスの継続更新
  - 後続仕様（`wintf-P1-dpi-scaling`）への具体的なアクション提言を記載
  - タスク 1-7 の全セクション完成後にサマリーを抽出
  - _Requirements: 6.2_

- [ ] 9. レポート統合・検証・最終化
  - タスク 1-8 で執筆した全セクションを requirements.md Req 6.2 定義の順序で結合（エグゼクティブサマリー、座標系インベントリ、WM メッセージマトリクス、DPI フロー、ドラッグチェーン、To-Be、ギャップマトリクス、ロードマップ）
  - Mermaid フロー図の構文を検証（すべてレンダリング可能であることを確認）
  - Markdown table のフォーマットを統一（列幅・区切り文字）
  - 各改善提案に `dpi-propagation`, `wintf-P1-dpi-scaling`, `event-drag-system` へのクロスリファレンスが付与されていることを確認
  - `.kiro/specs/dpi-coordinate-transform-survey/report.md` として出力
  - 全 24 受入基準（Req 1.1-1.4, 2.1-2.4, 3.1-3.3, 4.1-4.5, 5.1-5.3, 6.1-6.4）がレポートで満たされていることを検証
  - _Requirements: 6.1, 6.3, 6.4_

## Progress Tracking

- **Total Tasks**: 9 major tasks, 0 sub-tasks
- **Parallel Execution**: Tasks 1-5 can run concurrently (marked with `(P)`)
- **Sequential Dependencies**: 
  - Task 6 depends on Tasks 1-5
  - Task 7 depends on Task 6
  - Task 8 depends on Tasks 1-7
  - Task 9 depends on Tasks 1-8
- **Estimated Effort**: 1-2 hours per section task, 2-3 hours for integration task

## Notes

- 本仕様はコード実装を含まない調査レポート作成仕様です
- research.md (300行) と gap-analysis.md (404行) が既存成果物として完成済み
- report.md は既存成果物の構造化・統合作業として実施
- すべての Mermaid 図は research.md および design.md で既に設計済み
- タスク実行時は既存の調査データを参照し、レポート形式に整形することに注力
