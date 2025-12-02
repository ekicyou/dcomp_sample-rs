# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | event-mouse-basic 要件定義書 |
| **Version** | 1.0 (Draft) |
| **Date** | 2025-12-02 |
| **Parent Spec** | wintf-P0-event-system |
| **Author** | AI-DLC System |

---

## Introduction

本仕様書は wintf フレームワークにおけるマウス基本イベント処理の要件を定義する。親仕様「wintf-P0-event-system」の Requirement 3（マウスクリックイベント）と Requirement 4（マウスホバーイベント）を実装する。

### 背景

デスクトップマスコットアプリケーションでは、キャラクターへのクリック、ホバー、撫でる操作などのユーザーインタラクションが必須である。`event-hit-test` 仕様で実装されたヒットテストAPIを活用し、Win32メッセージからECSイベントへの変換を行う。

### スコープ

**含まれるもの**:
- マウスクリックイベント（左クリック、右クリック、ダブルクリック）
- マウスホバーイベント（Enter/Leave/Move）
- Win32メッセージハンドラとヒットテストAPIの統合
- ローカル座標変換（`hit_test_detailed` API）
- カーソル移動速度の計算（撫でる操作検出用）

**含まれないもの**:
- ドラッグイベント → `event-drag-system` 仕様で対応
- 名前付きヒット領域 → `event-hit-test-named-regions` 仕様で対応
- イベントバブリング・キャプチャ → `event-dispatch` 仕様で対応

### event-hit-test からの引き継ぎ事項

`event-hit-test` 仕様で実装されたAPIを統合し、以下の項目を本仕様で実装する：

| 引き継ぎ項目 | 説明 | 対応Requirement |
|-------------|------|-----------------|
| **ecs_wndproc 統合** | `WM_MOUSEMOVE`, `WM_LBUTTONDOWN` 等のハンドラから `hit_test` API を呼び出す | Req 6 |
| **ローカル座標変換** | `GlobalArrangement.bounds` を使用したエンティティローカル座標への変換 | Req 4 |
| **hit_test_detailed** | ローカル座標付きヒット結果を返す関数 | Req 4 |
| **キャッシュ機構** | 座標が同一の場合は前回結果を返す | Req 7 (オプション) |

---

## Requirements

### Requirement 1: マウスクリックイベント

**Objective:** ユーザーとして、キャラクターをクリックして反応を得たい。それによりキャラクターとのインタラクションが可能になる。

#### Acceptance Criteria

1. When ユーザーが左マウスボタンを押した時, the Mouse Event System shall `MouseDown { button: Left }` イベントを発火する
2. When ユーザーが左マウスボタンを離した時, the Mouse Event System shall `MouseUp { button: Left }` イベントを発火する
3. When ユーザーが左クリックを完了した時（Down→Up が同一エンティティ上）, the Mouse Event System shall `Click` イベントを発火する
4. When ユーザーが右クリックした時, the Mouse Event System shall `RightClick` イベントを発火する
5. When ユーザーがダブルクリックした時, the Mouse Event System shall `DoubleClick` イベントを発火する
6. The Mouse Event System shall クリックイベントにターゲットエンティティを含める
7. The Mouse Event System shall クリックイベントにスクリーン座標（物理ピクセル）を含める

#### 設計決定

- **Click判定**: `MouseDown` と `MouseUp` が同一エンティティ上で発生した場合に `Click` を発火
- **ダブルクリック検出**: Win32 の `WM_LBUTTONDBLCLK` メッセージを使用

---

### Requirement 2: マウスホバーイベント

**Objective:** 開発者として、マウスカーソルがウィジェット上にあることを検知したい。それによりホバー効果や撫でる操作を実装できる。

#### Acceptance Criteria

1. When マウスカーソルがエンティティに入った時, the Mouse Event System shall `MouseEnter` イベントを発火する
2. When マウスカーソルがエンティティから出た時, the Mouse Event System shall `MouseLeave` イベントを発火する
3. While マウスカーソルがエンティティ上にある間, the Mouse Event System shall `MouseMove` イベントを継続的に発火する
4. When カーソルが複数のエンティティを跨いで移動した時, the Mouse Event System shall 適切な順序で Enter/Leave イベントを発火する
5. The Mouse Event System shall 現在ホバー中のエンティティを `HoveredEntity` リソースとして保持する

#### イベント発火順序

カーソルが Entity A から Entity B に移動した場合：
1. `MouseLeave { entity: A }`
2. `MouseEnter { entity: B }`
3. `MouseMove { entity: B, ... }`

---

### Requirement 3: カーソル移動速度

**Objective:** 開発者として、カーソルの移動速度を取得したい。それにより「撫でる」操作の検出が可能になる。

#### Acceptance Criteria

