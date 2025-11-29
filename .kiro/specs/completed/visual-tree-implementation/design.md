# Design Specification: visual-tree-implementation

**Feature ID**: `visual-tree-implementation`  
**Created**: 2025-11-17  
**Status**: Design Phase  
**Language**: 日本語

---

## 1. Executive Summary

### 目的

ECSのEntity階層（bevy_ecs::hierarchy::{ChildOf, Children}）を用いたビジュアルツリー構造の実装。Arrangementコンポーネントによる座標変換システムと、階層的Surfaceレンダリング（WindowのSurfaceに全子孫を深さ優先描画）を実現する。

### スコープ

**今回実装**:
- bevy_ecs::hierarchy::{ChildOf, Children}のwintf統合
- Arrangement/GlobalArrangement座標変換システム（layout.rs拡張）
- arrangement.rsの新規作成（伝播システム: sync, mark, propagate）
- 階層的Surfaceレンダリング（render_surface拡張、Query::iter_descendants_depth_first使用）
- Rectangle/Label構造変更（x/yフィールド削除、Arrangement移行）
- simple_window.rsサンプル更新（4階層、6 Rectangle + 2 Label）

**将来実装（今回スコープ外）**:
- IDCompositionVisual階層構築（AddVisual）
- 子Widget独自Visual+Surface作成（アニメーション/スクロール時）
- Visual階層に基づく部分更新
- taffyレイアウトエンジン統合（BoxComputedLayout → Arrangement変換）

### 主要な技術決定事項

1. **Entity階層**: bevy_ecs::hierarchy::{ChildOf, Children}を採用（標準機能）
2. **Arrangement伝播**: tree_system.rsジェネリック関数パターンを適用
3. **深さ優先レンダリング**: Query::iter_descendants_depth_first::<Children>()メソッド使用
4. **Transform合成**: render_surface内で`SetTransform(GlobalArrangement)`を毎回計算（キャッシュなし）
5. **座標系統一**: Rectangle/Labelのx/y廃止、Arrangementに一本化

---

## 2. System Architecture

### 2.1 コンポーネント構成

```
┌─────────────────────────────────────────────────┐
│          bevy_ecs::hierarchy                    │
│  ChildOf<E> (子→親)  /  Children<E> (親→子)    │
└─────────────────────────────────────────────────┘
                      ↓ 使用
┌─────────────────────────────────────────────────┐
│               layout.rs (拡張)                  │
│  Offset, LayoutScale, Arrangement,              │
│  GlobalArrangement, ArrangementTreeChanged      │
└─────────────────────────────────────────────────┘
                      ↓ 伝播
┌─────────────────────────────────────────────────┐
│          arrangement.rs (新規)                  │
│  sync_simple_arrangements,                      │
│  mark_dirty_arrangement_trees,                  │
│  propagate_global_arrangements                  │
└─────────────────────────────────────────────────┘
                      ↓ 使用
┌─────────────────────────────────────────────────┐
│       graphics/systems.rs (拡張)               │
│  render_surface (階層的描画)                    │
│   - Query::iter_descendants_depth_first         │
│   - SetTransform(GlobalArrangement)             │
└─────────────────────────────────────────────────┘
```

### 2.2 座標変換フロー

```
Arrangement (ローカル: Offset + LayoutScale)
  ↓ (親から伝播)
GlobalArrangement (累積レイアウト変換: Matrix3x2)
  ↓ (描画時)
Transform (視覚効果: 回転、傾斜等、今回は未使用)
  ↓
最終変換 = GlobalArrangement * Transform
  ↓
SetTransform(最終変換) → Surface描画
```

**重要な区別**:
- **Arrangement** (layout.rs): レイアウト層の座標変換、親から子へ累積伝播
- **Transform** (transform.rs): 視覚効果層の変換（WPF RenderTransform相当）、累積伝播なし（今回は未使用）

### 2.3 Visual/Surface所有ルール

**今回のスコープ**:
- Window EntityのみがVisualGraphics + SurfaceGraphicsを所有
- 全Widget描画はWindowのSurfaceに集約
- Rectangle/Label等の子WidgetはVisual/Surfaceを持たない

**将来の拡張**:
- 子WidgetもVisualGraphics + SurfaceGraphicsを作成（条件: アニメーション/スクロール/その他）
- IDCompositionVisual階層を構築（AddVisual）
- Visual階層に基づく部分更新

---

## 3. Component Design

### 3.1 layout.rs 拡張

#### 新規コンポーネント

```rust
// crates/wintf/src/ecs/layout.rs
use bevy_ecs::prelude::*;
use windows_numerics::Matrix3x2;

/// オフセット（親からの相対位置）
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Offset {
    pub x: f32,
    pub y: f32,
}

impl Default for Offset {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0 }
    }
}

/// レイアウトスケール（DPIスケール、ViewBox等）
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct LayoutScale {
    pub x: f32,
    pub y: f32,
}

impl Default for LayoutScale {
    fn default() -> Self {
        Self { x: 1.0, y: 1.0 }
    }
}

/// ローカルレイアウト配置（オフセット + スケール）
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct Arrangement {
    pub offset: Offset,
    pub scale: LayoutScale,
}

impl Default for Arrangement {
    fn default() -> Self {
        Self {
            offset: Offset::default(),
            scale: LayoutScale::default(),
        }
    }
}

/// グローバルレイアウト変換（親からの累積変換）
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct GlobalArrangement(pub Matrix3x2);

impl Default for GlobalArrangement {
    fn default() -> Self {
        Self(Matrix3x2::identity())
    }
}

/// Arrangementツリー変更マーカー（ダーティビット伝播用）
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ArrangementTreeChanged;
```

