# ECSと依存関係プロパティの関係性

    
    /// インタラクティブなWidgetか判定
    fn is_interactive(&self, widget_id: WidgetId) -> bool {
        // インタラクションコンポーネントを持つか
        self.interactions.contains_key(widget_id)
            // または、テキスト選択可能か
            || self.is_text_selectable(widget_id)
            // または、背景があるか（クリック可能領域）
            || self.has_background(widget_id)
    }
    
    /// WM_NCHITTESTハンドラから呼ぶ
    pub fn window_hit_test(&self, point: Point2D) -> HitTestResult {
        if let Some(widget_id) = self.hit_test(point) {
            // ヒットしたWidgetに応じて処理
            if self.interactions.contains_key(widget_id) {
                HitTestResult::Client // インタラクティブな要素
            } else {
                HitTestResult::Client // 通常のUI要素
            }
        } else {
            HitTestResult::Transparent // 透過
        }
    }
}

pub enum HitTestResult {
    Client,      // HTCLIENT
    Transparent, // HTTRANSPARENT
    Caption,     // HTCAPTION（ドラッグ可能）
}
```

## 使用例

```rust
fn create_sample_ui(system: &mut WidgetSystem) -> WidgetId {
    // ルートコンテナ
    let root = system.create_stack_panel(Orientation::Vertical);
    
    // タイトルテキスト（縦書き）
    let title = system.create_text_block("伺か".to_string());
    system.set_text_flow(title, FlowDirection::TopToBottom);
    system.append_child(root, title);
    
    // キャラクター画像
    let character = system.create_image("character.png").unwrap();
    system.append_child(root, character);
    
    // ボタン
    let button = system.create_button(|system| {
        println!("ボタンがクリックされました！");
    });
    
    // ボタンのラベル
    let label = system.create_text_block("クリック".to_string());
    system.append_child(button, label);
    system.append_child(root, button);
    
    root
}

fn main() {
    let mut system = WidgetSystem::new();
    let root = create_sample_ui(&mut system);
    
    // レイアウト計算
    system.update_layout(root, Size2D::new(800.0, 600.0));
    
    // 描画更新
    system.update_visuals();
    
    // イベント処理例
    let click_point = Point2D::new(100.0, 200.0);
    if let Some(widget_id) = system.hit_test(click_point) {
        system.dispatch_event(widget_id, UiEvent::Click);
    }
}
```

## ビジュアルツリーの最適化

### Visual作成の判断フロー

```rust
impl WidgetSystem {
    /// Widgetの更新時、Visualが必要か判断
    fn update_widget_visual(&mut self, widget_id: WidgetId) {
        let needs_visual = self.needs_visual(widget_id);
        let has_visual = self.visuals.contains_key(widget_id);
        
        match (needs_visual, has_visual) {
            (true, false) => self.ensure_visual(widget_id),
            (false, true) => self.remove_visual(widget_id),
            (true, true) => self.dirty_visual.insert(widget_id),
            (false, false) => (), // 純粋なレイアウトノード
        }
    }
}
```

### Visualツリーの構造例

論理ツリーとビジュアルツリーは必ずしも1:1対応しない：

```text
論理ツリー (Widget):              ビジュアルツリー (Visual):
Root                               Root
├─ Container (no bg)              ├─ TextBlock1
│  ├─ TextBlock1                  ├─ Image1
│  └─ Container (no bg)           └─ TextBlock2
│     └─ Image1
└─ TextBlock2

中間のContainerはVisualを持たない（効率化）
```

## ECSと依存関係プロパティの関係性

WPFの依存関係プロパティは、実はECSと驚くほど似た構造を持っています。

### 構造的類似性の比較

| 要素 | WPF DependencyProperty | ECS |
|------|------------------------|-----|
| **エンティティ** | DependencyObject | WidgetId (SlotMap key) |
| **プロパティ定義** | static DependencyProperty | コンポーネント型（Layout, Visual等） |
| **値の保存場所** | DependencyObject内部の辞書 | SecondaryMap<WidgetId, Component> |
| **アクセス方法** | GetValue/SetValue | map.get(id) / map.insert(id, value) |
| **メモリ効率** | 使用するプロパティのみ保存 | 使用するコンポーネントのみ保存 |

**本質**: WPFのGetValue/SetValueは、ECSのSecondaryMap get/insertと同じパターン

impl PropertySystem {
    // GetValue相当
    pub fn get<P: Property>(&self, widget_id: WidgetId) -> Option<&P::Value> {
        // 型に応じて適切なSecondaryMapから取得
        // 実装はマクロやtrait経由で自動生成
        todo!()
    }
    
    // SetValue相当
    pub fn set<P: Property>(&mut self, widget_id: WidgetId, value: P::Value) {
        // 型に応じて適切なSecondaryMapに保存
        // ダーティフラグを立てる
        self.dirty_properties
            .entry(widget_id)
            .or_insert_with(HashSet::new)
            .insert(TypeId::of::<P>());
    }
}

// 使用例
let mut system = PropertySystem::new();
let button = system.create_widget();

// SetValue（WPF風）
system.set::<TextProperty>(button, "Click Me".to_string());
system.set::<WidthProperty>(button, 100.0);

// GetValue（WPF風）
if let Some(text) = system.get::<TextProperty>(button) {
    println!("Button text: {}", text);
}
```

