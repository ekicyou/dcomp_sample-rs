# Design: Phase 2 Milestone 1 - GraphicsCore初期化

**Feature ID**: `phase2-m1-graphics-core`  
**Phase**: Phase 2 - Design  
**Updated**: 2025-11-14

---

## 📐 Design Overview

### アーキテクチャ概要
既存の`GraphicsDevices`構造体を`GraphicsCore`に改造し、D2DFactoryとDWriteFactoryを追加することで、Phase 2の描画機能の基盤を構築する。ECSリソースパターンを維持しながら、COM APIの初期化順序を厳密に管理する。

### 設計原則
1. **最小限の変更**: 既存の動作コードを最大限活用
2. **厳密な初期化順序**: COM APIの依存関係を明示的に管理
3. **明確なエラー報告**: 初期化失敗時の段階を特定可能にする
4. **ECSパターン遵守**: Resourceとして管理し、システムからアクセス

---

## 🏗️ Component Design

### 1. GraphicsCore構造体

#### 構造体定義
```rust
#[derive(Resource, Debug)]
pub struct GraphicsCore {
    // 既存フィールド（維持）
    pub d3d: ID3D11Device,
    pub dxgi: IDXGIDevice4,
    pub d2d: ID2D1Device,
    pub desktop: IDCompositionDesktopDevice,
    pub dcomp: IDCompositionDevice3,
    
    // 新規フィールド（追加）
    pub d2d_factory: ID2D1Factory,
    pub dwrite_factory: IDWriteFactory2,
}

unsafe impl Send for GraphicsCore {}
unsafe impl Sync for GraphicsCore {}
```

#### フィールド説明

| フィールド | 型 | 用途 | 初期化元 |
|-----------|-----|------|---------|
| `d3d` | `ID3D11Device` | Direct3D11デバイス | `create_device_3d()` |
| `dxgi` | `IDXGIDevice4` | DXGIデバイス | `d3d.cast()` |
| `d2d_factory` | `ID2D1Factory` | Direct2Dファクトリ（新規） | `D2D1CreateFactory()` |
| `d2d` | `ID2D1Device` | Direct2Dデバイス | `d2d_create_device(&dxgi)` |
| `dwrite_factory` | `IDWriteFactory2` | DirectWriteファクトリ（新規） | `dwrite_create_factory()` |
| `desktop` | `IDCompositionDesktopDevice` | DCompデスクトップデバイス | `dcomp_create_desktop_device(&d2d)` |
| `dcomp` | `IDCompositionDevice3` | DCompデバイス | `desktop.cast()` |

#### フィールド追加の理由

**`d2d_factory: ID2D1Factory`**
- **用途**: ブラシ、ジオメトリ、ストロークスタイルなどのD2Dリソース作成に必要
- **必要性**: Milestone 3の描画処理で使用（`create_solid_color_brush`など）
- **スレッドモード**: `D2D1_FACTORY_TYPE_MULTI_THREADED`（ECSのマルチスレッド実行に対応）

**`dwrite_factory: IDWriteFactory2`**
- **用途**: テキストフォーマット、テキストレイアウトの作成に必要
- **必要性**: Phase 2の後半（テキスト描画）で使用
- **共有モード**: `DWRITE_FACTORY_TYPE_SHARED`（プロセス全体で共有）

---

## 🔄 Initialization Flow

### GraphicsCore::new()の実装フロー

