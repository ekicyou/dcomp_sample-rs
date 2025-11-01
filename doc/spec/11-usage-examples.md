# 第11章: 使用例

この章では、実際のサンプルコードを示します。

## 基本的な使用例

### シンプルなウィンドウとテキスト

```rust
use bevy_ecs::prelude::*;

pub fn create_simple_window(mut commands: Commands, dcomp_context: Res<DCompContext>) {
    // Windowを作成
    let window = commands.spawn((
        Window {
            hwnd: create_hwnd(),
        },
        Name::new("MainWindow"),
    )).id();
    
    // TextBlockを作成してWindowの子にする
    let text = commands.spawn((
        TextContent {
            text: "Hello, bevy_ecs!".to_string(),
            font_family: "Segoe UI".to_string(),
            font_size: 24.0,
            color: Color::BLACK,
            text_layout: None,
        },
        Size {
            width: Length::Auto,
            height: Length::Auto,
        },
        ComputedLayout::default(),
        Name::new("HelloText"),
    )).id();
    
    commands.entity(window).add_child(text);
}
```

### ボタン付きのUI

Window、Container、複数のボタンを含むUI例：

```rust
pub fn create_button_example(mut commands: Commands) {
    let window = commands.spawn((
        Window { hwnd: create_hwnd() },
        Name::new("ButtonWindow"),
    )).id();
    
    let container = commands.spawn((
        Size {
            width: Length::Pixels(400.0),
            height: Length::Pixels(300.0),
        },
        Padding::uniform(20.0),
        LayoutType::Stack(StackLayout {
            orientation: Orientation::Vertical,
            spacing: 10.0,
        }),
        ComputedLayout::default(),
    )).id();
    
    commands.entity(window).add_child(container);
    
    let button1 = create_button_with_label(&mut commands, "Click Me!", || {
        println!("Button 1 clicked!");
    });
    
    let button2 = create_button_with_label(&mut commands, "Another Button", || {
        println!("Button 2 clicked!");
    });
    
    commands.entity(container)
        .push_children(&[button1, button2]);
}
```

ボタンの作成ヘルパー関数：

```rust
fn create_button_with_label(
    commands: &mut Commands,
    label: &str,
    on_click: impl Fn() + Send + Sync + 'static,
) -> Entity {
    commands.spawn((
        Clickable {
            on_click: Some(Box::new(on_click)),
        },
        InteractionState::default(),
        ContainerStyle {
            background: Some(Brush::SolidColor(Color::rgb(220, 220, 220))),
            border: Some(Border {
                thickness: 1.0,
                color: Color::rgb(150, 150, 150),
            }),
            corner_radius: 3.0,
        },
        Size {
            width: Length::Auto,
            height: Length::Pixels(30.0),
        },
        Padding::uniform(10.0),
        ComputedLayout::default(),
    ))
    .with_children(|parent| {
        parent.spawn((
            TextContent {
                text: label.to_string(),
                font_size: 14.0,
                color: Color::BLACK,
                ..default()
            },
            Size::auto(),
            ComputedLayout::default(),
        ));
    })
    .id()
}
```

## レイアウトシステムの使用例

### 2パスレイアウト

bevy_ecsでは、システムの順序制御で2パスレイアウトを実現します。

**システムセットアップ**:

```rust
use bevy_ecs::prelude::*;

pub fn setup_layout_systems(app: &mut App) {
    app.add_systems(Update, (
        measure_system,
        propagate_measure_to_parent_system,
        arrange_system,
        propagate_arrange_to_children_system,
        layout_to_visual_system,
    ).chain());
}
```

**Measureパス**（子から親へ、必要なサイズを計算）:

```rust
pub fn measure_system(
    mut query: Query<(&mut ComputedLayout, &Size, Option<&Children>), With<LayoutInvalidated>>,
    children_query: Query<&ComputedLayout>,
) {
    for (mut layout, size, children) in query.iter_mut() {
        let children_size = if let Some(children) = children {
            calculate_children_desired_size(children, &children_query)
        } else {
            Size2D::zero()
        };
        
        layout.desired_size = apply_size_constraints(size, children_size);
    }
}
```

**Arrangeパス**（親から子へ、最終位置を決定）:

