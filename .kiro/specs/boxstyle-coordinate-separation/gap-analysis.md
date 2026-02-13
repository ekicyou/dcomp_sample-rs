# ギャップ分析: boxstyle-coordinate-separation

## 1. 現状分析

### 1.1 関連アセット一覧

| # | ファイル | 関連箇所 | 役割 |
|---|---------|---------|------|
| 1 | `ecs/window_proc/handlers.rs` L231-277 | `WM_WINDOWPOSCHANGED` | BoxStyle.inset（物理px）+ BoxStyle.size（論理px）書き込み |
| 2 | `ecs/drag/systems.rs` L23-82 | `apply_window_drag_movement` | BoxStyle.inset のleft/top更新 |
| 3 | `ecs/drag/dispatch.rs` L89-108 | `DragTransition::Started` | BoxStyle.inset からinitial_inset取得 |
| 4 | `ecs/drag/mod.rs` L65-72 | `DraggingState` | `initial_inset: (f32, f32)` フィールド |
| 5 | `ecs/layout/systems.rs` L114-139 | `build_taffy_styles_system` | `Changed<BoxStyle>` → `TaffyStyle` 変換 |
| 6 | `ecs/layout/systems.rs` L192-253 | `compute_taffy_layout_system` | `Changed<TaffyStyle>\|Changed<BoxStyle>` でレイアウト再計算 |
| 7 | `ecs/layout/systems.rs` L269-353 | `update_arrangements_system` | taffy結果 → `Arrangement` |
| 8 | `ecs/layout/systems.rs` L361-417 | `window_pos_sync_system` | `GlobalArrangement` → `WindowPos` (正方向同期) |
| 9 | `ecs/layout/systems.rs` L460-503 | `sync_window_arrangement_from_window_pos` | `WindowPos` → `Arrangement.offset` (逆方向同期) |
| 10 | `ecs/layout/high_level.rs` L466-541 | `From<&BoxStyle> for taffy::Style` | `BoxStyle.inset` → `taffy::Style.inset` マッピング |
| 11 | `ecs/layout/systems.rs` L537-570 | `initialize_layout_root` | LayoutRoot/MonitorのBoxStyle.inset設定 |
| 12 | `ecs/layout/systems.rs` L636-656 | `update_monitor_layout_system` | MonitorのBoxStyle.inset更新 |
| 13 | `ecs/world.rs` L223-381 | スケジュール登録 | 実行順序の定義 |
| 14 | examples 3ファイル | Window spawn | BoxStyle.inset で初期位置を指定 |

### 1.2 現行データフロー

#### WM_WINDOWPOSCHANGED 時（外部からのウィンドウ移動）

```
WM_WINDOWPOSCHANGED
  ├─→ WindowPos.position = client_pos (物理px)    ← Changed<WindowPos> 発火
  ├─→ WindowPos.size = client_size (物理px)
  ├─→ BoxStyle.size = logical_size (論理px/DIP)   ← Changed<BoxStyle> 発火
  └─→ BoxStyle.inset = physical_pos (物理px)      ← ★不要な書き込み

Layout スケジュール:
  build_taffy_styles_system  ← Changed<BoxStyle>で全フィールドをtaffy::Styleに変換
  compute_taffy_layout_system ← ★レイアウト全体を再計算（位置のみ変更でも）
  update_arrangements_system  ← taffy結果をArrangementに反映

PostLayout スケジュール:
  sync_window_arrangement_from_window_pos ← Changed<WindowPos>で Arrangement.offsetに物理座標を設定
  propagate_global_arrangements ← GlobalArrangement伝播
  window_pos_sync_system ← GlobalArrangement→WindowPos（同値なら変更なし）
```

#### ドラッグ時

```
Input スケジュール:
  apply_window_drag_movement → BoxStyle.inset.left/top 更新 ← ★Changed<BoxStyle> 発火

Layout スケジュール:
  build_taffy_styles → compute_taffy_layout → update_arrangements
  ★ドラッグのたびにレイアウト全体を再計算

PostLayout スケジュール:
  propagate_global → window_pos_sync → WindowPos
  → apply_window_pos_changes → SetWindowPos
```

### 1.3 アーキテクチャパターンと制約

