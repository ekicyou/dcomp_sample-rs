# DirectComposition ビジュアルツリー管理 (bevy_ecs版)

## 描画パイプライン

### bevy_ecsによる描画システム

```rust
use bevy_ecs::prelude::*;

/// フレーム更新時の描画処理
pub fn update_visuals_system(
    mut commands: Commands,
    mut dirty_query: Query<Entity, With<DirtyVisual>>,
    needs_visual_query: Query<(Entity, &TextContent, Option<&Visual>)>,
    dcomp_context: Res<DCompContext>,
) {
    for entity in dirty_query.iter_mut() {
        // 描画が必要か判定
        if needs_visual(entity, &needs_visual_query) {
            // Visualを確保（なければ作成）
            ensure_visual(&mut commands, entity);
            
            // 再描画フラグをマーク
            commands.entity(entity).insert(NeedsRedraw);
        } else {
            // 不要になったVisualを削除
            commands.entity(entity).remove::<Visual>();
        }
        
        commands.entity(entity).remove::<DirtyVisual>();
    }
    
    // DirectCompositionにコミット
    dcomp_context.commit().ok();
}
```

このドキュメントでは、**bevy_ecs**を使用したDirectCompositionビジュアルツリーの管理方法を説明します。
**Visualコンポーネントは描画が必要なEntityのみが持ち、動的に追加されます。**

## 重要な設計原則

1. **論理ツリーとビジュアルツリーの分離**
   - 論理ツリー: すべてのEntity（UI構造、bevy_ecsの階層）
   - ビジュアルツリー: 描画が必要なEntityのみ（DirectComposition）

2. **動的Visualコンポーネント追加**
   - テキスト、画像、背景色などを持つEntityのみがVisualコンポーネントを持つ
   - 純粋なレイアウトノードはVisualコンポーネントを持たない（メモリ効率化）

3. **自動的なツリー管理**
   - Visualは親でVisualコンポーネントを持つEntityに自動接続される
   - 中間のVisualなしEntityは透過的にスキップされる

## bevy_ecsアーキテクチャの利点

- **クエリベースの効率的な更新**: システムは必要なコンポーネントだけをクエリで抽出
- **並列処理**: 独立したシステムは自動的に並列実行可能
- **柔軟なコンポーネント構成**: 実行時にコンポーネントを追加/削除可能
- **変更検知**: `Changed<T>`フィルタで変更されたコンポーネントのみ処理

## ダーティ検出と変更管理戦略

### bevy_ecsの変更検知メカニズム

bevy_ecsは**自動的な変更追跡**を提供し、明示的なダーティフラグは最小限で済みます。

#### 変更追跡の基本原理

```rust
// bevy_ecsは内部的に各コンポーネントの変更を追跡
// - 各コンポーネントは「最後に変更されたティック」を記録
// - システムは「最後に実行されたティック」を記憶
// - Changed<T>は「システム実行後に変更された」コンポーネントのみを返す
```

### 3つの変更検知戦略

#### 1. **Changed<T>フィルタ** - コンポーネント変更の検知

最も基本的で推奨される方法。コンポーネントが変更されたEntityのみを処理。

```rust
use bevy_ecs::prelude::*;

/// テキストが変更されたら自動的にレイアウトを無効化
pub fn text_changed_system(
    mut query: Query<&mut Layout, Changed<TextContent>>,
) {
    for mut layout in query.iter_mut() {
        // Changed<TextContent>なので、テキストが変更されたEntityのみ
        layout.invalidate();
    }
}

/// 背景スタイルが変更されたら再描画をマーク
pub fn style_changed_system(
    mut commands: Commands,
    query: Query<Entity, Changed<ContainerStyle>>,
) {
    for entity in query.iter() {
        // Visualが存在する場合のみ再描画が必要
        commands.entity(entity).insert(NeedsRedraw);
    }
}

/// レイアウトが変更されたらVisualのトランスフォームを更新
pub fn layout_changed_system(
    mut visual_query: Query<(&Layout, &mut Visual), Changed<Layout>>,
) {
    for (layout, mut visual) in visual_query.iter_mut() {
        visual.offset = layout.final_rect.origin;
        visual.size = layout.final_rect.size;
        // ここでvisualを変更すると、次のフレームでChanged<Visual>になる
    }
}
```

**利点**:
- システム実行が必要な時だけスケジュール（自動最適化）
- 明示的なダーティフラグ管理が不要
- コードが宣言的で読みやすい

**注意点**:
- `Changed<T>`は「書き込みアクセス」でも発火する（実際の値変更がなくても）
- 初回はすべてのコンポーネントが"changed"扱い

#### 2. **Added<T>フィルタ** - 新規コンポーネントの検知

コンポーネントが新しく追加されたEntityを検知。

