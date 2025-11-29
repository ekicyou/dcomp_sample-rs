# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | wintf-event-system 要件定義書 |
| **Version** | 0.1 (Draft) |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Author** | AI-DLC System |

---

## Introduction

本仕様書は wintf フレームワークにおけるイベントシステムの要件を定義する。親仕様「伺的デスクトップマスコットアプリ」の実装前提条件（P0）として、ヒットテスト、マウスイベント配信、キャラクターウィンドウのドラッグ移動機能を提供する。

### 背景

wintf フレームワークは現在、イベントシステムの設計のみが存在し、実装が未完了である。デスクトップマスコットアプリケーションでは、キャラクターへのクリック、ドラッグ移動、撫でる操作などのユーザーインタラクションが必須であり、イベントシステムの完全実装が最優先課題となる。

### スコープ

**含まれるもの**:
- ヒットテストシステム（座標からエンティティ特定）
- マウスイベント（クリック、ダブルクリック、ドラッグ、ホバー）
- ウィンドウのドラッグ移動
- イベント配信・バブリング機構

**含まれないもの**:
- キーボードイベント（将来の拡張として検討）
- タッチイベント（将来の拡張として検討）
- ジェスチャー認識

### 親仕様からの要件マッピング

本仕様は以下の親要件に対応する：
- **Requirement 5.1**: キャラクターのどの位置かを識別してイベントを発火する
- **Requirement 5.2**: 領域（頭、胴体、手など）ごとに異なるヒット判定を設定できる
- **Requirement 5.3**: キャラクターをドラッグして移動させる
- **Requirement 5.8**: 触れている状態を検知し続ける（撫でる操作）

---

## Requirements

### Requirement 1: ヒットテストシステム

**Objective:** 開発者として、画面座標からどのウィジェットがクリックされたかを特定したい。それにより適切なイベントハンドラに処理を委譲できる。

#### Acceptance Criteria

1. **The** Event System **shall** 画面座標を受け取り、その位置にあるエンティティを特定できる
2. **When** 複数のエンティティが重なっている場合, **the** Event System **shall** Z順序に従って最前面のエンティティを返す
3. **The** Event System **shall** 透明領域（アルファ=0）をヒット対象外として扱うオプションを提供する
4. **The** Event System **shall** カスタムヒット領域（矩形以外の形状）を定義できる
5. **When** ヒット対象が存在しない場合, **the** Event System **shall** None を返す

---

### Requirement 2: ヒット領域定義

**Objective:** 開発者として、キャラクター画像上の特定領域（頭、胴体、手など）を定義したい。それにより部位ごとに異なる反応を実装できる。

#### Acceptance Criteria

1. **The** Event System **shall** 矩形（Rectangle）によるヒット領域定義をサポートする
2. **The** Event System **shall** 多角形（Polygon）によるヒット領域定義をサポートする
3. **The** Event System **shall** 1つのエンティティに複数の名前付きヒット領域を定義できる
4. **When** ヒットが検出された時, **the** Event System **shall** ヒット領域の名前を含むイベント情報を提供する
5. **The** Event System **shall** ヒット領域定義を外部ファイル（JSON/YAML）から読み込める

---

### Requirement 3: マウスクリックイベント

**Objective:** ユーザーとして、キャラクターをクリックして反応を得たい。それによりキャラクターとのインタラクションが可能になる。

#### Acceptance Criteria

1. **When** ユーザーが左クリックした時, **the** Event System **shall** MouseDown/MouseUp/Click イベントを発火する
2. **When** ユーザーが右クリックした時, **the** Event System **shall** RightClick イベントを発火する
3. **When** ユーザーがダブルクリックした時, **the** Event System **shall** DoubleClick イベントを発火する
4. **The** Event System **shall** クリック位置（画面座標、ローカル座標）をイベント情報に含める
5. **The** Event System **shall** クリックされた領域名をイベント情報に含める

---

### Requirement 4: マウスホバーイベント

**Objective:** 開発者として、マウスカーソルがウィジェット上にあることを検知したい。それによりホバー効果や撫でる操作を実装できる。

#### Acceptance Criteria

1. **When** マウスカーソルがエンティティに入った時, **the** Event System **shall** MouseEnter イベントを発火する
2. **When** マウスカーソルがエンティティから出た時, **the** Event System **shall** MouseLeave イベントを発火する
3. **While** マウスカーソルがエンティティ上にある間, **the** Event System **shall** MouseMove イベントを継続的に発火する
4. **The** Event System **shall** カーソル移動速度をイベント情報に含める（撫でる操作検出用）
5. **When** カーソルが複数のエンティティを跨いで移動した時, **the** Event System **shall** 適切な順序でEnter/Leaveイベントを発火する

