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

**主な操作**:
- `append_child()`: 子Widgetを末尾に追加
- `detach_widget()`: Widgetをツリーから切り離す（Widget自体は残り、再利用可能）
- `delete_widget()`: Widgetを完全に削除（子も再帰的に削除）
- `children()`: 子Widgetを列挙するイテレータ

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
    
    // ★ 依存関係登録: このシステムに依存するWidgetとその影響先
    dependents: DependencyMap,
}

// ビジュアル管理システム
pub struct VisualSystem {
    visual: SecondaryMap<WidgetId, Visual>,
    dirty: HashSet<WidgetId>,
    
    dependents: DependencyMap,
}

// 描画コンテンツ管理システム
pub struct DrawingContentSystem {
    drawing_content: SecondaryMap<WidgetId, DrawingContent>,
    dirty: HashSet<WidgetId>,
    
    dependents: DependencyMap,
}

// テキスト管理システム
pub struct TextSystem {
    text: SecondaryMap<WidgetId, TextContent>,
    dirty: HashSet<WidgetId>,
    
    dependents: DependencyMap,
}

// 画像管理システム
pub struct ImageSystem {
    image: SecondaryMap<WidgetId, ImageContent>,
    dirty: HashSet<WidgetId>,
    
    dependents: DependencyMap,
}

// コンテナスタイル管理システム
pub struct ContainerStyleSystem {
    container: SecondaryMap<WidgetId, ContainerStyle>,
    dirty: HashSet<WidgetId>,
    
    dependents: DependencyMap,
}

// インタラクション管理システム
pub struct InteractionSystem {
    interaction: SecondaryMap<WidgetId, InteractionState>,
    dirty: HashSet<WidgetId>,
    
    dependents: DependencyMap,
}

/// 依存関係マップ
/// Widget単位で「どのシステムが影響を受けるか」を登録
pub struct DependencyMap {
    // WidgetId -> 影響を受けるシステムのフラグ
    dependencies: SecondaryMap<WidgetId, DependencyFlags>,
}

bitflags::bitflags! {
    /// 影響を受けるシステムのフラグ
    pub struct DependencyFlags: u32 {
        const LAYOUT          = 0b0000_0001;
        const VISUAL          = 0b0000_0010;
        const DRAWING_CONTENT = 0b0000_0100;
        const TEXT            = 0b0000_1000;
        const IMAGE           = 0b0001_0000;
        const CONTAINER_STYLE = 0b0010_0000;
        const INTERACTION     = 0b0100_0000;
    }
}

**DependencyMapの主な操作**:
- `register()`: Widget単位で依存関係を登録
- `add_dependency()`: 依存関係フラグを追加
- `get()`: Widgetの依存関係を取得
- `get_widgets_with_flag()`: 特定フラグを持つ全Widgetを取得

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

### 依存関係登録システムの仕組み

この設計は、あなたの提案通り**Widgetごとに依存を登録し、変更時に自動的にダーティを配布**します。

#### 核心的な流れ

1. **Widget作成時に依存を登録**
   - Layout変更でDRAWING_CONTENTに影響
   - Text変更でDRAWING_CONTENTに影響

2. **システム変更時にdirtyマーク**
   - `LayoutSystem.dirty.insert(widget_id)`

3. **フレーム更新時にダーティ伝搬**
   - LayoutSystem → dependentsを確認
   - DRAWING_CONTENTフラグを持つwidget_id
   - DrawingContentSystemにダーティを配布

#### 処理の流れ

**Widget作成時**: 各システムへの依存関係をDependencyFlagsで登録
**プロパティ変更時**: 各システムが自身のdirtyフラグを更新
**フレーム更新時**: 
- レイアウトパス実行
- 各システムからダーティを伝搬（propagate_dirty）
- 影響を受けるシステムを順次更新
- 全ダーティフラグをクリア
- DirectCompositionにコミット
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

