# Requirements Document

## Introduction

ウィンドウのスクリーン座標（XY位置）が `BoxStyle.inset` に書き込まれることで、ウィンドウ移動のたびに `Changed<BoxStyle>` が発火し、taffy レイアウト再計算パイプライン全体がトリガーされる。ウィンドウ移動はウィンドウ内部のウィジェットレイアウトに影響しないため、XY座標を `BoxStyle` から除外してレイアウト再計算を抑制する。サイズ情報は現行通り `BoxStyle` に入力する。

### 影響範囲

| 箇所 | ファイル | 現在の動作 |
|------|----------|------------|
| `WM_WINDOWPOSCHANGED` ハンドラ | `ecs/window_proc/handlers.rs` | `BoxStyle.inset` にXY物理座標を書き込み |
| ドラッグシステム | `ecs/drag/systems.rs` | `BoxStyle.inset` のleft/topを更新 |
| ドラッグ開始 | `ecs/drag/dispatch.rs` | `BoxStyle.inset` からinitial_insetを取得 |
| PostLayout同期 | `ecs/layout/systems.rs` | `sync_window_arrangement_from_window_pos` がWindowPos→Arrangement.offsetを書く |
| Layout→Win32同期 | `ecs/layout/systems.rs` | `window_pos_sync_system` がGlobalArrangement→WindowPosに反映 |

### 前提確認

- **WindowPos コンポーネント**: `position: Option<POINT>` と `size: Option<SIZE>` を持ち、物理ピクセルのクライアント領域座標を保持する。ウィンドウエンティティは `WindowPos` を通じて生のXY座標およびサイズを常に参照可能。
- **Arrangement.offset**: PostLayout の `sync_window_arrangement_from_window_pos` で `WindowPos.position` → `Arrangement.offset` に反映済み。XY座標の描画パイプラインへの伝搬経路は BoxStyle を経由しなくても確保されている。
- **位置の source of truth**: Window entity のスクリーン座標の唯一の source of truth は `WindowPos.position` とする。`BoxStyle.inset` は Window entity の位置決定に関与しない。

## Requirements

### Requirement 1: BoxStyle からウィンドウXY座標の除外

**Objective:** フレームワーク開発者として、ウィンドウ移動時に不要なレイアウト再計算が発生しないようにしたい。これにより、ウィンドウドラッグ中のCPU負荷を低減し描画パフォーマンスを向上させる。

#### Acceptance Criteria

1. When `WM_WINDOWPOSCHANGED` メッセージを受信した際、the wintf system shall `BoxStyle.inset` にウィンドウXY座標（left, top）を書き込まないこと
2. When `WM_WINDOWPOSCHANGED` メッセージを受信した際、the wintf system shall `BoxStyle.size`（width, height）は現行通り論理ピクセル単位で更新すること
3. When ウィンドウがスクリーン上で移動（位置のみ変更、サイズ不変）された際、the wintf system shall `Changed<BoxStyle>` を発火させないこと
4. When ウィンドウサイズが変更された際、the wintf system shall `Changed<BoxStyle>` を発火させ、レイアウト再計算をトリガーすること
5. The wintf system shall Window entity の `BoxStyle.inset`（left, top）を常に `Auto`（または `Px(0.0)`）に保ち、スクリーン座標を格納しないこと

### Requirement 2: ドラッグによるウィンドウ移動のWndProcレベル化

**Objective:** フレームワーク開発者として、ドラッグによるウィンドウ移動をECSレイアウトパイプラインを経由せず、WndProcレベル（WM_MOUSEMOVEハンドラ内での直接SetWindowPos呼び出し）で処理したい。これにより、レイアウト再計算の抑制に加え、ECSフレーム待ちのレイテンシも排除され、ネイティブに近いドラッグ体感を実現する。

**背景（drag-wndproc-direct-move仕様より合流）:**
- 現行のECSパイプライン経由方式（BoxStyle.inset → taffy → Arrangement → WindowPos → SetWindowPos）では、タスクバードラッグの約1/3のフレームレートとなっている（計測: Debug build、DPI固定環境）
- カクつきの要因: ECSフレーム待ちレイテンシ、taffy全ツリー再計算、SetWindowPos同期API、WM_WINDOWPOSCHANGEDエコー再計算
- 既存インフラ: thread_local `DragState`（Idle/Preparing/Dragging）、`DragAccumulatorResource`（Arc<Mutex>）、`guarded_set_window_pos()`（IS_SELF_INITIATEDエコーバック防止）、`find_ancestor_with_drag_config()`

#### Acceptance Criteria

1. While ドラッグ中（Dragging状態）、the wndproc handler shall `WM_MOUSEMOVE` 内で直接 `SetWindowPos`（または同等Win32 API）を呼び出してウィンドウを移動すること
2. While ドラッグ中、the drag system shall ECSパイプライン（BoxStyle → taffy → Arrangement → WindowPos）を経由しないこと
3. When ドラッグ状態（Dragging）に遷移した際、the drag system shall HWNDと初期ウィンドウ位置をthread_local `DragState` にキャッシュすること
4. When ドラッグ終了時、the drag system shall 最終ウィンドウ位置をECSの `WindowPos` に反映し、`sync_window_arrangement_from_window_pos` を通じて `Arrangement` との整合性を回復すること
5. The drag system shall `DragConfig.move_window` フラグが `true` の場合のみWndProcレベル移動を実施すること
6. While ドラッグ中、the drag system shall `Changed<BoxStyle>` を発火させないこと

### Requirement 3: ウィンドウ位置の代替伝搬経路の保証

**Objective:** フレームワーク開発者として、`BoxStyle.inset` からXY座標を除外した後も、ウィンドウ位置がビジュアルツリーに正しく反映されることを保証したい。

#### Acceptance Criteria

1. The wintf system shall `WindowPos.position` を通じてウィンドウの物理ピクセル座標（XY）を常に参照可能であること
2. The wintf system shall `WindowPos.size` を通じてウィンドウの物理ピクセルサイズを常に参照可能であること
3. When 外部からウィンドウが移動された際、the PostLayout system shall `sync_window_arrangement_from_window_pos` 経由で `Arrangement.offset` にXY座標を反映すること
4. The wintf system shall `WindowPos` → `Arrangement.offset` → `GlobalArrangement` の伝搬経路によりウィンドウ位置がビジュアルツリーに反映されること
5. The layout system shall `update_arrangements_system` において Window entity の `Arrangement.offset` を taffy 計算結果で上書きしないこと（`WindowPos` が Window 位置の唯一の source of truth）

### Requirement 4: BoxStyle.inset のウィンドウ以外の用途保全

**Objective:** フレームワーク開発者として、`BoxStyle.inset` をウィンドウ以外のウィジェット（絶対配置の子要素等）では引き続き使用可能としたい。

#### Acceptance Criteria

1. The layout system shall 非Windowエンティティの `BoxStyle.inset` は現行通りtaffyレイアウトに反映すること
2. The layout system shall `BoxStyle.inset` の型定義・構造は変更しないこと
