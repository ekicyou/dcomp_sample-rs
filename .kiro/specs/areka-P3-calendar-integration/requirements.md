# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | areka カレンダー統合 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Priority** | P3 (将来機能) |

---

## Introduction

本仕様書は areka アプリケーションにおけるカレンダー統合機能の要件を定義する。外部カレンダーサービスとの連携により、スケジュール通知などを目的とする。

### 親仕様からのトレーサビリティ

本仕様は `ukagaka-desktop-mascot` の以下の要件をカバーする：

| 親要件ID | 内容 |
|----------|------|
| 24.1 | Googleカレンダーと連携できる |
| 24.2 | Outlookカレンダーと連携できる |
| 24.3 | カレンダーの予定を取得し、イベントとして発火できる |
| 24.4 | 予定のリマインダーをゴーストに通知できる |

### スコープ

**含まれるもの:**
- カレンダーAPI連携
- 予定取得
- リマインダー通知

**含まれないもの:**
- カレンダー編集機能
- タイマー基盤（areka-P1-timer-events の責務）

---

## Requirements

### Requirement 1: カレンダー接続

**Objective:** ユーザーとして、カレンダーサービスに接続したい。それにより予定を確認できる。

#### Acceptance Criteria

1. **The** Calendar Integration **shall** Googleカレンダーに OAuth で接続できる
2. **The** Calendar Integration **shall** Microsoft Outlook/365 に接続できる
3. **The** Calendar Integration **shall** CalDAV プロトコルに対応する（オプション）
4. **The** Calendar Integration **shall** 接続情報を安全に保存する

---

### Requirement 2: 予定取得

**Objective:** ゴースト制作者として、ユーザーの予定を取得したい。それにより予定に関する会話ができる。

#### Acceptance Criteria

1. **The** Calendar Integration **shall** 今日/明日/週間の予定を取得できる
2. **The** Calendar Integration **shall** 予定のタイトル、時間、場所を取得できる
3. **The** Calendar Integration **shall** 繰り返し予定を正しく処理できる
4. **The** Calendar Integration **shall** 複数カレンダーを統合できる

---

### Requirement 3: リマインダー

**Objective:** ユーザーとして、予定を事前に知らせてほしい。それにより忘れずに済む。

#### Acceptance Criteria

1. **When** 予定時刻が近づいた時, **the** Calendar Integration **shall** リマインダーイベントを発火する
2. **The** Calendar Integration **shall** リマインダー時間（〇分前）を設定できる
3. **The** Calendar Integration **shall** リマインダーの有効/無効をカレンダーごとに設定できる

---

### Requirement 4: プライバシー

**Objective:** ユーザーとして、予定のプライバシーを守りたい。それにより安心して使える。

#### Acceptance Criteria

1. **The** Calendar Integration **shall** 取得する情報の範囲を設定できる
2. **The** Calendar Integration **shall** センシティブな予定を除外できる
3. **The** Calendar Integration **shall** 認証情報を暗号化して保存する

---

## Non-Functional Requirements

### NFR-1: セキュリティ

1. OAuth 2.0 を使用すること
2. 最小限の権限のみを要求すること

### NFR-2: 信頼性

1. API障害時も他の機能に影響しないこと

---

## Dependencies

### 依存する仕様

| 仕様 | 依存内容 |
|------|----------|
| `areka-P0-mcp-server` | イベント通知 |
| `areka-P1-timer-events` | リマインダータイミング |

### 依存される仕様

なし（将来機能）

---

## Glossary

| 用語 | 定義 |
|------|------|
| **OAuth** | 認可プロトコル |
| **CalDAV** | カレンダー同期プロトコル |
| **リマインダー** | 予定前の通知 |
