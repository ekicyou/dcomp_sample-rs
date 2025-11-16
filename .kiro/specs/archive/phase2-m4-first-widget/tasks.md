# Tasks: Phase 2 Milestone 4 - 初めてのウィジット

**Feature ID**: `phase2-m4-first-widget`  
**Version**: 1.0  
**Last Updated**: 2025-11-14  
**Status**: Phase 3 - Tasks

---

## タスク概要

Phase 2-M4の実装を6つのPhaseに分けて実行します。各タスクは独立して実行可能で、テスト可能です。

**実装順序の重要性**:
1. Phase 1（モジュール構造）は他のすべてのタスクの前提
2. Phase 2-3（コンポーネント・ラッパー）は並行実施可能
3. Phase 4（システム）はPhase 2-3完了後に実施
4. Phase 5（スケジュール）はPhase 4完了後に実施
5. Phase 6（統合テスト）は全Phase完了後に実施

---

## Phase 1: モジュール構造の準備

### Task 1.1: graphics.rsのディレクトリ化

**目的**: graphics.rsを責務ごとにモジュール分割し、保守性を向上させる

**作業内容**:
1. `crates/wintf/src/ecs/graphics/`ディレクトリ作成
2. 既存の`graphics.rs`をバックアップ（念のため）
3. 以下のファイルを作成:
   - `graphics/mod.rs` - 公開API + Re-exports
   - `graphics/core.rs` - GraphicsCoreリソースとensure_graphics_core
   - `graphics/components.rs` - WindowGraphics, Visual, Surface
   - `graphics/command_list.rs` - 空ファイル（後で実装）
   - `graphics/systems.rs` - 描画システム群
4. 既存コードを各ファイルに移動:
   - `GraphicsCore` + `ensure_graphics_core` → `core.rs`
   - `WindowGraphics`, `Visual`, `Surface` → `components.rs`
   - 描画システム関数 → `systems.rs`
5. `mod.rs`でre-export設定:
   ```rust
   mod core;
   mod components;
   mod command_list;
   mod systems;
   
   pub use core::*;
   pub use components::*;
   pub use command_list::*;
   pub use systems::*;
   ```
6. 既存の`graphics.rs`削除

**検証**:
```bash
cargo build
# エラーなくビルドが通ること
```

**完了条件**:
- [ ] graphics/ディレクトリに5ファイル作成
- [ ] 既存コードが各ファイルに適切に配置
- [ ] `use crate::ecs::graphics::*;`で既存と同じくimport可能
- [ ] ビルドエラーなし

**推定時間**: 30分

---

### Task 1.2: widget/モジュールの作成

**目的**: Widget関連コードの配置場所を確立する

**作業内容**:
1. `crates/wintf/src/ecs/widget/`ディレクトリ作成
2. 以下のファイルを作成:
   - `widget/mod.rs` - モジュール定義
   - `widget/shapes/mod.rs` - Shapes配下のモジュール
   - `widget/shapes/rectangle.rs` - 空ファイル（次Phaseで実装）
3. `widget/mod.rs`に記述:
   ```rust
   pub mod shapes;
   ```
4. `widget/shapes/mod.rs`に記述:
   ```rust
   pub mod rectangle;
   pub use rectangle::*;
   ```
5. `ecs/mod.rs`にwidgetモジュール追加:
   ```rust
   pub mod widget;
   ```

**検証**:
```bash
cargo build
# エラーなくビルドが通ること
```

**完了条件**:
- [ ] widget/shapes/ディレクトリ作成
- [ ] 3ファイル作成（mod.rs x2, rectangle.rs）
- [ ] モジュール階層が正しく設定
- [ ] ビルドエラーなし

**推定時間**: 15分

---

## Phase 2: コンポーネント実装

### Task 2.1: Rectangleコンポーネントの実装

**目的**: 四角形の描画情報を保持するコンポーネントを実装

**作業内容**:
1. `crates/wintf/src/ecs/widget/shapes/rectangle.rs`に以下を実装:

