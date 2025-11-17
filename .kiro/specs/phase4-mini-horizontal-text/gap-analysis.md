# Gap Analysis: phase4-mini-horizontal-text

**Feature ID**: `phase4-mini-horizontal-text`  
**Analysis Date**: 2025-11-17  
**Status**: Complete

---

## Executive Summary

DirectWriteを使用した横書きテキストレンダリング機能を既存のwintfフレームワークに統合するギャップ分析。既存のRectangle描画パターンを踏襲し、Labelウィジットとして実装する。

**重要**: 本実装は横書きに焦点を当てるが、Phase 7での縦書き対応を前提とした設計とする。API命名、コンポーネント構造、システム設計は将来の方向指定拡張を考慮し、横書き専用にならないよう注意する。

### 主要な発見
- **既存基盤の充実度**: DirectWriteファクトリーはGraphicsCoreに統合済み、TextFormat作成APIも実装済み
- **実装パターンの明確性**: Rectangleウィジットが成熟した実装パターンを提供
- **主要ギャップ**: TextLayout生成API、DrawTextLayout呼び出し、Labelコンポーネント定義
- **統合の容易性**: 既存システムへの統合ポイントが明確で、破壊的変更不要

### 推奨アプローチ
**Option C: Hybrid Approach（拡張+新規作成）** を推奨
- `com/dwrite.rs`にTextLayout生成APIを追加（拡張）
- `ecs/widget/text/`に新規モジュール作成（新規）
- 既存のdraw_rectanglesパターンを踏襲

---

## 1. Current State Investigation

### 1.1 Existing Assets

#### DirectWrite COM Wrapper (`com/dwrite.rs`)
**既存機能**:
- ✅ `dwrite_create_factory()`: IDWriteFactory2作成
- ✅ `DWriteFactoryExt::create_text_format()`: IDWriteTextFormat作成
- ✅ `DWriteTextFormatExt`: SetTextAlignment/SetParagraphAlignment

**特徴**:
- IDWriteFactory2を使用（要件はIDWriteFactory7だが互換性あり）
- トレイト拡張パターンで型安全なAPIを提供
- windows-rsのPCWSTR型を適切に扱う

**ギャップ**:
- ❌ CreateTextLayout APIが未実装
- ❌ IDWriteFactory7への明示的アップグレードなし（互換性あるが要件と差異）

#### GraphicsCore (`ecs/graphics/core.rs`)
**統合状態**:
- ✅ `dwrite_factory: IDWriteFactory2`をGraphicsCoreInnerに保持
- ✅ `GraphicsCore::new()`で自動初期化
- ✅ `dwrite_factory()`メソッドでスレッドセーフなアクセス提供
- ✅ ResourceとしてWorldに登録済み

**アーキテクチャ**:
```rust
pub struct GraphicsCore {
    inner: Option<GraphicsCoreInner>,  // 無効化可能
}
struct GraphicsCoreInner {
    d2d_factory: ID2D1Factory,
    d2d: ID2D1Device,
    dwrite_factory: IDWriteFactory2,  // ← 既存
    dcomp: IDCompositionDevice3,
    // ...
}
```

#### Rectangle Widget Pattern (`ecs/widget/shapes/rectangle.rs`)
**実装パターン**:
1. **コンポーネント定義**: `Rectangle` (x, y, width, height, color)
2. **on_remove hook**: GraphicsCommandListをクリア
3. **draw_rectanglesシステム**:
   - クエリ: `Changed<Rectangle>` または `Without<GraphicsCommandList>`
   - GraphicsCoreから描画リソース取得
   - DeviceContext → CommandList生成
   - BeginDraw → 描画命令 → EndDraw → Close
   - GraphicsCommandListコンポーネントを挿入

**キーポイント**:
- 変更検知で効率的な再描画
- CommandListパターンでGPU描画を最適化
- WindowGraphicsとの連携（有効性チェック）

