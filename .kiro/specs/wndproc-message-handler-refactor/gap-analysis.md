# Gap Analysis: wndproc-message-handler-refactor

## 1. 現状調査

### 対象ファイル・モジュール

| ファイル | 行数 | 役割 |
|---------|------|-----|
| `src/ecs/window_proc.rs` | 366行 | `ecs_wndproc`、`set_ecs_world`、`get_entity_from_hwnd` |
| `src/ecs/mod.rs` | 32行 | `pub use window_proc::{...}`で再エクスポート |

### 現在のメッセージハンドラ構造

```
ecs_wndproc (lines 29-358)
├── WM_NCCREATE (lines 39-46)      → DefWindowProcW
├── WM_NCDESTROY (lines 47-64)     → DefWindowProcW
├── WM_NCHITTEST (line 66)         → DefWindowProcW (削除対象)
├── WM_ERASEBKGND (lines 67-69)    → LRESULT(1)
├── WM_PAINT (lines 70-73)         → LRESULT(0)
├── WM_CLOSE (lines 74-77)         → LRESULT(0)
├── WM_WINDOWPOSCHANGED (lines 78-276) → DefWindowProcW (最大処理)
├── WM_DISPLAYCHANGE (lines 277-289)   → DefWindowProcW
├── WM_DPICHANGED (lines 290-355)      → LRESULT(0)
└── _ (line 357)                        → DefWindowProcW
```

### 依存関係

| 依存先 | 用途 |
|-------|-----|
| `crate::ecs::window::*` | DPI, WindowHandle, WindowPos, WindowPosChanged, DpiChangeContext, flush_window_pos_commands |
| `crate::ecs::layout::*` | BoxStyle, BoxInset, BoxSize, Dimension, LengthPercentageAuto, Rect |
| `crate::ecs::world::*` | EcsWorld, VsyncTick |
| `crate::ecs::App` | mark_display_change |
| `bevy_ecs::prelude::*` | Entity |
| `windows::Win32::*` | HWND, WPARAM, LPARAM, LRESULT, WM_*, DefWindowProcW等 |
| `tracing` | debug!, trace!, warn! |

### 公開API

| 関数 | 現在の可視性 | 使用箇所 | 変更後 |
|-----|------------|---------|-------|
| `set_ecs_world` | `pub` | `win_thread_mgr.rs` | `pub(crate)` |
| `get_entity_from_hwnd` | `pub` | `window_proc.rs`内部 | `pub(crate)` |
| `ecs_wndproc` | `pub extern "system"` | Windows APIコールバック | `pub(crate) extern "system"` |

## 2. 要件実現可能性分析

### 要件-資産マッピング

| 要件 | 対応資産 | ギャップ |
|-----|---------|---------|
| R1: ハンドラ分離 | `match`式内の各アーム | なし（リファクタリングのみ） |
| R2: 命名規則 | 新規作成 | なし |
| R3: シグネチャ統一 | 新規作成 | なし |
| R4: インライン展開 | `#[inline]`属性 | なし |
| R5: デフォルト処理集約 | `DefWindowProcW`呼び出し | 現在複数箇所に分散 |
| R6: 既存機能維持 | 現在の処理ロジック | なし（ロジック移動のみ） |
| R7: unsafe管理 | 現在の`unsafe`ブロック | なし |
| R8: モジュール分離 | `window_proc.rs` → `window_proc/` | ディレクトリ変換が必要 |
| R9: API最小化 | `pub`関数 | `pub(crate)`への変更 |

### 技術的制約

1. **Windowsメッセージ定数との名前衝突**
   - `WM_NCCREATE`等は`windows`クレートで定数として定義済み
   - 関数名として同名を使用する場合、`#[allow(non_snake_case)]`が必要

2. **`extern "system"`コールバック**
   - `ecs_wndproc`は`extern "system"` ABIが必須
   - Windows APIから直接呼び出されるため変更不可

3. **`try_get_ecs_world()`のスコープ**
   - `handlers.rs`からアクセスする必要がある
   - `mod.rs`で`pub(super)`または`pub(crate)`で公開

## 3. 実装アプローチオプション

### Option A: 最小限のリファクタリング（推奨）

**概要**: 単一ファイル内で関数分離のみ行い、ディレクトリ構造は変更しない

**変更内容**:
- `window_proc.rs`内に8つのハンドラ関数を追加
- `ecs_wndproc`の`match`式を各ハンドラ呼び出しに変更
- 可視性を`pub(crate)`に変更

**トレードオフ**:
- ✅ 変更範囲が最小
- ✅ `mod.rs`の変更なし
- ❌ 将来的にファイルが巨大化する可能性

### Option B: ディレクトリ構造への変換（要件準拠）

**概要**: `window_proc.rs`を`window_proc/mod.rs` + `handlers.rs`に分離

**変更内容**:
```
src/ecs/
  window_proc.rs → 削除
  window_proc/
    mod.rs       # ecs_wndproc, set_ecs_world, get_entity_from_hwnd, try_get_ecs_world, ECS_WORLD
    handlers.rs  # WM_NCCREATE, WM_NCDESTROY, ..., WM_DPICHANGED
```

**トレードオフ**:
- ✅ 将来の拡張に備えた構造
- ✅ 要件R8に完全準拠
- ❌ ファイル移動・リネームが必要
- ❌ `mod.rs`の`mod window_proc;`宣言に変更なし（ディレクトリモジュールとして自動認識）

### Option C: ハイブリッド（段階的移行）

**概要**: Phase 1でOption A、Phase 2でOption Bに移行

**トレードオフ**:
- ✅ リスク分散
- ❌ 2回の変更が必要
- ❌ 今回の要件では過剰

## 4. 複雑度・リスク評価

### 工数見積もり: **S (1〜2日)**

**理由**:
- 既存パターンの適用（関数分離、インライン属性）
- 新規ロジックなし（ロジック移動のみ）
- 依存関係の変更なし
- テスト変更なし

### リスク: **Low**

**理由**:
- 既存のコード構造を維持
- 公開APIの動作は変わらない
- コンパイル時にエラーが検出される（型安全）
- `cargo test`で回帰確認可能

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ: **Option B（ディレクトリ構造への変換）**

要件R8で明示的にディレクトリ構造が求められているため、Option Bを推奨。

### 設計フェーズで決定すべき事項

1. **`handlers.rs`内の関数順序**
   - メッセージ番号順 or 機能グループ順

2. **`try_get_ecs_world()`の配置**
   - `mod.rs`に残すか、`handlers.rs`でも使えるように`pub(super)`で公開するか

3. **lint警告の抑制方法**
   - モジュールレベル `#![allow(non_snake_case)]` vs 関数ごと `#[allow(non_snake_case)]`

### Research Needed: なし

本リファクタリングは既存パターンの適用であり、外部調査は不要。
