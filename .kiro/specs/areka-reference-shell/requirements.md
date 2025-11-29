# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | areka-reference-shell 要件定義書 |
| **Version** | 0.1 (Draft) |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Author** | AI-DLC System |

---

## Introduction

本仕様書は「areka」プラットフォームにおける参照シェル（外見パッケージ）の要件を定義する。このシェルはプラットフォームの描画機能検証とシェル制作者向けのサンプルとして機能する。

### 背景

デスクトップマスコットアプリケーションにおける「シェル」とは、キャラクターの外見（サーフェス画像、アニメーション、ヒット領域）を定義するパッケージである。シェルはゴースト（頭脳）とは独立しており、同じゴーストに対して異なるシェルを適用できる。

参照シェルは、以下の目的で制作される：
1. プラットフォーム描画機能の検証（統合テスト）
2. シェル制作者向けのサンプル実装
3. サーフェス/アニメーション定義形式の参照実装

### スコープ

**含まれるもの**:
- 2体キャラクターのサーフェス画像セット
- アニメーション定義（JSON形式）
- ヒット領域定義
- 複数表情（サーフェス）

**含まれないもの**:
- 高度なアニメーション（ボーン、モーフィング等）
- 3Dモデル
- パーティクルエフェクト

### 親仕様からの要件マッピング

本仕様は以下の親要件に対応する：
- **Requirement 2.2**: 複数のサーフェス（表情・ポーズ）を切り替えて表示できる
- **Requirement 2.7**: アニメーション定義を外部ファイル（JSON/YAML等）から読み込める
- **Requirement 8.1**: 1つのゴーストに複数のシェル（外見セット）を持てる
- **Requirement 8.3**: シェルの切り替え完了イベントを発火する
- **Requirement 27.10-27.13**: シェルパッケージ仕様

---

## Requirements

### Requirement 1: シェル構成

**Objective:** シェル制作者として、2体キャラクターの外見セットを定義したい。それによりメインキャラクターと相方の外見を提供できる。

#### Acceptance Criteria

1. **The** Reference Shell **shall** メインキャラクター（\0）用のサーフェスセットを持つ
2. **The** Reference Shell **shall** 相方キャラクター（\1）用のサーフェスセットを持つ
3. **The** Reference Shell **shall** 各キャラクターの基本サイズ（幅×高さ）を定義する
4. **The** Reference Shell **shall** シェル名、作者、説明をメタデータとして持つ
5. **The** Reference Shell **shall** サムネイル画像を含む

---

### Requirement 2: サーフェス画像

**Objective:** シェル制作者として、キャラクターの表情・ポーズ画像を定義したい。それにより多彩な表現が可能になる。

#### Acceptance Criteria

1. **The** Reference Shell **shall** サーフェス番号（surface0, surface1, ...）で画像を管理する
2. **The** Reference Shell **shall** 透過PNG形式の画像をサポートする
3. **The** Reference Shell **shall** 各サーフェスに対応する画像ファイルパスを定義する
4. **The** Reference Shell **shall** 最低限以下のサーフェスを含む：
   - surface0: 通常表情
   - surface1: 笑顔
   - surface2: 驚き
   - surface10: 相方通常
5. **The** Reference Shell **shall** サーフェスのエイリアス（例: "happy" → surface1）を定義できる

---

### Requirement 3: アニメーション定義

**Objective:** シェル制作者として、キャラクターのアニメーションを定義したい。それにより生き生きとしたキャラクター表現が可能になる。

#### Acceptance Criteria

1. **The** Reference Shell **shall** アニメーション定義をJSON形式で記述できる
2. **The** Reference Shell **shall** フレームアニメーション（連番画像）を定義できる
3. **The** Reference Shell **shall** 各フレームの表示時間（ミリ秒）を指定できる
4. **The** Reference Shell **shall** アニメーションのループ設定を指定できる
5. **The** Reference Shell **shall** 最低限以下のアニメーションを含む：
   - idle: アイドル（待機）アニメーション
   - blink: まばたきアニメーション

---

### Requirement 4: ヒット領域定義

**Objective:** シェル制作者として、クリック判定領域を定義したい。それにより部位ごとに異なるインタラクションが可能になる。

#### Acceptance Criteria

1. **The** Reference Shell **shall** サーフェスごとにヒット領域を定義できる
2. **The** Reference Shell **shall** 矩形（rect）によるヒット領域をサポートする
3. **The** Reference Shell **shall** 多角形（polygon）によるヒット領域をサポートする
4. **The** Reference Shell **shall** 各ヒット領域に名前（head, body, hand等）を付けられる
5. **The** Reference Shell **shall** ヒット領域の優先度（重なり時の判定順）を指定できる
6. **The** Reference Shell **shall** 最低限以下のヒット領域を含む：
   - head: 頭部
   - body: 胴体
   - (任意) hand: 手

---

### Requirement 5: サーフェス切り替え

**Objective:** 開発者として、スクリプトからサーフェスを切り替えたい。それによりキャラクターの表情変化を実現できる。

#### Acceptance Criteria

1. **The** Reference Shell **shall** \s[n] コマンドでサーフェス番号を指定して切り替えられる
2. **The** Reference Shell **shall** \s[エイリアス名] でエイリアスを使って切り替えられる
3. **When** サーフェスが切り替わった時, **the** Reference Shell **shall** 新しいサーフェス画像を表示する
4. **When** 存在しないサーフェス番号が指定された時, **the** Reference Shell **shall** デフォルトサーフェス（surface0）を表示する
5. **The** Reference Shell **shall** サーフェス切り替え時のトランジション（フェード等）を定義できる

