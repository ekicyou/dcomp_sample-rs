# Layout to Graphics Synchronization Specification

## Overview
レイアウト計算結果をグラフィックスリソース（Visual、Surface、WindowPos）に正しく伝播させ、双方向の同期とループ回避を実現する。

## Status
- **Created**: 2025-11-24
- **Status**: Draft
- **Dependencies**: taffy-layout-integration (必須)

## Problem Statement

### 現状の問題
1. **Surface サイズ不整合**: Taffy計算は BoxSize (800×600) を使用するが、実際の Surface は Visual.size (782×553) で作成される
2. **GetClientRect への依存**: window_system.rs で GetClientRect → Visual.size の直接代入が行われている（仮実装）
3. **情報フローの不明確性**: レイアウト → Visual → Surface → WindowPos の伝播が未実装

### 影響
- Green rectangle が 8px しか表示されない（Surface 境界でクリップ）
- レイアウト計算結果が描画に正しく反映されない
- ウィンドウリサイズ時の無限ループリスク

## Goals

### 1. 正しい情報フロー実装
```
BoxSize (800×600)
  ↓ Taffy計算
GlobalArrangement.bounds (800×600)
  ↓ sync_visual_from_layout
Visual.size (800×600)
  ↓ resize_surface_from_visual
Surface (800×600)
  ↓ sync_window_pos
WindowPos (800×600)
  ↓ apply_window_pos_changes (無限ループ回避あり)
SetWindowPos(800×600)
  ↓
WM_WINDOWPOSCHANGED (800×600)
  ↓ エコーバック検知
スキップ ✓
```

### 2. 無限ループ回避メカニズム
- **エコーバック検知**: WindowPos に `last_sent_position/size` キャッシュを追加
- **WM_WINDOWPOSCHANGED 処理**: 送信値と一致する場合はスキップ
- **PartialEq 実装**: Component に PartialEq を derive して自動最適化

### 3. 双方向同期
- **レイアウト → ウィンドウ**: 上記フロー
- **ウィンドウ → レイアウ**: WM_WINDOWPOSCHANGED → WindowPos → BoxSize 更新

## Requirements

### R1: sync_visual_from_layout_root システム
- **目的**: LayoutRoot を持つエンティティの GlobalArrangement から Visual.size を計算
- **配置**: PostLayout スケジュール、propagate_global_arrangements の後
- **クエリ**: `Query<(&GlobalArrangement, &mut Visual), (With<LayoutRoot>, Changed<GlobalArrangement>)>`
- **処理**:
  ```rust
  visual.size.X = (bounds.right - bounds.left) as f32;
  visual.size.Y = (bounds.bottom - bounds.top) as f32;
  ```

### R2: resize_surface_from_visual システム
- **目的**: Visual サイズ変更時に Surface を再作成
- **配置**: PostLayout スケジュール、sync_visual_from_layout_root の後
- **クエリ**: `Query<(Entity, &Visual, &mut SurfaceGraphics), Changed<Visual>>`
- **処理**:
  - `Visual.size != SurfaceGraphics.size` なら `create_surface_for_window` 呼び出し
  - SurfaceGraphics.size を更新

### R3: sync_window_pos システム
- **目的**: Visual → WindowPos 同期
- **配置**: PostLayout スケジュール、resize_surface_from_visual の後
- **クエリ**: `Query<(&GlobalArrangement, &Visual, &mut WindowPos), (With<Window>, Or<(Changed<GlobalArrangement>, Changed<Visual>)>)>`
- **処理**:
  ```rust
  window_pos.position = (bounds.left, bounds.top);
  window_pos.size = (visual.size.X as i32, visual.size.Y as i32);
  ```

### R4: apply_window_pos_changes システム（改良版）
- **目的**: WindowPos → SetWindowPos、無限ループ回避
- **配置**: PostLayout スケジュール、sync_window_pos の後
- **処理**:
  ```rust
  let pos = window_pos.position;
  let size = window_pos.size;
  SetWindowPos(hwnd, pos.0, pos.1, size.0, size.1, ...);
  
  // エコーバック用キャッシュ（変更検知なし）
  window_pos.bypass_change_detection().last_sent_position = pos;
  window_pos.bypass_change_detection().last_sent_size = size;
  ```

### R5: WM_WINDOWPOSCHANGED ハンドラ（改良版）
- **目的**: ウィンドウメッセージ → BoxSize 更新、エコーバック検知
- **処理**:
  ```rust
  // エコーバック判定
  if window_pos.last_sent_position == new_pos 
     && window_pos.last_sent_size == new_size {
      return; // スキップ
  }
  
  // 外部変更 → レイアウト更新トリガー
  window_pos.position = new_pos;
  window_pos.size = new_size;
  
  // BoxSize も更新（レイアウト再計算のため）
  box_size.width = Some(Dimension::Px(new_size.0 as f32));
  box_size.height = Some(Dimension::Px(new_size.1 as f32));
  ```

### R6: WindowPos コンポーネント拡張
```rust
#[derive(Component, PartialEq)]
pub struct WindowPos {
    pub position: (i32, i32),
    pub size: (i32, i32),
    pub flags: u32,
    
    // エコーバック検知用（変更検知対象外）
    last_sent_position: (i32, i32),
    last_sent_size: (i32, i32),
}

impl WindowPos {
    pub fn is_echo(&self, position: (i32, i32), size: (i32, i32)) -> bool {
        self.last_sent_position == position && self.last_sent_size == size
    }
}
```