#### 変換トレイト実装

```rust
// crates/wintf/src/ecs/layout.rs
use bevy_ecs::prelude::*;
use windows_numerics::Matrix3x2;

/// OffsetからMatrix3x2への変換（平行移動）
impl From<Offset> for Matrix3x2 {
    fn from(offset: Offset) -> Self {
        Matrix3x2::translation(offset.x, offset.y)
    }
}

/// LayoutScaleからMatrix3x2への変換（スケール）
impl From<LayoutScale> for Matrix3x2 {
    fn from(scale: LayoutScale) -> Self {
        Matrix3x2::scale(scale.x, scale.y)
    }
}

/// ArrangementからMatrix3x2への変換（スケール + 平行移動）
impl From<Arrangement> for Matrix3x2 {
    fn from(arr: Arrangement) -> Self {
        let scale: Matrix3x2 = arr.scale.into();
        let translation: Matrix3x2 = arr.offset.into();
        scale * translation
    }
}

/// ArrangementからGlobalArrangementへの変換
impl From<Arrangement> for GlobalArrangement {
    fn from(arrangement: Arrangement) -> Self {
        Self(arrangement.into())
    }
}

/// GlobalArrangement同士の乗算（親の累積変換 * 子のローカル変換）
impl std::ops::Mul<Arrangement> for GlobalArrangement {
    type Output = GlobalArrangement;

    fn mul(self, rhs: Arrangement) -> Self::Output {
        let child_matrix: Matrix3x2 = rhs.into();
        GlobalArrangement(self.0 * child_matrix)
    }
}
```

### 3.2 arrangement.rs 新規作成

```rust
// crates/wintf/src/ecs/arrangement.rs

use bevy_ecs::prelude::*;
use bevy_ecs::hierarchy::{ChildOf, Children};
use bevy_ecs::relationship::RelationshipTarget;
use crate::ecs::layout::{Arrangement, GlobalArrangement, ArrangementTreeChanged};
use crate::ecs::tree_system::{sync_simple_transforms, mark_dirty_trees, propagate_parent_transforms};

/// 階層に属していないEntity（ルートWindow）のGlobalArrangementを更新
pub fn sync_simple_arrangements(
    query: ParamSet<(
        Query<(&Arrangement, &mut GlobalArrangement), (Without<ChildOf>, Without<Children>)>,
        Query<(Entity, &Arrangement, &mut GlobalArrangement, &mut ArrangementTreeChanged), Without<ChildOf>>,
    )>,
    orphaned: RemovedComponents<ChildOf>,
) {
    sync_simple_transforms::<Arrangement, GlobalArrangement, ArrangementTreeChanged>(query, orphaned);
}

/// 「ダーティビット」を階層の祖先に向かって伝播
pub fn mark_dirty_arrangement_trees(
    changed: Query<Entity, Or<(Changed<Arrangement>, Changed<ChildOf>, Added<GlobalArrangement>)>>,
    orphaned: RemovedComponents<ChildOf>,
    transforms: Query<(Option<&ChildOf>, &mut ArrangementTreeChanged)>,
) {
    mark_dirty_trees::<Arrangement, GlobalArrangement, ArrangementTreeChanged>(changed, orphaned, transforms);
}

/// 親から子へGlobalArrangementを伝播
pub fn propagate_global_arrangements(
    queue: Local<WorkQueue>,
    roots: Query<(Entity, Ref<Arrangement>, &mut GlobalArrangement, &Children), (Without<ChildOf>, Changed<ArrangementTreeChanged>)>,
    nodes: NodeQuery<Arrangement, GlobalArrangement, ArrangementTreeChanged>,
) {
    propagate_parent_transforms::<Arrangement, GlobalArrangement, ArrangementTreeChanged>(queue, roots, nodes);
}
```

**ポイント**:
- tree_system.rsのジェネリック関数を具体的な型パラメータでラップ
- `L = Arrangement`, `G = GlobalArrangement`, `M = ArrangementTreeChanged`
- bevy_ecs::hierarchy::{ChildOf, Children}を直接使用

### 3.3 Widget構造変更

#### Rectangle/Labelのx/y削除

```rust
// crates/wintf/src/ecs/widget/shapes/rectangle.rs (変更前)
#[derive(Component, Debug, Clone, Copy)]
pub struct Rectangle {
    pub x: f32,      // 削除
    pub y: f32,      // 削除
    pub width: f32,
    pub height: f32,
    pub color: u32,
}

// crates/wintf/src/ecs/widget/shapes/rectangle.rs (変更後)
#[derive(Component, Debug, Clone, Copy)]
pub struct Rectangle {
    pub width: f32,
    pub height: f32,
    pub color: u32,
}
```