```rust
use bevy_ecs::component::Component;
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;

/// 色の型エイリアス（D2D1_COLOR_Fをそのまま使用）
pub type Color = D2D1_COLOR_F;

/// 基本色定義
impl Color {
    /// 透明色
    pub const TRANSPARENT: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    /// 黒
    pub const BLACK: Color = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    /// 白
    pub const WHITE: Color = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    /// 赤
    pub const RED: Color = Color { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    /// 緑
    pub const GREEN: Color = Color { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    /// 青
    pub const BLUE: Color = Color { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
}

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
```

**検証**:
```bash
cargo build
# Rectangleコンポーネントがビルドできること
```

**完了条件**:
- [ ] Rectangle構造体定義
- [ ] Component, Debug, Cloneトレイト派生
- [ ] Color型エイリアス定義
- [ ] 6つの基本色定数定義
- [ ] ビルドエラーなし

**推定時間**: 15分

---

### Task 2.2: GraphicsCommandListコンポーネントの実装

**目的**: Direct2Dの描画命令を保持するコンポーネントを実装

**作業内容**:
1. `crates/wintf/src/ecs/graphics/command_list.rs`に以下を実装:

```rust
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

**検証**:
```bash
cargo build
# GraphicsCommandListコンポーネントがビルドできること
```

**完了条件**:
- [ ] GraphicsCommandList構造体定義
- [ ] Component, Debugトレイト派生
- [ ] new()メソッド実装
- [ ] command_list()アクセサメソッド実装
- [ ] Send/Sync実装（unsafe impl）
- [ ] ビルドエラーなし

**推定時間**: 15分

---

## Phase 3: COM APIラッパーの拡張

### Task 3.1: D2D1FactoryExtの実装

**目的**: ID2D1Factory7にCommandList生成機能を追加

**作業内容**:
1. `crates/wintf/src/com/d2d/mod.rs`に以下を追加:

```rust
use windows::Win32::Graphics::Direct2D::{ID2D1Factory7, ID2D1CommandList};

/// ID2D1Factory7の拡張トレイト
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

**検証**:
```bash
cargo build
# D2D1FactoryExtトレイトがビルドできること
```

**完了条件**:
- [ ] D2D1FactoryExtトレイト定義
- [ ] create_command_list()メソッド実装
- [ ] unsafeブロックで正しくCOM API呼び出し
- [ ] ビルドエラーなし

**推定時間**: 10分

---

### Task 3.2: D2D1CommandListExtの実装

**目的**: ID2D1CommandListに開閉機能を追加

**作業内容**:
1. `crates/wintf/src/com/d2d/mod.rs`に以下を追加:

```rust
use windows::Win32::Graphics::Direct2D::{ID2D1CommandList, ID2D1DeviceContext};

/// ID2D1CommandListの拡張トレイト
pub trait D2D1CommandListExt {
    /// CommandListを開いてDeviceContextを取得
    fn open(&self) -> windows::core::Result<ID2D1DeviceContext>;
    
    /// CommandListを閉じる
    fn close(&self) -> windows::core::Result<()>;
}

impl D2D1CommandListExt for ID2D1CommandList {
    fn open(&self) -> windows::core::Result<ID2D1DeviceContext> {
        unsafe { 
            let mut dc = None;
            self.Open(&mut dc)?;
            dc.ok_or_else(|| windows::core::Error::from_win32())
        }
    }
    
    fn close(&self) -> windows::core::Result<()> {
        unsafe { self.Close() }
    }
}
```

**検証**:
```bash
cargo build
# D2D1CommandListExtトレイトがビルドできること
```

**完了条件**:
- [ ] D2D1CommandListExtトレイト定義
- [ ] open()メソッド実装
- [ ] close()メソッド実装
- [ ] unsafeブロックで正しくCOM API呼び出し
- [ ] ビルドエラーなし

**推定時間**: 15分

---

### Task 3.3: D2D1DeviceContextExtのdraw_image追加

**目的**: ID2D1DeviceContextにImage描画機能を追加

**作業内容**:
1. `crates/wintf/src/com/d2d/mod.rs`の既存`D2D1DeviceContextExt`に以下を追加:

