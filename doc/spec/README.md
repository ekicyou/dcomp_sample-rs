# slotmapによるプロパティ管理システム

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

```rust
impl WidgetSystem {
    /// 子Widgetを末尾に追加
    pub fn append_child(&mut self, parent_id: WidgetId, child_id: WidgetId) {
        // 1. 子の親を設定
        // 2. 親のlast_childを更新
        // 3. 兄弟リストに連結
    }

    /// Widgetをツリーから切り離す（Widget自体は残る）
    pub fn detach_widget(&mut self, widget_id: WidgetId) {
        // 1. 親の子リストから削除
        // 2. 兄弟リストから切断
        // 3. 親のfirst/last_childを更新
        // 4. 自分のparentをNoneに設定
        // 注: Widgetは削除されず、再度append_childで別の親に追加可能
    }
    
    /// Widgetをツリーから切り離して削除（子も再帰的に削除）
    pub fn delete_widget(&mut self, widget_id: WidgetId) {
        // 1. まずツリーから切り離す
        self.detach_widget(widget_id);
        
        // 2. 子を再帰的に削除
        let children: Vec<_> = self.children(widget_id).collect();
        for child in children {
            self.delete_widget(child);
        }
        
        // 3. SlotMapから削除
        self.widgets.remove(widget_id);
    }

    /// 子を走査
    pub fn children(&self, parent_id: WidgetId) -> impl Iterator<Item = WidgetId> {
        // first_child -> next_sibling -> next_sibling ... と辿る
    }
}
```

## ECS的なプロパティ管理

### 基本方針
- **すべてのウィジットは必ずWidgetを持つ（論理ツリー）**
- **各機能は独立したコンポーネントとして管理（関心の分離）**
- **コンポーネントは独立して存在したりしなかったりする**
- `SecondaryMap`を使い、必要なWidgetだけがプロパティを持つ
- プロパティの変更は「ダーティフラグ」で追跡し、効率的に更新

### コンポーネントの独立性

各コンポーネントは異なるタイミングで必要になり、独立して存在する：

| コンポーネント | 関心のタイミング | 例 |
|--------------|----------------|-----|
| **Layout** | レイアウトパス | サイズ・配置の計算時 |
| **Visual** | 描画パス | ビジュアルツリー構築時 |
| **DrawingContent** | レンダリングパス | 実際の描画コマンド実行時 |
| **TextContent** | コンテンツ更新時 | テキスト変更時 |
| **Interaction** | イベント処理時 | マウス・キーボード入力時 |

### コンポーネントの種類

```rust
use slotmap::{SlotMap, SecondaryMap};

// ツリー構造管理（最も基本的なシステム）
pub struct WidgetSystem {
    widget: SlotMap<WidgetId, Widget>,
}

// レイアウト計算システム
pub struct LayoutSystem {
    // レイアウトプロパティ（後述）
    width: SecondaryMap<WidgetId, Length>,
    height: SecondaryMap<WidgetId, Length>,
    // ... その他のレイアウトプロパティ
    
    dirty: HashSet<WidgetId>,
}

// ビジュアル管理システム
pub struct VisualSystem {
    visual: SecondaryMap<WidgetId, Visual>,
    dirty: HashSet<WidgetId>,
}

// 描画コンテンツ管理システム
pub struct DrawingContentSystem {
    drawing_content: SecondaryMap<WidgetId, DrawingContent>,
    dirty: HashSet<WidgetId>,
}

// テキスト管理システム
pub struct TextSystem {
    text: SecondaryMap<WidgetId, TextContent>,
    dirty: HashSet<WidgetId>,
}

// 画像管理システム
pub struct ImageSystem {
    image: SecondaryMap<WidgetId, ImageContent>,
    dirty: HashSet<WidgetId>,
}

// コンテナスタイル管理システム
pub struct ContainerStyleSystem {
    container: SecondaryMap<WidgetId, ContainerStyle>,
    dirty: HashSet<WidgetId>,
}

// インタラクション管理システム
pub struct InteractionSystem {
    interaction: SecondaryMap<WidgetId, InteractionState>,
    dirty: HashSet<WidgetId>,
}

// 統合ランタイム（すべてのシステムを保持）
pub struct UiRuntime {
    pub widget: WidgetSystem,
    pub layout: LayoutSystem,
    pub visual: VisualSystem,
    pub drawing_content: DrawingContentSystem,
    pub text: TextSystem,
    pub image: ImageSystem,
    pub container_style: ContainerStyleSystem,
    pub interaction: InteractionSystem,
}
```

### システムの責務

#### WidgetSystem
- Widgetツリーの親子関係管理のみ
- 他のシステムの基盤

#### LayoutSystem
- サイズと位置の計算
- Measure/Arrangeパス

#### VisualSystem
- DirectCompositionビジュアルツリー管理
- GPU合成

#### DrawingContentSystem
- Direct2Dコンテンツキャッシュ管理

#### TextSystem / ImageSystem / ContainerStyleSystem
- 各種コンテンツタイプの管理

#### InteractionSystem
- マウス/キーボード入力処理

### コンポーネントの詳細定義

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

このアプローチのメリット：

1. **メモリ効率**: 設定されたプロパティのみメモリを使用
2. **柔軟性**: 各プロパティを独立して変更可能
3. **依存関係プロパティと同じ思想**: WPFのDependencyPropertyと同様の設計
4. **デフォルト値**: SecondaryMapにない場合は暗黙のデフォルト値を使用

```rust
impl LayoutSystem {
    /// Widthを設定（個別プロパティ）
    pub fn set_width(&mut self, widget_id: WidgetId, width: Length) {
        self.width.insert(widget_id, width);
        self.mark_dirty(widget_id);
    }
    
    /// Widthを取得（デフォルト値を返す）
    pub fn get_width(&self, widget_id: WidgetId) -> Length {
        self.width.get(widget_id)
            .cloned()
            .unwrap_or(Length::Auto)  // デフォルト値
    }
    
    /// Marginを設定（個別プロパティ）
    pub fn set_margin(&mut self, widget_id: WidgetId, margin: Margin) {
        self.margin.insert(widget_id, margin);
        self.mark_dirty(widget_id);
    }
    
    /// Marginを取得（デフォルト値を返す）
    pub fn get_margin(&self, widget_id: WidgetId) -> Margin {
        self.margin.get(widget_id)
            .cloned()
            .unwrap_or(Margin::zero())  // デフォルト値
    }
    
    /// 最終矩形を取得
    pub fn get_final_rect(&self, widget_id: WidgetId) -> Option<Rect> {
        self.final_rect.get(widget_id).cloned()
    }
}
```

### ダーティ伝搬戦略

#### 課題
各システムは独立したダーティフラグ(`HashSet<WidgetId>`)を持ちますが、システム間には依存関係があります：

```
Layout変更 → DrawingContent再生成 → Visual更新
Text変更   → DrawingContent再生成 → Visual更新
Image変更  → DrawingContent再生成 → Visual更新
```

#### 実装戦略の比較

##### 戦略1: Pull型（遅延評価・推奨）

各システムが更新時に必要な情報を**取りに行く**アプローチ。ECSの原則に最も適合。

```rust
impl UiRuntime {
    pub fn update_frame(&mut self, root_id: WidgetId) {
        // 1. レイアウトパス
        self.layout.update(&self.widget, root_id, window_size);
        
        // 2. 描画コンテンツパス
        // レイアウトが変更されたWidgetを取得
        let layout_changed: HashSet<_> = self.layout.dirty.iter().copied().collect();
        
        // 各システムのダーティと、レイアウト変更の影響を受けるWidgetを統合
        let mut drawing_dirty = self.text.dirty.clone();
        drawing_dirty.extend(&self.image.dirty);
        drawing_dirty.extend(&self.container_style.dirty);
        drawing_dirty.extend(&layout_changed); // レイアウト変更も含める
        
        // 描画コンテンツを更新
        for widget_id in drawing_dirty.drain() {
            self.update_drawing_content_for_widget(widget_id);
        }
        
        // 3. Visualパス
        // DrawingContent変更とレイアウト変更の両方を処理
        let mut visual_dirty = self.drawing_content.dirty.clone();
        visual_dirty.extend(&layout_changed); // レイアウト変更も含める
        
        for widget_id in visual_dirty.drain() {
            self.update_visual_for_widget(widget_id);
        }
        
        // 4. すべてのダーティフラグをクリア
        self.layout.dirty.clear();
        self.text.dirty.clear();
        self.image.dirty.clear();
        self.container_style.dirty.clear();
        self.drawing_content.dirty.clear();
        
        // 5. コミット
        self.visual.commit().ok();
    }
    
    fn update_drawing_content_for_widget(&mut self, widget_id: WidgetId) {
        // レイアウト情報を取得（Pull）
        let Some(rect) = self.layout.get_final_rect(widget_id) else { return };
        
        // どのシステムが描画内容を持っているか判定
        if self.text.has_text(widget_id) {
            self.drawing_content.rebuild_content(widget_id, rect.size, |dc| {
                let brush = create_brush(dc)?;
                self.text.draw_to_context(widget_id, dc, &brush, Point2D::zero())
            }).ok();
        } else if self.image.has_image(widget_id) {
            self.drawing_content.rebuild_content(widget_id, rect.size, |dc| {
                self.image.draw_to_context(widget_id, dc, rect)
            }).ok();
        } else if self.container_style.has_style(widget_id) {
            self.drawing_content.rebuild_content(widget_id, rect.size, |dc| {
                self.container_style.draw_to_context(widget_id, dc, rect)
            }).ok();
        }
    }
    
    fn update_visual_for_widget(&mut self, widget_id: WidgetId) {
        // DrawingContentとレイアウト情報を取得（Pull）
        let Some(content) = self.drawing_content.get_content(widget_id) else { return };
        let Some(rect) = self.layout.get_final_rect(widget_id) else { return };
        
        self.visual.apply_content(widget_id, content, rect.size).ok();
        self.visual.set_offset(widget_id, rect.origin).ok();
    }
}
```

**メリット**:
- ✅ ECS原則に忠実（システム間の結合度が低い）
- ✅ 各システムが独立して動作
- ✅ デバッグしやすい（データフローが明確）
- ✅ テストしやすい
- ✅ 実装がシンプル

**デメリット**:
- ⚠️ `UiRuntime`が依存関係を知る必要がある
- ⚠️ 更新順序を間違えるとバグになる可能性