```rust
// crates/wintf/src/ecs/widget/text/label.rs (変更前)
#[derive(Component, Debug, Clone)]
pub struct Label {
    pub text: String,
    pub font_family: String,
    pub font_size: f32,
    pub color: u32,
    pub x: f32,      // 削除
    pub y: f32,      // 削除
}

// crates/wintf/src/ecs/widget/text/label.rs (変更後)
#[derive(Component, Debug, Clone)]
pub struct Label {
    pub text: String,
    pub font_family: String,
    pub font_size: f32,
    pub color: u32,
}
```

#### 描画システムの調整

```rust
// crates/wintf/src/ecs/widget/shapes/draw_rectangles.rs (変更前)
pub fn draw_rectangles(
    mut query: Query<(&Rectangle, &mut GraphicsCommandList)>,
) {
    for (rect, mut cmd_list) in query.iter_mut() {
        cmd_list.add_rectangle(rect.x, rect.y, rect.width, rect.height, rect.color);
        //                       ^^^^^^  ^^^^^^ (x/yは直接参照)
    }
}

// crates/wintf/src/ecs/widget/shapes/draw_rectangles.rs (変更後)
pub fn draw_rectangles(
    mut query: Query<(&Rectangle, &mut GraphicsCommandList)>,
) {
    for (rect, mut cmd_list) in query.iter_mut() {
        // 座標はGlobalArrangementでSetTransformされるため、原点(0,0)から描画
        cmd_list.add_rectangle(0.0, 0.0, rect.width, rect.height, rect.color);
        //                       ^^^  ^^^ (常に原点から描画)
    }
}
```

**重要**: 座標変換はrender_surfaceでSetTransform(GlobalArrangement)により適用されるため、Widget描画は常に原点(0,0)基準で行う。

---

## 4. System Design

### 4.1 階層的Surfaceレンダリング (render_surface拡張)

#### 現在の実装

```rust
// crates/wintf/src/ecs/graphics/systems.rs (現在)
pub fn render_surface(
    mut query: Query<(&SurfaceGraphics, &GraphicsCommandList), Changed<GraphicsCommandList>>,
) {
    for (surface, cmd_list) in query.iter() {
        if let Some(s) = surface.surface() {
            // 単一EntityのGraphicsCommandListのみ描画
            s.begin_draw();
            s.draw(cmd_list);
            s.end_draw();
            s.commit();
        }
    }
}
```

#### 階層的描画への拡張

```rust
// crates/wintf/src/ecs/graphics/systems.rs (拡張後)
use bevy_ecs::hierarchy::Children;
use bevy_ecs::query::QueryIter;
use crate::ecs::layout::GlobalArrangement;

pub fn render_surface(
    windows: Query<(Entity, &SurfaceGraphics), With<Window>>,
    widgets: Query<(Option<&GlobalArrangement>, Option<&GraphicsCommandList>, Option<&Children>)>,
) {
    for (window_entity, surface) in windows.iter() {
        if let Some(surf) = surface.surface() {
            surf.begin_draw();
            
            // Window自身を描画（GlobalArrangementは恒等変換）
            if let Ok((global_arr, cmd_list, _)) = widgets.get(window_entity) {
                if let Some(arr) = global_arr {
                    surf.set_transform(&arr.0);
                }
                if let Some(list) = cmd_list {
                    surf.draw(list);
                }
            }
            
            // 全子孫を深さ優先で描画
            for descendant in widgets.iter_descendants_depth_first::<Children>(window_entity) {
                if let Ok((global_arr, cmd_list, _)) = widgets.get(descendant) {
                    if let Some(arr) = global_arr {
                        surf.set_transform(&arr.0);
                    }
                    if let Some(list) = cmd_list {
                        surf.draw(list);
                    }
                }
            }
            
            surf.end_draw();
            surf.commit();
        }
    }
}
```

**ポイント**:
- `Query::iter_descendants_depth_first::<Children>(window_entity)` - 深さ優先探索
- `Children`は`RelationshipTarget`を実装しているため、このメソッドが使用可能
- 各子孫描画前に`SetTransform(GlobalArrangement)`を適用
- 描画後のリセット不要（次の描画でoverrideされる）
- Window自身 → 第1子 → 第1子の子孫（深さ優先） → 第2子 → ... の順序

### 4.2 Window初期化システム

#### Window Entity自動セットアップ

```rust
// crates/wintf/src/ecs/graphics/systems.rs (既存、変更なし)
pub fn init_window_visual(
    mut commands: Commands,
    graphics_core: Res<GraphicsCore>,
    query: Query<(Entity, &WindowGraphics), Added<WindowGraphics>>,
) {
    for (entity, window_graphics) in query.iter() {
        if let Some(device) = graphics_core.dcomp_device() {
            match device.create_visual() {
                Ok(visual) => {
                    if let Some(target) = window_graphics.target() {
                        let _ = target.set_root(&visual);
                    }
                    commands.entity(entity).insert(VisualGraphics::new(visual));
                }
                Err(e) => {
                    eprintln!("Failed to create visual: {:?}", e);
                }
            }
        }
    }
}
```

#### Arrangement自動追加