```rust
/// 新しくVisualが追加されたらDirectCompositionツリーに接続
pub fn attach_new_visual_system(
    query: Query<(Entity, &Visual, Option<&Parent>), Added<Visual>>,
    parent_visual_query: Query<&Visual>,
    dcomp_context: Res<DCompContext>,
) {
    for (entity, visual, parent) in query.iter() {
        // Added<Visual>なので、今フレームで追加されたVisualのみ
        
        // 親のVisualを探してツリーに接続
        if let Some(parent) = parent {
            connect_to_parent_visual(visual, parent.get(), &parent_visual_query, &dcomp_context);
        } else {
            // ルートに接続
            unsafe {
                dcomp_context.root_visual.AddVisual(&visual.dcomp_visual, true, None).ok();
            }
        }
    }
}

/// TextContentが追加されたら自動的にVisualも追加
pub fn ensure_visual_for_text_system(
    mut commands: Commands,
    query: Query<Entity, Added<TextContent>>,
    dcomp_context: Res<DCompContext>,
) {
    for entity in query.iter() {
        // テキストが追加されたEntityにはVisualが必要
        let visual = create_visual(&dcomp_context);
        commands.entity(entity).insert(visual);
    }
}
```

**利点**:
- 初期化処理を明確に分離できる
- 1回だけ実行される処理に最適
- Added<T>とChanged<T>を組み合わせて細かい制御が可能

#### 3. **マーカーコンポーネント** - 明示的な状態管理

複雑な更新フローや、複数システム間での調整が必要な場合に使用。

```rust
/// マーカーコンポーネント（データを持たない）
#[derive(Component)]
pub struct NeedsRedraw;

#[derive(Component)]
pub struct NeedsLayoutUpdate;

#[derive(Component)]
pub struct LayoutInvalidated;

/// レイアウトが無効化されたことを明示的にマーク
pub fn invalidate_layout_system(
    mut commands: Commands,
    // 複数の条件でレイアウト無効化が必要
    text_changed: Query<Entity, Changed<TextContent>>,
    size_changed: Query<Entity, Changed<DesiredSize>>,
    children_changed: Query<Entity, Changed<Children>>,
) {
    // テキスト変更
    for entity in text_changed.iter() {
        commands.entity(entity).insert(LayoutInvalidated);
    }
    
    // サイズ変更
    for entity in size_changed.iter() {
        commands.entity(entity).insert(LayoutInvalidated);
    }
    
    // 子要素変更
    for entity in children_changed.iter() {
        commands.entity(entity).insert(LayoutInvalidated);
    }
}

/// レイアウト計算システム（無効化されたものだけ処理）
pub fn compute_layout_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Layout), With<LayoutInvalidated>>,
) {
    for (entity, mut layout) in query.iter_mut() {
        // レイアウトを再計算
        layout.compute();
        
        // マーカーを削除（処理完了）
        commands.entity(entity).remove::<LayoutInvalidated>();
    }
}

/// 再描画が必要なものだけ描画
pub fn draw_visual_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Visual), With<NeedsRedraw>>,
    dcomp_context: Res<DCompContext>,
) {
    for (entity, mut visual) in query.iter_mut() {
        // 描画処理
        draw_to_surface(&mut visual, &dcomp_context);
        
        // マーカーを削除
        commands.entity(entity).remove::<NeedsRedraw>();
    }
}
```

**利点**:
- 複数システム間での調整が容易
- 処理の段階を明確にできる（マーク → 処理 → クリア）
- デバッグしやすい（どのEntityがダーティか可視化できる）

**使い分け**:
- 単純な1対1の依存: `Changed<T>`
- 複数の原因で同じ処理が必要: マーカーコンポーネント

### 変更伝播の設計パターン

#### パターン1: カスケード更新（親→子）

```rust
/// 親のレイアウトが変わったら子も無効化
pub fn propagate_layout_invalidation_system(
    mut commands: Commands,
    changed_parents: Query<&Children, Changed<Layout>>,
) {
    for children in changed_parents.iter() {
        for child in children.iter() {
            commands.entity(*child).insert(LayoutInvalidated);
        }
    }
}
```

#### パターン2: リアクティブ更新（A→B）

```rust
/// TextContentが変更されたらLayoutを無効化
pub fn text_to_layout_system(
    mut commands: Commands,
    query: Query<Entity, Changed<TextContent>>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(LayoutInvalidated);
    }
}

/// Layoutが変更されたらVisualのトランスフォームを更新
pub fn layout_to_visual_system(
    mut query: Query<(&Layout, &mut Visual), Changed<Layout>>,
) {
    for (layout, mut visual) in query.iter_mut() {
        visual.update_transform_from_layout(layout);
        // visualを変更したので、自動的にChanged<Visual>になる
    }
}

/// Visualが変更されたら再描画をマーク
pub fn visual_to_redraw_system(
    mut commands: Commands,
    query: Query<Entity, Changed<Visual>>,
) {
    for entity in query.iter() {
        commands.entity(entity).insert(NeedsRedraw);
    }
}
```