##### 戦略2: Push型（即座伝搬）

変更時に影響を受けるシステムに**通知する**アプローチ。

```rust
impl LayoutSystem {
    pub fn set_width(&mut self, widget_id: WidgetId, width: Length) {
        self.width.insert(widget_id, width);
        self.mark_dirty(widget_id);
        
        // 依存システムに通知
        if let Some(propagator) = &mut self.dirty_propagator {
            propagator.notify_layout_changed(widget_id);
        }
    }
}

pub struct DirtyPropagator {
    drawing_content_dirty: HashSet<WidgetId>,
    visual_dirty: HashSet<WidgetId>,
}

impl DirtyPropagator {
    pub fn notify_layout_changed(&mut self, widget_id: WidgetId) {
        // レイアウト変更は描画コンテンツとVisualの両方に影響
        self.drawing_content_dirty.insert(widget_id);
        self.visual_dirty.insert(widget_id);
    }
    
    pub fn notify_text_changed(&mut self, widget_id: WidgetId) {
        // テキスト変更は描画コンテンツとVisualに影響
        self.drawing_content_dirty.insert(widget_id);
        self.visual_dirty.insert(widget_id);
    }
}

impl UiRuntime {
    pub fn update_frame(&mut self, root_id: WidgetId) {
        // すでに伝搬済み
        self.layout.update(&self.widget, root_id, window_size);
        
        // 伝搬されたダーティフラグを使用
        for widget_id in self.propagator.drawing_content_dirty.drain() {
            self.update_drawing_content_for_widget(widget_id);
        }
        
        for widget_id in self.propagator.visual_dirty.drain() {
            self.update_visual_for_widget(widget_id);
        }
        
        self.visual.commit().ok();
    }
}
```

**メリット**:
- ✅ 更新時の判断が不要（すでに伝搬済み）
- ✅ `update_frame`がシンプル

**デメリット**:
- ❌ システム間の結合度が高い（`DirtyPropagator`への参照が必要）
- ❌ ECS原則から外れる
- ❌ デバッグが難しい（伝搬経路が追いにくい）
- ❌ テストが複雑

##### 戦略3: ハイブリッド型（イベントバス）

システム間の通信を`EventBus`経由で行う。

```rust
pub enum SystemEvent {
    LayoutChanged(WidgetId),
    TextChanged(WidgetId),
    ImageChanged(WidgetId),
    ContainerStyleChanged(WidgetId),
}

pub struct EventBus {
    events: Vec<SystemEvent>,
}

impl LayoutSystem {
    pub fn set_width(&mut self, widget_id: WidgetId, width: Length, event_bus: &mut EventBus) {
        self.width.insert(widget_id, width);
        self.mark_dirty(widget_id);
        event_bus.emit(SystemEvent::LayoutChanged(widget_id));
    }
}

impl DrawingContentSystem {
    pub fn process_events(&mut self, events: &[SystemEvent]) {
        for event in events {
            match event {
                SystemEvent::LayoutChanged(id) 
                | SystemEvent::TextChanged(id)
                | SystemEvent::ImageChanged(id)
                | SystemEvent::ContainerStyleChanged(id) => {
                    self.dirty.insert(*id);
                }
            }
        }
    }
}
```

**メリット**:
- ✅ 疎結合
- ✅ 拡張しやすい

**デメリット**:
- ❌ オーバーエンジニアリング（この規模では不要）
- ❌ パフォーマンスオーバーヘッド
- ❌ 実装が複雑

#### 推奨：戦略1改（宣言的Pull型）

**前提認識**: 「影響を受ける側」が依存関係を知っているのが自然。

しかし各システムに依存関係を直接書くと、システム間の結合が発生します。そこで、**各システムが自分の依存を宣言し、UiRuntimeが自動的にチェーンを構築する**アプローチを提案します。

```rust
/// システムの依存関係を宣言
pub trait SystemDependencies {
    /// このシステムが依存する他システムのダーティフラグ
    fn dependencies(&self) -> Vec<SystemId>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemId {
    Widget,
    Layout,
    Visual,
    DrawingContent,
    Text,
    Image,
    ContainerStyle,
    Interaction,
}

/// DrawingContentSystemは複数のシステムに依存
impl SystemDependencies for DrawingContentSystem {
    fn dependencies(&self) -> Vec<SystemId> {
        vec![
            SystemId::Layout,        // レイアウト変更で再描画
            SystemId::Text,          // テキスト変更で再描画
            SystemId::Image,         // 画像変更で再描画
            SystemId::ContainerStyle,// スタイル変更で再描画
        ]
    }
}

/// VisualSystemも複数のシステムに依存
impl SystemDependencies for VisualSystem {
    fn dependencies(&self) -> Vec<SystemId> {
        vec![
            SystemId::Layout,        // レイアウト変更でoffset更新
            SystemId::DrawingContent,// 描画コンテンツ変更でcontent更新
        ]
    }
}

/// UiRuntimeが依存関係を自動解決
impl UiRuntime {
    pub fn update_frame(&mut self, root_id: WidgetId) {
        // 1. レイアウトパス
        self.layout.update(&self.widget, root_id, window_size);
        
        // 2. 描画コンテンツパス（宣言的に依存を収集）
        let drawing_dirty = self.collect_dirty_for_system(SystemId::DrawingContent);
        for widget_id in drawing_dirty {
            self.update_drawing_content_for_widget(widget_id);
        }
        
        // 3. Visualパス（宣言的に依存を収集）
        let visual_dirty = self.collect_dirty_for_system(SystemId::Visual);
        for widget_id in visual_dirty {
            self.update_visual_for_widget(widget_id);
        }
        
        // 4. すべてのダーティフラグをクリア
        self.clear_all_dirty();
        
        // 5. コミット
        self.visual.commit().ok();
    }
    
    /// 指定システムの依存関係から、更新が必要なWidgetを収集
    fn collect_dirty_for_system(&self, system_id: SystemId) -> HashSet<WidgetId> {
        let mut dirty = HashSet::new();
        
        // システムの依存関係を取得
        let dependencies = match system_id {
            SystemId::DrawingContent => self.drawing_content.dependencies(),
            SystemId::Visual => self.visual.dependencies(),
            _ => vec![],
        };
        
        // 依存する各システムのダーティフラグを統合
        for dep in dependencies {
            let dep_dirty = self.get_dirty_for_system(dep);
            dirty.extend(dep_dirty);
        }
        
        // 自分自身のダーティも含める
        let own_dirty = self.get_dirty_for_system(system_id);
        dirty.extend(own_dirty);
        
        dirty
    }
    
    /// システムIDからダーティフラグを取得
    fn get_dirty_for_system(&self, system_id: SystemId) -> &HashSet<WidgetId> {
        match system_id {
            SystemId::Layout => &self.layout.dirty,
            SystemId::Text => &self.text.dirty,
            SystemId::Image => &self.image.dirty,
            SystemId::ContainerStyle => &self.container_style.dirty,
            SystemId::DrawingContent => &self.drawing_content.dirty,
            SystemId::Visual => &self.visual.dirty,
            _ => &HashSet::new(), // 空のセット
        }
    }
}
```

**メリット**:
- ✅ **各システムが自分の依存を宣言**（影響を受ける側が知識を持つ）
- ✅ **依存関係が明示的**（`dependencies()`メソッドで一目瞭然）
- ✅ **システム間の結合度が低い**（SystemIdという抽象化のみ）
- ✅ **拡張が容易**（新システム追加時も依存を宣言するだけ）
- ✅ **テスト可能**（依存関係を変更してテスト可能）

**デメリット**:
- ⚠️ `SystemId` enumの維持が必要
- ⚠️ `get_dirty_for_system`のマッチが必要

##### さらなる改良：マクロによる自動化

依存関係の宣言をさらにシンプルにするマクロを導入できます：

```rust
/// システム定義マクロ
macro_rules! define_system {
    ($name:ident, depends_on: [$($dep:ident),*]) => {
        impl SystemDependencies for $name {
            fn dependencies(&self) -> Vec<SystemId> {
                vec![$(SystemId::$dep),*]
            }
        }
    };
}

// 使用例：宣言的で読みやすい
define_system!(DrawingContentSystem, depends_on: [Layout, Text, Image, ContainerStyle]);
define_system!(VisualSystem, depends_on: [Layout, DrawingContent]);
define_system!(LayoutSystem, depends_on: []); // 依存なし
```

##### 代替案：ビルダーパターンによる依存宣言

マクロを使いたくない場合、ビルダーパターンも検討できます：

```rust
impl DrawingContentSystem {
    pub fn new() -> Self {
        Self {
            // ... フィールド初期化
            dependencies: SystemDependencies::builder()
                .depends_on(SystemId::Layout)
                .depends_on(SystemId::Text)
                .depends_on(SystemId::Image)
                .depends_on(SystemId::ContainerStyle)
                .build(),
        }
    }
}
```

#### 比較まとめ

| アプローチ | 依存を知るのは | 宣言的 | 結合度 | 実装複雑度 |
|-----------|--------------|--------|--------|-----------|
| **戦略1改（宣言的Pull）** | 受ける側 ✓ | ✅ 高い | 🟢 低い | 🟡 中 |
| 戦略1（単純Pull） | UiRuntime | ❌ 低い | 🟡 中 | 🟢 低い |
| 戦略2（Push） | 与える側 | ❌ 低い | 🔴 高い | 🟢 低い |
| 戦略3（EventBus） | 受ける側 | 🟡 中 | 🟢 低い | 🔴 高い |

#### 最終推奨：戦略1改（宣言的Pull型）

**理由**:
1. **自然な依存関係**: 影響を受ける側が依存を宣言（プログラム設計的に正しい）
2. **ECS原則に忠実**: システム間の直接結合なし（SystemIdという抽象化のみ）
3. **保守性が高い**: 依存関係が`dependencies()`メソッドに集約
4. **拡張容易**: 新システム追加時も依存を宣言するだけ
5. **テスタビリティ**: 各システムの依存を個別にテスト可能
6. **パフォーマンス**: HashSetの`extend`操作は高速（O(n)）

**実装の選択肢**:
- **シンプル重視**: トレイトメソッドで宣言
- **簡潔重視**: マクロで宣言（`define_system!`）
- **明示重視**: ビルダーパターンで宣言

#### 段階的実装アプローチ

