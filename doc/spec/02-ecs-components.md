# ECSプロパティ管理


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