---

### Requirement 5: ドラッグイベント

**Objective:** ユーザーとして、キャラクターをドラッグして移動させたい。それによりキャラクターを好きな位置に配置できる。

#### Acceptance Criteria

1. **When** マウスボタンを押したまま移動した時, **the** Event System **shall** DragStart/Drag/DragEnd イベントを発火する
2. **The** Event System **shall** ドラッグ開始位置と現在位置の差分をイベント情報に含める
3. **The** Event System **shall** ドラッグ対象エンティティを識別できる
4. **When** ドラッグが特定の閾値（例: 5ピクセル）を超えた時, **the** Event System **shall** DragStartイベントを発火する
5. **The** Event System **shall** ドラッグキャンセル（Escキー等）をサポートする

---

### Requirement 6: ウィンドウドラッグ移動

**Objective:** ユーザーとして、キャラクターをドラッグしてウィンドウごと移動させたい。それによりデスクトップ上の好きな位置にキャラクターを配置できる。

#### Acceptance Criteria

1. **When** キャラクターがドラッグされた時, **the** Event System **shall** ウィンドウ位置を更新する
2. **The** Event System **shall** ドラッグ中のウィンドウ位置をリアルタイムで更新する
3. **When** マルチモニター環境の場合, **the** Event System **shall** モニター間の移動をサポートする
4. **The** Event System **shall** ドラッグによるウィンドウ移動の有効/無効を切り替えできる
5. **When** ドラッグが終了した時, **the** Event System **shall** 最終位置を保存用に通知する

---

### Requirement 7: イベント配信機構

**Objective:** 開発者として、イベントを適切なハンドラに配信したい。それにより柔軟なイベント処理が可能になる。

#### Acceptance Criteria

1. **The** Event System **shall** イベントをECSリソースとして配信する
2. **The** Event System **shall** イベントのバブリング（親エンティティへの伝播）をサポートする
3. **The** Event System **shall** イベントのキャプチャ（親から子への伝播）をサポートする
4. **When** イベントが消費された時, **the** Event System **shall** 以降の配信を停止する（stopPropagation相当）
5. **The** Event System **shall** イベント履歴を一定期間保持する（デバッグ用）

---

### Requirement 8: ECS統合

**Objective:** 開発者として、イベントシステムをECSアーキテクチャに統合したい。それにより既存のwintfパターンと一貫性を保てる。

#### Acceptance Criteria

1. **The** Event System **shall** ECSシステムとして実装される
2. **The** Event System **shall** Win32メッセージ（WM_MOUSEMOVE等）からECSイベントへの変換を行う
3. **The** Event System **shall** 既存のウィンドウシステム（window.rs）と統合される
4. **The** Event System **shall** 既存のレイアウトシステム（layout/）と統合される
5. **When** エンティティが削除された時, **the** Event System **shall** 関連するイベントリスナーを解除する

---

## Non-Functional Requirements

### NFR-1: パフォーマンス

- ヒットテスト: 1ms以内で完了（100エンティティまで）
- イベント配信: 16ms以内で完了（60fps維持）
- メモリ: イベント履歴は直近1000件まで保持

### NFR-2: レスポンス

- マウス移動からイベント発火まで: 1フレーム以内
- ドラッグによるウィンドウ移動: 遅延なくリアルタイム追従

### NFR-3: 信頼性

- イベントの取りこぼしなし
- 正確なヒット判定（オフバイワンエラーなし）

---

## Glossary

| 用語 | 説明 |
|------|------|
| ヒットテスト | 画面座標からエンティティを特定する処理 |
| バブリング | イベントが子から親へ伝播する仕組み |
| キャプチャ | イベントが親から子へ伝播する仕組み |
| ヒット領域 | クリック判定が有効な領域 |

---

## Appendix

### A. 関連ドキュメント

- 親仕様: `.kiro/specs/ukagaka-desktop-mascot/requirements.md`
- イベントシステム設計: `doc/spec/08-event-system.md`
- ヒットテスト設計: `doc/spec/09-hit-test.md`

### B. Win32メッセージマッピング

| Win32 Message | Event System Event |
|---------------|-------------------|
| WM_LBUTTONDOWN | MouseDown (Left) |
| WM_LBUTTONUP | MouseUp (Left), Click |
| WM_RBUTTONDOWN | MouseDown (Right) |
| WM_RBUTTONUP | MouseUp (Right), RightClick |
| WM_LBUTTONDBLCLK | DoubleClick |
| WM_MOUSEMOVE | MouseMove, Drag |
| WM_MOUSEWHEEL | MouseWheel |

---

_Document generated by AI-DLC System on 2025-11-29_
