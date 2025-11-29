# Implementation Plan

本タスクリストは `ukagaka-desktop-mascot` の設計に基づく実装タスクを定義する。

> **Note**: 本仕様は「メタ仕様」であり、全31要件を子仕様に分解して統括する。本タスクはPhase 1で全子仕様の要件定義原案を作成し、Phase 2以降は子仕様の実装フェーズを参照する形式をとる。

---

## Phase 1: 子仕様書の作成

### 1.A 既存子仕様（作成済み: 8件）

- [x] 1.1 (P0) wintf-P0-image-widget 仕様書を作成する
  - WIC画像読み込み、D2D描画、透過PNG対応の要件定義
  - GIF/WebPアニメーション画像のフレーム抽出要件
  - タイマー駆動再生の要件
  - _Requirements: 1.1, 1.3, 2.4_

- [x] 1.2 (P0) wintf-P0-event-system 仕様書を作成する
  - ヒットテストシステムの要件定義
  - マウスイベント（クリック、ドラッグ、ホバー）配信の要件
  - キャラクターウィンドウのドラッグ移動要件
  - _Requirements: 5.1, 5.2, 5.3, 5.8_

- [x] 1.3 (P0) wintf-P0-typewriter 仕様書を作成する
  - 文字単位の表示制御要件
  - 現Labelからの拡張範囲定義
  - ウェイト制御の要件
  - _Requirements: 3.5, 4.7_

- [x] 1.4 (P0) areka-P0-reference-ghost 仕様書を作成する
  - MVP参照ゴーストの要件定義
  - 里々インスパイアDSLの基本構文定義
  - 2体キャラクター掛け合い会話の要件
  - MCP通信インターフェースの要件
  - _Requirements: 4.1, 4.2, 4.4, 4.5, 4.6, 26.1, 26.2, 26.3_

- [x] 1.5 (P0) areka-P0-reference-shell 仕様書を作成する
  - MVP参照シェルの要件定義
  - サーフェス画像仕様
  - アニメーション定義仕様
  - ヒット領域定義仕様
  - _Requirements: 2.2, 2.7, 8.1, 8.3, 27.10, 27.11, 27.12, 27.13_

- [x] 1.6 (P0) areka-P0-reference-balloon 仕様書を作成する
  - MVP参照バルーンの要件定義
  - スタイル定義仕様（フォント、色、背景）
  - 縦書き/横書き対応要件
  - _Requirements: 3.4, 3.6, 27.15, 27.16_

- [x] 1.7 (P1) wintf-P1-clickthrough 仕様書を作成する
  - 透過領域のクリックスルー（マウスイベント透過）
  - 不透明領域のヒット判定
  - WM_NCHITTESTハンドリング
  - レイヤードウィンドウとの統合
  - _Requirements: 1.6, NFR-3_

- [x] 1.8 (P0) areka-P0-window-placement 仕様書を作成する
  - キャラクターウィンドウの配置ルール
  - タスクバー張り付き、画面端配置
  - マルチモニター対応
  - 複数キャラクター間の相対位置管理
  - 配置の保存・復元
  - _Requirements: 1.4, 1.5, 1.7, 9.3, 16.6_

### 1.B 追加子仕様（未作成: 17件）

#### wintf-* 層（3件）

- [ ] 1.9 (P0) wintf-P0-animation-system 仕様書を作成する
  - フレームアニメーション定義と再生
  - サーフェス切り替えトランジション
  - 連動アニメーション（複数キャラクター同期）
  - アイドルアニメーション自動再生
  - _Requirements: 2.1, 2.3, 2.5, 2.6, 2.8_

- [ ] 1.10 (P0) wintf-P0-balloon-system 仕様書を作成する
  - バルーンウィンドウ生成・配置
  - テキスト表示（縦書き/横書き）
  - 選択肢UI、入力ボックス
  - ルビ表示、リンククリック
  - _Requirements: 3.1, 3.2, 3.3, 3.7, 3.8, 3.9, 3.10_

