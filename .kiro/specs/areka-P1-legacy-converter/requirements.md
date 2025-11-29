# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | areka レガシーコンバーター 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Priority** | P1 (体験向上) |

---

## Introduction

本仕様書は areka アプリケーションにおけるレガシー（伺か）アセット変換機能の要件を定義する。既存の伺かゴースト/シェルをareksa形式に変換し、互換性を提供することを目的とする。

### 親仕様からのトレーサビリティ

本仕様は `ukagaka-desktop-mascot` の以下の要件をカバーする：

| 親要件ID | 内容 |
|----------|------|
| 17.1 | 既存の「伺か」シェル(shell)を読み込み、表示できる |
| 17.2 | 既存の「伺か」バルーンを読み込み、使用できる |
| 17.3 | descript.txtの基本パラメータを解釈できる |
| 17.4 | surfaces.txtのサーフェス定義を解釈できる |
| 17.5 | surface*.pngを表示に使用できる |
| 17.6 | SERIKO/MAYUNAアニメーション定義を解釈・再生できる |

### スコープ

**含まれるもの:**
- シェル変換（descript.txt, surfaces.txt, PNG）
- バルーン変換
- SERIKO/MAYUNAアニメーション変換
- areka形式への出力

**含まれないもの:**
- 里々/YAYAスクリプト変換（将来検討）
- 実行時のエミュレーション（変換後はネイティブ動作）

---

## Requirements

### Requirement 1: シェル構造解析

**Objective:** ユーザーとして、既存の伺かシェルを読み込みたい。それにより愛着のあるゴーストをarekで使える。

#### Acceptance Criteria

1. **The** Converter **shall** descript.txtを解析し、シェルメタデータを抽出する
2. **The** Converter **shall** surfaces.txtを解析し、サーフェス定義を抽出する
3. **The** Converter **shall** surface*.pngファイルを認識・処理する
4. **The** Converter **shall** サブディレクトリ（surface0/, surface1/等）の構造をサポートする

---

### Requirement 2: SERIKO/MAYUNAアニメーション変換

**Objective:** ユーザーとして、既存のアニメーション定義を使いたい。それによりキャラクターの動きを維持できる。

#### Acceptance Criteria

1. **The** Converter **shall** SERIKO（基本アニメーション）定義を解析する
2. **The** Converter **shall** MAYUNA（着せ替え）定義を解析する
3. **The** Converter **shall** アニメーションパターン（overlay, replace, base等）を変換する
4. **The** Converter **shall** アニメーションタイミング（interval, delay）を変換する
5. **The** Converter **shall** 当たり判定（collision）定義を変換する

---

### Requirement 3: バルーン変換

**Objective:** ユーザーとして、既存のバルーンスキンを使いたい。それにより見慣れたUIでゴーストと会話できる。

#### Acceptance Criteria

1. **The** Converter **shall** バルーンのdescript.txtを解析する
2. **The** Converter **shall** バルーン画像（PNG）を処理する
3. **The** Converter **shall** バルーンの位置指定（origin, offset）を変換する
4. **The** Converter **shall** バルーンフォント設定を変換する

---

### Requirement 4: areka形式出力

**Objective:** 開発者として、変換結果をareka形式で保存したい。それにより変換後は高速に読み込める。

#### Acceptance Criteria

1. **The** Converter **shall** 変換結果をareka ghost package形式で出力する
2. **The** Converter **shall** 画像をWebP形式に最適化できる（オプション）
3. **The** Converter **shall** 変換レポート（警告、未対応機能等）を生成する
4. **The** Converter **shall** 部分変換（シェルのみ、バルーンのみ等）をサポートする

---

### Requirement 5: 互換性レベル

**Objective:** ユーザーとして、変換の互換性を確認したい。それにより動作しない場合の原因を把握できる。

#### Acceptance Criteria

1. **The** Converter **shall** 対応するsurfaces.txt構文バージョンを明示する
2. **The** Converter **shall** 未対応の構文・機能を検出し、警告する
3. **The** Converter **shall** 変換後のプレビュー機能を提供する（オプション）

---

## Non-Functional Requirements

### NFR-1: 互換性

1. SSP互換のshell構造の90%以上をサポートすること
2. SERIKO基本アニメーションの95%以上をサポートすること

### NFR-2: パフォーマンス

1. 一般的なシェル（100サーフェス以下）は10秒以内に変換完了すること

---

## Dependencies

### 依存する仕様

| 仕様 | 依存内容 |
|------|----------|
| `areka-P0-package-manager` | パッケージ形式定義 |

### 依存される仕様

なし（ユーティリティ機能）

---

## Glossary

| 用語 | 定義 |
|------|------|
| **シェル** | キャラクターのグラフィック・アニメーション定義 |
| **サーフェス** | キャラクターの個別ポーズ/表情 |
| **SERIKO** | 伺かのアニメーション定義形式 |
| **MAYUNA** | 伺かの着せ替え（レイヤー合成）定義形式 |
| **descript.txt** | 伺かシェル/バルーンのメタデータファイル |
| **surfaces.txt** | 伺かサーフェス・アニメーション定義ファイル |
