# Technical Design: Visual Tree Synchronization

## 概要

### 1. 目的
ECSウィジェットツリー（`ChildOf`/`Children`）とDirectCompositionビジュアルツリー（`IDCompositionVisual`階層）を1:1で同期するシステムを設計する。

### 2. スコープ
- **対象**: R1〜R8、R4a、R5aの要件（12要件、58+受け入れ条件）
- **種別**: 既存システムの拡張（Extension）
- **影響範囲**: `com/dcomp.rs`、`ecs/graphics/*`、`ecs/layout/*`

### 3. アプローチ
ギャップ分析に基づくハイブリッドアプローチ（Option C）を採用:
- Phase 1: APIレイヤー + 軽量拡張（R1, R3, R4, R6）
- Phase 2: 新規システム（R4a, R5, R7, R8）
- Phase 3: レンダーパイプライン変更（R5a）

---

## アーキテクチャ設計

### 1. コンポーネント図

```text
┌─────────────────────────────────────────────────────────────────┐
│                         ECS World                                │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │   Entity A   │    │   Entity B   │    │   Entity C   │       │
│  │  - Visual    │    │  - Visual    │    │  - Visual    │       │
│  │  - ChildOf(R)│    │  - ChildOf(A)│    │  - ChildOf(A)│       │
│  │  - Visual    │    │  - Arrangement│   │  - Arrangement│      │
│  │    Graphics  │    │  - Visual    │    │  - Visual    │       │
│  │  - Surface   │    │    Graphics  │    │    Graphics  │       │
│  │    Graphics  │    │              │    │              │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
│         │                   ▲                   ▲                │
│         │                   │                   │                │
│         └───────────────────┴───────────────────┘                │
│                      Children relation                           │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼ 同期
┌─────────────────────────────────────────────────────────────────┐
│                  DirectComposition Visual Tree                   │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐                                               │
│  │ Root Visual  │ (WindowGraphics.root_visual)                  │
│  │    (A)       │                                               │
│  └──────┬───────┘                                               │
│         │                                                        │
│    ┌────┴────┐                                                  │
│    ▼         ▼                                                  │
│ ┌──────┐ ┌──────┐                                               │
│ │  B   │ │  C   │  Z-order: B(背面) → C(前面)                   │
│ └──────┘ └──────┘                                               │
└─────────────────────────────────────────────────────────────────┘
```

### 2. システムフロー

```text
┌─────────────────────────────────────────────────────────────────┐
│                     Schedule Pipeline                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  PreLayout ──────────► Layout ──────────► PostLayout ──────►    │
│      │                                        │                  │
│      ▼                                        ▼                  │
│  text_layout_measurement      create_visual_graphics_system     │
│  (R4a: テキスト測定)           (R2: VG作成)                      │
│                               deferred_surface_creation         │
│                               (R5: Surface作成)                  │
│                                                                  │
│  ──► UISetup ──► Draw ──► RenderSurface ──────────────────►    │
│      (Win32のみ)        │                                        │
│                         ▼                                        │
│                   render_to_surface                              │
│                   (R5a: 描画)                                    │
│                                                                  │
│  ──► Composition ──────────────────────► CommitComposition      │
│          │                                                       │
│          ▼                                                       │
│    visual_hierarchy_sync (R3: 階層同期)                         │
│    visual_zorder_sync (R6: Z-order)                             │
│    visual_transform_sync (R7, R8: 変換同期)                     │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

スケジュール責務:
  PostLayout   = レイアウト結果を利用したリソース作成（Visual/Surface）
  Composition  = Visual操作（階層同期、Transform、プロパティ）
  RenderSurface = Surfaceへの描画（BeginDraw/EndDraw）
```

---

## 詳細設計

### 1. API拡張 (R1)

#### 1.1 DCompositionVisualExtトレイトの拡張

**ファイル**: `crates/wintf/src/com/dcomp.rs`

```rust
pub trait DCompositionVisualExt {
    // 既存メソッド
    fn add_visual<P0, P1>(&self, visual: P0, insertabove: bool, referencevisual: P1) -> Result<()>
    where
        P0: Param<IDCompositionVisual>,
        P1: Param<IDCompositionVisual>;

    // 新規追加 (R1)
    fn remove_visual<P0>(&self, visual: P0) -> Result<()>
    where
        P0: Param<IDCompositionVisual>;
}

impl DCompositionVisualExt for IDCompositionVisual3 {
    // 既存実装...

    fn remove_visual<P0>(&self, visual: P0) -> Result<()>
    where
        P0: Param<IDCompositionVisual>,
    {
        unsafe { self.RemoveVisual(visual) }
    }
}
```

