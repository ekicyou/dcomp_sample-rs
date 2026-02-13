# Requirements Document

## Project Description (Input)

ドラッグによるウィンドウ移動のパフォーマンス最適化。現在のECSパイプライン経由（BoxStyle.inset → taffy → Arrangement → WindowPos → SetWindowPos）ではフレームレイテンシが大きく、ネイティブタスクバードラッグの約1/3のフレームレートしか出ない。Dragging状態に入った時点でHWNDと初期ウィンドウ位置をthread_local DragStateに保存し、以降のWM_MOUSEMOVEではECSを覗かず直接SetWindowPosを叩く方式に変更する。

## 背景情報（event-drag-system実装結果より）

### 現在のアーキテクチャ（event-drag-system完了時点）

#### ドラッグパイプライン（ECS経由 — 現行方式）
```
WM_MOUSEMOVE (wndproc thread)
→ DragAccumulatorResource.accumulate_delta() (Arc<Mutex>)
  ↓ (次のECSフレーム待ち)
dispatch_drag_events → Messages<DragEvent>
→ apply_window_drag_movement → BoxStyle.inset更新
→ compute_taffy_layout (Changed<BoxStyle>で全ツリー再計算)
→ update_arrangements → propagate_global_arrangements
→ window_pos_sync_system → WindowPos更新
→ apply_window_pos_changes → SetWindowPosCommand::enqueue()
→ flush_window_pos_commands() → guarded_set_window_pos()
→ WM_WINDOWPOSCHANGED (エコー → bypass_change_detection)
```

#### ウィンドウ位置パイプライン（wintf-fix1/3/4適用後）
```
BoxStyle.inset → taffy layout → Arrangement.offset → GlobalArrangement.bounds
→ window_pos_sync_system → WindowPos → apply_window_pos_changes
→ SetWindowPosCommand::enqueue() → flush_window_pos_commands()
→ guarded_set_window_pos() → WM_WINDOWPOSCHANGED (echo: bypass_change_detection)
```

逆方向（Win32→ECS）:
```
WM_WINDOWPOSCHANGED → WindowPos (bypass_change_detection or DerefMut)
→ sync_window_arrangement_from_window_pos → Arrangement.offset
```

### 計測結果（Debug build、DPI固定環境）
- **ECSフレーム間隔**: 約16ms（60fps相当でECSは動作）
- **体感フレームレート**: タスクバードラッグの約1/3（20〜30fps相当）
- **ログ**: compute_taffy_layoutが毎フレーム発火（サイズ未変更800x600でもinset変更で再計算）
- **カクつき要因**:
  1. Debug buildのオーバーヘッド（最適化なし）
  2. ECSフレーム待ち（wndproc→ECS間のレイテンシ）
  3. taffy全ツリー再計算（Changed<BoxStyle>トリガー）
  4. SetWindowPos同期API（DWM Commit待ち）
  5. WM_WINDOWPOSCHANGEDエコーによる次フレーム再計算

### 関連する実装済みインフラ
- **thread_local DragState**: `Preparing { entity, start_pos, start_time }` / `Dragging { entity, start_pos, prev_pos }` / `Idle`
- **DragAccumulatorResource**: `Arc<Mutex<DragAccumulator>>`でwndproc→ECS転送
- **guarded_set_window_pos()**: TLSフラグ `IS_SELF_INITIATED` でエコーバック防止（wintf-fix4）
- **find_ancestor_with_drag_config()**: hit_test結果から親階層DragConfig探索
- **DragConfig.move_window**: フラグでウィンドウ自動移動の有効/無効制御

### 提案する最適化方式
Dragging状態遷移時にHWNDと初期ウィンドウ位置をDragState（thread_local）にキャッシュし、WM_MOUSEMOVEハンドラ内で直接SetWindowPosを同期呼び出し。ECSパイプラインをバイパスすることで、WM_MOUSEMOVE → SetWindowPos の最短パスを実現する。

## Requirements
<!-- Will be generated in /kiro:spec-requirements phase -->
