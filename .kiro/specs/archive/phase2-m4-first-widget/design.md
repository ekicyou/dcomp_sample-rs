# Design: Phase 2 Milestone 4 - 初めてのウィジット

**Feature ID**: `phase2-m4-first-widget`  
**Version**: 1.0  
**Last Updated**: 2025-11-14  
**Status**: Phase 2 - Design

---

## 1. Architecture Overview

### 1.1 High-Level Design

Phase 2-M4では、Widget描画の基盤として以下のアーキテクチャを実装します：

```
┌─────────────────────────────────────────────────────────┐
│              ECS Application Layer                      │
│  (simple_window.rs: Rectangle追加)                      │
└─────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────┐
│           Draw Schedule (CommandList生成)               │
│  draw_rectangles: Rectangle → GraphicsCommandList      │
└─────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────┐
│         Render Schedule (Surface描画)                   │
│  render_surface: Option<&GraphicsCommandList>           │
│    1. 透明色クリア（常に実行）                           │
│    2. CommandList描画（Someの場合のみ）                 │
│    - Changed検知で削除時も対応                          │
└─────────────────────────────────────────────────────────┘
                         ↓
┌─────────────────────────────────────────────────────────┐
│            DirectComposition Layer                      │
│  (commit_composition: Surface → 画面表示)               │
└─────────────────────────────────────────────────────────┘
```

### 1.2 Entity構成

**WindowエンティティにWidgetコンポーネントを直接追加**（シンプル設計）：

```
Window Entity {
    // 既存コンポーネント (Phase 2-M1~M3)
    Window,              // ウィンドウハンドル
    WindowHandle,        // HWND
    WindowGraphics,      // デバイス情報
    Visual,              // DirectComposition Visual
    Surface,             // DirectComposition Surface
    
    // 新規コンポーネント (Phase 2-M4)
    Rectangle,           // Widget: 四角形の描画情報
    GraphicsCommandList, // 描画命令
}
```

**設計判断**: このマイルストーンでは、Entity階層構造（ChildOf/Children）やVisualツリーは構築せず、Windowエンティティに全コンポーネントを配置します。これにより、Widget描画の基本パイプラインを最小限の複雑性で実装できます。

---

## 2. Component Design

### 2.1 Rectangle Component

**役割**: 四角形の描画情報を宣言的に定義するWidgetコンポーネント

**定義**:
```rust
// crates/wintf/src/ecs/widget/shapes/rectangle.rs

use bevy_ecs::component::Component;
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;

// 型エイリアス: 既存のD2D1_COLOR_Fを使用
pub type Color = D2D1_COLOR_F;

/// 四角形ウィジット
#[derive(Component, Debug, Clone)]
pub struct Rectangle {
    /// X座標（ピクセル単位）
    pub x: f32,
    /// Y座標（ピクセル単位）
    pub y: f32,
    /// 幅（ピクセル単位）
    pub width: f32,
    /// 高さ（ピクセル単位）
    pub height: f32,
    /// 塗りつぶし色
    pub color: Color,
}

// 定数定義
impl Color {
    pub const RED: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const BLUE: Color = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
    pub const GREEN: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const TRANSPARENT: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
}
```

**モジュール構成**:
```rust
// crates/wintf/src/ecs/widget/mod.rs
pub mod shapes;

// crates/wintf/src/ecs/widget/shapes/mod.rs
pub mod rectangle;
pub use rectangle::{Rectangle, Color};
```

### 2.2 GraphicsCommandList Component

**役割**: Direct2Dの描画命令を保持し、効率的な描画パイプラインを実現

**定義**:
```rust
// crates/wintf/src/ecs/graphics/command_list.rs

use bevy_ecs::component::Component;
use windows::Win32::Graphics::Direct2D::ID2D1CommandList;

/// Direct2D描画命令リスト
#[derive(Component, Debug)]
pub struct GraphicsCommandList {
    command_list: ID2D1CommandList,
}

impl GraphicsCommandList {
    /// 新しいCommandListコンポーネントを作成
    pub fn new(command_list: ID2D1CommandList) -> Self {
        Self { command_list }
    }

    /// CommandListへの参照を取得
    pub fn command_list(&self) -> &ID2D1CommandList {
        &self.command_list
    }
}

// スレッド間送信を可能にする（windows-rsのスマートポインタはSend+Sync）
unsafe impl Send for GraphicsCommandList {}
unsafe impl Sync for GraphicsCommandList {}
```