### 依存関係プロパティの高度な機能とECS

#### 1. プロパティ値の優先順位（Value Precedence）

WPFでは複数のソースから値が設定される場合の優先順位があります：

```
優先順位（高→低）：
1. アニメーション
2. ローカル値（SetValue）
3. トリガー
4. スタイル
5. 継承値
6. デフォルト値
```

これをECSで表現：

```rust
pub struct PropertyValue<T> {
    animated: Option<T>,      // 優先度1
    local: Option<T>,         // 優先度2
    triggered: Option<T>,     // 優先度3
    styled: Option<T>,        // 優先度4
    inherited: Option<T>,     // 優先度5
    default: T,               // 優先度6
}

impl<T: Clone> PropertyValue<T> {
    pub fn effective_value(&self) -> T {
        self.animated.as_ref()
            .or(self.local.as_ref())
            .or(self.triggered.as_ref())
            .or(self.styled.as_ref())
            .or(self.inherited.as_ref())
            .unwrap_or(&self.default)
            .clone()
    }
}

pub struct PropertySystem {
    // 複数のソースを持つプロパティ値
    width: SecondaryMap<WidgetId, PropertyValue<f32>>,
}
```

#### 2. プロパティ変更通知（Property Changed Callback）

```rust
// WPF風のコールバック
pub struct PropertyMetadata<T> {
    default_value: T,
    // 値が変更されたときのコールバック
    property_changed: Option<fn(&mut PropertySystem, WidgetId, &T, &T)>,
    // 値を強制する（Coerceする）
    coerce_value: Option<fn(&PropertySystem, WidgetId, T) -> T>,
}

impl PropertySystem {
    pub fn set_with_callback<P: Property>(
        &mut self,
        widget_id: WidgetId,
        new_value: P::Value,
    ) {
        let old_value = self.get::<P>(widget_id).cloned();
        
        // 値を強制（例：0未満は0にする）
        let coerced = if let Some(coerce) = P::METADATA.coerce_value {
            coerce(self, widget_id, new_value)
        } else {
            new_value
        };
        
        // 値を設定
        self.set_internal::<P>(widget_id, coerced.clone());
        
        // 変更通知
        if let Some(callback) = P::METADATA.property_changed {
            callback(self, widget_id, &old_value.unwrap(), &coerced);
        }
        
        // ダーティフラグ
        self.mark_dirty(widget_id);
    }
}
```

#### 3. プロパティの継承（Inherited Properties）

```rust
// フォントサイズなど、親から継承するプロパティ
pub struct FontSizeProperty;
impl Property for FontSizeProperty {
    type Value = f32;
    const NAME: &'static str = "FontSize";
    const INHERITS: bool = true;  // ← 継承フラグ
}

impl PropertySystem {
    pub fn get_inherited<P: Property>(
        &self,
        widget_id: WidgetId,
    ) -> Option<&P::Value> 
    where
        P: Property,
        P::Value: Clone,
    {
        // まず自分の値を探す
        if let Some(value) = self.get::<P>(widget_id) {
            return Some(value);
        }
        
        // 継承プロパティなら親を辿る
        if P::INHERITS {
            let mut current = self.widgets.get(widget_id)?.parent;
            while let Some(parent_id) = current {
                if let Some(value) = self.get::<P>(parent_id) {
                    return Some(value);
                }
                current = self.widgets.get(parent_id)?.parent;
            }
        }
        
        // デフォルト値
        Some(&P::METADATA.default_value)
    }
}
```

### まとめ：依存関係プロパティはECSの先駆け

| 観点 | 結論 |
|------|------|
| **概念的類似性** | ✅ DependencyObject = Entity、DependencyProperty = Component |
| **実装的類似性** | ✅ グローバルストレージ = SecondaryMap |
| **メモリ効率** | ✅ 両方とも疎なストレージ（使用するプロパティのみ保存） |
| **拡張性** | ✅ 両方ともプロパティ/コンポーネントを動的に追加可能 |
| **変更追跡** | ✅ 両方ともダーティフラグで効率的な更新 |

**WPFの依存関係プロパティは、実質的にECSアーキテクチャの一種**と言えます。

違いは：
- WPF: クラスベースのOOP文法で隠蔽
- ECS: データ指向設計で明示的

どちらも「オブジェクトとプロパティを分離して管理する」という同じ設計思想を持っています。

## ECSシステム分離設計