- [ ] 1.11 (P1) wintf-P1-dpi-scaling 仕様書を作成する
  - 高DPI環境でのスケーリング
  - Per-Monitor DPI対応
  - DPI変更時の動的更新
  - _Requirements: 15.1, 15.2, NFR-1_

#### areka-* コア層（9件）

- [ ] 1.12 (P0) areka-P0-script-engine 仕様書を作成する
  - さくらスクリプト互換コマンド解析
  - 変数管理（グローバル/ローカル）
  - 条件分岐、ループ、関数呼び出し
  - 2体キャラクター会話制御（スコープ切替）
  - _Requirements: 4.1, 4.3, 4.4, 4.5, 4.6, 4.7, 29.6, 29.7, 29.8_

- [ ] 1.13 (P1) areka-P1-timer-events 仕様書を作成する
  - システム時刻イベント（朝/昼/夜等）
  - 予約イベント（誕生日、記念日）
  - スリープ復帰、ネットワーク変化
  - カスタムタイマー、アイドル検出
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7, 6.8_

- [ ] 1.14 (P0) areka-P0-package-manager 仕様書を作成する
  - パッケージ（頭脳/シェル/バルーン）インストール/アンインストール
  - manifest.toml解析、依存関係解決
  - アップデート検知、保護フォルダ対応
  - メタ情報（作者、ライセンス）表示
  - _Requirements: 7.1-7.7, 8.1-8.5, 27.1-27.27, 31.1-31.9_

- [ ] 1.15 (P0) areka-P0-persistence 仕様書を作成する
  - アプリケーション設定の保存/読み込み
  - ゴースト状態（変数、記憶）の永続化
  - 定期的自動保存
  - エクスポート/インポート
  - _Requirements: 9.1, 9.2, 9.4, 9.5, 9.6, 30.6, 30.7, 30.8_

- [ ] 1.16 (P0) areka-P0-mcp-server 仕様書を作成する
  - MCPサーバー基盤（JSON-RPC 2.0）
  - MCPツール定義（display_text, switch_surface等）
  - イベント通知（OnMouseClick, OnTimer等）
  - ゴースト間通信の媒介
  - _Requirements: 10.1, 10.2, 10.3, 10.4, 10.5, 26.11, 26.12, 26.13, 26.14, 26.15_

- [ ] 1.17 (P1) areka-P1-legacy-converter 仕様書を作成する
  - 旧シェル定義（descript.txt, surfaces.txt）変換
  - 座標ベースレイアウトから現代的レイアウトへの変換
  - さくらスクリプト互換出力
  - SHIORI互換プロトコル
  - _Requirements: 11.1-11.6, 29.1-29.11_

- [ ] 1.18 (P1) areka-P1-devtools 仕様書を作成する
  - リアルタイムログ表示
  - デバッグモード切替、エラー詳細表示
  - ホットリロード、イベントシミュレーター
  - パッケージバリデーション
  - _Requirements: 12.1-12.7, 28.1-28.10_

- [ ] 1.19 (P0) areka-P0-system-tray 仕様書を作成する
  - システムトレイアイコン表示
  - トレイメニュー
  - 最小化時のトレイ格納
  - Windows起動時の自動起動
  - _Requirements: 13.1, 13.2, 13.3, 13.4, 13.5_

- [ ] 1.20 (P1) areka-P1-error-recovery 仕様書を作成する
  - クラッシュログ（会話ログ、イベント履歴、MCP log）
  - スタックトレース記録
  - 状態復元（表示位置、起動ゴースト）
  - エラー通知と継続動作
  - _Requirements: 30.1, 30.2, 30.3, 30.4, 30.5, 30.9, 30.10_

#### areka-* 拡張層（5件）