**安全性の保証**:
- `ID2D1CommandList`は`windows-rs`が提供するスマートポインタ（`IUnknown`派生）
- COMオブジェクトの参照カウント管理は自動的に行われる
- `Send`/`Sync`の実装は、DirectXの仕様上スレッドセーフであることを前提とする

---

## 3. Module Refactoring

### 3.1 graphics.rs → graphics/ ディレクトリ化

**目的**: コードの責務を明確にし、保守性を向上させる

**Before**:
```
src/ecs/
  ├── graphics.rs        (すべてのコードが1ファイル)
  └── ...
```

**After**:
```
src/ecs/
  ├── graphics/
  │   ├── mod.rs         (公開API + Re-exports)
  │   ├── core.rs        (GraphicsCore リソース)
  │   ├── components.rs  (WindowGraphics, Visual, Surface)
  │   ├── command_list.rs (GraphicsCommandList)
  │   └── systems.rs     (描画システム群)
  └── ...
```

**ファイル責務**:

#### `graphics/mod.rs`
```rust
// 公開APIとRe-exports
mod core;
mod components;
mod command_list;
mod systems;

pub use core::*;
pub use components::*;
pub use command_list::*;
pub use systems::*;
```

#### `graphics/core.rs`
```rust
// GraphicsCoreリソース
pub struct GraphicsCore { ... }
pub fn ensure_graphics_core(...) { ... }
```

#### `graphics/components.rs`
```rust
// コンポーネント定義
pub struct WindowGraphics { ... }
pub struct Visual { ... }
pub struct Surface { ... }
```

#### `graphics/command_list.rs`
```rust
// GraphicsCommandListコンポーネント（新規）
pub struct GraphicsCommandList { ... }
```

#### `graphics/systems.rs`
```rust
// システム関数群
pub fn create_window_graphics(...) { ... }
pub fn create_window_visual(...) { ... }
pub fn create_window_surface(...) { ... }
pub fn render_window(...) { ... }        // 変更: Without<GraphicsCommandList>
pub fn render_surface(...) { ... }       // 新規
pub fn commit_composition(...) { ... }
```

**マイグレーション手順**:
1. `graphics/`ディレクトリを作成
2. 既存の`graphics.rs`の内容を分割して各ファイルに移動
3. `mod.rs`でre-exportを設定
4. 他モジュールの`use`文は変更不要（`use crate::ecs::graphics::*;`で互換性維持）
5. 既存テストを実行して動作確認

---

## 4. System Design

### 4.1 draw_rectangles System

**役割**: RectangleコンポーネントからGraphicsCommandListを生成

**実装場所**: `crates/wintf/src/ecs/widget/shapes/rectangle.rs`

**スケジュール**: `Draw`

**シグネチャ**:
```rust
pub fn draw_rectangles(
    mut commands: Commands,
    query: Query<(Entity, &Rectangle), Changed<Rectangle>>,
    graphics_core: Option<Res<GraphicsCore>>,
) {
    // 実装
}
```

**処理フロー**:
```
1. GraphicsCoreが存在するか確認
   ↓ No → 警告ログを出力して終了
   ↓ Yes
2. Changed<Rectangle>でフィルタリングされたエンティティをループ
   ↓
3. ID2D1CommandList生成
   - factory.CreateCommandList()
   ↓
4. CommandListを開く
   - command_list.Open() → ID2D1DeviceContext取得
   ↓
5. 描画命令を記録
   - dc.BeginDraw()
   - dc.Clear(透明色)
   - dc.FillRectangle(Rectangle情報)
   - dc.EndDraw()
   ↓
6. CommandListを閉じる
   - command_list.Close()
   ↓
7. GraphicsCommandListコンポーネントを挿入/更新
   - commands.entity(entity).insert(GraphicsCommandList::new(...))
```

**エラーハンドリング**:
- COM API呼び出しの失敗: エラーログを出力し、該当エンティティをスキップ
- GraphicsCoreが存在しない: 警告ログを出力して処理全体をスキップ