#### Drawing Pipeline (`ecs/world.rs`)
**システム実行順序**:
```rust
PostLayout:
  - init_graphics_core
  - init_window_graphics
  - init_window_visual
  - init_window_surface

Update:
  - invalidate_dependent_components
  - (ユーザーシステム)

Draw:
  - cleanup_graphics_needs_init
  - draw_rectangles  // ← ここにdraw_labelsを追加

RenderSurface:
  - render_surface   // CommandListをSurfaceに描画

CommitComposition:
  - commit_composition
```

**統合ポイント**: `Draw`スケジュールに`draw_labels`を追加するだけ

#### Direct2D Device Context
**アクセスパターン**:
- `WindowGraphics::device_context()` → `Option<&ID2D1DeviceContext>`
- `render_surface`システムがSurface::begin_draw()でDeviceContextを取得
- BeginDraw/EndDrawペアで描画

### 1.2 Conventions and Patterns

#### Naming Conventions
- **Files**: `snake_case.rs`
- **Modules**: `snake_case`
- **Components**: `PascalCase`
- **Systems**: `verb_noun` (e.g., `draw_rectangles`, `render_surface`)

#### Directory Structure
```
crates/wintf/src/
├── com/
│   ├── dwrite.rs        # DirectWrite COM wrapper
│   └── d2d/
│       └── mod.rs       # Direct2D extensions
└── ecs/
    ├── graphics/        # Core graphics systems
    └── widget/
        ├── shapes/      # Rectangle など
        └── text/        # ← 新規作成予定
            ├── mod.rs
            ├── label.rs
            └── draw_labels.rs
```

#### Component Lifecycle Pattern
1. **Component定義**: `#[derive(Component)]`
2. **on_remove hook**: 依存コンポーネントのクリーンアップ
3. **Changed検知**: `Changed<T>`でシステムトリガー
4. **キャッシング**: 高コストなCOMオブジェクトはComponentに格納

#### Error Handling
- COM API呼び出し: `windows::core::Result<T>`
- システム内エラー: `eprintln!`でログ、処理スキップ
- GraphicsCore無効時: 早期リターン

### 1.3 Integration Surfaces

#### ID2D1DeviceContext Integration
**DrawTextLayout呼び出しポイント**:
- `draw_labels`システム内でCommandList作成時
- `dc.DrawTextLayout(origin, text_layout, brush)`

**ブラシ管理**:
- Rectangleと同様に`create_solid_color_brush()`でブラシ作成
- 描画後は自動的に解放（RAII）

#### GraphicsCommandList Component
**既存実装**:
```rust
#[derive(Component)]
pub struct GraphicsCommandList {
    inner: Option<ID2D1CommandList>,
}
```

**使用方法**:
- draw_labelsで生成したCommandListをinsert
- render_surfaceが自動的にSurfaceに描画

---

## 2. Requirements Feasibility Analysis

### 2.1 Technical Needs by Requirement

#### Requirement 1: DirectWrite Factory統合
**現状**: ✅ **完全に実装済み**
- IDWriteFactory2がGraphicsCoreに統合済み
- スレッドセーフなアクセス提供済み
- エラーハンドリング実装済み

**ギャップ**: なし（IDWriteFactory7は後方互換性あり）

#### Requirement 2: テキストフォーマット作成
**現状**: ✅ **APIは実装済み**
- `DWriteFactoryExt::create_text_format()`実装済み
- フォントファミリー、サイズ、ウェイト、スタイル指定可能

**ギャップ**: 
- フォントフォールバック処理の明示的実装なし（DirectWriteが自動処理）

**実装要否**: システム側で追加実装不要、DirectWriteのデフォルト動作で要件満たす

#### Requirement 3: テキストレイアウト生成
**現状**: ❌ **未実装**

**必要なAPI**:
```rust
pub trait DWriteFactoryExt {
    fn create_text_layout<P0>(
        &self,
        text: P0,
        text_format: &IDWriteTextFormat,
        max_width: f32,
        max_height: f32,
    ) -> Result<IDWriteTextLayout>
    where
        P0: Param<PCWSTR>;
}
```

**実装場所**: `com/dwrite.rs`に追加