1. The Mouse Event System shall `MouseMove` イベントにカーソル移動速度（ピクセル/秒）を含める
2. The Mouse Event System shall 直前の位置と現在位置、および経過時間から速度を計算する
3. The Mouse Event System shall 速度計算に使用する履歴を最大5サンプル保持する
4. When 最初のマウス移動の場合, the Mouse Event System shall 速度を 0.0 として報告する

#### 速度計算

```rust
/// カーソル移動速度（ピクセル/秒）
pub struct CursorVelocity {
    pub x: f32,  // 水平方向速度
    pub y: f32,  // 垂直方向速度
    pub magnitude: f32,  // 速度の大きさ
}
```

---

### Requirement 4: ローカル座標変換

**Objective:** 開発者として、スクリーン座標をエンティティローカル座標に変換したい。それによりエンティティ内の正確な位置を特定できる。

#### Acceptance Criteria

1. The Mouse Event System shall スクリーン座標からエンティティローカル座標への変換を提供する
2. The Mouse Event System shall `GlobalArrangement.bounds` の left/top を使用してローカル座標を計算する
3. The Mouse Event System shall `hit_test_detailed` 関数でエンティティとローカル座標を同時に返す
4. The Mouse Event System shall すべてのマウスイベントにローカル座標を含める

#### API設計

```rust
/// ヒットテスト結果（詳細情報付き）
pub struct HitTestResult {
    pub entity: Entity,
    pub local_point: PhysicalPoint,  // エンティティローカル座標（物理ピクセル）
}

/// 詳細ヒットテスト
pub fn hit_test_detailed(
    world: &World, 
    root: Entity, 
    screen_point: PhysicalPoint
) -> Option<HitTestResult>;
```

#### 座標変換計算

```
local_x = screen_x - GlobalArrangement.bounds.left
local_y = screen_y - GlobalArrangement.bounds.top
```

**Note**: 現在の `GlobalArrangement` は軸平行変換のみをサポートするため、回転・スキュー変換は不要。

---

### Requirement 5: マウスイベントデータ構造

**Objective:** 開発者として、マウスイベントの情報を統一されたデータ構造で受け取りたい。それにより一貫性のあるイベントハンドリングが可能になる。

#### Acceptance Criteria

1. The Mouse Event System shall すべてのマウスイベントに共通の `MouseEventData` 構造体を使用する
2. The Mouse Event System shall イベントデータにターゲットエンティティを含める
3. The Mouse Event System shall イベントデータにスクリーン座標を含める
4. The Mouse Event System shall イベントデータにローカル座標を含める
5. The Mouse Event System shall イベントデータにタイムスタンプを含める

#### データ構造

```rust
/// マウスイベント共通データ
#[derive(Debug, Clone)]
pub struct MouseEventData {
    /// ターゲットエンティティ
    pub target: Entity,
    /// スクリーン座標（物理ピクセル）
    pub screen_point: PhysicalPoint,
    /// エンティティローカル座標（物理ピクセル）
    pub local_point: PhysicalPoint,
    /// イベント発生時刻
    pub timestamp: Instant,
    /// カーソル移動速度（MouseMove のみ有効）
    pub velocity: Option<CursorVelocity>,
}

/// マウスボタン
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// マウスイベント種別
#[derive(Debug, Clone)]
pub enum MouseEvent {
    MouseDown { data: MouseEventData, button: MouseButton },
    MouseUp { data: MouseEventData, button: MouseButton },
    Click { data: MouseEventData },
    RightClick { data: MouseEventData },
    DoubleClick { data: MouseEventData },
    MouseEnter { data: MouseEventData },
    MouseLeave { data: MouseEventData },
    MouseMove { data: MouseEventData },
}
```

---

### Requirement 6: Win32メッセージ統合

**Objective:** 開発者として、Win32メッセージをECSイベントに変換したい。それにより既存のwintfアーキテクチャと統合できる。

#### Acceptance Criteria

1. When `WM_MOUSEMOVE` を受信した時, the Mouse Event System shall `hit_test` を実行しホバー状態を更新する
2. When `WM_LBUTTONDOWN` を受信した時, the Mouse Event System shall `hit_test` を実行し `MouseDown` イベントを発火する
3. When `WM_LBUTTONUP` を受信した時, the Mouse Event System shall `MouseUp` イベントと条件付きで `Click` イベントを発火する
4. When `WM_RBUTTONDOWN`/`WM_RBUTTONUP` を受信した時, the Mouse Event System shall `RightClick` イベントを発火する
5. When `WM_LBUTTONDBLCLK` を受信した時, the Mouse Event System shall `DoubleClick` イベントを発火する
6. The Mouse Event System shall `ecs_wndproc` のハンドラとして実装する

#### Win32メッセージマッピング