---

### Requirement 6: パッケージ構造

**Objective:** シェル制作者として、シェルを配布可能なパッケージとして構成したい。それにより他のユーザーがシェルをインストールできる。

#### Acceptance Criteria

1. **The** Reference Shell **shall** manifest.toml を持つ
2. **The** Reference Shell **shall** surfaces/ ディレクトリにサーフェス画像を格納する
3. **The** Reference Shell **shall** surfaces.json（サーフェス定義）を含む
4. **The** Reference Shell **shall** animations.json（アニメーション定義）を含む
5. **The** Reference Shell **shall** collision.json（ヒット領域定義）を含む
6. **The** Reference Shell **shall** README.md（シェル説明）を含む
7. **The** Reference Shell **shall** パッケージ形式（ZIP）で配布できる

---

### Requirement 7: キャラクター配置

**Objective:** シェル制作者として、キャラクターのデフォルト配置を定義したい。それにより2体キャラクターの位置関係を指定できる。

#### Acceptance Criteria

1. **The** Reference Shell **shall** メインキャラクターのデフォルト表示位置を定義できる
2. **The** Reference Shell **shall** 相方キャラクターの相対位置（メインからのオフセット）を定義できる
3. **The** Reference Shell **shall** キャラクター間の間隔を調整できる
4. **The** Reference Shell **shall** 横並び/向かい合いなどの配置パターンを指定できる
5. **The** Reference Shell **shall** 画面端からのマージンを指定できる

---

## Non-Functional Requirements

### NFR-1: 画像品質

- PNG形式、透過対応
- 推奨解像度: 300×400ピクセル程度（キャラクターサイズ）
- ファイルサイズ: 1サーフェスあたり500KB以下推奨

### NFR-2: 互換性

- サーフェス番号は伺か互換（0, 10が基本）
- さくらスクリプト \s[n] コマンド互換

### NFR-3: 可読性

- JSON定義ファイルは人間が読み書きできる形式
- コメント付きサンプル定義を含む

---

## Glossary

| 用語 | 説明 |
|------|------|
| シェル | キャラクターの外見（画像、アニメーション、ヒット領域）パッケージ |
| サーフェス | キャラクターの表情・ポーズを表す画像 |
| ヒット領域 | クリック判定が有効な領域 |
| \0, \1 | メインキャラクター、相方キャラクターを指す |
| \s[n] | サーフェス番号nに切り替えるコマンド |

---

## Appendix

### A. 関連ドキュメント

- 親仕様: `.kiro/specs/ukagaka-desktop-mascot/requirements.md`
- 親仕様設計: `.kiro/specs/ukagaka-desktop-mascot/design.md`

### B. surfaces.json 例

```json
{
  "version": "1.0",
  "characters": {
    "0": {
      "name": "メインキャラクター",
      "size": { "width": 300, "height": 400 },
      "surfaces": {
        "0": { "file": "surfaces/main/surface0.png", "alias": ["normal", "default"] },
        "1": { "file": "surfaces/main/surface1.png", "alias": ["happy", "smile"] },
        "2": { "file": "surfaces/main/surface2.png", "alias": ["surprised"] }
      }
    },
    "1": {
      "name": "相方キャラクター",
      "size": { "width": 250, "height": 350 },
      "surfaces": {
        "10": { "file": "surfaces/sub/surface10.png", "alias": ["normal"] },
        "11": { "file": "surfaces/sub/surface11.png", "alias": ["happy"] }
      }
    }
  }
}
```

### C. animations.json 例

```json
{
  "version": "1.0",
  "animations": {
    "idle": {
      "character": 0,
      "frames": [
        { "surface": 0, "duration": 5000 },
        { "surface": 0, "duration": 100, "overlay": "surfaces/main/blink1.png" },
        { "surface": 0, "duration": 100, "overlay": "surfaces/main/blink2.png" },
        { "surface": 0, "duration": 100, "overlay": "surfaces/main/blink1.png" }
      ],
      "loop": true
    },
    "blink": {
      "character": 0,
      "frames": [
        { "surface": 0, "duration": 50, "overlay": "surfaces/main/blink1.png" },
        { "surface": 0, "duration": 100, "overlay": "surfaces/main/blink2.png" },
        { "surface": 0, "duration": 50, "overlay": "surfaces/main/blink1.png" }
      ],
      "loop": false
    }
  }
}
```

### D. collision.json 例

```json
{
  "version": "1.0",
  "collisions": {
    "0": {
      "surface": 0,
      "regions": [
        { "name": "head", "type": "rect", "rect": { "x": 100, "y": 0, "width": 100, "height": 120 }, "priority": 1 },
        { "name": "body", "type": "rect", "rect": { "x": 80, "y": 120, "width": 140, "height": 200 }, "priority": 2 }
      ]
    }
  }
}
```

### E. パッケージ構造例

```
areka-reference-shell/
├── manifest.toml
├── README.md
├── thumbnail.png
├── surfaces.json
├── animations.json
├── collision.json
└── surfaces/
    ├── main/
    │   ├── surface0.png
    │   ├── surface1.png
    │   ├── surface2.png
    │   ├── blink1.png
    │   └── blink2.png
    └── sub/
        ├── surface10.png
        └── surface11.png
```

---

_Document generated by AI-DLC System on 2025-11-29_
