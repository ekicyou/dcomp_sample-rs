# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | areka MCPサーバー 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Priority** | P0 (MVP必須) |

---

## Introduction

本仕様書は areka アプリケーションにおけるMCPサーバー基盤の要件を定義する。ゴースト（MCPクライアント）とプラットフォーム間の通信、およびゴースト間通信の媒介を行うことを目的とする。

### 親仕様からのトレーサビリティ

本仕様は `ukagaka-desktop-mascot` の以下の要件をカバーする：

| 親要件ID | 内容 |
|----------|------|
| 10.1 | 複数のマスコットアプリが起動している場合、ゴースト間でメッセージを送受信できる |
| 10.2 | HTTP/HTTPSリクエストを発行し、外部APIと通信できる |
| 10.3 | Webページからリンクがクリックされた時、独自プロトコルでイベントを受信できる |
| 10.4 | プラグインからのイベント・コマンドを処理できる |
| 10.5 | ローカルファイルシステムの監視を行い、変更をイベントとして発火できる |
| 26.11-26.15 | ゴースト間通信の媒介 |

### スコープ

**含まれるもの:**
- MCPサーバー基盤（JSON-RPC 2.0）
- MCPツール定義（display_text, switch_surface等）
- イベント通知（OnMouseClick, OnTimer等）
- ゴースト間通信の媒介

**含まれないもの:**
- LLM統合（areka-P2-llm-integration の責務）
- スクリプト実行（areka-P0-script-engine の責務）
- タイマーイベント詳細（areka-P1-timer-events の責務）

---

## Requirements

### Requirement 1: MCPサーバー基盤

**Objective:** 開発者として、ゴースト（MCPクライアント）と通信したい。それによりゴーストがプラットフォーム機能を利用できる。

#### Acceptance Criteria

1. **The** MCP Server **shall** JSON-RPC 2.0 プロトコルを実装する
2. **The** MCP Server **shall** stdio トランスポートをサポートする
3. **The** MCP Server **shall** 複数のゴースト（クライアント）と同時接続できる
4. **The** MCP Server **shall** リクエスト・レスポンスの非同期処理をサポートする
5. **The** MCP Server **shall** エラーハンドリング（JSON-RPCエラーコード）を実装する

---

### Requirement 2: MCPツール定義

**Objective:** ゴースト制作者として、プラットフォームの機能を呼び出したい。それによりキャラクターの動作を実現できる。

#### Acceptance Criteria

1. **The** MCP Server **shall** テキスト表示ツール（display_text）を提供する
2. **The** MCP Server **shall** サーフェス切り替えツール（switch_surface）を提供する
3. **The** MCP Server **shall** バルーン表示/非表示ツール（show_balloon, hide_balloon）を提供する
4. **The** MCP Server **shall** 選択肢表示ツール（show_choices）を提供する
5. **The** MCP Server **shall** ウィンドウ位置取得/設定ツールを提供する
6. **The** MCP Server **shall** カスタムツールの登録をサポートする

---

### Requirement 3: イベント通知

**Objective:** ゴースト制作者として、ユーザー操作やシステムイベントを受け取りたい。それによりインタラクティブな応答ができる。

#### Acceptance Criteria

1. **The** MCP Server **shall** マウスイベント（OnMouseClick, OnMouseDoubleClick, OnMouseHover）を通知する
2. **The** MCP Server **shall** イベント引数（座標、キャラクターID、領域ID等）を含める
3. **The** MCP Server **shall** システムイベント（OnStartup, OnShutdown）を通知する
4. **The** MCP Server **shall** 選択肢選択イベント（OnChoiceSelect）を通知する
5. **The** MCP Server **shall** カスタムイベントの登録・通知をサポートする

---

### Requirement 4: ゴースト間通信

**Objective:** ゴースト制作者として、他のゴーストとメッセージを交換したい。それによりキャラクター間の掛け合いを実現できる。

#### Acceptance Criteria

1. **The** MCP Server **shall** ゴースト間でメッセージを送受信できる
2. **The** MCP Server **shall** メッセージの宛先（特定ゴーストまたはブロードキャスト）を指定できる
3. **The** MCP Server **shall** メッセージの種類（会話要求、応答、通知等）を区別できる
4. **When** メッセージを受信した時, **the** MCP Server **shall** 受信イベントを宛先ゴーストに通知する
5. **The** MCP Server **shall** 接続中のゴースト一覧を取得できる

---

### Requirement 5: 外部連携

**Objective:** 開発者として、外部サービスやツールと連携したい。それにより拡張機能を実現できる。

#### Acceptance Criteria

1. **The** MCP Server **shall** HTTP/HTTPSリクエストツールを提供する
2. **The** MCP Server **shall** ファイルシステム監視ツールを提供する
3. **The** MCP Server **shall** 外部MCPサーバーへのプロキシをサポートする（オプション）
4. **When** 外部イベント（URLプロトコル呼び出し等）を受信した時, **the** MCP Server **shall** ゴーストに通知する

---

## Non-Functional Requirements

### NFR-1: パフォーマンス

1. メッセージ処理のレイテンシは10ms以内であること
2. 同時接続10クライアント以上をサポートすること
3. メモリリークなく長時間稼働できること

### NFR-2: 信頼性

1. クライアント切断時に適切にクリーンアップすること
2. 不正なメッセージを受信しても安定動作を維持すること

### NFR-3: 互換性

1. MCP仕様の基本サブセットに準拠すること
2. rmcp ライブラリとの互換性を考慮すること

---

## Dependencies

### 依存する仕様

なし（独立）

### 依存される仕様

| 仕様 | 依存内容 |
|------|----------|
| `areka-P0-reference-ghost` | MCPクライアントとの通信 |
| `areka-P1-timer-events` | タイマーイベント通知 |
| `areka-P1-character-communication` | ゴースト間通信 |
| `areka-P2-llm-integration` | LLMバックエンド連携 |

---

## Glossary

| 用語 | 定義 |
|------|------|
| **MCP** | Model Context Protocol。LLMとの連携を想定した通信プロトコル |
| **JSON-RPC** | JSONベースのリモートプロシージャコールプロトコル |
| **ツール** | MCPで定義される呼び出し可能な機能 |
| **イベント** | サーバーからクライアントへの通知 |
| **rmcp** | Rust向けMCP実装ライブラリ |
