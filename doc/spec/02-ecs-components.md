# bevy_ecsコンポーネント管理

## bevy_ecsによるコンポーネント設計

### Entityの特性

bevy_ecsの`Entity`は軽量で効率的な識別子です。

**特徴**:
- ただの数値: 64bit整数（インデックス32bit + 世代32bit）
- Copy可能: `Arc`や`Rc`のようなオーバーヘッドなし
- 所有権フリー: 参照やライフタイムの制約がない
- 一意性保証: 世代カウンタで削除されたEntityの再利用を安全に検出

**コード例**:

```rust
let entity1 = commands.spawn((...)).id();
let entity2 = entity1;

#[derive(Component)]
pub struct Parent(pub Entity);

let parent = world.spawn((...)).id();
let child = world.spawn((Parent(parent), ...)).id();
```

### 基本方針
- **すべてのUI要素はEntityとして存在（論理ツリー）**
- **各機能は独立したComponentとして管理（関心の分離）**
- **Componentは動的に追加/削除可能**
- 必要なEntityだけがコンポーネントを持つ
- 変更は`Changed<T>`/`Added<T>`で自動追跡、効率的に更新

### bevy_ecsのストレージ戦略

bevy_ecsはコンポーネントの特性に応じて最適なストレージを選択します。

**Table Storage**（高密度）:
- 対象: ほとんどのEntityが持つコンポーネント
- 例: `Transform`, `GlobalTransform`
- メリット: 高速なイテレーション、キャッシュ効率

**SparseSet Storage**（低密度）:
- 対象: 一部のEntityのみが持つコンポーネント
- 例: マーカーコンポーネント、一時的な状態
- メリット: 高速な追加/削除

UI設計では、ほとんどのコンポーネントがTable Storageで管理され、`LayoutInvalidated`などのマーカーがSparseSetで管理されます。

### コンポーネントの独立性

各コンポーネントは異なるタイミングで必要になり、独立して存在する：

| コンポーネント | 関心のタイミング | 例 |
|--------------|----------------|-----|
| **Layout** | レイアウトパス | サイズ・配置の計算時 |
| **Visual** | 描画パス | ビジュアルツリー構築時 |
| **DrawingContent** | レンダリングパス | 実際の描画コマンド実行時 |
| **TextContent** | コンテンツ更新時 | テキスト変更時 |
| **InteractionState** | イベント処理時 | マウス・キーボード入力時 |

### コンポーネントの定義

bevy_ecsでは、各データ構造に`#[derive(Component)]`を付けてコンポーネント化します。

#### 基本コンポーネント

```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Name {
    pub value: String,
}
```

親子関係は`bevy_hierarchy`が提供する`Parent`と`Children`を使用します。

#### レイアウトコンポーネント

```rust
#[derive(Component)]
pub struct Layout {
    pub desired_size: Size2D,
    pub final_rect: Rect,
    pub margin: Thickness,
    pub padding: Thickness,
}

/// サイズ指定
#[derive(Component)]
pub struct DesiredSize {
    pub width: Length,
    pub height: Length,
}

#[derive(Component)]
pub struct Alignment {
    pub horizontal: HorizontalAlignment,
    pub vertical: VerticalAlignment,
}
```

#### ビジュアルコンポーネント

```rust
#[derive(Component)]
pub struct Visual {
    pub dcomp_visual: IDCompositionVisual,
    pub dcomp_surface: Option<IDCompositionSurface>,
    pub d2d_device_context: Option<ID2D1DeviceContext>,
    pub offset: Point2D,
    pub size: Size2D,
    pub opacity: f32,
    pub visible: bool,
}

#[derive(Component)]
pub struct DrawingContent {
    pub cached_bitmap: Option<ID2D1Bitmap>,
    pub needs_redraw: bool,
}
```

#### コンテンツコンポーネント

```rust
#[derive(Component)]
pub struct TextContent {
    pub text: String,
    pub font_family: String,
    pub font_size: f32,
    pub color: Color,
    pub text_layout: Option<IDWriteTextLayout>,
}

#[derive(Component)]
pub struct ImageContent {
    pub source: String,
    pub bitmap: Option<ID2D1Bitmap>,
    pub stretch: Stretch,
}

#[derive(Component)]
pub struct ContainerStyle {
    pub background: Option<Brush>,
    pub border: Option<Border>,
    pub corner_radius: f32,
}
```

#### インタラクションコンポーネント