まずは**戦略1（単純Pull）** で実装を開始し、依存関係が複雑になってきたら**戦略1改（宣言的Pull）** にリファクタリングすることを推奨します。

**フェーズ1: 単純Pull（初期実装）**
```rust
// UiRuntimeが依存関係を直接記述（シンプルで明快）
impl UiRuntime {
    pub fn update_frame(&mut self, root_id: WidgetId) {
        // 明示的な順序で処理
        let layout_dirty = self.update_layout(root_id);
        let drawing_dirty = self.update_drawing_content(&layout_dirty);
        self.update_visuals(&layout_dirty, &drawing_dirty);
        self.visual.commit().ok();
    }
    
    fn update_layout(&mut self, root_id: WidgetId) -> HashSet<WidgetId> {
        self.layout.update(&self.widget, root_id, window_size);
        self.layout.dirty.drain().collect()
    }
    
    fn update_drawing_content(&mut self, layout_dirty: &HashSet<WidgetId>) 
        -> HashSet<WidgetId> 
    {
        // すべての描画系システムのダーティを統合
        let mut dirty = HashSet::new();
        dirty.extend(self.text.dirty.drain());
        dirty.extend(self.image.dirty.drain());
        dirty.extend(self.container_style.dirty.drain());
        dirty.extend(layout_dirty.iter().copied()); // レイアウト変更も影響
        
        for widget_id in &dirty {
            self.rebuild_drawing_content(*widget_id);
        }
        
        dirty
    }
    
    fn update_visuals(
        &mut self, 
        layout_dirty: &HashSet<WidgetId>,
        drawing_dirty: &HashSet<WidgetId>
    ) {
        let mut dirty = drawing_dirty.clone();
        dirty.extend(layout_dirty.iter().copied()); // レイアウト変更も影響
        
        for widget_id in dirty {
            self.apply_visual_update(widget_id);
        }
    }
}
```

**フェーズ2: 宣言的Pull（リファクタリング後）**

システム数や依存関係が増えてきたら、宣言的アプローチに移行：

```rust
// 各システムが自分の依存を宣言
impl DrawingContentSystem {
    fn dependencies(&self) -> Vec<SystemId> {
        vec![SystemId::Layout, SystemId::Text, SystemId::Image, SystemId::ContainerStyle]
    }
}

// UiRuntimeは依存を自動解決（保守性向上）
impl UiRuntime {
    pub fn update_frame(&mut self, root_id: WidgetId) {
        self.layout.update(&self.widget, root_id, window_size);
        
        let drawing_dirty = self.collect_dirty_for_system(SystemId::DrawingContent);
        for widget_id in drawing_dirty {
            self.rebuild_drawing_content(widget_id);
        }
        
        let visual_dirty = self.collect_dirty_for_system(SystemId::Visual);
        for widget_id in visual_dirty {
            self.apply_visual_update(widget_id);
        }
        
        self.clear_all_dirty();
        self.visual.commit().ok();
    }
}
```

この段階的アプローチにより：
- ✅ 初期実装がシンプル（オーバーエンジニアリング回避）
- ✅ 複雑化したときのリファクタリングパスが明確
- ✅ 各フェーズで動作するコードを維持

#### Visual（ビジュアルツリー管理）
描画が必要なWidgetのみ。DirectCompositionを使用するが、それと同一ではない。

```rust
pub struct Visual {
    // DirectCompositionオブジェクト
    dcomp_visual: IDCompositionVisual,
    
    // トランスフォーム
    offset: Point2D,
    scale: Vector2D,
    rotation: f32,
    opacity: f32,
    
    // 状態
    visible: bool,
    clip_rect: Option<Rect>,
}
```

#### DrawingContent（描画コマンド）
**ID2D1Imageベースで統一管理**。ほぼすべての描画要素が持つ。

```rust
pub struct DrawingContent {
    // 描画コンテンツ（ID2D1Imageで統一）
    content: ID2D1Image,
    
    // コンテンツの種類
    content_type: ContentType,
    
    // キャッシュ情報
    is_cached: bool,
    cache_valid: bool,
    last_update: Instant,
}

pub enum ContentType {
    // ID2D1Bitmap（画像ファイルなど）
    Bitmap,
    
    // ID2D1CommandList（描画コマンド記録）
    CommandList,
    
    // ID2D1Effect（エフェクト適用）
    Effect,
    
    // DirectWriteから生成
    Text,
}
```

### ID2D1Imageによる描画コマンド管理の利点

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

## Visual: DirectCompositionとの統合

### コンポーネントの分離

描画に関わる要素を3つのコンポーネントに分離：

1. **Visual** - ビジュアルツリーの管理（DirectCompositionを使用）
2. **DrawingContent** - 描画コマンド（ID2D1Image）
3. **Layout** - サイズ・配置情報

これらは独立して存在し、異なるタイミングで更新される。

### Visual の役割
- **描画が必要なWidgetのみが持つ（動的に作成）**
- ビジュアルツリーのノード（DirectCompositionを内部で使用）
- トランスフォーム、不透明度、クリッピングなどの表示属性

### Visualが必要なWidget
- テキストを表示する（TextBlock）
- 画像を表示する（Image）
- 背景色・枠線を持つ（Container with background）
- カスタム描画を行う

### Visualが不要なWidget
- 純粋なレイアウトコンテナ（透明、背景なし）
- 論理的なグループ化のみ

### Visual の定義

```rust
pub struct Visual {
    widget_id: WidgetId, // 対応するWidget
    
    // DirectCompositionオブジェクト（内部実装）
    dcomp_visual: IDCompositionVisual,
    
    // トランスフォーム（Visualが管理）
    offset: Point2D,
    scale: Vector2D,
    rotation: f32,
    
    // 表示属性
    opacity: f32,
    visible: bool,
    clip_rect: Option<Rect>,
}
```

### DrawingContent の役割
**ID2D1Imageベースの描画コマンド管理**。ほぼすべての描画要素が持つ。

```rust
pub struct DrawingContent {
    widget_id: WidgetId,
    
    // 描画コンテンツ（ID2D1Imageで統一）
    content: ID2D1Image,
    content_type: ContentType,
    
    // キャッシュ管理
    is_cached: bool,
    cache_valid: bool,
    
    // サイズ情報（レイアウトと協調）
    intrinsic_size: Option<Size2D>,
}
```

### 更新フローの分離

```rust
impl WidgetSystem {
    /// フレーム更新
    pub fn update_frame(&mut self) {
        // 1. レイアウトパス（サイズ・配置の計算）
        self.update_layouts();
        
        // 2. コンテンツパス（描画コマンドの生成）
        self.update_drawing_contents();
        
        // 3. ビジュアルパス（DirectCompositionツリーの更新）
        self.update_dcomp_visuals();
        
        // 4. コミット
        self.dcomp_context.commit().unwrap();
    }
    
    /// レイアウト更新（最優先）
    fn update_layouts(&mut self) {
        for widget_id in self.dirty_layout.drain().collect::<Vec<_>>() {
            self.measure_and_arrange(widget_id);
        }
    }
    
    /// 描画コンテンツ更新（レイアウト確定後）
    fn update_drawing_contents(&mut self) {
        for widget_id in self.dirty_content.drain().collect::<Vec<_>>() {
            if self.needs_drawing_content(widget_id) {
                self.rebuild_drawing_content(widget_id);
            }
        }
    }
    
    /// Visual更新（コンテンツ確定後）
    fn update_visuals(&mut self) {
        for widget_id in self.dirty_visual.drain().collect::<Vec<_>>() {
            if self.needs_visual(widget_id) {
                self.ensure_visual(widget_id);
                self.apply_content_to_visual(widget_id);
            } else {
                self.remove_visual(widget_id);
            }
        }
    }
}
```

### DrawingContent の生成

**ID2D1CommandListを使った描画コマンドの記録とキャッシュ**

```rust
impl WidgetSystem {
    /// 描画コンテンツを再構築
    fn rebuild_drawing_content(&mut self, widget_id: WidgetId) -> Result<()> {
        let layout = self.layouts.get(widget_id).unwrap();
        
        // コマンドリストを作成（描画を記録）
        let command_list = self.d2d_device_context.CreateCommandList()?;
        self.d2d_device_context.SetTarget(&command_list);
        
        self.d2d_device_context.BeginDraw();
        
        // コンテンツの種類に応じて描画
        if let Some(text) = self.texts.get(widget_id) {
            self.draw_text_to_context(text, layout)?;
        } else if let Some(image) = self.images.get(widget_id) {
            self.draw_image_to_context(image, layout)?;
        } else if let Some(container) = self.containers.get(widget_id) {
            self.draw_container_to_context(container, layout)?;
        }
        
        self.d2d_device_context.EndDraw(None, None)?;
        command_list.Close()?;
        
        // DrawingContentとして保存（ID2D1Imageとして扱える）
        let content = DrawingContent {
            widget_id,
            content: command_list.cast::<ID2D1Image>()?,
            content_type: ContentType::CommandList,
            is_cached: true,
            cache_valid: true,
            intrinsic_size: Some(layout.final_rect.size),
        };
        
        self.drawing_contents.insert(widget_id, content);
        
        Ok(())
    }
    
    /// このWidgetが描画コンテンツを必要とするか判定
    fn needs_drawing_content(&self, widget_id: WidgetId) -> bool {
        self.texts.contains_key(widget_id) 
            || self.images.contains_key(widget_id)
            || self.has_background(widget_id)
            || self.has_custom_draw(widget_id)
    }
    
    /// このWidgetがVisualを必要とするか判定
    fn needs_visual(&self, widget_id: WidgetId) -> bool {
        // DrawingContentを持つ = 描画が必要 = Visualが必要
        self.drawing_contents.contains_key(widget_id)
    }
}
```

### Widget と Visual の同期

