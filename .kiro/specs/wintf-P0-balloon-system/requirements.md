# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | wintf バルーンシステム 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Priority** | P0 (MVP必須) |

---

## Introduction

本仕様書は wintf フレームワークにおけるバルーン（吹き出し）システムの要件を定義する。キャラクターの発言を視覚的に表示し、ユーザーとの対話を実現することを目的とする。

### 親仕様からのトレーサビリティ

本仕様は `ukagaka-desktop-mascot` の以下の要件をカバーする：

| 親要件ID | 内容 |
|----------|------|
| 3.1 | キャラクターに紐付いた吹き出しウィンドウを表示できる |
| 3.2 | 複数キャラクターそれぞれに独立した吹き出しを表示できる |
| 3.3 | 吹き出し内にテキストを表示できる |
| 3.7 | テキストがクリックされた時、リンクとしてアクションを実行できる |
| 3.8 | テキスト表示中、ルビ（ふりがな）を表示できる |
| 3.9 | 選択肢形式の入力をユーザーに提示できる |
| 3.10 | ユーザーが選択肢をクリックした時、対応するイベントを発火する |

### スコープ

**含まれるもの:**
- バルーンウィンドウの生成・配置・管理
- テキスト表示（縦書き/横書き対応）
- 選択肢UI（選択肢ボタン、クリックイベント）
- リンク（クリッカブルテキスト）
- ルビ（ふりがな）表示
- 入力ボックス

**含まれないもの:**
- タイプライター表示制御（wintf-P0-typewriter の責務）
- バルーンスキンの定義（areka-P0-reference-balloon の責務）
- 縦書きテキストレンダリング詳細（wintf-P0-typewriter の責務）

---

## Requirements

### Requirement 1: バルーンウィンドウ管理

**Objective:** 開発者として、キャラクターに紐付いたバルーンウィンドウを表示・管理したい。それによりキャラクターの発言を視覚的に表現できる。

#### Acceptance Criteria

1. **The** Balloon System **shall** キャラクターに紐付いたバルーンウィンドウを生成できる
2. **The** Balloon System **shall** 複数のキャラクターそれぞれに独立したバルーンを表示できる
3. **The** Balloon System **shall** バルーンをキャラクターウィンドウの近傍に自動配置できる
4. **When** キャラクターウィンドウが移動した時, **the** Balloon System **shall** バルーンの位置を追従させる
5. **The** Balloon System **shall** バルーンの表示/非表示を制御できる
6. **The** Balloon System **shall** バルーンのサイズを内容に応じて自動調整できる
7. **The** Balloon System **shall** バルーンの配置方向（上/下/左/右）を指定できる

---

### Requirement 2: テキスト表示

**Objective:** 開発者として、バルーン内にテキストを表示したい。それによりキャラクターの発言を読者に伝えられる。

#### Acceptance Criteria

1. **The** Balloon System **shall** バルーン内にテキストを表示できる
2. **The** Balloon System **shall** 縦書きテキストをサポートする
3. **The** Balloon System **shall** 横書きテキストをサポートする
4. **The** Balloon System **shall** テキストの折り返し（ワードラップ）をサポートする
5. **The** Balloon System **shall** テキストのスクロールをサポートする（長文対応）
6. **The** Balloon System **shall** フォント、サイズ、色をカスタマイズできる

---

### Requirement 3: ルビ（ふりがな）表示

**Objective:** 開発者として、テキストにルビ（ふりがな）を付加したい。それにより漢字の読み方を示せる。

#### Acceptance Criteria

1. **The** Balloon System **shall** テキストにルビ（ふりがな）を表示できる
2. **The** Balloon System **shall** 縦書き時のルビ配置（右側）をサポートする
3. **The** Balloon System **shall** 横書き時のルビ配置（上側）をサポートする
4. **The** Balloon System **shall** ルビのフォントサイズを親文字に対して自動調整できる

---

### Requirement 4: リンク（クリッカブルテキスト）

**Objective:** 開発者として、テキスト内にクリック可能なリンクを設定したい。それによりユーザーのアクションをトリガーできる。

#### Acceptance Criteria

1. **The** Balloon System **shall** テキスト内にクリッカブルなリンクを設定できる
2. **When** リンクがクリックされた時, **the** Balloon System **shall** 対応するイベントを発火する
3. **The** Balloon System **shall** リンクの外観（色、下線等）をカスタマイズできる
4. **When** マウスがリンク上にある時, **the** Balloon System **shall** ホバー状態を視覚的にフィードバックする

---

### Requirement 5: 選択肢UI

**Objective:** 開発者として、ユーザーに選択肢を提示したい。それによりインタラクティブな会話を実現できる。

#### Acceptance Criteria

1. **The** Balloon System **shall** 選択肢形式の入力をユーザーに提示できる
2. **The** Balloon System **shall** 複数の選択肢を縦並びで表示できる
3. **When** ユーザーが選択肢をクリックした時, **the** Balloon System **shall** 対応するイベントを発火する
4. **The** Balloon System **shall** 選択肢のホバー状態を視覚的にフィードバックする
5. **The** Balloon System **shall** キーボード操作（上下キー、Enter）での選択をサポートする

---

### Requirement 6: 入力ボックス

**Objective:** 開発者として、ユーザーからテキスト入力を受け取りたい。それにより自由形式の応答を受け取れる。

#### Acceptance Criteria

1. **The** Balloon System **shall** テキスト入力ボックスをバルーン内に表示できる
2. **When** ユーザーがテキストを入力してEnterを押した時, **the** Balloon System **shall** 入力内容を含むイベントを発火する
3. **The** Balloon System **shall** 入力ボックスのプレースホルダーテキストを設定できる
4. **The** Balloon System **shall** 入力文字数の制限を設定できる

---

## Non-Functional Requirements

### NFR-1: パフォーマンス

1. バルーン表示時の描画遅延は16ms（60fps相当）以内であること
2. 長文テキストのスクロールが滑らかであること

### NFR-2: 互換性

1. Windows 10 (1803) 以降をサポートすること
2. 高DPI環境でスケーリングが正しく動作すること

### NFR-3: アクセシビリティ

1. キーボードのみでの操作をサポートすること（選択肢、入力ボックス）

---

## Dependencies

### 依存する仕様

| 仕様 | 依存内容 |
|------|----------|
| `wintf-P0-typewriter` | タイプライター表示、文字単位制御 |

### 依存される仕様

| 仕様 | 依存内容 |
|------|----------|
| `areka-P0-reference-balloon` | バルーンスキンの適用 |

---

## Glossary

| 用語 | 定義 |
|------|------|
| **バルーン** | キャラクターの発言を表示する吹き出しウィンドウ |
| **ルビ** | 漢字等の上または横に付けるふりがな |
| **選択肢** | ユーザーがクリックして選ぶ複数の選択肢ボタン |
| **リンク** | クリック可能なテキスト領域 |
| **入力ボックス** | ユーザーがテキストを入力するフィールド |