```rust
// crates/wintf/src/ecs/window_system.rs (新規システム追加)
use crate::ecs::layout::{Arrangement, GlobalArrangement, ArrangementTreeChanged};
use crate::ecs::window::Window;

pub fn init_window_arrangement(
    mut commands: Commands,
    query: Query<Entity, (With<Window>, Without<Arrangement>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert((
            Arrangement::default(), // offset: (0, 0), scale: (1, 1)
            GlobalArrangement::default(), // 恒等変換
            ArrangementTreeChanged,
        ));
    }
}
```

### 4.3 スケジュール実行順序

```rust
// crates/wintf/src/ecs/world.rs

pub fn create_schedules() -> Schedules {
    let mut schedules = Schedules::new();
    
    // PostLayout: Graphics初期化
    schedules.add_systems(PostLayout, (
        init_graphics_core,
        cleanup_command_list_on_reinit,
        init_window_graphics,
        init_window_visual,
        init_window_surface,
        init_window_arrangement, // 新規追加
    ).chain());
    
    // Draw: Widget描画、Arrangement伝播
    schedules.add_systems(Draw, (
        cleanup_graphics_needs_init,
        draw_rectangles, // 変更: x/y削除、原点描画
        draw_labels,     // 変更: x/y削除、原点描画
        sync_simple_arrangements,        // 新規追加
        mark_dirty_arrangement_trees,    // 新規追加
        propagate_global_arrangements,   // 新規追加
    ).chain());
    
    // RenderSurface: 階層的描画
    schedules.add_systems(RenderSurface, (
        render_surface, // 拡張: 階層的描画
    ));
    
    // CommitComposition: DirectComposition確定
    schedules.add_systems(CommitComposition, (
        commit_composition,
    ));
    
    schedules
}
```

**実行順序の理由**:
1. **PostLayout**: Graphics初期化、Window Arrangement初期化
2. **Draw**: Widget描画（原点基準）、Arrangement伝播（親→子）
3. **RenderSurface**: 階層的描画（GlobalArrangement適用）
4. **CommitComposition**: DirectComposition確定

---

## 5. Data Flow

### 5.1 Entity階層の構築

```
アプリケーション開発者
  ↓ (Entity作成、ChildOf設定)
Window Entity (ルート)
  ├─ Rectangle1 Entity
  │    ├─ Rectangle1-1 Entity
  │    │    └─ Label1 Entity
  │    └─ Rectangle1-2 Entity
  │         └─ Rectangle1-2-1 Entity
  │              └─ Label2 Entity
  ↓ (bevy_ecs::hierarchyが自動管理)
Children/ChildOfコンポーネント自動更新
```

**サンプルコード**:
```rust
// examples/simple_window.rs
let window_entity = commands.spawn((
    Window { /* ... */ },
    Arrangement::default(), // 自動追加されるが明示も可能
)).id();

let rect1 = commands.spawn((
    Rectangle { width: 200.0, height: 150.0, color: 0x0000FF },
    Arrangement { offset: Offset { x: 20.0, y: 20.0 }, scale: LayoutScale::default() },
    ChildOf::<Entity>::new(window_entity),
)).id();

let rect1_1 = commands.spawn((
    Rectangle { width: 80.0, height: 60.0, color: 0x00FF00 },
    Arrangement { offset: Offset { x: 10.0, y: 10.0 }, scale: LayoutScale::default() },
    ChildOf::<Entity>::new(rect1),
)).id();

// Label1: Rectangle1-1の子
commands.spawn((
    Label { text: "Hello".to_string(), font_family: "MS Gothic".to_string(), font_size: 16.0, color: 0xFF0000 },
    Arrangement { offset: Offset { x: 5.0, y: 5.0 }, scale: LayoutScale::default() },
    ChildOf::<Entity>::new(rect1_1),
));
```

### 5.2 Arrangement伝播フロー

```
sync_simple_arrangements
  ↓ (ルートEntityのGlobalArrangement更新)
mark_dirty_arrangement_trees
  ↓ (変更されたEntityから親方向にArrangementTreeChangedマーカー伝播)
propagate_global_arrangements
  ↓ (親から子へGlobalArrangement伝播)
Widget Entity (GlobalArrangement確定)
```

**伝播計算例**:
```
Window: Arrangement { offset: (0, 0), scale: (1, 1) }
  → GlobalArrangement: Identity

Rectangle1: Arrangement { offset: (20, 20), scale: (1, 1) }
  → GlobalArrangement: Window.GlobalArrangement * Rectangle1.Arrangement
  → Matrix3x2 { M31: 20, M32: 20, ... }

Rectangle1-1: Arrangement { offset: (10, 10), scale: (1, 1) }
  → GlobalArrangement: Rectangle1.GlobalArrangement * Rectangle1-1.Arrangement
  → Matrix3x2 { M31: 30, M32: 30, ... } (累積: 20+10, 20+10)

Label1: Arrangement { offset: (5, 5), scale: (1, 1) }
  → GlobalArrangement: Rectangle1-1.GlobalArrangement * Label1.Arrangement
  → Matrix3x2 { M31: 35, M32: 35, ... } (累積: 20+10+5, 20+10+5)
```

### 5.3 階層的描画フロー

