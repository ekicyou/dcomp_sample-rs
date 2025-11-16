# Tasks: Phase 2 Milestone 1 - GraphicsCore初期化

**Feature ID**: `phase2-m1-graphics-core`  
**Phase**: Phase 3 - Tasks  
**Updated**: 2025-11-14

---

## 📋 Task Overview

このマイルストーンを**5つの独立したタスク**に分解します。各タスクは順番に実行され、それぞれ独立してテスト可能です。

---

## 📦 Task List

### Task 1: import文の追加とGraphicsCore構造体の定義
**優先度**: 必須  
**所要時間**: 5分  
**ファイル**: `crates/wintf/src/ecs/graphics.rs`

#### 作業内容
1. 必要なimport文を追加
2. `GraphicsDevices`を`GraphicsCore`に名前変更
3. 新しいフィールド（`d2d_factory`, `dwrite_factory`）を追加
4. `unsafe impl`の名前を更新

#### 具体的な変更

```diff
 use crate::com::d2d::*;
 use crate::com::d3d11::*;
 use crate::com::dcomp::*;
+use crate::com::dwrite::*;
 use bevy_ecs::prelude::*;
 use windows::core::{Interface, Result};
 use windows::Win32::Foundation::*;
 use windows::Win32::Graphics::Direct2D::*;
+use windows::Win32::Graphics::Direct2D::Common::*;
 use windows::Win32::Graphics::Direct3D::*;
 use windows::Win32::Graphics::Direct3D11::*;
 use windows::Win32::Graphics::DirectComposition::*;
+use windows::Win32::Graphics::DirectWrite::*;
 use windows::Win32::Graphics::Dxgi::*;

 #[derive(Resource, Debug)]
-pub struct GraphicsDevices {
+pub struct GraphicsCore {
     pub d3d: ID3D11Device,
     pub dxgi: IDXGIDevice4,
+    pub d2d_factory: ID2D1Factory,
     pub d2d: ID2D1Device,
+    pub dwrite_factory: IDWriteFactory2,
     pub desktop: IDCompositionDesktopDevice,
     pub dcomp: IDCompositionDevice3,
 }

-unsafe impl Send for GraphicsDevices {}
-unsafe impl Sync for GraphicsDevices {}
+unsafe impl Send for GraphicsCore {}
+unsafe impl Sync for GraphicsCore {}
```

#### 受け入れ基準
- ✅ import文が追加されている
- ✅ 構造体名が`GraphicsCore`に変更されている
- ✅ `d2d_factory`フィールドが追加されている
- ✅ `dwrite_factory`フィールドが追加されている
- ✅ `Send`/`Sync`の実装が更新されている
- ✅ コンパイルエラーがない（後続のタスクで`new()`を修正するまでは実装ブロックでエラー）

---

### Task 2: create_d2d_factory()ヘルパー関数の追加
**優先度**: 必須  
**所要時間**: 5分  
**ファイル**: `crates/wintf/src/ecs/graphics.rs`  
**依存**: Task 1完了後

#### 作業内容
1. `create_d2d_factory()`関数を追加
2. マルチスレッドモードでD2DFactoryを作成

#### 具体的な変更

```rust
/// D2DFactoryを作成（マルチスレッド対応）
fn create_d2d_factory() -> Result<ID2D1Factory> {
    unsafe {
        D2D1CreateFactory::<ID2D1Factory>(
            D2D1_FACTORY_TYPE_MULTI_THREADED,
            None,
        )
    }
}
```

**追加位置**: `create_device_3d()`関数の直前

#### 受け入れ基準
- ✅ `create_d2d_factory()`関数が追加されている
- ✅ `D2D1_FACTORY_TYPE_MULTI_THREADED`を使用している
- ✅ 関数シグネチャが正しい（`Result<ID2D1Factory>`を返す）
- ✅ コンパイルエラーがない

---

### Task 3: create_device_3d()のデバッグフラグ追加
**優先度**: 必須  
**所要時間**: 3分  
**ファイル**: `crates/wintf/src/ecs/graphics.rs`  
**依存**: なし（Task 1と並行可能）