```rust
#[derive(Component)]
pub struct InteractionState {
    pub is_hovered: bool,
    pub is_pressed: bool,
    pub is_focused: bool,
}

#[derive(Component)]
pub struct Clickable {
    pub on_click: Option<EventHandler>,
}
```

#### マーカーコンポーネント

状態管理用の空コンポーネント：

```rust
#[derive(Component)]
pub struct LayoutInvalidated;

#[derive(Component)]
pub struct NeedsRedraw;

#[derive(Component)]
pub struct NeedsTransformUpdate;
```

### システムの定義

bevy_ecsでは、データ（Component）とロジック（System）を完全に分離します。

```rust
use bevy_ecs::prelude::*;

// ========================================
// レイアウトシステム
// ========================================

/// レイアウト計算システム
pub fn compute_layout_system(
    mut query: Query<(Entity, &mut Layout, &DesiredSize, Option<&Children>), With<LayoutInvalidated>>,
    mut commands: Commands,
) {
    for (entity, mut layout, desired_size, children) in query.iter_mut() {
        // レイアウト計算
        layout.compute(desired_size, children);
        
        // 処理完了のマーカーを削除
        commands.entity(entity).remove::<LayoutInvalidated>();
    }
}

/// テキスト変更時にレイアウトを無効化
pub fn text_invalidates_layout_system(
    mut commands: Commands,
    query: Query<Entity, Changed<TextContent>>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(LayoutInvalidated);
    }
}

// ========================================
// ビジュアルシステム
// ========================================

/// Visualを必要に応じて追加
pub fn ensure_visual_system(
    mut commands: Commands,
    query: Query<Entity, (Or<(
        With<TextContent>,
        With<ImageContent>,
        With<ContainerStyle>,
    )>, Without<Visual>)>,
    dcomp_context: Res<DCompContext>,
) {
    for entity in query.iter() {
        let visual = Visual::new(&dcomp_context);
        commands.entity(entity).insert(visual);
    }
}

/// レイアウト変更時にVisualを更新
pub fn layout_to_visual_system(
    mut query: Query<(&Layout, &mut Visual), Changed<Layout>>,
) {
    for (layout, mut visual) in query.iter_mut() {
        visual.offset = layout.final_rect.origin;
        visual.size = layout.final_rect.size;
    }
}

/// Visual変更時に再描画をマーク
pub fn visual_changed_system(
    mut commands: Commands,
    query: Query<Entity, Changed<Visual>>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(NeedsRedraw);
    }
}

// ========================================
// 描画システム
// ========================================

/// 再描画が必要なものを描画
pub fn draw_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Visual, Option<&TextContent>), With<NeedsRedraw>>,
    dcomp_context: Res<DCompContext>,
) {
    for (entity, mut visual, text) in query.iter_mut() {
        // 描画処理
        if let Some(text) = text {
            draw_text(&mut visual, text, &dcomp_context);
        }
        
        // マーカーを削除
        commands.entity(entity).remove::<NeedsRedraw>();
    }
}

// ========================================
// インタラクションシステム
// ========================================

/// マウスホバー検出
pub fn hover_detection_system(
    mut query: Query<(&Layout, &mut InteractionState)>,
    mouse_position: Res<MousePosition>,
) {
    for (layout, mut state) in query.iter_mut() {
        state.is_hovered = layout.final_rect.contains(mouse_position.0);
    }
}

/// クリックイベント処理
pub fn click_system(
    query: Query<(&InteractionState, &Clickable)>,
    mouse_button: Res<Input<MouseButton>>,
) {
    if mouse_button.just_pressed(MouseButton::Left) {
        for (state, clickable) in query.iter() {
            if state.is_hovered {
                if let Some(handler) = &clickable.on_click {
                    handler.invoke();
                }
            }
        }
    }
}
```

### システムの統合とスケジューリング

```rust
use bevy_ecs::prelude::*;

pub fn setup_ui_systems(app: &mut App) {
    // Resourceの登録
    app.insert_resource(DCompContext::new())
        .insert_resource(MousePosition::default());
    
    // システムの登録（実行順序を制御）
    app.add_systems(Update, (
        // 1. 入力処理
        update_mouse_position,
        hover_detection_system,
        click_system,
        
        // 2. コンテンツ変更検知
        text_invalidates_layout_system,
        
        // 3. レイアウト計算
        compute_layout_system,
        
        // 4. Visual管理
        ensure_visual_system,
        layout_to_visual_system,
        attach_new_visual_system,
        
        // 5. 描画
        visual_changed_system,
        draw_system,
        
        // 6. コミット
        commit_dcomp_system,
    ).chain()); // 順番に実行
    
    // 並列実行可能なシステム
    app.add_systems(Update, (
        text_invalidates_layout_system,
        image_changed_system,
        style_changed_system,
        // データ競合がないので並列実行される
    ));
}
```