### 2. Visual階層同期システム (R2, R3)

#### 2.1 VisualGraphicsコンポーネント設計

**ファイル**: `crates/wintf/src/ecs/graphics/components.rs`

```rust
/// VisualGraphics コンポーネント（親キャッシュ方式）
#[derive(Component)]
#[component(on_remove = on_visual_graphics_remove)]
pub struct VisualGraphics {
    pub visual: IDCompositionVisual3,
    /// 親Visual参照（RemoveVisual用にキャッシュ）
    /// 階層同期時にAddVisualと同時に設定される
    pub parent_visual: Option<IDCompositionVisual3>,
}

impl VisualGraphics {
    pub fn new(visual: IDCompositionVisual3) -> Self {
        Self {
            visual,
            parent_visual: None,
        }
    }
}

fn on_visual_graphics_remove(
    mut world: DeferredWorld,
    hook: HookContext,
) {
    // 親Visualから自分を削除
    // エラーは無視（親が先に削除されている場合など）
    if let Some(vg) = world.get::<VisualGraphics>(hook.entity) {
        if let Some(ref parent) = vg.parent_visual {
            let _ = parent.remove_visual(&vg.visual);  // エラー無視
        }
    }
}
```

#### 2.2 Visualコンポーネントフック

```rust
/// Visual コンポーネント (R2)
#[derive(Component)]
#[component(on_add = on_visual_add)]
pub struct Visual {
    pub size: SizeU,
    pub opacity: f32,
    pub is_visible: bool,
}

fn on_visual_add(
    mut world: DeferredWorld,
    hook: HookContext,
) {
    // VisualGraphicsの作成をトリガー
    // 注意: SurfaceGraphicsはここでは作成しない (R2 AC#5, R5で遅延作成)
    world.commands().entity(hook.entity).insert(VisualNeedsGraphics);
}
```

#### 2.3 階層同期システム

**ファイル**: `crates/wintf/src/ecs/graphics/hierarchy_sync.rs`（新規）

```rust
/// ChildOf変更を検出してVisual階層を同期するシステム (R3)
pub fn visual_hierarchy_sync_system(
    mut child_vg_query: Query<(Entity, &ChildOf, &mut VisualGraphics), Changed<ChildOf>>,
    parent_query: Query<&VisualGraphics>,
    removed_parents: RemovedComponents<ChildOf>,
    orphan_query: Query<&mut VisualGraphics>,
    window_graphics: Query<&WindowGraphics>,
) {
    // 1. 親が変更されたエンティティの処理
    for (entity, child_of, mut child_vg) in child_vg_query.iter_mut() {
        // 旧親からの削除は parent_visual キャッシュで行う（on_removeで処理済み or 初回追加）
        if let Some(ref old_parent) = child_vg.parent_visual {
            let _ = old_parent.remove_visual(&child_vg.visual);  // エラー無視
        }

        // 新しい親のVisualに追加し、parent_visualをキャッシュ
        if let Ok(parent_vg) = parent_query.get(child_of.0) {
            let _ = parent_vg.visual.add_visual(&child_vg.visual, false, None);  // エラー無視
            child_vg.parent_visual = Some(parent_vg.visual.clone());  // 親キャッシュ更新
        }
    }

    // 2. ChildOfが削除されたエンティティの処理（ルートになった場合）
    for entity in removed_parents.read() {
        if let Ok(mut vg) = orphan_query.get_mut(entity) {
            // 旧親から削除
            if let Some(ref old_parent) = vg.parent_visual {
                let _ = old_parent.remove_visual(&vg.visual);  // エラー無視
            }
            vg.parent_visual = None;

            // WindowGraphicsのroot_visualに接続（必要に応じて）
            // (実装詳細は省略)
        }
    }
}

/// Z-order同期システム (R6)
/// Children順序変更時に親キャッシュも更新
pub fn visual_zorder_sync_system(
    children_query: Query<(Entity, &Children), Changed<Children>>,
    mut child_vg_query: Query<&mut VisualGraphics>,
    parent_vg_query: Query<&VisualGraphics>,
) {
    for (parent_entity, children) in children_query.iter() {
        // 親のVisualGraphicsを取得
        let parent_vg = match parent_vg_query.get(parent_entity) {
            Ok(vg) => vg,
            Err(_) => continue,
        };

        // Children内の順序でZ-orderを再構築
        let mut prev_child_visual: Option<IDCompositionVisual3> = None;

        for &child in children.iter() {
            if let Ok(mut child_vg) = child_vg_query.get_mut(child) {
                // insertabove=true, referencevisual=prev で前の兄弟の上に配置
                let _ = parent_vg.visual.add_visual(
                    &child_vg.visual,
                    true,
                    prev_child_visual.as_ref(),
                );  // エラー無視
                // 親キャッシュを更新
                child_vg.parent_visual = Some(parent_vg.visual.clone());
                prev_child_visual = Some(child_vg.visual.clone());
            }
        }
    }
}
```