```rust
use windows::Win32::Graphics::Direct2D::{
    ID2D1DeviceContext, ID2D1Image,
    D2D1_INTERPOLATION_MODE_LINEAR, D2D1_COMPOSITE_MODE_SOURCE_OVER,
};

// 既存のD2D1DeviceContextExtトレイトに追加
pub trait D2D1DeviceContextExt {
    // ... 既存メソッド ...
    
    /// ImageをDeviceContextに描画
    fn draw_image(&self, image: &ID2D1Image) -> windows::core::Result<()>;
}

impl D2D1DeviceContextExt for ID2D1DeviceContext {
    // ... 既存メソッド実装 ...
    
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

**検証**:
```bash
cargo build
# draw_image()メソッドがビルドできること
```

**完了条件**:
- [ ] draw_image()メソッド定義
- [ ] unsafeブロックで正しくCOM API呼び出し
- [ ] 適切な補間モード・合成モード設定
- [ ] ビルドエラーなし

**推定時間**: 10分

---

## Phase 4: システム実装

### Task 4.1: draw_rectanglesシステムの実装

**目的**: Rectangle変更時にCommandListを自動生成するシステム

**作業内容**:
1. `crates/wintf/src/ecs/widget/shapes/rectangle.rs`に以下を追加:

```rust
use bevy_ecs::prelude::*;
use crate::ecs::graphics::{GraphicsCore, GraphicsCommandList};
use crate::com::d2d::{D2D1FactoryExt, D2D1CommandListExt, D2D1DeviceContextExt};
use windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F;

/// RectangleコンポーネントからGraphicsCommandListを生成
pub fn draw_rectangles(
    mut commands: Commands,
    query: Query<(Entity, &Rectangle), Changed<Rectangle>>,
    graphics_core: Option<Res<GraphicsCore>>,
) {
    let Some(graphics_core) = graphics_core else {
        tracing::warn!("GraphicsCore not available, skipping draw_rectangles");
        return;
    };

    for (entity, rectangle) in query.iter() {
        tracing::info!("draw_rectangles: Entity={:?}", entity);
        tracing::debug!(
            "Rectangle: x={}, y={}, width={}, height={}, color=({},{},{},{})",
            rectangle.x, rectangle.y, rectangle.width, rectangle.height,
            rectangle.color.r, rectangle.color.g, rectangle.color.b, rectangle.color.a
        );

        // CommandList生成
        let command_list = match graphics_core.d2d_factory().create_command_list() {
            Ok(cl) => cl,
            Err(err) => {
                tracing::error!("Failed to create CommandList for Entity={:?}: {:?}", entity, err);
                continue;
            }
        };

        // CommandListを開く
        let dc = match command_list.open() {
            Ok(dc) => dc,
            Err(err) => {
                tracing::error!("Failed to open CommandList for Entity={:?}: {:?}", entity, err);
                continue;
            }
        };

        // 描画命令を記録
        unsafe {
            if let Err(err) = dc.BeginDraw() {
                tracing::error!("BeginDraw failed for Entity={:?}: {:?}", entity, err);
                continue;
            }

            // 透明色クリア
            dc.Clear(Some(&Color::TRANSPARENT));

            // 四角形描画
            let rect = D2D_RECT_F {
                left: rectangle.x,
                top: rectangle.y,
                right: rectangle.x + rectangle.width,
                bottom: rectangle.y + rectangle.height,
            };

            // ソリッドカラーブラシ作成
            let brush = match dc.CreateSolidColorBrush(&rectangle.color, None) {
                Ok(b) => b,
                Err(err) => {
                    tracing::error!("Failed to create brush for Entity={:?}: {:?}", entity, err);
                    let _ = dc.EndDraw(None, None);
                    continue;
                }
            };

            dc.FillRectangle(&rect, &brush);

            if let Err(err) = dc.EndDraw(None, None) {
                tracing::error!("EndDraw failed for Entity={:?}: {:?}", entity, err);
                continue;
            }
        }

        // CommandListを閉じる
        if let Err(err) = command_list.close() {
            tracing::error!("Failed to close CommandList for Entity={:?}: {:?}", entity, err);
            continue;
        }

        // GraphicsCommandListコンポーネントを挿入
        commands.entity(entity).insert(GraphicsCommandList::new(command_list));
        tracing::info!("CommandList created for Entity={:?}", entity);
    }
}
```

**検証**:
```bash
cargo build
# draw_rectanglesシステムがビルドできること
```

**完了条件**:
- [ ] システム関数定義
- [ ] Query定義（Changed<Rectangle>）
- [ ] GraphicsCoreチェック
- [ ] CommandList生成処理
- [ ] 描画命令記録（Clear, FillRectangle）
- [ ] エラーハンドリング
- [ ] ログ出力（開始、成功、エラー）
- [ ] ビルドエラーなし

**推定時間**: 45分

---

### Task 4.2: render_surfaceシステムの実装（統合版）

**目的**: GraphicsCommandListの有無に応じてSurfaceを描画する統合システム

**作業内容**:
1. `crates/wintf/src/ecs/graphics/systems.rs`に以下を追加:

```rust
use crate::ecs::graphics::{GraphicsCore, GraphicsCommandList, Surface};
use crate::com::d2d::{D2D1SurfaceExt, D2D1DeviceContextExt};
use bevy_ecs::prelude::*;
use windows::Win32::Graphics::Direct2D::Common::D2D1_COLOR_F;

