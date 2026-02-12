# Requirements Document

## Introduction

本仕様は、ECS ↔ Win32 間のフィードバックループ防止メカニズムを簡素化するリファクタリングである。現在、防止機構は以下の3+1層で実装されている：

| 層 | 実装名 | 役割 |
|----|--------|------|
| L1 | `WindowPosChanged` (ECS Component) | `WM_WINDOWPOSCHANGED` 処理中の `apply_window_pos_changes` スキップ |
| L2 | エコーバック検知 (`is_echo()`) | `last_sent_*` 値比較による間接的エコーバック検知 |
| L3 | `RefCell` 再入保護 | `try_borrow_mut()` 失敗時スキップ |
| 遅延実行 | `SetWindowPosCommand` (TLS キュー) | World 借用中の `SetWindowPos` 再入防止 |

この構造は堅牢だが冗長である。核心的な洞察として、`SetWindowPos` → `WM_WINDOWPOSCHANGED` は**同期呼び出し**（同一コールスタック内で発火）であるため、`SetWindowPos` をラッパー関数で囲み TLS フラグを管理するだけで、L1（`WindowPosChanged`）と L2（エコーバック検知）の両方を代替できる。

本仕様では以下を行う：
- `SetWindowPos` ラッパー関数を導入し、呼び出しスコープで TLS フラグを自動管理する
- `WindowPosChanged` ECS コンポーネントを削除する
- `last_sent_*` / `is_echo()` エコーバック検知を削除する
- `SetWindowPosCommand` 遅延実行キューは維持し、`flush()` 内でラッパーを使用する
- `DpiChangeContext`（TLS）は `WndProc` コールスタック固有の性質を持つため維持する
- L3（`RefCell` 再入保護）は `Rc<RefCell<EcsWorld>>` アーキテクチャ固有のため維持する

**親仕様**: `dpi-coordinate-transform-survey` (report.md §1.2 項目4, §6.4.4, §8.2 Step 4)
**前提条件**: `wintf-fix3-sync-arrangement-enable` 完了（逆同期が有効化済み）

## Requirements

### Requirement 1: SetWindowPos ラッパーによるフィードバック防止

**Objective:** As a wintf 開発者, I want `SetWindowPos` をラッパー関数で囲み TLS フラグでフィードバックを制御する, so that `WindowPosChanged` コンポーネントと `is_echo()` エコーバック検知の両方が不要になり、防止メカニズムが根本的に簡素化される

#### Acceptance Criteria

1. The wintf system shall `SetWindowPos` Win32 API 呼び出しをラッパー関数で囲み、呼び出し前に TLS フラグを ON、完了後に OFF とする
2. While ラッパーの TLS フラグが ON の状態（＝`SetWindowPos` のコールスタック内）で `WM_WINDOWPOSCHANGED` が発火した場合, the wintf system shall これを自アプリ由来の echo と判定し、`apply_window_pos_changes` での再送信をスキップする
3. The wintf system shall `SetWindowPosCommand` 遅延実行キューを維持し、`flush()` 内でラッパー関数経由で `SetWindowPos` を実行する
4. The wintf system shall `WindowPosChanged` ECS コンポーネント、`last_sent_position` / `last_sent_size` フィールド、`is_echo()` メソッドを削除する
5. When `WM_DPICHANGED` ハンドラが `SetWindowPos` を呼び出す場合, the wintf system shall 同じラッパー関数を使用し、フィードバック防止を統一する

### Requirement 2: DpiChangeContext の維持

**Objective:** As a wintf 開発者, I want `DpiChangeContext`（TLS）を現行実装のまま維持する, so that `WM_DPICHANGED` → `SetWindowPos` → `WM_WINDOWPOSCHANGED` の同期コールチェーンで DPI 値の伝達が正しく継続される

#### Acceptance Criteria

1. The wintf system shall `DpiChangeContext` の TLS 構造（`set()` / `take()` API）を変更せずに維持する
2. When `WM_DPICHANGED` を受信した場合, the wintf system shall `DpiChangeContext::set()` で新 DPI 値を TLS に格納する
3. When `WM_WINDOWPOSCHANGED` ハンドラが実行された場合, the wintf system shall `DpiChangeContext::take()` で DPI 値を取得し、DPI コンポーネントを即時更新する
4. The wintf system shall 統合ゲートシステムと `DpiChangeContext` の責務を明確に分離する（ゲートは位置・サイズ同期、DpiChangeContext は DPI 値伝達）

### Requirement 3: フィードバックループ防止の正確性

**Objective:** As a wintf ユーザー, I want ウィンドウ操作時にフィードバックループが発生しない, so that ウィンドウの移動・リサイズ・DPI 変更がスムーズに動作する

#### Acceptance Criteria

1. When ユーザーが Win32 経由でウィンドウを移動・リサイズした場合, the wintf system shall 1フレーム以内にフィードバックを収束させ、無限ループを発生させない
2. When ECS 側からプログラム的にウィンドウ位置を変更した場合, the wintf system shall `SetWindowPos` → `WM_WINDOWPOSCHANGED` → ECS 更新の経路で1往復のみで収束する
3. When DPI 変更（モニタ間移動）が発生した場合, the wintf system shall `WM_DPICHANGED` → `SetWindowPos` → `WM_WINDOWPOSCHANGED` チェーンを正しく処理し、フィードバックループを発生させない
4. While 複数ウィンドウが同時に存在する状態で, when いずれかのウィンドウが移動・リサイズされた場合, the wintf system shall 他のウィンドウに不要なフィードバック連鎖を波及させない

### Requirement 4: コード簡素化と保守性向上

**Objective:** As a wintf 開発者, I want ラッパー方式の導入によりフィードバック防止の関連コードが削減される, so that 将来の機能追加やバグ修正が容易になる

#### Acceptance Criteria

1. The wintf system shall フィードバック防止の状態管理を TLS ラッパーフラグ + 遅延実行キューの2点に集約する（現在: `WindowPosChanged` コンポーネント + `WINDOW_POS_COMMANDS` TLS + `last_sent_*` エコーバックフィールド の3箇所）
2. The wintf system shall ラッパー関数の動作原理と TLS フラグのライフサイクルを doc comment で文書化する
3. The wintf system shall `WM_WINDOWPOSCHANGED` ハンドラ内の4ステップ（①設定→②tick→③flush→④リセット）からステップ④（第2World借用によるフラグリセット）を削除し簡素化する

### Requirement 5: 後方互換性とテスト

**Objective:** As a wintf 開発者, I want リファクタリング後も既存テストが全てパスし、動作が維持される, so that リファクタリングによる退行が発生しない

#### Acceptance Criteria

1. The wintf system shall 既存の `feedback_loop_convergence_test.rs` の全テストケースをパスする
2. The wintf system shall `cargo test` の全テストを退行なくパスする
3. When `taffy_flex_demo` サンプルを実行した場合, the wintf system shall ウィンドウの移動・リサイズが従来と同等にスムーズに動作する
4. The wintf system shall ラッパー関数の TLS フラグ動作を検証する単体テストを追加する