#### パターン3: バッチ処理

```rust
/// 複数の変更をまとめて処理
pub fn batch_visual_update_system(
    mut commands: Commands,
    // 複数の変更原因をORで結合
    query: Query<Entity, Or<(
        Changed<TextContent>,
        Changed<ImageContent>,
        Changed<ContainerStyle>,
        Changed<Layout>,
    )>>,
) {
    for entity in query.iter() {
        // どれか1つでも変更されていれば再描画
        commands.entity(entity).insert(NeedsRedraw);
    }
}
```

### システムの実行順序制御

bevy_ecsでは`.chain()`や`.before()`/`.after()`で順序を制御。

```rust
pub fn setup_visual_systems(app: &mut App) {
    app.add_systems(Update, (
        // 1. コンテンツ変更検知
        text_changed_system,
        style_changed_system,
        
        // 2. レイアウト無効化
        invalidate_layout_system,
        
        // 3. レイアウト計算
        compute_layout_system,
        
        // 4. Visual更新
        layout_to_visual_system,
        ensure_visual_system,
        attach_new_visual_system,
        
        // 5. 描画
        draw_visual_system,
    ).chain()); // 順番に実行
    
    // 並列実行可能なシステム
    app.add_systems(Update, (
        text_changed_system,
        image_changed_system,
        style_changed_system,
        // これらは独立しているので並列実行される
    ));
}
```

### 変更検知のパフォーマンス最適化

#### 1. 過剰な変更検知を避ける

```rust
// ❌ 悪い例: 毎フレーム変更扱いになる
pub fn bad_system(mut query: Query<&mut Transform>) {
    for mut transform in query.iter_mut() {
        // 読み取り専用でも&mutで取得すると変更扱い
        let pos = transform.position;
    }
}

// ✅ 良い例: 必要な時だけ&mut
pub fn good_system(
    read_query: Query<&Transform, Without<NeedsUpdate>>,
    mut write_query: Query<&mut Transform, With<NeedsUpdate>>,
) {
    // 読み取り専用
    for transform in read_query.iter() {
        let pos = transform.position;
    }
    
    // 更新が必要なものだけ&mut
    for mut transform in write_query.iter_mut() {
        transform.position.x += 1.0;
    }
}
```

#### 2. 変更フラグのリセット

```rust
// Changed<T>は自動的にリセットされる（次のフレームでは"changed"ではない）
// マーカーコンポーネントは明示的に削除が必要

pub fn process_and_clear_system(
    mut commands: Commands,
    query: Query<Entity, With<NeedsRedraw>>,
) {
    for entity in query.iter() {
        // 処理...
        
        // 必ず削除する（さもないと毎フレーム処理される）
        commands.entity(entity).remove::<NeedsRedraw>();
    }
}
```

#### 3. 階層的な変更伝播の最適化

```rust
/// 親が変更されても、子が影響を受けない場合は伝播しない
pub fn smart_propagation_system(
    mut commands: Commands,
    changed_parents: Query<(&Layout, &Children), Changed<Layout>>,
    child_layouts: Query<&Layout>,
) {
    for (parent_layout, children) in changed_parents.iter() {
        for child in children.iter() {
            if let Ok(child_layout) = child_layouts.get(*child) {
                // 親の変更が子に影響する場合のみ伝播
                if parent_layout.affects_child(child_layout) {
                    commands.entity(*child).insert(LayoutInvalidated);
                }
            }
        }
    }
}
```

### まとめ: ダーティ検出戦略の選択指針

| 状況 | 推奨方法 | 理由 |
|------|---------|------|
| コンポーネントの値変更を検知 | `Changed<T>` | 自動追跡、最も効率的 |
| コンポーネントの追加を検知 | `Added<T>` | 初期化処理に最適 |
| 複数の原因で同じ処理 | マーカーコンポーネント | 処理の集約が容易 |
| 段階的な処理フロー | マーカーコンポーネント | 状態遷移が明確 |
| 親→子への伝播 | `Changed<T>` + `Children` | カスケード更新 |
| デバッグ・可視化が必要 | マーカーコンポーネント | 状態が可視化できる |

**基本方針**:
1. まず`Changed<T>`/`Added<T>`を使う（シンプル、効率的）
2. 複雑な依存関係があればマーカーコンポーネントを追加
3. システムの実行順序を`.chain()`で明示的に制御
4. 不要な&mutアクセスを避ける

## DirectComposition アーキテクチャ

### 基本構造

