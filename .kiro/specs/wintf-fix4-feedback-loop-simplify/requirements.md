# Requirements Document

## Introduction

本仕様は、ECS ↔ Win32 間のフィードバックループ防止メカニズムを簡素化するリファクタリングである。現在、防止機構は以下の3+1層で実装されている：

| 層 | 実装名 | 役割 |
|----|--------|------|
| L1 | `WindowPosChanged` (ECS Component) | `WM_WINDOWPOSCHANGED` 処理中の `apply_window_pos_changes` スキップ |
| L2 | エコーバック検知 (`is_echo()`) | ECS→Win32 で送信した値が戻ってきた場合のスキップ |
| L3 | `RefCell` 再入保護 | `try_borrow_mut()` 失敗時スキップ |
| 遅延実行 | `SetWindowPosCommand` (TLS キュー) | World 借用中の `SetWindowPos` 再入防止 |

この構造は堅牢だが冗長であり、`SetWindowPosCommand`（遅延実行キュー）と `WindowPosChanged`（抑制フラグ）を **単一ゲートシステム** に統合する。`DpiChangeContext`（TLS）は `WndProc` コールスタック固有の性質を持つため維持する。

**親仕様**: `dpi-coordinate-transform-survey` (report.md §1.2 項目4, §6.4.4, §8.2 Step 4)
**前提条件**: `wintf-fix3-sync-arrangement-enable` 完了（逆同期が有効化済み）

## Requirements

### Requirement 1: 単一ゲートシステムへの統合

**Objective:** As a wintf 開発者, I want `SetWindowPosCommand`（遅延実行キュー）と `WindowPosChanged`（抑制フラグ）を単一の統合ゲート機構に統合する, so that フィードバックループ防止のコード経路が削減され、保守性が向上する

#### Acceptance Criteria

1. The wintf system shall `SetWindowPosCommand`（TLS キュー）と `WindowPosChanged`（ECS コンポーネント）の責務を単一のゲート機構で管理する
2. When ECS 側で `WindowPos` または `BoxStyle` が変更された場合, the wintf system shall 統合ゲート経由で `SetWindowPos` コマンドを蓄積し、World 借用解放後に一括実行する
3. While 統合ゲートの抑制フラグが有効（`WM_WINDOWPOSCHANGED` 処理中に設定）な場合, the wintf system shall ECS → Win32 再同期（`apply_window_pos_changes` での `SetWindowPos` 発行）をスキップする
4. When VSync tick、WndProc ハンドラ、メッセージループのいずれかのタイミングで flush が呼ばれた場合, the wintf system shall 蓄積された全コマンドを実行し、ゲート状態をリセットする

### Requirement 2: DpiChangeContext の維持

**Objective:** As a wintf 開発者, I want `DpiChangeContext`（TLS）を現行実装のまま維持する, so that `WM_DPICHANGED` → `SetWindowPos` → `WM_WINDOWPOSCHANGED` の同期コールチェーンで DPI 値の伝達が正しく継続される

#### Acceptance Criteria

1. The wintf system shall `DpiChangeContext` の TLS 構造（`set()` / `take()` API）を変更せずに維持する
2. When `WM_DPICHANGED` を受信した場合, the wintf system shall `DpiChangeContext::set()` で新 DPI 値を TLS に格納する
3. When `WM_WINDOWPOSCHANGED` ハンドラが実行された場合, the wintf system shall `DpiChangeContext::take()` で DPI 値を取得し、DPI コンポーネントを即時更新する
4. The wintf system shall 統合ゲートシステムと `DpiChangeContext` の責務を明確に分離する（ゲートは位置・サイズ同期、DpiChangeContext は DPI 値伝達）

### Requirement 3: エコーバック検知の統合

**Objective:** As a wintf 開発者, I want エコーバック検知（`last_sent_position` / `last_sent_size` / `is_echo()`）を統合ゲートの一部として明確に位置づける, so that フィードバック防止の全経路が単一概念で理解できる

#### Acceptance Criteria

1. When `apply_window_pos_changes` が `SetWindowPos` コマンドを生成する際, the wintf system shall 送信値（position, size）を記録する
2. When `WM_WINDOWPOSCHANGED` で受信した値が送信記録と一致する場合, the wintf system shall `bypass_change_detection()` を使用して `Changed<WindowPos>` の発火を抑制する
3. The wintf system shall エコーバック検知を統合ゲートの補助メカニズムとして明確にドキュメントまたはモジュール構造で位置づける

### Requirement 4: フィードバックループ防止の正確性

**Objective:** As a wintf ユーザー, I want ウィンドウ操作時にフィードバックループが発生しない, so that ウィンドウの移動・リサイズ・DPI 変更がスムーズに動作する

#### Acceptance Criteria

1. When ユーザーが Win32 経由でウィンドウを移動・リサイズした場合, the wintf system shall 1フレーム以内にフィードバックを収束させ、無限ループを発生させない
2. When ECS 側からプログラム的にウィンドウ位置を変更した場合, the wintf system shall `SetWindowPos` → `WM_WINDOWPOSCHANGED` → ECS 更新の経路で1往復のみで収束する
3. When DPI 変更（モニタ間移動）が発生した場合, the wintf system shall `WM_DPICHANGED` → `SetWindowPos` → `WM_WINDOWPOSCHANGED` チェーンを正しく処理し、フィードバックループを発生させない
4. While 複数ウィンドウが同時に存在する状態で, when いずれかのウィンドウが移動・リサイズされた場合, the wintf system shall 他のウィンドウに不要なフィードバック連鎖を波及させない

### Requirement 5: コード簡素化と保守性向上

**Objective:** As a wintf 開発者, I want 統合によりフィードバック防止の関連コードが削減される, so that 将来の機能追加やバグ修正が容易になる

#### Acceptance Criteria

1. The wintf system shall フィードバック防止の状態管理ポイント数を削減する（現在: `WindowPosChanged` コンポーネント + `WINDOW_POS_COMMANDS` TLS + エコーバックフィールド の3箇所 → 統合ゲートに集約）
2. The wintf system shall 統合ゲートの状態遷移を明確に文書化（コード内コメントまたは doc comment）する
3. The wintf system shall `WM_WINDOWPOSCHANGED` ハンドラ内の4ステップ（①設定→②tick→③flush→④リセット）を簡素化する
4. If 統合により不要になったコンポーネントまたは関数が存在する場合, the wintf system shall それらを削除する

### Requirement 6: 後方互換性とテスト

**Objective:** As a wintf 開発者, I want 統合後も既存テストが全てパスし、動作が維持される, so that リファクタリングによる退行が発生しない

#### Acceptance Criteria

1. The wintf system shall 既存の `feedback_loop_convergence_test.rs` の全テストケースをパスする
2. The wintf system shall `cargo test` の全テストを退行なくパスする
3. When `taffy_flex_demo` サンプルを実行した場合, the wintf system shall ウィンドウの移動・リサイズが従来と同等にスムーズに動作する
4. The wintf system shall 統合ゲートの単体テストを追加し、ゲート状態遷移の正確性を検証する