/// Surfaceへの描画（GraphicsCommandListの有無を統合処理）
pub fn render_surface(
    query: Query<
        (Entity, Option<&GraphicsCommandList>, &Surface),
        Or<(Changed<GraphicsCommandList>, Changed<Surface>)>
    >,
    graphics_core: Option<Res<GraphicsCore>>,
) {
    let Some(_graphics_core) = graphics_core else {
        tracing::warn!("GraphicsCore not available, skipping render_surface");
        return;
    };

    for (entity, command_list, surface) in query.iter() {
        tracing::info!(
            "render_surface: Entity={:?}, has_command_list={}",
            entity,
            command_list.is_some()
        );

        // Surface描画開始
        let dc = match surface.begin_draw() {
            Ok(dc) => dc,
            Err(err) => {
                tracing::error!("Failed to begin draw for Entity={:?}: {:?}", entity, err);
                continue;
            }
        };

        unsafe {
            // 透明色クリア（常に実行）
            dc.Clear(Some(&D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.0 }));

            // CommandListがある場合のみ描画
            if let Some(command_list) = command_list {
                if let Err(err) = dc.draw_image(command_list.command_list().cast().as_ref().unwrap()) {
                    tracing::error!("Failed to draw image for Entity={:?}: {:?}", entity, err);
                    let _ = dc.EndDraw(None, None);
                    let _ = surface.end_draw();
                    continue;
                }
            }

            if let Err(err) = dc.EndDraw(None, None) {
                tracing::error!("EndDraw failed for Entity={:?}: {:?}", entity, err);
                let _ = surface.end_draw();
                continue;
            }
        }

        // Surface描画終了
        if let Err(err) = surface.end_draw() {
            tracing::error!("Failed to end draw for Entity={:?}: {:?}", entity, err);
            continue;
        }

        if command_list.is_some() {
            tracing::info!("Surface rendered with CommandList for Entity={:?}", entity);
        } else {
            tracing::info!("Surface cleared (no CommandList) for Entity={:?}", entity);
        }
    }
}
```

**検証**:
```bash
cargo build
# render_surfaceシステムがビルドできること
```

**完了条件**:
- [ ] システム関数定義
- [ ] Query定義（Option<&GraphicsCommandList>, Or<Changed>）
- [ ] GraphicsCoreチェック
- [ ] Surface描画開始
- [ ] 透明色クリア（常に実行）
- [ ] CommandList描画（Someの場合のみ）
- [ ] Surface描画終了
- [ ] エラーハンドリング
- [ ] ログ出力（has_command_list情報含む）
- [ ] ビルドエラーなし

**推定時間**: 45分

---

### Task 4.3: render_windowシステムの削除

**目的**: Phase 2-M3のテスト実装を削除し、統合版に移行

**作業内容**:
1. `crates/wintf/src/ecs/graphics/systems.rs`から以下を削除:
   - `render_window`システム関数
   - `render_shapes`ヘルパー関数
   - `create_triangle_geometry`ヘルパー関数

**検証**:
```bash
cargo build
# 削除後もビルドエラーが発生しないこと（スケジュール登録は次Phaseで削除）
```

**完了条件**:
- [ ] render_window関数削除
- [ ] render_shapes関数削除
- [ ] create_triangle_geometry関数削除
- [ ] ビルドエラーなし（警告は許容）

**推定時間**: 10分

---

## Phase 5: スケジュール登録

### Task 5.1: Drawスケジュールにdraw_rectangles追加

**目的**: Rectangle変更時にCommandList生成を実行

**作業内容**:
1. `crates/wintf/src/ecs/world.rs`の`setup_schedules`関数を修正:

```rust
use crate::ecs::widget::shapes::rectangle::draw_rectangles;

