# DirectComposition ビジュアルツリー管理

## 描画パイプライン

### 描画が必要なWidgetの判定と処理

```rust
impl WidgetSystem {
    /// フレーム更新時の描画処理
    pub fn update_visuals(&mut self) -> Result<()> {
        for widget_id in self.dirty_visual.drain().collect::<Vec<_>>() {
            // 描画が必要か判定
            if self.needs_visual(widget_id) {
                // Visualを確保（なければ作成）
                self.ensure_visual(widget_id)?;
                
                // コンテンツを描画
                self.draw_visual(widget_id)?;
            } else {
                // 不要になったVisualを削除
                self.remove_visual(widget_id)?;
            }
        }
        
        // DirectCompositionにコミット
        self.dcomp_context.commit()?;
        
        Ok(())
    }
}
```

このドキュメントでは、DirectCompositionを使用したビジュアルツリーの管理方法を説明します。
**Visualは描画が必要なWidgetのみが持ち、動的に作成されます。**

## 重要な設計原則

1. **論理ツリーとビジュアルツリーの分離**
   - 論理ツリー: すべてのWidget（UI構造）
   - ビジュアルツリー: 描画が必要なWidgetのみ（DirectComposition）

2. **動的Visual作成**
   - テキスト、画像、背景色などを持つWidgetのみがVisualを持つ
   - 純粋なレイアウトノードはVisualを持たない（メモリ効率化）

3. **自動的なツリー管理**
   - Visualは親でVisualを持つWidgetに自動接続される
   - 中間のVisualなしWidgetは透過的にスキップされる

## DirectComposition アーキテクチャ

### 基本構造

```
Window
  └─ IDCompositionDesktopDevice (デバイス)
      └─ IDCompositionTarget (ウィンドウとの関連付け)
          └─ IDCompositionVisual (ルートビジュアル)
              ├─ IDCompositionVisual (子1)
              │   └─ IDCompositionSurface (描画サーフェス)
              ├─ IDCompositionVisual (子2)
              └─ IDCompositionVisual (子3)
```

### Visual の初期化

```rust
use windows::Win32::Graphics::DirectComposition::*;
use windows::Win32::Graphics::Direct2D::*;

pub struct DCompContext {
    device: IDCompositionDesktopDevice,
    target: IDCompositionTarget,
    d2d_device: ID2D1Device,
    root_visual: IDCompositionVisual,
}

impl DCompContext {
    pub fn new(hwnd: HWND) -> Result<Self> {
        unsafe {
            // DirectCompositionデバイスの作成
            let device: IDCompositionDesktopDevice = 
                DCompositionCreateDevice(None)?;
            
            // ターゲットの作成（ウィンドウに関連付け）
            let target = device.CreateTargetForHwnd(hwnd, true)?;
            
            // ルートビジュアルの作成
            let root_visual = device.CreateVisual()?;
            target.SetRoot(&root_visual)?;
            
            // Direct2Dデバイスの作成（描画用）
            let d2d_factory: ID2D1Factory1 = 
                D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
            let d2d_device = d2d_factory.CreateDevice(/* ... */)?;
            
            Ok(Self {
                device,
                target,
                d2d_device,
                root_visual,
            })
        }
    }
    
    pub fn commit(&self) -> Result<()> {
        unsafe { self.device.Commit() }
    }
}
```

## Visual 管理システム

### Visual の定義（詳細版）

**Visualは`SecondaryMap`で管理され、必要なWidgetのみが持つ**

```rust
pub struct Visual {
    // 識別子
    widget_id: WidgetId,  // 対応するWidget
    
    // DirectComposition オブジェクト
    dcomp_visual: IDCompositionVisual,
    dcomp_surface: Option<IDCompositionSurface>,
    d2d_device_context: Option<ID2D1DeviceContext>,
    
    // トランスフォーム
    offset: Point2D,
    size: Size2D,
    scale: Vector2D,
    rotation: f32, // ラジアン
    transform_matrix: Option<Matrix3x2>,
    
    // 表示設定
    opacity: f32,
    visible: bool,
    clip_rect: Option<Rect>,
    
    // ヒットテスト設定
    is_hit_testable: bool,
    hit_test_geometry: HitTestGeometry,
    
    // ダーティフラグ
    needs_redraw: bool,
    needs_layout_update: bool,
}

pub enum HitTestGeometry {
    Rectangle(Rect),
    Ellipse { center: Point2D, radius: Vector2D },
    Path(ID2D1Geometry),
    PixelPerfect, // サーフェスのアルファ値を使用
}
```

### Visual作成の判断