**ログ出力**:
```rust
// 開始
tracing::info!("draw_rectangles: Entity={:?}", entity);

// Rectangle情報
tracing::debug!(
    "Rectangle: x={}, y={}, width={}, height={}, color=({},{},{},{})",
    rect.x, rect.y, rect.width, rect.height,
    rect.color.r, rect.color.g, rect.color.b, rect.color.a
);

// CommandList生成成功
tracing::info!("CommandList created for Entity={:?}", entity);

// エラー
tracing::error!("Failed to create CommandList for Entity={:?}: {:?}", entity, err);
```

### 4.2 render_surface System（統合版）

**役割**: Surfaceへの描画（GraphicsCommandListがあれば描画、なければクリアのみ）

**実装場所**: `crates/wintf/src/ecs/graphics/systems.rs`

**スケジュール**: `Render`

**シグネチャ**:
```rust
pub fn render_surface(
    query: Query<
        (Entity, Option<&GraphicsCommandList>, &Surface),
        Or<(Changed<GraphicsCommandList>, Changed<Surface>)>
    >,
    graphics_core: Option<Res<GraphicsCore>>,
) {
    // 実装
}
```

**処理フロー**:
```
1. GraphicsCoreが存在するか確認
   ↓ No → 警告ログを出力して終了
   ↓ Yes
2. Changed<GraphicsCommandList> または Changed<Surface>のエンティティをループ
   ↓
3. Surface描画開始
   - surface.BeginDraw() → ID2D1DeviceContext取得
   ↓
4. 透明色クリア（常に実行）
   - dc.Clear(透明色: rgba 0.0, 0.0, 0.0, 0.0)
   ↓
5. CommandList描画（GraphicsCommandListがある場合のみ）
   - if let Some(command_list) = command_list {
       dc.DrawImage(command_list.command_list(), None, None, ...)
     }
   ↓
6. Surface描画終了
   - dc.EndDraw()
   - surface.EndDraw()
```

**Changed<GraphicsCommandList>の動作**:
- コンポーネント追加時: `Some` → クリア + CommandList描画
- コンポーネント変更時: `Some` → クリア + CommandList描画
- コンポーネント削除時: `None` → クリアのみ（自動対応！）

**重要**: すべてのケースで透明色クリアは必ず実行されます。CommandListの有無は描画の有無のみに影響します。

**エラーハンドリング**:
- 描画失敗: エラーログを出力し、該当エンティティをスキップ

**ログ出力**:
```rust
// 開始
tracing::info!("render_surface: Entity={:?}, has_command_list={}", entity, command_list.is_some());

// 成功
if command_list.is_some() {
    tracing::info!("Surface rendered with CommandList for Entity={:?}", entity);
} else {
    tracing::info!("Surface cleared (no CommandList) for Entity={:?}", entity);
}

// エラー
tracing::error!("Failed to render Surface for Entity={:?}: {:?}", entity, err);
```

### 4.3 render_window System の削除

**変更点**: `render_window`システムは不要となり、完全に削除します

**理由**:
- `render_surface`が`Option<&GraphicsCommandList>`で両方のケースを処理
- GraphicsCommandListの有無で処理を分ける必要がなくなった
- システムの重複を避け、設計がシンプルになる

**Phase 2-M3テストコードの扱い**:
- `render_window`システム全体を削除
- Phase 2-M3で追加した`render_shapes`および`create_triangle_geometry`も削除
- **参考実装として残す必要はない**（CommandListパイプラインに完全移行）

---

## 5. COM API Wrapper Extensions

### 5.1 D2D1FactoryExt

**実装場所**: `crates/wintf/src/com/d2d/mod.rs`

```rust
use windows::Win32::Graphics::Direct2D::{ID2D1Factory7, ID2D1CommandList};

pub trait D2D1FactoryExt {
    /// CommandListを作成
    fn create_command_list(&self) -> windows::core::Result<ID2D1CommandList>;
}

impl D2D1FactoryExt for ID2D1Factory7 {
    fn create_command_list(&self) -> windows::core::Result<ID2D1CommandList> {
        unsafe { self.CreateCommandList() }
    }
}
```

### 5.2 D2D1CommandListExt

**実装場所**: `crates/wintf/src/com/d2d/mod.rs`

```rust
use windows::Win32::Graphics::Direct2D::{ID2D1CommandList, ID2D1DeviceContext};

pub trait D2D1CommandListExt {
    /// CommandListを開いてDeviceContextを取得
    fn open(&self) -> windows::core::Result<ID2D1DeviceContext>;
    
    /// CommandListを閉じる
    fn close(&self) -> windows::core::Result<()>;
}

impl D2D1CommandListExt for ID2D1CommandList {
    fn open(&self) -> windows::core::Result<ID2D1DeviceContext> {
        unsafe { self.Open() }
    }
    
    fn close(&self) -> windows::core::Result<()> {
        unsafe { self.Close() }
    }
}
```