// Drawスケジュールを作成
let mut draw_schedule = Schedule::new(Draw);
draw_schedule.add_systems(draw_rectangles);
schedules.insert(draw_schedule);
```

2. `Draw`スケジュールの定義を確認（なければ追加）:
```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ScheduleLabel)]
pub struct Draw;
```

**検証**:
```bash
cargo build
# Drawスケジュールが正しく登録されること
```

**完了条件**:
- [ ] Drawスケジュール作成
- [ ] draw_rectanglesシステム登録
- [ ] ScheduleLabelの定義確認
- [ ] ビルドエラーなし

**推定時間**: 15分

---

### Task 5.2: Renderスケジュールの更新

**目的**: render_surfaceを登録し、render_windowを削除

**作業内容**:
1. `crates/wintf/src/ecs/world.rs`の`setup_schedules`関数を修正:

```rust
// Renderスケジュール
let mut render_schedule = Schedule::new(Render);
render_schedule.add_systems((
    render_surface,     // 統合版: Option<&GraphicsCommandList>
    commit_composition,
).chain());
schedules.insert(render_schedule);
```

2. `render_window`のimportと登録を削除

**検証**:
```bash
cargo build
# Renderスケジュールが正しく更新されること
```

**完了条件**:
- [ ] render_surface登録
- [ ] render_window削除
- [ ] システム実行順序確認（render_surface → commit_composition）
- [ ] ビルドエラーなし

**推定時間**: 10分

---

### Task 5.3: スケジュール実行順序の確認

**目的**: 各スケジュールが正しい順序で実行されることを確認

**作業内容**:
1. `crates/wintf/src/ecs/app.rs`（またはメインループ）でスケジュール実行順序を確認:
   - Startup → Update → Draw → Render

2. 必要に応じてログ出力を追加して実行順序を検証

**検証**:
```bash
cargo run --example simple_window
# ログでスケジュール実行順序を確認
```

**完了条件**:
- [ ] スケジュール実行順序がStartup → Update → Draw → Renderであること
- [ ] 実行確認済み

**推定時間**: 10分

---

## Phase 6: 統合テストと動作確認

### Task 6.1: simple_window.rsの更新

**目的**: Rectangleコンポーネントを使用した動作確認

**作業内容**:
1. `crates/wintf/examples/simple_window.rs`を編集:

```rust
use wintf::ecs::widget::shapes::{Rectangle, Color};

// Window 1にRectangle追加
world.entity_mut(window1_entity).insert(Rectangle {
    x: 100.0,
    y: 100.0,
    width: 200.0,
    height: 150.0,
    color: Color::RED,
});