### 3. テキストレイアウト測定システム (R4, R4a)

#### 3.1 PreLayoutスケジュールでの実行

**ファイル**: `crates/wintf/src/ecs/widget/text/systems.rs`（新規）

```rust
/// テキストレイアウト測定システム (R4a)
/// PreLayoutスケジュールで実行
pub fn text_layout_measurement_system(
    mut commands: Commands,
    text_factory: Res<TextFactory>,
    labels: Query<
        (Entity, &Label, &BoxStyle, Option<&TextLayoutMetrics>),
        Or<(Changed<Label>, Changed<BoxStyle>)>,
    >,
) {
    for (entity, label, box_style, existing_metrics) in labels.iter() {
        // AC#8: 既存のTextLayoutMetricsがあり、変更がない場合はスキップ
        if existing_metrics.is_some() && !label.is_changed() && !box_style.is_changed() {
            continue;
        }

        // テキストレイアウトの作成と測定
        let text_layout = text_factory.create_text_layout(
            &label.text,
            &label.format,
            box_style.max_width.unwrap_or(f32::MAX),
            box_style.max_height.unwrap_or(f32::MAX),
        );

        if let Ok(layout) = text_layout {
            let metrics = layout.get_metrics();
            if let Ok(m) = metrics {
                commands.entity(entity).insert(TextLayoutMetrics {
                    width: m.width,
                    height: m.height,
                    line_count: m.lineCount,
                    layout: Some(layout),
                });
            }
        }
    }
}
```

#### 3.2 描画システムからの分離

**ファイル**: `crates/wintf/src/ecs/widget/text/label.rs` (修正)

```rust
/// ラベル描画システム (R4)
/// Drawスケジュールで実行
pub fn draw_labels(
    // R4a AC#7: 測定ロジックは text_layout_measurement_system に分離
    labels: Query<(Entity, &Label, &TextLayoutMetrics, &GlobalArrangement)>,
    // ... 描画リソース
) {
    for (entity, label, metrics, arrangement) in labels.iter() {
        // 事前計算されたTextLayoutMetricsを使用して描画のみを行う
        if let Some(layout) = &metrics.layout {
            // 描画処理
        }
    }
}
```

### 4. 遅延Surface作成システム (R5)

#### 4.1 Surface作成条件

**ファイル**: `crates/wintf/src/ecs/graphics/surface_manager.rs` (新規)

```rust
/// Surface作成マーカーコンポーネント
#[derive(Component)]
pub struct SurfaceNeedsCreation;

/// 遅延Surface作成システム (R5)
/// PostLayoutスケジュールで実行
pub fn deferred_surface_creation_system(
    mut commands: Commands,
    graphics: Res<GraphicsCore>,
    // Surfaceが必要な条件を満たすエンティティ
    needs_surface: Query<
        (Entity, &Visual, &VisualGraphics, &GlobalArrangement),
        (
            With<SurfaceNeedsCreation>,
            Or<(With<Label>, With<Rectangle>)>,  // 描画可能コンテンツを持つ
        ),
    >,
) {
    let dcomp = match graphics.dcomp() {
        Some(d) => d,
        None => return,
    };

    for (entity, visual, vg, arrangement) in needs_surface.iter() {
        // R8: GlobalArrangementからスケールを抽出
        let scale_x = arrangement.transform.M11;
        let scale_y = arrangement.transform.M22;

        // Surface実サイズ = Visual論理サイズ × スケール
        let surface_width = (visual.size.X as f32 * scale_x).ceil() as u32;
        let surface_height = (visual.size.Y as f32 * scale_y).ceil() as u32;

        // 最小サイズ保証
        let width = surface_width.max(1);
        let height = surface_height.max(1);

        if let Ok(surface) = dcomp.create_surface(
            width, height,
            DXGI_FORMAT_B8G8R8A8_UNORM,
            DXGI_ALPHA_MODE_PREMULTIPLIED,
        ) {
            unsafe { let _ = vg.visual.SetContent(&surface); }
            commands.entity(entity)
                .insert(SurfaceGraphics::new(surface, (width, height)))
                .remove::<SurfaceNeedsCreation>();
        }
    }
}
```