```rust
impl WidgetSystem {
    /// 新しいWidgetを作成（Visualは作成しない）
    pub fn create_widget(&mut self) -> WidgetId {
        self.widgets.insert(Widget::new())
    }
    
    /// Visualを動的に作成・取得
    pub fn ensure_visual(&mut self, widget_id: WidgetId) -> Result<()> {
        if self.visuals.contains_key(widget_id) {
            return Ok(()); // 既に存在
        }
        
        unsafe {
            let dcomp_visual = self.dcomp_device.CreateVisual()?;
            
            let visual = Visual {
                widget_id,
                dcomp_visual,
                offset: Point2D::zero(),
                scale: Vector2D::new(1.0, 1.0),
                rotation: 0.0,
                opacity: 1.0,
                visible: true,
                clip_rect: None,
            };
            
            self.visuals.insert(widget_id, visual);
            
            // 親のVisualツリーに接続
            self.attach_visual_to_tree(widget_id)?;
        }
        
        Ok(())
    }
    
    /// DrawingContentをVisualのサーフェスに適用
    fn apply_content_to_visual(&mut self, widget_id: WidgetId) -> Result<()> {
        let visual = self.visuals.get(widget_id).unwrap();
        let layout = self.layouts.get(widget_id).unwrap();
        
        // サーフェスを作成
        let surface = self.dcomp_device.CreateSurface(
            layout.final_rect.size.width as u32,
            layout.final_rect.size.height as u32,
            DXGI_FORMAT_B8G8R8A8_UNORM,
            DXGI_ALPHA_MODE_PREMULTIPLIED,
        )?;
        
        unsafe {
            // サーフェスに描画
            let mut offset = POINT::default();
            let dc = surface.BeginDraw(None, &mut offset)?;
            
            dc.Clear(Some(&D2D1_COLOR_F {
                r: 0.0, g: 0.0, b: 0.0, a: 0.0, // 透明
            }));
            
            // DrawingContentを描画（ID2D1Imageとして）
            if let Some(content) = self.drawing_contents.get(widget_id) {
                dc.DrawImage(
                    &content.content,
                    None,
                    None,
                    D2D1_INTERPOLATION_MODE_LINEAR,
                    None,
                );
            }
            
            dc.Flush(None, None)?;
            surface.EndDraw()?;
            
            // サーフェスをVisualに設定
            visual.dcomp_visual.SetContent(&surface)?;
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
        self.interactions
            .entry(widget_id)
            .or_insert_with(InteractionState::new)
            .handlers
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(handler);
    }
    
    /// イベントをディスパッチ（バブリング）
    pub fn dispatch_event(&mut self, target_id: WidgetId, event: UiEvent) {
        let mut current_id = Some(target_id);
        
        while let Some(widget_id) = current_id {
            if let Some(interaction) = self.interactions.get_mut(widget_id) {
                if let Some(handlers) = interaction.handlers.get_mut(&event.event_type()) {
                    for handler in handlers {
                        match handler(&event, self) {
                            EventResponse::Handled => return,
                            EventResponse::Propagate => continue,
                        }
                    }
                }
            }
            
            // 親に伝播
            current_id = self.widgets.get(widget_id).and_then(|w| w.parent);
        }
    }
}
```

## 基本的なUI要素

### 1. Container（コンテナ）

もっともシンプルなUI要素。子を配置するための器。
**背景色や枠線がない場合、Visualは作成されない（効率化）**

```rust
pub struct ContainerStyle {
    padding: Padding,
    background: Option<Color>,
    border: Option<Border>,
}

impl WidgetSystem {
    pub fn create_container(&mut self) -> WidgetId {
        let widget_id = self.create_widget();
        
        // レイアウト情報を追加
        self.layouts.insert(widget_id, Layout {
            width: Length::Auto,
            height: Length::Auto,
            padding: Padding::zero(),
            ..Default::default()
        });
        
        // Visualは背景や枠線が設定されたときに作成される
        
        widget_id
    }
    
    /// 背景色を設定（Visualを作成）
    pub fn set_background(&mut self, widget_id: WidgetId, color: Color) {
        // スタイル情報を保存（新しいSecondaryMap）
        self.container_styles
            .entry(widget_id)
            .or_insert_with(ContainerStyle::default)
            .background = Some(color);
        
        // Visualが必要になったのでダーティフラグ
        self.dirty_visual.insert(widget_id);
    }
}
```

### 2. TextBlock（テキストブロック）

テキストを表示するUI要素。縦書き対応が重要。**Visualを動的に作成する。**

```rust
pub struct TextContent {
    text: String,
    
    // フォント設定
    font_family: String,
    font_size: f32,
    font_weight: u32,
    
    // 縦書き設定
    flow_direction: FlowDirection, // TopToBottom or LeftToRight
    reading_direction: ReadingDirection, // TopToBottom or LeftToRight
    
    // DirectWriteオブジェクト
    text_format: IDWriteTextFormat,
    text_layout: IDWriteTextLayout,
}

#[derive(Clone, Copy)]
pub enum FlowDirection {
    TopToBottom,  // 縦書き
    LeftToRight,  // 横書き
}

impl WidgetSystem {
    pub fn create_text_block(&mut self, text: String) -> WidgetId {
        let widget_id = self.create_widget();
        
        // テキストコンテンツを追加
        let text_content = TextContent::new(
            text,
            &self.dwrite_factory,
            FlowDirection::TopToBottom, // デフォルトは縦書き
        );
        self.texts.insert(widget_id, text_content);
        
        // レイアウト情報を追加
        self.layouts.insert(widget_id, Layout::default());
        
        // Visualは描画時に自動作成される
        self.dirty_visual.insert(widget_id);
        
        widget_id
    }
}
```

### 3. Image（画像）

画像を表示するUI要素。透過対応。**Visualを動的に作成する。**

```rust
pub struct ImageContent {
    // 画像データ
    bitmap: ID2D1Bitmap,
    source_rect: Option<Rect>,
    
    // 表示設定
    stretch: Stretch,
    opacity: f32,
}

#[derive(Clone, Copy)]
pub enum Stretch {
    None,           // 原寸
    Fill,           // 引き伸ばし
    Uniform,        // アスペクト比維持
    UniformToFill,  // アスペクト比維持して埋める
}

impl WidgetSystem {
    pub fn create_image(&mut self, image_path: &str) -> Result<WidgetId> {
        let widget_id = self.create_widget();
        
        // WICで画像を読み込み
        let bitmap = self.load_image_with_wic(image_path)?;
        
        let image_content = ImageContent {
            bitmap,
            source_rect: None,
            stretch: Stretch::Uniform,
            opacity: 1.0,
        };
        self.images.insert(widget_id, image_content);
        
        // レイアウト情報を追加
        self.layouts.insert(widget_id, Layout::default());
        
        // Visualは描画時に自動作成される
        self.dirty_visual.insert(widget_id);
        
        Ok(widget_id)
    }
}
```

### 4. Button（ボタン）

クリック可能なUI要素。

```rust
pub struct ButtonState {
    is_hovered: bool,
    is_pressed: bool,
    is_enabled: bool,
    
    // ビジュアルステート用のコンテンツ
    normal_visual: VisualId,
    hover_visual: VisualId,
    pressed_visual: VisualId,
}

impl WidgetSystem {
    pub fn create_button<F>(&mut self, on_click: F) -> WidgetId 
    where
        F: Fn(&mut WidgetSystem) + 'static,
    {
        let widget_id = self.create_widget();
        
        // インタラクション状態を追加
        let interaction = InteractionState::new();
        self.interactions.insert(widget_id, interaction);
        
        // クリックイベントハンドラを登録
        self.add_event_handler(
            widget_id,
            EventType::Click,
            Box::new(move |event, system| {
                on_click(system);
                EventResponse::Handled
            }),
        );
        
        // マウスホバー時の視覚的フィードバック
        self.add_event_handler(
            widget_id,
            EventType::MouseEnter,
            Box::new(|event, system| {
                // ホバー状態の更新
                EventResponse::Handled
            }),
        );
        
        widget_id
    }
}
```

### 5. StackPanel（スタックパネル）

子要素を縦または横に配置するコンテナ。

```rust
pub struct StackLayout {
    orientation: Orientation,
    spacing: f32,
}

#[derive(Clone, Copy)]
pub enum Orientation {
    Vertical,
    Horizontal,
}

impl WidgetSystem {
    pub fn create_stack_panel(&mut self, orientation: Orientation) -> WidgetId {
        let widget_id = self.create_widget();
        
        self.layouts.insert(widget_id, Layout {
            layout_type: LayoutType::Stack(StackLayout {
                orientation,
                spacing: 0.0,
            }),
            ..Default::default()
        });
        
        widget_id
    }
    
    /// スタックレイアウトの計算
    fn measure_stack(&self, widget_id: WidgetId) -> Size2D {
        let layout = self.layouts.get(widget_id).unwrap();
        let stack_layout = match &layout.layout_type {
            LayoutType::Stack(s) => s,
            _ => return Size2D::zero(),
        };
        
        let mut total_size = Size2D::zero();
        
        for child_id in self.children(widget_id) {
            let child_size = self.measure_widget(child_id);
            
            match stack_layout.orientation {
                Orientation::Vertical => {
                    total_size.height += child_size.height + stack_layout.spacing;
                    total_size.width = total_size.width.max(child_size.width);
                }
                Orientation::Horizontal => {
                    total_size.width += child_size.width + stack_layout.spacing;
                    total_size.height = total_size.height.max(child_size.height);
                }
            }
        }
        
        total_size
    }
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
    horizontal_alignment: Alignment,
    vertical_alignment: Alignment,
    
    // レイアウトタイプ
    layout_type: LayoutType,
    
    // 計算結果（キャッシュ）
    desired_size: Size2D,
    final_rect: Rect,
}

pub enum Length {
    Auto,
    Pixels(f32),
    Percent(f32),
}

pub enum LayoutType {
    None,
    Stack(StackLayout),
    // 将来的に追加
    // Grid(GridLayout),
    // Flex(FlexLayout),
}

impl WidgetSystem {
    /// レイアウトを更新（2パス）
    pub fn update_layout(&mut self, root_id: WidgetId, available_size: Size2D) {
        // パス1: Measure（子から親へ、必要なサイズを計算）
        self.measure_widget_recursive(root_id, available_size);
        
        // パス2: Arrange（親から子へ、最終位置を決定）
        let final_rect = Rect::new(Point2D::zero(), available_size);
        self.arrange_widget_recursive(root_id, final_rect);
        
        // Visualに反映
        self.apply_layout_to_visuals();
    }
    
    fn measure_widget_recursive(&mut self, widget_id: WidgetId, available: Size2D) -> Size2D {
        let layout = self.layouts.get(widget_id);
        
        // レイアウトタイプに応じて子を計測
        let desired_size = match &layout.map(|l| &l.layout_type) {
            Some(LayoutType::Stack(_)) => self.measure_stack(widget_id),
            _ => Size2D::zero(),
        };
        
        // 制約を適用
        let constrained = self.apply_constraints(widget_id, desired_size);
        
        // 結果を保存
        if let Some(layout) = self.layouts.get_mut(widget_id) {
            layout.desired_size = constrained;
        }
        
        constrained
    }
    
    fn arrange_widget_recursive(&mut self, widget_id: WidgetId, final_rect: Rect) {
        // 自分の最終矩形を保存
        if let Some(layout) = self.layouts.get_mut(widget_id) {
            layout.final_rect = final_rect;
        }
        
        // 子を配置
        for child_id in self.children(widget_id) {
            let child_rect = self.calculate_child_rect(widget_id, child_id, final_rect);
            self.arrange_widget_recursive(child_id, child_rect);
        }
    }
    
    fn apply_layout_to_visuals(&mut self) {
        for (widget_id, layout) in &self.layouts {
            if let Some(visual) = self.visuals.get_mut(widget_id) {
                visual.offset = layout.final_rect.origin;
                visual.size = layout.final_rect.size;
                
                // DirectCompositionに反映
                visual.dcomp_visual.SetOffsetX(layout.final_rect.origin.x).unwrap();
                visual.dcomp_visual.SetOffsetY(layout.final_rect.origin.y).unwrap();
            }
        }
    }
}
```

