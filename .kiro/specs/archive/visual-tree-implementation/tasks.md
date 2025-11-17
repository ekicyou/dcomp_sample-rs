# Implementation Tasks: visual-tree-implementation

**Feature ID**: `visual-tree-implementation`  
**Created**: 2025-11-17  
**Status**: Ready for Implementation  
**Language**: 日本語

---

## Task Overview

**総タスク数**: 38タスク  
**推定工数**: 5-7日  
**実装順序**: Phase 1 → Phase 6（段階的）

---

## Phase 1: Entity階層とArrangement基盤 (推定: 1日)

### Task 1.1: bevy_ecs::hierarchyインポート
**ファイル**: `crates/wintf/src/ecs/mod.rs`  
**内容**:
```rust
pub use bevy_ecs::hierarchy::{ChildOf, Children};
```
**検証**: コンパイル成功

---

### Task 1.2: layout.rsへのimport追加
**ファイル**: `crates/wintf/src/ecs/layout.rs`  
**内容**:
```rust
use bevy_ecs::prelude::*;
use windows_numerics::Matrix3x2;
```
**検証**: コンパイル成功

---

### Task 1.3: Offset構造体追加
**ファイル**: `crates/wintf/src/ecs/layout.rs`  
**内容**:
```rust
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
```
**検証**: コンパイル成功

---

### Task 1.4: LayoutScale構造体追加
**ファイル**: `crates/wintf/src/ecs/layout.rs`  
**内容**:
```rust
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
```
**検証**: コンパイル成功

---

### Task 1.5: Arrangement構造体追加
**ファイル**: `crates/wintf/src/ecs/layout.rs`  
**内容**:
```rust
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
```
**検証**: コンパイル成功

---

### Task 1.6: GlobalArrangement構造体追加
**ファイル**: `crates/wintf/src/ecs/layout.rs`  
**内容**:
```rust
#[derive(Component, Debug, Clone, Copy, PartialEq)]
pub struct GlobalArrangement(pub Matrix3x2);

impl Default for GlobalArrangement {
    fn default() -> Self {
        Self(Matrix3x2::identity())
    }
}
```
**検証**: コンパイル成功

---

### Task 1.7: ArrangementTreeChanged追加
**ファイル**: `crates/wintf/src/ecs/layout.rs`  
**内容**:
```rust
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct ArrangementTreeChanged;
```
**検証**: コンパイル成功

---

### Task 1.8: From<Offset> for Matrix3x2実装
**ファイル**: `crates/wintf/src/ecs/layout.rs`  
**内容**:
```rust
impl From<Offset> for Matrix3x2 {
    fn from(offset: Offset) -> Self {
        Matrix3x2::translation(offset.x, offset.y)
    }
}
```
**検証**: 単体テスト成功

---

### Task 1.9: From<LayoutScale> for Matrix3x2実装
**ファイル**: `crates/wintf/src/ecs/layout.rs`  
**内容**:
```rust
impl From<LayoutScale> for Matrix3x2 {
    fn from(scale: LayoutScale) -> Self {
        Matrix3x2::scale(scale.x, scale.y)
    }
}
```
**検証**: 単体テスト成功

---

### Task 1.10: From<Arrangement> for Matrix3x2実装
**ファイル**: `crates/wintf/src/ecs/layout.rs`  
**内容**:
```rust
impl From<Arrangement> for Matrix3x2 {
    fn from(arr: Arrangement) -> Self {
        let scale: Matrix3x2 = arr.scale.into();
        let translation: Matrix3x2 = arr.offset.into();
        scale * translation
    }
}
```
**検証**: 単体テスト成功

---

### Task 1.11: From<Arrangement> for GlobalArrangement実装
**ファイル**: `crates/wintf/src/ecs/layout.rs`  
**内容**:
```rust
impl From<Arrangement> for GlobalArrangement {
    fn from(arrangement: Arrangement) -> Self {
        Self(arrangement.into())
    }
}
```
**検証**: コンパイル成功