### bevy_ecsの利点

#### 1. データとロジックの分離

**slotmap時代**:

```rust
pub struct WidgetSystem {
    texts: SecondaryMap<WidgetId, TextContent>,
    
    pub fn set_text(&mut self, id: WidgetId, text: String) {
    }
}
```

**bevy_ecs**:

```rust
#[derive(Component)]
pub struct TextContent {
    pub text: String,
}

pub fn render_text_system(query: Query<&TextContent>) {
}

pub fn text_changed_system(query: Query<&TextContent, Changed<TextContent>>) {
}
```

データとロジックが完全に分離されています。

#### 2. スパース性（メモリ効率）

TextContentを持つEntityだけクエリされます。

```rust
pub fn process_text_system(query: Query<&TextContent>) {
}
```

#### 3. 変更検知の自動化

```rust
// slotmap時代: 手動でダーティフラグ管理
self.dirty_text.insert(widget_id);

// bevy_ecs: 自動追跡
pub fn system(query: Query<&TextContent, Changed<TextContent>>) {
    // 変更されたものだけ自動的に取得
}
```

#### 4. 並列実行

```rust
// これらのシステムは自動的に並列実行される
app.add_systems(Update, (
    process_text_system,      // TextContentを読む
    process_images_system,    // ImageContentを読む
    process_layouts_system,   // Layoutを読む
    // データ競合なし → 並列実行
));
```

### 変更伝播の実例

bevy_ecsでは、`Changed<T>`とマーカーコンポーネントを組み合わせて変更を伝播します。

```rust
// TextContentが変更されたらLayoutを無効化
pub fn text_to_layout(
    mut commands: Commands,
    query: Query<Entity, Changed<TextContent>>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(LayoutInvalidated);
    }
}

// Layoutが無効化されたら再計算
pub fn compute_layout(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Layout), With<LayoutInvalidated>>,
) {
    for (entity, mut layout) in query.iter_mut() {
        layout.compute();
        commands.entity(entity).remove::<LayoutInvalidated>();
    }
}

// Layout変更されたらVisualを更新
pub fn layout_to_visual(
    mut query: Query<(&Layout, &mut Visual), Changed<Layout>>,
) {
    for (layout, mut visual) in query.iter_mut() {
        visual.sync_from_layout(layout);
    }
}

// Visual変更されたら再描画マーク
pub fn visual_to_redraw(
    mut commands: Commands,
    query: Query<Entity, Changed<Visual>>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(NeedsRedraw);
    }
}
```

この連鎖により、**TextContent変更 → Layout無効化 → Layout再計算 → Visual更新 → 再描画**という流れが自動的に実行されます。

### まとめ: slotmap vs bevy_ecs

| 観点 | slotmap | bevy_ecs |
|------|---------|----------|
| データ管理 | `SecondaryMap<WidgetId, T>` | `#[derive(Component)]` |
| ID管理 | `WidgetId` (SlotMap key) | `Entity` (自動管理) |
| 親子関係 | 連結リスト（必須フィールド） | `Parent`/`Children` コンポーネント（オプション） |
| ルート判定 | `parent.is_none()` | `!query.contains(Parent)` |
| システム | メソッド (`impl WidgetSystem`) | 関数 (`pub fn system()`) |
| ダーティ検知 | 手動 (`dirty: HashSet`) | 自動 (`Changed<T>`) |
| 変更伝播 | 手動 (DependencyMap) | システムチェーン |
| 並列処理 | 手動実装が必要 | 自動並列化 |
| メモリ効率 | スパース | 最適化済み（Table/SparseSet） |

**重要な設計上の違い**:
- **slotmap**: すべてのWidgetが親フィールドを持つ（`Option<WidgetId>`）
- **bevy_ecs**: 親を持つEntityのみが`Parent`コンポーネントを持つ（より効率的）

bevy_ecsへの移行により、**宣言的で保守しやすく、自動的に最適化されるコード**になります。

