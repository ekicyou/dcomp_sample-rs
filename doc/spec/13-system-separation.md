# システム設計 (bevy_ecs版)

## 設計原則

bevy_ecsアーキテクチャの基本原則に従い、関心事を明確に分離：

1. **Entity（実体）**: `Entity` - bevy_ecsが自動管理する一意のID
2. **Component（コンポーネント）**: 各機能を独立したComponentとして定義
3. **System（システム）**: Componentに対する処理ロジックを関数として実装
4. **Query**: 必要なComponentを持つEntityだけを効率的に抽出

## 1. 親子関係管理（bevy_hierarchy）

すべてのEntityの親子関係を管理する基盤。bevy_hierarchyが標準で提供。

```rust
use bevy_ecs::prelude::*;

/// 親への参照
/// 注: ルートEntity（Window）はこのコンポーネントを持たない
#[derive(Component)]
pub struct Parent(pub Entity);

/// 子への参照
/// 注: 子を持つEntityのみがこのコンポーネントを持つ
#[derive(Component)]
pub struct Children(pub Vec<Entity>);
```

**主な操作**:
- `commands.entity(parent).add_child(child)`: 子Entityを追加（自動的にParent/Childrenを設定）
- `commands.entity(parent).push_children(&[child1, child2])`: 複数の子を追加
- `commands.entity(child).remove_parent()`: 親子関係を切断（Parentコンポーネントを削除）
- `commands.entity(entity).despawn_recursive()`: Entityと子を再帰的に削除

### ルート判定

```rust
/// Entityがルート（親を持たない）かどうか
pub fn is_root(entity: Entity, parent_query: Query<&Parent>) -> bool {
    parent_query.get(entity).is_err()
}

/// Windowを見つける
pub fn find_window(
    entity: Entity,
    parent_query: Query<&Parent>,
    window_query: Query<&Window>,
) -> Option<Entity> {
    let mut current = entity;
    loop {
        if window_query.get(current).is_ok() {
            return Some(current);
        }
        // Parentコンポーネントがあれば親をたどる
        if let Ok(parent) = parent_query.get(current) {
            current = parent.0;
        } else {
            return None; // ルートに到達
        }
    }
}
```
```rust
pub fn traverse_system(
    parent_query: Query<&Children>,
    entity_query: Query<&Name>,
) {
    fn visit(entity: Entity, parent_query: &Query<&Children>, entity_query: &Query<&Name>) {
        if let Ok(name) = entity_query.get(entity) {
            println!("Entity: {}", name.value);
        }
        if let Ok(children) = parent_query.get(entity) {
            for child in children.iter() {
                visit(*child, parent_query, entity_query);
            }
        }
    }
}
```

## 2. レイアウトシステム

Entityのサイズと位置を計算する。2パスレイアウト（Measure/Arrange）を実装。
各プロパティは独立したComponentとして定義。

```rust
use bevy_ecs::prelude::*;

/// サイズ指定
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

/// 余白
#[derive(Component)]
pub struct Margin {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

#[derive(Component)]
pub struct Padding {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

/// 配置
#[derive(Component)]
pub struct Alignment {
    pub horizontal: HorizontalAlignment,
    pub vertical: VerticalAlignment,
}

/// レイアウトタイプ
#[derive(Component)]
pub enum LayoutType {
    None,
    Stack(StackLayout),
    // Grid, Flexなど将来追加
}

/// 計算結果
#[derive(Component)]
pub struct ComputedLayout {
    pub desired_size: Size2D,
    pub final_rect: Rect,
}

/// ダーティマーカー
#[derive(Component)]
pub struct LayoutInvalidated;
```

