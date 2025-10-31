# Visual: DirectCompositionとの統合


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
