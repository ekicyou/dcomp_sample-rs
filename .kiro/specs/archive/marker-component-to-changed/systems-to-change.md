# マーカーコンポーネント移行 - 変更箇所詳細

## 1. SurfaceUpdateRequested → SurfaceRenderTrigger移行

### 1.1 コンポーネント定義変更

**ファイル**: `crates/wintf/src/ecs/graphics/components.rs`

**変更前** (line 200-203):
```rust
/// 描画更新が必要なサーフェスを示すマーカーコンポーネント
#[derive(Component, Default)]
pub struct SurfaceUpdateRequested;
```

**変更後**:
```rust
/// Surface描画トリガーコンポーネント
/// Changed<SurfaceRenderTrigger>で検出し、自動リセットされる
#[derive(Component, Default)]
pub struct SurfaceRenderTrigger {
    /// 最後に描画をリクエストしたフレーム番号
    pub requested_frame: u64,
}

impl SurfaceRenderTrigger {
    /// 描画リクエストをトリガー
    pub fn trigger(&mut self, frame: u64) {
        self.requested_frame = frame;
    }
}
```

### 1.2 SafeInsertSurfaceUpdateRequested削除

**ファイル**: `crates/wintf/src/ecs/graphics/components.rs`

**削除対象** (line 182-199):
```rust
struct SafeInsertSurfaceUpdateRequested {
    entity: Entity,
}

impl Command for SafeInsertSurfaceUpdateRequested {
    fn apply(self, world: &mut World) {
        if let Ok(mut entity_mut) = world.get_entity_mut(self.entity) {
            entity_mut.insert(SurfaceUpdateRequested);
        }
    }
}

fn on_surface_graphics_changed(mut world: DeferredWorld, context: HookContext) {
    let mut commands = world.commands();
    commands.queue(SafeInsertSurfaceUpdateRequested {
        entity: context.entity,
    });
}
```

**代替**: SurfaceGraphicsにon_addフックを設定し、SurfaceRenderTriggerも一緒に挿入するか、
別途システムで初期化する。

### 1.3 render_surfaceシステム変更

**ファイル**: `crates/wintf/src/ecs/graphics/systems.rs`

**変更前** (line 155-165):
```rust
pub fn render_surface(
    mut commands: Commands,
    surfaces: Query<
        (
            Entity,
            &SurfaceGraphics,
            &GlobalArrangement,
            Option<&GraphicsCommandList>,
            Option<&Name>,
        ),
        With<SurfaceUpdateRequested>,
    >,
```

**変更後**:
```rust
pub fn render_surface(
    surfaces: Query<
        (
            Entity,
            &SurfaceGraphics,
            &GlobalArrangement,
            Option<&GraphicsCommandList>,
            Option<&Name>,
        ),
        Changed<SurfaceRenderTrigger>,
    >,
```

**追加変更** (line 278): `remove::<SurfaceUpdateRequested>()`の削除
```rust
// 削除: commands.entity(entity).remove::<SurfaceUpdateRequested>();
```

### 1.4 mark_dirty_surfacesシステム変更

**ファイル**: `crates/wintf/src/ecs/graphics/systems.rs`

**変更前** (line 826-843):
```rust
pub fn mark_dirty_surfaces(
    mut commands: Commands,
    changed_query: Query<
        Entity,
        (
            Or<(
                Changed<GraphicsCommandList>,
                Changed<SurfaceGraphics>,
                Changed<GlobalArrangement>,
            )>,
            With<SurfaceGraphics>,
        ),
    >,
) {
    for entity in changed_query.iter() {
        commands.entity(entity).insert(SurfaceUpdateRequested);
    }
}
```

**変更後**:
```rust
pub fn mark_dirty_surfaces(
    mut trigger_query: Query<&mut SurfaceRenderTrigger>,
    changed_query: Query<
        Entity,
        (
            Or<(
                Changed<GraphicsCommandList>,
                Changed<SurfaceGraphics>,
                Changed<GlobalArrangement>,
            )>,
            With<SurfaceGraphics>,
        ),
    >,
    frame_count: Res<crate::ecs::world::FrameCount>,
) {
    for entity in changed_query.iter() {
        if let Ok(mut trigger) = trigger_query.get_mut(entity) {
            trigger.trigger(frame_count.0);
        }
    }
}
```

### 1.5 deferred_surface_creation_system変更

**ファイル**: `crates/wintf/src/ecs/graphics/systems.rs`

**変更前** (line 1126-1129):
```rust
// SurfaceUpdateRequestedも挿入して描画をトリガー
commands
    .entity(entity)
    .insert(super::components::SurfaceUpdateRequested);
```

**変更後**:
```rust
// SurfaceRenderTriggerを挿入して描画をトリガー
// 初期値のrequest_frame=0がChanged検出をトリガーする
commands
    .entity(entity)
    .insert(SurfaceRenderTrigger::default());
```

または、すでにSurfaceRenderTriggerがある場合:
```rust
// 既存のSurfaceRenderTriggerをトリガー
// （deferred_surface_creationではSurfaceGraphicsと同時に挿入されるはず）
```

---

## 2. GraphicsNeedsInit → GraphicsInitState移行

