# Gap Analysis: event-mouse-basic

## Current State Investigation

### 既存コード調査結果

#### 1. window_proc モジュール (`crates/wintf/src/ecs/window_proc/`)

| ファイル | 行数 | 主要な関数/構造 |
|---------|------|----------------|
| `mod.rs` | ~65行 | `ecs_wndproc`, `set_ecs_world`, `try_get_ecs_world`, `get_entity_from_hwnd` |
| `handlers.rs` | ~389行 | `WM_NCCREATE`, `WM_NCDESTROY`, `WM_ERASEBKGND`, `WM_PAINT`, `WM_CLOSE`, `WM_WINDOWPOSCHANGED`, `WM_DISPLAYCHANGE`, `WM_DPICHANGED` |

**現在ハンドルされているメッセージ:**
- `WM_NCCREATE`, `WM_NCDESTROY` - ウィンドウ作成/破棄
- `WM_ERASEBKGND`, `WM_PAINT` - 描画
- `WM_CLOSE` - クローズ
- `WM_WINDOWPOSCHANGED` - 位置/サイズ変更
- `WM_DISPLAYCHANGE`, `WM_DPICHANGED` - ディスプレイ/DPI変更

**マウス関連メッセージは未実装:**
- `WM_NCHITTEST` - ✗ 未実装（DefWindowProcWに委譲）
- `WM_MOUSEMOVE` - ✗ 未実装
- `WM_LBUTTONDOWN` 等 - ✗ 未実装
- `WM_MOUSELEAVE` - ✗ 未実装
- `WM_MOUSEWHEEL` - ✗ 未実装
- `WM_LBUTTONDBLCLK` 等 - ✗ 未実装

#### 2. 既存パターン分析

**ハンドラシグネチャ（`handlers.rs`）:**
```rust
type HandlerResult = Option<LRESULT>;

fn WM_XXXX(hwnd: HWND, _message: u32, wparam: WPARAM, lparam: LPARAM) -> HandlerResult {
    // Some(LRESULT) = 処理完了
    // None = DefWindowProcWに委譲
}
```

**EcsWorld借用パターン（`handlers.rs` L120-290）:**
```rust
if let Some(entity) = super::get_entity_from_hwnd(hwnd) {
    if let Some(world) = super::try_get_ecs_world() {
        // 第1借用セクション
        if let Ok(mut world_borrow) = world.try_borrow_mut() {
            // コンポーネント操作
        }
        // 借用解放

        // try_tick_on_vsync()
        {
            use crate::ecs::world::VsyncTick;
            let _ = world.try_tick_on_vsync();
        }

        // flush_window_pos_commands()

        // 第2借用セクション（必要に応じて）
    }
}
```

**thread_local!パターン（`window.rs`）:**
```rust
thread_local! {
    static DPI_CHANGE_CONTEXT: RefCell<Option<DpiChangeContext>> = const { RefCell::new(None) };
    static WINDOW_POS_COMMANDS: RefCell<Vec<SetWindowPosCommand>> = const { RefCell::new(Vec::new()) };
}
```

#### 3. VsyncTick トレイト（`world.rs` L28-66）

```rust
pub trait VsyncTick {
    fn try_tick_on_vsync(&self) -> bool;
}

impl VsyncTick for Rc<RefCell<EcsWorld>> {
    fn try_tick_on_vsync(&self) -> bool {
        match self.try_borrow_mut() {
            Ok(mut world) => world.try_tick_on_vsync(),
            Err(_) => false,  // 借用失敗時は安全にスキップ
        }
    }
}
```

#### 4. ECSコンポーネント定義パターン

**Component derive:**
```rust
#[derive(Component, Debug, Clone)]
pub struct SomeComponent { ... }

// SparseSet storage（requirements.md で指定済み）
#[derive(Component, Debug, Clone)]
#[component(storage = "SparseSet")]
pub struct MouseState { ... }
```

---

## Feasibility Analysis

### 実装オプション評価

| 観点 | Option A: handlers.rs拡張 | Option B: 新規mouse.rsモジュール | Option C: ハイブリッド |
|------|--------------------------|--------------------------------|----------------------|
| **既存パターン踏襲** | ◎ 完全一致 | △ 新パターン導入 | ○ 一部新規 |
| **コード分離** | △ handlers.rs肥大化 | ◎ 責務明確 | ○ バランス |
| **保守性** | ○ 一貫性 | ○ モジュール独立 | ○ |
| **再利用性** | △ ハンドラ固有 | ◎ MouseBuffer再利用可能 | ◎ |
| **実装工数** | 低 | 中 | 低〜中 |

### 推奨: Option C（ハイブリッド）

**理由:**
1. **handlers.rs**: WM_XXメッセージ処理関数を追加（既存パターン踏襲）
2. **新規mouse.rs**: `MouseState`, `MouseLeave`, `MouseBuffer`, `ButtonBuffer` 定義（責務分離）
3. **thread_local!**: `MouseBuffer` は既存の `DpiChangeContext` パターンに準拠

---

## Implementation Options

### Option C 詳細設計

#### 新規ファイル構成

```
crates/wintf/src/ecs/
├── mouse.rs              # NEW: MouseState, MouseLeave, MouseBuffer, ButtonBuffer
├── window_proc/
│   ├── mod.rs            # 変更なし
│   ├── handlers.rs       # 拡張: WM_NCHITTEST, WM_MOUSEMOVE, etc.
│   └── mouse_handlers.rs # NEW (オプション): マウス関連ハンドラ分離
```

