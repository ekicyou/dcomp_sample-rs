# Gap Analysis: event-dispatch

| 項目 | 内容 |
|------|------|
| **Feature** | event-dispatch |
| **Date** | 2025-12-03 |
| **Parent Spec** | wintf-P0-event-system |
| **Phase** | Gap Analysis Complete |

---

## 1. Executive Summary

- **スコープ**: イベント伝播機構（バブリング/キャプチャ）とハンドラディスパッチシステム
- **既存基盤**: hit_test → MouseState 付与まで実装済み（event-mouse-basic, event-hit-test）
- **主要ギャップ**: ハンドラコンポーネント、バブリング経路収集、ディスパッチシステムが未実装
- **推奨アプローチ**: Option B（新規コンポーネント作成）- 既存 mouse.rs と分離し、新モジュール `dispatch.rs` を作成
- **リスク**: Low - 既存パターン（排他システム、SparseSet）が確立済み

---

## 2. Current State Investigation

### 2.1 Domain-Related Assets

| カテゴリ | ファイル/モジュール | 関連度 |
|---------|---------------------|--------|
| **マウス状態** | `ecs/mouse.rs` | 高 - MouseState, MouseLeave 定義 |
| **ハンドラ** | `ecs/window_proc/handlers.rs` | 中 - WM_MOUSEMOVE でhit_test呼び出し |
| **階層システム** | `ecs/common/tree_iter.rs` | 高 - DepthFirstReversePostOrder（ヒットテスト用） |
| **階層コンポーネント** | bevy_ecs `ChildOf`/`Children` | 高 - 親子関係 |
| **スケジュール** | `ecs/world.rs` | 高 - Input/Update スケジュール定義 |
| **レイアウト** | `ecs/layout/hit_test.rs` | 中 - hit_test_in_window API |
| **排他システム例** | `ecs/window_system.rs` | 中 - `fn create_windows(world: &mut World)` |

### 2.2 Existing Patterns

#### ストレージ戦略
```rust
// SparseSet: 頻繁な挿入/削除、少数エンティティ向け
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct MouseState { ... }
```

#### 排他システム
```rust
// &mut World を受け取る排他システム
pub fn create_windows(world: &mut World) {
    // world.entity_mut(), world.get::<T>() 等を直接使用
}
```

#### 階層走査
```rust
// 子→親への走査には ChildOf.parent() を使用
let parent_entity = world.get::<ChildOf>(entity)?.parent();
```

#### スケジュール統合
```rust
schedules.add_systems(
    Input,
    my_system.after(crate::ecs::mouse::process_mouse_buffers),
);
```

### 2.3 Integration Surfaces

| 統合ポイント | 詳細 |
|-------------|------|
| **MouseState** | 既存コンポーネントをEventContextに含める |
| **Input スケジュール** | `process_mouse_buffers` の後に dispatch 実行 |
| **ChildOf/Children** | バブリング経路の親子走査に使用 |
| **hit_test_in_window** | ヒット対象エンティティの特定（既に handlers.rs で呼び出し中） |

---

## 3. Requirements Feasibility Analysis

### 3.1 Requirements to Asset Mapping

| Requirement | 必要な実装 | 既存アセット | ギャップ |
|-------------|-----------|-------------|---------|
| R1: バブリング | 子→親経路収集 | `ChildOf.parent()` | 経路収集関数 (Missing) |
| R2: キャプチャ | 親→子経路収集 | `DepthFirstReversePostOrder` | P2 (将来) |
| R3: ハンドラコンポーネント | `MouseEventHandler` | なし | 新規作成 (Missing) |
| R4: EventContext | コンテキスト構造体 | `MouseState` 参照可 | 新規作成 (Missing) |
| R5: ディスパッチシステム | 排他システム | パターン確立済み | 新規作成 (Missing) |
| R6: ECS統合 | スケジュール登録 | `world.rs` | 統合作業 (Constraint) |
| R7: SparseSet | ストレージ戦略 | パターン確立済み | なし |
| R8: 汎用ディスパッチ | ジェネリック設計 | なし | 新規作成 (Missing) |
| R9: イベント履歴 | リングバッファ | なし | P2 (将来) |

### 3.2 Technical Gaps

#### G1: バブリング経路収集関数（Missing）
```rust
// 必要な関数（未実装）
fn collect_bubble_path(world: &World, target: Entity) -> Vec<Entity> {
    let mut path = vec![target];
    let mut current = target;
    while let Some(child_of) = world.get::<ChildOf>(current) {
        current = child_of.parent();
        path.push(current);
    }
    path
}
```

