# 第8章: イベントシステム

この章では、マウス・キーボードイベント処理について説明します。

```rust
/// 描画コンポーネント（マーカートレイト）
pub trait RenderComponent: 'static {
    /// このコンポーネントが依存するシステム
    const DEPENDENCIES: &'static [SystemId];
}

/// テキスト描画コンポーネント
#[derive(Clone)]
pub struct TextRender {
    pub text: String,
    pub font_size: f32,
    pub color: Color,
}

impl RenderComponent for TextRender {
    const DEPENDENCIES: &'static [SystemId] = &[
        SystemId::Text,
        SystemId::Layout,
    ];
}

/// 画像描画コンポーネント
#[derive(Clone)]
pub struct ImageRender {
    pub image_id: ImageId,
    pub stretch: Stretch,
}

impl RenderComponent for ImageRender {
    const DEPENDENCIES: &'static [SystemId] = &[
        SystemId::Image,
        SystemId::Layout,
    ];
}

/// 背景描画コンポーネント
#[derive(Clone)]
pub struct BackgroundRender {
    pub fill: Brush,
    pub border: Option<Border>,
}

impl RenderComponent for BackgroundRender {
    const DEPENDENCIES: &'static [SystemId] = &[
        SystemId::ContainerStyle,
        SystemId::Layout,
    ];
}

/// カスタム描画コンポーネント
pub struct CustomRender {
    pub renderer: Box<dyn CustomRenderer>,
}

impl RenderComponent for CustomRender {
    const DEPENDENCIES: &'static [SystemId] = &[
        SystemId::Layout,  // 最小限の依存
        // カスタムレンダラーが追加の依存を持つ場合は動的に処理
    ];
}
```

#### コンポーネントの組み合わせで複雑な描画を表現

```rust
/// Widgetは複数の描画コンポーネントを持てる
pub struct Widget {
    id: WidgetId,
    // 描画コンポーネントのリスト（動的）
    render_components: Vec<RenderComponentType>,
}

/// 型安全なコンポーネント列挙
pub enum RenderComponentType {
    Text(TextRender),
    Image(ImageRender),
    Background(BackgroundRender),
    Custom(CustomRender),
}

impl RenderComponentType {
    /// このコンポーネントの依存を取得
    fn dependencies(&self) -> &'static [SystemId] {
        match self {
            Self::Text(_) => TextRender::DEPENDENCIES,
            Self::Image(_) => ImageRender::DEPENDENCIES,
            Self::Background(_) => BackgroundRender::DEPENDENCIES,
            Self::Custom(_) => CustomRender::DEPENDENCIES,
        }
