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
**Contains**:
- `window.rs` - ウィンドウ関連コンポーネント
- `graphics.rs` - 描画関連コンポーネント
- `layout.rs` - レイアウト計算コンポーネント
- `world.rs` - ECS World / schedule管理

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