// Window 2にRectangle追加
world.entity_mut(window2_entity).insert(Rectangle {
    x: 150.0,
    y: 150.0,
    width: 180.0,
    height: 120.0,
    color: Color::BLUE,
});
```

2. Phase 2-M3のテストコード削除:
   - Surface検証ログ
   - render_window関連のコメント

3. Widget描画の説明コメント追加:
```rust
// Widget描画の例:
// 1. WindowエンティティにRectangleコンポーネントを追加
// 2. draw_rectanglesシステムが自動的にGraphicsCommandListを生成
// 3. render_surfaceシステムがSurfaceに描画
// 4. commit_compositionで画面に表示
```

**検証**:
```bash
cargo build
# ビルドエラーなし
```

**完了条件**:
- [ ] Rectangle追加（赤・青）
- [ ] Phase 2-M3テストコード削除
- [ ] コメント追加
- [ ] ビルドエラーなし

**推定時間**: 20分

---

### Task 6.2: ビルド確認

**目的**: 全体がエラーなくビルドできることを確認

**作業内容**:
```bash
cargo clean
cargo build
```

**完了条件**:
- [ ] ビルドエラーなし
- [ ] 警告の確認（重大な警告がないこと）

**推定時間**: 5分

---

### Task 6.3: 実行確認と動作テスト

**目的**: Widget描画が正しく動作することを視覚的に確認

**作業内容**:
```bash
cargo run --example simple_window
```

**確認項目**:
1. ✅ 1つ目のウィンドウに赤い四角が表示される
2. ✅ 2つ目のウィンドウに青い四角が表示される
3. ✅ 位置・サイズが仕様通り（Window 1: 100,100から200x150、Window 2: 150,150から180x120）
4. ✅ エラーログが出力されない
5. ✅ 以下のログが確認できる:
   - `draw_rectangles: Entity=...`
   - `CommandList created for Entity=...`
   - `render_surface: Entity=..., has_command_list=true`
   - `Surface rendered with CommandList for Entity=...`

**完了条件**:
- [ ] 赤い四角の表示確認
- [ ] 青い四角の表示確認
- [ ] 位置・サイズの確認
- [ ] エラーログなし
- [ ] 期待されるログ出力確認

**推定時間**: 15分

---

### Task 6.4: GraphicsCommandList削除時の動作確認（オプション）

**目的**: CommandList削除時にクリアのみが実行されることを確認

**作業内容**:
1. `simple_window.rs`に以下のテストコードを一時追加:

```rust
// Rectangle削除テスト
use std::time::Duration;
std::thread::sleep(Duration::from_secs(2));
world.entity_mut(window1_entity).remove::<Rectangle>();
world.run_schedule(Draw);
world.run_schedule(Render);
```

2. 実行して動作確認:
   - Window 1が透明になる（クリアのみ実行）
   - ログに`Surface cleared (no CommandList) for Entity=...`が出力される

3. テストコード削除

**完了条件**:
- [ ] Rectangle削除時にクリアのみ実行確認
- [ ] ログ確認
- [ ] テストコード削除

**推定時間**: 15分（オプション）

---

## タスク完了チェックリスト

### Phase 1: モジュール構造（45分）
- [ ] Task 1.1: graphics.rsディレクトリ化
- [ ] Task 1.2: widget/モジュール作成

### Phase 2: コンポーネント（30分）
- [ ] Task 2.1: Rectangle実装
- [ ] Task 2.2: GraphicsCommandList実装

### Phase 3: COM APIラッパー（35分）
- [ ] Task 3.1: D2D1FactoryExt実装
- [ ] Task 3.2: D2D1CommandListExt実装
- [ ] Task 3.3: D2D1DeviceContextExt拡張

### Phase 4: システム（110分）
- [ ] Task 4.1: draw_rectangles実装
- [ ] Task 4.2: render_surface実装（統合版）
- [ ] Task 4.3: render_window削除

### Phase 5: スケジュール（35分）
- [ ] Task 5.1: Drawスケジュール登録
- [ ] Task 5.2: Renderスケジュール更新
- [ ] Task 5.3: 実行順序確認

### Phase 6: 統合テスト（55分）
- [ ] Task 6.1: simple_window.rs更新
- [ ] Task 6.2: ビルド確認
- [ ] Task 6.3: 実行確認
- [ ] Task 6.4: CommandList削除テスト（オプション）

**合計推定時間**: 約5時間（オプション除く: 約4.5時間）

---

## 実装ガイドライン

### エラーハンドリング
- すべてのCOM API呼び出しで`Result`をチェック
- エラー時はログ出力してcontinue（パニックしない）
- Entity IDをログに含める

### ログレベル
- `info`: システム開始・完了
- `debug`: 詳細情報（Rectangle座標等）
- `warn`: GraphicsCore未初期化
- `error`: COM APIエラー

### テスト方針
- 各Phaseごとにビルド確認
- Phase 6で統合テスト
- 視覚的確認を重視

---

## トラブルシューティング

### ビルドエラー
1. モジュールパスの確認（`use`文）
2. トレイト実装の確認
3. `cargo clean`して再ビルド

### 実行時エラー
1. GraphicsCore初期化確認
2. ログ出力で処理フローを追跡
3. COM APIのHRESULT確認

### 描画されない
1. Rectangle座標がウィンドウ内か確認
2. Colorが正しいか確認（透明になっていないか）
3. CommandList生成ログを確認
4. render_surfaceログを確認

---

_Tasks phase completed. Ready for implementation phase._
