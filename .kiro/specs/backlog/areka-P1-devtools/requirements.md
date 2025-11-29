# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | areka 開発者ツール 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Priority** | P1 (体験向上) |

---

## Introduction

本仕様書は areka アプリケーションにおける開発者ツール（devtools）の要件を定義する。ゴースト制作者の開発効率向上とデバッグ支援を目的とする。

### 親仕様からのトレーサビリティ

本仕様は `ukagaka-desktop-mascot` の以下の要件をカバーする：

| 親要件ID | 内容 |
|----------|------|
| 18.1 | ゴースト開発者向けのデバッグログ表示機能を提供する |
| 18.2 | イベント発火のモニタリング・ログ表示ができる |
| 18.3 | サーフェスのプレビュー・切り替えテストができる |
| 18.4 | スクリプト実行のステップ実行・ブレークポイントをサポートする |
| 18.5 | 当たり判定領域の可視化ができる |
| 18.6 | ホットリロード（スクリプト変更の即時反映）をサポートする |

### スコープ

**含まれるもの:**
- デバッグログビューア
- イベントモニター
- サーフェスプレビュー
- 当たり判定可視化
- ホットリロード

**含まれないもの:**
- 統合IDE（外部エディタ連携のみ）
- ビジュアルエディタ（将来検討）

---

## Requirements

### Requirement 1: デバッグログ

**Objective:** ゴースト制作者として、デバッグ情報を確認したい。それにより問題の原因を特定できる。

#### Acceptance Criteria

1. **The** Devtools **shall** デバッグログをリアルタイムで表示するウィンドウを提供する
2. **The** Devtools **shall** ログレベル（debug, info, warn, error）でフィルタリングできる
3. **The** Devtools **shall** ログをファイルに保存できる
4. **The** Devtools **shall** タイムスタンプとログソースを表示する
5. **The** Devtools **shall** ゴーストからprint/log関数で出力を追加できる

---

### Requirement 2: イベントモニター

**Objective:** ゴースト制作者として、発火するイベントを監視したい。それによりイベント処理の動作を確認できる。

#### Acceptance Criteria

1. **The** Devtools **shall** 発火したイベントをリスト表示する
2. **The** Devtools **shall** イベントの引数（パラメータ）を展開表示できる
3. **The** Devtools **shall** イベントの種類でフィルタリングできる
4. **The** Devtools **shall** イベントを手動で発火できる（テスト用）

---

### Requirement 3: サーフェスプレビュー

**Objective:** ゴースト制作者として、サーフェスを個別にプレビューしたい。それにより見た目の確認ができる。

#### Acceptance Criteria

1. **The** Devtools **shall** 全サーフェスの一覧をサムネイル表示する
2. **The** Devtools **shall** サーフェスをクリックで切り替えられる
3. **The** Devtools **shall** アニメーションをステップ再生できる
4. **The** Devtools **shall** サーフェスIDを指定して直接ジャンプできる

---

### Requirement 4: 当たり判定可視化

**Objective:** ゴースト制作者として、当たり判定領域を確認したい。それにより意図通りの領域を設定できる。

#### Acceptance Criteria

1. **The** Devtools **shall** 当たり判定領域をオーバーレイ表示できる
2. **The** Devtools **shall** 各領域のIDを表示できる
3. **When** マウスが領域上にある時, **the** Devtools **shall** その領域をハイライトする
4. **The** Devtools **shall** 当たり判定の有効/無効を切り替えられる

---

### Requirement 5: ホットリロード

**Objective:** ゴースト制作者として、変更を即座に反映したい。それにより開発サイクルを高速化できる。

#### Acceptance Criteria

1. **The** Devtools **shall** スクリプトファイルの変更を検知して再読み込みする
2. **The** Devtools **shall** シェルファイルの変更を検知して再読み込みする
3. **The** Devtools **shall** 再読み込み時に状態を維持するか選択できる
4. **The** Devtools **shall** 再読み込みエラーを通知する

---

### Requirement 6: インスペクタ

**Objective:** ゴースト制作者として、内部状態を確認したい。それにより変数の値やゴースト状態を把握できる。

#### Acceptance Criteria

1. **The** Devtools **shall** ゴーストの変数（メモリ）をツリー表示できる
2. **The** Devtools **shall** 変数の値を編集できる
3. **The** Devtools **shall** 現在のサーフェス・バルーン状態を表示する

---

## Non-Functional Requirements

### NFR-1: パフォーマンス

1. devtools有効時のオーバーヘッドは5%以内であること
2. ログ表示は1000行/秒以上の出力に対応すること

### NFR-2: 使いやすさ

1. devtoolsはショートカットキーで開閉できること
2. リリースビルドではdevtools機能を無効化できること

---

## Dependencies

### 依存する仕様

| 仕様 | 依存内容 |
|------|----------|
| `areka-P0-mcp-server` | イベント監視 |
| `areka-P0-script-engine` | スクリプト状態取得 |

### 依存される仕様

なし（開発支援機能）

---

## Glossary

| 用語 | 定義 |
|------|------|
| **devtools** | 開発者ツール。デバッグ・テスト支援機能の総称 |
| **ホットリロード** | アプリケーション再起動なしでコード変更を反映する機能 |
| **インスペクタ** | オブジェクトの内部状態を表示・編集するツール |
