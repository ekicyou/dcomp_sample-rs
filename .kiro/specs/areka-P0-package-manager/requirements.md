# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | areka パッケージマネージャ 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Priority** | P0 (MVP必須) |

---

## Introduction

本仕様書は areka アプリケーションにおけるパッケージマネージャの要件を定義する。ゴースト、シェル、バルーンの導入・管理を行い、多様なキャラクターとの出会いを実現することを目的とする。

### 親仕様からのトレーサビリティ

本仕様は `ukagaka-desktop-mascot` の以下の要件をカバーする：

| 親要件ID | 内容 |
|----------|------|
| 7.1 | ゴーストパッケージを読み込める |
| 7.2 | 複数のゴーストをインストール・管理できる |
| 7.3 | ゴースト切り替え操作が行われた時、現在のゴーストを終了し、新しいゴーストを起動する |
| 7.4 | ゴーストのメタ情報を表示できる |
| 7.5 | オンラインからゴーストをダウンロード・インストールできる |
| 7.6 | ゴーストのアップデートを検知・適用できる |
| 7.7 | インストール完了イベントをゴーストに通知する |
| 8.1-8.5 | シェル管理 |
| 27.1 | パッケージフォーマット定義 |
| 31.1-31.5 | インストーラ機能 |

### スコープ

**含まれるもの:**
- パッケージ（ゴースト/シェル/バルーン）のインストール・アンインストール
- manifest.toml の解析
- 依存関係の解決
- アップデート検知
- パッケージメタ情報の表示

**含まれないもの:**
- ゴースト実行ロジック（areka-P0-script-engine の責務）
- シェル描画（wintf-P0-animation-system の責務）
- オンラインマーケットプレイス（将来仕様）

---

## Requirements

### Requirement 1: パッケージ読み込み

**Objective:** 開発者として、パッケージ（ゴースト/シェル/バルーン）を読み込みたい。それによりキャラクターを表示・動作させられる。

#### Acceptance Criteria

1. **The** Package Manager **shall** ゴーストパッケージを読み込める
2. **The** Package Manager **shall** シェルパッケージを読み込める
3. **The** Package Manager **shall** バルーンパッケージを読み込める
4. **The** Package Manager **shall** manifest.toml からパッケージ情報を解析する
5. **The** Package Manager **shall** パッケージ内のリソース（画像、スクリプト等）へのアクセスを提供する

---

### Requirement 2: パッケージインストール

**Objective:** ユーザーとして、新しいパッケージをインストールしたい。それにより様々なキャラクターと出会える。

#### Acceptance Criteria

1. **The** Package Manager **shall** ZIPアーカイブ形式のパッケージをインストールできる
2. **The** Package Manager **shall** フォルダ形式のパッケージをインストールできる
3. **The** Package Manager **shall** URLからパッケージをダウンロード・インストールできる
4. **When** インストールが完了した時, **the** Package Manager **shall** インストール完了イベントを発火する
5. **The** Package Manager **shall** インストール先ディレクトリを設定できる
6. **The** Package Manager **shall** 同名パッケージの上書きインストールを確認する

---

### Requirement 3: パッケージアンインストール

**Objective:** ユーザーとして、不要なパッケージを削除したい。それによりディスク容量を管理できる。

#### Acceptance Criteria

1. **The** Package Manager **shall** インストール済みパッケージをアンインストールできる
2. **The** Package Manager **shall** アンインストール時にユーザーデータの保持/削除を選択できる
3. **The** Package Manager **shall** 依存されているパッケージの削除を警告する
4. **When** アンインストールが完了した時, **the** Package Manager **shall** アンインストール完了イベントを発火する

---

### Requirement 4: パッケージ管理

**Objective:** ユーザーとして、インストール済みパッケージを一覧・管理したい。それにより所持キャラクターを把握できる。

#### Acceptance Criteria

