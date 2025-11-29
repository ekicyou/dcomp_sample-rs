# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | areka LLM統合 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Priority** | P2 (拡張機能) |

---

## Introduction

本仕様書は areka アプリケーションにおけるLLM（大規模言語モデル）統合機能の要件を定義する。ゴーストのAI応答バックエンドとしてLLMを活用することを目的とする。

### 親仕様からのトレーサビリティ

本仕様は `ukagaka-desktop-mascot` の以下の要件をカバーする：

| 親要件ID | 内容 |
|----------|------|
| 19.1 | 外部LLMサービス（OpenAI API等）と連携できる |
| 19.2 | ローカルLLM（Ollama等）と連携できる |
| 19.3 | LLM応答をゴーストの発言としてフォーマットできる |
| 19.4 | キャラクター設定（persona）をLLMプロンプトに注入できる |
| 19.5 | 会話履歴をコンテキストとして管理できる |
| 19.6 | LLM応答のストリーミング表示をサポートする |

### スコープ

**含まれるもの:**
- LLMバックエンド接続（API/ローカル）
- プロンプト管理
- 会話コンテキスト管理
- ストリーミング表示

**含まれないもの:**
- スクリプト実行（areka-P0-script-engine の責務）
- MCP通信基盤（areka-P0-mcp-server の責務）

---

## Requirements

### Requirement 1: LLMバックエンド接続

**Objective:** ゴースト制作者として、LLMサービスに接続したい。それによりAI応答を活用できる。

#### Acceptance Criteria

1. **The** LLM Integration **shall** OpenAI API互換エンドポイントに接続できる
2. **The** LLM Integration **shall** ローカルLLM（Ollama）に接続できる
3. **The** LLM Integration **shall** APIキー/認証情報を安全に管理できる
4. **The** LLM Integration **shall** 接続先を設定で切り替えられる
5. **The** LLM Integration **shall** タイムアウト・リトライ処理を実装する

---

### Requirement 2: プロンプト管理

**Objective:** ゴースト制作者として、キャラクター性をLLMに伝えたい。それにより一貫した応答を得られる。

#### Acceptance Criteria

1. **The** LLM Integration **shall** システムプロンプト（ペルソナ設定）を設定できる
2. **The** LLM Integration **shall** プロンプトテンプレートを定義できる
3. **The** LLM Integration **shall** ユーザー入力をプロンプトに組み込める
4. **The** LLM Integration **shall** プロンプトをゴーストパッケージに含められる

---

### Requirement 3: 会話コンテキスト

**Objective:** ゴースト制作者として、会話の流れを維持したい。それにより自然な対話ができる。

#### Acceptance Criteria

1. **The** LLM Integration **shall** 会話履歴をコンテキストとして送信できる
2. **The** LLM Integration **shall** コンテキスト長の上限を設定できる
3. **The** LLM Integration **shall** 古い履歴を要約/削除して長さを制御できる
4. **The** LLM Integration **shall** セッション間で会話履歴を永続化できる（オプション）

---

### Requirement 4: ストリーミング応答

**Objective:** ユーザーとして、応答をリアルタイムで見たい。それにより待ち時間を短く感じられる。

#### Acceptance Criteria

1. **The** LLM Integration **shall** ストリーミング応答をサポートする
2. **The** LLM Integration **shall** 受信中のテキストを逐次表示できる
3. **The** LLM Integration **shall** ストリーミング中にキャンセルできる
4. **The** LLM Integration **shall** ストリーミング完了後に後処理を実行できる

---

### Requirement 5: 応答フォーマット

**Objective:** ゴースト制作者として、LLM応答をゴースト形式に変換したい。それによりサーフェス変更等を含められる。

#### Acceptance Criteria

1. **The** LLM Integration **shall** LLM応答からテキストを抽出できる
2. **The** LLM Integration **shall** LLM応答から感情タグ等を抽出できる
3. **The** LLM Integration **shall** 感情タグをサーフェス切り替えにマッピングできる
4. **The** LLM Integration **shall** カスタム後処理関数を登録できる

---

## Non-Functional Requirements

### NFR-1: セキュリティ

1. APIキーを平文で保存しないこと
2. API呼び出し回数を制限できること

### NFR-2: パフォーマンス

1. API呼び出しはバックグラウンドで実行すること
2. 応答待ち中もUIがフリーズしないこと

---

## Dependencies

### 依存する仕様

| 仕様 | 依存内容 |
|------|----------|
| `areka-P0-mcp-server` | ツール呼び出し |
| `areka-P0-script-engine` | 応答処理スクリプト |

### 依存される仕様

なし（拡張機能）

---

## Glossary

| 用語 | 定義 |
|------|------|
| **LLM** | Large Language Model。大規模言語モデル |
| **ペルソナ** | キャラクターの性格・設定を定義したシステムプロンプト |
| **コンテキスト** | LLMに送信する会話履歴・背景情報 |
| **ストリーミング** | 応答を逐次受信・表示する方式 |