**LayoutSystemの主な操作**:
- `set_width()` / `get_width()`: Width プロパティの設定・取得
- `set_height()` / `get_height()`: Height プロパティの設定・取得
- `set_margin()` / `get_margin()`: Margin プロパティの設定・取得（デフォルト値付き）
- `set_padding()` / `get_padding()`: Padding プロパティの設定・取得
- `get_final_rect()`: レイアウト計算後の最終矩形を取得

### ダーティ伝搬戦略

#### 課題
各システムは独立したダーティフラグを持ちますが、システム間には依存関係があります：

```text
Layout変更 → DrawingContent再生成 → Visual更新
Text変更   → DrawingContent再生成 → Visual更新
```

#### 実装戦略: Pull型（遅延評価・推奨）

各システムが更新時に必要な情報を**取りに行く**アプローチ。ECSの原則にもっとも適合。

**処理の流れ**:
1. レイアウトパス実行
2. 描画コンテンツを更新（レイアウト情報をPull）
3. Visualを更新（描画コンテンツをPull）
4. ダーティフラグをクリア
5. DirectCompositionにコミット

**メリット**:
- ECS原則に忠実（システム間の結合度が低い）
- データフローが明確でデバッグしやすい
- 実装がシンプル

**デメリット**:
- UiRuntimeが依存関係を知る必要がある

#### 段階的実装アプローチ

**初期実装**: 単純Pull（UiRuntimeが依存関係を直接記述）

```rust
impl UiRuntime {
    pub fn update_frame(&mut self, root_id: WidgetId) {
        // 1. レイアウトパス
        self.layout.update(&self.widget, root_id, window_size);
        
        // 2. 描画コンテンツパス（Text/Image/ContainerStyleのダーティを統合）
        let mut drawing_dirty = HashSet::new();
        drawing_dirty.extend(self.text.dirty.drain());
        drawing_dirty.extend(self.image.dirty.drain());
        drawing_dirty.extend(self.layout.dirty.iter().copied());
        
        for widget_id in &drawing_dirty {
            self.rebuild_drawing_content(*widget_id);
        }
        
        // 3. Visualパス
        for widget_id in drawing_dirty {
            self.apply_visual_update(widget_id);
        }
        
        // 4. コミット
        self.clear_all_dirty();
        self.visual.commit().ok();
    }
}
```

### プロパティ変更の流れ

各システムが自分のダーティフラグを管理し、変更を追跡する：

```rust
impl LayoutSystem {
    /// レイアウト情報を更新
    pub fn set_layout(&mut self, widget_id: WidgetId, layout: Layout) {
        self.layouts.insert(widget_id, layout);
        self.dirty.insert(widget_id);
        // 子孫もダーティにする（レイアウト伝播）
        self.mark_descendants_dirty(widget_id);
    }
}

impl TextSystem {
    /// テキスト内容を更新
    pub fn set_text(&mut self, widget_id: WidgetId, text: String) {
        if let Some(content) = self.texts.get_mut(widget_id) {
            content.text = text;
            content.invalidate_layout();
            self.dirty.insert(widget_id);
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
- 純粋なレイアウトコンテナー（透明、背景なし）
- 論理的なグループ化のみ

### Visual の定義

```rust
pub struct Visual {
    widget_id: WidgetId, // 対応するWidget
    
    // DirectCompositionオブジェクト（内部実装）
    dcomp_visual: IDCompositionVisual,
    
    // トランスフォーム（Visualが管理）
    offset: Point2D,
    opacity: f32,
    