```
Window
  └─ IDCompositionDesktopDevice (デバイス)
      └─ IDCompositionTarget (ウィンドウとの関連付け)
          └─ IDCompositionVisual (ルートビジュアル)
              ├─ IDCompositionVisual (子1)
              │   └─ IDCompositionSurface (描画サーフェス)
              ├─ IDCompositionVisual (子2)
              └─ IDCompositionVisual (子3)
```

### Visual の初期化

```rust
use bevy_ecs::prelude::*;
use windows::Win32::Graphics::DirectComposition::*;
use windows::Win32::Graphics::Direct2D::*;

#[derive(Resource)]
pub struct DCompContext {
    device: IDCompositionDesktopDevice,
    target: IDCompositionTarget,
    d2d_device: ID2D1Device,
    root_visual: IDCompositionVisual,
}

impl DCompContext {
    pub fn new(hwnd: HWND) -> Result<Self> {
        unsafe {
            // DirectCompositionデバイスの作成
            let device: IDCompositionDesktopDevice = 
                DCompositionCreateDevice(None)?;
            
            // ターゲットの作成（ウィンドウに関連付け）
            let target = device.CreateTargetForHwnd(hwnd, true)?;
            
            // ルートビジュアルの作成
            let root_visual = device.CreateVisual()?;
            target.SetRoot(&root_visual)?;
            
            // Direct2Dデバイスの作成（描画用）
            let d2d_factory: ID2D1Factory1 = 
                D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, None)?;
            let d2d_device = d2d_factory.CreateDevice(/* ... */)?;
            
            Ok(Self {
                device,
                target,
                d2d_device,
                root_visual,
            })
        }
    }
    
    pub fn commit(&self) -> Result<()> {
        unsafe { self.device.Commit() }
    }
}
```

## Visual 管理システム

### Visual コンポーネントの定義（bevy_ecs版）

**Visualは`Component`として定義され、必要なEntityのみが持つ**

```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Visual {
    // DirectComposition オブジェクト
    pub dcomp_visual: IDCompositionVisual,
    pub dcomp_surface: Option<IDCompositionSurface>,
    pub d2d_device_context: Option<ID2D1DeviceContext>,
    
    // トランスフォーム
    pub offset: Point2D,
    pub size: Size2D,
    pub scale: Vector2D,
    pub rotation: f32, // ラジアン
    pub transform_matrix: Option<Matrix3x2>,
    
    // 表示設定
    pub opacity: f32,
    pub visible: bool,
    pub clip_rect: Option<Rect>,
    
    // ヒットテスト設定
    pub is_hit_testable: bool,
    pub hit_test_geometry: HitTestGeometry,
}

#[derive(Component)]
pub struct DirtyVisual;

#[derive(Component)]
pub struct NeedsRedraw;

#[derive(Component)]
pub struct NeedsLayoutUpdate;

pub enum HitTestGeometry {
    Rectangle(Rect),
    Ellipse { center: Point2D, radius: Vector2D },
    Path(ID2D1Geometry),
    PixelPerfect, // サーフェスのアルファ値を使用
}
```

### Visual作成の判断

```rust
use bevy_ecs::prelude::*;

/// Entityが描画を必要とするか判定するシステム
pub fn needs_visual_system(
    mut commands: Commands,
    query: Query<
        (Entity, 
         Option<&TextContent>, 
         Option<&ImageContent>,
         Option<&ContainerStyle>,
         Option<&CustomDraw>),
        Without<Visual>
    >,
) {
    for (entity, text, image, style, custom) in query.iter() {
        let needs_visual = text.is_some() 
            || image.is_some()
            || style.map(|s| s.background.is_some() || s.border.is_some()).unwrap_or(false)
            || custom.is_some();
        
        if needs_visual {
            commands.entity(entity).insert(DirtyVisual);
        }
    }
}
```

### Visual の生成と管理

**Visualコンポーネントは動的に追加され、親のVisualツリーに自動接続される**

