# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | areka システムトレイ 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Priority** | P0 (MVP必須) |

---

## Introduction

本仕様書は areka アプリケーションにおけるシステムトレイ機能の要件を定義する。バックグラウンドでの常駐とユーザーアクセスの提供を目的とする。

### 親仕様からのトレーサビリティ

本仕様は `ukagaka-desktop-mascot` の以下の要件をカバーする：

| 親要件ID | 内容 |
|----------|------|
| 13.1 | システムトレイにアイコンを表示できる |
| 13.2 | システムトレイアイコンをクリックした時、メニューを表示する |
| 13.3 | 最小化操作が行われた時、システムトレイに格納できる |
| 13.4 | Windows起動時に自動起動するオプションを提供する |
| 13.5 | 終了操作が行われた時、終了確認ダイアログを表示できる |

### スコープ

**含まれるもの:**
- システムトレイアイコン表示
- トレイメニュー
- 最小化時のトレイ格納
- Windows起動時の自動起動

**含まれないもの:**
- 設定UI本体（別仕様）
- ゴースト切り替えUI本体（areka-P0-package-manager の責務）

---

## Requirements

### Requirement 1: システムトレイアイコン

**Objective:** ユーザーとして、アプリケーションがバックグラウンドで動作していることを確認したい。それにより常駐状態を把握できる。

#### Acceptance Criteria

1. **The** System Tray **shall** システムトレイにアイコンを表示できる
2. **The** System Tray **shall** アイコンをアプリケーションロゴに設定できる
3. **The** System Tray **shall** アイコンにツールチップ（アプリ名等）を表示できる
4. **The** System Tray **shall** アイコンの状態（通常/通知あり等）を変更できる

---

### Requirement 2: トレイメニュー

**Objective:** ユーザーとして、トレイアイコンからアプリケーションを操作したい。それにより素早くアクセスできる。

#### Acceptance Criteria

1. **When** システムトレイアイコンを右クリックした時, **the** System Tray **shall** コンテキストメニューを表示する
2. **The** System Tray **shall** メニュー項目「表示/非表示」を提供する
3. **The** System Tray **shall** メニュー項目「ゴースト切り替え」（サブメニュー）を提供する
4. **The** System Tray **shall** メニュー項目「設定」を提供する
5. **The** System Tray **shall** メニュー項目「終了」を提供する
6. **The** System Tray **shall** カスタムメニュー項目を追加できる

---

### Requirement 3: トレイ格納

**Objective:** ユーザーとして、アプリケーションを最小化してトレイに格納したい。それによりデスクトップを整理できる。

#### Acceptance Criteria

1. **When** 最小化操作が行われた時, **the** System Tray **shall** キャラクターウィンドウを非表示にできる
2. **When** 最小化操作が行われた時, **the** System Tray **shall** タスクバーから非表示にできる（オプション）
3. **When** トレイアイコンをダブルクリックした時, **the** System Tray **shall** キャラクターウィンドウを表示する
4. **The** System Tray **shall** 「閉じる」ボタンでトレイ格納するかを設定できる

---

### Requirement 4: 自動起動

**Objective:** ユーザーとして、PC起動時にアプリケーションを自動起動したい。それによりいつでもキャラクターに会える。

#### Acceptance Criteria

1. **The** System Tray **shall** Windows起動時に自動起動するオプションを提供する
2. **The** System Tray **shall** 自動起動の有効/無効を設定できる
3. **When** 自動起動が有効な時, **the** System Tray **shall** Windowsスタートアップに登録する
4. **The** System Tray **shall** 自動起動時に最小化状態で起動するオプションを提供する

---

### Requirement 5: 終了確認

**Objective:** ユーザーとして、誤って終了することを防ぎたい。それにより意図しない終了を回避できる。

#### Acceptance Criteria

1. **When** 終了操作が行われた時, **the** System Tray **shall** 終了確認ダイアログを表示できる（オプション）
2. **The** System Tray **shall** 終了確認の有効/無効を設定できる
3. **The** System Tray **shall** 「今後表示しない」オプションを提供する
4. **When** 終了が確定した時, **the** System Tray **shall** アプリケーションを安全に終了する

---

## Non-Functional Requirements

### NFR-1: パフォーマンス

1. トレイアイコン表示は起動時に即座に行うこと
2. メニュー表示は100ms以内で完了すること

### NFR-2: 互換性

1. Windows 10/11 のシステムトレイに正しく表示されること
2. 高DPI環境でアイコンが正しく表示されること

---

## Dependencies

### 依存する仕様

なし（独立）

### 依存される仕様

| 仕様 | 依存内容 |
|------|----------|
| `areka-P0-package-manager` | ゴースト切り替えメニュー |

---

## Glossary

| 用語 | 定義 |
|------|------|
| **システムトレイ** | Windowsタスクバー右側の通知領域 |
| **トレイアイコン** | システムトレイに表示されるアプリケーションアイコン |
| **自動起動** | Windows起動時にアプリケーションを自動的に起動する機能 |