---

### Task 1.12: Mul<Arrangement> for GlobalArrangement実装
**ファイル**: `crates/wintf/src/ecs/layout.rs`  
**内容**:
```rust
impl std::ops::Mul<Arrangement> for GlobalArrangement {
    type Output = GlobalArrangement;

    fn mul(self, rhs: Arrangement) -> Self::Output {
        let child_matrix: Matrix3x2 = rhs.into();
        GlobalArrangement(self.0 * child_matrix)
    }
}
```
**検証**: 単体テスト成功

---

### Task 1.13: layout.rs単体テスト追加
**ファイル**: `crates/wintf/src/ecs/layout.rs`  
**内容**: Section 8.1のテストコードを実装
- `test_offset_from()`
- `test_scale_from()`
- `test_arrangement_from()`
- `test_global_arrangement_mul()`

**検証**: `cargo test --lib layout` 成功

---

## Phase 2: Arrangement伝播システム (推定: 1-2日)

### Task 2.1: arrangement.rs新規作成
**ファイル**: `crates/wintf/src/ecs/arrangement.rs`  
**内容**: 基本骨格（import、コメント）
```rust
use bevy_ecs::prelude::*;
use bevy_ecs::hierarchy::{ChildOf, Children};
use crate::ecs::layout::{Arrangement, GlobalArrangement, ArrangementTreeChanged};
use crate::ecs::tree_system::{sync_simple_transforms, mark_dirty_trees, propagate_parent_transforms};
```
**検証**: コンパイル成功

---

### Task 2.2: sync_simple_arrangementsシステム実装
**ファイル**: `crates/wintf/src/ecs/arrangement.rs`  
**内容**:
```rust
pub fn sync_simple_arrangements(
    query: ParamSet<(
        Query<(&Arrangement, &mut GlobalArrangement), (Without<ChildOf>, Without<Children>)>,
        Query<(Entity, &Arrangement, &mut GlobalArrangement, &mut ArrangementTreeChanged), Without<ChildOf>>,
    )>,
    orphaned: RemovedComponents<ChildOf>,
) {
    sync_simple_transforms::<Arrangement, GlobalArrangement, ArrangementTreeChanged>(query, orphaned);
}
```
**検証**: コンパイル成功

---

### Task 2.3: mark_dirty_arrangement_treesシステム実装
**ファイル**: `crates/wintf/src/ecs/arrangement.rs`  
**内容**:
```rust
pub fn mark_dirty_arrangement_trees(
    changed: Query<Entity, Or<(Changed<Arrangement>, Changed<ChildOf>, Added<GlobalArrangement>)>>,
    orphaned: RemovedComponents<ChildOf>,
    transforms: Query<(Option<&ChildOf>, &mut ArrangementTreeChanged)>,
) {
    mark_dirty_trees::<Arrangement, GlobalArrangement, ArrangementTreeChanged>(changed, orphaned, transforms);
}
```
**検証**: コンパイル成功

---

### Task 2.4: propagate_global_arrangementsシステム実装
**ファイル**: `crates/wintf/src/ecs/arrangement.rs`  
**内容**:
```rust
pub fn propagate_global_arrangements(
    queue: Local<WorkQueue>,
    roots: Query<(Entity, Ref<Arrangement>, &mut GlobalArrangement, &Children), (Without<ChildOf>, Changed<ArrangementTreeChanged>)>,
    nodes: NodeQuery<Arrangement, GlobalArrangement, ArrangementTreeChanged>,
) {
    propagate_parent_transforms::<Arrangement, GlobalArrangement, ArrangementTreeChanged>(queue, roots, nodes);
}
```
**検証**: コンパイル成功

---

### Task 2.5: mod.rsにarrangement登録
**ファイル**: `crates/wintf/src/ecs/mod.rs`  
**内容**:
```rust
pub mod arrangement;
pub use arrangement::*;
```
**検証**: コンパイル成功