#### 作業内容
1. デバッグビルド時に`D3D11_CREATE_DEVICE_DEBUG`フラグを追加
2. `#[cfg(debug_assertions)]`で条件分岐

#### 具体的な変更

```diff
 fn create_device_3d() -> Result<ID3D11Device> {
+    #[cfg(debug_assertions)]
+    let flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_DEBUG;
+    
+    #[cfg(not(debug_assertions))]
+    let flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT;
+    
     d3d11_create_device(
         None,
         D3D_DRIVER_TYPE_HARDWARE,
         HMODULE::default(),
-        D3D11_CREATE_DEVICE_BGRA_SUPPORT,
+        flags,
         None,
         D3D11_SDK_VERSION,
         None,
         None,
     )
 }
```

#### 受け入れ基準
- ✅ デバッグビルド時にデバッグフラグが有効
- ✅ リリースビルド時はBGRAサポートのみ
- ✅ コンパイルエラーがない
- ✅ 既存の動作が維持されている

---

### Task 4: GraphicsCore::new()の実装更新
**優先度**: 必須  
**所要時間**: 10分  
**ファイル**: `crates/wintf/src/ecs/graphics.rs`  
**依存**: Task 1, 2完了後

#### 作業内容
1. `GraphicsDevices::new()`を`GraphicsCore::new()`に変更
2. D2DFactoryとDWriteFactoryの初期化を追加
3. 詳細なログ出力を追加
4. 構造体の返却時に新しいフィールドを含める

#### 具体的な変更

```diff
-impl GraphicsDevices {
+impl GraphicsCore {
     pub fn new() -> Result<Self> {
+        eprintln!("[GraphicsCore] 初期化開始");
+        
+        eprintln!("[GraphicsCore] D3D11Deviceを作成中...");
         let d3d = create_device_3d()?;
+        eprintln!("[GraphicsCore] D3D11Device作成完了");
+        
+        eprintln!("[GraphicsCore] IDXGIDevice4を取得中...");
         let dxgi = d3d.cast()?;
+        eprintln!("[GraphicsCore] IDXGIDevice4取得完了");
+        
+        eprintln!("[GraphicsCore] D2DFactoryを作成中...");
+        let d2d_factory = create_d2d_factory()?;
+        eprintln!("[GraphicsCore] D2DFactory作成完了");
+        
+        eprintln!("[GraphicsCore] D2DDeviceを作成中...");
         let d2d = d2d_create_device(&dxgi)?;
+        eprintln!("[GraphicsCore] D2DDevice作成完了");
+        
+        eprintln!("[GraphicsCore] DWriteFactoryを作成中...");
+        let dwrite_factory = dwrite_create_factory(DWRITE_FACTORY_TYPE_SHARED)?;
+        eprintln!("[GraphicsCore] DWriteFactory作成完了");
+        
+        eprintln!("[GraphicsCore] DCompositionDesktopDeviceを作成中...");
         let desktop = dcomp_create_desktop_device(&d2d)?;
+        eprintln!("[GraphicsCore] DCompositionDesktopDevice作成完了");
+        
+        eprintln!("[GraphicsCore] IDCompositionDevice3を取得中...");
         let dcomp: IDCompositionDevice3 = desktop.cast()?;
+        eprintln!("[GraphicsCore] IDCompositionDevice3取得完了");
+        
+        eprintln!("[GraphicsCore] 初期化完了");
+        
         Ok(Self {
             d3d,
             dxgi,
+            d2d_factory,
             d2d,
+            dwrite_factory,
             desktop,
             dcomp,
         })
     }
 }
```

#### 受け入れ基準
- ✅ `impl`ブロックの名前が`GraphicsCore`に変更されている
- ✅ `d2d_factory`と`dwrite_factory`の初期化が追加されている
- ✅ 初期化順序が正しい（設計書の順序に従う）
- ✅ すべてのログメッセージが追加されている
- ✅ 構造体の返却時に7つすべてのフィールドが含まれている
- ✅ コンパイルエラーがない

---

### Task 5: ensure_graphics_core()システムの更新とECS統合
**優先度**: 必須  
**所要時間**: 10分  
**ファイル**: `crates/wintf/src/ecs/graphics.rs`, `crates/wintf/src/ecs/world.rs`  
**依存**: Task 4完了後