**複雑度**: 低（既存のcreate_text_formatと同パターン）

#### Requirement 4 & 5: Label & TextLayoutコンポーネント
**現状**: ❌ **未実装**

**必要なコンポーネント**:
```rust
// ecs/widget/text/label.rs
#[derive(Component, Clone)]
pub struct Label {
    pub text: String,
    pub font_family: String,
    pub font_size: f32,
    pub color: D2D1_COLOR_F,
    pub x: f32,
    pub y: f32,
}

// ecs/widget/text/label.rs
#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct TextLayout {
    inner: Option<IDWriteTextLayout>,
}
```

**パターン**: Rectangleコンポーネントと同構造

#### Requirement 6: draw_labelsシステム
**現状**: ❌ **未実装**

**必要なシステム**:
```rust
pub fn draw_labels(
    mut commands: Commands,
    query: Query<(Entity, &Label, &WindowGraphics), Or<(
        Changed<Label>,
        Without<GraphicsCommandList>,
    )>>,
    graphics_core: Option<Res<GraphicsCore>>,
) {
    // draw_rectanglesと同パターン
}
```

**統合**: `world.rs`のDrawスケジュールに追加

#### Requirement 7: DrawTextLayout呼び出し
**現状**: ❌ **未実装**

**必要なAPI**:
```rust
// com/d2d/mod.rs または com/d2d/device_context.rs
pub trait D2D1DeviceContextExt {
    fn draw_text_layout(
        &self,
        origin: D2D_POINT_2F,
        text_layout: &IDWriteTextLayout,
        default_fill_brush: &ID2D1Brush,
        options: D2D1_DRAW_TEXT_OPTIONS,
    );
}
```

**複雑度**: 低（unsafeラッパーのみ）

#### Requirement 8-11: 複数Label、パフォーマンス、エラーハンドリング、サンプル
**現状**: フレームワーク機能で対応可能
- 複数Label: ECSクエリが自動処理
- パフォーマンス: Changed検知とCommandListキャッシングで対応
- エラーハンドリング: 既存パターン踏襲
- サンプル: `simple_window.rs`を拡張、または`simple_window.rs`をベースにした新規サンプル作成

### 2.2 Gaps and Constraints

#### Missing Capabilities

| 機能 | 状態 | 優先度 | 実装箇所 |
|------|------|--------|---------|
| CreateTextLayout API | ❌ 未実装 | High | `com/dwrite.rs` |
| DrawTextLayout API | ❌ 未実装 | High | `com/d2d/mod.rs` |
| Labelコンポーネント | ❌ 未実装 | High | `ecs/widget/text/label.rs` |
| TextLayoutコンポーネント | ❌ 未実装 | High | `ecs/widget/text/label.rs` |
| draw_labelsシステム | ❌ 未実装 | High | `ecs/widget/text/draw_labels.rs` |
| label_demoサンプル | ❌ 未実装 | Medium | `examples/`（simple_window.rs拡張または新規） |

#### Constraints from Existing Architecture

**制約1: GraphicsCoreはIDWriteFactory2を使用**
- 要件: IDWriteFactory7
- 現状: IDWriteFactory2
- 影響: 後方互換性あり、機能的問題なし
- 対応: 将来的なアップグレードパスとして文書化

**制約2: WindowGraphicsとの依存関係**
- Labelエンティティは`WindowGraphics`コンポーネント必須
- 既存パターン通り、システムで有効性チェック

**制約3: Drawスケジュールのタイミング**
- draw_labelsはrender_surfaceの前に実行必須
- 既存のスケジュール順序に追加するだけで対応可能

#### Research Needed

**なし**: 全ての技術要素は既存実装で確立済み

---

## 3. Implementation Approach Options

### Option A: Extend Existing Components ❌ 不適切

**理由**: テキスト描画はshapesとは独立した関心事

**検討内容**:
- `ecs/widget/shapes/`にtext.rsを追加？
  - ❌ shapesは幾何図形の責務、テキストは異なるドメイン