```rust
impl GraphicsCore {
    pub fn new() -> Result<Self> {
        eprintln!("[GraphicsCore] 初期化開始");
        
        // Step 1: D3D11Device作成（独立）
        eprintln!("[GraphicsCore] D3D11Deviceを作成中...");
        let d3d = create_device_3d()?;
        eprintln!("[GraphicsCore] D3D11Device作成完了");
        
        // Step 2: IDXGIDevice4取得（D3D11から）
        eprintln!("[GraphicsCore] IDXGIDevice4を取得中...");
        let dxgi: IDXGIDevice4 = d3d.cast()?;
        eprintln!("[GraphicsCore] IDXGIDevice4取得完了");
        
        // Step 3: D2DFactory作成（独立・新規）
        eprintln!("[GraphicsCore] D2DFactoryを作成中...");
        let d2d_factory = create_d2d_factory()?;
        eprintln!("[GraphicsCore] D2DFactory作成完了");
        
        // Step 4: D2DDevice作成（DXGIから）
        eprintln!("[GraphicsCore] D2DDeviceを作成中...");
        let d2d = d2d_create_device(&dxgi)?;
        eprintln!("[GraphicsCore] D2DDevice作成完了");
        
        // Step 5: DWriteFactory作成（独立・新規）
        eprintln!("[GraphicsCore] DWriteFactoryを作成中...");
        let dwrite_factory = dwrite_create_factory(DWRITE_FACTORY_TYPE_SHARED)?;
        eprintln!("[GraphicsCore] DWriteFactory作成完了");
        
        // Step 6: DCompDesktopDevice作成（D2Dから）
        eprintln!("[GraphicsCore] DCompositionDesktopDeviceを作成中...");
        let desktop = dcomp_create_desktop_device(&d2d)?;
        eprintln!("[GraphicsCore] DCompositionDesktopDevice作成完了");
        
        // Step 7: DCompDevice3取得（Desktopから）
        eprintln!("[GraphicsCore] IDCompositionDevice3を取得中...");
        let dcomp: IDCompositionDevice3 = desktop.cast()?;
        eprintln!("[GraphicsCore] IDCompositionDevice3取得完了");
        
        eprintln!("[GraphicsCore] 初期化完了");
        
        Ok(Self {
            d3d,
            dxgi,
            d2d_factory,
            d2d,
            dwrite_factory,
            desktop,
            dcomp,
        })
    }
}
```

### 初期化順序の依存関係図

```
[独立]
├─ D3D11Device ──cast──> IDXGIDevice4 ──┐
├─ D2DFactory (新規)                   │
│                                      ├──> D2DDevice ──┐
└─ DWriteFactory (新規)                │                │
                                       │                │
                            [依存関係] │     [依存関係] │
                                              │
                                              ├──> DCompDesktopDevice ──cast──> DCompDevice3
```

**重要な依存関係**:
1. `d2d` は `dxgi` に依存
2. `desktop` は `d2d` に依存
3. `dcomp` は `desktop` に依存
4. `d2d_factory` と `dwrite_factory` は独立（並列作成可能だが、順序は固定）

---

## 🔧 Helper Function Design

### create_d2d_factory() - 新規追加

```rust
/// D2DFactoryを作成（マルチスレッド対応）
fn create_d2d_factory() -> Result<ID2D1Factory> {
    use windows::Win32::Graphics::Direct2D::Common::*;
    
    unsafe {
        D2D1CreateFactory::<ID2D1Factory>(
            D2D1_FACTORY_TYPE_MULTI_THREADED,
            None,
        )
    }
}
```

**設計判断**:
- `D2D1_FACTORY_TYPE_MULTI_THREADED`: ECSがマルチスレッドで実行されるため必須
- `None`: デフォルトオプション（デバッグレイヤーは不要）

### create_device_3d() - 既存関数を改造

```rust
/// D3D11Deviceを作成
fn create_device_3d() -> Result<ID3D11Device> {
    // デバッグビルド時はデバッグレイヤーを有効化
    #[cfg(debug_assertions)]
    let flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_DEBUG;
    
    #[cfg(not(debug_assertions))]
    let flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT;
    
    d3d11_create_device(
        None,
        D3D_DRIVER_TYPE_HARDWARE,
        HMODULE::default(),
        flags,
        None,
        D3D11_SDK_VERSION,
        None,
        None,
    )
}
```

**変更点**:
- デバッグビルド時に`D3D11_CREATE_DEVICE_DEBUG`を追加
- `#[cfg(debug_assertions)]`で条件分岐