```rust
/// Visualコンポーネントを動的に作成または取得
pub fn ensure_visual_system(
    mut commands: Commands,
    query: Query<Entity, With<DirtyVisual>>,
    dcomp_context: Res<DCompContext>,
) {
    for entity in query.iter() {
        // DirectCompositionビジュアルを作成
        let dcomp_visual = unsafe { 
            dcomp_context.device.CreateVisual().unwrap()
        };
        
        let visual = Visual {
            dcomp_visual,
            dcomp_surface: None,
            d2d_device_context: None,
            offset: Point2D::zero(),
            size: Size2D::zero(),
            scale: Vector2D::new(1.0, 1.0),
            rotation: 0.0,
            transform_matrix: None,
            opacity: 1.0,
            visible: true,
            clip_rect: None,
            is_hit_testable: true,
            hit_test_geometry: HitTestGeometry::Rectangle(Rect::zero()),
        };
        
        commands.entity(entity)
            .insert(visual)
            .insert(NeedsRedraw)
            .insert(NeedsLayoutUpdate);
    }
}

/// VisualをDirectCompositionツリーに接続
pub fn attach_visual_to_tree_system(
    mut commands: Commands,
    visual_query: Query<(Entity, &Visual, &Parent), Added<Visual>>,
    parent_visual_query: Query<&Visual, Without<Added<Visual>>>,
    dcomp_context: Res<DCompContext>,
) {
    for (entity, visual, parent) in visual_query.iter() {
        // 親でVisualを持つ最も近いEntityを探す
        if let Some(parent_visual) = find_parent_with_visual(parent.get(), &parent_visual_query) {
            unsafe {
                parent_visual.dcomp_visual
                    .AddVisual(&visual.dcomp_visual, true, None)
                    .ok();
            }
        } else {
            // 親がない場合、ルートのVisualに接続
            unsafe {
                dcomp_context.root_visual
                    .AddVisual(&visual.dcomp_visual, true, None)
                    .ok();
            }
        }
        
        commands.entity(entity).remove::<DirtyVisual>();
    }
}

/// 親でVisualコンポーネントを持つEntityを探す（再帰的に上へ）
fn find_parent_with_visual(
    entity: Entity,
    parent_visual_query: &Query<&Visual, Without<Added<Visual>>>,
) -> Option<&Visual> {
    // bevy_ecsの階層システムを使用して親を辿る
    parent_visual_query.get(entity).ok()
}

/// Visualを削除（不要になった場合）
pub fn remove_visual_system(
    mut commands: Commands,
    mut removed: RemovedComponents<Visual>,
    visual_query: Query<(&Visual, Option<&Parent>)>,
    parent_visual_query: Query<&Visual>,
    dcomp_context: Res<DCompContext>,
) {
    for entity in removed.read() {
        if let Ok((visual, parent)) = visual_query.get(entity) {
            // DirectCompositionツリーから削除
            if let Some(parent) = parent {
                if let Ok(parent_visual) = parent_visual_query.get(parent.get()) {
                    unsafe {
                        parent_visual.dcomp_visual
                            .RemoveVisual(&visual.dcomp_visual)
                            .ok();
                    }
                }
            } else {
                // ルートから削除
                unsafe {
                    dcomp_context.root_visual
                        .RemoveVisual(&visual.dcomp_visual)
                        .ok();
                }
            }
        }
    }
}
```

## 論理ツリーとビジュアルツリーの関係

### ツリー構造の例

論理ツリー（Entity）とビジュアルツリー（DirectComposition）は1:1対応しない：

```
論理ツリー (Entity, bevy_ecs):           ビジュアルツリー (DirectComposition):

Window                                     Window Root Visual
  └─ Root                                    ├─ Panel Visual (背景あり)
      ├─ Panel (背景あり)                    │   ├─ TextBlock1 Visual
      │   ├─ TextBlock1                     │   └─ Image1 Visual
      │   └─ LayoutContainer (透明)         └─ TextBlock2 Visual
      │       ├─ Image1
      │       └─ Spacer (透明)
      └─ TextBlock2

Visualなし: LayoutContainer, Spacer
Visualあり: Panel, TextBlock1, Image1, TextBlock2
```

### 子の追加時の処理

```rust
/// 子をEntityに追加するシステム
pub fn append_child_system(
    mut commands: Commands,
    // bevy_ecsのParentコンポーネントを使用
) {
    // bevy_ecsの標準的な親子関係システムを使用
    // commands.entity(parent).add_child(child);
}

/// 新しく追加されたVisualを親ツリーに接続
pub fn connect_new_visual_system(
    visual_query: Query<(Entity, &Visual), Added<Visual>>,
    parent_query: Query<&Parent>,
    parent_visual_query: Query<&Visual>,
    dcomp_context: Res<DCompContext>,
) {
    for (entity, visual) in visual_query.iter() {
        // 親でVisualを持つEntityを探す
        if let Ok(parent) = parent_query.get(entity) {
            let mut current = parent.get();
            
            while let Ok(parent_entity) = parent_query.get(current) {
                if let Ok(parent_visual) = parent_visual_query.get(current) {
                    // 親のVisualに接続
                    unsafe {
                        parent_visual.dcomp_visual
                            .AddVisual(&visual.dcomp_visual, true, None)
                            .ok();
                    }
                    break;
                }
                current = parent_entity.get();
            }
        }
    }
}
```

### サーフェスの作成と描画

