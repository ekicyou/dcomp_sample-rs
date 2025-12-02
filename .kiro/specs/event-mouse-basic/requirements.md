# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | event-mouse-basic 要件定義書 |
| **Version** | 1.1 (Draft) |
| **Date** | 2025-12-02 |
| **Parent Spec** | wintf-P0-event-system |
| **Author** | AI-DLC System |

---

## Introduction

本仕様書は wintf フレームワークにおけるマウス基本イベント処理の要件を定義する。親仕様「wintf-P0-event-system」の Requirement 3（マウスクリックイベント）と Requirement 4（マウスホバーイベント）を実装する。

### 背景

デスクトップマスコットアプリケーションでは、キャラクターへのクリック、ホバー、撫でる操作などのユーザーインタラクションが必須である。`event-hit-test` 仕様で実装されたヒットテストAPIを活用し、Win32メッセージからECSコンポーネントへの変換を行う。

### スコープ

**含まれるもの**:
- `MouseState` コンポーネントによるマウス状態のECS表現
- ヒットテストで当たった子エンティティへのコンポーネント付与
- Enter/Leave パターンによる状態変化検出
- Win32メッセージハンドラとヒットテストAPIの統合
- ローカル座標変換
- カーソル移動速度の計算（撫でる操作検出用）

**含まれないもの**:
- Click/RightClick 判定 → `event-dispatch` 仕様で対応（親への伝播が必要）
- ドラッグイベント → `event-drag-system` 仕様で対応
- 名前付きヒット領域 → `event-hit-test-named-regions` 仕様で対応
- イベントバブリング・キャプチャ → `event-dispatch` 仕様で対応

**設計方針**:
- 本仕様は「Win32マウスメッセージをECSコンポーネントとして持ち込む」ことに集中
- `MouseState` コンポーネントがあるエンティティ = ホバー中（マウスは1つ）
- `Added<MouseState>` で Enter、`MouseLeave` マーカーで Leave を検出
- Click判定（MouseDown→MouseUpの対応付け）は `event-dispatch` に委譲

### event-hit-test からの引き継ぎ事項

`event-hit-test` 仕様で実装されたAPIを統合し、以下の項目を本仕様で実装する：

| 引き継ぎ項目 | 説明 | 対応Requirement |
|-------------|------|-----------------|
| **ecs_wndproc 統合** | `WM_MOUSEMOVE`, `WM_LBUTTONDOWN` 等のハンドラから `hit_test` API を呼び出す | Req 5 |
| **ローカル座標変換** | `GlobalArrangement.bounds` を使用したエンティティローカル座標への変換 | Req 1 |
| **hit_test_detailed** | ローカル座標付きヒット結果を返す関数 | Req 1 |
| **キャッシュ機構** | 座標が同一の場合は前回結果を返す | Req 6 (オプション) |

---

## Requirements

### Requirement 1: MouseState コンポーネント

**Objective:** 開発者として、マウス状態をECSコンポーネントとして取得したい。それによりECSパターンで一貫したマウス処理ができる。

#### Acceptance Criteria

1. The Mouse Event System shall ヒットテストでヒットしたエンティティに `MouseState` コンポーネントを付与する
2. The Mouse Event System shall マウスがエンティティから離れた時に `MouseState` を削除する
3. The Mouse Event System shall `MouseState` にスクリーン座標（物理ピクセル）を含める
4. The Mouse Event System shall `MouseState` にエンティティローカル座標を含める
5. The Mouse Event System shall `MouseState` に各ボタンの押下状態（`left_down`, `right_down`, `middle_down`）を含める
6. The Mouse Event System shall `MouseState` にタイムスタンプを含める
7. The Mouse Event System shall `MouseState` にカーソル移動速度を含める
8. The Mouse Event System shall ダブルクリック検出時に `MouseState.double_click` を `true` に設定する

#### コンポーネント定義