---

### Task 2.6: world.rsのDrawスケジュールに登録
**ファイル**: `crates/wintf/src/ecs/world.rs`  
**内容**: Drawスケジュールに3システム追加
```rust
schedules.add_systems(Draw, (
    cleanup_graphics_needs_init,
    draw_rectangles,
    draw_labels,
    sync_simple_arrangements,
    mark_dirty_arrangement_trees,
    propagate_global_arrangements,
).chain());
```
**検証**: コンパイル成功

---

### Task 2.7: 単純な階層で動作確認
**内容**: テストプログラム作成（Window → Rectangle1個）
- Window EntityにArrangement追加
- Rectangle EntityにArrangement + ChildOf追加
- GlobalArrangementが正しく伝播されることを確認

**検証**: `cargo run --example simple_window`で確認

---

## Phase 3: 階層的Surfaceレンダリング (推定: 1-2日)

### Task 3.1: render_surfaceシグネチャ変更
**ファイル**: `crates/wintf/src/ecs/graphics/systems.rs`  
**内容**: Queryを拡張
```rust
pub fn render_surface(
    windows: Query<(Entity, &SurfaceGraphics), With<Window>>,
    widgets: Query<(Option<&GlobalArrangement>, Option<&GraphicsCommandList>, Option<&Children>)>,
) {
    // 実装...
}
```
**検証**: コンパイル成功

---

### Task 3.2: Window自身の描画実装
**ファイル**: `crates/wintf/src/ecs/graphics/systems.rs`  
**内容**: Window Entityの描画ロジック
```rust
if let Ok((global_arr, cmd_list, _)) = widgets.get(window_entity) {
    if let Some(arr) = global_arr {
        surf.set_transform(&arr.0);
    }
    if let Some(list) = cmd_list {
        surf.draw(list);
    }
}
```
**検証**: コンパイル成功

---

### Task 3.3: 深さ優先子孫探索実装
**ファイル**: `crates/wintf/src/ecs/graphics/systems.rs`  
**内容**: `iter_descendants_depth_first`使用
```rust
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
```
**検証**: コンパイル成功

---

### Task 3.4: エラーハンドリング追加
**ファイル**: `crates/wintf/src/ecs/graphics/systems.rs`  
**内容**: begin_draw/end_draw/commitのエラー処理
```rust
if let Err(e) = surf.begin_draw() {
    eprintln!("Failed to begin draw for window {:?}: {:?}", window_entity, e);
    continue;
}
// ...
```
**検証**: コンパイル成功

---

### Task 3.5: Rectangle → Label階層描画テスト
**内容**: テストプログラム作成
- Rectangle Entity作成
- Label Entity作成（Rectangleの子）
- 両方が正しく描画されることを確認

**検証**: `cargo run --example simple_window`で視覚確認

---

## Phase 4: Rectangle/Label移行 (推定: 1日)

### Task 4.1: Rectangle構造体からx/y削除
**ファイル**: `crates/wintf/src/ecs/widget/shapes/rectangle.rs`  
**内容**: x/yフィールド削除
```rust
#[derive(Component, Debug, Clone, Copy)]
pub struct Rectangle {
    pub width: f32,
    pub height: f32,
    pub color: Color,
}
```
**検証**: コンパイルエラーの修正確認

---

### Task 4.2: Label構造体からx/y削除
**ファイル**: `crates/wintf/src/ecs/widget/text/label.rs`  
**内容**: x/yフィールド削除
```rust
#[derive(Component, Debug, Clone)]
pub struct Label {
    pub text: String,
    pub font_family: String,
    pub font_size: f32,
    pub color: Color,
}
```
**検証**: コンパイルエラーの修正確認

---