#### mouse.rs 構成

```rust
// ============================================================================
// ECS Components
// ============================================================================

#[derive(Component, Debug, Clone)]
#[component(storage = "SparseSet")]
pub struct MouseState { ... }

#[derive(Component, Debug, Clone, Copy)]
#[component(storage = "SparseSet")]
pub struct MouseLeave;

// ============================================================================
// WndProc-side Buffers (thread_local!)
// ============================================================================

thread_local! {
    static MOUSE_BUFFER: RefCell<MouseBuffer> = const { RefCell::new(MouseBuffer::new()) };
}

pub struct MouseBuffer {
    position_history: VecDeque<PositionSample>,
    button_buffers: [ButtonBuffer; 5],
    wheel_delta_v: i16,
    wheel_delta_h: i16,
    modifiers: Modifiers,
    // ...
}
```

#### handlers.rs 拡張

```rust
// 既存の関数と同列に追加
pub(super) unsafe fn WM_NCHITTEST(...) -> HandlerResult { ... }
pub(super) unsafe fn WM_MOUSEMOVE(...) -> HandlerResult { ... }
pub(super) unsafe fn WM_LBUTTONDOWN(...) -> HandlerResult { ... }
// ...

// mod.rs の ecs_wndproc match文に追加
WM_NCHITTEST => handlers::WM_NCHITTEST(hwnd, message, wparam, lparam),
WM_MOUSEMOVE => handlers::WM_MOUSEMOVE(hwnd, message, wparam, lparam),
// ...
```

---

## Effort & Risk Evaluation

### 実装工数見積もり

| タスク | 工数 | 複雑度 |
|--------|------|--------|
| MouseState/MouseLeave コンポーネント定義 | 1-2h | 低 |
| MouseBuffer/ButtonBuffer 定義 | 2-3h | 中 |
| WM_NCHITTEST ハンドラ | 1-2h | 低（仮実装）|
| WM_MOUSEMOVE ハンドラ | 3-4h | 中 |
| WM_LBUTTONDOWN 等ボタンハンドラ（5種） | 2-3h | 低 |
| WM_LBUTTONDBLCLK 等ダブルクリック（3種） | 1-2h | 低 |
| WM_MOUSEWHEEL ハンドラ | 1-2h | 低 |
| WM_MOUSELEAVE ハンドラ | 1h | 低 |
| WindowMouseTracking リソース | 1h | 低 |
| 速度計算ロジック | 2-3h | 中 |
| FrameFinalize リネーム | 1h | 低 |
| テスト | 3-4h | 中 |
| **合計** | **18-28h** | - |

### リスク評価

| リスク | 影響度 | 発生確率 | 対策 |
|--------|--------|----------|------|
| RefCell借用競合 | 高 | 低 | 既存try_borrow_mutパターン踏襲 |
| WM_NCHITTEST高頻度 | 中 | 高 | event-hit-test-cacheで後続対応 |
| ダブルクリック時間取得 | 低 | 中 | GetDoubleClickTime() API使用 |
| 速度計算精度 | 低 | 低 | QueryPerformanceCounter使用 |
| thread_local!初期化 | 低 | 低 | const初期化パターン踏襲 |

### 依存関係

**前提:**
- `event-hit-test` 仕様が提供する `hit_test` API（未実装の場合は仮スタブ）

**後続:**
- `event-hit-test-cache` がキャッシュ機構を提供（WM_NCHITTEST最適化）

---

## Gap Summary

### 既存コードとの差分

| 要件 | 既存コード | 必要な変更 |
|------|-----------|-----------|
| Req 1: MouseState | なし | 新規追加（mouse.rs） |
| Req 2: MouseLeave | なし | 新規追加（mouse.rs） |
| Req 3: 速度計算 | なし | MouseBuffer + 計算ロジック追加 |
| Req 4: ローカル座標 | なし | hit_test_detailed連携 |
| Req 5: Win32統合 | handlers.rs存在 | WM_XXハンドラ追加（7種+） |
| Req 5A: MouseBuffer | なし | 新規追加（thread_local!） |
| Req 6: WindowMouseTracking | なし | 新規リソース追加 |
| Req 7: FrameFinalize | CommitComposition存在 | リネームのみ |
| Req 8: 命名規則 | - | 設計時に反映 |

### 統合ポイント

1. **ecs_wndproc**（`window_proc/mod.rs`）: match文にマウスメッセージ追加
2. **handlers.rs**: WM_XXハンドラ関数追加
3. **ecs/mod.rs**: mouse.rsモジュールを公開（`pub mod mouse;`）
4. **EcsWorld schedules**: MouseStateクリーンアップをFrameFinalize/Inputに追加

---

## Conclusion

**Gap Analysis結果:**
- 既存のwindow_proc/handlers.rsパターンに沿った拡張が可能
- thread_local!による MouseBuffer パターンは既存の DpiChangeContext パターンに準拠
- 主要なリスクは WM_NCHITTEST 高頻度問題だが、event-hit-test-cache で後続対応

**次のステップ:**
1. `/kiro-spec-design event-mouse-basic` でDesignフェーズへ進行
2. mouse.rs と handlers.rs の詳細設計を策定