### 5.3 D2D1DeviceContextExt

**実装場所**: `crates/wintf/src/com/d2d/mod.rs`

```rust
use windows::Win32::Graphics::Direct2D::{
    ID2D1DeviceContext, ID2D1Image,
    D2D1_INTERPOLATION_MODE_LINEAR, D2D1_COMPOSITE_MODE_SOURCE_OVER,
};

pub trait D2D1DeviceContextExt {
    /// ImageをDeviceContextに描画
    fn draw_image(&self, image: &ID2D1Image) -> windows::core::Result<()>;
}

impl D2D1DeviceContextExt for ID2D1DeviceContext {
    fn draw_image(&self, image: &ID2D1Image) -> windows::core::Result<()> {
        unsafe {
            self.DrawImage(
                image,
                None, // target_offset
                None, // image_rectangle
                D2D1_INTERPOLATION_MODE_LINEAR,
                D2D1_COMPOSITE_MODE_SOURCE_OVER,
            );
            Ok(())
        }
    }
}
```

**モジュール構成**:
```rust
// crates/wintf/src/com/d2d/mod.rs

// 既存のExtension Trait
pub use self::device_context::D2D1DeviceContextExt;
pub use self::surface::D2D1SurfaceExt;

// 新規追加
pub trait D2D1FactoryExt { ... }
pub trait D2D1CommandListExt { ... }

// 既存のD2D1DeviceContextExtに draw_image() を追加
```

---

## 6. Schedule and System Registration

### 6.1 スケジュール構成

```
Startup Schedule
  ├── ensure_graphics_core
  ├── create_window_graphics
  ├── create_window_visual
  └── create_window_surface

Draw Schedule (CommandList生成)
  └── draw_rectangles         // 新規

Render Schedule (描画)
  ├── render_surface          // 新規: Option<&GraphicsCommandList>で統合
  └── commit_composition

Update Schedule
  └── (将来のイベント処理等)
```

### 6.2 システム登録

**変更点**:

```rust
// crates/wintf/src/ecs/world.rs

use crate::ecs::graphics::*;
use crate::ecs::widget::shapes::rectangle::draw_rectangles;

pub fn setup_schedules(world: &mut World) -> Schedules {
    let mut schedules = Schedules::new();
    
    // ... (Startup, Updateは既存のまま) ...
    
    // Draw Schedule
    let mut draw_schedule = Schedule::new(Draw);
    draw_schedule.add_systems(draw_rectangles);
    schedules.insert(draw_schedule);
    
    // Render Schedule
    let mut render_schedule = Schedule::new(Render);
    render_schedule.add_systems((
        render_surface,     // 統合版: Option<&GraphicsCommandList>
        commit_composition,
    ).chain());
    schedules.insert(render_schedule);
    
    schedules
}
```

---

## 7. Integration Test Design

### 7.1 simple_window.rs の更新

**実装場所**: `crates/wintf/examples/simple_window.rs`

**変更内容**:

1. **Rectangleコンポーネントの追加**:
```rust
use wintf::ecs::widget::shapes::{Rectangle, Color};

// Window 1: 赤い四角
commands.entity(window1_entity).insert(Rectangle {
    x: 100.0,
    y: 100.0,
    width: 200.0,
    height: 150.0,
    color: Color::RED,
});

// Window 2: 青い四角
commands.entity(window2_entity).insert(Rectangle {
    x: 150.0,
    y: 150.0,
    width: 180.0,
    height: 120.0,
    color: Color::BLUE,
});
```

2. **Phase 2-M3テストコードの削除**:
```rust
// 削除: render_windowシステムと関連コード（Phase 2-M3のテスト実装全体）
// render_surface（統合版）に完全移行
```

3. **コメント追加**:
```rust
// Widget描画の例:
// 1. WindowエンティティにRectangleコンポーネントを追加
// 2. draw_rectanglesシステムが自動的にGraphicsCommandListを生成
// 3. render_surfaceシステムがSurfaceに描画
// 4. commit_compositionで画面に表示
```

### 7.2 テスト観点

