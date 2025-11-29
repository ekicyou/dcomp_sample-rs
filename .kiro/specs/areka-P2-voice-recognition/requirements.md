# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | areka 音声認識 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Priority** | P2 (拡張機能) |

---

## Introduction

本仕様書は areka アプリケーションにおける音声認識（STT: Speech-to-Text）機能の要件を定義する。ユーザーの音声入力によりキャラクターと対話することを目的とする。

### 親仕様からのトレーサビリティ

本仕様は `ukagaka-desktop-mascot` の以下の要件をカバーする：

| 親要件ID | 内容 |
|----------|------|
| 21.1 | マイク入力をテキストに変換できる（STT） |
| 21.2 | Windows音声認識APIと連携できる |
| 21.3 | 外部STTサービス（Whisper等）と連携できる |
| 21.4 | ウェイクワード（起動ワード）検出をサポートする |

### スコープ

**含まれるもの:**
- 音声認識エンジン連携
- マイク入力制御
- ウェイクワード検出
- テキスト出力

**含まれないもの:**
- 音声認識エンジン自体の実装
- LLM処理（areka-P2-llm-integration の責務）

---

## Requirements

### Requirement 1: 音声認識エンジン連携

**Objective:** ユーザーとして、声でキャラクターと話したい。それによりハンズフリーで会話できる。

#### Acceptance Criteria

1. **The** Voice Recognition **shall** Windows音声認識APIに接続できる
2. **The** Voice Recognition **shall** Whisper（OpenAI）に接続できる
3. **The** Voice Recognition **shall** 音声認識エンジンを設定で切り替えられる
4. **The** Voice Recognition **shall** 認識言語を設定できる

---

### Requirement 2: マイク入力

**Objective:** ユーザーとして、マイクで話しかけたい。それにより自然な対話ができる。

#### Acceptance Criteria

1. **The** Voice Recognition **shall** マイクからの音声入力を取得できる
2. **The** Voice Recognition **shall** 使用するマイクデバイスを選択できる
3. **The** Voice Recognition **shall** 入力レベル（音量）を表示できる
4. **The** Voice Recognition **shall** 音声入力の有効/無効を切り替えられる

---

### Requirement 3: 認識制御

**Objective:** ユーザーとして、意図したときだけ認識してほしい。それにより誤認識を減らせる。

#### Acceptance Criteria

1. **The** Voice Recognition **shall** プッシュトゥトーク（ボタン押下中のみ認識）をサポートする
2. **The** Voice Recognition **shall** 音声区間検出（VAD）で自動的に認識開始・終了する
3. **The** Voice Recognition **shall** 認識中の視覚的フィードバックを提供する
4. **The** Voice Recognition **shall** 認識タイムアウトを設定できる

---

### Requirement 4: ウェイクワード

**Objective:** ユーザーとして、名前を呼んで起動したい。それにより自然に話しかけられる。

#### Acceptance Criteria

1. **The** Voice Recognition **shall** ウェイクワードを設定できる
2. **When** ウェイクワードが検出された時, **the** Voice Recognition **shall** 音声認識を開始する
3. **The** Voice Recognition **shall** 複数のウェイクワードを登録できる
4. **The** Voice Recognition **shall** ウェイクワード検出の感度を調整できる

---

### Requirement 5: テキスト出力

**Objective:** ゴースト制作者として、認識結果を受け取りたい。それにより応答を生成できる。

#### Acceptance Criteria

1. **The** Voice Recognition **shall** 認識結果をテキストとしてイベント発火する
2. **The** Voice Recognition **shall** 中間結果（認識中テキスト）を提供する（オプション）
3. **The** Voice Recognition **shall** 認識信頼度を提供する
4. **The** Voice Recognition **shall** 認識エラーを通知する

---

## Non-Functional Requirements

### NFR-1: パフォーマンス

1. 音声認識の遅延は500ms以内であること
2. バックグラウンドで処理しUIをブロックしないこと

### NFR-2: プライバシー

1. ローカル処理オプションを提供すること
2. 音声データの取り扱いを明示すること

---

## Dependencies

### 依存する仕様

| 仕様 | 依存内容 |
|------|----------|
| `areka-P0-mcp-server` | イベント通知 |

### 依存される仕様

なし（拡張機能）

---

## Glossary

| 用語 | 定義 |
|------|------|
| **STT** | Speech-to-Text。音声認識 |
| **VAD** | Voice Activity Detection。音声区間検出 |
| **ウェイクワード** | 音声認識を起動するキーワード（「OK Google」等） |
| **Whisper** | OpenAIの音声認識モデル |