```
render_surface
  ↓ (Window Surfaceに対して)
1. surf.begin_draw()
  ↓
2. Window自身を描画
   - surf.set_transform(Window.GlobalArrangement)
   - surf.draw(Window.GraphicsCommandList)
  ↓
3. 全子孫を深さ優先で描画
   for descendant in widgets.iter_descendants_depth_first::<Children>(window_entity)
     - surf.set_transform(descendant.GlobalArrangement)
     - surf.draw(descendant.GraphicsCommandList)
  ↓
4. surf.end_draw()
  ↓
5. surf.commit()
```

**描画順序例**:
```
Window Surface描画開始
  → Window (GlobalArrangement: Identity)
  → Rectangle1 (GlobalArrangement: (20, 20))
    → Rectangle1-1 (GlobalArrangement: (30, 30))
      → Label1 (GlobalArrangement: (35, 35))
    → Rectangle1-2 (GlobalArrangement: (30, 100))
      → Rectangle1-2-1 (GlobalArrangement: (40, 110))
        → Label2 (GlobalArrangement: (45, 115))
Window Surface描画終了
```

---

## 6. Error Handling

### 6.1 Visual作成エラー

```rust
// crates/wintf/src/ecs/graphics/systems.rs (既存パターン)
pub fn init_window_visual(
    mut commands: Commands,
    graphics_core: Res<GraphicsCore>,
    query: Query<(Entity, &WindowGraphics), Added<WindowGraphics>>,
) {
    for (entity, window_graphics) in query.iter() {
        if let Some(device) = graphics_core.dcomp_device() {
            match device.create_visual() {
                Ok(visual) => {
                    if let Some(target) = window_graphics.target() {
                        if let Err(e) = target.set_root(&visual) {
                            eprintln!("Failed to set root visual for entity {:?}: {:?}", entity, e);
                            continue; // 他のWindowの処理を継続
                        }
                    }
                    commands.entity(entity).insert(VisualGraphics::new(visual));
                }
                Err(e) => {
                    eprintln!("Failed to create visual for entity {:?}: {:?}", entity, e);
                    // 該当Window Entityをスキップ、他のWindowの処理を継続
                }
            }
        }
    }
}
```

### 6.2 Arrangement伝播エラー

```rust
// crates/wintf/src/ecs/arrangement.rs (エラーハンドリング不要)
// tree_system.rsジェネリック関数内でエラーハンドリングは実装済み
// 親Entityが存在しない場合でもpanicせず、警告ログ出力（将来実装）
```

### 6.3 階層的描画エラー

```rust
// crates/wintf/src/ecs/graphics/systems.rs (拡張後)
pub fn render_surface(
    windows: Query<(Entity, &SurfaceGraphics), With<Window>>,
    widgets: Query<(Option<&GlobalArrangement>, Option<&GraphicsCommandList>, Option<&Children>)>,
) {
    for (window_entity, surface) in windows.iter() {
        if let Some(surf) = surface.surface() {
            if let Err(e) = surf.begin_draw() {
                eprintln!("Failed to begin draw for window {:?}: {:?}", window_entity, e);
                continue; // 他のWindowの処理を継続
            }
            
            // Window自身を描画
            if let Ok((global_arr, cmd_list, _)) = widgets.get(window_entity) {
                if let Some(arr) = global_arr {
                    if let Err(e) = surf.set_transform(&arr.0) {
                        eprintln!("Failed to set transform for window {:?}: {:?}", window_entity, e);
                    }
                }
                if let Some(list) = cmd_list {
                    if let Err(e) = surf.draw(list) {
                        eprintln!("Failed to draw window {:?}: {:?}", window_entity, e);
                    }
                }
            }
            
            // 全子孫を深さ優先で描画
            for descendant in widgets.iter_descendants_depth_first::<Children>(window_entity) {
                if let Ok((global_arr, cmd_list, _)) = widgets.get(descendant) {
                    if let Some(arr) = global_arr {
                        let _ = surf.set_transform(&arr.0); // エラー時は該当Widgetスキップ
                    }
                    if let Some(list) = cmd_list {
                        let _ = surf.draw(list); // エラー時は該当Widgetスキップ
                    }
                }
            }
            
            if let Err(e) = surf.end_draw() {
                eprintln!("Failed to end draw for window {:?}: {:?}", window_entity, e);
            }
            
            if let Err(e) = surf.commit() {
                eprintln!("Failed to commit for window {:?}: {:?}", window_entity, e);
            }
        }
    }
}
```

---

## 7. Performance Considerations

### 7.1 Arrangement伝播の最適化

**tree_system.rsの最適化機能**:
- **並列処理**: `par_iter_mut()`でルートEntityを並列処理
- **静的シーン最適化**: `!tree.is_changed() && !parent_global.is_changed()`でスキップ
- **ダーティビット伝播**: 変更されたEntityから親方向にマーカー伝播、不要な計算を削減