- `com/dwrite.rs`にDirect2D拡張を追加？
  - ❌ DrawTextLayoutはDirect2D API、レイヤー分離を維持すべき

**結論**: 新規モジュール作成が適切

---

### Option B: Create New Components ⚠️ 部分的に適切

**新規作成対象**:
1. `ecs/widget/text/` モジュール
   - `label.rs`: Label/TextLayoutコンポーネント
   - `draw_labels.rs`: draw_labelsシステム
   - `mod.rs`: モジュールエクスポート

**Rationale**:
- テキスト描画は独立した機能領域
- 将来の拡張（縦書き、richtextなど）に対応しやすい
- 既存のshapesモジュールと並列な構造

**Integration Points**:
- `ecs/widget/mod.rs`に`pub mod text;`追加
- `ecs/world.rs`のDrawスケジュールに`draw_labels`追加

**Responsibility Boundaries**:
- `text/label.rs`: Labelデータとライフサイクル
- `text/draw_labels.rs`: 描画ロジックとCommandList生成

**Trade-offs**:
- ✅ 明確な関心の分離
- ✅ テストとメンテナンスが容易
- ❌ COM API拡張が含まれていない

---

### Option C: Hybrid Approach ✅ **推奨**

**組み合わせ戦略**:

#### Phase 1: COM API拡張（既存ファイル拡張）
**対象**: `com/dwrite.rs`, `com/d2d/mod.rs`

**追加API**:
```rust
// com/dwrite.rs
impl DWriteFactoryExt for IDWriteFactory2 {
    fn create_text_layout<P0>(
        &self,
        text: P0,
        text_format: &IDWriteTextFormat,
        max_width: f32,
        max_height: f32,
    ) -> Result<IDWriteTextLayout> { /* ... */ }
}

// com/d2d/mod.rs
pub trait D2D1DeviceContextExt {
    fn draw_text_layout(/* ... */);
}
impl D2D1DeviceContextExt for ID2D1DeviceContext { /* ... */ }
```

**理由**: 既存のCOMラッパーレイヤーを拡張、一貫性維持

#### Phase 2: 新規モジュール作成（ECSレイヤー）
**対象**: `ecs/widget/text/`

**ファイル構成**:
```
ecs/widget/text/
├── mod.rs           # pub use label::*; pub use draw_labels::*;
├── label.rs         # Label/TextLayoutコンポーネント定義
└── draw_labels.rs   # draw_labelsシステム実装
```

**理由**: テキスト描画の独立した責務、将来の拡張性

#### Phase 3: システム統合（既存ファイル拡張）
**対象**: `ecs/world.rs`, `ecs/widget/mod.rs`

**変更内容**:
```rust
// ecs/widget/mod.rs
pub mod shapes;
pub mod text;  // ← 追加

// ecs/world.rs (Drawスケジュール)
schedules.add_systems(
    Draw,
    (
        cleanup_graphics_needs_init,
        draw_rectangles,
        draw_labels,  // ← 追加
    ),
);
```

**理由**: 最小限の変更で既存システムに統合

#### Phase 4: サンプル作成（新規ファイル）
**対象**: `examples/label_demo.rs`（新規）または`examples/simple_window.rs`（拡張）

**内容**: simple_window.rsをベースにLabelウィジット使用例を実装

**Trade-offs**:
- ✅ レイヤー分離を維持（COM/ECS/統合）
- ✅ 既存パターンとの一貫性
- ✅ 段階的な実装とテストが可能
- ✅ 最小限の既存コード変更
- ⚠️ 複数ファイルの調整が必要（ただし明確な責務分離により管理容易）

---

## 4. Implementation Complexity & Risk

### Effort Estimation: **M (Medium, 3-7 days)**

**内訳**:
- COM API拡張: 0.5日（既存パターン踏襲）
- Labelコンポーネント: 0.5日（Rectangleパターン踏襲）
- draw_labelsシステム: 1.5日（draw_rectanglesベース、TextLayout生成ロジック追加）
- 統合とテスト: 1日（既存システムへの統合、動作確認）
- サンプル作成: 0.5日（simple_window.rs拡張または新規作成）
- ドキュメント: 0.5日

