# Gap Analysis: event-dispatch

| 項目 | 内容 |
|------|------|
| **Document Title** | event-dispatch ギャップ分析 |
| **Version** | 1.0 |
| **Date** | 2025-12-03 |
| **Spec Reference** | `.kiro/specs/event-dispatch/requirements.md` v0.2 |

---

## Executive Summary

| 項目 | 評価 |
|------|------|
| **工数** | M (3-7日) |
| **リスク** | Low |
| **推奨アプローチ** | Option B: 新規モジュール作成 |

**主要ポイント**:
- 既存のヒットテスト（`hit_test.rs`）と `MouseState` コンポーネントが完成済み
- 階層走査パターン（`DepthFirstReversePostOrder`, `ChildOf.parent()`）が確立
- 新規 `dispatch.rs` モジュール作成が最適（既存モジュールへの影響最小）
- 排他システム (`&mut World`) の実装は bevy_ecs の標準パターン

---

## 1. Current State Investigation

### 1.1 関連資産マップ

| 資産 | ファイル | 責務 | 状態 |
|------|----------|------|------|
| **MouseState** | `ecs/mouse.rs` | マウス状態コンポーネント | ✅ 完成 |
| **hit_test_in_window** | `ecs/layout/hit_test.rs` | ウィンドウ座標→Entity特定 | ✅ 完成 |
| **DepthFirstReversePostOrder** | `ecs/common/tree_iter.rs` | 深さ優先逆順走査（最前面優先） | ✅ 完成 |
| **ChildOf / Children** | bevy_ecs (re-exported) | ECS親子関係 | ✅ bevy_ecs標準 |
| **handlers.rs** | `ecs/window_proc/handlers.rs` | WM_* → MouseState付与 | ✅ 完成 |
| **world.rs** | `ecs/world.rs` | スケジュール登録 | ✅ 拡張ポイント確定 |

### 1.2 確立済みパターン

#### 階層走査（バブリング用）
```rust
// 子→親の走査パターン（ChildOf.parent()）
// graphics/systems.rs:1017-1020 より
let mut current = entity;
while let Ok(co) = child_of_query.get(current) {
    depth += 1;
    current = co.parent();
}
```

#### ヒットテスト統合
```rust
// handlers.rs:536-539 より
let hit_entity = hit_test_in_window(
    world_borrow.world(),
    window_entity,
    HitTestPoint::new(x as f32, y as f32),
);
```

#### SparseSet コンポーネント
```rust
// mouse.rs:98-99 より
#[derive(Component, Debug, Clone)]
#[component(storage = "SparseSet")]
pub struct MouseState { ... }
```

### 1.3 スケジュール統合ポイント

```
Input スケジュール:
  ├── process_mouse_buffers  ← 既存
  └── dispatch_mouse_events  ← 新規追加（process_mouse_buffersの後）

FrameFinalize スケジュール:
  └── clear_transient_mouse_state  ← 既存
```

---

## 2. Requirements Feasibility Analysis

### 2.1 要件→資産マッピング

| Requirement | 必要資産 | 既存資産 | ギャップ |
|-------------|----------|----------|----------|
| R1: バブリング | 親取得API | `ChildOf.parent()` | なし |
| R2: キャプチャ | 子走査API | `Children.iter()` | なし（P2） |
| R3: ハンドラコンポーネント | `MouseEventHandler` | - | **Missing** |
| R4: イベントコンテキスト | `EventContext<E>` | - | **Missing** |
| R5: ディスパッチシステム | `dispatch_mouse_events` | - | **Missing** |
| R6: ECS統合 | スケジュール登録 | `world.rs` | 拡張のみ |
| R7: メモリ戦略 | SparseSet | 既存パターン | なし |
| R8: 汎用ディスパッチ | ジェネリック設計 | - | **Missing** |
| R9: イベント履歴 | リングバッファ | - | P2（後回し）|

### 2.2 ギャップ詳細

#### G1: MouseEventHandler コンポーネント【Missing】
```rust
/// 新規作成
#[derive(Component, Clone, Copy)]
#[component(storage = "SparseSet")]
pub struct MouseEventHandler {
    pub handler: fn(&mut World, Entity, &EventContext<MouseState>) -> bool,
}
```
- **複雑度**: Low（単純な構造体）
- **依存**: なし

#### G2: EventContext 構造体【Missing】
```rust
/// 新規作成
#[derive(Clone)]
pub struct EventContext<E> {
    pub original_target: Entity,
    pub event_data: E,
    pub phase: Phase,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Capture,
    Target,
    Bubble,
}
```
- **複雑度**: Low
- **依存**: なし