- **ECS変更検知**: bevy_ecs の `Changed<T>` はコンポーネント単位。`BoxStyle` の1フィールドだけ変えても全体が「変更あり」と検知される
- **taffy レイアウト再計算**: `Changed<BoxStyle>` または `Changed<TaffyStyle>` があれば `compute_taffy_layout` が全ツリーを再計算
- **座標系の不整合**: `BoxStyle.inset` は物理ピクセル、`BoxStyle.size` は論理ピクセル（DIP）— 同一コンポーネント内で単位が混在
- **二重経路**: ウィンドウ位置は `BoxStyle.inset → taffy → Arrangement` と `WindowPos → sync_window_arrangement_from_window_pos → Arrangement.offset` の2経路で伝搬。後者が最終的に勝つ（PostLayoutで上書き）

## 2. 要件ごとの実現可能性分析

### Requirement 1: BoxStyle からウィンドウXY座標の除外

| 受入基準 | 実現性 | 既存資産 | ギャップ |
|---------|--------|---------|---------|
| AC1: WM_WINDOWPOSCHANGED で BoxStyle.inset 不書き込み | ✅ 容易 | handlers.rs L253-261 を削除 | なし |
| AC2: BoxStyle.size は現行通り更新 | ✅ 容易 | handlers.rs L247-250 を維持 | サイズのみ変更時の `Changed<BoxStyle>` 制御が必要 |
| AC3: 位置のみ変更で Changed<BoxStyle> 不発火 | ⚠️ 検討要 | 現在は size と inset を同一 `get_mut` で更新 | サイズ不変時に `get_mut` を呼ばない分岐が必要 |
| AC4: サイズ変更時に Changed<BoxStyle> 発火 | ✅ 容易 | 現行動作をサイズ変更時のみに限定 | なし |

**技術的ポイント**: `get_mut::<BoxStyle>()` を呼ぶと `DerefMut` で `Changed` が発火する。サイズ不変のウィンドウ移動で `Changed<BoxStyle>` を抑制するには、サイズ変更有無を事前判定し、不変なら `get_mut` を呼ばないようにする必要がある。

### Requirement 2: ドラッグシステムのBoxStyle.inset依存除去

| 受入基準 | 実現性 | 既存資産 | ギャップ |
|---------|--------|---------|---------|
| AC1: BoxStyle.inset を使わずにウィンドウ位置更新 | ⚠️ 設計判断要 | ドラッグは現在 BoxStyle.inset → taffy パイプライン経由 | 代替パイプラインの設計が必要 |
| AC2: 初期位置を WindowPos/Arrangement から取得 | ✅ 容易 | WindowPos.position が同等情報を持つ | DraggingState.initial_inset の型変更 |
| AC3: ドラッグ中に Changed<BoxStyle> 不発火 | ⚠️ 設計判断要 | 現在は BoxStyle.inset 経由でパイプライン駆動 | ドラッグの位置更新先を変更する必要あり |

**技術的ポイント**: ドラッグの代替パイプラインとして2つの候補がある：
- **案A**: `WindowPos` に直接書き込み → PostLayout の `sync_window_arrangement_from_window_pos` が `Arrangement.offset` に反映 → `GlobalArrangement` → `window_pos_sync` → `SetWindowPos`
- **案B**: `Arrangement.offset` に直接書き込み → PostLayout の `propagate_global_arrangements` → `window_pos_sync` → `WindowPos` → `SetWindowPos`

### Requirement 3: 代替伝搬経路の保証

| 受入基準 | 実現性 | 既存資産 | ギャップ |
|---------|--------|---------|---------|
| AC1: WindowPos.position 参照可能 | ✅ 既存 | WM_WINDOWPOSCHANGED で更新済み | なし |
| AC2: WindowPos.size 参照可能 | ✅ 既存 | WM_WINDOWPOSCHANGED で更新済み | なし |
| AC3: sync_window_arrangement_from_window_pos 経由反映 | ✅ 既存 | PostLayout で動作済み | なし |
| AC4: WindowPos→Arrangement→GlobalArrangement 伝搬 | ✅ 既存 | 全パイプライン実装済み | なし |

**結論**: 代替伝搬経路は完全に実装済み。追加実装不要。

### Requirement 4: BoxStyle.inset のウィンドウ以外の用途保全

| 受入基準 | 実現性 | 既存資産 | ギャップ |
|---------|--------|---------|---------|
| AC1: 非WindowエンティティのBoxStyle.inset維持 | ✅ 影響なし | Monitor/LayoutRoot は独自に設定 | なし |
| AC2: BoxStyle.inset の型定義不変 | ✅ 変更不要 | 構造体定義に変更なし | なし |

**結論**: Window エンティティへの **書き込み** のみ除去するため、型定義や他エンティティの使用には影響なし。

## 3. 実装アプローチ検討

