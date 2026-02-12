# Requirements Document

## Project Description (Input)
`sync_window_arrangement_from_window_pos`（現在コメントアウト中）の有効化条件と副作用を評価し、必要に応じて有効化する。このシステムは、ユーザーがウィンドウを移動した際に Win32 側の位置変更を ECS の `Arrangement` に逆反映する役割を持つ。

## 親仕様からの参照情報

### 調査レポート参照
- **親仕様**: `dpi-coordinate-transform-survey`
- **根拠セクション**: report.md §1.2 項目3, §5.5, §6.4.3, §8.2 Step 3, §7.1 G5

### 調査結果サマリ
- `ecs/layout/systems.rs` L451-503 に `sync_window_arrangement_from_window_pos` が定義されている
- `ecs/world.rs` L360-363 でコメントアウトにより無効化されている
- 無効化理由は「DIP 座標への二重変換」のリスク
- 調査の結果、現在の座標系設計は `WindowPos.position` = **物理px**、`Arrangement.offset` = **DIP（論理ピクセル）** であり、関数内で `position / DPI.scale` による物理px→DIP変換が正しく実装されている。単一方向の変換であり二重変換は発生しない
- wintf-fix1 完了により重複システム（`update_window_pos_system`）は削除済み。同期方向の競合リスクは解消

### 関連するコード箇所
| ファイル | 行番号 | 内容 |
|---------|--------|------|
| `crates/wintf/src/ecs/layout/systems.rs` | L451-503 | `sync_window_arrangement_from_window_pos` 本体 |
| `crates/wintf/src/ecs/world.rs` | L360-363 | コメントアウト箇所（有効化対象） |
| `crates/wintf/src/ecs/layout/systems.rs` | L370-439 | `window_pos_sync_system`（順方向同期、fix1 で統合済み） |

### 依存関係
- **前提条件**: `wintf-fix1-sync-window-pos-consolidation` 完了済み ✅
- **後続仕様**: `wintf-fix4-feedback-loop-simplify`（本仕様の完了が前提）

### コスト見積もり
- Small（1日以内）— gap analysis により Medium から下方修正

### 検証基準（調査レポートより）
- ユーザーがウィンドウを移動した際に ECS 側の `Arrangement` が更新されること
- フィードバックループが発生しないこと

## Introduction

本要件は、コメントアウトにより無効化されている逆方向同期システム `sync_window_arrangement_from_window_pos` を安全に有効化するための条件と振る舞いを定義する。

このシステムは **Win32 → ECS** 方向の同期を担い、ユーザーがウィンドウをドラッグ移動・リサイズした際に、Win32 側で確定した位置（`WindowPos`）を ECS 側の `Arrangement` コンポーネントに逆反映する。これにより `GlobalArrangement.bounds` がスクリーン上の実際の位置を正確に反映し、ヒットテストやレイアウト計算が正しく動作する。

既存の順方向同期 `window_pos_sync_system`（ECS → Win32）と合わせて **双方向同期ループ** を形成するため、フィードバックループの防止が設計上の最重要課題となる。

## Requirements

### Requirement 1: 逆方向同期の有効化

**Objective:** 開発者として、`sync_window_arrangement_from_window_pos` システムが ECS スケジュールに登録され、正しい順序で実行されるようにしたい。これにより Win32 側で発生したウィンドウ位置変更が ECS の `Arrangement` に反映される。

#### Acceptance Criteria
1. The wintf system shall `sync_window_arrangement_from_window_pos` を PostLayout スケジュールに登録し、毎フレーム実行すること
2. The wintf system shall PostLayout スケジュール内の実行順序を以下のとおり保証すること: `sync_window_arrangement_from_window_pos` → `sync_simple_arrangements` → `mark_dirty_arrangement_trees` → `propagate_global_arrangements` → `window_pos_sync_system`
3. The wintf system shall `world.rs` のコメントアウトを解除し、有効化後のレガシーコメントを削除すること

### Requirement 2: 座標変換の正確性

