# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | areka キャラクターコミュニケーション 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Priority** | P1 (体験向上) |

---

## Introduction

本仕様書は areka アプリケーションにおけるキャラクター間コミュニケーション（同一ゴースト内のキャラクター間会話・連携）の要件を定義する。複数キャラクター（メインキャラ・相方キャラ等）の掛け合いを実現することを目的とする。

### 親仕様からのトレーサビリティ

本仕様は `ukagaka-desktop-mascot` の以下の要件をカバーする：

| 親要件ID | 内容 |
|----------|------|
| 26.1 | 同一ゴースト内の複数キャラクター（スコープ0=メインキャラ、スコープ1=相方）をサポートする |
| 26.2 | 各キャラクターが独立したサーフェスセットを持てる |
| 26.3 | 各キャラクターが独立したバルーンを持てる |
| 26.4 | キャラクター間での会話の受け渡し（交互発言）をサポートする |
| 26.5 | キャラクターごとのサーフェス同時変更をサポートする |
| 26.6 | スコープ指定（\0, \1等）でキャラクターを切り替えて表示制御できる |
| 26.7 | キャラクター間のウィンドウ位置関係を維持する（相対位置、重なり順） |
| 26.8 | キャラクターの表示/非表示を個別に制御できる |
| 26.9 | キャラクター数は設定で変更可能（デフォルト2、最大4程度） |
| 26.10 | 各キャラクターに独自の当たり判定領域を設定できる |

### スコープ

**含まれるもの:**
- マルチキャラクター（スコープ）管理
- キャラクター間会話制御
- ウィンドウ位置関係管理
- 複数サーフェス・バルーン管理

**含まれないもの:**
- バルーン実装詳細（wintf-P0-balloon-system の責務）
- ゴースト間通信（areka-P0-mcp-server の責務）
- アニメーション実装（wintf-P0-animation-system の責務）

---

## Requirements

### Requirement 1: マルチキャラクター管理

**Objective:** ゴースト制作者として、複数のキャラクターを管理したい。それにより掛け合いのあるゴーストを作成できる。

#### Acceptance Criteria

1. **The** Character System **shall** 同一ゴースト内で複数キャラクター（スコープ0, 1, 2...）をサポートする
2. **The** Character System **shall** 各キャラクターに独立した識別子（スコープID）を割り当てる
3. **The** Character System **shall** キャラクター数を設定可能にする（デフォルト2、最大4）
4. **The** Character System **shall** 各キャラクターの表示/非表示を個別に制御できる
5. **The** Character System **shall** キャラクターの追加・削除を動的に行える

---

### Requirement 2: 独立リソース管理

**Objective:** ゴースト制作者として、キャラクターごとに異なる見た目を設定したい。それにより個性的なキャラクターを表現できる。

#### Acceptance Criteria

1. **The** Character System **shall** 各キャラクターに独立したサーフェスセットを割り当てられる
2. **The** Character System **shall** 各キャラクターに独立したバルーンを割り当てられる
3. **The** Character System **shall** 各キャラクターに独立した当たり判定領域を設定できる
4. **The** Character System **shall** サーフェス番号の名前空間をキャラクターごとに分離する

---

### Requirement 3: 会話制御

**Objective:** ゴースト制作者として、キャラクター間の会話を制御したい。それにより自然な掛け合いを実現できる。

#### Acceptance Criteria

1. **The** Character System **shall** スコープ指定（\0, \1等相当）でアクティブキャラクターを切り替えられる
2. **The** Character System **shall** 複数キャラクターの同時発言（テキスト表示）をサポートする
3. **The** Character System **shall** 会話のターン制御（交互発言）をサポートする
4. **The** Character System **shall** キャラクターごとのサーフェス同時変更をサポートする
5. **The** Character System **shall** 会話待ちタイミングをキャラクター間で同期できる

---

### Requirement 4: ウィンドウ位置管理

**Objective:** ユーザーとして、キャラクターが自然に並んで表示されてほしい。それにより見た目の一体感が保たれる。

#### Acceptance Criteria

1. **The** Character System **shall** キャラクター間の相対位置を維持する
2. **When** メインキャラクターを移動した時, **the** Character System **shall** 相方キャラクターを追従させるオプションを提供する
3. **The** Character System **shall** 各キャラクターを独立して移動できるオプションを提供する
4. **The** Character System **shall** キャラクターの重なり順（Z順）を制御できる
5. **The** Character System **shall** 画面端での配置調整をサポートする

---

### Requirement 5: イベント配信

**Objective:** ゴースト制作者として、どのキャラクターがクリックされたか知りたい。それにより適切な反応を実装できる。

#### Acceptance Criteria

1. **The** Character System **shall** マウスイベントに対象キャラクター（スコープID）を含める
2. **The** Character System **shall** キャラクター固有の当たり判定ヒット情報を提供する
3. **The** Character System **shall** キャラクター間でのイベント伝搬をサポートする

---

## Non-Functional Requirements

### NFR-1: パフォーマンス

1. キャラクター数増加による描画遅延は最小限であること
2. 4キャラクター同時表示で60fps維持すること

### NFR-2: 互換性

1. 伺かの\0, \1スコープ概念と互換性を持つこと

---

## Dependencies

### 依存する仕様

| 仕様 | 依存内容 |
|------|----------|
| `areka-P0-mcp-server` | イベント配信 |
| `wintf-P0-balloon-system` | バルーン管理 |

### 依存される仕様

| 仕様 | 依存内容 |
|------|----------|
| `areka-P0-reference-ghost` | マルチキャラクター実装例 |

---

## Glossary

| 用語 | 定義 |
|------|------|
| **スコープ** | キャラクターを識別する番号（0=メインキャラ、1=相方等） |
| **\0, \1** | 伺かスクリプトにおけるスコープ切り替えコマンド |
| **掛け合い** | 複数キャラクターが交互に発言する会話形式 |
| **相対位置** | メインキャラクターを基準とした他キャラクターの位置 |
