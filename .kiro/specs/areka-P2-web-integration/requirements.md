# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | areka Web統合 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Priority** | P2 (拡張機能) |

---

## Introduction

本仕様書は areka アプリケーションにおけるWeb統合機能の要件を定義する。Webブラウザとの連携やRSSフィード取得などのWeb関連機能を目的とする。

### 親仕様からのトレーサビリティ

本仕様は `ukagaka-desktop-mascot` の以下の要件をカバーする：

| 親要件ID | 内容 |
|----------|------|
| 22.1 | RSSフィードを購読し、更新をイベントとして発火できる |
| 22.2 | Webページのスクレイピング結果を取得できる |
| 22.3 | ブラウザで特定URLを開くことができる |
| 22.4 | クリップボードの監視・操作ができる |
| 22.5 | 独自URLプロトコル（areka://）からのイベントを受信できる |

### スコープ

**含まれるもの:**
- RSSフィード購読
- HTTPリクエスト/スクレイピング
- ブラウザ連携
- クリップボード操作
- カスタムURLプロトコル

**含まれないもの:**
- HTTP基盤（areka-P0-mcp-server で提供）

---

## Requirements

### Requirement 1: RSSフィード

**Objective:** ゴースト制作者として、RSSフィードを監視したい。それにより最新ニュースをキャラクターに話させられる。

#### Acceptance Criteria

1. **The** Web Integration **shall** RSSフィードURLを登録できる
2. **The** Web Integration **shall** 定期的にフィードを取得・更新チェックする
3. **When** 新しい記事が見つかった時, **the** Web Integration **shall** イベントを発火する
4. **The** Web Integration **shall** フィードの購読解除ができる
5. **The** Web Integration **shall** Atom形式もサポートする

---

### Requirement 2: HTTPリクエスト

**Objective:** ゴースト制作者として、Web APIを呼び出したい。それにより外部サービスと連携できる。

#### Acceptance Criteria

1. **The** Web Integration **shall** GET/POST/PUT/DELETEリクエストを発行できる
2. **The** Web Integration **shall** リクエストヘッダーを設定できる
3. **The** Web Integration **shall** JSONレスポンスをパースできる
4. **The** Web Integration **shall** タイムアウト・エラーハンドリングを実装する

---

### Requirement 3: ブラウザ連携

**Objective:** ゴースト制作者として、ブラウザを操作したい。それによりユーザーにWebページを見せられる。

#### Acceptance Criteria

1. **The** Web Integration **shall** デフォルトブラウザでURLを開ける
2. **The** Web Integration **shall** 指定ブラウザでURLを開ける
3. **The** Web Integration **shall** URL開封前に確認ダイアログを表示できる（オプション）

---

### Requirement 4: クリップボード

**Objective:** ゴースト制作者として、クリップボードを操作したい。それによりテキストのコピー・ペーストができる。

#### Acceptance Criteria

1. **The** Web Integration **shall** クリップボードのテキストを取得できる
2. **The** Web Integration **shall** クリップボードにテキストを設定できる
3. **The** Web Integration **shall** クリップボード変更を監視できる（オプション）

---

### Requirement 5: カスタムURLプロトコル

**Objective:** ゴースト制作者として、Webページからゴーストを呼び出したい。それによりWeb連携機能を実現できる。

#### Acceptance Criteria

1. **The** Web Integration **shall** areka:// URLプロトコルを登録できる
2. **When** areka:// URLがクリックされた時, **the** Web Integration **shall** アプリケーションを起動/アクティブ化する
3. **The** Web Integration **shall** URLパラメータをイベントとしてゴーストに渡す
4. **The** Web Integration **shall** プロトコル登録の有効/無効を設定できる

---

## Non-Functional Requirements

### NFR-1: セキュリティ

1. HTTPSを推奨し、警告を表示すること
2. 信頼できないURLへのアクセスを制限できること

### NFR-2: パフォーマンス

1. HTTPリクエストはバックグラウンドで実行すること
2. RSSフェッチ間隔は設定可能であること

---

## Dependencies

### 依存する仕様

| 仕様 | 依存内容 |
|------|----------|
| `areka-P0-mcp-server` | HTTP基盤 |

### 依存される仕様

なし（拡張機能）

---

## Glossary

| 用語 | 定義 |
|------|------|
| **RSS** | Really Simple Syndication。Webサイト更新配信フォーマット |
| **スクレイピング** | Webページから情報を抽出すること |
| **カスタムURLプロトコル** | アプリケーション固有のURLスキーム（areka://等） |