---

## 🎯 System Design

### ensure_graphics_core() - 既存システムを改造

```rust
/// GraphicsCoreが存在しない場合に作成するシステム
/// 
/// UISetupスケジュールで実行され、create_windowsより前に実行される。
pub fn ensure_graphics_core(
    graphics: Option<Res<GraphicsCore>>, 
    mut commands: Commands
) {
    if graphics.is_none() {
        eprintln!("[System] GraphicsCore初期化を開始");
        
        match GraphicsCore::new() {
            Ok(graphics) => {
                commands.insert_resource(graphics);
                eprintln!("[System] GraphicsCoreをECSリソースとして登録完了");
            }
            Err(e) => {
                eprintln!("[System] GraphicsCore初期化失敗: {:?}", e);
                panic!("GraphicsCoreの初期化に失敗しました。アプリケーションを終了します。");
            }
        }
    }
}
```

**設計判断**:
- **冪等性**: `Option<Res<GraphicsCore>>`で既存確認、存在すれば何もしない
- **エラー処理**: 失敗時は`panic!`でアプリケーションを終了（回復不可能なエラー）
- **ログ**: システムレベルのログは`[System]`プレフィックス

---

## 📦 Module Integration

### ecs/graphics.rs の変更

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
+   pub d2d_factory: ID2D1Factory,
    pub d2d: ID2D1Device,
+   pub dwrite_factory: IDWriteFactory2,
    pub desktop: IDCompositionDesktopDevice,
    pub dcomp: IDCompositionDevice3,
}

-unsafe impl Send for GraphicsDevices {}
-unsafe impl Sync for GraphicsDevices {}
+unsafe impl Send for GraphicsCore {}
+unsafe impl Sync for GraphicsCore {}