**手動確認項目**:
- ✅ 1つ目のウィンドウに赤い四角が表示される
- ✅ 2つ目のウィンドウに青い四角が表示される
- ✅ エラーなく実行される
- ✅ ログに`draw_rectangles`と`render_surface`の実行が記録される
- ✅ Rectangleの座標・サイズが正しく反映される

**コマンド**:
```bash
cargo run --example simple_window
```

---

## 8. Error Handling Strategy

### 8.1 エラー分類

| エラー種別 | 対応 | ログレベル |
|-----------|------|-----------|
| GraphicsCoreが存在しない | 処理全体をスキップ | WARN |
| COM API呼び出し失敗 | 該当エンティティをスキップ | ERROR |
| CommandList生成失敗 | 該当エンティティをスキップ | ERROR |
| 描画失敗 | 該当エンティティをスキップ | ERROR |

### 8.2 エラーハンドリングパターン

```rust
// GraphicsCoreチェック
let Some(graphics_core) = graphics_core else {
    tracing::warn!("GraphicsCore not available, skipping draw_rectangles");
    return;
};

// COM API呼び出し
match factory.create_command_list() {
    Ok(command_list) => {
        // 処理継続
    }
    Err(err) => {
        tracing::error!("Failed to create CommandList for Entity={:?}: {:?}", entity, err);
        continue; // 次のエンティティへ
    }
}
```

---

## 9. Non-Functional Requirements

### 9.1 Performance

- **CommandList生成頻度**: `Changed<Rectangle>`フィルターにより、変更時のみ生成
- **描画頻度**: `Changed<GraphicsCommandList>`または`Changed<Surface>`でトリガー
- **目標**: 60fps（16.67ms/frame）を維持
- **メモリ**: CommandListはCOMオブジェクトとして適切に参照カウント管理

### 9.2 Maintainability

- **モジュール分離**: widget、graphics、comで責務を明確化
- **拡張性**: 将来的なShape追加（Ellipse、Path等）が容易
- **テスタビリティ**: COM APIラッパーにより、テストコード作成が容易

### 9.3 Code Quality

- **unsafe使用**: COM APIラッパー層のみに限定
- **エラーハンドリング**: すべてのCOM API呼び出しでエラーチェック
- **ログ出力**: 開始・成功・失敗を適切にログ記録
- **ドキュメント**: 各Component、System、Traitにdocコメント付与

---

## 10. Implementation Checklist

### Phase 1: モジュール構造の準備
- [ ] `graphics.rs` → `graphics/`ディレクトリへの分割
  - [ ] `mod.rs`, `core.rs`, `components.rs`, `command_list.rs`, `systems.rs`作成
  - [ ] 既存コードの移動とre-export設定
  - [ ] 既存テストの実行確認
- [ ] `widget/`モジュールの作成
  - [ ] `widget/shapes/mod.rs`
  - [ ] `widget/shapes/rectangle.rs`

### Phase 2: コンポーネント実装
- [ ] `Rectangle`コンポーネント
  - [ ] 構造体定義（x, y, width, height, color）
  - [ ] `Color`型エイリアス定義（`type Color = D2D1_COLOR_F`）
  - [ ] `Color`定数定義（RED, BLUE等）
- [ ] `GraphicsCommandList`コンポーネント
  - [ ] 構造体定義（ID2D1CommandList保持）
  - [ ] アクセサメソッド
  - [ ] `Send`/`Sync`実装

### Phase 3: COM APIラッパー
- [ ] `D2D1FactoryExt`
  - [ ] `create_command_list()`メソッド
- [ ] `D2D1CommandListExt`
  - [ ] `open()`メソッド
  - [ ] `close()`メソッド
- [ ] `D2D1DeviceContextExt`
  - [ ] `draw_image()`メソッド

### Phase 4: システム実装
- [ ] `draw_rectangles`システム
  - [ ] Query定義（`Changed<Rectangle>`）
  - [ ] CommandList生成処理
  - [ ] エラーハンドリング
  - [ ] ログ出力
- [ ] `render_surface`システム（統合版）
  - [ ] Query定義（`Option<&GraphicsCommandList>`, `Or<Changed<...>>`）
  - [ ] クリア処理（常に実行）
  - [ ] DrawImage処理（GraphicsCommandListがある場合のみ）
  - [ ] エラーハンドリング
  - [ ] ログ出力
