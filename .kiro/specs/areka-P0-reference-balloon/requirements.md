# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | areka-reference-balloon 要件定義書 |
| **Version** | 0.1 (Draft) |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Author** | AI-DLC System |

---

## Introduction

本仕様書は「areka」プラットフォームにおける参照バルーン（会話UI パッケージ）の要件を定義する。このバルーンはプラットフォームのテキスト表示機能検証とバルーン制作者向けのサンプルとして機能する。

### 背景

デスクトップマスコットアプリケーションにおける「バルーン」とは、キャラクターの発言を表示する吹き出しUIのスタイル定義パッケージである。バルーンはゴースト（頭脳）やシェル（外見）とは独立しており、ユーザーが好みのバルーンを選択できる。

参照バルーンは、以下の目的で制作される：
1. プラットフォーム テキスト表示機能の検証（統合テスト）
2. バルーン制作者向けのサンプル実装
3. スタイル定義形式の参照実装

### スコープ

**含まれるもの**:
- バルーン（吹き出し）外観定義
- フォント、色、背景のスタイル設定
- 縦書き/横書き対応
- 2体キャラクター対応（メイン/相方用バルーン）

**含まれないもの**:
- 選択肢UI（将来の拡張として検討）
- 入力ボックスUI
- アニメーション効果

### 親仕様からの要件マッピング

本仕様は以下の親要件に対応する：
- **Requirement 3.4**: 縦書きテキストをサポートする
- **Requirement 3.6**: 吹き出しのスキン（外観）をカスタマイズできる
- **Requirement 27.15, 27.16**: バルーンパッケージ仕様

---

## Requirements

### Requirement 1: バルーン構成

**Objective:** バルーン制作者として、2体キャラクター用のバルーンセットを定義したい。それによりメインキャラクターと相方それぞれの発言を区別して表示できる。

#### Acceptance Criteria

1. **The** Reference Balloon **shall** メインキャラクター（\0）用のバルーンスタイルを持つ
2. **The** Reference Balloon **shall** 相方キャラクター（\1）用のバルーンスタイルを持つ
3. **The** Reference Balloon **shall** バルーン名、作者、説明をメタデータとして持つ
4. **The** Reference Balloon **shall** サムネイル画像を含む
5. **The** Reference Balloon **shall** バルーンの基本サイズ（幅×高さ）を定義する

---

### Requirement 2: バルーン外観

**Objective:** バルーン制作者として、吹き出しの外観を定義したい。それにより個性的なバルーンデザインが可能になる。

#### Acceptance Criteria

1. **The** Reference Balloon **shall** 背景色を指定できる
2. **The** Reference Balloon **shall** 背景画像（9-patch形式）を指定できる
3. **The** Reference Balloon **shall** 枠線の色と太さを指定できる
4. **The** Reference Balloon **shall** 角丸半径を指定できる
5. **The** Reference Balloon **shall** パディング（内側余白）を指定できる
6. **The** Reference Balloon **shall** しっぽ（吹き出しの三角部分）の位置と形状を定義できる

---

### Requirement 3: テキストスタイル

**Objective:** バルーン制作者として、テキストの表示スタイルを定義したい。それによりキャラクターの雰囲気に合った文字表現が可能になる。

#### Acceptance Criteria

1. **The** Reference Balloon **shall** フォントファミリーを指定できる
2. **The** Reference Balloon **shall** フォントサイズを指定できる
3. **The** Reference Balloon **shall** テキスト色を指定できる
4. **The** Reference Balloon **shall** 行間（line-height）を指定できる
5. **The** Reference Balloon **shall** 文字間隔（letter-spacing）を指定できる
6. **When** ルビ（ふりがな）が指定された時, **the** Reference Balloon **shall** ルビ用のフォントサイズを適用する

---

### Requirement 4: 縦書き/横書き対応

**Objective:** バルーン制作者として、縦書きと横書きの両方に対応したい。それにより日本語表現の幅が広がる。

#### Acceptance Criteria

1. **The** Reference Balloon **shall** 横書きモードをサポートする
2. **The** Reference Balloon **shall** 縦書きモードをサポートする
3. **The** Reference Balloon **shall** デフォルトの書字方向を指定できる
4. **When** 縦書きモードの場合, **the** Reference Balloon **shall** テキストを右から左に配置する
5. **The** Reference Balloon **shall** 書字方向に応じてしっぽの位置を調整できる

---

### Requirement 5: バルーン配置

**Objective:** バルーン制作者として、バルーンのキャラクターに対する配置を定義したい。それによりキャラクターとバルーンの位置関係が自然になる。

#### Acceptance Criteria

1. **The** Reference Balloon **shall** キャラクターに対する相対位置（上/下/左/右）を指定できる
2. **The** Reference Balloon **shall** キャラクターからのオフセット距離を指定できる
3. **When** バルーンが画面端にはみ出す場合, **the** Reference Balloon **shall** 自動的に位置を調整する
4. **The** Reference Balloon **shall** しっぽがキャラクターを指すように調整される
5. **The** Reference Balloon **shall** メインキャラクターと相方キャラクターで異なる配置を指定できる

---

### Requirement 6: リンク/アンカースタイル

**Objective:** バルーン制作者として、クリック可能なリンクのスタイルを定義したい。それによりインタラクティブなテキスト要素を視覚的に区別できる。

