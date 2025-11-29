# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | areka スクリプトエンジン 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Priority** | P0 (MVP必須) |

---

## Introduction

本仕様書は areka アプリケーションにおけるスクリプトエンジンの要件を定義する。キャラクターとの自然な会話を実現し、人格と魅力を表現することを目的とする。

### 親仕様からのトレーサビリティ

本仕様は `ukagaka-desktop-mascot` の以下の要件をカバーする：

| 親要件ID | 内容 |
|----------|------|
| 4.1 | 里々にインスパイアされた対話記述DSLを解釈・実行できる |
| 4.3 | ユーザーがキャラクターをダブルクリックした時、対話イベントを発火する |
| 4.4 | 変数を保持し、スクリプト内で参照・更新できる |
| 4.5 | 条件分岐・ループ等の制御構文をサポートする |
| 4.6 | 複数キャラクター間での会話（掛け合い、漫才的やりとり）をスクリプトで記述できる |
| 4.7 | 発言者の切り替え、割り込み、同時発言などの会話制御ができる |
| 29.6 | さくらスクリプトの基本コマンドを出力できる |
| 29.7 | 独自拡張コマンドを定義できる |
| 29.8 | スクリプト出力とMCPコマンドを組み合わせられる |

### スコープ

**含まれるもの:**
- 対話記述DSL（里々インスパイア）のパーサーと実行エンジン
- さくらスクリプト互換コマンドの出力
- 変数管理（グローバル/ローカル）
- 制御構文（条件分岐、ループ、関数）
- 複数キャラクター会話制御

**含まれないもの:**
- LLM連携（areka-P2-llm-integration の責務）
- ゴーストパッケージ管理（areka-P0-package-manager の責務）
- タイプライター表示（wintf-P0-typewriter の責務）

---

## Requirements

### Requirement 1: 対話記述DSL

**Objective:** ゴースト制作者として、自然な会話を簡潔に記述したい。それにより効率的にゴーストを制作できる。

#### Acceptance Criteria

1. **The** Script Engine **shall** 里々にインスパイアされた対話記述DSLを解釈・実行できる
2. **The** Script Engine **shall** トーク（発言ブロック）を定義できる
3. **The** Script Engine **shall** トーク内でテキストと制御コマンドを混在できる
4. **The** Script Engine **shall** トークの呼び出し（関数呼び出し）をサポートする
5. **The** Script Engine **shall** UTF-8エンコーディングのスクリプトファイルを読み込める

---

### Requirement 2: さくらスクリプト互換出力

**Objective:** ゴースト制作者として、既存のさくらスクリプト知識を活用したい。それにより学習コストを削減できる。

#### Acceptance Criteria

1. **The** Script Engine **shall** さくらスクリプトの基本コマンドを出力できる
2. **The** Script Engine **shall** サーフェス切り替えコマンド（`\s[n]`）を出力できる
3. **The** Script Engine **shall** ウェイトコマンド（`\w[n]`）を出力できる
4. **The** Script Engine **shall** 発言者切り替えコマンド（`\0`, `\1`等）を出力できる
5. **The** Script Engine **shall** 改行コマンド（`\n`）を出力できる
6. **The** Script Engine **shall** 独自拡張コマンドを定義・出力できる

---

### Requirement 3: 変数管理

**Objective:** ゴースト制作者として、キャラクターの状態を変数で管理したい。それにより動的な会話を実現できる。

#### Acceptance Criteria

1. **The** Script Engine **shall** グローバル変数を保持・参照・更新できる
2. **The** Script Engine **shall** ローカル変数（トーク内スコープ）をサポートする
3. **The** Script Engine **shall** 変数の型（文字列、数値、真偽値）をサポートする
4. **The** Script Engine **shall** 変数を文字列展開（`%(変数名)`等）できる
5. **The** Script Engine **shall** システム変数（日時、カウンター等）を提供する

---

### Requirement 4: 制御構文

**Objective:** ゴースト制作者として、条件分岐やループで複雑なロジックを記述したい。それにより多様な会話パターンを実現できる。

#### Acceptance Criteria

1. **The** Script Engine **shall** 条件分岐（if/else）をサポートする
2. **The** Script Engine **shall** ループ（while/for/repeat）をサポートする
3. **The** Script Engine **shall** 比較演算子（==, !=, <, >, <=, >=）をサポートする
4. **The** Script Engine **shall** 論理演算子（and, or, not）をサポートする
5. **The** Script Engine **shall** 算術演算子（+, -, *, /, %）をサポートする
6. **The** Script Engine **shall** ランダム選択（複数候補から1つを選択）をサポートする

---

### Requirement 5: 複数キャラクター会話制御

**Objective:** ゴースト制作者として、複数キャラクター間の掛け合いを記述したい。それにより漫才的なやりとりを実現できる。

#### Acceptance Criteria

1. **The** Script Engine **shall** 発言者（キャラクター）を切り替えられる
2. **The** Script Engine **shall** 複数キャラクターが同時に発言できる（同期発言）
3. **The** Script Engine **shall** キャラクターが他のキャラクターの発言に割り込めむ
4. **The** Script Engine **shall** キャラクター間でスコープ（変数、状態）を共有できる
5. **The** Script Engine **shall** 2体以上のキャラクター会話をサポートする

---

### Requirement 6: イベントハンドリング

**Objective:** ゴースト制作者として、ユーザーの操作に応じた会話を記述したい。それによりインタラクティブな体験を実現できる。

#### Acceptance Criteria

1. **When** ユーザーがキャラクターをクリックした時, **the** Script Engine **shall** 対応するイベントハンドラを呼び出す
2. **When** ユーザーがキャラクターをダブルクリックした時, **the** Script Engine **shall** 対話イベントを発火する
3. **The** Script Engine **shall** イベント名でハンドラを定義できる
4. **The** Script Engine **shall** イベント引数（クリック位置、キャラクターID等）をハンドラに渡す
5. **The** Script Engine **shall** 未定義イベントのデフォルトハンドラを設定できる

---

## Non-Functional Requirements

### NFR-1: パフォーマンス

1. スクリプト解析は起動時に完了すること
2. イベントハンドラの実行は10ms以内に開始すること
3. メモリ使用量は妥当な範囲に抑えること（スクリプトサイズに比例）

### NFR-2: エラーハンドリング

1. 構文エラーを検出し、エラー位置（行番号）を報告すること
2. ランタイムエラー発生時も可能な限り継続動作すること
3. エラーメッセージは制作者が理解しやすいものであること

### NFR-3: 拡張性

1. 新しいコマンドを追加可能な設計とすること
2. 外部モジュール（MCP等）との連携が可能な設計とすること

---

## Dependencies

### 依存する仕様

| 仕様 | 依存内容 |
|------|----------|
| `wintf-P0-animation-system` | サーフェス切り替えの実行 |
| `wintf-P0-balloon-system` | テキスト表示の実行 |

### 依存される仕様

| 仕様 | 依存内容 |
|------|----------|
| `areka-P0-reference-ghost` | スクリプト実行 |
| `areka-P1-devtools` | デバッグ機能 |
| `areka-P2-llm-integration` | LLM応答との統合 |

---

## Glossary

| 用語 | 定義 |
|------|------|
| **DSL** | Domain Specific Language。特定用途向けの言語 |
| **里々** | 伺かゴースト制作用の対話記述言語 |
| **さくらスクリプト** | 伺かの標準スクリプト形式 |
| **トーク** | 1つの発言ブロック（会話単位） |
| **サーフェス** | キャラクターの表情・ポーズ画像 |
| **イベントハンドラ** | 特定のイベントに応じて実行される処理 |