-impl GraphicsDevices {
+impl GraphicsCore {
    pub fn new() -> Result<Self> {
+       eprintln!("[GraphicsCore] 初期化開始");
+       
+       eprintln!("[GraphicsCore] D3D11Deviceを作成中...");
        let d3d = create_device_3d()?;
+       eprintln!("[GraphicsCore] D3D11Device作成完了");
+       
+       eprintln!("[GraphicsCore] IDXGIDevice4を取得中...");
        let dxgi = d3d.cast()?;
+       eprintln!("[GraphicsCore] IDXGIDevice4取得完了");
+       
+       eprintln!("[GraphicsCore] D2DFactoryを作成中...");
+       let d2d_factory = create_d2d_factory()?;
+       eprintln!("[GraphicsCore] D2DFactory作成完了");
+       
+       eprintln!("[GraphicsCore] D2DDeviceを作成中...");
        let d2d = d2d_create_device(&dxgi)?;
+       eprintln!("[GraphicsCore] D2DDevice作成完了");
+       
+       eprintln!("[GraphicsCore] DWriteFactoryを作成中...");
+       let dwrite_factory = dwrite_create_factory(DWRITE_FACTORY_TYPE_SHARED)?;
+       eprintln!("[GraphicsCore] DWriteFactory作成完了");
+       
+       eprintln!("[GraphicsCore] DCompositionDesktopDeviceを作成中...");
        let desktop = dcomp_create_desktop_device(&d2d)?;
+       eprintln!("[GraphicsCore] DCompositionDesktopDevice作成完了");
+       
+       eprintln!("[GraphicsCore] IDCompositionDevice3を取得中...");
        let dcomp: IDCompositionDevice3 = desktop.cast()?;
+       eprintln!("[GraphicsCore] IDCompositionDevice3取得完了");
+       
+       eprintln!("[GraphicsCore] 初期化完了");
+       
        Ok(Self {
            d3d,
            dxgi,
+           d2d_factory,
            d2d,
+           dwrite_factory,
            desktop,
            dcomp,
        })
    }
}

+/// D2DFactoryを作成（マルチスレッド対応）
+fn create_d2d_factory() -> Result<ID2D1Factory> {
+    unsafe {
+        D2D1CreateFactory::<ID2D1Factory>(
+            D2D1_FACTORY_TYPE_MULTI_THREADED,
+            None,
+        )
+    }
+}

fn create_device_3d() -> Result<ID3D11Device> {
+   #[cfg(debug_assertions)]
+   let flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT | D3D11_CREATE_DEVICE_DEBUG;
+   
+   #[cfg(not(debug_assertions))]
+   let flags = D3D11_CREATE_DEVICE_BGRA_SUPPORT;
+   
    d3d11_create_device(
        None,
        D3D_DRIVER_TYPE_HARDWARE,
        HMODULE::default(),
-       D3D11_CREATE_DEVICE_BGRA_SUPPORT,
+       flags,
        None,
        D3D11_SDK_VERSION,
        None,
        None,
    )
}

-/// GraphicsDevicesが存在しない場合に作成するシステム
-pub fn ensure_graphics_devices(devices: Option<Res<GraphicsDevices>>, mut commands: Commands) {
+/// GraphicsCoreが存在しない場合に作成するシステム
+pub fn ensure_graphics_core(graphics: Option<Res<GraphicsCore>>, mut commands: Commands) {
-   if devices.is_none() {
+   if graphics.is_none() {
+       eprintln!("[System] GraphicsCore初期化を開始");
+       
-       match GraphicsDevices::new() {
+       match GraphicsCore::new() {
            Ok(graphics) => {
                commands.insert_resource(graphics);
-               eprintln!("Graphics devices created successfully");
+               eprintln!("[System] GraphicsCoreをECSリソースとして登録完了");
            }
            Err(e) => {
-               eprintln!("Failed to create graphics devices: {:?}", e);
+               eprintln!("[System] GraphicsCore初期化失敗: {:?}", e);
+               panic!("GraphicsCoreの初期化に失敗しました。アプリケーションを終了します。");
            }
        }
    }
}
```

### ecs/world.rs の変更

```diff
        // デフォルトシステムの登録
        {
            let mut schedules = world.resource_mut::<Schedules>();
+           schedules.add_systems(
+               UISetup, 
+               crate::ecs::graphics::ensure_graphics_core
+                   .before(crate::ecs::window_system::create_windows)
+           );
            schedules.add_systems(UISetup, crate::ecs::window_system::create_windows);
            // on_window_handle_addedとon_window_handle_removedはフックで代替
        }
```

**重要**: `ensure_graphics_core`は`create_windows`より前に実行する必要がある。

---

## 🧪 Testing Strategy

### 単体テスト（実装時）

1. **GraphicsCore::new()のテスト**
   ```rust
   #[test]
   fn test_graphics_core_creation() {
       let graphics = GraphicsCore::new().expect("初期化に失敗");
       // 各フィールドが有効なハンドルを持つことを確認
       assert!(!graphics.d3d.is_invalid());
       assert!(!graphics.d2d_factory.is_invalid());
       assert!(!graphics.dwrite_factory.is_invalid());
   }
   ```

2. **create_d2d_factory()のテスト**
   ```rust
   #[test]
   fn test_d2d_factory_creation() {
       let factory = create_d2d_factory().expect("D2DFactory作成失敗");
       assert!(!factory.is_invalid());
   }
   ```

### 統合テスト

1. **ECS統合テスト**
   - `ensure_graphics_core`システムが冪等であることを確認
   - 複数回実行してもリソースが1つだけであることを確認

2. **アプリケーション起動テスト**
   - 既存のサンプル（`examples/areka.rs`, `examples/dcomp_demo.rs`）が正常に動作することを確認

---

## ⚠️ Error Handling

### エラー分類と対処

| エラー段階 | 原因 | 対処 |
|----------|------|------|
| D3D11Device作成失敗 | ドライバ問題、ハードウェア非対応 | `panic!`（回復不可能） |
| IDXGIDevice4取得失敗 | D3D11Deviceの型が不正 | `panic!`（回復不可能） |
| D2DFactory作成失敗 | システムリソース不足 | `panic!`（回復不可能） |
| D2DDevice作成失敗 | DXGIDeviceが無効 | `panic!`（回復不可能） |
| DWriteFactory作成失敗 | システムリソース不足 | `panic!`（回復不可能） |
| DCompDevice作成失敗 | D2DDeviceが無効 | `panic!`（回復不可能） |

**エラーハンドリング戦略**:
- すべてのエラーは**回復不可能**として扱う
- 初期化失敗時は`panic!`でアプリケーションを終了
- エラーメッセージには失敗した段階を明記

### ログ出力設計

```
正常時:
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

失敗時（例: D2DFactory作成失敗）:
[GraphicsCore] 初期化開始
[GraphicsCore] D3D11Deviceを作成中...
[GraphicsCore] D3D11Device作成完了
[GraphicsCore] IDXGIDevice4を取得中...
[GraphicsCore] IDXGIDevice4取得完了
[GraphicsCore] D2DFactoryを作成中...
[System] GraphicsCore初期化を開始
[System] GraphicsCore初期化失敗: Error { ... }
thread 'main' panicked at 'GraphicsCoreの初期化に失敗しました。アプリケーションを終了します。'
```

---

## 🔍 Design Decisions

### 判断1: ProcessSingletonではなくECSリソースパターンを採用

**理由**:
- 既存実装が`Resource`パターンを使用
- ECSの`Res<T>`で統一的にアクセス可能
- `OnceLock`パターンよりシンプル

### 判断2: パニック戦略を採用

**理由**:
- グラフィックスコアの初期化失敗は致命的
- 部分的な初期化状態での継続は危険
- 早期失敗（fail-fast）が適切

### 判断3: 詳細なログ出力

**理由**:
- 初期化の各段階を追跡可能にする
- デバッグ時に失敗箇所を特定しやすい
- ユーザーにも進捗が見える

### 判断4: D2DFactoryのマルチスレッドモード

**理由**:
- ECSがマルチスレッドで実行される
- `Draw`スケジュールが並列実行される可能性
- スレッドセーフな実装が必須

### 判断5: DWriteFactoryの共有モード

**理由**:
- システム全体でフォント情報を共有
- メモリ効率が良い
- 複数のウィンドウで同じフォントを使用可能

---

## 📊 Performance Considerations

### 初期化パフォーマンス

**目標**: 100ms以内

**推定時間**:
- D3D11Device作成: ~20ms
- D2DFactory作成: ~10ms
- D2DDevice作成: ~10ms
- DWriteFactory作成: ~10ms
- DCompDevice作成: ~10ms
- 合計: ~60ms（目標達成可能）

**最適化ポイント**:
- 初期化は起動時1回のみ（アモタイズ可能）
- 並列化の余地なし（依存関係が強い）

### 実行時パフォーマンス

**アクセスコスト**:
- `Res<GraphicsCore>`でのアクセス: O(1)
- フィールドアクセス: 直接参照（オーバーヘッドなし）

---

## ✅ Design Validation

### 設計の受け入れ基準

- ✅ すべての要件を満たす実装が可能
- ✅ 既存コードへの影響が最小限
- ✅ COM APIの初期化順序が正しい
- ✅ エラー処理が適切
- ✅ ログ出力が追跡可能
- ✅ パフォーマンス要件を満たす

---

## 📚 References

- [Requirements Document](./REQUIREMENTS.md)
- [Milestone Overview](./../brainstorming-next-features/MILESTONES.md)
- [Technology Stack](./../../steering/tech.md)
- [Existing Implementation](../../../crates/wintf/src/ecs/graphics.rs)

---

## 🎯 Next Steps

```bash
/kiro-spec-tasks phase2-m1-graphics-core
```

設計フェーズ完了。次はタスク分解フェーズに進みます。

---

_Phase 2 (Design) completed. Ready for task breakdown phase._