```rust
/// マウス状態コンポーネント
/// 
/// hit_test がヒットしたエンティティに付与される。
/// コンポーネントの存在 = ホバー中。
/// Added<MouseState> で Enter を検出。
/// 
/// マウスは1つなので、同時に1エンティティのみが持つ。
/// 
/// メモリ戦略: SparseSet - 頻繁な挿入/削除
#[derive(Component, Debug, Clone)]
#[component(storage = "SparseSet")]
pub struct MouseState {
    /// スクリーン座標（物理ピクセル）
    pub screen_point: PhysicalPoint,
    /// エンティティローカル座標（物理ピクセル）
    pub local_point: PhysicalPoint,
    /// 左ボタン押下中
    pub left_down: bool,
    /// 右ボタン押下中
    pub right_down: bool,
    /// 中ボタン押下中
    pub middle_down: bool,
    /// ダブルクリック検出（1フレームのみtrue）
    pub double_click: bool,
    /// カーソル移動速度
    pub velocity: CursorVelocity,
    /// タイムスタンプ
    pub timestamp: Instant,
}

/// カーソル移動速度（ピクセル/秒）
#[derive(Debug, Clone, Default)]
pub struct CursorVelocity {
    pub x: f32,
    pub y: f32,
    pub magnitude: f32,
}
```

#### 使用パターン

| 検出したいこと | クエリ |
|---------------|--------|
| Enter（入った瞬間） | `Added<MouseState>` |
| ホバー中 | `With<MouseState>` |
| 状態変化 | `Changed<MouseState>` |
| Leave（離れた瞬間） | `With<MouseLeave>` |
| 左ボタン押下中 | `query.get(e).map(\|s\| s.left_down)` |
| ダブルクリック | `query.get(e).map(\|s\| s.double_click)` |

---

### Requirement 2: MouseLeave マーカー

**Objective:** 開発者として、マウスがエンティティから離れた瞬間を検出したい。それによりLeaveアニメーション等を実行できる。

#### Acceptance Criteria

1. The Mouse Event System shall `MouseState` 削除時に `MouseLeave` マーカーを付与する
2. The Mouse Event System shall `FrameFinalize` スケジュールで `MouseLeave` を削除する
3. When ウィンドウ外にマウスが移動した時（WM_MOUSELEAVE）, the Mouse Event System shall `MouseLeave` を付与する

#### コンポーネント定義

```rust
/// マウス離脱マーカー（1フレーム限り）
/// 
/// MouseState が削除されたフレームに付与される。
/// FrameFinalize で削除されるため、1フレームのみ存在。
/// 
/// メモリ戦略: SparseSet - 一時的マーカー
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
#[component(storage = "SparseSet")]
pub struct MouseLeave;
```

#### ライフサイクル

```
1. マウスがエンティティに入る
   → MouseState を追加
   → Added<MouseState> で Enter を検出

2. マウスがエンティティ上で動く / ボタン操作
   → MouseState を更新
   → Changed<MouseState> で変化を検出

3. マウスがエンティティから離れる
   → MouseState を削除
   → MouseLeave を追加（With<MouseLeave> で検出）

4. FrameFinalize スケジュール
   → MouseLeave を削除
```

---

### Requirement 3: カーソル移動速度

**Objective:** 開発者として、カーソルの移動速度を取得したい。それにより「撫でる」操作の検出が可能になる。

#### Acceptance Criteria

1. The Mouse Event System shall `MouseState.velocity` にカーソル移動速度（ピクセル/秒）を含める
2. The Mouse Event System shall 直前の位置と現在位置、および経過時間から速度を計算する
3. The Mouse Event System shall 速度計算に使用する履歴を最大5サンプル保持する
4. When 最初のマウス移動の場合, the Mouse Event System shall 速度を 0.0 として報告する

---

### Requirement 4: ローカル座標変換

**Objective:** 開発者として、スクリーン座標をエンティティローカル座標に変換したい。それによりエンティティ内の正確な位置を特定できる。

#### Acceptance Criteria

1. The Mouse Event System shall `MouseState.local_point` にエンティティローカル座標を含める
2. The Mouse Event System shall `GlobalArrangement.bounds` の left/top を使用してローカル座標を計算する
3. The Mouse Event System shall `hit_test_detailed` 関数でエンティティとローカル座標を同時に返す

#### 座標変換計算

```
local_x = screen_x - GlobalArrangement.bounds.left
local_y = screen_y - GlobalArrangement.bounds.top
```

**Note**: 現在の `GlobalArrangement` は軸平行変換のみをサポートするため、回転・スキュー変換は不要。

---