```rust
pub fn arrange_system(
    mut query: Query<(&mut ComputedLayout, &Size, &ComputedLayout, &Parent)>,
    parent_query: Query<&ComputedLayout>,
) {
    for (mut layout, size, computed, parent) in query.iter_mut() {
        if let Ok(parent_layout) = parent_query.get(parent.get()) {
            layout.final_rect = calculate_final_rect(
                parent_layout.final_rect,
                computed.desired_size,
                size,
            );
        }
    }
}
```

### StackPanelの実装例

StackLayoutの子サイズ計算ロジック：

```rust
pub fn measure_stack_panel(
    mut query: Query<(&mut ComputedLayout, &LayoutType, &Children)>,
    children_query: Query<&ComputedLayout>,
) {
    for (mut layout, layout_type, children) in query.iter_mut() {
        if let LayoutType::Stack(stack) = layout_type {
            let mut total_size = Size2D::zero();
            
            for child in children.iter() {
                if let Ok(child_layout) = children_query.get(*child) {
                    match stack.orientation {
                        Orientation::Vertical => {
                            total_size.width = total_size.width.max(child_layout.desired_size.width);
                            total_size.height += child_layout.desired_size.height + stack.spacing;
                        }
                        Orientation::Horizontal => {
                            total_size.width += child_layout.desired_size.width + stack.spacing;
                            total_size.height = total_size.height.max(child_layout.desired_size.height);
                        }
                    }
                }
            }
            
            if !children.is_empty() {
                match stack.orientation {
                    Orientation::Vertical => total_size.height -= stack.spacing,
                    Orientation::Horizontal => total_size.width -= stack.spacing,
                }
            }
            
            layout.desired_size = total_size;
        }
    }
}
```

## 完全な例：カウンターアプリ

インタラクティブなカウンターアプリケーションの実装例です。

**コンポーネント定義**:

```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
struct Counter {
    value: i32,
}

#[derive(Component)]
struct CounterDisplay;
```

**UI構築**:

```rust
pub fn create_counter_app(mut commands: Commands) {
    let window = commands.spawn((
        Window { hwnd: create_hwnd() },
        Name::new("CounterApp"),
    )).id();
    
    let container = commands.spawn((
        Size {
            width: Length::Pixels(200.0),
            height: Length::Auto,
        },
        Padding::uniform(20.0),
        LayoutType::Stack(StackLayout {
            orientation: Orientation::Vertical,
            spacing: 10.0,
        }),
        ComputedLayout::default(),
    )).id();
    
    let display = commands.spawn((
        TextContent {
            text: "Count: 0".to_string(),
            font_size: 24.0,
            color: Color::BLACK,
            ..default()
        },
        Size::auto(),
        ComputedLayout::default(),
        CounterDisplay,
        Counter { value: 0 },
    )).id();
    
    let inc_button = create_button_with_label(&mut commands, "+1", move || {
        increment_counter(display);
    });
    
    let dec_button = create_button_with_label(&mut commands, "-1", move || {
        decrement_counter(display);
    });
    
    commands.entity(window).add_child(container);
    commands.entity(container).push_children(&[display, inc_button, dec_button]);
}
```

**カウンター更新システム**（`Changed<Counter>`で自動検知）:

```rust
pub fn update_counter_display_system(
    mut query: Query<(&Counter, &mut TextContent), (Changed<Counter>, With<CounterDisplay>)>,
) {
    for (counter, mut text) in query.iter_mut() {
        text.text = format!("Count: {}", counter.value);
    }
}

pub fn setup_counter_systems(app: &mut App) {
    app.add_systems(Update, update_counter_display_system);
}
```

## システムスケジュールの全体像

全システムの実行順序を定義：

```rust
pub fn setup_all_systems(app: &mut App) {
    app.insert_resource(DCompContext::new())
        .insert_resource(MousePosition::default())
        .insert_resource(Time::default());
    
    app.add_systems(Update, (
        process_mouse_input,
        process_keyboard_input,
        hover_detection_system,
        click_system,
        text_content_changed_system,
        image_content_changed_system,
        invalidate_layout_system,
        measure_system,
        arrange_system,
        ensure_visual_system,
        layout_to_visual_system,
        attach_visual_to_tree_system,
        visual_changed_system,
        draw_visual_system,
        commit_dcomp_system,
        update_counter_display_system,
    ).chain());
}
```