#### G2: MouseEventHandler コンポーネント（Missing）
```rust
#[derive(Component, Clone, Copy)]
#[component(storage = "SparseSet")]
pub struct MouseEventHandler {
    pub handler: fn(&mut World, Entity, &EventContext<MouseState>) -> bool,
}
```

#### G3: ディスパッチシステム（Missing）
```rust
// 排他システムとして実装
pub fn dispatch_mouse_events(world: &mut World) {
    // 1. MouseState を持つエンティティを収集
    // 2. バブリング経路を構築
    // 3. ハンドラを収集（fnポインタはCopy）
    // 4. ハンドラを順次実行
}
```

### 3.3 Constraints

| 制約 | 詳細 |
|------|------|
| **フレーム遅延禁止** | 2パス方式で同一フレーム内完結必須 |
| **排他システム必須** | ハンドラが `&mut World` を必要とするため |
| **ハンドラ型制約** | `fn` ポインタのみ（クロージャ不可）、状態はコンポーネントに |

---

## 4. Implementation Approach Options

### Option A: 既存 mouse.rs を拡張

**概要**: `ecs/mouse.rs` に `MouseEventHandler` と dispatch ロジックを追加

**Trade-offs**:
- ✅ ファイル数増加なし
- ❌ mouse.rs が肥大化（既に 824 行）
- ❌ 責務の混在（状態管理 + 伝播ロジック）

**推奨度**: ❌ 非推奨

---

### Option B: 新規 dispatch.rs モジュール作成 ⭐推奨

**概要**: `ecs/dispatch.rs` を新規作成し、伝播ロジックを分離

**構成**:
```
ecs/
├── mouse.rs         # MouseState, MouseLeave（既存）
├── dispatch.rs      # MouseEventHandler, EventContext, dispatch_mouse_events（新規）
└── dispatch/        # 将来の拡張用（汎用ディスパッチ等）
    ├── mod.rs
    ├── mouse.rs     # マウス固有
    └── context.rs   # EventContext<E>
```

**Trade-offs**:
- ✅ 責務の明確な分離
- ✅ 将来の拡張（キーボード、タイマー）に対応しやすい
- ✅ テストが容易
- ❌ 新規ファイル追加

**推奨度**: ⭐ 推奨

---

### Option C: ハイブリッド（フェーズ分割）

**概要**: Phase 1 で mouse.rs 拡張、Phase 2 で分離リファクタ

**Trade-offs**:
- ✅ 初期実装が速い
- ❌ リファクタリングコスト
- ❌ 技術的負債

**推奨度**: △ 条件付き

---

## 5. Implementation Complexity & Risk

### Effort: M (3-7 days)

**根拠**:
- 新規コンポーネント/型: 3つ（MouseEventHandler, EventContext, Phase）
- 新規関数: 2-3つ（バブリング経路収集、ディスパッチシステム）
- スケジュール統合: 既存パターンあり
- テスト: 単体テスト + 統合テスト

### Risk: Low

**根拠**:
- 排他システムパターンが確立済み
- SparseSet ストレージパターンが確立済み
- ChildOf/Children 階層走査が実装済み
- 複雑な外部統合なし

---

## 6. Design Phase Recommendations

### 6.1 Preferred Approach

**Option B: 新規 dispatch.rs モジュール作成**

### 6.2 Key Design Decisions

1. **モジュール構成**
   - 初期: 単一ファイル `ecs/dispatch.rs`
   - 拡張時: `ecs/dispatch/` ディレクトリ化

2. **型定義**
   - `EventContext<E>` をジェネリックに（将来拡張対応）
   - `MouseEventHandler` は具象型として開始

3. **システム登録**
   - Input スケジュール、`process_mouse_buffers` の後

### 6.3 Research Items for Design Phase

| 項目 | 詳細 |
|------|------|
| なし | 主要な技術的不明点はすべて議論済み |

---

## 7. Appendix: Code References

### A. 親エンティティ走査パターン
```rust
// ecs/graphics/systems.rs:1023
current = co.parent();

// ecs/layout/systems.rs:166
let parent_entity = parent_ref.parent();
```

### B. 排他システムパターン
```rust
// ecs/window_system.rs:22
pub fn create_windows(world: &mut World) {
    // ...
}
```

### C. スケジュール登録パターン
```rust
// ecs/world.rs:254
schedules.add_systems(
    Input,
    crate::ecs::mouse::process_mouse_buffers
        .after(crate::ecs::widget::bitmap_source::systems::drain_task_pool_commands),
);
```

---

## 8. Next Steps

1. **設計フェーズ開始**: `/kiro-spec-design event-dispatch` を実行
2. **設計ドキュメント**: dispatch.rs のモジュール構成、API 詳細、統合方法を記述
3. **タスク分割**: 実装タスクを定義
