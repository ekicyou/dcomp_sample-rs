# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | areka 永続化システム 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Priority** | P0 (MVP必須) |

---

## Introduction

本仕様書は areka アプリケーションにおける永続化システムの要件を定義する。アプリケーション設定やキャラクターとの思い出を保存し、継続的な関係性を築くことを目的とする。

### 親仕様からのトレーサビリティ

本仕様は `ukagaka-desktop-mascot` の以下の要件をカバーする：

| 親要件ID | 内容 |
|----------|------|
| 9.1 | アプリケーション設定をファイルに保存・読み込みできる |
| 9.2 | ゴーストごとの変数を永続化できる |
| 9.4 | 設定が変更された時、即座に変更を反映する |
| 9.5 | 設定UIを提供する |
| 9.6 | 設定のエクスポート・インポートをサポートする |
| 30.6 | 定期的自動保存 |
| 30.7 | 保存失敗時のリトライ |
| 30.8 | バックアップの作成 |

### スコープ

**含まれるもの:**
- アプリケーション設定の保存・読み込み
- ゴースト状態（変数、記憶）の永続化
- 定期的自動保存
- エクスポート・インポート
- バックアップ

**含まれないもの:**
- 会話履歴の高度な管理（areka-P2-memory-system の責務）
- クラウド同期（areka-P3-cloud-sync の責務）
- クラッシュからの復元（areka-P1-error-recovery の責務）

---

## Requirements

### Requirement 1: アプリケーション設定

**Objective:** ユーザーとして、アプリケーションの設定を保存したい。それにより好みの設定を維持できる。

#### Acceptance Criteria

1. **The** Persistence System **shall** アプリケーション設定をファイルに保存できる
2. **The** Persistence System **shall** アプリケーション設定をファイルから読み込める
3. **When** 設定が変更された時, **the** Persistence System **shall** 即座に変更を反映する（再起動不要）
4. **The** Persistence System **shall** 設定ファイルの形式は TOML とする
5. **The** Persistence System **shall** 設定ファイルが存在しない場合、デフォルト値を使用する

---

### Requirement 2: ゴースト状態の永続化

**Objective:** ユーザーとして、キャラクターとの関係性を保存したい。それにより継続的な関係を築ける。

#### Acceptance Criteria

1. **The** Persistence System **shall** ゴーストごとの変数（好感度、フラグ等）を永続化できる
2. **The** Persistence System **shall** ゴーストの状態を個別のファイルに保存する
3. **The** Persistence System **shall** 変数の型（文字列、数値、真偽値、配列、辞書）を保持する
4. **When** ゴーストが終了した時, **the** Persistence System **shall** 状態を保存する

---

### Requirement 3: 自動保存

**Objective:** ユーザーとして、データ消失を防ぎたい。それにより安心してアプリケーションを使用できる。

#### Acceptance Criteria

1. **The** Persistence System **shall** 定期的に自動保存を実行する（デフォルト: 5分間隔）
2. **The** Persistence System **shall** 自動保存の間隔を設定できる
3. **The** Persistence System **shall** 自動保存を無効化できる
4. **When** 保存が失敗した時, **the** Persistence System **shall** リトライを行う
5. **When** 保存が失敗した時, **the** Persistence System **shall** ユーザーに通知する

---

### Requirement 4: バックアップ

**Objective:** ユーザーとして、データの破損に備えたい。それにより万が一の際に復元できる。

#### Acceptance Criteria

1. **The** Persistence System **shall** 保存前に既存ファイルのバックアップを作成する
2. **The** Persistence System **shall** 指定世代数のバックアップを保持する（デフォルト: 3世代）
3. **The** Persistence System **shall** バックアップから復元できる
4. **The** Persistence System **shall** 古いバックアップを自動削除する

---

### Requirement 5: エクスポート・インポート

**Objective:** ユーザーとして、設定やデータを移行したい。それにより環境移行やバックアップができる。

#### Acceptance Criteria

1. **The** Persistence System **shall** 設定をエクスポートできる
2. **The** Persistence System **shall** ゴースト状態をエクスポートできる
3. **The** Persistence System **shall** エクスポートファイルからインポートできる
4. **The** Persistence System **shall** インポート時に既存データとのマージ/上書きを選択できる
5. **The** Persistence System **shall** エクスポート形式はZIPアーカイブとする

---

### Requirement 6: データパス管理

**Objective:** 開発者として、データの保存場所を適切に管理したい。それにより整合性のあるデータ管理ができる。

#### Acceptance Criteria

1. **The** Persistence System **shall** Windows標準のアプリケーションデータフォルダを使用する（%APPDATA%/areka）
2. **The** Persistence System **shall** ポータブルモード（実行ファイルと同じディレクトリ）をサポートする
3. **The** Persistence System **shall** データディレクトリのパスを設定できる
4. **The** Persistence System **shall** 必要なディレクトリを自動作成する

---

## Non-Functional Requirements

### NFR-1: パフォーマンス

1. 設定読み込みは100ms以内で完了すること
2. 自動保存はバックグラウンドで実行し、UI操作を妨げないこと
3. 大量の変数（1000件以上）でも適切なパフォーマンスを維持すること

### NFR-2: 信頼性

1. 書き込み途中での電源断に対してデータを保護すること（アトミック書き込み）
2. ファイル破損を検出し、バックアップから復元を提案すること

### NFR-3: セキュリティ

1. 機密性の高いデータ（将来的な暗号化対応の準備）の格納場所を分離すること

---

## Dependencies

### 依存する仕様

なし（独立）

### 依存される仕様

| 仕様 | 依存内容 |
|------|----------|
| `areka-P0-package-manager` | パッケージ設定の保存 |
| `areka-P0-reference-ghost` | ゴースト変数の保存 |
| `areka-P2-memory-system` | 会話履歴の保存 |
| `areka-P2-privacy-security` | 暗号化保存 |
| `areka-P1-error-recovery` | 状態復元 |

---

## Glossary

| 用語 | 定義 |
|------|------|
| **永続化** | データをファイル等に保存し、アプリケーション終了後も保持すること |
| **アトミック書き込み** | 書き込みが完全に成功するか、全く書き込まれないかのどちらかを保証する方式 |
| **バックアップ** | データのコピーを別の場所に保存すること |
| **エクスポート** | データを外部ファイルに出力すること |
| **インポート** | 外部ファイルからデータを取り込むこと |