**適用例**:
```rust
// tree_system.rs (既存実装、そのまま利用)
pub fn propagate_parent_transforms<L, G, M>(
    mut queue: Local<WorkQueue>,
    roots: Query<(Entity, Ref<L>, &mut G, &Children), (Without<ChildOf>, Changed<M>)>,
    mut nodes: NodeQuery<L, G, M>,
) {
    queue.par_iter_mut().for_each(|root_entity| {
        let (local, mut global, children) = nodes.get_mut(root_entity).unwrap();
        
        // 静的シーン最適化: 変更なしならスキップ
        if !local.is_changed() && !global.is_changed() {
            return;
        }
        
        global.set_if_neq(local.into());
        
        for child in children.iter() {
            propagate_descendants_unchecked(child, &global, &mut nodes);
        }
    });
}
```

### 7.2 階層的描画の最適化

**描画スキップ条件**:
- `Changed<GraphicsCommandList>`フィルター: コマンドリスト未変更のEntityはスキップ
- `Changed<ArrangementTreeChanged>`フィルター: Arrangement未変更の場合は再描画不要

**実装例** (将来の最適化):
```rust
// crates/wintf/src/ecs/graphics/systems.rs (将来の最適化)
pub fn render_surface(
    windows: Query<(Entity, &SurfaceGraphics), Or<(Changed<ArrangementTreeChanged>, Changed<GraphicsCommandList>)>>,
    widgets: Query<(Option<&GlobalArrangement>, Option<&GraphicsCommandList>, Option<&Children>)>,
) {
    // Changed検知により、変更があったWindowのみ再描画
}
```

### 7.3 パフォーマンス目標

**R10 パフォーマンス要件**:
1. 50個のRectangle/Label描画で60fps以上維持
2. 変更なしフレームではCommit以外のDirectComposition API呼び出しなし
3. Window Visual作成とSetRootを1フレームあたり1ms以内
4. COM参照カウント管理によりメモリリークなし
5. 将来の子Widget Visual実装時、階層構築を1フレームあたり5ms以内

**測定方法**:
- `std::time::Instant`でシステム実行時間を計測
- `eprintln!`でフレーム時間をログ出力
- 実装後にR10検証

---

## 8. Testing Strategy

### 8.1 単体テスト（Unit Tests）

#### Arrangement変換テスト

```rust
// crates/wintf/src/ecs/layout.rs
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_from() {
        let offset = Offset { x: 10.0, y: 20.0 };
        let matrix: Matrix3x2 = offset.into();
        assert_eq!(matrix.M31, 10.0);
        assert_eq!(matrix.M32, 20.0);
    }

    #[test]
    fn test_scale_from() {
        let scale = LayoutScale { x: 2.0, y: 3.0 };
        let matrix: Matrix3x2 = scale.into();
        assert_eq!(matrix.M11, 2.0);
        assert_eq!(matrix.M22, 3.0);
    }

    #[test]
    fn test_arrangement_from() {
        let arr = Arrangement {
            offset: Offset { x: 10.0, y: 20.0 },
            scale: LayoutScale { x: 2.0, y: 3.0 },
        };
        
        let matrix: Matrix3x2 = arr.into();
        assert_eq!(matrix.M11, 2.0);
        assert_eq!(matrix.M22, 3.0);
        assert_eq!(matrix.M31, 10.0);
        assert_eq!(matrix.M32, 20.0);
    }

    #[test]
    fn test_global_arrangement_mul() {
        let parent = GlobalArrangement(Matrix3x2::translation(10.0, 20.0));
        
        let child = Arrangement {
            offset: Offset { x: 5.0, y: 7.0 },
            scale: LayoutScale { x: 1.0, y: 1.0 },
        };
        
        let result = parent * child;
        assert_eq!(result.0.M31, 15.0); // 10 + 5
        assert_eq!(result.0.M32, 27.0); // 20 + 7
    }
}
```

#### ChildOf/Children統合テスト

```rust
// crates/wintf/tests/arrangement_test.rs (新規)
use bevy_ecs::prelude::*;
use bevy_ecs::hierarchy::{ChildOf, Children};
use wintf::ecs::layout::{Arrangement, GlobalArrangement, ArrangementTreeChanged};
use wintf::ecs::arrangement::{sync_simple_arrangements, mark_dirty_arrangement_trees, propagate_global_arrangements};

#[test]
fn test_arrangement_propagation() {
    let mut world = World::new();
    
    // Window Entity (ルート)
    let window = world.spawn((
        Arrangement::default(),
        GlobalArrangement::default(),
        ArrangementTreeChanged,
    )).id();
    
    // Rectangle1 Entity
    let rect1 = world.spawn((
        Arrangement { offset: Offset { x: 20.0, y: 20.0 }, scale: LayoutScale::default() },
        GlobalArrangement::default(),
        ArrangementTreeChanged,
        ChildOf::<Entity>::new(window),
    )).id();
    
    // システム実行
    let mut schedule = Schedule::default();
    schedule.add_systems((
        sync_simple_arrangements,
        mark_dirty_arrangement_trees,
        propagate_global_arrangements,
    ));
    schedule.run(&mut world);
    
    // 検証
    let rect1_global = world.get::<GlobalArrangement>(rect1).unwrap();
    assert_eq!(rect1_global.0.M31, 20.0);
    assert_eq!(rect1_global.0.M32, 20.0);
}
```

### 8.2 統合テスト（Integration Tests）