#### 作業内容

**5-1: graphics.rsの更新**
1. `ensure_graphics_devices`を`ensure_graphics_core`に名前変更
2. 引数の型を`GraphicsCore`に変更
3. ログメッセージを日本語化
4. エラー時に`panic!`を追加

```diff
-/// GraphicsDevicesが存在しない場合に作成するシステム
-pub fn ensure_graphics_devices(devices: Option<Res<GraphicsDevices>>, mut commands: Commands) {
-    if devices.is_none() {
-        match GraphicsDevices::new() {
+/// GraphicsCoreが存在しない場合に作成するシステム
+pub fn ensure_graphics_core(graphics: Option<Res<GraphicsCore>>, mut commands: Commands) {
+    if graphics.is_none() {
+        eprintln!("[System] GraphicsCore初期化を開始");
+        
+        match GraphicsCore::new() {
             Ok(graphics) => {
                 commands.insert_resource(graphics);
-                eprintln!("Graphics devices created successfully");
+                eprintln!("[System] GraphicsCoreをECSリソースとして登録完了");
             }
             Err(e) => {
-                eprintln!("Failed to create graphics devices: {:?}", e);
+                eprintln!("[System] GraphicsCore初期化失敗: {:?}", e);
+                panic!("GraphicsCoreの初期化に失敗しました。アプリケーションを終了します。");
             }
         }
     }
 }
```

**5-2: world.rsの更新**
1. `UISetup`スケジュールに`ensure_graphics_core`を登録
2. `create_windows`より前に実行されるよう`.before()`を使用

```diff
         // デフォルトシステムの登録
         {
             let mut schedules = world.resource_mut::<Schedules>();
+            schedules.add_systems(
+                UISetup, 
+                crate::ecs::graphics::ensure_graphics_core
+                    .before(crate::ecs::window_system::create_windows)
+            );
             schedules.add_systems(UISetup, crate::ecs::window_system::create_windows);
             // on_window_handle_addedとon_window_handle_removedはフックで代替
         }
```

#### 受け入れ基準
- ✅ システム名が`ensure_graphics_core`に変更されている
- ✅ 引数の型が`Option<Res<GraphicsCore>>`に変更されている
- ✅ ログメッセージが日本語化されている
- ✅ エラー時に`panic!`が呼ばれている
- ✅ `world.rs`にシステム登録が追加されている
- ✅ `.before(create_windows)`が指定されている
- ✅ コンパイルエラーがない

---

## 🧪 Testing Plan

### Task完了後のテスト

各タスク完了後に以下を確認：

1. **コンパイルテスト**
   ```bash
   cargo build
   ```
   - すべてのタスク完了後にコンパイルが成功すること

2. **実行テスト**
   ```bash
   cargo run --example areka
   ```
   - アプリケーションが正常に起動すること
   - 初期化ログが表示されること
   - ウィンドウが表示されること

3. **ログ確認**
   期待されるログ出力：
   ```
   [GraphicsCore] 初期化開始
   [GraphicsCore] D3D11Deviceを作成中...
   [GraphicsCore] D3D11Device作成完了
   [GraphicsCore] IDXGIDevice4を取得中...
   [GraphicsCore] IDXGIDevice4取得完了
   [GraphicsCore] D2DFactoryを作成中...
   [GraphicsCore] D2DFactory作成完了
   [GraphicsCore] D2DDeviceを作成中...
   [GraphicsCore] D2DDevice作成完了
   [GraphicsCore] DWriteFactoryを作成中...
   [GraphicsCore] DWriteFactory作成完了
   [GraphicsCore] DCompositionDesktopDeviceを作成中...
   [GraphicsCore] DCompositionDesktopDevice作成完了
   [GraphicsCore] IDCompositionDevice3を取得中...
   [GraphicsCore] IDCompositionDevice3取得完了
   [GraphicsCore] 初期化完了
   [System] GraphicsCore初期化を開始
   [System] GraphicsCoreをECSリソースとして登録完了
   ```

