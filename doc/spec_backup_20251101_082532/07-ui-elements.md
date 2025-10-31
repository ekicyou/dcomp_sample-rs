# 基本的なUI要素

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