### 5. 変換同期システム (R7, R8)

#### 5.1 Arrangement変換の適用

**ファイル**: `crates/wintf/src/ecs/graphics/transform_sync.rs` (新規)

```rust
/// Visual変換同期システム (R7)
/// Compositionスケジュールで実行
pub fn visual_transform_sync_system(
    changed_arrangements: Query<
        (&Arrangement, &VisualGraphics),
        Changed<Arrangement>,
    >,
) {
    for (arrangement, vg) in changed_arrangements.iter() {
        // Offset → Visual.SetOffsetX/Y
        let _ = vg.visual.set_offset_x(arrangement.offset.x);
        let _ = vg.visual.set_offset_y(arrangement.offset.y);

        // Scale → Visual.SetTransform (必要に応じて)
        // 注意: スケールはSurfaceサイズで吸収するため、
        // Visual変換にはoffsetのみを適用
    }
}

/// Surfaceリサイズシステム (R8)
/// スケール変更時にSurfaceサイズを更新
pub fn surface_resize_on_scale_change_system(
    mut commands: Commands,
    graphics: Res<GraphicsCore>,
    changed_scales: Query<
        (Entity, &Visual, &GlobalArrangement, &SurfaceGraphics),
        Changed<GlobalArrangement>,
    >,
) {
    for (entity, visual, arrangement, surface) in changed_scales.iter() {
        let scale_x = arrangement.transform.M11;
        let scale_y = arrangement.transform.M22;

        let new_width = (visual.size.X as f32 * scale_x).ceil() as u32;
        let new_height = (visual.size.Y as f32 * scale_y).ceil() as u32;

        // 現在のサイズと異なる場合のみリサイズ
        if (new_width, new_height) != surface.size {
            // Surfaceの再作成をトリガー
            commands.entity(entity).insert(SurfaceNeedsResize {
                new_size: (new_width, new_height),
            });
        }
    }
}
```

### 6. CommandList描画システム (R5a)

#### 6.1 遅延描画パターン

**ファイル**: `crates/wintf/src/ecs/graphics/systems.rs` (修正)

```rust
/// Surface描画システム (R5a)
/// RenderSurfaceスケジュールで実行
pub fn render_to_surface_system(
    graphics: Res<GraphicsCore>,
    dirty_surfaces: Query<
        (Entity, &SurfaceGraphics, &CommandListReady),
        Changed<CommandListReady>,
    >,
) {
    let d2d = match graphics.d2d_device() {
        Some(d) => d,
        None => return,
    };

    for (entity, surface, cmd_list) in dirty_surfaces.iter() {
        // BeginDraw
        let (dc, offset) = match surface.begin_draw() {
            Ok(result) => result,
            Err(_) => continue,
        };

        // Clear
        unsafe { dc.Clear(None); }

        // CommandListの再生
        unsafe { dc.DrawImage(&cmd_list.0, None, None, Default::default()); }

        // EndDraw
        let _ = surface.end_draw();
    }
}
```

---

## スケジュール統合

### スケジュール設計原則

| スケジュール | スレッド | 責務 |
|-------------|---------|------|
| `UISetup` | シングル（UIスレッド固定） | Win32 API（CreateWindowEx等）のみ |
| `PostLayout` | マルチ可 | レイアウト結果を利用したリソース作成（Visual/Surface作成） |
| `Composition` | マルチ可 | Visual操作（階層同期、Transform設定、プロパティ変更） |
| `RenderSurface` | マルチ可 | Surfaceへの描画（BeginDraw/EndDraw）、`Changed<GraphicsCommandList>`で変更検知 |
| ~~`PreRenderSurface`~~ | - | **廃止**（階層的描画廃止により不要） |

### システム登録

