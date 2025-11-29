# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | areka-reference-ghost 要件定義書 |
| **Version** | 0.1 (Draft) |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Author** | AI-DLC System |

---

## Introduction

本仕様書は「areka」プラットフォームにおける参照ゴースト（頭脳パッケージ）の要件を定義する。このゴーストはプラットフォームの機能検証とゴースト制作者向けのサンプルとして機能する。

### 背景

デスクトップマスコットアプリケーションにおける「ゴースト」とは、キャラクターの人格・会話・記憶を担う頭脳部分である。プラットフォームはMCPサーバーとして描画・イベント処理を担当し、ゴーストはMCPクライアントとして会話スクリプトの提供やイベントへの応答を担当する。

参照ゴーストは、以下の目的で制作される：
1. プラットフォーム機能の検証（統合テスト）
2. ゴースト制作者向けのサンプル実装
3. 里々インスパイアDSLの参照実装

### スコープ

**含まれるもの**:
- 2体キャラクター構成（メイン＋相方）
- 里々インスパイアDSLによる会話スクリプト
- MCP通信インターフェース実装
- ランダムトーク、イベント応答
- 掛け合い会話（2体間での会話）

**含まれないもの**:
- LLM連携（将来の拡張として検討）
- 音声合成連携
- 複雑な人格シミュレーション
- 記憶・学習機能

### 親仕様からの要件マッピング

本仕様は以下の親要件に対応する：
- **Requirement 4.1**: 里々にインスパイアされた対話記述DSLを解釈・実行できる
- **Requirement 4.2**: ランダムトーク（時間経過で自発的に話す）を実行できる
- **Requirement 4.4**: 変数を保持し、スクリプト内で参照・更新できる
- **Requirement 4.5**: 条件分岐・ループ等の制御構文をサポートする
- **Requirement 4.6**: 複数キャラクター間での会話（掛け合い）をスクリプトで記述できる
- **Requirement 26.1, 26.2, 26.3**: キャラクター間通信（MCPベース）

---

## Requirements

### Requirement 1: ゴースト構成

**Objective:** 開発者として、2体キャラクター構成のゴーストを定義したい。それによりメインキャラクターと相方の掛け合いを実現できる。

#### Acceptance Criteria

1. **The** Reference Ghost **shall** メインキャラクター（\0）を持つ
2. **The** Reference Ghost **shall** 相方キャラクター（\1）を持つ
3. **The** Reference Ghost **shall** 各キャラクターの名前、説明をメタデータとして持つ
4. **The** Reference Ghost **shall** 使用するシェルパッケージを指定できる
5. **The** Reference Ghost **shall** 使用するバルーンパッケージを指定できる

---

### Requirement 2: MCP通信

**Objective:** 開発者として、MCPプロトコルでプラットフォームと通信したい。それによりプラットフォームのイベントを受信し、応答を返すことができる。

#### Acceptance Criteria

1. **The** Reference Ghost **shall** MCPクライアントとしてプラットフォームに接続できる
2. **The** Reference Ghost **shall** プラットフォームからのイベント通知を受信できる
3. **The** Reference Ghost **shall** さくらスクリプト形式の応答を送信できる
4. **When** OnBoot イベントを受信した時, **the** Reference Ghost **shall** 起動トークを返す
5. **When** OnMouseClick イベントを受信した時, **the** Reference Ghost **shall** クリック応答を返す
6. **The** Reference Ghost **shall** MCPのTools/Resources仕様に準拠する

---

### Requirement 3: 会話スクリプト（里々インスパイアDSL）

**Objective:** ゴースト制作者として、自然な構文で会話を記述したい。それにより直感的にキャラクターの人格を表現できる。

#### Acceptance Criteria

1. **The** Reference Ghost **shall** テキストファイル形式でスクリプトを記述できる
2. **The** Reference Ghost **shall** ランダム選択構文（選択肢の中から1つを選ぶ）をサポートする
3. **The** Reference Ghost **shall** 条件分岐構文（if/else相当）をサポートする
4. **The** Reference Ghost **shall** 変数参照構文（${変数名}）をサポートする
5. **The** Reference Ghost **shall** コメント構文（#または//）をサポートする

---

### Requirement 4: ランダムトーク

**Objective:** ユーザーとして、キャラクターが自発的に話しかけてほしい。それによりキャラクターに自律性を感じられる。

#### Acceptance Criteria

1. **When** OnRandomTalk イベントを受信した時, **the** Reference Ghost **shall** ランダムトークを返す
2. **The** Reference Ghost **shall** 複数のランダムトークセットを持つ
3. **The** Reference Ghost **shall** 時間帯に応じたランダムトークを選択できる（朝/昼/夜）
4. **The** Reference Ghost **shall** トーク履歴を考慮して同じトークの連続を避ける
5. **The** Reference Ghost **shall** ランダムトークの出現確率を調整できる