## ヒットテストシステム

### ヒットテストの実装

**Visualの有無に関わらず、Widgetツリーでヒットテストを行う**

```rust
impl WidgetSystem {
    /// 座標からWidgetを検索
    pub fn hit_test(&self, point: Point2D) -> Option<WidgetId> {
        // ルートから深さ優先探索（Z順序を考慮）
        self.hit_test_recursive(self.root_id, point)
    }
    
    fn hit_test_recursive(&self, widget_id: WidgetId, point: Point2D) -> Option<WidgetId> {
        // レイアウト情報から矩形を取得
        let layout = self.layouts.get(widget_id)?;
        
        // この矩形内か？
        if !layout.final_rect.contains(point) {
            return None;
        }
        
        // 子を逆順で検索（後に追加した子が上に表示される）
        let children: Vec<_> = self.children(widget_id).collect();
        for child_id in children.iter().rev() {
            // 子の座標系に変換
            let local_point = self.to_local_coordinates(*child_id, point);
            if let Some(hit) = self.hit_test_recursive(*child_id, local_point) {
                return Some(hit);
            }
        }
        
        // 子でヒットしなければ、インタラクティブなWidgetならヒット
        if self.is_interactive(widget_id) {
            Some(widget_id)
        } else {
            None // 透過（親に伝播）
        }
    }
    
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
            (true, false) => {
                // Visualを新規作成
                self.ensure_visual(widget_id);
                self.dirty_visual.insert(widget_id);
            }
            (false, true) => {
                // Visualを削除（不要になった）
                self.remove_visual(widget_id);
            }
            (true, true) => {
                // Visualを更新
                self.dirty_visual.insert(widget_id);
            }
            (false, false) => {
                // 何もしない（純粋なレイアウトノード）
            }
        }
    }
    
    fn remove_visual(&mut self, widget_id: WidgetId) {
        if let Some(visual) = self.visuals.remove(widget_id) {
            // DirectCompositionツリーから削除
            if let Some(parent_id) = self.find_parent_with_visual(widget_id) {
                let parent_visual = self.visuals.get(parent_id).unwrap();
                unsafe {
                    parent_visual.dcomp_visual
                        .RemoveVisual(&visual.dcomp_visual)
                        .ok();
                }
            }
        }
    }
}
```

### Visualツリーの構造例

論理ツリーとビジュアルツリーは必ずしも1:1対応しない：

```
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

### 依存関係プロパティ（DependencyProperty）の本質

WPFの依存関係プロパティは、一見複雑に見えますが、実はECSと驚くほど似た構造を持っています。

#### WPFの依存関係プロパティ（従来の理解）

```csharp
// WPF の DependencyProperty
public class Button : UIElement
{
    // スタティックな「プロパティ定義」
    public static readonly DependencyProperty TextProperty =
        DependencyProperty.Register(
            "Text",
            typeof(string),
            typeof(Button)
        );
    
    // インスタンスプロパティ（アクセサ）
    public string Text
    {
        get { return (string)GetValue(TextProperty); }
        set { SetValue(TextProperty, value); }
    }
}

// 使用例
Button button = new Button();
button.Text = "Click Me";  // 実際は GetValue/SetValue を呼んでいる
```

#### ECS的に読み解く

```rust
// ECS的な解釈：DependencyProperty = コンポーネント型の定義

// 1. スタティックな「プロパティ定義」= コンポーネント型
pub struct TextProperty;  // 型がIDの役割

// 2. 実体は外部の「ストレージ」に保存
pub struct PropertySystem {
    // DependencyObject(=Entity) ごとに値を保存
    text_value: SecondaryMap<DependencyObjectId, String>,
    width_value: SecondaryMap<DependencyObjectId, f64>,
    // ... 各プロパティごとにマップ
}

// 3. GetValue/SetValue = コンポーネントのget/set
impl DependencyObject {
    pub fn get_value<T>(&self, property: &Property<T>) -> Option<&T> {
        // プロパティシステムから値を取得
        PROPERTY_SYSTEM.get(self.id, property)
    }
    
    pub fn set_value<T>(&mut self, property: &Property<T>, value: T) {
        // プロパティシステムに値を保存
        PROPERTY_SYSTEM.set(self.id, property, value);
        // ダーティフラグを立てる
        self.invalidate();
    }
}
```

### 構造的類似性の比較

| 要素 | WPF DependencyProperty | ECS |
|------|------------------------|-----|
| **エンティティ** | DependencyObject | WidgetId (SlotMap key) |
| **プロパティ定義** | static DependencyProperty | コンポーネント型（Layout, Visual等） |
| **値の保存場所** | DependencyObject内部の辞書 | SecondaryMap<WidgetId, Component> |
| **アクセス方法** | GetValue/SetValue | map.get(id) / map.insert(id, value) |
| **プロパティの追加** | 動的に登録可能 | 新しいSecondaryMapを追加 |
| **メモリ効率** | 使用するプロパティのみ保存 | 使用するコンポーネントのみ保存 |

### WPFの内部実装（概念的）

```csharp
// WPFの内部実装（簡略化）
public class DependencyObject
{
    private int _objectId;  // ← Entity ID
    
    // すべてのDependencyObjectが共有する「プロパティストレージ」
    private static Dictionary<(int objectId, DependencyProperty prop), object> 
        _globalPropertyStore = new();
    
    public object GetValue(DependencyProperty property)
    {
        // グローバルストレージから取得（ECSのSecondaryMap.get相当）
        var key = (_objectId, property);
        if (_globalPropertyStore.TryGetValue(key, out var value))
            return value;
        return property.DefaultValue;  // デフォルト値
    }
    
    public void SetValue(DependencyProperty property, object value)
    {
        // グローバルストレージに保存（ECSのSecondaryMap.insert相当）
        var key = (_objectId, property);
        _globalPropertyStore[key] = value;
        
        // 変更通知（ダーティフラグ相当）
        InvalidateProperty(property);
    }
}
```

### ECS版の依存関係プロパティ

```rust
// Rust + ECSで依存関係プロパティを実装

// プロパティ定義（型レベル）
pub trait Property {
    type Value;
    const NAME: &'static str;
}

// 具体的なプロパティ定義
pub struct WidthProperty;
impl Property for WidthProperty {
    type Value = f32;
    const NAME: &'static str = "Width";
}

pub struct TextProperty;
impl Property for TextProperty {
    type Value = String;
    const NAME: &'static str = "Text";
}

// プロパティシステム（グローバルストレージ）
pub struct PropertySystem {
    widget: SlotMap<WidgetId, Widget>,
    
    // 各プロパティのストレージ（WPFの_globalPropertyStore相当）
    width: SecondaryMap<WidgetId, f32>,
    height: SecondaryMap<WidgetId, f32>,
    text: SecondaryMap<WidgetId, String>,
    color: SecondaryMap<WidgetId, Color>,
    
    // 変更通知（ダーティフラグ）
    dirty_properties: HashMap<WidgetId, HashSet<TypeId>>,
}

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

### 設計原則

ECSアーキテクチャの基本原則に従い、関心事を明確に分離：

1. **Entity（実体）**: `WidgetId` - 全システムで共通のID
2. **Component（コンポーネント）**: 各システムが独自のデータを`SecondaryMap`で管理
3. **System（システム）**: 特定のコンポーネントに対する処理ロジック

### 1. WidgetSystem - ツリー構造管理（コア）

すべてのWidgetの親子関係を管理する基盤。他のシステムはこれを参照してツリーを走査する。
rootは持たず、WindowSystemが管理するWindowがroot Widgetを所有する。

