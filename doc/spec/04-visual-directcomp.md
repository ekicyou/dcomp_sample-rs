# Visual: DirectCompositionとbevy_ecsの統合

## bevy_ecsによる変更伝播

bevy_ecsでは、システムチェーンと`Changed<T>`フィルタで自動的に変更を伝播します。

### 変更伝播の流れ

```text
TextContent変更 → LayoutInvalidated → Layout再計算 → Visual更新 → NeedsRedraw → 描画
```

このフローはシステムの実行順序で制御されます：

```rust
use bevy_ecs::prelude::*;

pub fn setup_update_systems(app: &mut App) {
    app.add_systems(Update, (
        // 1. プロパティ変更検知
        text_content_changed_system,
        size_changed_system,
        
        // 2. レイアウト無効化
        invalidate_layout_system,
        
        // 3. レイアウト計算
        compute_layout_system,
        
        // 4. Visual更新
        layout_to_visual_system,
        ensure_visual_system,
        
        // 5. 描画マーク
        visual_changed_system,
        
        // 6. 実際の描画
        draw_visual_system,
        
        // 7. コミット
        commit_dcomp_system,
    ).chain()); // 順番に実行
}
```

### プロパティ変更の流れ

bevy_ecsでは`Changed<T>`で自動追跡、マーカーコンポーネントで状態管理：

```rust
use bevy_ecs::prelude::*;

/// テキストが変更されたらレイアウト無効化
pub fn text_content_changed_system(
    mut commands: Commands,
    query: Query<Entity, Changed<TextContent>>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(LayoutInvalidated);
    }
}

/// レイアウトが変更されたらVisual更新
pub fn layout_to_visual_system(
    mut query: Query<(&ComputedLayout, &mut Visual), Changed<ComputedLayout>>,
) {
    for (layout, mut visual) in query.iter_mut() {
        visual.offset = layout.final_rect.origin;
        visual.size = layout.final_rect.size;
        // visualを変更したので、自動的にChanged<Visual>になる
    }
}
        }
    }
}
```

## Visual: DirectCompositionとの統合

### コンポーネントの分離

描画に関わる要素を3つのコンポーネントに分離：

1. **Visual** - ビジュアルツリーの管理（DirectCompositionを使用）
2. **DrawingContent** - 描画コマンド（ID2D1Image）
3. **Layout** - サイズ・配置情報

これらは独立して存在し、異なるタイミングで更新される。

### Visual の役割
- **描画が必要なWidgetのみが持つ（動的に作成）**
- ビジュアルツリーのノード（DirectCompositionを内部で使用）
- トランスフォーム、不透明度、クリッピングなどの表示属性

### Visualが必要なWidget
- テキストを表示する（TextBlock）
- 画像を表示する（Image）
- 背景色・枠線を持つ（Container with background）
- カスタム描画を行う

```

### Visual変更時に再描画をマーク
pub fn visual_changed_system(
    mut commands: Commands,
    query: Query<Entity, Changed<Visual>>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(NeedsRedraw);
    }
}
```

## Visualの動的作成

### Visualが必要なEntity
- テキストを表示する要素（`TextContent`コンポーネント）
- 画像を表示する要素（`ImageContent`コンポーネント）
- 背景や枠線を持つ要素（`ContainerStyle`コンポーネント）

### Visualが不要なEntity
- 純粋なレイアウトコンテナー（透明、背景なし）
- 論理的なグループ化のみ

### Visual コンポーネントの定義

```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Visual {
    // DirectCompositionオブジェクト（内部実装）
    pub dcomp_visual: IDCompositionVisual,
    
    // トランスフォーム（Visualが管理）
    pub offset: Point2D,
    pub opacity: f32,
    
    // DrawingContentへの参照
    pub drawing_content: Option<ID2D1Image>,
}
```

## システムの統合と更新フロー

bevy_ecsでは、システムの実行順序で依存関係を制御します。

```rust
use bevy_ecs::prelude::*;

/// Visualを必要に応じて追加
pub fn ensure_visual_system(
    mut commands: Commands,
    query: Query<Entity, (
        Or<(
            With<TextContent>,
            With<ImageContent>,
            With<ContainerStyle>,
        )>,
        Without<Visual>
    )>,
    dcomp_context: Res<DCompContext>,
) {
    for entity in query.iter() {
        let visual = Visual::new(&dcomp_context);
        commands.entity(entity).insert(visual);
    }
}

/// 描画コンテンツを再構築
pub fn rebuild_drawing_content_system(
    mut query: Query<(Entity, &mut Visual, &ComputedLayout), With<NeedsRedraw>>,
    text_query: Query<&TextContent>,
    image_query: Query<&ImageContent>,
    style_query: Query<&ContainerStyle>,
    dcomp_context: Res<DCompContext>,
) {
    for (entity, mut visual, layout) in query.iter_mut() {
        // Direct2Dコンテキストで描画
        let dc = create_drawing_context(&dcomp_context);
        
        // 各コンポーネントに応じて描画
        if let Ok(text) = text_query.get(entity) {
            draw_text(&dc, text, layout);
        }
        if let Ok(image) = image_query.get(entity) {
            draw_image(&dc, image, layout);
        }
        if let Ok(style) = style_query.get(entity) {
            draw_container_style(&dc, style, layout);
        }
        
        // 描画結果をVisualに設定
        visual.drawing_content = Some(finalize_drawing(&dc));
    }
}
```

### bevy_ecsによる型ベースのディスパッチ

bevy_ecsでは、クエリシステムが自動的に適切なEntityを選択します：

```rust
// TextContentを持つEntityだけ処理
pub fn process_text_system(query: Query<&TextContent>) {
    for text in query.iter() {
        // TextContentがあるEntityだけ自動選択
    }
}

// ImageContentを持つEntityだけ処理
pub fn process_image_system(query: Query<&ImageContent>) {
    for image in query.iter() {
        // ImageContentがあるEntityだけ自動選択
    }
}

// 両方持つEntityを処理
pub fn process_text_and_image_system(
    query: Query<(&TextContent, &ImageContent)>
) {
    for (text, image) in query.iter() {
        // 両方のコンポーネントを持つEntityのみ
    }
}
```

**bevy_ecsの利点**:
- ✅ コンポーネントの有無で自動フィルタリング
- ✅ 型安全：コンパイル時に検証
- ✅ パフォーマンス：クエリは最適化されている
- ✅ 柔軟性：コンポーネントの組み合わせを自由に指定
- ✅ 並列実行：独立したクエリは自動的に並列化