### R7: GetClientRect 依存の削除
- **削除対象**: window_system.rs の `GetClientRect(hwnd, &mut rect)` → Visual.size 代入
- **理由**: レイアウトシステムが Visual.size を管理すべき
- **初期化**: Window 作成時は BoxSize から初期 Visual.size を設定

### R8: WM_MOVE, WM_SIZE メッセージの削除
- **削除対象**: WM_MOVE, WM_SIZE ハンドラ
- **理由**: WM_WINDOWPOSCHANGED で位置とサイズ両方を取得可能
- **効率化**: 1つのメッセージで完結

## Non-Goals
- アニメーション中のフレーム間補間
- 複数モニタDPI対応（別スコープ）
- ウィンドウ最小化/最大化の特殊処理

## Success Criteria
1. ✅ Taffy 計算結果が Surface サイズに正確に反映される
2. ✅ Green rectangle が期待通り 45px の高さで表示される
3. ✅ ユーザーリサイズ時に無限ループが発生しない
4. ✅ レイアウト起点のリサイズでも無限ループが発生しない
5. ✅ WM_WINDOWPOSCHANGED でエコーバックが正しくスキップされる
6. ✅ すべてのテストが通過する

## Testing Strategy
- Unit tests: WindowPos.is_echo(), エコーバック検知ロジック
- Integration tests: レイアウト → Surface → ウィンドウの完全フロー
- Manual tests: taffy_flex_demo.rs でリサイズ動作確認

## Background Knowledge

### Surface サイズ不整合の原因
- **GetClientRect**: タイトルバーとボーダーを除いたクライアント領域を返す
- **実測値**: 800×600 ウィンドウ → 782×553 クライアント領域（幅-18px, 高さ-47px）
- **現状の問題**: BoxSize (800×600) で Taffy 計算 → Visual.size (782×553) で Surface 作成 → サイズ不一致

### WinUI3 ループ回避パターン
```cpp
// DesktopWindowImpl::RaiseWindowSizeChangedEvent
if (m_previousWindowSizeChangedSize.Width != size.Width
    || m_previousWindowSizeChangedSize.Height != size.Height)
{
    // イベント発火
    m_sizeChangedEventSource.Raise(...);
    m_previousWindowSizeChangedSize = size;  // キャッシュ更新
}
```

**仕組み**:
1. SetWindowPos(800, 600) 呼び出し → m_previousSize = (800, 600)
2. WM_SIZE受信 → size = (800, 600)
3. 比較: m_previousSize == size → スキップ（ループ回避）
4. ユーザーリサイズ → size = (1024, 768)
5. 比較: m_previousSize != size → イベント発火

### Bevy ECS 変更検知の仕組み
```rust
// Bevy 0.14+ の Mut<T> 実装（簡略版）
impl<T: Component + PartialEq> Drop for Mut<'_, T> {
    fn drop(&mut self) {
        if self.deref_mut_called && self.value != self.original {
            self.ticks.set_changed(self.this_run);
        }
    }
}
```

**動作**:
- `component.size = (800, 600)` → DerefMut 呼び出し
- スコープ終了 → Drop 実行
- 元の値と比較 → 同じなら変更フラグを立てない
- `PartialEq` derive が必須

**利点**: 
- 値比較による自動最適化
- `set_if_neq` が不要になる方向

### WM_WINDOWPOSCHANGED の優位性
```
Windows メッセージフロー:
SetWindowPos()
  ↓
WM_WINDOWPOSCHANGING (変更前)
  ↓
[実際の変更]
  ↓
WM_WINDOWPOSCHANGED (変更後) ← これを処理
  ↓
DefWindowProc() → WM_MOVE, WM_SIZE に分解
```

**WM_WINDOWPOSCHANGED の利点**:
- 位置とサイズを1つのメッセージで取得
- WINDOWPOS 構造体に flags も含まれる（SWP_NOSIZE, SWP_NOMOVE）
- DefWindowProc を呼ばなければ WM_MOVE/WM_SIZE は生成されない

**実装**:
```rust
WM_WINDOWPOSCHANGED => {
    let windowpos = &*(lparam.0 as *const WINDOWPOS);
    handle_window_pos_changed(hwnd, windowpos);
    return LRESULT(0); // DefWindowProc を呼ばない
}
```

### エコーバック検知の原理
```
【レイアウト起点】
Layout → WindowPos.size = (800, 600)
  ↓
apply_window_pos_changes: 
  - SetWindowPos(800, 600)
  - last_sent_size = (800, 600) [bypass_change_detection]
  ↓
WM_WINDOWPOSCHANGED → (800, 600)
  ↓
is_echo(800, 600): last_sent == received → true
  ↓
スキップ ✓ ループ回避

【ユーザー操作起点】
ユーザーリサイズ → 1024×768
  ↓
WM_WINDOWPOSCHANGED → (1024, 768)
  ↓
is_echo(1024, 768): last_sent(800, 600) != received → false
  ↓
WindowPos.size = (1024, 768) [Changed 発火]
  ↓
Layout 再計算 ✓
```

**重要**: 双方向で値を比較することで、どちら起点でも自然にループが止まる

## References
- WinUI3 ループ回避: `microsoft/microsoft-ui-xaml` の `DesktopWindowImpl.cpp`
- Bevy ECS 変更検知: `Mut::bypass_change_detection()`, PartialEq による Drop 時最適化
- Windows メッセージ: WM_WINDOWPOSCHANGED (0x0047), WM_SIZE (0x0005), WM_MOVE (0x0003)
- 現在の問題詳細: conversation-summary の "3. Codebase Status" 参照
