# Research Log: Visual Tree Synchronization

## Discovery Process (Light Discovery - Extension Type)

### 1. 既存APIパターンの検証

#### 1.1 DCompositionVisualExt トレイト
**ファイル**: `crates/wintf/src/com/dcomp.rs`

```rust
pub trait DCompositionVisualExt {
    fn add_visual<P0, P1>(&self, visual: P0, insertabove: bool, referencevisual: P1) -> Result<()>
    where
        P0: Param<IDCompositionVisual>,
        P1: Param<IDCompositionVisual>;
    // ... 他のメソッド
}
```

**発見事項**:
- `add_visual` は既に実装済み
- `remove_visual` は未実装（R1で追加が必要）
- `insertabove` パラメータでZ-order制御が可能
- `referencevisual` が `None` の場合、末尾または先頭に追加

#### 1.2 Z-order制御パターン
**DirectComposition API挙動**:
- `insertabove = true, referencevisual = None` → 最前面に追加
- `insertabove = false, referencevisual = None` → 最背面に追加
- `insertabove = true, referencevisual = Some(ref)` → refの前面に追加
- `insertabove = false, referencevisual = Some(ref)` → refの背面に追加

**設計決定**: ECS Childrenの順序（インデックス0が最背面）とDirectCompositionのZ-orderを一致させる

### 2. コンポーネントフック実装パターン

#### 2.1 既存のon_addフック
**ファイル**: `crates/wintf/src/ecs/layout/arrangement.rs`

```rust
#[derive(Component)]
#[component(on_add = on_arrangement_add)]
pub struct Arrangement { ... }

fn on_arrangement_add(
    mut world: bevy_ecs::world::DeferredWorld,
    hook: bevy_ecs::lifecycle::HookContext,
) {
    world
        .commands()
        .entity(hook.entity)
        .insert((GlobalArrangement::default(), ArrangementTreeChanged));
}
```

#### 2.2 既存のon_removeフック
**ファイル**: `crates/wintf/src/ecs/widget/text/label.rs`

```rust
#[derive(Component)]
#[component(storage = "SparseSet", on_remove = on_label_remove)]
pub struct Label { ... }
```

**発見事項**:
- `DeferredWorld` + `HookContext` パターンが標準
- `commands()` 経由でエンティティ操作
- `SparseSet` ストレージは頻繁な追加/削除に最適

### 3. 階層管理パターン

#### 3.1 ChildOf/Childrenの使用
**ファイル**: `crates/wintf/src/ecs/window.rs`

```rust
use bevy_ecs::hierarchy::ChildOf;

// on_window_add内
if !entity_mut.contains::<ChildOf>() {
    entity_mut.insert(ChildOf(root));
}
```

#### 3.2 階層トラバーサル
**ファイル**: `crates/wintf/src/ecs/layout/systems.rs`

```rust
use bevy_ecs::hierarchy::{ChildOf, Children};

// ルートエンティティの検出
Query<..., Without<ChildOf>>

// 子エンティティのイテレーション
Query<(Entity, ..., &Children), ...>
```

### 4. グラフィックスリソース管理

#### 4.1 Visual作成パターン
**ファイル**: `crates/wintf/src/ecs/graphics/visual_manager.rs`

```rust
fn create_visual_resources(
    commands: &mut Commands,
    entity: Entity,
    visual: &Visual,
    dcomp: &IDCompositionDevice3,
) {
    let visual_res = dcomp.create_visual();
    match visual_res {
        Ok(v3) => {
            commands.entity(entity).insert(VisualGraphics::new(v3.clone()));
            // Surface作成...
        }
        ...
    }
}
```

**問題点**: 現在はVisual追加時にSurfaceも即座に作成している（R5の遅延作成と矛盾）

#### 4.2 SurfaceGraphicsのフック
**ファイル**: `crates/wintf/src/ecs/graphics/components.rs`

```rust
#[component(on_add = on_surface_graphics_changed, on_replace = on_surface_graphics_changed)]
pub struct SurfaceGraphics { ... }
```

### 5. 変換とスケール

#### 5.1 Matrix3x2変換パターン
**ファイル**: `crates/wintf/src/ecs/layout/arrangement.rs`

```rust
impl From<Arrangement> for Matrix3x2 {
    fn from(arr: Arrangement) -> Self {
        let scale: Matrix3x2 = arr.scale.into();
        let translation: Matrix3x2 = arr.offset.into();
        translation * scale  // 正しい順序
    }
}
```

#### 5.2 スケール抽出
軸平行変換の前提条件下では:
- `M11` = X軸スケール
- `M22` = Y軸スケール
- 回転が含まれる場合は無効

### 6. スケジュール構成

#### 6.1 現在のスケジュール順序
```
Input → Update → PreLayout → Layout → PostLayout → 
UISetup → Draw → PreRenderSurface → RenderSurface → 
Composition → CommitComposition
```

**R4a要件**: `draw_labels` を `PreLayout` スケジュールで実行

## 設計上の決定事項

### D1: Visual階層同期のタイミング
- **決定**: `Composition` スケジュールで同期
- **理由**: レイアウト完了後、コミット前に実行

### D2: Z-order同期戦略
- **決定**: `Children` の順序に従い、index 0を最背面とする
- **実装**: `insertabove = false, referencevisual = None` で順次追加

### D3: Surface作成の分離
- **決定**: Visual追加時にSurfaceを作成しない（R5で遅延作成）
- **影響**: `visual_resource_management_system` の修正が必要

### D4: remove_visual API追加
- **決定**: `DCompositionVisualExt` トレイトに `remove_visual` を追加
- **シグネチャ**: `fn remove_visual<P0>(&self, visual: P0) -> Result<()>`

## 未解決事項

### U1: 既存Surfaceの即時作成ロジック
`visual_manager.rs` の `create_visual_resources` が Surface を即座に作成している。
R5（遅延Surface作成）との整合性を取るため、この動作を変更する必要がある。

### U2: ルートVisualへの接続
`WindowGraphics.root_visual` への最初のVisual追加タイミングと方法を明確化する必要がある。

### U3: 階層変更の検出
`ChildOf` コンポーネントの追加/削除/変更を効率的に検出する方法。
`RemovedComponents<ChildOf>` + `Changed<ChildOf>` の組み合わせで対応可能。