```rust
impl WidgetSystem {
    /// このWidgetが描画を必要とするか判定
    fn needs_visual(&self, widget_id: WidgetId) -> bool {
        // テキストコンテンツを持つ
        self.texts.contains_key(widget_id) 
            // または画像を持つ
            || self.images.contains_key(widget_id)
            // または背景色/枠線を持つ
            || self.has_background(widget_id)
            // またはカスタム描画を行う
            || self.has_custom_draw(widget_id)
    }
    
    fn has_background(&self, widget_id: WidgetId) -> bool {
        self.container_styles
            .get(widget_id)
            .map(|s| s.background.is_some() || s.border.is_some())
            .unwrap_or(false)
    }
}
```
```

### Visual の生成と管理

**Visualは動的に作成され、親のVisualツリーに自動接続される**

```rust
impl WidgetSystem {
    /// Visualを動的に作成または取得
    fn ensure_visual(&mut self, widget_id: WidgetId) -> Result<()> {
        if self.visuals.contains_key(widget_id) {
            return Ok(()); // 既に存在
        }
        
        unsafe {
            // DirectCompositionビジュアルを作成
            let dcomp_visual = self.dcomp_context.device.CreateVisual()?;
            
            let visual = Visual {
                widget_id,
                dcomp_visual,
                dcomp_surface: None,
                d2d_device_context: None,
                offset: Point2D::zero(),
                size: Size2D::zero(),
                scale: Vector2D::new(1.0, 1.0),
                rotation: 0.0,
                transform_matrix: None,
                opacity: 1.0,
                visible: true,
                clip_rect: None,
                is_hit_testable: true,
                hit_test_geometry: HitTestGeometry::Rectangle(Rect::zero()),
                needs_redraw: true,
                needs_layout_update: true,
            };
            
            self.visuals.insert(widget_id, visual);
            
            // 親のVisualツリーに接続
            self.attach_visual_to_tree(widget_id)?;
        }
        
        Ok(())
    }
    
    /// VisualをDirectCompositionツリーに接続
    fn attach_visual_to_tree(&mut self, widget_id: WidgetId) -> Result<()> {
        // 親でVisualを持つ最も近いWidgetを探す
        let parent_visual_id = self.find_parent_with_visual(widget_id);
        
        if let Some(parent_id) = parent_visual_id {
            let child_visual = self.visuals.get(widget_id).unwrap();
            let parent_visual = self.visuals.get(parent_id).unwrap();
            
            unsafe {
                parent_visual.dcomp_visual
                    .AddVisual(&child_visual.dcomp_visual, true, None)?;
            }
        } else {
            // 親がない場合、ルートのVisualに接続
            let child_visual = self.visuals.get(widget_id).unwrap();
            unsafe {
                self.dcomp_context.root_visual
                    .AddVisual(&child_visual.dcomp_visual, true, None)?;
            }
        }
        
        Ok(())
    }
    
    /// 親でVisualを持つWidgetを探す（再帰的に上へ）
    fn find_parent_with_visual(&self, widget_id: WidgetId) -> Option<WidgetId> {
        let mut current = self.widgets.get(widget_id)?.parent;
        
        while let Some(parent_id) = current {
            if self.visuals.contains_key(parent_id) {
                return Some(parent_id);
            }
            current = self.widgets.get(parent_id)?.parent;
        }
        
        None
    }
    
    /// Visualを削除（不要になった場合）
    fn remove_visual(&mut self, widget_id: WidgetId) -> Result<()> {
        if let Some(visual) = self.visuals.remove(widget_id) {
            // DirectCompositionツリーから削除
            if let Some(parent_id) = self.find_parent_with_visual(widget_id) {
                let parent_visual = self.visuals.get(parent_id).unwrap();
                unsafe {
                    parent_visual.dcomp_visual
                        .RemoveVisual(&visual.dcomp_visual)?;
                }
            } else {
                // ルートから削除
                unsafe {
                    self.dcomp_context.root_visual
                        .RemoveVisual(&visual.dcomp_visual)?;
                }
            }
        }
        
        Ok(())
    }
}
```

## 論理ツリーとビジュアルツリーの関係

### ツリー構造の例

論理ツリー（Widget）とビジュアルツリー（DirectComposition）は1:1対応しない：

```
論理ツリー (Widget):                    ビジュアルツリー (DirectComposition):

Window                                   Window Root Visual
  └─ Root                                  ├─ Panel Visual (背景あり)
      ├─ Panel (背景あり)                  │   ├─ TextBlock1 Visual
      │   ├─ TextBlock1                   │   └─ Image1 Visual
      │   └─ LayoutContainer (透明)       └─ TextBlock2 Visual
      │       ├─ Image1
      │       └─ Spacer (透明)
      └─ TextBlock2

