# Widgetツリー構造

## 基本的な考え方

- UIではツリー構造を管理することが多い
- ツリー構造をrustで管理しようとするとArcなどを使う必要がありコードが煩雑になる
- ECSのように、EntityIDでオブジェクトにアクセスし、ツリー構造もID管理とすることで参照関係の管理を整理する。
- また、メモリ管理が配列ベースになり、キャッシュに乗りやすくなることも期待される。
- rustではIDベースのデータ構造を管理するのに`slotmap`クレートが適切である。
- slotmapに全データを載せていく管理をシステムの基本とする

## UIツリーを表現する「Widget」

### Widget の役割
- WidgetはUIツリーのノードを連結リストで表現する。
- Widgetは`WidgetId`をもち、slotmapによって管理する。
- 親子関係は`WidgetId`で管理
- **Windowも概念的にはWidgetであり、Widgetツリーのルート要素となる**

### Windowの特殊性

Windowは他のUI要素（TextBlock、Image、Containerなど）と同様にWidgetとして扱われるが、以下の点で特別：

1. **ルートWidget**: Windowは常にWidgetツリーのルート（親を持たない）
2. **OSウィンドウとの関連**: HWNDと1:1で対応
3. **WindowSystemが管理**: `WindowSystem`が各WindowのWidgetIdを保持
4. **DirectComposition接続点**: ウィンドウのDCompTargetがビジュアルツリーの起点

```rust
// 概念的な構造
Window (WidgetId)                    // WindowSystem が管理
  └─ Container (WidgetId)            // レイアウトコンテナ
       ├─ TextBlock (WidgetId)       // テキスト要素
       └─ Image (WidgetId)           // 画像要素
```

### Widget ID の定義
```rust
use slotmap::new_key_type;

// WidgetIdは世代付きインデックス (Generation + Index)
new_key_type! {
    pub struct WidgetId;
}
```

### Widget の定義

```rust
struct Widget {
    id: WidgetId,
    parent: Option<WidgetId>,
    first_child: Option<WidgetId>,
    last_child: Option<WidgetId>,
    next_sibling: Option<WidgetId>,
}
```

### Widget の操作

連結リスト構造を維持しながら、子の追加・切り離し・削除・走査を行う。

**主な操作**:
- `append_child()`: 子Widgetを末尾に追加
- `detach_widget()`: Widgetをツリーから切り離す（Widget自体は残り、再利用可能）
- `delete_widget()`: Widgetを完全に削除（子も再帰的に削除）
- `children()`: 子Widgetを列挙するイテレータ

## WidgetSystem

ツリー構造管理（最も基本的なシステム）

```rust
pub struct WidgetSystem {
    widget: SlotMap<WidgetId, Widget>,
}
```

**主な操作**:
- `create_widget()`: 新しいWidgetを作成
- `append_child()`: 子Widgetを親に追加（連結リスト操作）
- `detach_widget()`: ツリーから切り離す（再利用可能）
- `delete_widget()`: 完全に削除（子も再帰的に削除）
- `children()`: 子Widgetのイテレータ
- `parent()`: 親Widgetを取得
- `contains()`: Widgetの存在確認
