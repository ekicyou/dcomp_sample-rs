# 第9章: ヒットテストシステム

この章では、座標からEntityを検索する方法について説明します。

    }
}
```

## 基本的なUI要素

### 1. Container（コンテナー）

シンプルなUI要素。子を配置するための器。
**背景色や枠線がない場合、Visualは作成されない（効率化）**

### 2. TextBlock（テキストブロック）

テキストを表示。縦書き対応が重要（FlowDirection）。**Visualを動的に作成**

```rust
pub struct TextContent {
    text: String,
    font_family: String,
    font_size: f32,
    flow_direction: FlowDirection, // TopToBottom or LeftToRight
    text_format: IDWriteTextFormat,
    text_layout: IDWriteTextLayout,
}
```

### 3. Image（画像）

画像を表示。透過対応。**Visualを動的に作成**

```rust
pub struct ImageContent {
    bitmap: ID2D1Bitmap,
    source_rect: Option<Rect>,
    stretch: Stretch, // None, Fill, Uniform, UniformToFill
    opacity: f32,
}
```

### 4. Button（ボタン）

クリック可能なUI要素。インタラクション状態（hover, pressed）を管理。

### 5. StackPanel（スタックパネル）

子要素を縦または横に配置するコンテナー。

```rust
pub struct StackLayout {
    orientation: Orientation, // Vertical or Horizontal
    spacing: f32,
}
```

## レイアウトシステム

### Measure/Arrange パス

WPFやFlutterと同様の2パスレイアウト。

```rust
pub struct Layout {
    // 制約
    width: Length,
    height: Length,
    min_width: f32,
    max_width: f32,
    min_height: f32,
    max_height: f32,
    
    // 間隔
    margin: Margin,
    padding: Padding,
    
    // 配置