**主なシステム**:
```rust
/// プロパティ変更検知
pub fn invalidate_layout_on_change(
    mut commands: Commands,
    query: Query<Entity, Or<(
        Changed<Size>,
        Changed<Margin>,
        Changed<Padding>,
        Changed<LayoutType>,
    )>>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(LayoutInvalidated);
    }
}

/// レイアウト計算
pub fn compute_layout_system(
    mut commands: Commands,
    mut query: Query<(Entity, &Size, &mut ComputedLayout), With<LayoutInvalidated>>,
) {
    for (entity, size, mut layout) in query.iter_mut() {
        // Measure/Arrange処理
        layout.compute(size);
        commands.entity(entity).remove::<LayoutInvalidated>();
    }
}
```
        self.measure_recursive(widget_system, root_id, available_size);
        
        // Arrangeパス（親から子へ、最終位置を決定）
        let final_rect = Rect::new(Point2D::zero(), available_size);
        self.arrange_recursive(widget_system, root_id, final_rect);
        
        self.dirty.clear();
    }
    
    /// 最終矩形を取得
    pub fn get_final_rect(&self, widget_id: WidgetId) -> Option<Rect> {
        self.final_rects.get(widget_id).cloned()
    }
    
    /// 希望サイズを取得
    pub fn get_desired_size(&self, widget_id: WidgetId) -> Option<Size2D> {
        self.desired_sizes.get(widget_id).cloned()
    }
    
    // 内部メソッド
    fn measure_recursive(&mut self, widget_system: &WidgetSystem, widget_id: WidgetId, available: Size2D) -> Size2D {
        // レイアウトタイプに応じた計測
        // 子を先に計測してから自分のサイズを決定
        let layout_type = self.get_layout_type(widget_id);
        
        let desired = match layout_type {
            LayoutType::Stack(stack) => {
                self.measure_stack(widget_system, widget_id, &stack, available)
            }
            LayoutType::None => Size2D::zero(),
        };
        
        // 計算結果を保存
        self.desired_sizes.insert(widget_id, desired);
        desired
    }
    
    fn arrange_recursive(&mut self, widget_system: &WidgetSystem, widget_id: WidgetId, final_rect: Rect) {
        // 自分の最終矩形を保存
        self.final_rects.insert(widget_id, final_rect);
        
        // 子を配置
        for child_id in widget_system.children(widget_id) {
            let child_rect = self.calculate_child_rect(widget_system, widget_id, child_id, final_rect);
            self.arrange_recursive(widget_system, child_id, child_rect);
        }
    }
}
```

### 3. DrawingContentSystem - 描画コマンド管理

ID2D1Imageベースの描画コマンドを生成・管理する。

```rust
pub struct DrawingContentSystem {
    /// 描画コンテンツ
    contents: SecondaryMap<WidgetId, DrawingContent>,
    
    /// Direct2Dデバイスコンテキスト
    d2d_context: ID2D1DeviceContext,
    
    /// ダーティフラグ
    dirty: HashSet<WidgetId>,
}
```

**主な操作**:
- `rebuild_content()`: ID2D1CommandListに描画コマンドを記録
- `get_content()`: 描画コンテンツ（ID2D1Image）を取得
- `invalidate()`: キャッシュを無効化
- `mark_dirty()`: ダーティマーク

### 4. TextSystem - テキスト描画

DirectWriteを使ってテキストレイアウトを管理。

```rust
pub struct TextSystem {
    /// テキストコンテンツ
    texts: SecondaryMap<WidgetId, TextContent>,
    
    /// DirectWriteファクトリ
    dwrite_factory: IDWriteFactory,
    
    /// ダーティフラグ
    dirty: HashSet<WidgetId>,
}
```

**主な操作**:
- `set_text()`: テキスト内容を設定（レイアウトを再計算）
- `get_text()`: テキスト内容を取得
- `set_font()`: フォント設定（ファミリ、サイズ）
- `draw_to_context()`: Direct2Dコンテキストに描画
- `measure_text()`: テキストの固有サイズを計算

### 5. ImageSystem - 画像管理

WICで画像を読み込み、ID2D1Bitmapとして管理。

```rust
pub struct ImageSystem {
    /// 画像コンテンツ
    images: SecondaryMap<WidgetId, ImageContent>,
    