```rust
/// Visualにコンテンツを描画するシステム
pub fn draw_visual_system(
    mut visual_query: Query<(Entity, &mut Visual), With<NeedsRedraw>>,
    text_query: Query<&TextContent>,
    image_query: Query<&ImageContent>,
    style_query: Query<&ContainerStyle>,
    dcomp_context: Res<DCompContext>,
) {
    for (entity, mut visual) in visual_query.iter_mut() {
        unsafe {
            // サーフェスを作成（まだなければ）
            if visual.dcomp_surface.is_none() {
                let surface = dcomp_context.device.CreateSurface(
                    visual.size.width as u32,
                    visual.size.height as u32,
                    DXGI_FORMAT_B8G8R8A8_UNORM,
                    DXGI_ALPHA_MODE_PREMULTIPLIED,
                ).ok();
                
                if let Some(ref surf) = surface {
                    visual.dcomp_visual.SetContent(surf).ok();
                }
                visual.dcomp_surface = surface;
            }
            
            // サーフェスに描画開始
            if let Some(ref surface) = visual.dcomp_surface {
                let mut offset = POINT::default();
                if let Ok(dc) = surface.BeginDraw(None, &mut offset) {
                    // Direct2Dデバイスコンテキストを作成
                    if visual.d2d_device_context.is_none() {
                        visual.d2d_device_context = dcomp_context.d2d_device
                            .CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE)
                            .ok();
                    }
                    
                    if let Some(ref d2d_dc) = visual.d2d_device_context {
                        d2d_dc.SetTarget(/* サーフェスのビットマップ */);
                        
                        d2d_dc.BeginDraw();
                        d2d_dc.Clear(Some(&D2D1_COLOR_F {
                            r: 0.0, g: 0.0, b: 0.0, a: 0.0, // 透明
                        }));
                        
                        // コンテンツの種類に応じて描画
                        if let Ok(text) = text_query.get(entity) {
                            draw_text_content(d2d_dc, text).ok();
                        }
                        if let Ok(image) = image_query.get(entity) {
                            draw_image_content(d2d_dc, image).ok();
                        }
                        if let Ok(style) = style_query.get(entity) {
                            draw_container_style(d2d_dc, style).ok();
                        }
                        
                        d2d_dc.EndDraw(None, None).ok();
                    }
                    surface.EndDraw().ok();
                }
            }
        }
    }
}
```

### テキスト描画（縦書き対応）

```rust
fn draw_text_content(
    dc: &ID2D1DeviceContext,
    text: &TextContent,
) -> Result<()> {
    unsafe {
        // DirectWriteテキストレイアウトを作成
        let text_layout = text.dwrite_factory.CreateTextLayout(
            &text.text,
            &text.text_format,
            text.max_width,
            text.max_height,
        )?;
        
        // 縦書き設定
        text_layout.SetReadingDirection(match text.reading_direction {
            ReadingDirection::TopToBottom => DWRITE_READING_DIRECTION_TOP_TO_BOTTOM,
            ReadingDirection::LeftToRight => DWRITE_READING_DIRECTION_LEFT_TO_RIGHT,
        })?;
        
        text_layout.SetFlowDirection(match text.flow_direction {
            FlowDirection::TopToBottom => DWRITE_FLOW_DIRECTION_TOP_TO_BOTTOM,
            FlowDirection::LeftToRight => DWRITE_FLOW_DIRECTION_LEFT_TO_RIGHT,
        })?;
        
        // ブラシを作成
        let brush = dc.CreateSolidColorBrush(
            &D2D1_COLOR_F {
                r: 0.0, g: 0.0, b: 0.0, a: 1.0, // 黒
            },
            None,
        )?;
        
        // 描画
        dc.DrawTextLayout(
            D2D_POINT_2F { x: 0.0, y: 0.0 },
            &text_layout,
            &brush,
            D2D1_DRAW_TEXT_OPTIONS_NONE,
        );
    }
    
    Ok(())
}
```

### 画像描画

```rust
fn draw_image_content(
    dc: &ID2D1DeviceContext,
    image: &ImageContent,
) -> Result<()> {
    unsafe {
        // ビットマップを描画
        let dest_rect = D2D_RECT_F {
            left: 0.0,
            top: 0.0,
            right: image.width,
            bottom: image.height,
        };
        
        let source_rect = image.source_rect.map(|r| D2D_RECT_F {
            left: r.x,
            top: r.y,
            right: r.x + r.width,
            bottom: r.y + r.height,
        });
        
        dc.DrawBitmap(
            &image.bitmap,
            Some(&dest_rect),
            image.opacity,
            D2D1_INTERPOLATION_MODE_LINEAR,
            source_rect.as_ref(),
        );
    }
    
    Ok(())
}
```

## トランスフォーム管理

### トランスフォームの適用

