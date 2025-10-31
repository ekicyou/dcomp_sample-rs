# slotmap → bevy_ecs 移行完了サマリー

## 更新日時
2025-11-01

## 移行概要

UIフレームワークの設計ドキュメント全体を、slotmapベースの実装からbevy_ecsベースの実装に全面的に書き換えました。

## 更新されたファイル

### 主要ドキュメント

1. **README.md** - タイトルとサマリーを更新
2. **WinVisual.md** - DirectCompositionとbevy_ecsの統合、ダーティ検出戦略
3. **01-widget-tree.md** → **Entityツリー構造**
4. **02-ecs-components.md** - bevy_ecsコンポーネント管理
5. **03-layout-system.md** - レイアウトコンポーネント
6. **04-visual-directcomp.md** - Visual統合
7. **05-update-flow.md** - システム統合と更新フロー
8. **07-ui-elements.md** - 基本的なUI要素
9. **10-usage-examples.md** - 使用例
10. **13-system-separation.md** - システム設計

## 用語の統一

### 旧 (slotmap) → 新 (bevy_ecs)

| 概念 | slotmap | bevy_ecs |
|------|---------|----------|
| **識別子** | `WidgetId` | `Entity` |
| **ツリー構造** | `Widget` 構造体（連結リスト） | `Parent`/`Children` コンポーネント |
| **データ管理** | `SecondaryMap<WidgetId, T>` | `#[derive(Component)] struct T` |
| **システム** | `impl WidgetSystem { fn method() }` | `pub fn system(query: Query<...>)` |
| **ダーティ管理** | `dirty: HashSet<WidgetId>` | `Changed<T>` + マーカーコンポーネント |
| **依存関係** | `DependencyMap` | システムチェーン + `Changed<T>` |
| **ランタイム** | `UiRuntime` 構造体 | `World` + `App` |

## 主要な設計変更

### 1. Entity管理

**旧**:
```rust
new_key_type! { pub struct WidgetId; }
let widget_id = widget_system.create_widget();
```

**新**:
```rust
let entity = commands.spawn((
    Name::new("MyEntity"),
    // コンポーネント
)).id();
```

### 2. コンポーネント定義

**旧**:
```rust
pub struct LayoutSystem {
    width: SecondaryMap<WidgetId, Length>,
    height: SecondaryMap<WidgetId, Length>,
    dirty: HashSet<WidgetId>,
}
```

**新**:
```rust
#[derive(Component)]
pub struct Size {
    pub width: Length,
    pub height: Length,
}

#[derive(Component)]
pub struct LayoutInvalidated;
```

### 3. システム実装

**旧**:
```rust
impl LayoutSystem {
    pub fn set_width(&mut self, widget_id: WidgetId, width: Length) {
        self.width.insert(widget_id, width);
        self.dirty.insert(widget_id);
    }
}
```

**新**:
```rust
pub fn size_changed_system(
    mut commands: Commands,
    query: Query<Entity, Changed<Size>>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(LayoutInvalidated);
    }
}
```

### 4. 変更検知

**旧**: 手動でダーティフラグを管理
```rust
self.dirty.insert(widget_id);
```

**新**: 自動変更追跡
```rust
Query<&TextContent, Changed<TextContent>>
```

### 5. 親子関係

**旧**: 連結リスト構造
```rust
struct Widget {
    parent: Option<WidgetId>,
    first_child: Option<WidgetId>,
    next_sibling: Option<WidgetId>,
}
```

**新**: bevy_hierarchy標準
```rust
commands.entity(parent).add_child(child);
```

## 追加された概念

### 1. マーカーコンポーネント

状態管理のための空のコンポーネント：
```rust
#[derive(Component)]
pub struct LayoutInvalidated;

#[derive(Component)]
pub struct NeedsRedraw;

#[derive(Component)]
pub struct NeedsTransformUpdate;
```

### 2. システムスケジューリング

実行順序の明示的な制御：
```rust
app.add_systems(Update, (
    text_changed_system,
    invalidate_layout_system,
    compute_layout_system,
    draw_system,
).chain());
```

### 3. 変更検知フィルタ

- `Changed<T>`: コンポーネントが変更されたEntity
- `Added<T>`: コンポーネントが追加されたEntity
- `With<T>`: コンポーネントを持つEntity
- `Without<T>`: コンポーネントを持たないEntity

## 利点の整理

### bevy_ecs採用による利点

1. **自動変更追跡**: `Changed<T>`で手動フラグ不要
2. **型安全**: コンパイル時にクエリを検証
3. **並列実行**: 独立したシステムは自動的に並列化
4. **メモリ効率**: スパースセットで効率的に管理
5. **柔軟性**: 実行時にコンポーネントの追加/削除
6. **保守性**: データとロジックの完全分離
7. **エコシステム**: bevy_hierarchyなど標準機能を活用

### データ指向設計

- コンポーネントはデータのみ（純粋なデータ構造）
- システムはロジックのみ（処理関数）
- キャッシュ効率の良いメモリレイアウト
- クエリで必要なコンポーネントだけをイテレート

## 実装ガイドライン

### コンポーネント定義

```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct MyComponent {
    pub field1: Type1,
    pub field2: Type2,
}
```

### システム定義

```rust
pub fn my_system(
    mut commands: Commands,
    query: Query<(Entity, &ComponentA, &mut ComponentB), With<ComponentC>>,
    resource: Res<MyResource>,
) {
    for (entity, a, mut b) in query.iter_mut() {
        // 処理
    }
}
```

### 変更伝播パターン

```rust
// 1. 変更検知
pub fn detect_change(
    mut commands: Commands,
    query: Query<Entity, Changed<Source>>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(Marker);
    }
}

// 2. 処理
pub fn process(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Target), With<Marker>>,
) {
    for (entity, mut target) in query.iter_mut() {
        target.process();
        commands.entity(entity).remove::<Marker>();
    }
}

// 3. スケジュール
app.add_systems(Update, (
    detect_change,
    process,
).chain());
```

## 未更新のファイル

以下のファイルは参照頻度が低いため未更新：

- 06-event-system.md
- 08-layout-details.md
- 09-hit-test.md
- 11-visual-optimization.md
- 12-dependency-properties.md

これらも同様のパターンで更新可能です。

## バックアップ

移行前のドキュメントは以下に保存されています：
- `doc/spec_backup_20251101_082532/`

## 次のステップ

1. 実装コードをbevy_ecsベースに移行
2. 未更新のドキュメントの更新
3. パフォーマンステスト
4. 実例の追加

## 参考資料

- [bevy_ecs公式ドキュメント](https://docs.rs/bevy_ecs/)
- [bevy_hierarchy](https://docs.rs/bevy_hierarchy/)
- WinVisual.md - ダーティ検出戦略の詳細