#### G3: dispatch_mouse_events システム【Missing】
```rust
/// 新規作成 - 排他システム
pub fn dispatch_mouse_events(world: &mut World) {
    // 1. MouseState を持つエンティティを収集
    // 2. 各エンティティについて:
    //    a. 親チェーンを収集（バブリングパス）
    //    b. ハンドラをCopyで収集（fnポインタ）
    //    c. 各ハンドラを順次呼び出し（target → bubble）
    //    d. 戻り値trueで停止
}
```
- **複雑度**: Medium（2パス実装、世界借用管理）
- **依存**: G1, G2

#### G4: スケジュール登録【拡張のみ】
```rust
// world.rs への追加
schedules.add_systems(
    Input,
    crate::ecs::dispatch::dispatch_mouse_events
        .after(crate::ecs::mouse::process_mouse_buffers),
);
```
- **複雑度**: Low
- **依存**: G3

---

## 3. Implementation Approach Options

### Option A: 既存 mouse.rs 拡張

**概要**: `mouse.rs` に `MouseEventHandler` とディスパッチロジックを追加

**Trade-offs**:
- ✅ 既存モジュールとの統合が自然
- ❌ mouse.rs が既に800行超、肥大化リスク
- ❌ 単一責任原則違反（状態管理 + ディスパッチ）

**評価**: 非推奨

### Option B: 新規 dispatch.rs モジュール作成【推奨】

**概要**: `ecs/dispatch.rs` として独立モジュールを作成

**構成**:
```
ecs/
├── dispatch.rs          ← 新規（ディスパッチシステム）
│   ├── MouseEventHandler
│   ├── EventContext<E>
│   ├── Phase
│   └── dispatch_mouse_events()
├── mouse.rs             ← 既存（状態管理のみ）
└── mod.rs               ← 追加: pub mod dispatch;
```

**Trade-offs**:
- ✅ 責務明確化（状態管理 vs ディスパッチ）
- ✅ 既存コードへの影響最小
- ✅ 汎用ディスパッチへの拡張容易
- ❌ 新規ファイル追加

**評価**: **推奨**

### Option C: layout/event.rs としてレイアウトモジュールに配置

**概要**: `ecs/layout/event.rs` としてレイアウトモジュール配下に配置

**Trade-offs**:
- ✅ hit_test.rs と近い位置
- ❌ レイアウトとイベントは異なる責務
- ❌ 将来のキーボード/タイマーイベントとの整合性低

**評価**: 非推奨

---

## 4. Implementation Complexity & Risk

### 工数評価: M (3-7日)

| タスク | 見積もり |
|--------|----------|
| G1: MouseEventHandler | 0.5日 |
| G2: EventContext | 0.5日 |
| G3: dispatch_mouse_events | 2-3日 |
| G4: スケジュール統合 | 0.5日 |
| テスト | 1-2日 |
| **合計** | **4-6日** |

**根拠**:
- 既存パターン（SparseSet, 階層走査）の再利用
- 排他システムはbevy_ecsの標準パターン
- 複雑なアルゴリズムなし

### リスク評価: Low

| リスク要因 | 評価 | 軽減策 |
|-----------|------|--------|
| 世界借用競合 | Low | 排他システムで解決済み |
| パフォーマンス | Low | fnポインタCopyで最適化済み |
| 既存機能影響 | Low | 新規モジュールで分離 |

---

## 5. Recommendations for Design Phase

### 推奨アプローチ

**Option B: 新規 dispatch.rs モジュール作成**

### 設計フェーズでの検討事項

1. **ディスパッチ順序の詳細設計**
   - Target フェーズの扱い（最初に呼ぶ or バブリング中に呼ぶ）
   - 複数エンティティに MouseState がある場合の処理順

2. **エラーハンドリング**
   - ハンドラ内でのパニック対処
   - エンティティ削除時の安全性

3. **テスト戦略**
   - 単体テスト: ハンドラ呼び出し順序
   - 統合テスト: hit_test → dispatch → 状態変更

### Research Items

- なし（技術的未知要素なし）

---

## Appendix: 参照コード

### A. 親チェーン取得パターン
```rust
// graphics/systems.rs:1017-1020
let mut current = entity;
while let Ok(co) = child_of_query.get(current) {
    depth += 1;
    current = co.parent();
}
```

### B. SparseSet コンポーネント定義
```rust
// mouse.rs:98-99
#[derive(Component, Debug, Clone)]
#[component(storage = "SparseSet")]
pub struct MouseState { ... }
```

### C. スケジュール登録パターン
```rust
// world.rs:247-255
schedules.add_systems(
    Input,
    crate::ecs::mouse::process_mouse_buffers
        .after(crate::ecs::widget::bitmap_source::systems::drain_task_pool_commands),
);
```