### 2.1 コンポーネント定義変更

**ファイル**: `crates/wintf/src/ecs/graphics/components.rs`

**変更前** (line 14-16):
```rust
/// 初期化が必要な状態を示す動的マーカー
#[derive(Component, Default)]
pub struct GraphicsNeedsInit;
```

**変更後**:
```rust
/// グラフィックス初期化状態コンポーネント
/// Changed<GraphicsInitState>とneeds_init()で検出
#[derive(Component, Default)]
pub struct GraphicsInitState {
    /// 初期化が必要な世代番号
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

### 2.2 init_graphics_coreシステム変更

**ファイル**: `crates/wintf/src/ecs/graphics/systems.rs`

**変更箇所** (line 363-365, 390-392):

**変更前**:
```rust
for entity in query.iter() {
    commands.entity(entity).insert(GraphicsNeedsInit);
}
```

**変更後**:
```rust
for entity in query.iter() {
    if let Ok(mut state) = init_state_query.get_mut(entity) {
        state.request_init();
    }
}
```

### 2.3 init_window_graphicsシステム変更

**ファイル**: `crates/wintf/src/ecs/graphics/systems.rs`

**変更前** (line 414-418):
```rust
    mut query: Query<
        (...),
        Or<(Without<WindowGraphics>, With<GraphicsNeedsInit>)>,
    >,
```

**変更後**:
```rust
    mut query: Query<
        (..., Option<&GraphicsInitState>),
        Or<(Without<WindowGraphics>, Changed<GraphicsInitState>)>,
    >,
```

システム内で`state.needs_init()`をチェック。

### 2.4 cleanup_graphics_needs_initシステム変更

**ファイル**: `crates/wintf/src/ecs/graphics/systems.rs`

**変更前** (line 748-763):
```rust
pub fn cleanup_graphics_needs_init(
    query: Query<
        (Entity, &WindowGraphics, &VisualGraphics, &SurfaceGraphics),
        With<GraphicsNeedsInit>,
    >,
    mut commands: Commands,
) {
    for (entity, window_graphics, visual, surface) in query.iter() {
        if window_graphics.is_valid() && visual.is_valid() && surface.is_valid() {
            commands.entity(entity).remove::<GraphicsNeedsInit>();
        }
    }
}
```

**変更後**:
```rust
pub fn cleanup_graphics_needs_init(
    mut query: Query<
        (&WindowGraphics, &VisualGraphics, &SurfaceGraphics, &mut GraphicsInitState),
        Changed<GraphicsInitState>,
    >,
) {
    for (window_graphics, visual, surface, mut state) in query.iter_mut() {
        if state.needs_init() && window_graphics.is_valid() && visual.is_valid() && surface.is_valid() {
            state.mark_initialized();
        }
    }
}
```

### 2.5 cleanup_command_list_on_reinitシステム変更

**ファイル**: `crates/wintf/src/ecs/graphics/systems.rs`

**変更前** (line 767-779):
```rust
pub fn cleanup_command_list_on_reinit(
    query: Query<
        Entity,
        (
            With<GraphicsNeedsInit>,
            With<crate::ecs::graphics::GraphicsCommandList>,
        ),
    >,
    mut commands: Commands,
) {
```

**変更後**:
```rust
pub fn cleanup_command_list_on_reinit(
    query: Query<
        (Entity, &GraphicsInitState),
        (
            Changed<GraphicsInitState>,
            With<crate::ecs::graphics::GraphicsCommandList>,
        ),
    >,
    mut commands: Commands,
) {
    for (entity, state) in query.iter() {
        if state.needs_init() {
            commands
                .entity(entity)
                .remove::<crate::ecs::graphics::GraphicsCommandList>();
        }
    }
}
```

### 2.6 visual_manager.rsの変更

**ファイル**: `crates/wintf/src/ecs/graphics/visual_manager.rs`

**変更前** (line 113):
```rust
    query: Query<(Entity, &Visual), With<GraphicsNeedsInit>>,
```

**変更後**:
```rust
    query: Query<(Entity, &Visual, &GraphicsInitState), Changed<GraphicsInitState>>,
```

---

## 3. テストコード変更

### 3.1 surface_optimization_test.rs

**ファイル**: `crates/wintf/tests/surface_optimization_test.rs`

すべての`SurfaceUpdateRequested`を`SurfaceRenderTrigger`に置き換え、
検出ロジックを`Changed<SurfaceRenderTrigger>`に対応させる。

---

## 4. 公開API更新

**ファイル**: `crates/wintf/src/ecs/mod.rs`または該当箇所

`SurfaceUpdateRequested`のre-exportを`SurfaceRenderTrigger`に変更。

---

## 5. 移行時の注意点

1. **Changed検出の初回発火**: コンポーネントが追加された直後のフレームでも`Changed`が発火する
2. **フレーム末リセット**: `Changed`は各フレーム末に自動リセットされる
3. **同一フレーム内の複数変更**: 同じフレーム内で複数回値を変更しても1回の`Changed`として扱われる
4. **クエリ順序**: `Changed`を使うシステムは、変更を行うシステムの後に実行する必要がある
