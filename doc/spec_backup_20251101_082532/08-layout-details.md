# レイアウトシステム詳細


1. **統一的なインターフェース**
   ```rust
   // すべてID2D1Imageとして扱える
   fn draw_content(dc: &ID2D1DeviceContext, content: &ID2D1Image) {
       dc.DrawImage(content, None, None, D2D1_INTERPOLATION_MODE_LINEAR, None);
   }
   ```

2. **効率的なキャッシュ**
   ```rust
   // ID2D1CommandListに描画を記録してキャッシュ
   let command_list = dc.CreateCommandList()?;
   dc.SetTarget(&command_list);
   // 複雑な描画処理
   draw_complex_shape(dc);
   dc.EndDraw()?;
   command_list.Close()?;
   
   // 次回からはコマンドリストを再生（高速）
   dc.DrawImage(&command_list, ...);
   ```

3. **エフェクトの適用が容易**
   ```rust
   // ブラー、影、色調整などをID2D1Effectで
   let blur_effect = dc.CreateEffect(&CLSID_D2D1GaussianBlur)?;
   blur_effect.SetInput(0, &drawing_content.content, ...)?;
   
   // エフェクト適用済みもID2D1Image
   dc.DrawImage(&blur_effect, ...);
   ```

4. **DirectCompositionとの親和性**
   ```rust
   // DirectCompositionサーフェスの描画結果もID2D1Imageとして取得可能
   // → 複雑なUI要素を事前レンダリングしてキャッシュ
   ```
```

### プロパティ変更の流れ

```rust
impl WidgetSystem {
    /// レイアウト情報を更新
    pub fn set_layout(&mut self, widget_id: WidgetId, layout: Layout) {
        self.layouts.insert(widget_id, layout);
        self.dirty_layout.insert(widget_id);
        // 子孫もダーティにする（レイアウト伝播）
        self.mark_descendants_dirty(widget_id);
    }
    
    /// テキスト内容を更新
    pub fn set_text(&mut self, widget_id: WidgetId, text: String) {
        if let Some(content) = self.texts.get_mut(widget_id) {
            content.text = text;
            self.dirty_visual.insert(widget_id);
        }
    }
}
```

## イベントシステム

### イベントの種類

```rust
pub enum UiEvent {
    // マウスイベント
    MouseEnter,
    MouseLeave,
    MouseMove { x: f32, y: f32 },
    MouseDown { button: MouseButton, x: f32, y: f32 },
    MouseUp { button: MouseButton, x: f32, y: f32 },
    Click,
    
    // キーボードイベント
    KeyDown { key: VirtualKey },
    KeyUp { key: VirtualKey },
    Char { ch: char },
    
    // フォーカスイベント
    GotFocus,
    LostFocus,
    
    // レイアウトイベント
    SizeChanged { new_size: Size2D },
}
```

### イベントハンドラの管理

```rust
pub type EventHandler = Box<dyn Fn(&UiEvent, &mut WidgetSystem) -> EventResponse>;

pub enum EventResponse {
    Handled,      // イベント処理完了
    Propagate,    // 親に伝播
}

pub struct InteractionState {
    is_hovered: bool,
    is_pressed: bool,
    has_focus: bool,
    
    // イベントハンドラ
    handlers: HashMap<EventType, Vec<EventHandler>>,
}

impl WidgetSystem {
    /// イベントハンドラを登録
    pub fn add_event_handler(
        &mut self, 
        widget_id: WidgetId, 
        event_type: EventType,
        handler: EventHandler
    ) {
        // ハンドラを登録
    }
    
    pub fn dispatch_event(&mut self, target_id: WidgetId, event: UiEvent) {
        // イベントをバブリング（親に伝播）