    // DrawingContentへの参照
    drawing_content: Option<ID2D1Image>,
}
```

## システムの統合と更新フロー
                    dirty.insert(widget_id);
                }
            }
        }
        
        dirty
    }
    
    fn rebuild_drawing_content(&mut self, widget_id: WidgetId) {
        let Some(widget) = self.widget.get(widget_id) else { return };
        let Some(rect) = self.layout.get_final_rect(widget_id) else { return };
        
        self.drawing_content.rebuild_content(widget_id, rect.size, |dc| {
            // フラグに応じて描画
            if widget.render_flags.contains(RenderFlags::USES_TEXT) {
                self.text.draw_to_context(widget_id, dc, &brush, Point2D::zero())?;
            }
            if widget.render_flags.contains(RenderFlags::USES_IMAGE) {
                self.image.draw_to_context(widget_id, dc, rect)?;
            }
            if widget.render_flags.contains(RenderFlags::USES_CONTAINER) {
                self.container_style.draw_to_context(widget_id, dc, rect)?;
            }
            Ok(())
        }).ok();
    }
}
```

**メリット**:
- ✅ メモリ効率が良い（bitflags）
- ✅ 高速（ビット演算）
- ✅ 複数システムの組み合わせが簡単

**デメリット**:
- ⚠️ 描画順序の制御が難しい

#### 最終推奨：戦略A（Widget型による静的ディスパッチ）

**理由**:
1. **依存関係が明確**: match文で一目瞭然、コンパイル時に検証可能
2. **ECS原則に適合**: データ（Widget）とシステム（描画ロジック）の分離
3. **パフォーマンス**: 仮想ディスパッチなし、最適化しやすい
4. **拡張性**: 新しいWidgetTypeを追加するだけ
5. **Rustらしい**: enumのパターンマッチを活用

**WinUI3との違い**:
- WinUI3: OOP + 仮想メソッド（C#の得意分野）
- この設計: ECS + パターンマッチ（Rustの得意分野）

同じ問題を、それぞれの言語の強みを活かして解決しています。

### 他のUIフレームワークの依存管理戦略

主要フレームワークの比較：

| フレームワーク | 戦略 | 依存解決 | カスタム描画 |
|------------|------|---------|------------|
| **Flutter** | RenderObjectツリー + 明示的マーキング | `markNeedsLayout()`/`markNeedsPaint()`を開発者が呼ぶ | ✅ 細かく制御可能 |
| **React** | 仮想DOM + Reconciliation | 変更があったらコンポーネント全体を再レンダリング | useEffect依存配列で制御 |
| **SwiftUI** | @State/@Binding + 自動依存追跡 | プロパティラッパーがアクセスを自動追跡 | ✅ `animatableData`で宣言 |
| **Jetpack Compose** | 再コンポーズ + スマート追跡 | コンパイラが依存グラフを自動生成 | ✅ 自動追跡 |
| **Godot** | ノードシステム + 通知 | `queue_redraw()`を開発者が呼ぶ | ✅ 明示的 |
| **Dear ImGui** | 即時モード | 毎フレーム全再描画 | ❌ 差分なし |

**本設計の位置づけ**: Flutter/Godot的な明示的マーキング + ECS的なシステム分離

```cpp
void RenderUI() {
    // 毎フレーム呼ばれる
    ImGui::Begin("Window");
    
    ImGui::Text("Hello: %s", text.c_str());
    ImGui::ColorEdit3("Color", color);
    
    // カスタム描画
    ImDrawList* draw_list = ImGui::GetWindowDrawList();
    draw_list->AddRect(pos, pos + size, ImColor(color));
    
    ImGui::End();
}

// メインループ
while (running) {
    RenderUI();  // ← 毎フレーム全UIを再構築
}
```

**特徴**:
- ✅ **依存管理不要**：毎フレーム全部再描画
- ✅ 実装が超シンプル
- ⚠️ パフォーマンス：複雑なUIには向かない

**依存解決**: そもそも依存を追跡しない（毎回全部作り直す）

#### 比較まとめ

