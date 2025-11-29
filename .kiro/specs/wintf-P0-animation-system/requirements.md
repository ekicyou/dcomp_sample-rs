# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | wintf アニメーションシステム 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Priority** | P0 (MVP必須) |

---

## Introduction

本仕様書は wintf フレームワークにおけるアニメーションシステムの要件を定義する。キャラクターが「生き生きとした動き」を持ち、「生きている」存在として感じられることを目的とする。

### 親仕様からのトレーサビリティ

本仕様は `ukagaka-desktop-mascot` の以下の要件をカバーする：

| 親要件ID | 内容 |
|----------|------|
| 2.1 | キャラクターのアイドル（待機）アニメーションを再生できる |
| 2.3 | サーフェス切り替え命令を受けた時、滑らかなトランジション効果で切り替える |
| 2.5 | アニメーション再生中、60fps以上の滑らかな描画を維持する |
| 2.6 | 拡張アニメーションプラグインが有効な場合、呼吸のような微細な動きをアイドル時に付与する |
| 2.8 | 複数キャラクター間で連動したアニメーション（掛け合い、同期動作等）を再生できる |

### スコープ

**含まれるもの:**
- フレームアニメーション（サーフェス切り替え）の定義と再生
- サーフェス間トランジション効果
- アイドルアニメーション自動再生
- 連動アニメーション（複数キャラクター同期）
- Windows Animation API 統合

**含まれないもの:**
- 画像の読み込み・描画（wintf-P0-image-widget の責務）
- キャラクター外見定義（areka-P0-reference-shell の責務）
- アニメーション定義ファイルフォーマット（シェル仕様の責務）

---

## Requirements

### Requirement 1: フレームアニメーション再生

**Objective:** 開発者として、連番画像によるフレームアニメーションを定義・再生したい。それによりキャラクターに動きを与えられる。

#### Acceptance Criteria

1. **The** Animation System **shall** 複数のサーフェス画像を順番に切り替えて再生できる（フレームアニメーション）
2. **The** Animation System **shall** 各フレームの表示時間（デュレーション）を個別に指定できる
3. **The** Animation System **shall** アニメーションのループ再生（無限/指定回数）をサポートする
4. **When** アニメーション再生命令を受けた時, **the** Animation System **shall** 指定されたアニメーションを開始する
5. **When** アニメーションが終了した時, **the** Animation System **shall** 完了イベントを発火する
6. **The** Animation System **shall** アニメーションの一時停止・再開・停止をサポートする
7. **While** アニメーション再生中, **the** Animation System **shall** 60fps以上の滑らかな描画を維持する

---

### Requirement 2: サーフェストランジション

**Objective:** 開発者として、サーフェス切り替え時に滑らかなトランジション効果を適用したい。それにより唐突な切り替えを避け、自然な見た目を実現できる。

#### Acceptance Criteria

1. **When** サーフェス切り替え命令を受けた時, **the** Animation System **shall** トランジション効果を適用して切り替える
2. **The** Animation System **shall** クロスフェード（フェードイン/フェードアウト）トランジションをサポートする
3. **The** Animation System **shall** 即時切り替え（トランジションなし）をサポートする
4. **The** Animation System **shall** トランジション時間を指定できる
5. **When** トランジション中に新しい切り替え命令を受けた時, **the** Animation System **shall** 現在のトランジションを中断して新しいトランジションを開始する

---

### Requirement 3: アイドルアニメーション

**Objective:** 開発者として、待機中に自動再生されるアイドルアニメーションを定義したい。それによりキャラクターが静止画ではなく「生きている」印象を与えられる。

#### Acceptance Criteria

1. **The** Animation System **shall** アイドル（待機）アニメーションを定義できる
2. **When** キャラクターがアイドル状態になった時, **the** Animation System **shall** アイドルアニメーションを自動的に開始する
3. **The** Animation System **shall** 複数のアイドルアニメーションからランダムに選択できる
4. **The** Animation System **shall** アイドルアニメーション間隔（次のアニメーション開始までの待機時間）を設定できる
5. **Where** 微細動作モードが有効な場合, **the** Animation System **shall** 呼吸のような微細な動き（スケール/位置の微小変化）をアイドル時に付与する

---

### Requirement 4: 連動アニメーション

**Objective:** 開発者として、複数キャラクター間で同期したアニメーションを再生したい。それにより掛け合いや同期動作を表現できる。

#### Acceptance Criteria

1. **The** Animation System **shall** 複数のアニメーションを同期して開始できる
2. **The** Animation System **shall** アニメーショングループ（連動セット）を定義できる
3. **When** グループアニメーション開始命令を受けた時, **the** Animation System **shall** グループ内の全アニメーションを同時に開始する
4. **The** Animation System **shall** キャラクター間でアニメーション完了を待ち合わせできる

---

### Requirement 5: Windows Animation API 統合

**Objective:** 開発者として、Windows Animation API を活用したプロパティアニメーションを使用したい。それによりGPUアクセラレーションを活用した滑らかなアニメーションを実現できる。

#### Acceptance Criteria

1. **The** Animation System **shall** Windows Animation Manager との統合をサポートする
2. **The** Animation System **shall** DirectComposition のプロパティ（位置、不透明度、スケール）をアニメーションできる
3. **The** Animation System **shall** イージング関数（ease-in, ease-out, ease-in-out, linear）をサポートする
4. **The** Animation System **shall** 複数のプロパティアニメーションを同時に実行できる
5. **When** アニメーションが完了した時, **the** Animation System **shall** 完了コールバックを発火する

---

## Non-Functional Requirements

### NFR-1: パフォーマンス

1. アニメーション再生中、60fps以上を維持すること
2. アイドル時のCPU使用率は1%未満であること
3. Windows Animation API を使用する場合、GPU アクセラレーションを活用すること

### NFR-2: 互換性

1. Windows 10 (1803) 以降をサポートすること
2. DirectComposition 対応環境を前提とすること

### NFR-3: 拡張性

1. 新しいトランジション効果を追加可能な設計とすること
2. カスタムイージング関数を追加可能な設計とすること

---

## Dependencies

### 依存する仕様

| 仕様 | 依存内容 |
|------|----------|
| `wintf-P0-image-widget` | サーフェス画像の読み込み・描画 |

### 依存される仕様

| 仕様 | 依存内容 |
|------|----------|
| `areka-P0-reference-shell` | アニメーション定義の使用 |

---

## Glossary

| 用語 | 定義 |
|------|------|
| **サーフェス** | キャラクターの1枚の表示画像（表情・ポーズ） |
| **フレームアニメーション** | 複数のサーフェスを順番に切り替えて表現するアニメーション |
| **トランジション** | サーフェス切り替え時の視覚効果（フェード等） |
| **アイドルアニメーション** | 待機中に自動再生されるアニメーション |
| **連動アニメーション** | 複数キャラクター間で同期して再生されるアニメーション |
| **Windows Animation API** | Windowsのアニメーション管理API（UIAnimationManager） |