#### Acceptance Criteria

1. **The** Reference Balloon **shall** リンクテキストの色を指定できる
2. **The** Reference Balloon **shall** リンクテキストの下線スタイルを指定できる
3. **When** マウスホバー時, **the** Reference Balloon **shall** ホバースタイル（色変更等）を適用できる
4. **The** Reference Balloon **shall** 訪問済みリンクのスタイルを指定できる
5. **The** Reference Balloon **shall** クリック領域の拡張（パディング）を指定できる

---

### Requirement 7: パッケージ構造

**Objective:** バルーン制作者として、バルーンを配布可能なパッケージとして構成したい。それにより他のユーザーがバルーンをインストールできる。

#### Acceptance Criteria

1. **The** Reference Balloon **shall** manifest.toml を持つ
2. **The** Reference Balloon **shall** balloon.json（スタイル定義）を含む
3. **The** Reference Balloon **shall** images/ ディレクトリに背景画像を格納する（使用する場合）
4. **The** Reference Balloon **shall** README.md（バルーン説明）を含む
5. **The** Reference Balloon **shall** thumbnail.png（プレビュー画像）を含む
6. **The** Reference Balloon **shall** パッケージ形式（ZIP）で配布できる

---

## Non-Functional Requirements

### NFR-1: 可読性

- JSON定義ファイルは人間が読み書きできる形式
- コメント付きサンプル定義を含む
- 日本語での説明ドキュメント

### NFR-2: フォント互換性

- システムフォントをデフォルトで使用
- 日本語フォント（ゴシック/明朝）に対応
- フォント未インストール時のフォールバック対応

### NFR-3: 表示品質

- 高DPI環境での鮮明な表示
- アンチエイリアスによる滑らかなテキスト描画

---

## Glossary

| 用語 | 説明 |
|------|------|
| バルーン | キャラクターの発言を表示する吹き出しUI |
| スキン | バルーンの外観定義 |
| しっぽ | 吹き出しの三角形部分（キャラクターを指す） |
| 9-patch | 伸縮可能な背景画像形式 |
| 縦書き | テキストを上から下、列を右から左に配置する書字方向 |

---

## Appendix

### A. 関連ドキュメント

- 親仕様: `.kiro/specs/ukagaka-desktop-mascot/requirements.md`
- 親仕様設計: `.kiro/specs/ukagaka-desktop-mascot/design.md`

### B. balloon.json 例

```json
{
  "version": "1.0",
  "name": "Reference Balloon",
  "author": "areka-project",
  "description": "参照バルーン - シンプルな白背景",
  
  "defaults": {
    "direction": "horizontal",
    "font": {
      "family": "Yu Gothic UI",
      "fallback": ["Meiryo", "MS Gothic"],
      "size": 14,
      "color": "#333333",
      "lineHeight": 1.6,
      "letterSpacing": 0.5
    }
  },
  
  "balloons": {
    "0": {
      "comment": "メインキャラクター用バルーン",
      "size": { "width": 300, "minHeight": 80, "maxHeight": 400 },
      "background": {
        "color": "#FFFFFF",
        "borderColor": "#CCCCCC",
        "borderWidth": 1,
        "borderRadius": 8
      },
      "padding": { "top": 12, "right": 16, "bottom": 12, "left": 16 },
      "tail": {
        "position": "bottom-left",
        "size": { "width": 16, "height": 12 },
        "offset": 30
      },
      "placement": {
        "anchor": "top-left",
        "offsetX": 0,
        "offsetY": -10
      }
    },
    "1": {
      "comment": "相方キャラクター用バルーン",
      "size": { "width": 280, "minHeight": 60, "maxHeight": 350 },
      "background": {
        "color": "#F8F8FF",
        "borderColor": "#AAAACC",
        "borderWidth": 1,
        "borderRadius": 8
      },
      "padding": { "top": 10, "right": 14, "bottom": 10, "left": 14 },
      "tail": {
        "position": "bottom-right",
        "size": { "width": 14, "height": 10 },
        "offset": 30
      },
      "placement": {
        "anchor": "top-right",
        "offsetX": 0,
        "offsetY": -10
      }
    }
  },
  
  "verticalMode": {
    "0": {
      "size": { "width": 150, "minHeight": 200, "maxHeight": 500 },
      "tail": { "position": "right-bottom" }
    }
  },
  
  "link": {
    "color": "#0066CC",
    "underline": true,
    "hoverColor": "#0088FF",
    "visitedColor": "#660099"
  },
  
  "ruby": {
    "fontSize": 8,
    "offset": 2
  }
}
```

### C. パッケージ構造例

```
areka-reference-balloon/
├── manifest.toml
├── README.md
├── thumbnail.png
├── balloon.json
└── images/
    ├── background_main.png (optional)
    └── background_sub.png (optional)
```

### D. manifest.toml 例

```toml
[package]
type = "balloon"
name = "Reference Balloon"
id = "areka-reference-balloon"
version = "1.0.0"
author = "areka-project"
description = "参照バルーン - シンプルな白背景のバルーンセット"

[balloon]
definition = "balloon.json"
thumbnail = "thumbnail.png"

[features]
vertical = true
horizontal = true

[compatibility]
platform_version = ">=1.0.0"
```

---

_Document generated by AI-DLC System on 2025-11-29_
