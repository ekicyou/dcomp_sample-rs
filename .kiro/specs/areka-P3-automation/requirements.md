# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | areka オートメーション 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Priority** | P3 (将来機能) |

---

## Introduction

本仕様書は areka アプリケーションにおけるオートメーション（自動化）機能の要件を定義する。ユーザー定義のワークフロー自動実行を目的とする。

### 親仕様からのトレーサビリティ

本仕様は `ukagaka-desktop-mascot` の以下の要件をカバーする：

| 親要件ID | 内容 |
|----------|------|
| 29.1 | 条件に基づいてアクションを自動実行できる |
| 29.2 | トリガー（イベント）とアクションを組み合わせたルールを定義できる |
| 29.3 | スケジュール実行をサポートする |
| 29.4 | 外部アプリケーション連携を自動化できる |

### スコープ

**含まれるもの:**
- ルール定義
- トリガー/アクション
- スケジュール実行

**含まれないもの:**
- 基本タイマー機能（areka-P1-timer-events の責務）
- MCPツール実行（areka-P0-mcp-server の責務）

---

## Requirements

### Requirement 1: ルール定義

**Objective:** ユーザーとして、自動化ルールを定義したい。それにより繰り返し作業を省略できる。

#### Acceptance Criteria

1. **The** Automation **shall** 「トリガー→条件→アクション」形式のルールを定義できる
2. **The** Automation **shall** 複数のトリガーを OR で組み合わせられる
3. **The** Automation **shall** 複数の条件を AND/OR で組み合わせられる
4. **The** Automation **shall** 複数のアクションを順次実行できる

---

### Requirement 2: トリガー

**Objective:** ゴースト制作者として、様々なトリガーを使いたい。それにより柔軟な自動化ができる。

#### Acceptance Criteria

1. **The** Automation **shall** 時刻トリガー（特定時刻、繰り返し）をサポートする
2. **The** Automation **shall** イベントトリガー（システムイベント等）をサポートする
3. **The** Automation **shall** ファイル変更トリガーをサポートする
4. **The** Automation **shall** カスタムトリガーの追加をサポートする

---

### Requirement 3: アクション

**Objective:** ゴースト制作者として、様々なアクションを実行したい。それにより多様な自動化ができる。

#### Acceptance Criteria

1. **The** Automation **shall** MCPツール呼び出しをアクションとして実行できる
2. **The** Automation **shall** 外部プログラム起動をアクションとして実行できる
3. **The** Automation **shall** ゴーストへのイベント通知をアクションとして実行できる
4. **The** Automation **shall** 待機（遅延）をアクションとして実行できる

---

### Requirement 4: ルール管理

**Objective:** ユーザーとして、ルールを管理したい。それにより自動化を整理できる。

#### Acceptance Criteria

1. **The** Automation **shall** ルールの有効/無効を切り替えられる
2. **The** Automation **shall** ルールをエクスポート/インポートできる
3. **The** Automation **shall** ルールの実行ログを表示できる
4. **The** Automation **shall** ルールのテスト実行ができる

---

## Non-Functional Requirements

### NFR-1: 信頼性

1. ルール実行失敗時にエラーハンドリングすること
2. 無限ループを検出・防止すること

### NFR-2: セキュリティ

1. 危険なアクション実行前に確認を求められること

---

## Dependencies

### 依存する仕様

| 仕様 | 依存内容 |
|------|----------|
| `areka-P0-mcp-server` | ツール実行 |
| `areka-P1-timer-events` | スケジュールトリガー |

### 依存される仕様

なし（将来機能）

---

## Glossary

| 用語 | 定義 |
|------|------|
| **オートメーション** | 条件に基づく自動実行 |
| **トリガー** | ルール実行のきっかけとなるイベント |
| **アクション** | ルールで実行される処理 |