```rust
// world.rs への追加

// PreLayout: テキスト測定（レイアウト計算に必要なサイズ情報を取得）
app.add_systems(PreLayout, (
    text_layout_measurement_system,  // R4a: テキスト測定
).chain());

// PostLayout: レイアウト結果を利用したリソース作成
app.add_systems(PostLayout, (
    create_visual_graphics_system,   // R2: Visual→VisualGraphics作成
    deferred_surface_creation_system, // R5: 遅延Surface作成（レイアウトサイズ確定後）
).chain());

// Composition: Visual階層操作・プロパティ設定
// ※VisualGraphics作成（PostLayout）の後に実行される
app.add_systems(Composition, (
    visual_hierarchy_sync_system,    // R3: ChildOf→Visual階層同期（AddVisual/RemoveVisual）
    visual_zorder_sync_system,       // R6: Z-order同期
    visual_transform_sync_system,    // R7: Offset同期（SetOffsetX/Y）
    surface_resize_on_scale_change_system, // R8: スケール変更時リサイズ
).chain());

// RenderSurface: Surfaceへの描画
// Changed<GraphicsCommandList>でフィルタ（自身の変更のみ描画）
app.add_systems(RenderSurface, (
    render_to_surface_system,        // R5a: Surface描画
).chain());
```

---

## 廃止項目

本設計による階層的描画の廃止（各Entityが独自Visualを持つ方式への移行）に伴い、以下を削除する：

| 削除対象 | 種別 | 理由 |
|---------|------|------|
| `PreRenderSurface` スケジュール | Schedule | 使用システムがなくなる |
| `mark_dirty_surfaces` システム | System | 子→親の変更伝播が不要（各Entityが独自Surface） |
| `SurfaceUpdateRequested` マーカー | Component | 上記システム廃止に伴い不要 |

### render_surface の修正

旧アーキテクチャ:
```rust
// SurfaceUpdateRequested を持つEntityのみ描画
Query<..., With<SurfaceUpdateRequested>>
```

新アーキテクチャ:
```rust
// 自身のCommandListが変更されたEntityのみ描画
Query<..., Changed<GraphicsCommandList>>
```

---

## エラー処理戦略

### 1. DirectComposition API エラー
- `Result<()>` を返し、呼び出し元でログ出力
- 部分的失敗を許容（他のVisualの処理は継続）

### 2. リソース作成失敗
- `GraphicsNeedsInit` マーカーで再初期化をトリガー
- 次フレームで再試行

### 3. 階層不整合
- デバッグビルドで `debug_assert!` による検証
- リリースビルドでは自己修復（孤立Visualの検出と再接続）

---

## テスト戦略

### 1. 単体テスト
- `remove_visual` API呼び出しのモック検証
- `Arrangement` → `Matrix3x2` 変換の正確性
- スケール抽出ロジックの境界値テスト

### 2. 統合テスト
- 階層追加/削除シナリオ
- Z-order変更シナリオ
- 動的リサイズシナリオ

### 3. ビジュアル回帰テスト
- 既存の `dcomp_demo.rs` による目視確認
- スクリーンショット比較（将来的に自動化）

---

## 実装フェーズ

### Phase 1: 基盤整備（推定工数: 2日）
1. `remove_visual` API追加 (R1)
2. `visual_hierarchy_sync_system` 実装 (R3)
3. Z-order同期 (R6)
4. 基本的な統合テスト

### Phase 2: 遅延作成（推定工数: 3日）
1. `visual_resource_management_system` 修正（Surface即時作成の削除）
2. `deferred_surface_creation_system` 実装 (R5)
3. `text_layout_measurement_system` 実装 (R4a)
4. スケールベースリサイズ (R8)

### Phase 3: 変換統合（推定工数: 2日）
1. `visual_transform_sync_system` 実装 (R7)
2. CommandList描画への移行 (R5a)
3. 全体統合テスト
4. パフォーマンス検証

---

## リスクと緩和策

| リスク | 影響 | 緩和策 |
|--------|------|--------|
| R5aのCommandList移行で描画品質低下 | 高 | 段階的移行、既存パスをフォールバックとして維持 |
| 階層同期のパフォーマンス問題 | 中 | Changed<>クエリによる差分更新、バッチ処理 |
| Z-order同期のちらつき | 中 | フレーム単位での一括コミット |
| スケール抽出の精度問題 | 低 | 軸平行変換の前提条件をドキュメント化 |
