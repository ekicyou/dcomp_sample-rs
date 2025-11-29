# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | areka 音声合成 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Priority** | P2 (拡張機能) |

---

## Introduction

本仕様書は areka アプリケーションにおける音声合成（TTS: Text-to-Speech）機能の要件を定義する。キャラクターの発言を音声で読み上げることを目的とする。

### 親仕様からのトレーサビリティ

本仕様は `ukagaka-desktop-mascot` の以下の要件をカバーする：

| 親要件ID | 内容 |
|----------|------|
| 20.1 | テキストを音声に変換して再生できる（TTS） |
| 20.2 | 外部TTSエンジン（VOICEVOX、COEIROINK等）と連携できる |
| 20.3 | 音声再生とテキスト表示を同期できる |
| 20.4 | キャラクターごとに異なる音声設定を指定できる |
| 20.5 | 音量・速度・ピッチを調整できる |

### スコープ

**含まれるもの:**
- TTSエンジン連携
- 音声再生制御
- テキスト同期
- 音声パラメータ設定

**含まれないもの:**
- TTSエンジン自体の実装
- バルーンテキスト表示（wintf-P0-balloon-system の責務）

---

## Requirements

### Requirement 1: TTSエンジン連携

**Objective:** ゴースト制作者として、音声合成エンジンを使いたい。それによりキャラクターに声を与えられる。

#### Acceptance Criteria

1. **The** Voice System **shall** VOICEVOX API に接続できる
2. **The** Voice System **shall** COEIROINK API に接続できる
3. **The** Voice System **shall** Windows SAPI に接続できる
4. **The** Voice System **shall** TTSエンジンを設定で切り替えられる
5. **The** Voice System **shall** カスタムTTSエンジンの追加をサポートする

---

### Requirement 2: 音声再生

**Objective:** ユーザーとして、キャラクターの声を聞きたい。それにより臨場感のある会話ができる。

#### Acceptance Criteria

1. **The** Voice System **shall** 生成された音声を再生できる
2. **The** Voice System **shall** 音声の一時停止・再開・停止ができる
3. **The** Voice System **shall** 音声のキューイング（順次再生）をサポートする
4. **The** Voice System **shall** 音声再生完了のコールバックを提供する

---

### Requirement 3: テキスト同期

**Objective:** ユーザーとして、音声とテキストが同期してほしい。それにより読みやすい。

#### Acceptance Criteria

1. **The** Voice System **shall** 音声再生に合わせてテキストを表示できる
2. **The** Voice System **shall** 音声がない場合は通常速度でテキストを表示する
3. **The** Voice System **shall** 音声生成中のローディング表示をサポートする

---

### Requirement 4: 音声パラメータ

**Objective:** ゴースト制作者として、声質を調整したい。それによりキャラクターに合った声にできる。

#### Acceptance Criteria

1. **The** Voice System **shall** 話者（声）を選択できる
2. **The** Voice System **shall** 音量を調整できる
3. **The** Voice System **shall** 話速を調整できる
4. **The** Voice System **shall** ピッチを調整できる
5. **The** Voice System **shall** キャラクターごとに音声設定を保存できる

---

### Requirement 5: 音声有効/無効

**Objective:** ユーザーとして、音声のオン/オフを切り替えたい。それにより状況に応じて使い分けられる。

#### Acceptance Criteria

1. **The** Voice System **shall** 音声機能の有効/無効を切り替えられる
2. **The** Voice System **shall** ミュート状態でもテキスト表示は行う
3. **The** Voice System **shall** システムトレイから音声オン/オフを操作できる

---

## Non-Functional Requirements

### NFR-1: パフォーマンス

1. 音声生成はバックグラウンドで行うこと
2. 音声生成中もUIがフリーズしないこと

### NFR-2: 品質

1. 音声再生は途切れなく行うこと
2. 低遅延で再生を開始できること

---

## Dependencies

### 依存する仕様

| 仕様 | 依存内容 |
|------|----------|
| `areka-P0-mcp-server` | 音声再生ツール提供 |

### 依存される仕様

なし（拡張機能）

---

## Glossary

| 用語 | 定義 |
|------|------|
| **TTS** | Text-to-Speech。テキスト音声合成 |
| **VOICEVOX** | オープンソースの音声合成エンジン |
| **COEIROINK** | 音声合成エンジン |
| **SAPI** | Speech API。Windowsの音声機能 |