- [ ] 1.21 (P2) areka-P2-presence-style 仕様書を作成する
  - 存在スタイル（控えめ/標準/活発）選択
  - 時間帯・作業状況による自動調整
  - フルスクリーン時の自動非表示
  - 移動範囲制限
  - _Requirements: 16.1, 16.2, 16.3, 16.4, 16.5, 16.7_

- [ ] 1.22 (P2) areka-P2-memory-system 仕様書を作成する
  - 会話履歴の永続保存
  - ユーザー情報記憶（名前、好み）
  - 過去の文脈参照
  - RAG連携
  - 成長パラメータ（好感度、親密度）
  - _Requirements: 17.1-17.8_

- [ ] 1.23 (P2) areka-P2-llm-integration 仕様書を作成する
  - ローカルLLM（llama.cpp, Ollama）連携
  - クラウドAPI（OpenAI, Claude）連携
  - 人格設定（システムプロンプト）
  - 人格フィルター、不適切応答フィルタリング
  - キャラクター間LLM会話
  - _Requirements: 18.1-18.7, 4.8, 4.9, 4.10, 26.7-26.10, 26.16-26.20_

- [ ] 1.24 (P2) areka-P2-creator-tools 仕様書を作成する
  - 新規パッケージテンプレート生成
  - 人格テンプレート（LLMプロンプト）共有
  - AI人格自動生成
  - 派生・アレンジ作成
  - クリエイター支援リンク
  - _Requirements: 24.1-24.8_

- [ ] 1.25 (P2) areka-P2-privacy-security 仕様書を作成する
  - ローカルファースト設計
  - 外部API送信時の同意確認
  - 会話履歴暗号化
  - 「秘密」マーク機能
  - 選択的削除
  - _Requirements: 25.1-25.5, NFR-3_

#### areka-* 将来層（4件）

- [ ] 1.26 (P3) areka-P3-voice-system 仕様書を作成する
  - 音声合成（VOICEVOX, Style-BERT-VITS2等）連携
  - キャラクターごとの音声設定
  - 音声認識、ウェイクワード
  - 連続音声対話
  - _Requirements: 19.1-19.8_

- [ ] 1.27 (P3) areka-P3-screen-awareness 仕様書を作成する
  - アクティブウィンドウ認識
  - スクリーンショット認識（オプション）
  - エラー/ビルドエラー検知
  - 離席検出、「おかえり」機能
  - _Requirements: 20.1-20.8_

- [ ] 1.28 (P3) areka-P3-environment-sense 仕様書を作成する
  - 時刻、曜日、季節
  - 天気情報取得
  - 祝日、イベント
  - PC状態（バッテリー、負荷）
  - MCPサーバー連携（外部ツール）
  - _Requirements: 21.1-21.7, 22.1-22.7_

- [ ] 1.29 (P3) areka-P3-cloud-sync 仕様書を作成する
  - ゴースト状態エクスポート/インポート
  - クラウド同期
  - デバイス間認識
  - 同期競合解決
  - _Requirements: 23.1-23.6_

#### areka-* IDE層（1件）

- [ ] 1.30 (P3) areka-P3-ide-integration 仕様書を作成する
  - DAP（Debug Adapter Protocol）サーバー
  - LSP（Language Server Protocol）サーバー
  - ブレークポイント、ステップ実行
  - コード補完、構文エラー通知
  - _Requirements: 28.11-28.20_

#### キャラクター間通信（1件）

- [ ] 1.31 (P1) areka-P1-character-communication 仕様書を作成する
  - ゴースト内キャラクター間会話（スクリプトベース）
  - 関係性パラメータ（親密度、ライバル度）
  - ゴースト間LLM会話（オプション）
  - ゴースト間物理インタラクション（近接、相対位置）
  - 会話トリガー、傍観モード
  - _Requirements: 26.1-26.37_

#### 開発プロセス支援（1件）