### Option A: ハンドラ修正のみ（最小変更）

**概要**: `WM_WINDOWPOSCHANGED` ハンドラで BoxStyle.inset 書き込みを削除し、サイズ変更時のみ BoxStyle を更新。ドラッグシステムは BoxStyle.inset → `WindowPos` への切り替え。

**変更対象ファイル**:
| ファイル | 変更内容 |
|---------|---------|
| `ecs/window_proc/handlers.rs` | BoxStyle.inset 書き込みを削除、サイズ変更判定を追加 |
| `ecs/drag/systems.rs` | BoxStyle.inset → WindowPos に直接書き込み |
| `ecs/drag/dispatch.rs` | initial_inset を WindowPos.position から取得 |
| `ecs/drag/mod.rs` | DraggingState.initial_inset の型/名称変更 |
| examples 3ファイル | Window spawn時の BoxStyle.inset 削除 |

**ドラッグ代替パイプライン**:
```
apply_window_drag_movement → WindowPos.position 更新 (Changed<WindowPos> 発火)
  ↓ PostLayout
sync_window_arrangement_from_window_pos → Arrangement.offset 更新
  ↓
propagate_global_arrangements → GlobalArrangement
  ↓
window_pos_sync_system → WindowPos（同値なら無変更、フィードバック停止）
  ↓ UISetup
apply_window_pos_changes → SetWindowPos (echo 経由)
```

**トレードオフ**:
- ✅ 変更箇所が少なく影響範囲が限定的
- ✅ 既存の PostLayout パイプラインをそのまま活用
- ✅ BoxStyle の型定義に一切変更なし
- ❌ Window spawn 時に `BoxStyle.inset` で初期位置を指定するパターンが使えなくなる
- ❌ ドラッグ時に `Changed<WindowPos>` が発火し、`window_pos_sync_system` との相互作用を検証する必要がある

### Option B: ドラッグをArrangement.offset直接更新に変更

**概要**: Option A と同じくハンドラから BoxStyle.inset 除去。ドラッグは `Arrangement.offset` に直接書き込み、PostLayout の `propagate_global_arrangements` 以降で伝搬。

**変更対象ファイル**: Option A と同一 + `ecs/drag/systems.rs` の書き込み先が `Arrangement.offset` に変更

**ドラッグ代替パイプライン**:
```
apply_window_drag_movement → Arrangement.offset 更新 (Changed<Arrangement> 発火)
  ↓ PostLayout
propagate_global_arrangements → GlobalArrangement
  ↓
window_pos_sync_system → WindowPos (Changed<WindowPos> 発火)
  ↓ UISetup
apply_window_pos_changes → SetWindowPos (echo 経由)
```

**トレードオフ**:
- ✅ `sync_window_arrangement_from_window_pos` をバイパスでき、パイプラインがシンプル
- ✅ PostLayout の伝搬パスが1本に集約
- ❌ Input スケジュールで `Arrangement` を直接操作する設計の是非（レイヤー越境）
- ❌ `Arrangement` は本来 Layout システムの出力であり、外部からの直接書き込みはアーキテクチャの一貫性に疑問

### Option C: WndProcレベル直接SetWindowPos方式（drag-wndproc-direct-move 合流）

**概要**: ドラッグ中のウィンドウ移動をECSパイプラインから完全に切り離し、WM_MOUSEMOVEハンドラ内でthread_local `DragState` を参照して直接 `SetWindowPos` を呼び出す。ドラッグ終了時に最終位置をECSに書き戻す。

**背景 (計測結果)**:
- ECS経由ドラッグ: タスクバードラッグの約1/3のフレームレート（Debug build）
- 主因: ECSフレーム待ちレイテンシ + taffy全ツリー再計算 + SetWindowPos同期API

**既存インフラ（活用可能）**:
| インフラ | 状態 | 用途 |
|---------|------|------|
| thread_local `DragState` (Idle/Preparing/Dragging) | 実装済み | ドラッグ状態管理 |
| `DragAccumulatorResource` (Arc<Mutex>) | 実装済み | wndproc→ECS転送（ドラッグ中は不使用に変更） |
| `guarded_set_window_pos()` + IS_SELF_INITIATED | 実装済み | エコーバック防止 |
| `find_ancestor_with_drag_config()` | 実装済み | hit_test結果からDragConfig探索 |

