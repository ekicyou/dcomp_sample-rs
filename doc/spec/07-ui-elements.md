# 基本的なUI要素 (bevy_ecs版)

## UI要素の構成

bevy_ecsでは、UI要素を複数のComponentの組み合わせで表現します。

### コンポーネントの組み合わせパターン

| UI要素 | 必須コンポーネント | オプションコンポーネント |
|--------|------------------|----------------------|
| **Container** | `Parent`, `Children` | `ContainerStyle`, `Size`, `Padding` |
| **TextBlock** | `TextContent` | `Size`, `Margin`, `TextStyle` |
| **Image** | `ImageContent` | `Size`, `Margin`, `Stretch` |
| **Button** | `Clickable`, `InteractionState` | `ContainerStyle`, `TextContent` |
| **StackPanel** | `LayoutType::Stack`, `Children` | `Orientation`, `Spacing` |

### Container（コンテナ）

レイアウトコンテナの作成例：

```rust
use bevy_ecs::prelude::*;

pub fn create_container(mut commands: Commands) -> Entity {
    commands.spawn((
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
        ContainerStyle {
            background: Some(Brush::SolidColor(Color::WHITE)),
            border: Some(Border {
                thickness: 1.0,
                color: Color::GRAY,
            }),
            corner_radius: 5.0,
        },
        ComputedLayout::default(),
        Name::new("Container"),
    )).id()
}
```

### TextBlock（テキスト）

テキスト表示要素の作成例：

```rust
pub fn create_text_block(mut commands: Commands, text: &str) -> Entity {
    commands.spawn((
        TextContent {
            text: text.to_string(),
            font_family: "Segoe UI".to_string(),
            font_size: 14.0,
            color: Color::BLACK,
            text_layout: None,
        },
        Size {
            width: Length::Auto,
            height: Length::Auto,
        },
        Margin {
            left: 0.0,
            top: 5.0,
            right: 0.0,
            bottom: 5.0,
        },
        
        // 計算結果
        ComputedLayout::default(),
        
        // Visual（自動追加される）
        // ensure_visual_systemが追加
        
        Name::new("TextBlock"),
    )).id()
}
```

### Image（画像）

```rust
/// Imageを作成
pub fn create_image(mut commands: Commands, source: &str) -> Entity {
    commands.spawn((
        // 画像コンテンツ
        ImageContent {
            source: source.to_string(),
            bitmap: None, // 後でロード
            stretch: Stretch::Uniform,
        },
        
        // サイズ
        Size {
            width: Length::Pixels(100.0),
            height: Length::Pixels(100.0),
        },
        SizeConstraints {
            min_width: Some(50.0),
            max_width: Some(200.0),
            min_height: Some(50.0),
            max_height: Some(200.0),
        },
        
        ComputedLayout::default(),
        Name::new("Image"),
    )).id()
}
```

### Button（ボタン）

```rust
/// Buttonを作成
pub fn create_button(
    mut commands: Commands,
    label: &str,
    on_click: impl Fn() + Send + Sync + 'static,
) -> Entity {
    commands.spawn((
        // インタラクション
        Clickable {
            on_click: Some(Box::new(on_click)),
        },
        InteractionState {
            is_hovered: false,
            is_pressed: false,
            is_focused: false,
        },
        
        // スタイル
        ContainerStyle {
            background: Some(Brush::SolidColor(Color::BUTTON_FACE)),
            border: Some(Border {
                thickness: 1.0,
                color: Color::BUTTON_SHADOW,
            }),
            corner_radius: 3.0,
        },
        
        // レイアウト
        Size {
            width: Length::Pixels(100.0),
            height: Length::Pixels(30.0),
        },
        Padding::uniform(10.0),
        
        ComputedLayout::default(),
        Name::new("Button"),
    ))
    .with_children(|parent| {
        // ボタンラベル
        parent.spawn((
            TextContent {
                text: label.to_string(),
                font_size: 14.0,
                color: Color::BUTTON_TEXT,
                ..default()
            },
            Size::auto(),
            ComputedLayout::default(),
        ));
    })
    .id()
}
```

### StackPanel（スタックパネル）

```rust
/// StackPanelを作成
pub fn create_stack_panel(
    mut commands: Commands,
    orientation: Orientation,
) -> Entity {
    commands.spawn((
        // レイアウトタイプ
        LayoutType::Stack(StackLayout {
            orientation,
            spacing: 5.0,
        }),
        
        // サイズ
        Size {
            width: Length::Auto,
            height: Length::Auto,
        },
        
        ComputedLayout::default(),
        Name::new("StackPanel"),
    )).id()
}
```

## 複雑なUI要素の構築

### 複数コンポーネントの組み合わせ

bevy_ecsでは、必要なコンポーネントを自由に組み合わせられます：

```rust
/// 背景+テキスト+画像アイコンを持つ複雑な要素
pub fn create_rich_content(mut commands: Commands) -> Entity {
    commands.spawn((
        // 背景
        ContainerStyle {
            background: Some(Brush::LinearGradient {
                start: Color::LIGHT_BLUE,
                end: Color::DARK_BLUE,
            }),
            corner_radius: 10.0,
            ..default()
        },
        
        // テキスト（同じEntityに複数のコンテンツは不可なので子として追加）
        Size {
            width: Length::Pixels(300.0),
            height: Length::Auto,
        },
        Padding::uniform(15.0),
        
        ComputedLayout::default(),
    ))
    .with_children(|parent| {
        // アイコン
        parent.spawn((
            ImageContent {
                source: "icon.png".to_string(),
                bitmap: None,
                stretch: Stretch::None,
            },
            Size {
                width: Length::Pixels(32.0),
                height: Length::Pixels(32.0),
            },
            Margin {
                right: 10.0,
                ..default()
            },
            ComputedLayout::default(),
        ));
        
        // テキスト
        parent.spawn((
            TextContent {
                text: "Title".to_string(),
                font_size: 16.0,
                color: Color::WHITE,
                ..default()
            },
            Size::auto(),
            ComputedLayout::default(),
        ));
    })
    .id()
}
```

## ECS原則の利点

### 1. データとロジックの完全分離

```rust
// データ（Component）
#[derive(Component)]
pub struct TextContent {
    pub text: String,
    pub font_size: f32,
}

// ロジック（System）
pub fn render_text_system(query: Query<&TextContent>) {
    for text in query.iter() {
        // 描画ロジック
    }
}
```

### 2. 組み合わせ可能性

1つのEntityが複数の機能を持てる：

```rust
// Container + Clickable + Hoverable
commands.spawn((
    ContainerStyle::default(),
    Clickable::default(),
    InteractionState::default(),
    // ...
));
```

### 3. 動的な機能追加/削除

```rust
// 実行時にコンポーネントを追加
commands.entity(entity).insert(Visual::default());

// 実行時にコンポーネントを削除
commands.entity(entity).remove::<Visual>();
```

### 4. クエリによる効率的な処理

```rust
// TextContentを持つEntityだけ処理
Query<&TextContent>

// TextContent + Visualを持つEntityだけ処理
Query<(&TextContent, &Visual)>

// TextContentを持つが、Visualを持たないEntityを処理
Query<&TextContent, Without<Visual>>
```