- [ ] 1.32 (P2) kiro-P2-roadmap-management 仕様書を作成する
  - 全体計画の駆動文書（ロードマップ管理）の作成
  - steering ドキュメントへの開発フォーカス記載
  - 複数子仕様の進捗追跡の仕組み
  - AIエージェントが全体計画を認識する方法の検証
  - _背景: 設計レビュー Issue #3 での議論を受けて追加_
  - _Note: ukagaka-desktop-mascot 駆動完了後に実施_

---

## Phase 2+: 子仕様実装フェーズ

> **Note**: Phase 2以降は各子仕様の実装フェーズに委譲される。子仕様ごとに `/kiro-spec-impl {child-spec}` を実行して実装を進める。

### 実装優先順位

本仕様は全31要件を31件の子仕様に分解する「メタ仕様」である。実装は以下の優先順位で各子仕様を順次実行する。

| 優先度 | 分類 | 子仕様群 | 目標 |
|--------|------|----------|------|
| **P0** | MVP必須 | wintf-P0-image-widget, wintf-P0-event-system, wintf-P0-typewriter, wintf-P0-balloon-system, wintf-P0-animation-system, areka-P0-script-engine, areka-P0-package-manager, areka-P0-persistence, areka-P0-mcp-server, areka-P0-system-tray, areka-P0-reference-ghost, areka-P0-reference-shell, areka-P0-reference-balloon, areka-P0-window-placement | 2体キャラクター掛け合い会話可能 |
| **P1** | リリース品質 | wintf-P1-clickthrough, wintf-P1-dpi-scaling, areka-P1-legacy-converter, areka-P1-devtools, areka-P1-error-recovery, areka-P1-timer-events, areka-P1-character-communication | 互換性・安定性・開発支援 |
| **P2** | 差別化機能 | areka-P2-presence-style, areka-P2-memory-system, areka-P2-llm-integration, areka-P2-creator-tools, areka-P2-privacy-security, kiro-P2-roadmap-management | 独自価値の創出 |
| **P3** | 将来展望 | areka-P3-voice-system, areka-P3-screen-awareness, areka-P3-environment-sense, areka-P3-cloud-sync, areka-P3-ide-integration | 長期ロードマップ |

---

## Phase 2: 子仕様の要件定義承認

Phase 1で作成した31件の子仕様原案について、順次レビューと承認を行う。

- [ ] 2.1 P0子仕様の要件レビューと承認（14件）
  - 各子仕様の `/kiro-spec-requirements {child}` を完了状態にする
  - 親仕様の要件トレーサビリティを検証

- [ ] 2.2 P1子仕様の要件レビューと承認（7件）
  - 各子仕様の `/kiro-spec-requirements {child}` を完了状態にする
  - 親仕様の要件トレーサビリティを検証

- [ ] 2.3 P2子仕様の要件レビューと承認（5件）
  - 各子仕様の `/kiro-spec-requirements {child}` を完了状態にする
  - 親仕様の要件トレーサビリティを検証

- [ ] 2.4 P3子仕様の要件レビューと承認（5件）
  - 各子仕様の `/kiro-spec-requirements {child}` を完了状態にする
  - 親仕様の要件トレーサビリティを検証

---

## Phase 3: 子仕様の設計・タスク定義

各子仕様について設計とタスク定義を行う。

- [ ] 3.1 P0子仕様の設計・タスク定義
  - `/kiro-spec-design {child}` → `/kiro-spec-tasks {child}` を順次実行
  - 依存関係を考慮した実行順序で進める

- [ ] 3.2 P1子仕様の設計・タスク定義
  - P0完了後に順次実行

- [ ] 3.3 P2子仕様の設計・タスク定義
  - P1の主要機能完了後に順次実行

- [ ] 3.4 P3子仕様の設計・タスク定義
  - 長期計画として準備、実行は市場反応を見て判断

---

## Phase 4: 子仕様の実装