```rust
/// ツリー構造管理（最も基本的なシステム）
pub struct WidgetSystem {
    /// 全Widgetの親子関係
    widget: SlotMap<WidgetId, Widget>,
}

impl WidgetSystem {
    /// 新しいWidgetを作成
    pub fn create_widget(&mut self) -> WidgetId {
        self.widget.insert(Widget::new())
    }
    
    /// 子Widgetを追加
    pub fn append_child(&mut self, parent_id: WidgetId, child_id: WidgetId) -> Result<()> {
        // 連結リスト操作
        let child = self.widget.get_mut(child_id)
            .ok_or(Error::InvalidWidgetId)?;
        child.parent = Some(parent_id);
        
        let parent = self.widget.get_mut(parent_id)
            .ok_or(Error::InvalidWidgetId)?;
        
        if let Some(last_child) = parent.last_child {
            self.widget.get_mut(last_child).unwrap().next_sibling = Some(child_id);
        } else {
            parent.first_child = Some(child_id);
        }
        parent.last_child = Some(child_id);
        
        Ok(())
    }
    
    /// Widgetをツリーから切り離す（Widgetは削除されない）
    pub fn detach_widget(&mut self, widget_id: WidgetId) -> Result<()> {
        let widget = self.widgets.get_mut(widget_id)
            .ok_or(Error::InvalidWidgetId)?;
        
        let parent_id = widget.parent;
        let next_sibling = widget.next_sibling;
        
        // 親から切り離す
        if let Some(parent_id) = parent_id {
            let parent = self.widgets.get_mut(parent_id).unwrap();
            
            // 親のfirst_childを更新
            if parent.first_child == Some(widget_id) {
                parent.first_child = next_sibling;
            }
            
            // 親のlast_childを更新
            if parent.last_child == Some(widget_id) {
                // 前の兄弟を探す
                let mut prev_sibling = None;
                let mut current = parent.first_child;
                while let Some(current_id) = current {
                    if current_id == widget_id {
                        break;
                    }
                    prev_sibling = current;
                    current = self.widgets.get(current_id).and_then(|w| w.next_sibling);
                }
                parent.last_child = prev_sibling;
            }
            
            // 前の兄弟のnext_siblingを更新
            let mut current = parent.first_child;
            while let Some(current_id) = current {
                let current_widget = self.widgets.get(current_id).unwrap();
                if current_widget.next_sibling == Some(widget_id) {
                    self.widgets.get_mut(current_id).unwrap().next_sibling = next_sibling;
                    break;
                }
                current = current_widget.next_sibling;
            }
        }
        
        // Widgetのツリー情報をクリア
        let widget = self.widgets.get_mut(widget_id).unwrap();
        widget.parent = None;
        widget.next_sibling = None;
        // 注: first_child, last_childはそのまま（子はまだ存在）
        
        Ok(())
    }
    
    /// Widgetを完全に削除（子も再帰的に削除）
    pub fn delete_widget(&mut self, widget_id: WidgetId) -> Result<()> {
        // 1. ツリーから切り離す
        self.detach_widget(widget_id)?;
        
        // 2. 子を再帰的に削除
        let children: Vec<_> = self.children(widget_id).collect();
        for child in children {
            self.delete_widget(child)?;
        }
        
        // 3. SlotMapから削除
        self.widgets.remove(widget_id);
        
        Ok(())
    }
    
    /// 子を列挙
    pub fn children(&self, parent_id: WidgetId) -> impl Iterator<Item = WidgetId> + '_ {
        WidgetChildrenIterator::new(self, parent_id)
    }
    
    /// 親を取得
    pub fn parent(&self, widget_id: WidgetId) -> Option<WidgetId> {
        self.widgets.get(widget_id).and_then(|w| w.parent)
    }
    
    /// Widgetの存在確認
    pub fn contains(&self, widget_id: WidgetId) -> bool {
        self.widgets.contains_key(widget_id)
    }
}
```

### 2. LayoutSystem - レイアウト計算

Widgetのサイズと位置を計算する。2パスレイアウト（Measure/Arrange）を実装。
各プロパティは個別のSecondaryMapで管理（ECS/依存関係プロパティの原則）。

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

impl LayoutSystem {
    /// Widthを設定
    pub fn set_width(&mut self, widget_id: WidgetId, width: Length) {
        self.width.insert(widget_id, width);
        self.mark_dirty(widget_id);
    }
    
    /// Widthを取得（デフォルト値付き）
    pub fn get_width(&self, widget_id: WidgetId) -> Length {
        self.width.get(widget_id).cloned().unwrap_or(Length::Auto)
    }
    
    /// Heightを設定
    pub fn set_height(&mut self, widget_id: WidgetId, height: Length) {
        self.height.insert(widget_id, height);
        self.mark_dirty(widget_id);
    }
    
    /// Heightを取得（デフォルト値付き）
    pub fn get_height(&self, widget_id: WidgetId) -> Length {
        self.height.get(widget_id).cloned().unwrap_or(Length::Auto)
    }
    
    /// Marginを設定
    pub fn set_margin(&mut self, widget_id: WidgetId, margin: Margin) {
        self.margin.insert(widget_id, margin);
        self.mark_dirty(widget_id);
    }
    
    /// Marginを取得（デフォルト値付き）
    pub fn get_margin(&self, widget_id: WidgetId) -> Margin {
        self.margin.get(widget_id).cloned().unwrap_or(Margin::zero())
    }
    
    /// Paddingを設定
    pub fn set_padding(&mut self, widget_id: WidgetId, padding: Padding) {
        self.padding.insert(widget_id, padding);
        self.mark_dirty(widget_id);
    }
    
    /// Paddingを取得（デフォルト値付き）
    pub fn get_padding(&self, widget_id: WidgetId) -> Padding {
        self.padding.get(widget_id).cloned().unwrap_or(Padding::zero())
    }
    
    /// レイアウトタイプを設定
    pub fn set_layout_type(&mut self, widget_id: WidgetId, layout_type: LayoutType) {
        self.layout_type.insert(widget_id, layout_type);
        self.mark_dirty(widget_id);
    }
    
    /// レイアウトタイプを取得
    pub fn get_layout_type(&self, widget_id: WidgetId) -> LayoutType {
        self.layout_type.get(widget_id).cloned().unwrap_or(LayoutType::None)
    }
    
    /// ダーティマーク（子孫も再帰的に）
    pub fn mark_dirty(&mut self, widget_id: WidgetId) {
        self.dirty.insert(widget_id);
    }
    