#### simple_window.rsサンプル実行

```bash
# サンプル実行
cargo run --example simple_window

# 期待される動作:
# - Windowが表示される
# - 6個のRectangle（青、緑、黄、紫）が階層的に描画される
# - 2個のLabel（赤「Hello」、白「World」）がRectangle上に描画される
# - 階層構造（最大4階層）が視覚的に確認できる
```

#### 描画順序検証

```rust
// tests/render_order_test.rs (将来実装)
// render_surfaceの描画順序をモックで検証
```

### 8.3 パフォーマンステスト

```rust
// tests/performance_test.rs (実装後)
use std::time::Instant;

#[test]
fn test_50_widgets_60fps() {
    // 50個のRectangle/Labelを作成
    // システム実行時間を計測
    // 1フレーム16.6ms以内（60fps）を検証
}
```

---

## 9. Migration Guide

### 9.1 Rectangle/Label使用箇所の移行

#### 変更前

```rust
// examples/simple_window.rs (変更前)
commands.spawn((
    Rectangle {
        x: 20.0,
        y: 20.0,
        width: 200.0,
        height: 150.0,
        color: 0x0000FF,
    },
));
```

#### 変更後

```rust
// examples/simple_window.rs (変更後)
commands.spawn((
    Rectangle {
        width: 200.0,
        height: 150.0,
        color: 0x0000FF,
    },
    Arrangement {
        offset: Offset { x: 20.0, y: 20.0 },
        scale: LayoutScale::default(),
    },
    ChildOf::<Entity>::new(parent_entity),
));
```

### 9.2 Entity階層の構築

```rust
// examples/simple_window.rs (階層構造のサンプル)
use bevy_ecs::hierarchy::ChildOf;
use wintf::ecs::layout::{Arrangement, Offset, LayoutScale};

let window_entity = commands.spawn((
    Window { /* ... */ },
    Arrangement::default(), // Window自身は(0, 0)、自動追加
)).id();

let rect1 = commands.spawn((
    Rectangle { width: 200.0, height: 150.0, color: 0x0000FF },
    Arrangement {
        offset: Offset { x: 20.0, y: 20.0 },
        scale: LayoutScale::default(),
    },
    ChildOf::<Entity>::new(window_entity),
)).id();

let rect1_1 = commands.spawn((
    Rectangle { width: 80.0, height: 60.0, color: 0x00FF00 },
    Arrangement {
        offset: Offset { x: 10.0, y: 10.0 },
        scale: LayoutScale::default(),
    },
    ChildOf::<Entity>::new(rect1),
)).id();

commands.spawn((
    Label {
        text: "Hello".to_string(),
        font_family: "MS Gothic".to_string(),
        font_size: 16.0,
        color: 0xFF0000,
    },
    Arrangement {
        offset: Offset { x: 5.0, y: 5.0 },
        scale: LayoutScale::default(),
    },
    ChildOf::<Entity>::new(rect1_1),
));
```

---

## 10. Future Enhancements

### 10.1 IDCompositionVisual階層構築

**目的**: 子Widget独自のVisual+Surfaceを作成し、DirectCompositionの階層的合成機能を活用。

**実装方針**:
- `build_visual_tree`システムで`AddVisual`を呼び出し、親子Visual関係を構築
- アニメーション/スクロール等の条件で子Widget EntityにもVisualGraphics + SurfaceGraphicsを追加
- Visual階層に基づく部分更新（変更されたVisualのみ再描画）

### 10.2 taffyレイアウトエンジン統合

**目的**: BoxComputedLayout → Arrangementの自動変換により、動的レイアウト計算を実現。

**実装方針**:
- `layout_to_arrangement`システムで`BoxComputedLayout`を`Arrangement`に変換
- taffyのFlexbox/Grid計算結果をArrangementに反映
- Arrangement伝播システムはそのまま利用

### 10.3 Transform統合

**目的**: 視覚効果（回転、傾斜等）とレイアウト変換を分離。

**実装方針**:
- transform.rsの`Transform`コンポーネントを視覚効果専用に変更
- render_surfaceで`final_transform = GlobalArrangement * Transform`を計算
- `GlobalTransform`と`TransformTreeChanged`を削除（誤った設計）

---

## 11. Implementation Checklist

### Phase 1: Entity階層とArrangement基盤
- [ ] bevy_ecs::hierarchy::{ChildOf, Children}をインポート
- [ ] layout.rsにwindows_numerics::Matrix3x2をインポート
- [ ] layout.rsにOffset, LayoutScale, Arrangement, GlobalArrangement, ArrangementTreeChangedを追加
- [ ] From<Offset> for Matrix3x2トレイト実装
- [ ] From<LayoutScale> for Matrix3x2トレイト実装
- [ ] From<Arrangement> for Matrix3x2トレイト実装
- [ ] From<Arrangement> for GlobalArrangementトレイト実装
- [ ] Mul<Arrangement> for GlobalArrangement演算子実装

### Phase 2: Arrangement伝播システム
- [ ] arrangement.rs新規作成
- [ ] sync_simple_arrangementsシステム実装
- [ ] mark_dirty_arrangement_treesシステム実装
- [ ] propagate_global_arrangementsシステム実装
- [ ] world.rsのDrawスケジュールに登録
- [ ] 単純な階層（Window → Rectangle1個）で動作確認