    /// WICイメージングファクトリ
    wic_factory: IWICImagingFactory,
    
    /// ダーティフラグ
    dirty: HashSet<WidgetId>,
}
```

**主な操作**:
- `load_image()`: 画像ファイルを読み込み（WIC経由）
- `get_image()`: ID2D1Bitmapを取得
- `set_stretch()`: 伸縮モード設定
    pub fn set_stretch(&mut self, widget_id: WidgetId, stretch: Stretch) {
        if let Some(image) = self.images.get_mut(widget_id) {
            image.stretch = stretch;
            self.mark_dirty(widget_id);
        }
    }
    
    /// 描画コマンドを生成
    pub fn draw_to_context(
        &self,
        widget_id: WidgetId,
        dc: &ID2D1DeviceContext,
        rect: Rect,
    ) -> Result<()> {
        if let Some(image) = self.images.get(widget_id) {
            let dest_rect = self.calculate_dest_rect(image, rect);
            
            unsafe {
                dc.DrawBitmap(
                    &image.bitmap,
                    Some(&dest_rect.into()),
                    image.opacity,
                    D2D1_INTERPOLATION_MODE_LINEAR,
                    image.source_rect.map(|r| r.into()).as_ref(),
                )?;
            }
        }
        Ok(())
    }
    
    /// 固有サイズを取得
    pub fn get_intrinsic_size(&self, widget_id: WidgetId) -> Option<Size2D> {
        self.images.get(widget_id).and_then(|img| {
            let size = unsafe { img.bitmap.GetSize() };
            Some(Size2D::new(size.width, size.height))
        })
    }
    
    fn mark_dirty(&mut self, widget_id: WidgetId) {
        self.dirty.insert(widget_id);
    }
}
```

### 6. VisualSystem - DirectCompositionツリー管理

DirectCompositionのビジュアルツリーを管理。

```rust
pub struct VisualSystem {
    /// Visual情報
    visuals: SecondaryMap<WidgetId, Visual>,
    
    /// DirectCompositionデバイス
    dcomp_device: IDCompositionDevice,
    
    /// DirectCompositionターゲット
    dcomp_target: IDCompositionTarget,
    
    /// ダーティフラグ
    dirty: HashSet<WidgetId>,
}
```

**主な操作**:
- `ensure_visual()`: IDCompositionVisualを作成または取得
- `remove_visual()`: Visualを削除
- `apply_content()`: DrawingContent（ID2D1Image）をVisualに適用（サーフェス作成→描画）
- `set_offset()`: オフセット（位置）を設定
- `set_opacity()`: 不透明度を設定
- `commit()`: 変更を画面に反映

### 7. InteractionSystem - イベント処理

マウス、キーボード、フォーカスなどのインタラクションを管理。

```rust
pub struct InteractionSystem {
    /// インタラクション状態
    interactions: SecondaryMap<WidgetId, InteractionState>,
    
    /// フォーカス中のWidget
    focused_widget: Option<WidgetId>,
    
    /// ホバー中のWidget
    hovered_widget: Option<WidgetId>,
}
```

**主な操作**:
- `add_handler()`: イベントハンドラを登録
- `dispatch_event()`: イベントをディスパッチ（バブリング）
- `hit_test()`: 座標からWidgetを検索（深さ優先探索）
- `set_focus()`: フォーカスを設定

### 8. ContainerStyleSystem - コンテナスタイル管理

背景色、枠線などのスタイル情報を管理。

```rust
pub struct ContainerStyleSystem {
    /// コンテナスタイル
    styles: SecondaryMap<WidgetId, ContainerStyle>,
    
    /// ダーティフラグ
    dirty: HashSet<WidgetId>,
}
```

**主な操作**:
- `set_background()`: 背景色を設定
- `set_border()`: 枠線を設定
- `set_padding()`: パディングを設定
- `draw_to_context()`: 描画コマンドを生成（背景・枠線）

### 統合レイヤー: UiRuntime