| フレームワーク | 依存追跡方法 | カスタム描画の制御 | 実装複雑度 | パフォーマンス |
|--------------|-------------|------------------|-----------|-------------|
| **WPF/WinUI3** | プロパティメタデータ | 簡素化（フラグ） | 🟡 中 | 🟢 良好 |
| **Flutter** | 明示的マーキング | 細かく制御可能 | 🟡 中 | 🟢 良好 |
| **React** | 仮想DOM差分 | 保守的（全体再描画） | 🟢 低 | 🟡 中（最適化必要） |
| **SwiftUI** | 自動追跡（@State） | 自動 + 宣言的 | 🟢 低 | 🟢 良好 |
| **Compose** | コンパイラ解析 | 自動追跡 | 🟢 低 | 🟢 良好 |
| **Godot** | 明示的マーキング | 細かく制御可能 | 🟡 中 | 🟢 良好 |
| **ImGui** | 追跡なし（毎フレーム） | 不要（常に再描画） | 🟢 超低 | 🔴 複雑UIで低下 |

#### この設計への示唆

あなたの設計（Rust + ECS）に最適なアプローチは：

##### 推奨：**Flutter/Godotスタイル（明示的マーキング）+ Widget型**

```rust
impl TextSystem {
    pub fn set_text(&mut self, widget_id: WidgetId, text: String) {
        self.text.insert(widget_id, text);
        
        // 明示的に影響範囲を指定（Flutter/Godotスタイル）
        self.mark_dirty(widget_id);  // 自分のシステムのダーティ
        // UiRuntimeが後で依存チェーンを解決
    }
}

// Widget型で静的に依存を表現（前述の戦略A）
pub enum WidgetType {
    Text,      // Text + Layout に依存
    Image,     // Image + Layout に依存
    Container, // ContainerStyle + Layout に依存
    Custom {   // カスタム描画
        renderer_id: TypeId,
        // カスタムレンダラーが依存を宣言
        dependencies: &'static [SystemId],
    },
}

// カスタムレンダラーの例
pub trait CustomRenderer: Send + Sync {
    /// 依存するシステム（コンパイル時定数）
    const DEPENDENCIES: &'static [SystemId];
    
    /// 描画処理
    fn render(&self, ctx: &RenderContext, widget_id: WidgetId) -> Result<()>;
}

struct GradientRenderer;
impl CustomRenderer for GradientRenderer {
    const DEPENDENCIES: &'static [SystemId] = &[
        SystemId::Layout,  // サイズ情報が必要
        // Textなどは不要
    ];
    
    fn render(&self, ctx: &RenderContext, widget_id: WidgetId) -> Result<()> {
        let rect = ctx.layout.get_final_rect(widget_id)?;
        // グラデーション描画
        Ok(())
    }
}
```

**この設計の利点**:
1. ✅ **静的な依存宣言**：`WidgetType`と`CustomRenderer::DEPENDENCIES`
2. ✅ **Rustの型システム活用**：コンパイル時に検証
3. ✅ **拡張性**：カスタムレンダラーが自分の依存を宣言
4. ✅ **パフォーマンス**：不要な再描画を回避
5. ✅ **シンプル**：SwiftUI/Composeのような複雑なコンパイラ不要

**結論**:
- **標準Widget**：`WidgetType` enumで静的に依存を表現
- **カスタム描画**：`CustomRenderer::DEPENDENCIES`定数で依存を宣言
- **依存解決**：UiRuntimeが型情報とDEPENDENCIESから自動構築

これにより、WPFの「プロパティごとのフラグ」よりも細かく、SwiftUI/Composeのような複雑なコンパイラなしで、カスタム描画の依存を厳密に制御できます。

### ECS原則による革新的な依存管理

従来のUIフレームワークは「Widgetが中心」ですが、ECSでは**コンポーネント（データ）とシステム（ロジック）の完全分離**が原則です。この原則を活かした新しいアプローチを提案します。

#### アプローチ: コンポーネントタグによる依存宣言

**核心的アイデア**: Widgetが「どの描画コンポーネントを持つか」で依存関係が決まる。

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
    }
}
```

#### DrawingContentSystemの実装

```rust
pub struct DrawingContentSystem {
    content: SecondaryMap<WidgetId, ID2D1Image>,
    dirty: HashSet<WidgetId>,
    