Visualなし: LayoutContainer, Spacer
Visualあり: Panel, TextBlock1, Image1, TextBlock2
```

### 子の追加時の処理

```rust
impl WidgetSystem {
    pub fn append_child(&mut self, parent_id: WidgetId, child_id: WidgetId) -> Result<()> {
        // 論理ツリーに追加
        self.append_child_to_widget_tree(parent_id, child_id);
        
        // 子がVisualを持つ場合、ビジュアルツリーも更新
        if let Some(child_visual) = self.visuals.get(child_id) {
            // 親でVisualを持つWidgetを探す
            if let Some(parent_visual_id) = self.find_visual_parent(parent_id) {
                let parent_visual = self.visuals.get(parent_visual_id).unwrap();
                
                unsafe {
                    parent_visual.dcomp_visual
                        .AddVisual(&child_visual.dcomp_visual, true, None)?;
                }
            }
        }
        
        Ok(())
    }
    
    fn find_visual_parent(&self, widget_id: WidgetId) -> Option<WidgetId> {
        // widget_id自身がVisualを持つならそれを返す
        if self.visuals.contains_key(widget_id) {
            return Some(widget_id);
        }
        
        // 持たないなら、親に向かって探す
        self.find_parent_with_visual(widget_id)
    }
}
```

### サーフェスの作成と描画

```rust
impl WidgetSystem {
    /// Visualにコンテンツを描画
    pub fn draw_visual(&mut self, widget_id: WidgetId) -> Result<()> {
        let visual = self.visuals.get_mut(widget_id).unwrap();
        
        if !visual.needs_redraw {
            return Ok(());
        }
        
        unsafe {
            // サーフェスを作成（まだなければ）
            if visual.dcomp_surface.is_none() {
                let surface = self.dcomp_context.device.CreateSurface(
                    visual.size.width as u32,
                    visual.size.height as u32,
                    DXGI_FORMAT_B8G8R8A8_UNORM,
                    DXGI_ALPHA_MODE_PREMULTIPLIED,
                )?;
                
                visual.dcomp_visual.SetContent(&surface)?;
                visual.dcomp_surface = Some(surface);
            }
            
            // サーフェスに描画開始
            let surface = visual.dcomp_surface.as_ref().unwrap();
            let mut offset = POINT::default();
            let dc = surface.BeginDraw(None, &mut offset)?;
            
            // Direct2Dデバイスコンテキストを作成
            if visual.d2d_device_context.is_none() {
                visual.d2d_device_context = Some(
                    self.dcomp_context.d2d_device.CreateDeviceContext(
                        D2D1_DEVICE_CONTEXT_OPTIONS_NONE
                    )?
                );
            }
            
            let d2d_dc = visual.d2d_device_context.as_ref().unwrap();
            d2d_dc.SetTarget(/* サーフェスのビットマップ */);
            
            d2d_dc.BeginDraw();
            d2d_dc.Clear(Some(&D2D1_COLOR_F {
                r: 0.0, g: 0.0, b: 0.0, a: 0.0, // 透明
            }));
            
            // コンテンツの種類に応じて描画
            self.draw_visual_content(widget_id, d2d_dc)?;
            
            d2d_dc.EndDraw(None, None)?;
            surface.EndDraw()?;
            
            visual.needs_redraw = false;
        }
        
        Ok(())
    }
    