**Objective:** 開発者として、`WindowPos.position`（物理ピクセル）から `Arrangement.offset`（DIP 座標）への変換が DPI 設定に応じて正確に行われることを保証したい。これにより異なる DPI 環境でもウィンドウ位置が正しく反映される。

#### Acceptance Criteria
1. When `WindowPos.position` が変更された場合, the `sync_window_arrangement_from_window_pos` system shall `DPI` コンポーネントの `scale_x()` / `scale_y()` を使用して物理ピクセルを DIP 座標に変換し、`Arrangement.offset` に設定すること
2. When DPI が 96（100%）の場合, the system shall `WindowPos.position` の値をそのまま `Arrangement.offset` に設定すること（スケール係数 = 1.0）
3. When DPI が 192（200%）の場合, the system shall `WindowPos.position` の値を 0.5 倍して `Arrangement.offset` に設定すること
4. If `DPI.scale_x()` または `DPI.scale_y()` が 0.0 以下の場合, the system shall 変換をスキップし `Arrangement` を更新しないこと

### Requirement 3: エッジケースの安全な処理

**Objective:** 開発者として、ウィンドウ初期化中や不正な状態においても逆方向同期がクラッシュや不正な値を設定しないことを保証したい。

#### Acceptance Criteria
1. When `WindowPos.position` が `None` の場合, the system shall 当該エンティティの処理をスキップすること
2. When `WindowPos.position` が `CW_USEDEFAULT` の場合, the system shall 当該エンティティの処理をスキップすること（ウィンドウ作成時の初期値）
3. While `Arrangement.offset` が新しい変換値と同一の場合, the system shall `Arrangement` コンポーネントを更新しないこと（不要な変更検知を防止）

### Requirement 4: フィードバックループの非発生

**Objective:** 開発者として、逆方向同期（Win32 → ECS）と順方向同期（ECS → Win32）が互いに無限にトリガーし合うフィードバックループが発生しないことを保証したい。

#### Acceptance Criteria
1. When ユーザーがウィンドウをドラッグ移動した場合, the system shall `WindowPos` → `Arrangement` → `GlobalArrangement` → `WindowPos` の更新連鎖が 1 フレーム内で収束し、次フレームで同一値の再更新が発生しないこと
2. The `window_pos_sync_system` shall `GlobalArrangement` が `Changed` フィルタにより変更検知された場合のみ `WindowPos` を更新すること（既存動作の確認）
3. The `sync_window_arrangement_from_window_pos` shall 変換後の値が `Arrangement.offset` の現在値と一致する場合は更新をスキップすること（既存実装の確認）
4. While エコーバック検知（`last_sent_position` / `last_sent_size`）機構が存在する場合, the system shall 当該機構と協調して動作し、ECS 発の `SetWindowPos` による `WM_WINDOWPOSCHANGED` エコーバックを逆方向同期の入力としないこと

### Requirement 5: 既存テストとの整合性

**Objective:** 開発者として、逆方向同期の有効化により既存のテスト・サンプルが破壊されないことを保証したい。

#### Acceptance Criteria
1. The wintf system shall 有効化後に `cargo test` の全テストがパスすること
2. The wintf system shall 有効化後に `cargo run --example taffy_flex_demo` が正常に動作し、ウィンドウの表示・移動・リサイズが正しく機能すること
3. The wintf system shall 有効化後に `cargo run --example areka` が正常に動作すること

### Requirement 6: ログ出力

**Objective:** 開発者として、逆方向同期の動作を `tracing` クレートによるログで確認できるようにしたい。デバッグ時の問題追跡に必要。

#### Acceptance Criteria
1. When `Arrangement.offset` が更新された場合, the system shall `debug!` レベルで、エンティティ名・変更前後のオフセット値・`WindowPos.position`・DPI スケール値をログ出力すること
2. When `Arrangement.offset` が変更不要の場合, the system shall 追加のログ出力を行わないこと（ノイズ削減）
3. The system shall ログメッセージのプレフィックスに `[sync_window_arrangement_from_window_pos]` を使用すること
