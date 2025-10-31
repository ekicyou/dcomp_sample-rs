# Entityツリー構造

## 基本的な考え方

- UIではツリー構造を管理することが多い
- ツリー構造をrustで管理しようとするとArcなどを使う必要がありコードが煩雑になる
- bevy_ecsでは、`Entity`でオブジェクトにアクセスし、ツリー構造もEntityで管理することで参照関係の管理を整理する
- また、メモリ管理が配列ベースになり、キャッシュに乗りやすくなることも期待される
- bevy_ecsは内部的にスパースセットで効率的にEntityとComponentを管理する
- すべてのUI要素をEntityとして管理するシステムを基本とする

## UIツリーを表現する親子関係

### 親子関係の役割
- 親子関係はUIツリーのノードを接続する
- Entityはbevy_ecsによって自動管理される（世代付きインデックス）
- 親子関係は`Parent`と`Children`コンポーネントで管理（bevy_hierarchyが提供）
- **WindowもEntityであり、UIツリーのルート要素となる**

### Windowの特殊性

Windowは他のUI要素（TextBlock、Image、Containerなど）と同様にEntityとして扱われるが、以下の点で特別：

1. **ルートEntity**: Windowは常にUIツリーのルート（Parentコンポーネントを持たない）
2. **OSウィンドウとの関連**: HWNDと1:1で対応
3. **Windowマーカー**: `Window`コンポーネントで識別
4. **DirectComposition接続点**: ウィンドウのDCompTargetがビジュアルツリーの起点

```rust
// 概念的な構造
Window (Entity)                    // Windowコンポーネントを持つ
  └─ Container (Entity)            // レイアウトコンテナ
       ├─ TextBlock (Entity)       // テキスト要素
       └─ Image (Entity)           // 画像要素
```

### Entityの型

```rust
use bevy_ecs::prelude::*;

// bevy_ecsが提供する標準型
// Entity: 世代付きインデックス (Generation + Index)
// 自動的に管理され、明示的な型定義は不要
```

### 親子関係コンポーネントの定義

```rust
use bevy_ecs::prelude::*;

// bevy_hierarchyが提供（標準）
#[derive(Component)]
pub struct Parent(pub Entity);

#[derive(Component)]
pub struct Children(pub Vec<Entity>);

// Windowマーカー
#[derive(Component)]
pub struct Window {
    pub hwnd: HWND,
}
```

### 親子関係の操作

bevy_hierarchyが提供する標準的なコマンドを使用。

**主な操作**:
- `commands.entity(parent).add_child(child)`: 子Entityを追加
- `commands.entity(parent).push_children(&[child1, child2])`: 複数の子を追加
- `commands.entity(child).remove_parent()`: 親子関係を切断
- `commands.entity(entity).despawn_recursive()`: Entityと子を再帰的に削除

## UI要素の作成

bevy_ecsのCommandsを使ってEntityを作成。

```rust
use bevy_ecs::prelude::*;

pub fn create_ui(mut commands: Commands) {
    // Windowを作成
    let window = commands.spawn((
        Window { hwnd: create_hwnd() },
        Name::new("MainWindow"),
    )).id();
    
    // Containerを作成してWindowの子にする
    let container = commands.spawn((
        Layout::default(),
        ContainerStyle::default(),
        Name::new("RootContainer"),
    )).id();
    commands.entity(window).add_child(container);
    
    // TextBlockを作成
    let text = commands.spawn((
        TextContent {
            text: "Hello".into(),
            font_size: 16.0,
            ..default()
        },
        Layout::default(),
        Name::new("TextBlock1"),
    )).id();
    commands.entity(container).add_child(text);
}
```

### Entityの走査

bevy_ecsのクエリシステムでツリーを走査。

```rust
use bevy_ecs::prelude::*;

/// すべての子Entityを走査
pub fn traverse_children_system(
    parent_query: Query<&Children>,
    entity_query: Query<&Name>,
) {
    fn visit(entity: Entity, parent_query: &Query<&Children>, entity_query: &Query<&Name>) {
        if let Ok(name) = entity_query.get(entity) {
            println!("Entity: {}", name.value);
        }
        
        // 子を再帰的に訪問
        if let Ok(children) = parent_query.get(entity) {
            for child in children.iter() {
                visit(*child, parent_query, entity_query);
            }
        }
    }
}

/// 親をたどる
pub fn find_window_system(
    entity: Entity,
    parent_query: Query<&Parent>,
    window_query: Query<&Window>,
) -> Option<Entity> {
    let mut current = entity;
    
    loop {
        // Windowコンポーネントを持つならそれがルート
        if window_query.get(current).is_ok() {
            return Some(current);
        }
        
        // 親をたどる
        if let Ok(parent) = parent_query.get(current) {
            current = parent.get();
        } else {
            return None; // ルートに到達（Parentなし）
        }
    }
}
```

## Worldによる管理

bevy_ecsでは`World`がすべてのEntityとComponentを管理。

```rust
use bevy_ecs::prelude::*;

// Worldの作成
let mut world = World::new();

// Entityの作成
let entity = world.spawn((
    TextContent { text: "Hello".into(), ..default() },
    Layout::default(),
)).id();

// Componentの取得
if let Some(text) = world.get::<TextContent>(entity) {
    println!("{}", text.text);
}

// Componentの追加/削除
world.entity_mut(entity).insert(Visual::default());
world.entity_mut(entity).remove::<Visual>();

// Entityの削除
world.despawn(entity);
```