- [ ] `render_window`システムの削除
  - [ ] システム関数の削除
  - [ ] `render_shapes`, `create_triangle_geometry`の削除
  - [ ] スケジュール登録の削除

### Phase 5: スケジュール登録
- [ ] `Draw`スケジュールに`draw_rectangles`追加
- [ ] `Render`スケジュールに`render_surface`（統合版）追加
- [ ] `render_window`のスケジュール登録を削除
- [ ] システム実行順序の確認

### Phase 6: 統合テスト
- [ ] `simple_window.rs`の更新
  - [ ] Rectangle追加（赤・青）
  - [ ] Phase 2-M3テストコード削除（render_window関連すべて）
  - [ ] コメント追加
- [ ] 手動動作確認
  - [ ] 赤い四角の表示
  - [ ] 青い四角の表示
  - [ ] エラーログの確認
  - [ ] GraphicsCommandList削除時の動作確認（クリアのみ）
- [ ] ビルド確認（`cargo build`）
- [ ] 実行確認（`cargo run --example simple_window`）

---

## 11. Testing Strategy

### 11.1 Unit Testing

**対象外**: このマイルストーンでは、既存のシステムが正常動作していることを前提とし、新規ユニットテストは作成しません（要件Req 10に記載なし）

### 11.2 Integration Testing

**手動テスト**: `simple_window.rs`の実行による視覚的確認

**確認項目**:
1. 赤い四角がウィンドウ1に表示される
2. 青い四角がウィンドウ2に表示される
3. 位置・サイズが仕様通り
4. エラーログが出力されない

### 11.3 Regression Testing

**既存機能の確認**:
- `cargo build`が成功する
- `graphics.rs`のモジュール分割後も既存テストがパスする
- Phase 2-M1～M3の機能が正常動作する

---

## 12. Migration Path

### 12.1 Phase 2-M3からの変更点

| 項目 | Phase 2-M3 | Phase 2-M4 |
|------|-----------|-----------|
| 描画コンテンツ | 三角形（render_window内） | Rectangle（CommandList） |
| 描画システム | `render_window`のみ | `render_surface`（統合版）のみ |
| スケジュール | Render | Draw + Render |
| コンポーネント | Surface | Surface + Rectangle + GraphicsCommandList |
| システム統合 | なし | `Option<&GraphicsCommandList>`で有無を統合処理 |

### 12.2 将来の拡張ポイント

- **Entity階層構造**: ChildOf/Childrenを使用した親子関係
- **Visualツリー**: DirectCompositionのVisual階層と同期
- **他のShape**: Ellipse, Path, Text等の追加
- **レイアウトシステム**: taffyとの統合
- **イベント処理**: Phase 6での実装
- **色定義の拡張**: 色関係クレート（`css-colors`, `palette`等）の導入を検討
  - C++ Direct2Dの`ColorF`には140色以上の定数がある（AliceBlue, CornflowerBlue等）
  - 現状は基本色のみ（RED, BLUE, GREEN, TRANSPARENT）
  - 将来必要になれば、専用モジュール（`colors.rs`）で管理

---

## 13. Traceability Matrix

| Requirement | Design Section | Implementation Phase |
|-------------|---------------|---------------------|
| Req 1: Rectangle Component | 2.1 | Phase 2 |
| Req 2: GraphicsCommandList | 2.2 | Phase 2 |
| Req 3: graphics.rs Refactoring | 3.1 | Phase 1 |
| Req 4: draw_rectangles System | 4.1 | Phase 4 |
| Req 5: render_surface System | 4.2 (統合版) | Phase 4 |
| Req 6: render_window Deletion | 4.3 | Phase 4 |
| Req 7: System Unification | 4.2 (Option使用) | Phase 4 |
| Req 8: COM API Wrappers | 5 | Phase 3 |
| Req 9: Error Handling | 8 | Phase 4 |
| Req 10: Integration Test | 7 | Phase 6 |

---

## 14. References

- **Requirements**: `.kiro/specs/phase2-m4-first-widget/requirements.md`
- **Phase 2-M3**: `.kiro/specs/phase2-m3-first-rendering/`
- **Direct2D Command Lists**: https://learn.microsoft.com/en-us/windows/win32/direct2d/direct2d-command-lists
- **Bevy ECS**: https://docs.rs/bevy_ecs/

---

_Design phase completed. Ready for tasks phase._