1. **The** Package Manager **shall** インストール済みパッケージを一覧表示できる
2. **The** Package Manager **shall** パッケージのメタ情報（名前、作者、説明、バージョン）を表示できる
3. **The** Package Manager **shall** パッケージの有効/無効を切り替えられる
4. **The** Package Manager **shall** パッケージをカテゴリ（ゴースト/シェル/バルーン）でフィルタできる

---

### Requirement 5: ゴースト切り替え

**Objective:** ユーザーとして、起動するゴーストを切り替えたい。それにより気分に応じたキャラクターと過ごせる。

#### Acceptance Criteria

1. **When** ゴースト切り替え操作が行われた時, **the** Package Manager **shall** 現在のゴーストを終了する
2. **When** ゴースト切り替え操作が行われた時, **the** Package Manager **shall** 新しいゴーストを起動する
3. **The** Package Manager **shall** 最後に起動したゴーストを記憶し、次回起動時に復元する
4. **The** Package Manager **shall** ゴースト切り替え時に状態を保存する

---

### Requirement 6: シェル切り替え

**Objective:** ユーザーとして、同じゴーストの異なるシェルを切り替えたい。それにより気分に応じた外見を楽しめる。

#### Acceptance Criteria

1. **The** Package Manager **shall** 1つのゴーストに複数のシェルを関連付けられる
2. **When** シェル切り替え操作が行われた時, **the** Package Manager **shall** キャラクターの外見を切り替える
3. **The** Package Manager **shall** 使用中のシェルを記憶し、次回起動時に復元する
4. **The** Package Manager **shall** シェル固有のメタ情報を表示できる

---

### Requirement 7: アップデート管理

**Objective:** ユーザーとして、パッケージのアップデートを知りたい。それにより最新のコンテンツを楽しめる。

#### Acceptance Criteria

1. **The** Package Manager **shall** パッケージのバージョンを管理できる
2. **When** アップデートが利用可能な時, **the** Package Manager **shall** ユーザーに通知する
3. **The** Package Manager **shall** アップデートを適用できる
4. **The** Package Manager **shall** アップデート時にユーザーデータを保持する
5. **The** Package Manager **shall** アップデートソースURL を manifest.toml から取得する

---

### Requirement 8: マニフェスト仕様

**Objective:** パッケージ制作者として、パッケージ情報を記述したい。それによりユーザーにパッケージを伝えられる。

#### Acceptance Criteria

1. **The** Package Manager **shall** manifest.toml 形式をサポートする
2. **The** Package Manager **shall** 必須フィールド（name, version, type）を検証する
3. **The** Package Manager **shall** オプションフィールド（author, description, license, homepage）を読み込める
4. **The** Package Manager **shall** 依存関係（dependencies）を読み込める
5. **The** Package Manager **shall** クリエイター支援先リンク（support_links）を読み込める

---

## Non-Functional Requirements

### NFR-1: パフォーマンス

1. パッケージ一覧の取得は100ms以内で完了すること
2. パッケージインストールの進捗を表示すること

### NFR-2: 信頼性

1. インストール失敗時は元の状態に戻せること（ロールバック）
2. 破損したパッケージを検出・報告すること

### NFR-3: セキュリティ

1. インストール前にパッケージの整合性を検証すること（オプション）
2. ユーザーの明示的な許可なしに外部通信を行わないこと

---

## Dependencies

### 依存する仕様

なし（独立）

### 依存される仕様

| 仕様 | 依存内容 |
|------|----------|
| `areka-P0-reference-ghost` | ゴーストパッケージの読み込み |
| `areka-P0-reference-shell` | シェルパッケージの読み込み |
| `areka-P0-reference-balloon` | バルーンパッケージの読み込み |
| `areka-P2-creator-tools` | パッケージ作成支援 |

---

## Glossary

| 用語 | 定義 |
|------|------|
| **パッケージ** | ゴースト、シェル、バルーンの配布単位 |
| **ゴースト** | キャラクターの頭脳（スクリプト、人格） |
| **シェル** | キャラクターの外見（画像、アニメーション定義） |
| **バルーン** | 会話表示UIのスタイル |
| **manifest.toml** | パッケージ情報を記述するファイル |
