# 第7章: システム統合と更新フロー

この章では、フレーム更新の流れについて説明します。

## bevy_ecsによる更新フロー

bevy_ecsでは、システムの実行順序（スケジューリング）で更新フローを制御します。

### フレーム更新の基本構造

```rust
use bevy_ecs::prelude::*;

pub fn setup_ui_update_systems(app: &mut App) {
    app.add_systems(Update, (
        // 1. 入力処理
        process_input_system,
        
        // 2. プロパティ変更検知
        (
            text_content_changed_system,
            image_content_changed_system,
            size_changed_system,
        ), // 並列実行可能
        
        // 3. レイアウト無効化
        invalidate_layout_system,
        propagate_layout_invalidation_system,
        
        // 4. レイアウト計算
        compute_layout_system,
        
        // 5. Visual管理
        (
            ensure_visual_system,
            layout_to_visual_system,
            attach_new_visual_system,
        ).chain(),
        
        // 6. 描画マーク
        visual_changed_system,
        
        // 7. 実際の描画
        draw_visual_system,
        
        // 8. DirectCompositionコミット
        commit_dcomp_system,
    ).chain()); // 全体を順番に実行
}
```

## 変更検知と伝播

### Changed<T>による自動検知

bevy_ecsの`Changed<T>`フィルタで変更を自動的に検知：

```rust
/// テキストが変更されたらレイアウト無効化
pub fn text_content_changed_system(
    mut commands: Commands,
    query: Query<Entity, Changed<TextContent>>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(LayoutInvalidated);
    }
}

/// 画像が変更されたらレイアウト無効化
pub fn image_content_changed_system(
    mut commands: Commands,
    query: Query<Entity, Changed<ImageContent>>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(LayoutInvalidated);
    }
}
```

### マーカーコンポーネントによる状態管理

複雑な更新フローはマーカーコンポーネントで制御：

```rust
#[derive(Component)]
pub struct LayoutInvalidated;

#[derive(Component)]
pub struct NeedsRedraw;

/// レイアウトが無効化されたものを計算
pub fn compute_layout_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ComputedLayout), With<LayoutInvalidated>>,
) {
    for (entity, mut layout) in query.iter_mut() {
        layout.compute();
        commands.entity(entity).remove::<LayoutInvalidated>();
    }
}

/// 再描画が必要なものを描画
pub fn draw_visual_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Visual), With<NeedsRedraw>>,
) {
    for (entity, mut visual) in query.iter_mut() {
        visual.draw();
        commands.entity(entity).remove::<NeedsRedraw>();
    }
}
```

## 他のUIフレームワークとの比較

### 依存管理戦略の比較

| フレームワーク | 戦略 | 依存解決 | Rust実装 |
|------------|------|---------|---------|
| **bevy_ecs (本設計)** | Changed<T> + マーカー | システムチェーン + クエリ | ✅ ネイティブ |
| **Flutter** | RenderObjectツリー + マーキング | `markNeedsLayout()`/`markNeedsPaint()` | 🟡 要移植 |
| **React** | 仮想DOM + Reconciliation | 変更検知→再レンダリング | 🟡 要移植 |
| **SwiftUI** | @State/@Binding | プロパティラッパー自動追跡 | 🔴 Swift専用 |
| **ImGui** | 即時モード | 毎フレーム全再描画 | ✅ 実装容易 |

### bevy_ecsの位置づけ

**本設計の特徴**:
- **自動変更追跡**: `Changed<T>`で自動検知（SwiftUI的）
- **明示的な状態遷移**: マーカーコンポーネント（Flutter/Godot的）
- **システム分離**: ECS原則に忠実
- **並列処理**: 自動並列実行

## ECS原則による依存管理

### コンポーネントベースの依存宣言

**核心的アイデア**: Entityが「どのコンポーネントを持つか」で依存関係が決まる。

```rust
// TextContentを持つ → レイアウトに影響
Query<Entity, Changed<TextContent>>

// ImageContentを持つ → レイアウトに影響
Query<Entity, Changed<ImageContent>>

// ComputedLayoutを持つ → Visualに影響
Query<Entity, Changed<ComputedLayout>>

// Visualを持つ → 再描画が必要
Query<Entity, Changed<Visual>>
```

### 型安全な依存管理

bevy_ecsのクエリシステムは**コンパイル時に検証**されます：

```rust
// ✅ OK: TextContentとLayoutを持つEntityのみ
Query<(&TextContent, &mut ComputedLayout)>

// ❌ コンパイルエラー: Layoutがない可能性がある
// Query<&TextContent> で mut ComputedLayout にアクセス不可

// ✅ OK: Optionで安全に処理
Query<(&TextContent, Option<&mut ComputedLayout>)>
```

## カスタム描画の実装

### traitベースのカスタムレンダラー

```rust
use bevy_ecs::prelude::*;

/// カスタム描画を行うコンポーネント
#[derive(Component)]
pub struct CustomRenderer {
    pub renderer: Box<dyn Render>,
}

pub trait Render: Send + Sync {
    fn render(&self, ctx: &RenderContext) -> Result<()>;
}

/// グラデーション描画の例
struct GradientRenderer {
    colors: Vec<Color>,
}

impl Render for GradientRenderer {
    fn render(&self, ctx: &RenderContext) -> Result<()> {
        // カスタム描画ロジック
        Ok(())
    }
}

/// カスタムレンダラーを持つEntityの描画
pub fn custom_render_system(
    query: Query<(&CustomRenderer, &ComputedLayout, &Visual)>,
    render_context: Res<RenderContext>,
) {
    for (renderer, layout, visual) in query.iter() {
        renderer.renderer.render(&render_context).ok();
    }
}
```

### システムの依存関係

bevy_ecsでは、システムの実行順序で依存を表現：

```rust
app.add_systems(Update, (
    // カスタムレンダラーはレイアウト後に実行
    compute_layout_system,
    custom_render_system.after(compute_layout_system),
).chain());
```