**合計**: 4.5日（余裕を見て5-6日）

**根拠**:
- 既存のRectangle実装が成熟したテンプレート提供
- DirectWrite APIは既に部分的に統合済み
- 新規パターンは不要、既存アーキテクチャに適合

### Risk Assessment: **Low**

**技術リスク**: **Low**
- DirectWrite/Direct2D APIは十分に文書化済み
- 既存のGraphicsCore統合パターンが確立
- COM APIラッパーパターンが一貫して適用済み
- パフォーマンス最適化パターン（CommandListキャッシング）が確立

**統合リスク**: **Low**
- 既存システムへの影響が最小限
- Drawスケジュールへの追加のみ
- 破壊的変更なし

**パフォーマンスリスク**: **Low**
- Changed検知で不要な再描画を回避
- CommandListキャッシングでGPU効率最大化
- 既存のRectangle実装で60fps安定動作実績（Vsync同期環境）

**未知の領域**: なし
- 全ての技術要素が既知で実装済み

---

## 5. Requirement-to-Asset Mapping

| 要件 | 必要なアセット | 状態 | ギャップ | 対応方針 |
|------|--------------|------|---------|---------|
| Req 1: DirectWrite Factory | GraphicsCore, dwrite_create_factory | ✅ 実装済み | なし | そのまま使用 |
| Req 2: TextFormat作成 | DWriteFactoryExt::create_text_format | ✅ 実装済み | なし | そのまま使用 |
| Req 3: TextLayout生成 | DWriteFactoryExt::create_text_layout | ❌ Missing | API追加 | com/dwrite.rs拡張 |
| Req 4: Labelコンポーネント | Label struct | ❌ Missing | 新規作成 | ecs/widget/text/label.rs |
| Req 5: TextLayoutコンポーネント | TextLayout struct | ❌ Missing | 新規作成 | ecs/widget/text/label.rs |
| Req 6: draw_labelsシステム | draw_labels fn | ❌ Missing | 新規作成 | ecs/widget/text/draw_labels.rs |
| Req 7: DrawTextLayout | D2D1DeviceContextExt::draw_text_layout | ❌ Missing | API追加 | com/d2d/mod.rs拡張 |
| Req 8: 複数Label | ECSクエリ | ✅ 実装済み | なし | 既存機能で対応 |
| Req 9: パフォーマンス | Changed検知, CommandList | ✅ 実装済み | なし | 既存パターン踏襲 |
| Req 10: エラーハンドリング | Result, eprintln | ✅ 実装済み | なし | 既存パターン踏襲 |
| Req 11: サンプル | label_demo.rs または simple_window.rs拡張 | ❌ Missing | 新規作成 | examples/ |

---

## 6. Recommendations for Design Phase

### 6.1 Preferred Approach

**Option C: Hybrid Approach（拡張+新規作成）**

**理由**:
1. **レイヤー分離の維持**: COM/ECS/統合の3層アーキテクチャを維持
2. **既存パターンとの一貫性**: Rectangle実装パターンを踏襲
3. **将来の拡張性**: text/モジュールで縦書き等の拡張に対応
4. **最小限の影響**: 既存コードへの変更は統合ポイントのみ

### 6.1.1 縦書き対応を考慮した設計原則

**重要**: 本Phase 4の実装は、Phase 7での縦書き対応を前提とした設計とする。

1. **命名規則**:
   - ○ `draw_labels`（汎用的）
   - × `draw_horizontal_labels`（方向依存）
   - ○ `Label`（汎用的）
   - × `HorizontalLabel`（方向依存）

2. **コンポーネント設計**:
   - 現時点では方向指定フィールドは含めない
   - Phase 7で`writing_mode: WritingMode`フィールド追加を想定
   - Default値は横書きとすることで後方互換性維持

3. **API設計**:
   - CreateTextLayoutはREADING_DIRECTIONパラメータをサポート
   - DrawTextLayoutは方向に依存しないAPI設計
   - DirectWriteのIDWriteTextLayoutは縦書きをネイティブサポート