4. **デバッグビルドテスト**
   ```bash
   cargo build
   cargo run --example areka
   ```
   - デバッグレイヤーが有効になっていること（追加のログが出る可能性）

5. **リリースビルドテスト**
   ```bash
   cargo build --release
   cargo run --release --example areka
   ```
   - リリースビルドでも正常に動作すること

---

## 📊 Task Dependencies

```
Task 1: 構造体定義
  │
  ├─> Task 2: create_d2d_factory()追加
  │     │
  │     └─> Task 4: GraphicsCore::new()更新
  │           │
  └─> Task 3: create_device_3d()更新
        │
        └─> Task 5: システム更新とECS統合
              (全タスク完了後)
```

**並行作業可能**:
- Task 1とTask 3は並行実施可能

**順次実行必須**:
- Task 2はTask 1完了後
- Task 4はTask 1, 2完了後
- Task 5はTask 4完了後

---

## 🎯 Implementation Strategy

### 推奨実装順序

1. **Task 1**: まず構造体定義を完了（これがベース）
2. **Task 3**: デバッグフラグ追加（独立したタスク）
3. **Task 2**: D2DFactory関数追加（Task 4の準備）
4. **Task 4**: new()メソッドの大幅更新（コアロジック）
5. **Task 5**: ECS統合（最終統合）

### 各タスクのチェックポイント

#### Task 1チェックポイント
```bash
# コンパイルエラーが出るが、それは予想通り（new()がまだ古い実装）
cargo check
```

#### Task 2チェックポイント
```bash
# 新しい関数が追加されたことを確認
cargo check
```

#### Task 3チェックポイント
```bash
# 既存機能が壊れていないことを確認
cargo build
cargo run --example areka  # 動作確認
```

#### Task 4チェックポイント
```bash
# すべてのコンパイルエラーが解消されることを確認
cargo build
```

#### Task 5チェックポイント
```bash
# 統合テスト
cargo build
cargo run --example areka
# ログ出力を確認
```

---

## ⚠️ 注意事項

### 実装時の注意

1. **型名の一貫性**
   - すべての`GraphicsDevices`を`GraphicsCore`に変更
   - `Res<GraphicsDevices>`も`Res<GraphicsCore>`に変更

2. **フィールド順序**
   - 構造体のフィールド順序は設計書通りに維持
   - `d2d_factory`は`dxgi`と`d2d`の間
   - `dwrite_factory`は`d2d`と`desktop`の間

3. **初期化順序**
   - `GraphicsCore::new()`の初期化順序は厳密に守る
   - 依存関係を無視すると実行時エラーが発生

4. **ログメッセージ**
   - すべてのログは日本語で記述
   - プレフィックスは`[GraphicsCore]`または`[System]`

5. **エラーハンドリング**
   - `?`演算子で早期リターン
   - システムでのエラーは`panic!`で終了

---

## 📚 References

- [Design Document](./DESIGN.md)
- [Requirements Document](./REQUIREMENTS.md)
- [Existing Implementation](../../../crates/wintf/src/ecs/graphics.rs)

---

## ✅ Completion Criteria

すべてのタスクが完了し、以下の条件を満たしていること：

### 必須条件
1. ✅ すべてのタスクが完了している
2. ✅ コンパイルエラーがない（`cargo build`が成功）
3. ✅ サンプルアプリが起動する（`cargo run --example areka`）
4. ✅ 初期化ログが正しく表示される
5. ✅ デバッグビルドとリリースビルドの両方が動作する

### 品質条件
6. ✅ 既存の機能が壊れていない（後方互換性）
7. ✅ すべてのフィールドが正しく初期化されている
8. ✅ メモリリークがない（COM APIの適切な管理）
9. ✅ エラー時に適切なメッセージが表示される
10. ✅ パフォーマンス要件を満たしている（100ms以内の初期化）

---

## 🎯 Next Steps

すべてのタスク完了後:

```bash
/kiro-spec-impl phase2-m1-graphics-core
```

または個別タスクの実装:

```bash
/kiro-spec-impl phase2-m1-graphics-core task1
/kiro-spec-impl phase2-m1-graphics-core task2
# ... 以下同様
```

---

_Phase 3 (Tasks) completed. Ready for implementation phase._