### Phase 3: 階層的Surfaceレンダリング
- [ ] render_surfaceをQuery::iter_descendants_depth_firstで拡張
- [ ] 各子孫描画前にSetTransform(GlobalArrangement)適用
- [ ] Window自身 + 全子孫の描画順序を実装
- [ ] Rectangle → Label1個で階層的描画をテスト

### Phase 4: Rectangle/Label移行
- [ ] Rectangle構造体からx/yフィールド削除
- [ ] Label構造体からx/yフィールド削除
- [ ] draw_rectangles: 原点(0, 0)基準描画に変更
- [ ] draw_labels: 原点(0, 0)基準描画に変更
- [ ] init_window_arrangementシステム実装
- [ ] world.rsのPostLayoutスケジュールに登録

### Phase 5: サンプル更新
- [ ] simple_window.rsに4階層構造追加（6 Rectangle + 2 Label）
- [ ] ChildOf関係設定
- [ ] Arrangement座標設定
- [ ] 色指定追加（青、緑、黄、紫、赤、白）
- [ ] cargo run --example simple_windowで動作確認

### Phase 6: テストと検証
- [ ] tests/arrangement_test.rs作成（Arrangement伝播テスト）
- [ ] layout.rsに単体テスト追加（Arrangement変換テスト）
- [ ] パフォーマンステスト実行（R10検証）
- [ ] エラーハンドリング確認（Visual作成失敗、描画エラー）
- [ ] ドキュメント更新（README.md、AGENTS.md）

---

## 12. Dependencies

### 12.1 既存依存関係

- bevy_ecs 0.17.2 (hierarchy機能を利用)
- windows 0.62.2 (DirectComposition API)
- euclid 0.22.11 (Matrix3x2等の幾何計算)

### 12.2 新規依存関係

なし（既存依存関係で実装可能）

---

## 13. Risks and Mitigation

### 13.1 深さ優先レンダリングの正確性

**リスク**: 描画順序の誤りにより、Widgetが正しく重なって表示されない可能性。

**軽減策**:
- `Query::iter_descendants_depth_first::<Children>`の動作を単純なケース（2階層）で検証
- 描画順序をログ出力して確認
- simple_window.rsサンプルで視覚的に確認

### 13.2 GlobalArrangement累積計算の精度

**リスク**: 行列乗算の累積誤差により、深い階層で座標がずれる可能性。

**軽減策**:
- 単体テストで行列乗算を検証
- サンプルで最大4階層の座標を視覚的に確認
- 将来、より深い階層でテスト

### 13.3 Rectangle/Label移行の破壊的変更

**リスク**: 既存サンプル（areka.rs等）が動作しなくなる可能性。

**軽減策**:
- gap-analysis.mdで既存サンプルの影響範囲を確認済み（areka.rs未実装、dcomp_demo.rsはECS未使用）
- 移行ガイドを提供
- 段階的コミット（Phase 4で集中対応）

### 13.4 パフォーマンス

**リスク**: 階層深度増加時に描画負荷が増大する可能性。

**軽減策**:
- tree_system.rsの最適化機能（並列処理、静的シーン最適化）を活用
- Changed検知により不要な再描画をスキップ
- パフォーマンステストで60fps維持を検証

---

## 14. Success Criteria

### 14.1 機能要件

- [ ] bevy_ecs::hierarchy::{ChildOf, Children}がwintfで動作する
- [ ] Arrangementコンポーネントが正しく伝播する（親→子、累積変換）
- [ ] 階層的Surfaceレンダリングが深さ優先順序で動作する
- [ ] Rectangle/Labelがx/yなしで動作する（Arrangement使用）
- [ ] simple_window.rsサンプルが4階層構造を表示する

### 14.2 品質要件

- [ ] 単体テストが全て成功する（Arrangement変換、行列乗算）
- [ ] 統合テストが全て成功する（Arrangement伝播、Entity階層）
- [ ] パフォーマンステストが60fpsを維持する（50個のWidget）
- [ ] エラーハンドリングが適切に動作する（Visual作成失敗、描画エラー）

### 14.3 ドキュメント要件

- [ ] design.mdが完成している
- [ ] 移行ガイドが提供されている
- [ ] サンプルコードが動作する（simple_window.rs）

---

## 15. Approval

### Design Review

- **Reviewer**: Human + AI Design Validation
- **Status**: ✅ APPROVED
- **Score**: 10.0/10
- **Date**: 2025-11-17

### Design Approval

- **Approver**: Project Owner
- **Status**: ✅ APPROVED
- **Date**: 2025-11-17
- **Comments**: 設計承認。Matrix3x2変換パターンを3段階で洗練（Fromトレイト → ヘルパーメソッド → 細粒度From実装）し、完璧な設計に到達。

### Next Steps

- **Ready for**: Task Generation (`/kiro-spec-tasks visual-tree-implementation`)
- **Implementation**: 6フェーズ、約1週間の見込み

---

_Design specification created on 2025-11-17_  
_Design approved on 2025-11-17_  
_Ready for task generation and implementation_
