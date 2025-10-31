# Entityツリー構造

## 基本的な考え方

- UIではツリー構造を管理する
- ツリー構造をRustで管理しようとするとArcなどを使う必要がありコードが煩雑になる
- bevy_ecsの`Entity`は軽量なID（Copy可能な数値）であり、参照の代わりに使える
- Entityはただの数値なので、Arcなどの重いデータ構造を持たずにツリー構造を管理できる
- メモリ管理が配列ベースになり、キャッシュ効率が向上する
- bevy_ecsはコンポーネントの特性に応じて最適なストレージ（Table/SparseSet）を選択する
- すべてのUI要素をEntityとして管理するシステムを基本とする

## UIツリーを表現する親子関係

### 親子関係の役割
- 親子関係はUIツリーのノードを接続する
- Entityはbevy_ecsによって自動管理される一意のID（世代付きインデックス）
- Entityは`Copy`トレイトを実装しており、軽量にコピー・参照できる
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

bevy_ecsの`Entity`は軽量で効率的なID型です。

**特徴**:
- 内部的には64bit整数（u32のインデックス + u32の世代）
- `Copy`トレイトを実装しており、軽量にコピー可能
- 所有権やライフタイムの問題がない
- `Arc<T>`や`Rc<T>`のようなオーバーヘッドがない
- 世代カウンタにより、無効なEntityへのアクセスを検出可能

**使用例**:

```rust
use bevy_ecs::prelude::*;

let entity1: Entity = commands.spawn((...)).id();
let entity2 = entity1;

#[derive(Component)]
pub struct Parent(pub Entity);

#[derive(Component)]
pub struct Children(pub Vec<Entity>);
```

**利点**:
- ✅ **軽量**: ただの64bit整数
- ✅ **Copy可能**: 参照カウントなしでコピー
- ✅ **安全**: 世代により無効なEntityへのアクセスを検出
- ✅ **効率的**: メモリ局所性が高く、キャッシュフレンドリー

### 親子関係コンポーネントの定義

### 親子関係コンポーネントの定義

bevy_hierarchyが標準で提供するコンポーネント：

**注意**: 
- `Parent`コンポーネントは親を持つEntityのみが持つ
- ルートEntity（Window）は`Parent`コンポーネントを持たない

```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Parent(pub Entity);

#[derive(Component)]
pub struct Children(pub Vec<Entity>);

#[derive(Component)]
pub struct Window {
    pub hwnd: HWND,
}
```

### Entityの構造

ルートEntityと子Entityの構成例：

```rust
commands.spawn((
    Window { hwnd },
    Children(vec![...]),
));

commands.spawn((
    TextContent { ... },
    Parent(window_entity),
));

fn process_entity(entity: Entity, parent: Entity) {
    println!("Processing: {:?}, Parent: {:?}", entity, parent);
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

### 親をたどる

bevy_ecsでは、`Parent`コンポーネントの有無でルートかどうかを判定：

```rust
/// 親をたどってWindowを見つける
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
        
        // 親をたどる（Parentコンポーネントがあれば）
        if let Ok(parent) = parent_query.get(current) {
            current = parent.0;
        } else {
            // Parentコンポーネントがない = ルートに到達
            return None;
        }
    }
}

/// ルートEntityを判定
pub fn is_root(entity: Entity, parent_query: Query<&Parent>) -> bool {
    // Parentコンポーネントを持たない = ルート
    parent_query.get(entity).is_err()
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