各子仕様の実装は、それぞれの仕様内で `/kiro-spec-impl {child}` を実行して進める。

### 4.1 P0実装（MVP達成）

- [ ] 4.1.1 wintf基盤層の実装
  - `/kiro-spec-impl wintf-P0-image-widget`
  - `/kiro-spec-impl wintf-P0-event-system`
  - `/kiro-spec-impl wintf-P0-typewriter`
  - `/kiro-spec-impl wintf-P0-balloon-system`
  - `/kiro-spec-impl wintf-P0-animation-system`

- [ ] 4.1.2 arekaコア層の実装
  - `/kiro-spec-impl areka-P0-script-engine`
  - `/kiro-spec-impl areka-P0-package-manager`
  - `/kiro-spec-impl areka-P0-persistence`
  - `/kiro-spec-impl areka-P0-mcp-server`
  - `/kiro-spec-impl areka-P0-system-tray`

- [ ] 4.1.3 参照実装の完成
  - `/kiro-spec-impl areka-P0-reference-ghost`
  - `/kiro-spec-impl areka-P0-reference-shell`
  - `/kiro-spec-impl areka-P0-reference-balloon`
  - `/kiro-spec-impl areka-P0-window-placement`

### 4.2 P1実装（リリース品質）

- [ ] 4.2.1 安定性・互換性の実装
  - `/kiro-spec-impl wintf-P1-clickthrough`
  - `/kiro-spec-impl wintf-P1-dpi-scaling`
  - `/kiro-spec-impl areka-P1-legacy-converter`
  - `/kiro-spec-impl areka-P1-error-recovery`

- [ ] 4.2.2 開発支援・拡張の実装
  - `/kiro-spec-impl areka-P1-devtools`
  - `/kiro-spec-impl areka-P1-timer-events`
  - `/kiro-spec-impl areka-P1-character-communication`

### 4.3 P2実装（差別化機能）

- [ ] 4.3.1 独自機能の実装
  - `/kiro-spec-impl areka-P2-presence-style`
  - `/kiro-spec-impl areka-P2-memory-system`
  - `/kiro-spec-impl areka-P2-llm-integration`
  - `/kiro-spec-impl areka-P2-creator-tools`
  - `/kiro-spec-impl areka-P2-privacy-security`

- [ ] 4.3.2 開発プロセス支援の実装
  - `/kiro-spec-impl kiro-P2-roadmap-management`

### 4.4 P3実装（将来展望）

- [ ] 4.4.1 長期機能の実装
  - `/kiro-spec-impl areka-P3-voice-system`
  - `/kiro-spec-impl areka-P3-screen-awareness`
  - `/kiro-spec-impl areka-P3-environment-sense`
  - `/kiro-spec-impl areka-P3-cloud-sync`
  - `/kiro-spec-impl areka-P3-ide-integration`

---

## Phase 5: 統合・検証

- [ ] 5.1 P0統合テスト（MVP検証）
  - 起動フロー: アプリ起動 → ゴースト表示 → 初期トーク表示
  - 対話フロー: クリック → イベント → 応答 → 表示
  - 2体キャラクター掛け合い会話
  - _Requirements: 1.1, 4.6, 5.1, 7.1, 26.1_

- [ ] 5.2 P1統合テスト（リリース品質検証）
  - パフォーマンス: アイドルCPU < 1%, メモリ < 100MB, 60fps維持
  - 互換性: 旧シェル変換、DPIスケーリング
  - 安定性: クラッシュ復旧、エラーハンドリング
  - _Requirements: 14.1, 14.2, 14.3, 15.1, 15.2, 30.1-30.10_

- [ ] 5.3 P2機能テスト（差別化機能検証）
  - LLM連携: ローカル/クラウド両対応
  - 記憶システム: 会話履歴、ユーザー情報記憶
  - プライバシー: ローカルファースト動作確認