    // 各Widgetが持つ描画コンポーネントのマップ
    widget_components: SecondaryMap<WidgetId, Vec<RenderComponentType>>,
}
```

**主な操作**:
- `add_render_component()`: 描画コンポーネントを追加（例: Text, Image, Background）
- `get_dependencies()`: Widgetの依存システムを動的に計算
- `rebuild_content()`: ID2D1CommandListに描画コマンドを記録

**使用例**: 複雑なWidget（背景+テキスト+画像アイコン）を構築可能

#### このアプローチの利点（ECS原則）

1. **データとロジックの完全分離**: `RenderComponent`（データ）と`DrawingContentSystem`（ロジック）
2. **組み合わせ可能性**: 1つのWidgetが複数の描画コンポーネントを持てる
   - 例: Background + Text + Image の組み合わせ
3. **静的な依存宣言**: 各`RenderComponent`が`const DEPENDENCIES`を持つ
4. **動的な依存解決**: Widgetが持つコンポーネントから依存を動的に計算
5. **拡張性**: 新しい`RenderComponent`を追加するだけ
6. **型安全**: `RenderComponentType` enumでコンパイル時チェック

#### 比較まとめ

| 観点 | Widget型アプローチ | ECS的コンポーネントアプローチ |
|------|-------------------|---------------------------|
| **依存宣言** | WidgetTypeごと | RenderComponentごと |
| **組み合わせ** | 難しい（型が固定） | 容易（複数コンポーネント） |
| **拡張性** | enumに追加必要 | 新コンポーネント追加のみ |
| **ECS原則** | 🟡 部分的 | ✅ 完全 |

このアプローチは、ECS原則にもっとも忠実で、かつ実用的な解決策です。

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

### 設計原則

ECSアーキテクチャの基本原則に従い、関心事を明確に分離：

1. **Entity（実体）**: `WidgetId` - 全システムで共通のID
2. **Component（コンポーネント）**: 各システムが独自のデータを`SecondaryMap`で管理
3. **System（システム）**: 特定のコンポーネントに対する処理ロジック

### 1. WidgetSystem - ツリー構造管理（コア）

すべてのWidgetの親子関係を管理する基盤。他のシステムはこれを参照してツリーを走査する。
rootは持たず、WindowSystemが管理するWindowがroot Widgetを所有する。

```rust
/// ツリー構造管理（もっとも基本的なシステム）
pub struct WidgetSystem {
    /// 全Widgetの親子関係
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
```

**主な操作**:
- `set_width()` / `get_width()`: Widthプロパティの設定・取得
- `set_height()` / `get_height()`: Heightプロパティの設定・取得
- `set_margin()` / `get_margin()`: Marginプロパティの設定・取得
- `set_padding()` / `get_padding()`: Paddingプロパティの設定・取得
- `set_layout_type()` / `get_layout_type()`: レイアウトタイプの設定・取得
- `mark_dirty()`: ダーティマーク（子孫も再帰的に）
- `update()`: レイアウト更新（Measure/Arrange）
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
2. **テスト容易性**: システムごとに独立してユニットテスト可能
3. **並列処理**: 依存関係のないシステムは並列実行可能（TextとImageなど）
4. **拡張性**: 新しいシステムを追加しやすい（例: AnimationSystem）
5. **メンテナンス性**: 変更の影響範囲が明確
6. **再利用性**: 特定のシステムだけを他のプロジェクトで使える

### パフォーマンス最適化

1. **ダーティフラグ管理**: 変更があったWidgetだけを更新
2. **バッチ処理**: DirectCompositionのコミットは1フレームに1回
3. **キャッシュ活用**: ID2D1CommandListをキャッシュ、レイアウト不変時は再描画不要
4. **並列処理**: TextSystemとImageSystemを並列実行（Rayon等）

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