**変更対象ファイル**:
| ファイル | 変更内容 |
|---------|--------|
| `ecs/window_proc/handlers.rs` | BoxStyle.inset 書き込み削除、サイズ変更判定追加 |
| `ecs/drag/systems.rs` | `apply_window_drag_movement` のBoxStyle.inset書き込みを廃止 |
| `ecs/drag/dispatch.rs` | Dragging状態遷移時にHWND+初期位置をDragStateにキャッシュ |
| `ecs/drag/mod.rs` | DraggingState.initial_inset → 不要化（DragStateに移行） |
| `ecs/window_proc/` (WM_MOUSEMOVE) | Dragging状態チェック → 直接SetWindowPos呼び出し |
| examples 3ファイル | Window spawn時の BoxStyle.inset 削除 |

**ドラッグ新パイプライン**:
```
WM_MOUSEMOVE (wndproc thread)
  → DragState::Dragging { hwnd, initial_pos, prev_pos } 参照
  → delta計算 → guarded_set_window_pos(hwnd, new_x, new_y)
  → WM_WINDOWPOSCHANGED (echo → WindowPos bypass更新)
  → (ECSパイプラインをバイパス — Changed<BoxStyle> 不発火)

ドラッグ終了時:
  → DragState::Idle に遷移
  → WindowPos に最終位置反映（Changed<WindowPos> 発火）
  → sync_window_arrangement_from_window_pos → Arrangement 整合性回復
```

**トレードオフ**:
- ✅ WM_MOUSEMOVE → SetWindowPos の最短パス（ECSフレーム待ち不要）
- ✅ レイアウト再計算を完全にバイパス
- ✅ 既存インフラ（DragState, guarded_set_window_pos）を活用可能
- ✅ `drag-wndproc-direct-move` 仕様と本仕様を一括で解決
- ❌ WndProcレベルのロジック増加（スレッド安全性の注意）
- ❌ ドラッグ終了時のECS同期タイミングの設計が必要
- ❌ DragConstraint の適用ロジックをWndProcレベルにも移植する必要あり

## 4. 追加調査事項（設計フェーズ向け）

### Research Needed

1. **Window spawn時の初期位置指定方法**: BoxStyle.inset を初期位置に使用する例が3ファイルある。BoxStyle.inset を使わない場合、初期位置は `WindowPos.position` で指定するか、あるいは `Window` コンポーネントに初期位置フィールドを追加するか — **設計判断**
2. **DragConstraint の WndProc レベル移植**: 現在 `DragConstraint` は ECS システム内で適用されている。WndProc レベル方式では、WM_MOUSEMOVE ハンドラ内で制約を適用する必要があり、thread_local への制約情報キャッシュ方法を検討する必要がある — **設計判断**
3. **ドラッグ終了時の ECS 同期タイミング**: ドラッグ終了（WM_LBUTTONUP等）時に `DragState::Idle` に遷移し最終位置をECSに書き戻す。`DragEndEvent` で `WindowPos` を更新し `sync_window_arrangement_from_window_pos` で整合性を回復するフローの安全性 — **Research Needed**
4. **DragAccumulatorResource の役割変更**: WndProcレベル方式ではドラッグ中のデルタ蓄積が不要になるが、DragEvent（ECS側）の発行は維持すべきか（ユーザーコールバック用） — **設計判断**
5. **フィードバックループ収束**: ドラッグ終了→WindowPos更新→sync_window_arrangement→propagate_global→window_pos_sync のパスでの収束検証 — **テスト追加の検討**

## 5. 工数・リスク評価

| 項目 | 評価 | 根拠 |
|------|------|------|
| **工数** | **M** (3-7日) | handlers.rs + ドラッグ WndProc レベル化 + examples変更 + テスト。drag-wndproc-direct-move 合流により ECS/WndProc 境界の設計が必要 |
| **リスク** | **Medium** | 既存インフラ（DragState, guarded_set_window_pos）活用により未知の技術リスクは低い。DragConstraint移植とドラッグ終了時同期の設計精度が鍵 |

## 6. 推奨事項

- **推奨アプローチ: Option C（WndProcレベル直接方式）** — drag-wndproc-direct-move 合流を踏まえ、ECSパイプライン完全バイパスによる最大の効果を実現
- **Option A/B は非推奨**: ECSパイプライン経由のドラッグではECSフレーム待ちレイテンシが残り、体感改善が限定的
- **設計フェーズでの決定事項**:
  - WM_MOUSEMOVE ハンドラ内のSetWindowPos呼び出し設計
  - DragConstraint の thread_local キャッシュ方法
  - ドラッグ終了時のECS書き戻しタイミングと安全性
  - DragAccumulatorResource / DragEvent の役割変更
  - Window spawn時の初期位置指定パターン