    /// レイアウト更新（Measure/Arrange）
    pub fn update(&mut self, widget_system: &WidgetSystem, root_id: WidgetId, available_size: Size2D) {
        if self.dirty.is_empty() {
            return; // 変更なし
        }
        
        // Measureパス（子から親へ、必要なサイズを計算）
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

impl DrawingContentSystem {
    /// コンテンツを再構築（ID2D1CommandListに記録）
    pub fn rebuild_content<F>(
        &mut self,
        widget_id: WidgetId,
        size: Size2D,
        draw_fn: F,
    ) -> Result<()>
    where
        F: FnOnce(&ID2D1DeviceContext) -> Result<()>,
    {
        unsafe {
            let command_list = self.d2d_context.CreateCommandList()?;
            self.d2d_context.SetTarget(&command_list);
            
            self.d2d_context.BeginDraw();
            draw_fn(&self.d2d_context)?;
            self.d2d_context.EndDraw(None, None)?;
            
            command_list.Close()?;
            
            let content = DrawingContent {
                widget_id,
                content: command_list.cast()?,
                content_type: ContentType::CommandList,
                is_cached: true,
                cache_valid: true,
                intrinsic_size: Some(size),
            };
            
            self.contents.insert(widget_id, content);
        }
        
        Ok(())
    }
    
    /// コンテンツ取得
    pub fn get_content(&self, widget_id: WidgetId) -> Option<&ID2D1Image> {
        self.contents.get(widget_id).map(|c| &c.content)
    }
    
    /// コンテンツを無効化
    pub fn invalidate(&mut self, widget_id: WidgetId) {
        if let Some(content) = self.contents.get_mut(widget_id) {
            content.cache_valid = false;
        }
        self.mark_dirty(widget_id);
    }
    
    /// ダーティマーク
    pub fn mark_dirty(&mut self, widget_id: WidgetId) {
        self.dirty.insert(widget_id);
    }
}
```

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

impl TextSystem {
    /// テキストを設定
    pub fn set_text(&mut self, widget_id: WidgetId, text: String) {
        if let Some(content) = self.texts.get_mut(widget_id) {
            content.text = text;
            // テキストレイアウトを再作成
            content.invalidate_layout();
            self.mark_dirty(widget_id);
        } else {
            let content = TextContent::new(text, &self.dwrite_factory, FlowDirection::TopToBottom);
            self.texts.insert(widget_id, content);
            self.mark_dirty(widget_id);
        }
    }
    
    /// テキストを取得
    pub fn get_text(&self, widget_id: WidgetId) -> Option<&str> {
        self.texts.get(widget_id).map(|c| c.text.as_str())
    }
    
    /// フォント設定
    pub fn set_font(&mut self, widget_id: WidgetId, family: String, size: f32) {
        if let Some(content) = self.texts.get_mut(widget_id) {
            content.font_family = family;
            content.font_size = size;
            content.invalidate_layout();
            self.mark_dirty(widget_id);
        }
    }
    
    /// 描画コマンドを生成（DrawingContentSystemと連携）
    pub fn draw_to_context(
        &self,
        widget_id: WidgetId,
        dc: &ID2D1DeviceContext,
        brush: &ID2D1Brush,
        origin: Point2D,
    ) -> Result<()> {
        if let Some(text) = self.texts.get(widget_id) {
            unsafe {
                dc.DrawTextLayout(
                    D2D1_POINT_2F { x: origin.x, y: origin.y },
                    &text.text_layout,
                    brush,
                    D2D1_DRAW_TEXT_OPTIONS_NONE,
                )?;
            }
        }
        Ok(())
    }
    
    /// 固有サイズを計算（レイアウト用）
    pub fn measure_text(&self, widget_id: WidgetId) -> Option<Size2D> {
        self.texts.get(widget_id).and_then(|t| {
            unsafe {
                let mut metrics = DWRITE_TEXT_METRICS::default();
                t.text_layout.GetMetrics(&mut metrics).ok()?;
                Some(Size2D::new(metrics.width, metrics.height))
            }
        })
    }
    
    fn mark_dirty(&mut self, widget_id: WidgetId) {
        self.dirty.insert(widget_id);
    }
}
```

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

impl ImageSystem {
    /// 画像をロード
    pub fn load_image(
        &mut self,
        widget_id: WidgetId,
        path: &str,
        d2d_context: &ID2D1DeviceContext,
    ) -> Result<()> {
        // WICで画像を読み込み
        let bitmap = self.load_bitmap_from_file(path, d2d_context)?;
        
        let content = ImageContent {
            bitmap,
            source_rect: None,
            stretch: Stretch::Uniform,
            opacity: 1.0,
        };
        
        self.images.insert(widget_id, content);
        self.mark_dirty(widget_id);
        
        Ok(())
    }
    
    /// 画像を取得
    pub fn get_image(&self, widget_id: WidgetId) -> Option<&ID2D1Bitmap> {
        self.images.get(widget_id).map(|c| &c.bitmap)
    }
    
    /// 伸縮モードを設定
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

impl VisualSystem {
    /// Visualを作成または取得
    pub fn ensure_visual(&mut self, widget_id: WidgetId) -> Result<()> {
        if !self.visuals.contains_key(widget_id) {
            let dcomp_visual = unsafe { self.dcomp_device.CreateVisual()? };
            
            let visual = Visual {
                widget_id,
                dcomp_visual,
                offset: Point2D::zero(),
                scale: Vector2D::new(1.0, 1.0),
                rotation: 0.0,
                opacity: 1.0,
                visible: true,
                clip_rect: None,
            };
            
            self.visuals.insert(widget_id, visual);
        }
        
        Ok(())
    }
    
    /// Visualを削除
    pub fn remove_visual(&mut self, widget_id: WidgetId) {
        self.visuals.remove(widget_id);
    }
    
    /// DrawingContentをVisualに適用
    pub fn apply_content(
        &mut self,
        widget_id: WidgetId,
        content: &ID2D1Image,
        size: Size2D,
    ) -> Result<()> {
        self.ensure_visual(widget_id)?;
        let visual = self.visuals.get(widget_id).unwrap();
        
        // サーフェスを作成
        let surface = unsafe {
            self.dcomp_device.CreateSurface(
                size.width as u32,
                size.height as u32,
                DXGI_FORMAT_B8G8R8A8_UNORM,
                DXGI_ALPHA_MODE_PREMULTIPLIED,
            )?
        };
        
        // サーフェスに描画
        unsafe {
            let mut offset = POINT::default();
            let dc = surface.BeginDraw(None, &mut offset)?;
            
            dc.Clear(Some(&D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }));
            dc.DrawImage(content, None, None, D2D1_INTERPOLATION_MODE_LINEAR, None);
            
            surface.EndDraw()?;
            visual.dcomp_visual.SetContent(&surface)?;
        }
        
        Ok(())
    }
    
    /// トランスフォームを更新
    pub fn set_offset(&mut self, widget_id: WidgetId, offset: Point2D) -> Result<()> {
        if let Some(visual) = self.visuals.get_mut(widget_id) {
            visual.offset = offset;
            unsafe {
                visual.dcomp_visual.SetOffsetX(offset.x)?;
                visual.dcomp_visual.SetOffsetY(offset.y)?;
            }
        }
        Ok(())
    }
    
    /// 不透明度を設定
    pub fn set_opacity(&mut self, widget_id: WidgetId, opacity: f32) -> Result<()> {
        if let Some(visual) = self.visuals.get_mut(widget_id) {
            visual.opacity = opacity;
            unsafe {
                visual.dcomp_visual.SetOpacity(opacity)?;
            }
        }
        Ok(())
    }
    
    /// コミット（画面に反映）
    pub fn commit(&self) -> Result<()> {
        unsafe { self.dcomp_device.Commit() }
    }
    
    fn mark_dirty(&mut self, widget_id: WidgetId) {
        self.dirty.insert(widget_id);
    }
}
```

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

impl InteractionSystem {
    /// イベントハンドラを登録
    pub fn add_handler(
        &mut self,
        widget_id: WidgetId,
        event_type: EventType,
        handler: EventHandler,
    ) {
        self.interactions
            .entry(widget_id)
            .or_insert_with(InteractionState::new)
            .handlers
            .entry(event_type)
            .or_insert_with(Vec::new)
            .push(handler);
    }
    
    /// イベントをディスパッチ（バブリング）
    pub fn dispatch_event(
        &mut self,
        widget_system: &WidgetSystem,
        target_id: WidgetId,
        event: &UiEvent,
    ) -> EventResponse {
        let mut current_id = Some(target_id);
        
        while let Some(widget_id) = current_id {
            if let Some(interaction) = self.interactions.get_mut(widget_id) {
                if let Some(handlers) = interaction.handlers.get_mut(&event.event_type()) {
                    for handler in handlers {
                        if let EventResponse::Handled = handler(event) {
                            return EventResponse::Handled;
                        }
                    }
                }
            }
            
            // 親に伝播
            current_id = widget_system.parent(widget_id);
        }
        
        EventResponse::Propagate
    }
    
    /// ヒットテスト
    pub fn hit_test(
        &self,
        widget_system: &WidgetSystem,
        layout_system: &LayoutSystem,
        root_id: WidgetId,
        point: Point2D,
    ) -> Option<WidgetId> {
        // ルートから深さ優先探索
        self.hit_test_recursive(widget_system, layout_system, root_id, point)
    }
    
    /// フォーカスを設定
    pub fn set_focus(&mut self, widget_id: Option<WidgetId>) {
        if let Some(old_focus) = self.focused_widget {
            if let Some(interaction) = self.interactions.get_mut(old_focus) {
                interaction.has_focus = false;
            }
        }
        
        if let Some(new_focus) = widget_id {
            if let Some(interaction) = self.interactions.get_mut(new_focus) {
                interaction.has_focus = true;
            }
        }
        
        self.focused_widget = widget_id;
    }
    
    fn hit_test_recursive(
        &self,
        widget_system: &WidgetSystem,
        layout_system: &LayoutSystem,
        widget_id: WidgetId,
        point: Point2D,
    ) -> Option<WidgetId> {
        let rect = layout_system.get_final_rect(widget_id)?;
        
        if !rect.contains(point) {
            return None;
        }
        
        // 子を逆順で検索（Z順序）
        let children: Vec<_> = widget_system.children(widget_id).collect();
        for child_id in children.iter().rev() {
            if let Some(hit) = self.hit_test_recursive(widget_system, layout_system, *child_id, point) {
                return Some(hit);
            }
        }
        
        // インタラクティブなら自分を返す
        if self.is_interactive(widget_id) {
            Some(widget_id)
        } else {
            None // 透過
        }
    }
    
    fn is_interactive(&self, widget_id: WidgetId) -> bool {
        self.interactions.contains_key(widget_id)
    }
}
```

### 8. ContainerStyleSystem - コンテナスタイル管理

背景色、枠線などのスタイル情報を管理。

```rust
pub struct ContainerStyleSystem {
    /// コンテナスタイル
    styles: SecondaryMap<WidgetId, ContainerStyle>,
    
    /// ダーティフラグ
    dirty: HashSet<WidgetId>,
}

impl ContainerStyleSystem {
    /// 背景色を設定
    pub fn set_background(&mut self, widget_id: WidgetId, color: Color) {
        self.styles
            .entry(widget_id)
            .or_insert_with(ContainerStyle::default)
            .background = Some(color);
        self.mark_dirty(widget_id);
    }
    
    /// 枠線を設定
    pub fn set_border(&mut self, widget_id: WidgetId, border: Border) {
        self.styles
            .entry(widget_id)
            .or_insert_with(ContainerStyle::default)
            .border = Some(border);
        self.mark_dirty(widget_id);
    }
    
    /// パディングを設定
    pub fn set_padding(&mut self, widget_id: WidgetId, padding: Padding) {
        self.styles
            .entry(widget_id)
            .or_insert_with(ContainerStyle::default)
            .padding = padding;
        self.mark_dirty(widget_id);
    }
    
    /// 描画コマンドを生成
    pub fn draw_to_context(
        &self,
        widget_id: WidgetId,
        dc: &ID2D1DeviceContext,
        rect: Rect,
    ) -> Result<()> {
        if let Some(style) = self.styles.get(widget_id) {
            unsafe {
                // 背景を描画
                if let Some(color) = style.background {
                    let brush = dc.CreateSolidColorBrush(
                        &D2D1_COLOR_F {
                            r: color.r,
                            g: color.g,
                            b: color.b,
                            a: color.a,
                        },
                        None,
                    )?;
                    dc.FillRectangle(&rect.into(), &brush);
                }
                
                // 枠線を描画
                if let Some(border) = &style.border {
                    let brush = dc.CreateSolidColorBrush(
                        &D2D1_COLOR_F {
                            r: border.color.r,
                            g: border.color.g,
                            b: border.color.b,
                            a: border.color.a,
                        },
                        None,
                    )?;
                    dc.DrawRectangle(&rect.into(), &brush, border.thickness, None);
                }
            }
        }
        Ok(())
    }
    
    fn mark_dirty(&mut self, widget_id: WidgetId) {
        self.dirty.insert(widget_id);
    }
}
```

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

impl UiRuntime {
    /// フレーム更新（すべてのシステムを協調して更新）
    /// root_id: Windowが所有するroot Widget
    pub fn update_frame(&mut self, root_id: WidgetId) {
        // 1. レイアウトパス（サイズ・位置計算）
        let window_size = Size2D::new(800.0, 600.0); // 仮
        self.layout.update(&self.widget, root_id, window_size);
        
        // 2. 描画コンテンツパス
        self.update_drawing_contents();
        
        // 3. Visualパス（DirectCompositionツリー更新）
        self.update_visuals();
        
        // 4. コミット
        self.visual.commit().ok();
    }
    
    /// 描画コンテンツを更新
    fn update_drawing_contents(&mut self) {
        // テキスト、画像、コンテナスタイルから描画コマンドを生成
        
        // テキストの描画コンテンツ
        for widget_id in self.text.dirty.drain().collect::<Vec<_>>() {
            if let Some(rect) = self.layout.get_final_rect(widget_id) {
                self.drawing_content.rebuild_content(widget_id, rect.size, |dc| {
                    // ブラシを作成
                    let brush = unsafe {
                        dc.CreateSolidColorBrush(
                            &D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
                            None,
                        )?
                    };
                    
                    self.text.draw_to_context(widget_id, dc, &brush, Point2D::zero())
                }).ok();
            }
        }
        
        // 画像の描画コンテンツ
        for widget_id in self.image.dirty.drain().collect::<Vec<_>>() {
            if let Some(rect) = self.layout.get_final_rect(widget_id) {
                self.drawing_content.rebuild_content(widget_id, rect.size, |dc| {
                    self.image.draw_to_context(widget_id, dc, rect)
                }).ok();
            }
        }
        
        // コンテナスタイルの描画コンテンツ
        for widget_id in self.container_style.dirty.drain().collect::<Vec<_>>() {
            if let Some(rect) = self.layout.get_final_rect(widget_id) {
                self.drawing_content.rebuild_content(widget_id, rect.size, |dc| {
                    self.container_style.draw_to_context(widget_id, dc, rect)
                }).ok();
            }
        }
    }
    
    /// Visualを更新
    fn update_visuals(&mut self) {
        for widget_id in self.drawing_content.dirty.drain().collect::<Vec<_>>() {
            if let Some(content) = self.drawing_content.get_content(widget_id) {
                if let Some(rect) = self.layout.get_final_rect(widget_id) {
                    self.visual.apply_content(widget_id, content, rect.size).ok();
                    self.visual.set_offset(widget_id, rect.origin).ok();
                }
            }
        }
    }
    
    /// Widgetを作成（高レベルAPI）
    pub fn create_text_widget(&mut self, text: String) -> WidgetId {
        let widget_id = self.widget.create_widget();
        self.text.set_text(widget_id, text);
        // レイアウトプロパティは個別に設定（必要なものだけ）
        self.layout.set_width(widget_id, Length::Auto);
        self.layout.set_height(widget_id, Length::Auto);
        widget_id
    }
    
    /// イメージWidgetを作成
    pub fn create_image_widget(&mut self, path: &str) -> Result<WidgetId> {
        let widget_id = self.widget.create_widget();
        self.image.load_image(widget_id, path, &self.drawing_content.d2d_context)?;
        // レイアウトプロパティは個別に設定
        self.layout.set_width(widget_id, Length::Auto);
        self.layout.set_height(widget_id, Length::Auto);
        Ok(widget_id)
    }
    
    /// コンテナWidgetを作成
    pub fn create_container(&mut self) -> WidgetId {
        let widget_id = self.widget.create_widget();
        // デフォルトではプロパティを設定しない（全てデフォルト値）
        // 必要に応じて個別に設定
        widget_id
    }
    
    /// スタックパネルを作成
    pub fn create_stack_panel(&mut self, orientation: Orientation) -> WidgetId {
        let widget_id = self.widget.create_widget();
        self.layout.set_layout_type(widget_id, LayoutType::Stack(StackLayout {
            orientation,
            spacing: 0.0,
        }));
        widget_id
    }
    
    /// イベント処理
    /// root_id: Windowが所有するroot Widget
    pub fn handle_mouse_down(&mut self, root_id: WidgetId, x: f32, y: f32) {
        let point = Point2D::new(x, y);
        if let Some(widget_id) = self.interaction.hit_test(
            &self.widget,
            &self.layout,
            root_id,
            point
        ) {
            let event = UiEvent::MouseDown { button: MouseButton::Left, x, y };
            self.interaction.dispatch_event(&self.widget, widget_id, &event);
        }
    }
}
```

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
/// WindowSystemが管理する各Window
pub struct Window {
    hwnd: HWND,
    root_widget_id: WidgetId,  // このWindowのルートWidget
    dcomp_target: IDCompositionTarget,
}

pub struct WindowSystem {
    windows: HashMap<HWND, Window>,
}

impl WindowSystem {
    /// 新しいWindowを作成（WidgetSystemにルートWidgetを作成）
    pub fn create_window(
        &mut self,
        ui_runtime: &mut UiRuntime,
    ) -> Result<HWND> {
        // OSウィンドウを作成
        let hwnd = unsafe { CreateWindowExW(...) };
        
        // ルートWidgetを作成（Windowとして機能）
        let root_widget_id = ui_runtime.widget_system.create_widget();
        
        // DirectCompositionターゲットを作成
        let dcomp_target = unsafe {
            ui_runtime.visual.dcomp_device
                .CreateTargetForHwnd(hwnd, true)?
        };
        
        // Windowを登録
        let window = Window {
            hwnd,
            root_widget_id,
            dcomp_target,
        };
        self.windows.insert(hwnd, window);
        
        Ok(hwnd)
    }
    
    /// WindowのルートWidgetを取得
    pub fn get_root_widget(&self, hwnd: HWND) -> Option<WidgetId> {
        self.windows.get(&hwnd).map(|w| w.root_widget_id)
    }
    
    /// Windowを閉じる（ルートWidgetも削除）
    pub fn close_window(
        &mut self,
        hwnd: HWND,
        ui_runtime: &mut UiRuntime,
    ) -> Result<()> {
        if let Some(window) = self.windows.remove(&hwnd) {
            // OSウィンドウを閉じる
            unsafe { DestroyWindow(hwnd) };
            
            // ルートWidgetを削除（子も再帰的に削除される）
            ui_runtime.widget_system.delete_widget(window.root_widget_id)?;
        }
        Ok(())
    }
}
```

### UiRuntimeとWindowSystemの協調

```rust
// UiRuntimeは特定のWindowに依存しない（汎用的なUI管理）
let mut ui_runtime = UiRuntime::new();

// WindowSystemは複数のWindowを管理
let mut window_system = WindowSystem::new();

// Window1を作成
let hwnd1 = window_system.create_window(&mut ui_runtime)?;
let root1 = window_system.get_root_widget(hwnd1).unwrap();

// Window1にUI要素を追加
let text = ui_runtime.create_text_widget("Hello Window 1".to_string());
ui_runtime.widget_system.append_child(root1, text)?;

// Window2を作成（別のツリー）
let hwnd2 = window_system.create_window(&mut ui_runtime)?;
let root2 = window_system.get_root_widget(hwnd2).unwrap();

// Window2にUI要素を追加
let image = ui_runtime.create_image_widget("icon.png")?;
ui_runtime.widget_system.append_child(root2, image)?;

// 各Windowを個別に更新
ui_runtime.update_frame(root1);
ui_runtime.update_frame(root2);

// Widgetをあるウィンドウから別のウィンドウへ移動
// textをWindow1から切り離し
ui_runtime.widget_system.detach_widget(text)?;
// textをWindow2に追加
ui_runtime.widget_system.append_child(root2, text)?;

// レイアウトプロパティを個別に設定（ECS的）
let container = ui_runtime.create_container();
ui_runtime.layout.set_width(container, Length::Pixels(200.0));
ui_runtime.layout.set_height(container, Length::Pixels(100.0));
ui_runtime.layout.set_margin(container, Margin {
    left: 10.0,
    top: 10.0,
    right: 10.0,
    bottom: 10.0,
});
ui_runtime.layout.set_padding(container, Padding {
    left: 5.0,
    top: 5.0,
    right: 5.0,
    bottom: 5.0,
});

// 背景色を設定
ui_runtime.container_style.set_background(container, Color {
    r: 1.0, g: 1.0, b: 1.0, a: 1.0,
});
```

この設計により：
- **マルチウィンドウ対応**: 複数のWindowが独立したWidgetツリーを持てる
- **統一的なWidget管理**: WindowもTextBlockも同じWidgetSystemで管理
- **柔軟なUI構築**: detach/appendでWidget（UIコンポーネント）を自由に移動可能
- **効率的なリソース管理**: 切り離したWidgetは削除せずに再利用できる

### detach_widgetとdelete_widgetの使い分け

```rust
// パターン1: Widgetを別の親に移動（detach → append）
let widget = ui_runtime.create_text_widget("移動可能".to_string());
ui_runtime.widget_system.append_child(parent1, widget)?;

// 後で親を変更
ui_runtime.widget_system.detach_widget(widget)?;  // parent1から切り離す
ui_runtime.widget_system.append_child(parent2, widget)?;  // parent2に追加

// パターン2: Widgetを一時的に非表示（detachのみ）
ui_runtime.widget_system.detach_widget(widget)?;  // ツリーから外れる
// Widgetは存在するが、どのツリーにも属さない（描画されない）

// 後で再表示
ui_runtime.widget_system.append_child(parent1, widget)?;

// パターン3: Widgetを完全に削除（delete）
ui_runtime.widget_system.delete_widget(widget)?;  // 完全に削除
// この後、widgetは無効なIDになる
```

### 分離のメリット

1. **単一責任**: 各システムが1つの明確な責務を持つ
2. **テスト容易性**: システムごとに独立してユニットテスト可能
3. **並列処理**: 依存関係のないシステムは並列実行可能（TextとImageなど）
4. **拡張性**: 新しいシステムを追加しやすい
5. **メンテナンス性**: 変更の影響範囲が明確
6. **再利用性**: 特定のシステムだけを他のプロジェクトで使える

### システム追加の例: AnimationSystem

```rust
pub struct AnimationSystem {
    animations: SecondaryMap<WidgetId, Vec<Animation>>,
    active_animations: HashSet<WidgetId>,
}

impl AnimationSystem {
    pub fn animate_opacity(
        &mut self,
        widget_id: WidgetId,
        from: f32,
        to: f32,
        duration: Duration,
    ) {
        let animation = Animation::Opacity { from, to, duration, elapsed: Duration::ZERO };
        self.animations
            .entry(widget_id)
            .or_insert_with(Vec::new)
            .push(animation);
        self.active_animations.insert(widget_id);
    }
    
    pub fn update(&mut self, delta_time: Duration, visual_system: &mut VisualSystem) {
        for widget_id in &self.active_animations {
            if let Some(animations) = self.animations.get_mut(*widget_id) {
                for animation in animations.iter_mut() {
                    animation.update(delta_time);
                    
                    // アニメーション値をVisualSystemに適用
                    match animation {
                        Animation::Opacity { current, .. } => {
                            visual_system.set_opacity(*widget_id, *current).ok();
                        }
                    }
                }
            }
        }
    }
}
```

### パフォーマンス最適化

1. **ダーティフラグ管理**
   - 各システムが自分のダーティフラグを持つ
   - 変更があったWidgetだけを更新

2. **バッチ処理**
   - 複数のWidgetの更新を一度に処理
   - DirectCompositionのコミットは1フレームに1回

3. **キャッシュ活用**
   - DrawingContentSystemでID2D1CommandListをキャッシュ
   - レイアウトが変わらなければ再描画不要

4. **並列処理**
   - TextSystemとImageSystemは並列実行可能
   - Rayon等を使った並列化を検討

## まとめ

このUI構造設計の要点：

1. **ECS的な管理**: SlotMapとSecondaryMapで柔軟なプロパティ管理
2. **必須コンポーネント**: すべてのWidgetはWidget（ツリー構造）を持つ
3. **動的Visual作成**: 描画が必要なWidgetのみがVisual（DirectComposition）を持つ
4. **オプショナルコンポーネント**: Layout、TextContent、ImageContent、InteractionStateなど必要に応じて追加
5. **イベントシステム**: ハンドラベースで柔軟なイベント処理
6. **2パスレイアウト**: Measure/Arrangeで効率的なレイアウト計算
7. **ヒットテスト**: Widgetツリーを使った深さ優先探索（Visualの有無に依存しない）
8. **基本UI要素**: Container、TextBlock、Image、Button、StackPanelを提供
9. **効率的なメモリ使用**: 不要なVisualを作成しない
10. **段階的な分離**: 現在は`WidgetSystem`で統合管理、将来的にシステム分離を検討