### Task 4.3: draw_rectangles修正
**ファイル**: `crates/wintf/src/ecs/widget/shapes/draw_rectangles.rs`  
**内容**: 原点(0, 0)基準描画に変更
```rust
pub fn draw_rectangles(
    mut query: Query<(&Rectangle, &mut GraphicsCommandList)>,
) {
    for (rect, mut cmd_list) in query.iter_mut() {
        cmd_list.add_rectangle(0.0, 0.0, rect.width, rect.height, rect.color);
    }
}
```
**検証**: コンパイル成功

---

### Task 4.4: draw_labels修正
**ファイル**: `crates/wintf/src/ecs/widget/text/draw_labels.rs`  
**内容**: 原点(0, 0)基準描画に変更
```rust
pub fn draw_labels(
    mut query: Query<(&Label, &mut GraphicsCommandList)>,
) {
    for (label, mut cmd_list) in query.iter_mut() {
        cmd_list.add_text(0.0, 0.0, &label.text, label.font_size, label.color);
    }
}
```
**検証**: コンパイル成功

---

### Task 4.5: init_window_arrangementシステム実装
**ファイル**: `crates/wintf/src/ecs/window_system.rs`  
**内容**:
```rust
pub fn init_window_arrangement(
    mut commands: Commands,
    query: Query<Entity, (With<Window>, Without<Arrangement>)>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert((
            Arrangement::default(),
            GlobalArrangement::default(),
            ArrangementTreeChanged,
        ));
    }
}
```
**検証**: コンパイル成功

---

### Task 4.6: world.rsのPostLayoutに登録
**ファイル**: `crates/wintf/src/ecs/world.rs`  
**内容**: init_window_arrangementを追加
```rust
schedules.add_systems(PostLayout, (
    init_graphics_core,
    cleanup_command_list_on_reinit,
    init_window_graphics,
    init_window_visual,
    init_window_surface,
    init_window_arrangement,
).chain());
```
**検証**: コンパイル成功

---

## Phase 5: サンプル更新 (推定: 1日)

### Task 5.1: simple_window.rs構造設計
**ファイル**: `crates/wintf/examples/simple_window.rs`  
**内容**: 4階層構造の設計
```
Window
├─ Rectangle1 (青、200x150、offset: 20,20)
│  ├─ Rectangle1-1 (緑、80x60、offset: 10,10)
│  │  └─ Label1 (赤「Hello」、offset: 5,5)
│  └─ Rectangle1-2 (黄、80x60、offset: 10,80)
│     └─ Rectangle1-2-1 (紫、60x40、offset: 10,10)
│        └─ Label2 (白「World」、offset: 5,5)
```
**検証**: 設計レビュー

---

### Task 5.2: Window Entity作成
**ファイル**: `crates/wintf/examples/simple_window.rs`  
**内容**:
```rust
let window_entity = commands.spawn((
    Window { /* ... */ },
    Arrangement::default(),
)).id();
```
**検証**: コンパイル成功

---

### Task 5.3: Rectangle階層作成
**ファイル**: `crates/wintf/examples/simple_window.rs`  
**内容**: 6個のRectangle Entity作成
- Rectangle1: 青、ChildOf(window)
- Rectangle1-1: 緑、ChildOf(rect1)
- Rectangle1-2: 黄、ChildOf(rect1)
- Rectangle1-2-1: 紫、ChildOf(rect1_2)

**検証**: コンパイル成功

---

### Task 5.4: Label追加
**ファイル**: `crates/wintf/examples/simple_window.rs`  
**内容**: 2個のLabel Entity作成
- Label1: 赤「Hello」、ChildOf(rect1_1)
- Label2: 白「World」、ChildOf(rect1_2_1)

**検証**: コンパイル成功

---

### Task 5.5: 動作確認
**内容**: `cargo run --example simple_window`実行
- 6個のRectangleが階層的に表示される
- 2個のLabelが正しい位置に表示される
- 色が正しい（青、緑、黄、紫、赤、白）

**検証**: 視覚的確認 ✅

---