---

### Requirement 5: 掛け合い会話

**Objective:** ユーザーとして、2体のキャラクターが会話する様子を見たい。それにより漫才のような楽しい掛け合いを楽しめる。

#### Acceptance Criteria

1. **The** Reference Ghost **shall** \0（メイン）と \1（相方）の発言を交互に指定できる
2. **The** Reference Ghost **shall** 1つのトーク内で複数回の発言者切り替えができる
3. **The** Reference Ghost **shall** 同時発言（両者が同時にしゃべる）を指定できる
4. **The** Reference Ghost **shall** 相方の割り込み発言を表現できる
5. **When** 掛け合い会話が記述された時, **the** Reference Ghost **shall** 適切なさくらスクリプト（\0, \1, \h, \u）を生成する

---

### Requirement 6: イベント応答

**Objective:** ゴースト制作者として、様々なイベントに対する応答を定義したい。それによりインタラクティブなキャラクターを作成できる。

#### Acceptance Criteria

1. **The** Reference Ghost **shall** OnBoot（起動時）イベントに応答できる
2. **The** Reference Ghost **shall** OnClose（終了時）イベントに応答できる
3. **The** Reference Ghost **shall** OnMouseClick（クリック時）イベントに応答できる
4. **The** Reference Ghost **shall** OnMouseDoubleClick（ダブルクリック時）イベントに応答できる
5. **The** Reference Ghost **shall** OnFirstBoot（初回起動時）イベントに応答できる
6. **The** Reference Ghost **shall** イベントに付随する情報（クリック位置、領域名等）を参照できる

---

### Requirement 7: 変数管理

**Objective:** ゴースト制作者として、ゴーストの状態を変数で管理したい。それにより会話の文脈や累積情報を保持できる。

#### Acceptance Criteria

1. **The** Reference Ghost **shall** 変数を宣言・初期化できる
2. **The** Reference Ghost **shall** 変数を更新できる
3. **The** Reference Ghost **shall** 変数を会話スクリプト内で参照できる
4. **The** Reference Ghost **shall** 変数をファイルに永続化できる
5. **When** ゴーストが再起動された時, **the** Reference Ghost **shall** 永続化された変数を復元する

---

### Requirement 8: パッケージ構造

**Objective:** 開発者として、ゴーストを配布可能なパッケージとして構成したい。それにより他のユーザーがゴーストをインストールできる。

#### Acceptance Criteria

1. **The** Reference Ghost **shall** manifest.toml を持つ
2. **The** Reference Ghost **shall** スクリプトファイル（*.txt または *.satori）を含む
3. **The** Reference Ghost **shall** README.md（ゴースト説明）を含む
4. **The** Reference Ghost **shall** プレビュー画像（thumbnail.png）を含む
5. **The** Reference Ghost **shall** パッケージ形式（ZIP）で配布できる

---

## Non-Functional Requirements

### NFR-1: 可読性

- スクリプトは非プログラマでも読み書きできる構文
- 日本語コメントによるドキュメント付き
- サンプルトーク付き

### NFR-2: 応答速度

- イベント受信から応答送信まで: 100ms以内
- スクリプト解析: 1000行を100ms以内

### NFR-3: 互換性

- さくらスクリプト出力の基本コマンド互換
- MCPプロトコル準拠

---

## Glossary

| 用語 | 説明 |
|------|------|
| ゴースト | キャラクターの人格・会話を担う頭脳パッケージ |
| 里々 | 伺かで使用される対話記述言語 |
| さくらスクリプト | 伺かで使用される表示制御スクリプト |
| \0, \1 | メインキャラクター、相方キャラクターを指す |
| MCP | Model Context Protocol |
| ランダムトーク | 時間経過で自発的に発生する会話 |

---

## Appendix

### A. 関連ドキュメント

- 親仕様: `.kiro/specs/ukagaka-desktop-mascot/requirements.md`
- 親仕様設計: `.kiro/specs/ukagaka-desktop-mascot/design.md`

### B. スクリプト例（里々インスパイア）

```
# ランダムトーク
*ランダムトーク
:今日もいい天気ですね。
:少し眠いです。\_w[3]ふわぁ。
:${username}さん、何か面白いことありました？

# 掛け合いトーク
*掛け合い:おやつ
\0:おなかすいたー。
\1:さっき食べたばかりでしょ。
\0:でも、おやつは別腹なの！
\1:はいはい…

# 起動トーク
*OnBoot
:おはようございます、${username}さん。
:今日も一日頑張りましょう。
```

### C. パッケージ構造例

```
areka-reference-ghost/
├── manifest.toml
├── README.md
├── thumbnail.png
├── scripts/
│   ├── boot.txt
│   ├── random.txt
│   ├── menu.txt
│   └── events.txt
└── save/
    └── variables.json
```

---

_Document generated by AI-DLC System on 2025-11-29_
