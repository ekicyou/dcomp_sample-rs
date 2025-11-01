# 第4章: レイアウトシステム

この章では、レイアウトコンポーネントの定義について説明します。

## bevy_ecsによるレイアウト管理

bevy_ecsでは、レイアウト関連のプロパティをComponentとして定義し、必要なEntityだけが各Componentを持ちます。

### レイアウトコンポーネントの定義

```rust
use bevy_ecs::prelude::*;

// ========================================
// サイズ制約コンポーネント
// ========================================

/// 要素のサイズ指定
#[derive(Component)]
pub struct Size {
    pub width: Length,
    pub height: Length,
}

/// サイズ制約
#[derive(Component)]
pub struct SizeConstraints {
    pub min_width: Option<f32>,
    pub max_width: Option<f32>,
    pub min_height: Option<f32>,
    pub max_height: Option<f32>,
}

// ========================================
// 間隔コンポーネント
// ========================================

/// 外側の余白
#[derive(Component)]
pub struct Margin {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

/// 内側の余白
#[derive(Component)]
pub struct Padding {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

// ========================================
// 配置コンポーネント
// ========================================

/// 配置方法
#[derive(Component)]
pub struct Alignment {
    pub horizontal: HorizontalAlignment,
    pub vertical: VerticalAlignment,
}

// ========================================
// レイアウト結果（計算されたキャッシュ）
// ========================================

/// レイアウト計算結果
#[derive(Component)]
pub struct ComputedLayout {
    pub desired_size: Size2D,
    pub final_rect: Rect,
}

// ========================================
// 型定義
// ========================================

pub enum Length {
    Auto,
    Pixels(f32),
    Percent(f32),
}

pub enum HorizontalAlignment {
    Left,
    Center,
    Right,
    Stretch,
}

pub enum VerticalAlignment {
    Top,
    Center,
    Bottom,
    Stretch,
}
    End,
    Stretch,
}

// ========================================
// レイアウトタイプ
// ========================================

/// レイアウトの種類
#[derive(Component)]
pub enum LayoutType {
    None,
    Stack(StackLayout),
    // 将来的に追加
    // Grid(GridLayout),
    // Flex(FlexLayout),
}

pub struct StackLayout {
    pub orientation: Orientation,
    pub spacing: f32,
}

pub enum Orientation {
    Horizontal,
    Vertical,
}
```

### レイアウトシステムの実装

```rust
use bevy_ecs::prelude::*;

/// レイアウト計算システム
pub fn compute_layout_system(
    mut query: Query<
        (
            Entity,
            &Size,
            Option<&SizeConstraints>,
            Option<&Margin>,
            Option<&Padding>,
            &mut ComputedLayout,
        ),
        With<LayoutInvalidated>,
    >,
    children_query: Query<&Children>,
    mut commands: Commands,
) {
    for (entity, size, constraints, margin, padding, mut computed) in query.iter_mut() {
        // Measure phase: 望ましいサイズを計算
        let desired_size = measure_entity(
            entity,
            size,
            constraints,
            &children_query,
        );
        
        // Arrange phase: 最終的な配置矩形を決定
        let final_rect = arrange_entity(
            desired_size,
            margin,
            padding,
        );
        
        computed.desired_size = desired_size;
        computed.final_rect = final_rect;
        
        // レイアウト完了のマーカーを削除
        commands.entity(entity).remove::<LayoutInvalidated>();
    }
}

/// プロパティ変更時にレイアウトを無効化
pub fn invalidate_layout_on_size_change(
    mut commands: Commands,
    query: Query<Entity, Or<(
        Changed<Size>,
        Changed<SizeConstraints>,
        Changed<Margin>,
        Changed<Padding>,
        Changed<LayoutType>,
    )>>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(LayoutInvalidated);
    }
}

/// 子要素が変更されたら親のレイアウトを無効化
pub fn invalidate_parent_on_children_change(
    mut commands: Commands,
    changed_query: Query<&Parent, Changed<Children>>,
) {
    for parent in changed_query.iter() {
        commands.entity(parent.get()).insert(LayoutInvalidated);
    }
}
```

### 使用例

```rust
use bevy_ecs::prelude::*;

pub fn create_layout_example(mut commands: Commands) {
    // コンテナを作成
    let container = commands.spawn((
        Size {
            width: Length::Pixels(400.0),
            height: Length::Auto,
        },
        Padding {
            left: 10.0,
            top: 10.0,
            right: 10.0,
            bottom: 10.0,
        },
        LayoutType::Stack(StackLayout {
            orientation: Orientation::Vertical,
            spacing: 5.0,
        }),
        ComputedLayout::default(),
    )).id();
    
    // 子要素を作成
    let child1 = commands.spawn((
        Size {
            width: Length::Percent(100.0),
            height: Length::Pixels(50.0),
        },
        Margin {
            left: 0.0,
            top: 0.0,
            right: 0.0,
            bottom: 5.0,
        },
        ComputedLayout::default(),
    )).id();
    
    commands.entity(container).add_child(child1);
}
```

### レイアウト計算の流れ

1. **無効化**: プロパティ変更時に`LayoutInvalidated`マーカーを追加
2. **計算**: `compute_layout_system`で`LayoutInvalidated`を持つEntityを処理
3. **伝播**: `ComputedLayout`が`Changed`になり、Visualシステムが反応
4. **完了**: `LayoutInvalidated`を削除

```rust
// システムスケジュール
app.add_systems(Update, (
    // 1. レイアウト無効化
    invalidate_layout_on_size_change,
    invalidate_parent_on_children_change,
    
    // 2. レイアウト計算
    compute_layout_system,
    
    // 3. Visual更新（WinVisual.mdで定義）
    layout_to_visual_system,
).chain());
```

### Componentの組み合わせパターン

| UI要素 | Size | Constraints | Margin | Padding | LayoutType |
|--------|------|-------------|--------|---------|------------|
| TextBlock | ✓ | ✓ | ✓ | - | - |
| Image | ✓ | ✓ | ✓ | - | - |
| Container | ✓ | ✓ | ✓ | ✓ | ✓ |
| StackPanel | ✓ | - | ✓ | ✓ | ✓ (Stack) |

必要なComponentだけを追加することで、メモリ効率が向上します。