### Requirement 5: Win32メッセージ統合

**Objective:** 開発者として、Win32メッセージをECSコンポーネントに変換したい。それにより既存のwintfアーキテクチャと統合できる。

#### Acceptance Criteria

1. When `WM_NCHITTEST` を受信し座標がクライアント領域外の場合, the Mouse Event System shall `None` を返して `DefWindowProcW` に委譲する
2. When `WM_NCHITTEST` を受信し座標がクライアント領域内の場合, the Mouse Event System shall `hit_test` を実行する
3. When `hit_test` が `None` を返した場合, the Mouse Event System shall `HTTRANSPARENT` を返す（クリックスルー）
4. When `hit_test` がエンティティを返した場合, the Mouse Event System shall `HTCLIENT` を返す
5. When `WM_MOUSEMOVE` を受信し `WindowMouseTracking` が `false` の場合, the Mouse Event System shall `TrackMouseEvent(TME_LEAVE)` を呼び出して `true` に設定する
6. When `WM_MOUSEMOVE` を受信した時, the Mouse Event System shall `hit_test` を実行し `MouseState` を更新する
7. When `WM_MOUSELEAVE` を受信した時, the Mouse Event System shall `WindowMouseTracking` を `false` に設定し、`MouseState` を削除して `MouseLeave` を付与する
8. When `WM_LBUTTONDOWN` を受信した時, the Mouse Event System shall `MouseState.left_down` を `true` に更新する
9. When `WM_LBUTTONUP` を受信した時, the Mouse Event System shall `MouseState.left_down` を `false` に更新する
10. When `WM_RBUTTONDOWN` を受信した時, the Mouse Event System shall `MouseState.right_down` を `true` に更新する
11. When `WM_RBUTTONUP` を受信した時, the Mouse Event System shall `MouseState.right_down` を `false` に更新する
12. When `WM_LBUTTONDBLCLK` を受信した時, the Mouse Event System shall `MouseState.double_click` を `true` に設定する（次フレームでリセット）
13. The Mouse Event System shall `ecs_wndproc` のハンドラとして実装する

#### Win32メッセージマッピング

| Win32 Message | MouseState 更新 |
|---------------|-----------------|
| WM_NCHITTEST | クライアント領域判定 + hit_test |
| WM_MOUSEMOVE | 座標更新、ボタン状態（wParam）、Enter/Leave処理 |
| WM_MOUSELEAVE | MouseState削除 + MouseLeave付与 |
| WM_LBUTTONDOWN | left_down = true |
| WM_LBUTTONUP | left_down = false |
| WM_RBUTTONDOWN | right_down = true |
| WM_RBUTTONUP | right_down = false |
| WM_LBUTTONDBLCLK | double_click = true |
| WM_RBUTTONDBLCLK | double_click = true |
| WM_MBUTTONDOWN | middle_down = true |
| WM_MBUTTONUP | middle_down = false |

#### ボタン状態の取得

`WM_MOUSEMOVE` の `wParam` からボタン押下状態を取得：
- `MK_LBUTTON` (0x0001): 左ボタン押下中
- `MK_RBUTTON` (0x0002): 右ボタン押下中
- `MK_MBUTTON` (0x0010): 中ボタン押下中

**Note**: Click / RightClick 判定は `event-dispatch` 仕様で実装。本仕様では Win32 メッセージを直接 `MouseState` コンポーネントに反映するのみ。

---

### Requirement 6: ヒットテストキャッシュ（オプション）

**Objective:** 開発者として、同一座標での重複ヒットテストを避けたい。それによりパフォーマンスを向上できる。

#### Acceptance Criteria

1. The Mouse Event System shall 前回のマウス座標とヒット結果をキャッシュする
2. When マウス座標が前回と同一の場合, the Mouse Event System shall キャッシュからヒット結果を返す
3. When `ArrangementTreeChanged` イベントを受信した時, the Mouse Event System shall キャッシュを無効化する
4. The Mouse Event System shall キャッシュヒット率をデバッグログで報告する（開発時のみ）

**Note**: 本要件はオプションであり、初期実装では省略可能。

---

### Requirement 7: WindowMouseTracking コンポーネント

**Objective:** 開発者として、ウィンドウごとのマウストラッキング状態を管理したい。それによりWM_MOUSELEAVEを正しく受信できる。

