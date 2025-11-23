# Project Structure

## Organization Philosophy

**レイヤードアーキテクチャ** - Windows COM APIラッパー（`com/`）、ECSコンポーネント（`ecs/`）、メッセージハンドリング（ルート）の3層構造で責務を分離。論理ツリーとビジュアルツリーの二層構成により、アプリケーションロジックと描画処理を独立させる。

## Directory Patterns

### Workspace Root
**Location**: `/`  
**Purpose**: Cargoワークスペース設定、ドキュメント、CI/CD設定  
**Example**: `Cargo.toml`, `README.md`, `.kiro/`

### Library Crate
**Location**: `/crates/wintf/`  
**Purpose**: メインライブラリの実装  
**Structure**:
- `src/` - ライブラリソースコード
- `examples/` - サンプルアプリケーション（`areka.rs`, `dcomp_demo.rs`）

### COM Wrapper Layer
**Location**: `/crates/wintf/src/com/`  
**Purpose**: Windows COMインターフェースのRustラッパー  
**Contains**:
- `dcomp.rs` - DirectComposition API
- `d3d11.rs` - Direct3D11 API
- `dwrite.rs` - DirectWrite API (縦書き対応)
- `wic.rs` - Windows Imaging Component
- `animation.rs` - Windows Animation API
- `d2d/` - Direct2D関連

### ECS Component Layer
**Location**: `/crates/wintf/src/ecs/`  
**Purpose**: ECSアーキテクチャのコンポーネント定義  
**Structure**:
- `common/` - 共通インフラ（階層伝播システム）
- `layout/` - レイアウトシステム（taffy統合、配置計算）
- `transform/` - 実験的変換（非推奨、WinUI3模倣）
- `widget/` - UIウィジェット（Label、Rectangle等）
- `window.rs` - ウィンドウ管理
- `graphics.rs` - グラフィックスリソース
- `world.rs` - ECS World / schedule管理

#### ECS機能グループ詳細

**1. Common Infrastructure** (`common/`)
- 責務: ECS階層システムの汎用的な伝播ロジック
- 代表的な関数: `sync_simple_transforms<L,G,M>()`, `propagate_parent_transforms<L,G,M>()`
- 特徴: 完全ジェネリック化、`Arrangement`/`Transform`両対応

**2. Window Management** (`window.rs`, `window_system.rs`, `window_proc.rs`)
- 責務: Win32ウィンドウのライフサイクル管理とECS統合
- 代表的なコンポーネント: `Window`, `WindowHandle`, `WindowPos`, `WindowStyle`, `ZOrder`
- 特徴: HWNDとEntityの双方向マッピング、マルチスレッド対応

**3. Graphics Resources** (`graphics.rs`, `graphics/`)
- 責務: Direct2D/DirectCompositionリソースのライフサイクル管理
- 代表的なコンポーネント: `GraphicsCore`, `WindowGraphics`, `Visual`, `Surface`, `DeviceContext`
- 特徴: デバイスロスト対応、遅延初期化、階層的描画

**4. Layout System** (`layout/`)
- 責務: taffyレイアウトエンジン統合と配置計算
- サブモジュール: `taffy.rs`, `metrics.rs`, `arrangement.rs`, `rect.rs`, `systems.rs`
- 代表的なコンポーネント: `TaffyStyle`, `TaffyComputedLayout`, `Arrangement`, `GlobalArrangement`, `Size`, `Offset`
- 特徴: 軸平行変換最適化、Common Infrastructure活用、Surface生成最適化

**5. Transform** (`transform/`, **非推奨**)
- 責務: WinUI3/WPF/XAML `RenderTransform`模倣（回転・スキュー対応）
- 代表的なコンポーネント: `Transform`, `GlobalTransform`, `Translate`, `Scale`, `Rotate`, `Skew`
- **非推奨理由**: taffyレイアウトエンジンとの統合不足、軸平行変換最適化が適用できない
- **推奨代替**: `Arrangement`ベースのLayout System

### Message Handling
**Location**: `/crates/wintf/src/`（ルート）  
**Purpose**: Windowsメッセージループとスレッド管理  
**Contains**:
- `winproc.rs` - ウィンドウプロシージャ
- `win_message_handler.rs` - メッセージハンドリング
- `win_thread_mgr.rs` - スレッド管理
- `api.rs` - Windows API safeラッパー

## Naming Conventions

- **Files**: `snake_case.rs` (Rust標準)
- **Modules**: `snake_case`
- **Types**: `PascalCase` (structs, enums, traits)
- **Functions**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`

### Component Naming Conventions

COMオブジェクトをラップするECSコンポーネントは、以下の命名規則に従う：

#### GPUリソース (`XxxGraphics`)
- **特性**: Direct3D/Direct2D/DirectCompositionデバイスに依存
- **デバイスロスト対応**: `invalidate()`メソッドと`generation`フィールドを実装
- **命名**: `XxxGraphics`サフィックス
- **例**:
  - `WindowGraphics` - ウィンドウレベルGPU資源
  - `VisualGraphics` - ウィジェットレベルGPU資源
  - `SurfaceGraphics` - ウィジェットレベルGPU資源
  - 将来: `BrushGraphics`, `BitmapGraphics`

#### CPUリソース (`XxxResource`)
- **特性**: デバイス非依存、永続的
- **デバイスロスト対応**: 不要（通常の参照カウント管理のみ）
- **命名**: `XxxResource`サフィックス
- **例**:
  - `TextLayoutResource` - テキストレイアウト（Label、TextBlock等で再利用）
  - 将来: `TextFormatResource`, `PathGeometryResource`

#### レベル分類
- **ウィンドウレベル**: Windowエンティティに配置（例: `WindowGraphics`）
- **ウィジェットレベル**: 個別ウィジェットエンティティに配置（例: `VisualGraphics`, `TextLayoutResource`）
- **共有リソース**: 複数ウィジェットで再利用（例: 将来の`BrushGraphics`、`GeometryResource`）

#### 非COMコンポーネント
- **論理コンポーネント**: サフィックスなし（例: `Label`, `Rectangle`, `Button`）
- **マーカーコンポーネント**: 用途に応じた名前（例: `HasGraphicsResources`, `GraphicsNeedsInit`）

#### COMアクセスメソッド命名
COMリソースコンポーネント内部のアクセスメソッドは、COMインターフェイス型に対応：
- `WindowGraphics::target()` → `Option<&IDCompositionTarget>`
- `VisualGraphics::visual()` → `Option<&IDCompositionVisual3>`
- `SurfaceGraphics::surface()` → `Option<&IDCompositionSurface>`
- `TextLayoutResource::get()` → `Option<&IDWriteTextLayout>`

## Import Organization

```rust
// 標準ライブラリ
use std::sync::Arc;

// 外部クレート（アルファベット順）
use bevy_ecs::prelude::*;
use windows::Win32::Graphics::DirectComposition::*;

// 内部モジュール（相対パス）
use crate::com::dcomp::*;
use crate::ecs::window::*;
```

## Code Organization Principles

- **レイヤー分離**: COM → ECS → Message Handling の依存方向を厳守
- **COMライフタイム**: `windows-rs`提供のスマートポインタで直接管理
- **unsafe隔離**: `unsafe`ブロックはCOMラッパー層に集約し、安全なAPIを上位層に提供
- **モジュール独立性**: 各モジュールは独立してテスト可能な単位として設計

---
_Workspace構成により将来的な機能拡張（別クレート追加）が容易_