```rust
/// トランスフォームを更新するシステム
pub fn update_transform_system(
    mut visual_query: Query<&mut Visual, With<NeedsLayoutUpdate>>,
    dcomp_context: Res<DCompContext>,
) {
    for mut visual in visual_query.iter_mut() {
        unsafe {
            // オフセット
            visual.dcomp_visual.SetOffsetX(visual.offset.x).ok();
            visual.dcomp_visual.SetOffsetY(visual.offset.y).ok();
            
            // 複合トランスフォーム（スケール + 回転）
            if visual.scale != Vector2D::new(1.0, 1.0) || visual.rotation != 0.0 {
                if let Ok(transform) = dcomp_context.device.CreateMatrixTransform() {
                    let matrix = calculate_transform_matrix(&visual);
                    transform.SetMatrix(&matrix).ok();
                    visual.dcomp_visual.SetTransform(&transform).ok();
                }
            }
            
            // 不透明度
            visual.dcomp_visual.SetOpacity(visual.opacity).ok();
            
            // クリッピング
            if let Some(clip) = visual.clip_rect {
                if let Ok(clip_visual) = dcomp_context.device.CreateRectangleClip() {
                    clip_visual.SetLeft(clip.x).ok();
                    clip_visual.SetTop(clip.y).ok();
                    clip_visual.SetRight(clip.x + clip.width).ok();
                    clip_visual.SetBottom(clip.y + clip.height).ok();
                    visual.dcomp_visual.SetClip(&clip_visual).ok();
                }
            }
        }
    }
}

fn calculate_transform_matrix(visual: &Visual) -> D2D_MATRIX_3X2_F {
    // スケール行列
    let scale_matrix = D2D_MATRIX_3X2_F {
        _11: visual.scale.x, _12: 0.0,
        _21: 0.0, _22: visual.scale.y,
        _31: 0.0, _32: 0.0,
    };
    
    // 回転行列
    let cos = visual.rotation.cos();
    let sin = visual.rotation.sin();
    let rotation_matrix = D2D_MATRIX_3X2_F {
        _11: cos, _12: sin,
        _21: -sin, _22: cos,
        _31: 0.0, _32: 0.0,
    };
    
    // 合成
    matrix_multiply(&scale_matrix, &rotation_matrix)
}
```

## 更新サイクル

### フレーム更新の流れ（bevy_ecs版）

```rust
use bevy_ecs::prelude::*;

/// メインの更新スケジュール
pub fn build_update_schedule(app: &mut App) {
    app.add_systems(Update, (
        // 1. レイアウト更新
        update_layouts_system,
        
        // 2. レイアウト情報をVisualに反映
        sync_layout_to_visuals_system,
        
        // 3. Visualを必要に応じて追加/削除
        needs_visual_system,
        ensure_visual_system,
        attach_visual_to_tree_system,
        
        // 4. トランスフォーム更新
        update_transform_system,
        
        // 5. 再描画
        draw_visual_system,
        
        // 6. DirectCompositionにコミット
        commit_dcomp_system,
    ).chain());
}

pub fn sync_layout_to_visuals_system(
    layout_query: Query<(&Layout, &mut Visual), Changed<Layout>>,
) {
    for (layout, mut visual) in layout_query.iter_mut() {
        visual.offset = layout.final_rect.origin;
        visual.size = layout.final_rect.size;
    }
}

pub fn commit_dcomp_system(
    dcomp_context: Res<DCompContext>,
    mut query: Query<Entity, Or<(With<NeedsRedraw>, With<NeedsLayoutUpdate>)>>,
    mut commands: Commands,
) {
    if !query.is_empty() {
        dcomp_context.commit().ok();
        
        // ダーティフラグをクリア
        for entity in query.iter_mut() {
            commands.entity(entity)
                .remove::<NeedsRedraw>()
                .remove::<NeedsLayoutUpdate>();
        }
    }
}
```

## アニメーション

DirectCompositionは高性能なアニメーションをサポートします。

```rust
use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Animation {
    property: AnimationProperty,
    duration: Duration,
    easing: EasingFunction,
    elapsed: Duration,
}

pub enum AnimationProperty {
    Opacity(f32, f32), // from, to
    Offset(Point2D, Point2D),
    Scale(Vector2D, Vector2D),
    Rotation(f32, f32),
}

/// アニメーションを開始するシステム
pub fn start_animation_system(
    mut commands: Commands,
    query: Query<(Entity, &Visual, &Animation), Added<Animation>>,
    dcomp_context: Res<DCompContext>,
) {
    for (entity, visual, anim) in query.iter() {
        unsafe {
            let animation = dcomp_context.device.CreateAnimation().ok();
            
            if let Some(dcomp_anim) = animation {
                // アニメーションパラメータを設定
                match anim.property {
                    AnimationProperty::Opacity(from, to) => {
                        dcomp_anim.AddCubic(
                            0.0,     // beginOffset
                            from,    // constantCoefficient
                            0.0,     // linearCoefficient
                            0.0,     // quadraticCoefficient
                            (to - from) / anim.duration.as_secs_f32(), // cubicCoefficient
                        ).ok();
                        
                        visual.dcomp_visual.SetOpacity_2(&dcomp_anim).ok();
                    }
                    // 他のプロパティも同様に...
                    _ => {}
                }
            }
        }
    }
}

/// アニメーションの進行を更新
pub fn update_animation_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Animation)>,
    time: Res<Time>,
) {
    for (entity, mut anim) in query.iter_mut() {
        anim.elapsed += time.delta();
        
        if anim.elapsed >= anim.duration {
            // アニメーション完了
            commands.entity(entity).remove::<Animation>();
        }
    }
}
```