    fn draw_visual_content(
        &self,
        widget_id: WidgetId,
        dc: &ID2D1DeviceContext,
    ) -> Result<()> {
        // テキストコンテンツ
        if let Some(text) = self.texts.get(widget_id) {
            self.draw_text_content(dc, text)?;
        }
        
        // 画像コンテンツ
        if let Some(image) = self.images.get(widget_id) {
            self.draw_image_content(dc, image)?;
        }
        
        // カスタム描画
        // ...
        
        Ok(())
    }
}
```

### テキスト描画（縦書き対応）

```rust
impl WidgetSystem {
    fn draw_text_content(
        &self,
        dc: &ID2D1DeviceContext,
        text: &TextContent,
    ) -> Result<()> {
        unsafe {
            // DirectWriteテキストレイアウトを作成
            let text_layout = self.dwrite_factory.CreateTextLayout(
                &text.text,
                &text.text_format,
                text.max_width,
                text.max_height,
            )?;
            
            // 縦書き設定
            text_layout.SetReadingDirection(match text.reading_direction {
                ReadingDirection::TopToBottom => DWRITE_READING_DIRECTION_TOP_TO_BOTTOM,
                ReadingDirection::LeftToRight => DWRITE_READING_DIRECTION_LEFT_TO_RIGHT,
            })?;
            
            text_layout.SetFlowDirection(match text.flow_direction {
                FlowDirection::TopToBottom => DWRITE_FLOW_DIRECTION_TOP_TO_BOTTOM,
                FlowDirection::LeftToRight => DWRITE_FLOW_DIRECTION_LEFT_TO_RIGHT,
            })?;
            
            // ブラシを作成
            let brush = dc.CreateSolidColorBrush(
                &D2D1_COLOR_F {
                    r: 0.0, g: 0.0, b: 0.0, a: 1.0, // 黒
                },
                None,
            )?;
            
            // 描画
            dc.DrawTextLayout(
                D2D_POINT_2F { x: 0.0, y: 0.0 },
                &text_layout,
                &brush,
                D2D1_DRAW_TEXT_OPTIONS_NONE,
            );
        }
        
        Ok(())
    }
}
```

### 画像描画

```rust
impl WidgetSystem {
    fn draw_image_content(
        &self,
        dc: &ID2D1DeviceContext,
        image: &ImageContent,
    ) -> Result<()> {
        unsafe {
            // ビットマップを描画
            let dest_rect = D2D_RECT_F {
                left: 0.0,
                top: 0.0,
                right: image.width,
                bottom: image.height,
            };
            
            let source_rect = image.source_rect.map(|r| D2D_RECT_F {
                left: r.x,
                top: r.y,
                right: r.x + r.width,
                bottom: r.y + r.height,
            });
            
            dc.DrawBitmap(
                &image.bitmap,
                Some(&dest_rect),
                image.opacity,
                D2D1_INTERPOLATION_MODE_LINEAR,
                source_rect.as_ref(),
            );
        }
        
        Ok(())
    }
}
```

## トランスフォーム管理

### トランスフォームの適用

```rust
impl Visual {
    /// トランスフォームを更新
    pub fn update_transform(&mut self, device: &IDCompositionDevice) -> Result<()> {
        unsafe {
            // オフセット
            self.dcomp_visual.SetOffsetX(self.offset.x)?;
            self.dcomp_visual.SetOffsetY(self.offset.y)?;
            
            // 複合トランスフォーム（スケール + 回転）
            if self.scale != Vector2D::new(1.0, 1.0) || self.rotation != 0.0 {
                let transform = device.CreateMatrixTransform()?;
                
                let matrix = self.calculate_transform_matrix();
                transform.SetMatrix(&matrix)?;
                
                self.dcomp_visual.SetTransform(&transform)?;
            }
            
            // 不透明度
            self.dcomp_visual.SetOpacity(self.opacity)?;
            
            // クリッピング
            if let Some(clip) = self.clip_rect {
                let clip_visual = device.CreateRectangleClip()?;
                clip_visual.SetLeft(clip.x)?;
                clip_visual.SetTop(clip.y)?;
                clip_visual.SetRight(clip.x + clip.width)?;
                clip_visual.SetBottom(clip.y + clip.height)?;
                
                self.dcomp_visual.SetClip(&clip_visual)?;
            }
        }
        
        Ok(())
    }
    
    fn calculate_transform_matrix(&self) -> D2D_MATRIX_3X2_F {
        // スケール行列
        let scale_matrix = D2D_MATRIX_3X2_F {
            _11: self.scale.x, _12: 0.0,
            _21: 0.0, _22: self.scale.y,
            _31: 0.0, _32: 0.0,
        };
        
        // 回転行列
        let cos = self.rotation.cos();
        let sin = self.rotation.sin();
        let rotation_matrix = D2D_MATRIX_3X2_F {
            _11: cos, _12: sin,
            _21: -sin, _22: cos,
            _31: 0.0, _32: 0.0,
        };
        
        // 合成
        matrix_multiply(&scale_matrix, &rotation_matrix)
    }
}
```

## 更新サイクル

### フレーム更新の流れ

```rust
impl WidgetSystem {
    /// フレーム更新（メインループから呼ばれる）
    pub fn update_frame(&mut self) {
        // 1. ダーティなレイアウトを更新
        if !self.dirty_layout.is_empty() {
            self.update_layouts();
        }
        
        // 2. レイアウト情報をVisualに反映
        self.sync_layout_to_visuals();
        
        // 3. ダーティなVisualを再描画
        for widget_id in self.dirty_visual.drain().collect::<Vec<_>>() {
            self.draw_visual(widget_id).ok();
        }
        
        // 4. DirectCompositionにコミット
        self.dcomp_context.commit().unwrap();
    }
    
