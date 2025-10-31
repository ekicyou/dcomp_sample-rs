# システムの統合と更新フロー

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