#### Acceptance Criteria

1. The Mouse Event System shall `WindowMouseTracking` コンポーネントでトラッキング状態を管理する
2. When `WM_MOUSEMOVE` 受信時にトラッキングが無効な場合, the Mouse Event System shall `TrackMouseEvent(TME_LEAVE)` を呼び出す
3. When `WM_MOUSELEAVE` を受信した時, the Mouse Event System shall トラッキングを無効にする

#### コンポーネント定義

```rust
/// ウィンドウのマウストラッキング状態
/// 
/// Win32 の TrackMouseEvent 呼び出し状態を管理。
/// WM_MOUSELEAVE を受信するために必要。
/// 
/// メモリ戦略: SparseSet - Window は全エンティティの1〜5%程度
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Default)]
#[component(storage = "SparseSet")]
pub struct WindowMouseTracking(pub bool);
```

---

### Requirement 8: FrameFinalize クリーンアップ

**Objective:** 開発者として、一時的マーカーコンポーネントを自動クリーンアップしたい。それにより手動削除の手間を省ける。

#### Acceptance Criteria

1. The `FrameFinalize` schedule shall `MouseLeave` コンポーネントを全削除するクリーンアップシステムを実行する
2. The `FrameFinalize` schedule shall `MouseState.double_click` を `false` にリセットする
3. The cleanup systems shall Commit システムの後に実行される
4. The `FrameFinalize` schedule shall 将来の他の一時的マーカーのクリーンアップにも使用可能であること

#### スケジュール定義

```rust
/// フレーム最終化スケジュール
/// 
/// DirectComposition の Commit と一時マーカーのクリーンアップを行う。
/// フレームの最後に実行される。
/// 
/// 実行内容:
/// 1. IDCompositionDevice3::Commit() - ビジュアル変更の確定
/// 2. MouseLeave 等の一時マーカーコンポーネントの削除
/// 3. MouseState.double_click のリセット
#[derive(ScheduleLabel, Clone, Debug, PartialEq, Eq, Hash)]
pub struct FrameFinalize;
```

#### 実行順序

```
Input → Update → PreLayout → Layout → PostLayout → UISetup → 
GraphicsSetup → Draw → PreRenderSurface → RenderSurface → 
Composition → FrameFinalize
                    ├── commit_composition（既存）
                    ├── cleanup_mouse_leave（新規）
                    └── reset_double_click（新規）
```

---

### Requirement 9: CommitComposition → FrameFinalize リネーム

**Objective:** 開発者として、フレーム終了時の処理を統一された名前で理解したい。それによりスケジュール構造が直感的になる。

#### Acceptance Criteria

1. The ECS World shall `CommitComposition` スケジュールを `FrameFinalize` にリネームする
2. The `FrameFinalize` schedule shall DirectComposition の Commit を実行する

#### 移行作業

| 変更箇所 | Before | After |
|---------|--------|-------|
| スケジュール定義 | `CommitComposition` | `FrameFinalize` |
| world.rs 登録 | `schedules.insert(Schedule::new(CommitComposition))` | `schedules.insert(Schedule::new(FrameFinalize))` |
| try_tick_world | `try_run_schedule(CommitComposition)` | `try_run_schedule(FrameFinalize)` |
| システム登録 | `add_systems(CommitComposition, ...)` | `add_systems(FrameFinalize, ...)` |

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
| MouseState | マウス状態コンポーネント（座標、ボタン、速度を含む） |
| MouseLeave | マウス離脱マーカー（1フレーム限り） |
| Enter | マウスがエンティティに入った瞬間（Added<MouseState>で検出） |
| Leave | マウスがエンティティから離れた瞬間（With<MouseLeave>で検出） |
| ホバー中 | MouseStateが付与されている状態 |
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

1. **Phase 1**: MouseState コンポーネント + Win32メッセージハンドラ
2. **Phase 2**: MouseLeave マーカー + FrameFinalize クリーンアップ
3. **Phase 3**: ローカル座標変換（hit_test_detailed）
4. **Phase 4**: カーソル速度計算
5. **Phase 5**: ヒットテストキャッシュ（オプション）

---

_Document generated by AI-DLC System on 2025-12-02_