## trait WtfVisual

すべての描画可能な要素が実装すべきトレイト。

```rust
use bevy_ecs::prelude::*;

pub trait WtfVisual {
    /// Visualにコンテンツを描画
    fn draw(&self, dc: &ID2D1DeviceContext, visual: &Visual) -> Result<()>;
    
    /// 望ましいサイズを計算
    fn measure(&self, available_size: Size2D) -> Size2D;
    
    /// ヒットテスト
    fn hit_test(&self, visual: &Visual, point: Point2D) -> bool {
        visual.hit_test_geometry.contains(point)
    }
    
    /// プロパティ変更通知
    fn on_property_changed(&mut self, property: &str) {
        // デフォルト実装：何もしない
    }
}

// TextContentの実装例
#[derive(Component)]
pub struct TextContent {
    pub text: String,
    pub text_format: IDWriteTextFormat,
    pub text_layout: IDWriteTextLayout,
    pub reading_direction: ReadingDirection,
    pub flow_direction: FlowDirection,
    pub max_width: f32,
    pub max_height: f32,
}

impl WtfVisual for TextContent {
    fn draw(&self, dc: &ID2D1DeviceContext, visual: &Visual) -> Result<()> {
        // DirectWriteでテキストを描画
        unsafe {
            let brush = dc.CreateSolidColorBrush(
                &D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 1.0 },
                None,
            )?;
            
            dc.DrawTextLayout(
                D2D_POINT_2F { x: 0.0, y: 0.0 },
                &self.text_layout,
                &brush,
                D2D1_DRAW_TEXT_OPTIONS_NONE,
            );
        }
        Ok(())
    }
    
    fn measure(&self, available_size: Size2D) -> Size2D {
        // テキストレイアウトからサイズを取得
        unsafe {
            let metrics = self.text_layout.GetMetrics().unwrap();
            Size2D::new(metrics.width, metrics.height)
        }
    }
}

/// テキストが変更されたときの処理システム
pub fn text_changed_system(
    mut commands: Commands,
    query: Query<Entity, Changed<TextContent>>,
) {
    for entity in query.iter() {
        commands.entity(entity)
            .insert(NeedsRedraw)
            .insert(DirtyVisual);
    }
}
```

## まとめ

DirectCompositionとbevy_ecsを使ったビジュアル管理の特徴：

1. **ハードウェアアクセラレーション**: GPUで合成、高性能
2. **独立したコンポジション**: UIスレッドをブロックしない
3. **スムーズなアニメーション**: 60FPS以上も容易
4. **透過ウィンドウ対応**: アルファブレンディングが標準
5. **動的Visualコンポーネント**: 必要なEntityのみがVisualを持つ（メモリ効率）
6. **自動ツリー管理**: Visualなし中間ノードを透過的にスキップ
7. **効率的な更新**: 変更検知とクエリで必要な部分のみ処理
8. **並列処理**: bevy_ecsのシステム並列実行で高速化

### bevy_ecs採用の利点

**メモリ効率**
- 純粋なレイアウトノードはVisualコンポーネントを持たない
- 数千のEntity中、実際に描画するのは一部のみ
- コンポーネントベースで必要な機能のみ追加

**柔軟性**
- Entityの追加/削除時、Visualツリーを自動調整
- 背景色の追加/削除でVisualの動的追加/削除
- 実行時にコンポーネントの組み合わせを変更可能

**パフォーマンス**
- DirectCompositionはVisual数が少ないほど高速
- 不要なVisualを作らないことで合成が軽量化
- クエリベースで変更された要素のみ効率的に処理
- システムの並列実行で複数コアを活用

**保守性**
- システムごとに責務が明確に分離
- 各システムは独立してテスト可能
- 新機能追加時は新しいコンポーネント/システムを追加するだけ

**データ指向設計**
- コンポーネントはデータのみ、ロジックはシステムに分離
- キャッシュ効率の良いメモリレイアウト
- クエリで必要なコンポーネントだけをイテレート