各システムを統合して、協調動作させる中心的なランタイム。

```rust
pub struct UiRuntime {
    // コア
    widget_system: WidgetSystem,
    
    // 各システム
    layout: LayoutSystem,
    drawing_content: DrawingContentSystem,
    text: TextSystem,
    image: ImageSystem,
    container_style: ContainerStyleSystem,
    visual: VisualSystem,
    interaction: InteractionSystem,
}
```

**主な操作**:
- `update_frame()`: フレーム更新（レイアウト→描画コンテンツ→Visual→コミット）
- `update_drawing_contents()`: テキスト、画像、スタイルから描画コマンドを生成
- `update_visuals()`: DrawingContentをDirectComposition Visualに反映
- `create_text_widget()`: テキストWidget作成
- `create_image_widget()`: 画像Widget作成
- `create_container()`: コンテナWidget作成
- `create_stack_panel()`: スタックパネル作成
- `handle_mouse_down()`: マウスイベント処理（ヒットテスト→ディスパッチ）

### システム間の依存関係図

```text
┌──────────────┐
│WidgetSystem  │ ◄─── すべてのシステムが参照
└──────────────┘
      │
      ▼
┌─────────────┐
│LayoutSystem │ ◄─── 多くのシステムが参照（サイズ情報）
└─────────────┘
      │
      ▼
┌──────────────────────────────────────┐
│ TextSystem / ImageSystem /           │
│ ContainerStyleSystem                 │ ─┐
└──────────────────────────────────────┘  │
      │                                    │
      ▼                                    │
┌─────────────────────┐                   │
│ DrawingContentSystem│ ◄─────────────────┘
└─────────────────────┘
      │
      ▼
┌─────────────┐
│ VisualSystem│ ─── DirectComposition
└─────────────┘

┌──────────────────┐
│ InteractionSystem│ ─── イベント処理（並行）
└──────────────────┘

注: rootはWindowSystemが所有するWindowが管理
```

### WindowとWidgetの関係

Windowは特殊なWidget（ルートWidget）として扱われる：

```rust
pub struct Window {
    hwnd: HWND,
    root_widget_id: WidgetId,  // このWindowのルートWidget
    dcomp_target: IDCompositionTarget,
}

pub struct WindowSystem {
    windows: HashMap<HWND, Window>,
}
```

**主な操作**:
- `create_window()`: OSウィンドウとルートWidgetを作成
- `get_root_widget()`: WindowのルートWidgetを取得
- `close_window()`: Window閉鎖（ルートWidget削除→子も再帰削除）

### UiRuntimeとWindowSystemの協調

```rust
// UiRuntimeは汎用的なUI管理
let mut ui_runtime = UiRuntime::new();
let mut window_system = WindowSystem::new();

// Window1を作成
let hwnd1 = window_system.create_window(&mut ui_runtime)?;
let root1 = window_system.get_root_widget(hwnd1).unwrap();
let text = ui_runtime.create_text_widget("Hello".to_string());
ui_runtime.widget_system.append_child(root1, text)?;

// Window2を作成（別のツリー）
let hwnd2 = window_system.create_window(&mut ui_runtime)?;
let root2 = window_system.get_root_widget(hwnd2).unwrap();

// Widgetを別Windowへ移動
ui_runtime.widget_system.detach_widget(text)?;
ui_runtime.widget_system.append_child(root2, text)?;
```

**マルチウィンドウ対応の特徴**:
- 複数のWindowが独立したWidgetツリーを持てる
- WindowもTextBlockも同じWidgetSystemで管理
- detach/appendでWidget（UIコンポーネント）を自由に移動可能
- 切り離したWidgetは削除せずに再利用できる

### detach_widgetとdelete_widgetの使い分け

- **detach_widget**: ツリーから切り離すが存在は維持（再利用可能）
- **delete_widget**: 完全に削除（子も再帰削除）

### 分離のメリット

1. **単一責任**: 各システムが1つの明確な責務