4. **システム設計**:
   - `draw_labels`システムは将来的に`writing_mode`で分岐処理
   - TextLayout生成ロジックは方向指定に対応可能な構造

### 6.2 Key Decisions for Design Phase

#### Decision 1: IDWriteFactory2 vs IDWriteFactory7
**推奨**: 現状のIDWriteFactory2を継続使用
**理由**:
- 横書きテキストに必要な機能は全てIDWriteFactory2で利用可能
- IDWriteFactory7は後方互換性あり
- 将来的なアップグレードパスを文書化すれば十分

**実装ノート**:
```rust
// 現在
let dwrite_factory = dwrite_create_factory(DWRITE_FACTORY_TYPE_SHARED)?;
// → IDWriteFactory2を返す

// 将来（必要に応じて）
let dwrite_factory: IDWriteFactory7 = dwrite_factory.cast()?;
```

#### Decision 2: TextLayoutのキャッシング戦略
**推奨**: Componentとして永続化
**理由**:
- IDWriteTextLayoutの生成は比較的高コスト
- Changed<Label>検知で再生成タイミング制御
- Rectangleパターンと一貫性

**実装ノート**:
```rust
#[derive(Component)]
#[component(on_remove = on_text_layout_remove)]
pub struct TextLayout {
    inner: Option<IDWriteTextLayout>,
}
```

#### Decision 3: Labelコンポーネントのフィールド
**推奨**: 要件通りの最小フィールドセット（縦書き拡張の余地あり）
**理由**:
- 複雑さを避け、Phase 7（縦書き）での拡張に備える
- 既存のRectangle（5フィールド）と同レベルの複雑さ
- 将来的に`writing_mode`または`direction`フィールド追加が可能

**実装ノート**:
```rust
// Phase 4: 横書きのみ
pub struct Label {
    pub text: String,
    pub font_family: String,  // "メイリオ"
    pub font_size: f32,       // 16.0
    pub color: D2D1_COLOR_F,
    pub x: f32,
    pub y: f32,
    // Phase 7で追加予定:
    // pub writing_mode: WritingMode,  // Horizontal | Vertical
}
```

#### Decision 4: デフォルト値
**推奨**: Defaultトレイト実装
```rust
impl Default for Label {
    fn default() -> Self {
        Self {
            text: String::new(),
            font_family: "メイリオ".to_string(),
            font_size: 16.0,
            color: colors::BLACK,
            x: 0.0,
            y: 0.0,
        }
    }
}
```

### 6.3 Research Items

**なし**: 全ての技術要素は既知で文書化済み

### 6.4 Implementation Sequence

**推奨順序**:
1. COM API拡張（`com/dwrite.rs`, `com/d2d/mod.rs`）
   - 独立してテスト可能
2. Labelコンポーネント定義（`ecs/widget/text/label.rs`）
   - 依存なしでコンパイル確認
3. draw_labelsシステム（`ecs/widget/text/draw_labels.rs`）
   - ユニットテスト可能
4. システム統合（`ecs/world.rs`, `ecs/widget/mod.rs`）
   - 既存システムとの動作確認
5. サンプル作成（`examples/simple_window.rs`拡張または新規サンプル）
   - エンドツーエンドテスト

---

## 7. Conclusion

DirectWriteを使用した横書きテキストレンダリングの実装は、既存のwintfアーキテクチャに自然に適合する。GraphicsCoreへのDirectWrite統合、Rectangle描画パターン、ECSシステムスケジュールが全て確立されており、新機能追加は既存パターンの踏襲で実現可能。

**主要な強み**:
- 成熟したRectangle実装がテンプレート提供
- DirectWrite基盤は既に統合済み
- レイヤー分離されたアーキテクチャで影響範囲が明確
- Vsync同期により安定した60fpsパフォーマンス

**実装容易性**: Medium（3-7日）、Risk: Low

**次のステップ**: 設計フェーズでAPI詳細とコンポーネント構造を定義

---

_Gap Analysis completed on 2025-11-17_
