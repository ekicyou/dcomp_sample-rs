# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | wintf DPIスケーリング 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Priority** | P1 (リリース品質) |

---

## Introduction

本仕様書は wintf フレームワークにおける高DPI環境およびPer-Monitor DPI対応の要件を定義する。様々な環境でアプリケーションが正しく表示されることを目的とする。

### 親仕様からのトレーサビリティ

本仕様は `ukagaka-desktop-mascot` の以下の要件をカバーする：

| 親要件ID | 内容 |
|----------|------|
| 15.1 | 高DPI環境で適切にスケーリングされる |
| 15.2 | システムのDPI設定が変更された時、表示サイズを即座に更新する |
| NFR-1 | Windows 10以降をサポートする |

### スコープ

**含まれるもの:**
- 高DPI環境での正しいスケーリング
- Per-Monitor DPI対応（マルチモニター環境）
- DPI変更時の動的更新
- DPI対応API（GetDpiForWindow等）の統合

**含まれないもの:**
- コンテンツ（ゴースト/シェル/バルーン）のスケーリング制御（作者責務）
- レガシーDPI対応（System DPI Aware以前）

### 現状

steering/tech.md によると、wintf は既にマルチモニタDPI対応が「ほぼ完成」している。本仕様はその検証と残りの対応を行う。

---

## Requirements

### Requirement 1: 高DPI環境でのスケーリング

**Objective:** 開発者として、高DPI環境でUIが正しくスケーリングされるようにしたい。それによりHiDPIディスプレイで鮮明な表示を実現できる。

#### Acceptance Criteria

1. **The** DPI System **shall** Per-Monitor DPI Aware v2 を宣言する
2. **The** DPI System **shall** ウィンドウのDPIを正しく取得する（GetDpiForWindow）
3. **The** DPI System **shall** DPIスケールファクター（100%, 125%, 150%, 175%, 200%等）を正しく計算する
4. **The** DPI System **shall** 論理座標から物理座標への変換を提供する
5. **The** DPI System **shall** 物理座標から論理座標への変換を提供する
6. **While** DPIスケーリング中, **the** DPI System **shall** サブピクセル精度でレンダリングを維持する

---

### Requirement 2: Per-Monitor DPI対応

**Objective:** 開発者として、異なるDPIを持つ複数のモニターでアプリケーションを使用できるようにしたい。それにより4KモニターとフルHDモニターの混在環境でも正しく表示できる。

#### Acceptance Criteria

1. **The** DPI System **shall** 各モニターのDPIを個別に取得する
2. **When** ウィンドウが異なるDPIのモニターに移動した時, **the** DPI System **shall** そのモニターのDPIに合わせてスケーリングを更新する
3. **When** ウィンドウが複数モニターにまたがっている時, **the** DPI System **shall** 主要モニター（最大面積）のDPIを使用する
4. **The** DPI System **shall** WM_DPICHANGEDメッセージを処理する

---

### Requirement 3: DPI変更時の動的更新

**Objective:** 開発者として、システムDPI設定の変更に即座に対応したい。それによりログアウトなしでDPI変更を反映できる。

#### Acceptance Criteria

1. **When** システムのDPI設定が変更された時, **the** DPI System **shall** 表示サイズを即座に更新する
2. **When** DPIが変更された時, **the** DPI System **shall** ウィンドウサイズを適切にリサイズする
3. **When** DPIが変更された時, **the** DPI System **shall** DirectCompositionリソースを再作成または更新する
4. **When** DPIが変更された時, **the** DPI System **shall** DPI変更イベントを発火する
5. **The** DPI System **shall** DPI変更時にちらつきなく滑らかに更新する

---

### Requirement 4: DPI情報のECS統合

**Objective:** 開発者として、ECSシステムからDPI情報にアクセスしたい。それによりECSコンポーネントがDPIを考慮したレイアウトを行える。

#### Acceptance Criteria

1. **The** DPI System **shall** ウィンドウごとのDPI情報をECSコンポーネントとして提供する
2. **The** DPI System **shall** DPI変更をECSイベントとして通知する
3. **The** DPI System **shall** DPIスケールファクターをレイアウトシステム（taffy）に伝達する

---

## Non-Functional Requirements

### NFR-1: パフォーマンス

1. DPI取得は1ms以内で完了すること
2. DPI変更時の更新は16ms（60fps相当）以内で完了すること
3. DPI変更時のリソース再作成は最小限に抑えること

### NFR-2: 互換性

1. Windows 10 (1803) 以降をサポートすること（Per-Monitor DPI Aware v2）
2. Windows 11でも正常に動作すること

---

## Dependencies

### 依存する仕様

| 仕様 | 依存内容 |
|------|----------|
| `wintf-P0-image-widget` | 画像のDPIスケーリング |
| `wintf-P0-event-system` | DPI変更イベント配信 |
| `wintf-P0-typewriter` | テキストのDPIスケーリング |

### 依存される仕様

| 仕様 | 依存内容 |
|------|----------|
| `areka-P0-window-placement` | 座標のDPI変換 |
| `wintf-P0-balloon-system` | バルーンのDPIスケーリング |

---

## Glossary

| 用語 | 定義 |
|------|------|
| **DPI** | Dots Per Inch。ディスプレイの解像度密度 |
| **高DPI** | 100%（96 DPI）を超えるDPI設定 |
| **Per-Monitor DPI** | モニターごとに異なるDPIをサポートする機能 |
| **DPIスケールファクター** | 基準DPI（96）に対する倍率（例: 150% = 1.5） |
| **論理座標** | DPI非依存の座標系 |
| **物理座標** | DPIスケーリング後の実際のピクセル座標 |