- [ ] 5.4 P3機能テスト（将来機能検証）
  - 音声合成連携
  - 画面認識連携
  - クラウド同期

---

## Phase 6: リリース管理

- [ ] 6.1 MVPリリース（P0完了時）
  - 最小限の動作するデスクトップマスコット
  - 参照ゴースト/シェル/バルーン同梱

- [ ] 6.2 v1.0リリース（P1完了時）
  - リリース品質の安定版
  - 開発者ツール同梱

- [ ] 6.3 v2.0リリース（P2完了時）
  - 差別化機能搭載版
  - LLM/記憶/プライバシー対応

- [ ] 6.4 将来リリース（P3順次）
  - 長期ロードマップに基づく機能追加

---

## 要件カバレッジ（子仕様分担表）

全31要件グループは、32件の子仕様によって完全にカバーされる（31件機能仕様 + 1件プロセス支援）。

| 要件グループ | 子仕様 | 優先度 |
|-------------|--------|--------|
| Req 1 (ウィンドウ表示) | wintf-P0-image-widget, wintf-P1-clickthrough, areka-P0-window-placement | P0/P1 |
| Req 2 (サーフェス) | wintf-P0-image-widget, wintf-P0-animation-system, areka-P0-reference-shell | P0 |
| Req 3 (バルーン) | wintf-P0-typewriter, wintf-P0-balloon-system, areka-P0-reference-balloon | P0 |
| Req 4 (対話) | areka-P0-script-engine, areka-P0-reference-ghost, areka-P2-llm-integration | P0/P2 |
| Req 5 (入力) | wintf-P0-event-system, wintf-P0-balloon-system | P0 |
| Req 6 (時間イベント) | areka-P1-timer-events | P1 |
| Req 7-8 (パッケージ管理) | areka-P0-package-manager | P0 |
| Req 9 (設定) | areka-P0-persistence | P0 |
| Req 10 (MCP) | areka-P0-mcp-server | P0 |
| Req 11 (互換性) | areka-P1-legacy-converter | P1 |
| Req 12 (開発者支援) | areka-P1-devtools | P1 |
| Req 13 (システムトレイ) | areka-P0-system-tray | P0 |
| Req 14 (パフォーマンス) | 全子仕様のNFR | - |
| Req 15 (DPI) | wintf-P1-dpi-scaling | P1 |
| Req 16 (存在スタイル) | areka-P2-presence-style | P2 |
| Req 17 (記憶) | areka-P2-memory-system | P2 |
| Req 18 (LLM) | areka-P2-llm-integration | P2 |
| Req 19 (音声) | areka-P3-voice-system | P3 |
| Req 20 (画面認識) | areka-P3-screen-awareness | P3 |
| Req 21-22 (環境認識) | areka-P3-environment-sense | P3 |
| Req 23 (同期) | areka-P3-cloud-sync | P3 |
| Req 24 (クリエイター) | areka-P2-creator-tools | P2 |
| Req 25 (プライバシー) | areka-P2-privacy-security | P2 |
| Req 26 (キャラクター間) | areka-P1-character-communication, areka-P2-llm-integration | P1/P2 |
| Req 27 (パッケージ詳細) | areka-P0-package-manager, areka-P0-reference-* | P0 |
| Req 28 (デバッグ) | areka-P1-devtools, areka-P3-ide-integration | P1/P3 |
| Req 29 (さくらスクリプト) | areka-P0-script-engine, areka-P1-legacy-converter | P0/P1 |
| Req 30 (エラー処理) | areka-P1-error-recovery | P1 |
| Req 31 (インストーラ) | areka-P0-package-manager | P0 |
| (プロセス支援) | kiro-P2-roadmap-management | P2 |

> **Note**: 本仕様は「メタ仕様」として全31要件を子仕様に分解し、今後数か月〜年単位での開発を駆動する。各子仕様は独立して要件定義→設計→タスク→実装のサイクルを回す。
