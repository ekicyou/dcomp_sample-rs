# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | areka タイマーイベント 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Priority** | P1 (体験向上) |

---

## Introduction

本仕様書は areka アプリケーションにおけるタイマーイベント機能の要件を定義する。時間経過に基づくキャラクターの自発的な行動を実現することを目的とする。

### 親仕様からのトレーサビリティ

本仕様は `ukagaka-desktop-mascot` の以下の要件をカバーする：

| 親要件ID | 内容 |
|----------|------|
| 11.1 | 一定時間ごとにイベントが発火し、ゴーストに通知される |
| 11.2 | PCシステムの日時情報を取得し、条件として使用できる |
| 11.3 | 経過時間に基づくイベント（ユーザー無操作、アイドル等）を発火できる |
| 11.4 | 特定時刻にアラームとしてイベントを発火できる |
| 11.5 | タイムゾーンを考慮した日時処理を行う |

### スコープ

**含まれるもの:**
- 定期タイマーイベント
- システム日時取得
- アイドル検出
- アラーム/スケジュールイベント

**含まれないもの:**
- アニメーションタイマー（wintf-P0-animation-system の責務）
- MCPサーバー基盤（areka-P0-mcp-server の責務）

---

## Requirements

### Requirement 1: 定期タイマーイベント

**Objective:** ゴースト制作者として、定期的にイベントを受け取りたい。それによりランダムトークなどの自発行動を実現できる。

#### Acceptance Criteria

1. **The** Timer System **shall** 設定可能な間隔で定期イベント（OnSecondChange, OnMinuteChange）を発火する
2. **The** Timer System **shall** 定期イベントの間隔を設定できる（秒単位）
3. **The** Timer System **shall** 定期イベントにタイムスタンプを含める
4. **The** Timer System **shall** 複数の独立したタイマーを登録できる

---

### Requirement 2: システム日時取得

**Objective:** ゴースト制作者として、現在の日時情報を取得したい。それにより時刻に応じた挨拶などを実現できる。

#### Acceptance Criteria

1. **The** Timer System **shall** 現在の日時（年、月、日、時、分、秒）を取得できる
2. **The** Timer System **shall** 曜日情報を取得できる
3. **The** Timer System **shall** タイムゾーン情報を考慮した日時を提供する
4. **The** Timer System **shall** 祝日判定のためのAPIを提供する（オプション）

---

### Requirement 3: アイドル検出

**Objective:** ゴースト制作者として、ユーザーの操作状態を把握したい。それにより適切なタイミングで話しかけられる。

#### Acceptance Criteria

1. **The** Timer System **shall** ユーザー無操作時間を検出できる
2. **When** ユーザー無操作時間が閾値を超えた時, **the** Timer System **shall** アイドルイベントを発火する
3. **When** アイドル状態からユーザー操作が再開された時, **the** Timer System **shall** 復帰イベントを発火する
4. **The** Timer System **shall** アイドル閾値を設定できる

---

### Requirement 4: アラーム・スケジュール

**Objective:** ゴースト制作者として、特定時刻にイベントを発火したい。それにより時報やリマインダーを実現できる。

#### Acceptance Criteria

1. **The** Timer System **shall** 特定時刻にアラームイベントを発火できる
2. **The** Timer System **shall** 繰り返しアラーム（毎日、毎週等）を設定できる
3. **The** Timer System **shall** アラームにラベルやカスタムデータを付与できる
4. **The** Timer System **shall** アラームの一覧取得・削除ができる
5. **The** Timer System **shall** 時報イベント（毎正時）を提供する

---

### Requirement 5: タイムゾーン対応

**Objective:** 開発者として、タイムゾーンを正しく扱いたい。それにより国際的なユーザーに対応できる。

#### Acceptance Criteria

1. **The** Timer System **shall** システムのタイムゾーン設定を取得できる
2. **The** Timer System **shall** UTCと現地時間の変換をサポートする
3. **When** システムのタイムゾーンが変更された時, **the** Timer System **shall** タイムゾーン変更イベントを発火する

---

## Non-Functional Requirements

### NFR-1: 精度

1. 秒単位のタイマーは±1秒以内の精度であること
2. アラームは設定時刻から1秒以内に発火すること

### NFR-2: 効率

1. タイマー処理はCPU負荷を最小限に抑えること
2. スリープ/復帰に正しく対応すること

---

## Dependencies

### 依存する仕様

| 仕様 | 依存内容 |
|------|----------|
| `areka-P0-mcp-server` | イベント通知チャネル |

### 依存される仕様

| 仕様 | 依存内容 |
|------|----------|
| `areka-P0-reference-ghost` | タイマーイベント処理 |

---

## Glossary

| 用語 | 定義 |
|------|------|
| **タイマーイベント** | 時間経過に基づいて自動的に発火するイベント |
| **アイドル** | ユーザーが一定時間操作を行っていない状態 |
| **アラーム** | 特定時刻に発火するスケジュールイベント |
