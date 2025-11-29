# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | areka メディアプレイヤー連携 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Priority** | P3 (将来機能) |

---

## Introduction

本仕様書は areka アプリケーションにおけるメディアプレイヤー連携機能の要件を定義する。音楽プレイヤーとの連携により、再生中の楽曲情報取得などを目的とする。

### 親仕様からのトレーサビリティ

本仕様は `ukagaka-desktop-mascot` の以下の要件をカバーする：

| 親要件ID | 内容 |
|----------|------|
| 25.1 | 音楽プレイヤー（Spotify, foobar2000等）の再生情報を取得できる |
| 25.2 | 再生中の曲名・アーティスト名をイベントとして受け取れる |
| 25.3 | 再生/一時停止等の状態変化をイベントとして受け取れる |
| 25.4 | プレイヤーの基本操作（再生、停止、次曲等）を送信できる |

### スコープ

**含まれるもの:**
- メディアプレイヤー情報取得
- 再生状態監視
- 基本操作制御

**含まれないもの:**
- 音声再生自体（areka-P2-voice-synthesis の責務）
- プレイリスト管理

---

## Requirements

### Requirement 1: プレイヤー検出

**Objective:** ゴースト制作者として、使用中のプレイヤーを検出したい。それにより適切な連携ができる。

#### Acceptance Criteria

1. **The** Media Player **shall** Windows Media Session APIに対応したプレイヤーを検出できる
2. **The** Media Player **shall** Spotify（Windows版）を検出できる
3. **The** Media Player **shall** 複数プレイヤーからアクティブなものを選択できる

---

### Requirement 2: 楽曲情報取得

**Objective:** ゴースト制作者として、再生中の曲を知りたい。それにより楽曲について会話できる。

#### Acceptance Criteria

1. **The** Media Player **shall** 曲名を取得できる
2. **The** Media Player **shall** アーティスト名を取得できる
3. **The** Media Player **shall** アルバム名を取得できる
4. **The** Media Player **shall** アルバムアートを取得できる（オプション）
5. **The** Media Player **shall** 再生位置/長さを取得できる

---

### Requirement 3: 状態監視

**Objective:** ゴースト制作者として、再生状態の変化を知りたい。それにより適切なタイミングで反応できる。

#### Acceptance Criteria

1. **When** 楽曲が変わった時, **the** Media Player **shall** 曲変更イベントを発火する
2. **When** 再生/一時停止された時, **the** Media Player **shall** 状態変更イベントを発火する
3. **The** Media Player **shall** 現在の再生状態（再生中/停止/一時停止）を取得できる

---

### Requirement 4: 再生操作

**Objective:** ゴースト制作者として、プレイヤーを操作したい。それによりキャラクターから音楽を制御できる。

#### Acceptance Criteria

1. **The** Media Player **shall** 再生/一時停止コマンドを送信できる
2. **The** Media Player **shall** 次曲/前曲コマンドを送信できる
3. **The** Media Player **shall** 操作対象プレイヤーを指定できる

---

## Non-Functional Requirements

### NFR-1: 互換性

1. Windows 10/11 のMedia Session APIに対応すること

### NFR-2: パフォーマンス

1. 情報取得のポーリング間隔を適切に設定すること

---

## Dependencies

### 依存する仕様

| 仕様 | 依存内容 |
|------|----------|
| `areka-P0-mcp-server` | イベント通知 |

### 依存される仕様

なし（将来機能）

---

## Glossary

| 用語 | 定義 |
|------|------|
| **Media Session API** | Windowsのメディア制御API |
| **アルバムアート** | 楽曲のジャケット画像 |
