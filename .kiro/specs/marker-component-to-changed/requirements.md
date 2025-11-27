# マーカーコンポーネントからChanged検出への移行

## 1. 概要

bevy_ecsにおけるマーカーコンポーネントの`With<Marker>` + `remove()` パターンを、
`Changed<T>` パターンに移行し、アーキタイプ変更のオーバーヘッドを排除する。

## 2. 背景と動機

### 2.1 現状の問題

現在のマーカーコンポーネントパターンでは：
- `commands.entity(entity).insert(MarkerComponent)` → アーキタイプ変更（高コスト）
- `commands.entity(entity).remove::<MarkerComponent>()` → アーキタイプ変更（高コスト）

これが毎フレーム複数エンティティで発生すると、パフォーマンス上の問題となる。

### 2.2 Changed\<T\>パターンの利点

- `Changed<T>` は各フレーム末に自動的にリセットされる
- コンポーネントの値を変更するだけでフラグが立つ
- アーキタイプ変更が発生しない（`insert`/`remove`不要）
- 低オーバーヘッド

## 3. 影響を受けるマーカーコンポーネント

### 3.1 SurfaceUpdateRequested

**用途**: Surface描画更新が必要なことを示す

**現在のコード**:
```rust
// components.rs
#[derive(Component, Default)]
pub struct SurfaceUpdateRequested;
```

**使用箇所**:

| ファイル | 行 | 用途 | パターン |
|---------|-----|------|---------|
| `systems.rs` | 164 | `render_surface` クエリフィルター | `With<SurfaceUpdateRequested>` |
| `systems.rs` | 278 | `render_surface` 処理後削除 | `commands.entity(entity).remove::<SurfaceUpdateRequested>()` |
| `systems.rs` | 842 | `mark_dirty_surfaces` マーカー挿入 | `commands.entity(entity).insert(SurfaceUpdateRequested)` |
| `systems.rs` | 1129 | `deferred_surface_creation_system` 描画トリガー | `commands.entity(entity).insert(SurfaceUpdateRequested)` |
| `components.rs` | 189-196 | `on_surface_graphics_changed` フック | `SafeInsertSurfaceUpdateRequested` Command |

### 3.2 GraphicsNeedsInit

**用途**: グラフィックスリソースの初期化/再初期化が必要なことを示す

**現在のコード**:
```rust
// components.rs
#[derive(Component, Default)]
pub struct GraphicsNeedsInit;
```

**使用箇所**:

| ファイル | 行 | 用途 | パターン |
|---------|-----|------|---------|
| `systems.rs` | 365 | `init_graphics_core` 再初期化時マーカー挿入 | `commands.entity(entity).insert(GraphicsNeedsInit)` |
| `systems.rs` | 392 | `init_graphics_core` 初期化時マーカー挿入 | `commands.entity(entity).insert(GraphicsNeedsInit)` |
| `systems.rs` | 416 | `init_window_graphics` クエリフィルター | `With<GraphicsNeedsInit>` |
| `systems.rs` | 483 | `init_window_visual` クエリフィルター | `With<GraphicsNeedsInit>` |
| `systems.rs` | 752 | `cleanup_graphics_needs_init` クエリフィルター | `With<GraphicsNeedsInit>` |
| `systems.rs` | 762 | `cleanup_graphics_needs_init` マーカー削除 | `commands.entity(entity).remove::<GraphicsNeedsInit>()` |
| `systems.rs` | 772 | `cleanup_command_list_on_reinit` クエリフィルター | `With<GraphicsNeedsInit>` |
| `visual_manager.rs` | 113 | `create_visuals_for_init_marked` クエリフィルター | `With<GraphicsNeedsInit>` |

## 4. 変換方針

### 4.1 SurfaceUpdateRequested

**現在**: Unit structマーカー
```rust
pub struct SurfaceUpdateRequested;
```

**変更後**: フレームカウントを持つコンポーネント
```rust
#[derive(Component, Default)]
pub struct SurfaceRenderTrigger {
    /// 最後に描画をリクエストしたフレーム番号
    pub requested_frame: u64,
}
```

**検出方法**:
- `With<SurfaceUpdateRequested>` → `Changed<SurfaceRenderTrigger>`
- `remove::<SurfaceUpdateRequested>()` → 削除不要（`Changed`は自動リセット）
- `insert(SurfaceUpdateRequested)` → `trigger.requested_frame = current_frame`

### 4.2 GraphicsNeedsInit

**現在**: Unit structマーカー
```rust
pub struct GraphicsNeedsInit;
```

**変更後**: 初期化世代を持つコンポーネント
```rust
#[derive(Component, Default)]
pub struct GraphicsInitState {
    /// 初期化が必要な世代番号（0=初期化不要）
    pub needs_init_generation: u32,
    /// 処理済みの世代番号
    pub processed_generation: u32,
}

impl GraphicsInitState {
    /// 初期化をリクエスト
    pub fn request_init(&mut self) {
        self.needs_init_generation = self.processed_generation.wrapping_add(1);
    }
    
    /// 初期化が必要か判定
    pub fn needs_init(&self) -> bool {
        self.needs_init_generation != self.processed_generation
    }
    
    /// 初期化完了をマーク
    pub fn mark_initialized(&mut self) {
        self.processed_generation = self.needs_init_generation;
    }
}
```

**検出方法**:
- `With<GraphicsNeedsInit>` → `Changed<GraphicsInitState>` + `state.needs_init()`
- `remove::<GraphicsNeedsInit>()` → `state.mark_initialized()`
- `insert(GraphicsNeedsInit)` → `state.request_init()`

## 5. 移行手順

### Phase 1: SurfaceUpdateRequested移行
1. `SurfaceRenderTrigger` コンポーネント定義追加
2. `render_surface` システム変更
3. `mark_dirty_surfaces` システム変更
4. `deferred_surface_creation_system` システム変更
5. `on_surface_graphics_changed` フック変更
6. テストコード更新

### Phase 2: GraphicsNeedsInit移行
1. `GraphicsInitState` コンポーネント定義追加
2. `init_graphics_core` システム変更
3. `init_window_graphics` システム変更
4. `init_window_visual` システム変更
5. `cleanup_graphics_needs_init` システム変更
6. `cleanup_command_list_on_reinit` システム変更
7. `visual_manager.rs` の関連システム変更

### Phase 3: 旧コンポーネント削除
1. `SurfaceUpdateRequested` 定義削除
2. `GraphicsNeedsInit` 定義削除
3. 関連する `SafeInsertSurfaceUpdateRequested` Command削除

## 6. 互換性への影響

### 6.1 テストコード

`surface_optimization_test.rs` の以下のテストが影響を受ける：
- `test_surface_update_requested_component_exists`
- `test_mark_dirty_surfaces_propagation`
- `test_surface_update_requested_on_add_hook`

### 6.2 公開API

`wintf::ecs::SurfaceUpdateRequested` が公開されている場合、
`SurfaceRenderTrigger` への移行が必要。

## 7. 期待される効果

1. **パフォーマンス向上**: アーキタイプ変更の排除
2. **コード簡素化**: `insert`/`remove` の冗長なコードが削減
3. **デバッグ容易性**: フレーム番号や世代番号による追跡が可能
4. **一貫性**: 全マーカーコンポーネントが同じパターンに

## 8. 関連仕様

- `surface-allocation-optimization`: Surface作成/再作成の最適化（本仕様と連携）