    fn sync_layout_to_visuals(&mut self) {
        for (widget_id, layout) in &self.layouts {
            if let Some(visual) = self.visuals.get_mut(widget_id) {
                if visual.needs_layout_update {
                    visual.offset = layout.final_rect.origin;
                    visual.size = layout.final_rect.size;
                    
                    visual.update_transform(&self.dcomp_context.device).ok();
                    visual.needs_layout_update = false;
                    visual.needs_redraw = true; // サイズ変更時は再描画
                }
            }
        }
    }
}
```

## アニメーション

DirectCompositionは高性能なアニメーションをサポートします。

```rust
pub struct Animation {
    target_visual: VisualId,
    property: AnimationProperty,
    duration: Duration,
    easing: EasingFunction,
}

pub enum AnimationProperty {
    Opacity(f32, f32), // from, to
    Offset(Point2D, Point2D),
    Scale(Vector2D, Vector2D),
    Rotation(f32, f32),
}

impl WidgetSystem {
    pub fn animate(
        &mut self,
        widget_id: WidgetId,
        property: AnimationProperty,
        duration: Duration,
    ) -> Result<()> {
        let visual = self.visuals.get(widget_id).unwrap();
        
        unsafe {
            let animation = self.dcomp_context.device.CreateAnimation()?;
            
            // アニメーションパラメータを設定
            match property {
                AnimationProperty::Opacity(from, to) => {
                    animation.AddCubic(
                        0.0,     // beginOffset
                        from,    // constantCoefficient
                        0.0,     // linearCoefficient
                        0.0,     // quadraticCoefficient
                        (to - from) / duration.as_secs_f32(), // cubicCoefficient
                    )?;
                    
                    visual.dcomp_visual.SetOpacity_2(&animation)?;
                }
                // 他のプロパティも同様に...
                _ => {}
            }
            
            self.dcomp_context.commit()?;
        }
        
        Ok(())
    }
}
```

## trait WtfVisual

すべての描画可能な要素が実装すべきトレイト。

```rust
pub trait WtfVisual {
    /// Visualにコンテンツを描画
    fn draw(&self, dc: &ID2D1DeviceContext, visual: &Visual) -> Result<()>;
    
    /// 望ましいサイズを計算
    fn measure(&self, available_size: Size2D) -> Size2D;
    
    /// ヒットテスト
    fn hit_test(&self, visual: &Visual, point: Point2D) -> bool {
        visual.hit_test_geometry.contains(point)
    }
    
    /// プロパティ変更通知
    fn on_property_changed(&mut self, property: &str) {
        // デフォルト実装：何もしない
    }
}

// TextContentの実装例
impl WtfVisual for TextContent {
    fn draw(&self, dc: &ID2D1DeviceContext, visual: &Visual) -> Result<()> {
        // DirectWriteでテキストを描画
        unsafe {
            let brush = dc.CreateSolidColorBrush(
                &D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
                None,
            )?;
            
            dc.DrawTextLayout(
                D2D_POINT_2F { x: 0.0, y: 0.0 },
                &self.text_layout,
                &brush,
                D2D1_DRAW_TEXT_OPTIONS_NONE,
            );
        }
        Ok(())
    }
    
    fn measure(&self, available_size: Size2D) -> Size2D {
        // テキストレイアウトからサイズを取得
        unsafe {
            let metrics = self.text_layout.GetMetrics().unwrap();
            Size2D::new(metrics.width, metrics.height)
        }
    }
}
```

## まとめ

DirectCompositionを使ったビジュアル管理の特徴：

1. **ハードウェアアクセラレーション**: GPUで合成、高性能
2. **独立したコンポジション**: UIスレッドをブロックしない
3. **スムーズなアニメーション**: 60FPS以上も容易
4. **透過ウィンドウ対応**: アルファブレンディングが標準
5. **動的Visual作成**: 必要なWidgetのみがVisualを持つ（メモリ効率）
6. **自動ツリー管理**: Visualなし中間ノードを透過的にスキップ
7. **効率的な更新**: ダーティフラグで必要な部分のみ再描画

### 設計の利点

**メモリ効率**
- 純粋なレイアウトノードはVisualを持たない
- 数千のWidget中、実際に描画するのは一部のみ

**柔軟性**
- Widgetの追加/削除時、Visualツリーを自動調整
- 背景色の追加/削除でVisualの動的作成/削除

**パフォーマンス**
- DirectCompositionはVisual数が少ないほど高速
- 不要なVisualを作らないことで合成が軽量化