## Phase 6: テストと検証 (推定: 1日)

### Task 6.1: arrangement_test.rs作成
**ファイル**: `crates/wintf/tests/arrangement_test.rs`  
**内容**: Section 8.1のテストコード実装
```rust
#[test]
fn test_arrangement_propagation() {
    // Window → Rectangle1 の階層でGlobalArrangement伝播を検証
}
```
**検証**: `cargo test --test arrangement_test` 成功

---

### Task 6.2: パフォーマンステスト実装
**ファイル**: `crates/wintf/tests/performance_test.rs`  
**内容**: 50個のWidget描画パフォーマンス測定
```rust
#[test]
fn test_50_widgets_60fps() {
    // 50個のRectangle/Labelで60fps維持を検証
}
```
**検証**: 1フレーム16.6ms以内 ✅

---

### Task 6.3: エラーハンドリング確認
**内容**: 手動テスト
- Visual作成失敗のシミュレーション
- 描画エラーのシミュレーション
- エラーログが適切に出力されることを確認

**検証**: エラー時の動作確認 ✅

---

### Task 6.4: ドキュメント更新（README.md）
**ファイル**: `README.md`  
**内容**: Entity階層の説明追加
- bevy_ecs::hierarchy::{ChildOf, Children}の使用方法
- Arrangementコンポーネントの説明
- サンプルコード追加

**検証**: ドキュメントレビュー

---

### Task 6.5: ドキュメント更新（AGENTS.md）
**ファイル**: `AGENTS.md`  
**内容**: 実装完了の記録
- visual-tree-implementationの完了マーク
- 次のフェーズへの移行準備

**検証**: ドキュメントレビュー

---

## Task Dependencies

```
Phase 1: 1.1 → 1.2 → 1.3-1.7 (並列) → 1.8-1.12 (並列) → 1.13
Phase 2: 2.1 → 2.2-2.4 (並列) → 2.5 → 2.6 → 2.7
Phase 3: 3.1 → 3.2-3.4 (並列) → 3.5
Phase 4: 4.1-4.2 (並列) → 4.3-4.4 (並列) → 4.5 → 4.6
Phase 5: 5.1 → 5.2 → 5.3 → 5.4 → 5.5
Phase 6: 6.1-6.3 (並列) → 6.4-6.5 (並列)
```

---

## Success Criteria

### 機能要件
- [ ] bevy_ecs::hierarchy::{ChildOf, Children}がwintfで動作する
- [ ] Arrangementコンポーネントが正しく伝播する（親→子、累積変換）
- [ ] 階層的Surfaceレンダリングが深さ優先順序で動作する
- [ ] Rectangle/Labelがx/yなしで動作する（Arrangement使用）
- [ ] simple_window.rsサンプルが4階層構造を表示する

### 品質要件
- [ ] 単体テストが全て成功する（Arrangement変換、行列乗算）
- [ ] 統合テストが全て成功する（Arrangement伝播、Entity階層）
- [ ] パフォーマンステストが60fpsを維持する（50個のWidget）
- [ ] エラーハンドリングが適切に動作する（Visual作成失敗、描画エラー）

### ドキュメント要件
- [ ] README.mdが更新されている
- [ ] AGENTS.mdが更新されている
- [ ] サンプルコードが動作する（simple_window.rs）

---

## Estimated Timeline

| Phase | 推定工数 | 依存関係 |
|-------|---------|---------|
| Phase 1 | 1日 | なし |
| Phase 2 | 1-2日 | Phase 1 |
| Phase 3 | 1-2日 | Phase 2 |
| Phase 4 | 1日 | Phase 3 |
| Phase 5 | 1日 | Phase 4 |
| Phase 6 | 1日 | Phase 5 |
| **Total** | **5-7日** | 段階的 |

---

_Tasks generated on 2025-11-17_  
_Ready for implementation (`/kiro-spec-impl visual-tree-implementation`)_
