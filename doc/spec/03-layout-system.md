# レイアウトシステム

#### Layout関連プロパティ（個別のSecondaryMapで管理）
**最優先**：サイズが決まらないと描画できない

ECS/依存関係プロパティの原則に従い、各プロパティは独立したSecondaryMapで管理：

```rust
pub struct LayoutSystem {
    // サイズ制約（個別管理）
    width: SecondaryMap<WidgetId, Length>,
    height: SecondaryMap<WidgetId, Length>,
    min_width: SecondaryMap<WidgetId, f32>,
    max_width: SecondaryMap<WidgetId, f32>,
    min_height: SecondaryMap<WidgetId, f32>,
    max_height: SecondaryMap<WidgetId, f32>,
    
    // 間隔（個別管理）
    margin: SecondaryMap<WidgetId, Margin>,
    padding: SecondaryMap<WidgetId, Padding>,
    
    // 配置（個別管理）
    horizontal_alignment: SecondaryMap<WidgetId, Alignment>,
    vertical_alignment: SecondaryMap<WidgetId, Alignment>,
    
    // レイアウトタイプ（個別管理）
    layout_type: SecondaryMap<WidgetId, LayoutType>,
    
    // 計算結果（キャッシュ、個別管理）
    desired_size: SecondaryMap<WidgetId, Size2D>,
    final_rect: SecondaryMap<WidgetId, Rect>,
    
    // ダーティフラグ
    dirty: HashSet<WidgetId>,
}

// プロパティの型定義
pub enum Length {
    Auto,
    Pixels(f32),
    Percent(f32),
}

pub struct Margin {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

pub struct Padding {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

pub enum Alignment {
    Start,
    Center,
    End,
    Stretch,
}

pub enum LayoutType {
    None,
    Stack(StackLayout),
    // 将来的に追加
    // Grid(GridLayout),
    // Flex(FlexLayout),
}
```
