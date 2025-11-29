# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | areka エラーリカバリ 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Priority** | P1 (体験向上) |

---

## Introduction

本仕様書は areka アプリケーションにおけるエラーリカバリ機能の要件を定義する。異常発生時もユーザー体験を維持し、安全に回復することを目的とする。

### 親仕様からのトレーサビリティ

本仕様は `ukagaka-desktop-mascot` の以下の要件をカバーする：

| 親要件ID | 内容 |
|----------|------|
| 27.3 | スクリプトエラー発生時、エラー情報を表示してゴーストを継続動作させる |
| 27.4 | ゴースト読み込みエラー時、エラー表示ゴーストに切り替える |
| 27.5 | 致命的エラー発生時、クラッシュレポートを生成する |

### スコープ

**含まれるもの:**
- スクリプトエラー処理
- ゴースト読み込みエラー処理
- クラッシュレポート生成
- フォールバックゴースト

**含まれないもの:**
- スクリプト実行自体（areka-P0-script-engine の責務）
- パッケージ管理（areka-P0-package-manager の責務）

---

## Requirements

### Requirement 1: スクリプトエラー処理

**Objective:** ユーザーとして、スクリプトエラーが発生してもゴーストを使い続けたい。それにより中断なく会話できる。

#### Acceptance Criteria

1. **When** スクリプト実行エラーが発生した時, **the** Recovery System **shall** エラー情報を収集する
2. **When** スクリプト実行エラーが発生した時, **the** Recovery System **shall** ゴーストを安全な状態に戻す
3. **The** Recovery System **shall** エラーメッセージをユーザーに通知する（devtools/バルーン）
4. **The** Recovery System **shall** 連続エラー回数を監視し、閾値超過時に警告する
5. **The** Recovery System **shall** エラー後も他の機能（最小化、終了等）を利用可能に維持する

---

### Requirement 2: ゴースト読み込みエラー

**Objective:** ユーザーとして、壊れたゴーストがあってもアプリを使いたい。それにより他のゴーストを選択できる。

#### Acceptance Criteria

1. **When** ゴーストの読み込みに失敗した時, **the** Recovery System **shall** エラー表示ゴーストに切り替える
2. **The** Recovery System **shall** エラー表示ゴーストでエラー内容を表示する
3. **The** Recovery System **shall** エラー表示ゴーストからゴースト切り替えを可能にする
4. **The** Recovery System **shall** 読み込みエラーの詳細をログに記録する
5. **The** Recovery System **shall** 部分的な読み込み（シェルのみ等）をフォールバックとしてサポートする

---

### Requirement 3: フォールバックゴースト

**Objective:** 開発者として、どのような状況でも最低限の動作を保証したい。それによりユーザーが完全にブロックされることを防ぐ。

#### Acceptance Criteria

1. **The** Recovery System **shall** 組み込みのフォールバックゴーストを提供する
2. **The** Recovery System **shall** フォールバックゴーストは外部依存なしで動作する
3. **The** Recovery System **shall** フォールバックゴーストで基本操作（ゴースト切替、終了）を提供する
4. **The** Recovery System **shall** フォールバックゴーストでエラー情報を表示する

---

### Requirement 4: クラッシュレポート

**Objective:** 開発者として、致命的エラーの詳細を収集したい。それにより問題を修正できる。

#### Acceptance Criteria

1. **When** 致命的エラー（パニック）が発生した時, **the** Recovery System **shall** クラッシュレポートを生成する
2. **The** Recovery System **shall** クラッシュレポートにスタックトレースを含める
3. **The** Recovery System **shall** クラッシュレポートに環境情報（OS、バージョン等）を含める
4. **The** Recovery System **shall** クラッシュレポートをファイルに保存する
5. **The** Recovery System **shall** ユーザーにクラッシュレポートの場所を通知する（次回起動時）

---

### Requirement 5: 状態保存と復旧

**Objective:** ユーザーとして、クラッシュ後も状態を復元したい。それにより作業を継続できる。

#### Acceptance Criteria

1. **The** Recovery System **shall** 定期的に状態をスナップショット保存する
2. **When** 異常終了から復帰した時, **the** Recovery System **shall** 状態復元を提案する
3. **The** Recovery System **shall** 復元する状態の範囲をユーザーが選択できる

---

## Non-Functional Requirements

### NFR-1: 信頼性

1. エラー処理自体がエラーを起こさないこと
2. フォールバックゴーストは確実に起動すること

### NFR-2: ユーザビリティ

1. エラーメッセージはユーザーフレンドリーであること
2. 復旧手順を明示すること

---

## Dependencies

### 依存する仕様

| 仕様 | 依存内容 |
|------|----------|
| `areka-P0-script-engine` | スクリプトエラー検出 |
| `areka-P0-package-manager` | ゴースト切り替え |

### 依存される仕様

なし（基盤機能）

---

## Glossary

| 用語 | 定義 |
|------|------|
| **エラーリカバリ** | エラー発生後に安全な状態に復帰する機能 |
| **フォールバックゴースト** | どのゴーストも読み込めない場合に使用される組み込みゴースト |
| **クラッシュレポート** | 致命的エラー発生時に生成される診断情報 |
| **パニック** | Rustにおける回復不能なエラー状態 |