| Win32 Message | Mouse Event |
|---------------|-------------|
| WM_MOUSEMOVE | MouseMove, MouseEnter, MouseLeave |
| WM_LBUTTONDOWN | MouseDown (Left) |
| WM_LBUTTONUP | MouseUp (Left), Click |
| WM_RBUTTONDOWN | MouseDown (Right) |
| WM_RBUTTONUP | MouseUp (Right), RightClick |
| WM_LBUTTONDBLCLK | DoubleClick |
| WM_MBUTTONDOWN | MouseDown (Middle) |
| WM_MBUTTONUP | MouseUp (Middle) |

---

### Requirement 7: ヒットテストキャッシュ（オプション）

**Objective:** 開発者として、同一座標での重複ヒットテストを避けたい。それによりパフォーマンスを向上できる。

#### Acceptance Criteria

1. The Mouse Event System shall 前回のマウス座標とヒット結果をキャッシュする
2. When マウス座標が前回と同一の場合, the Mouse Event System shall キャッシュからヒット結果を返す
3. When `ArrangementTreeChanged` イベントを受信した時, the Mouse Event System shall キャッシュを無効化する
4. The Mouse Event System shall キャッシュヒット率をデバッグログで報告する（開発時のみ）

#### キャッシュ構造

```rust
/// ヒットテストキャッシュ
#[derive(Resource, Default)]
pub struct HitTestCache {
    /// 前回のスクリーン座標
    pub last_point: Option<PhysicalPoint>,
    /// 前回のヒット結果
    pub last_result: Option<HitTestResult>,
    /// キャッシュ世代（ArrangementTreeChanged でインクリメント）
    pub generation: u64,
}
```

**Note**: 本要件はオプションであり、初期実装では省略可能。

---

### Requirement 8: ECSリソース・イベント統合

**Objective:** 開発者として、マウスイベントをECSのイベントキューで受け取りたい。それにより既存のbevy_ecsパターンと一貫性を保てる。

#### Acceptance Criteria

1. The Mouse Event System shall マウスイベントを `Events<MouseEvent>` として配信する
2. The Mouse Event System shall 現在ホバー中のエンティティを `HoveredEntity` リソースとして保持する
3. The Mouse Event System shall 現在押下中のマウスボタン状態を `MouseButtonState` リソースとして保持する
4. When エンティティが削除された時, the Mouse Event System shall 関連するホバー状態をクリアする

#### リソース定義

```rust
/// 現在ホバー中のエンティティ
#[derive(Resource, Default)]
pub struct HoveredEntity(pub Option<Entity>);

/// マウスボタン状態
#[derive(Resource, Default)]
pub struct MouseButtonState {
    pub left: bool,
    pub right: bool,
    pub middle: bool,
    /// 最後に MouseDown が発生したエンティティ（Click 判定用）
    pub down_target: Option<Entity>,
}
```

---

## Non-Functional Requirements

### NFR-1: パフォーマンス

- マウスイベント処理: 1ms以内で完了
- ヒットテスト込みで 16ms 以内（60fps 維持）
- キャッシュヒット時は 0.1ms 以内

### NFR-2: レスポンス

- Win32メッセージからイベント発火まで: 1フレーム以内
- ホバー状態の更新: リアルタイム

### NFR-3: 信頼性

- イベントの取りこぼしなし
- Enter/Leave の順序保証
- Click 判定の正確性

---

## Glossary

| 用語 | 説明 |
|------|------|
| MouseDown | マウスボタン押下イベント |
| MouseUp | マウスボタン解放イベント |
| Click | 同一エンティティ上での Down→Up 完了 |
| MouseEnter | カーソルがエンティティ領域に入った |
| MouseLeave | カーソルがエンティティ領域から出た |
| ローカル座標 | エンティティ左上を原点とした座標 |
| 物理ピクセル | DPIスケール適用後の実ピクセル |

---

## Appendix

### A. 関連ドキュメント

- 親仕様: `.kiro/specs/wintf-P0-event-system/requirements.md`
- ヒットテスト仕様: `.kiro/specs/completed/event-hit-test/requirements.md`
- イベントシステム設計: `doc/spec/08-event-system.md`

### B. 依存関係

```
event-mouse-basic
  └── event-hit-test (completed)
        ├── hit_test(world, root, screen_point)
        ├── hit_test_entity(world, entity, screen_point)
        └── hit_test_in_window(world, window, client_point)
```

### C. 実装優先順位

1. **Phase 1**: Win32メッセージハンドラ + MouseMove/Enter/Leave
2. **Phase 2**: MouseDown/MouseUp/Click
3. **Phase 3**: hit_test_detailed（ローカル座標）
4. **Phase 4**: カーソル速度計算
5. **Phase 5**: キャッシュ機構（オプション）

---

_Document generated by AI-DLC System on 2025-12-02_
