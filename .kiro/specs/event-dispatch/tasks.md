# Implementation Plan

## Task Overview

| 項目 | 内容 |
|------|------|
| **Total Tasks** | 5 major tasks, 14 sub-tasks |
| **Requirements Coverage** | 1, 3, 4, 5, 6, 7, 8 (P0-P1) |
| **Excluded** | 2 (P2), 9 (P2) |

---

## Tasks

- [x] 1. Mouse → Pointer リネーム
  - 既存のマウス関連コンポーネント・システムを WinUI3 スタイルの Pointer 命名規則に統一する
  - `cargo build --all-targets` および `cargo test` が通ることを確認
  - _Requirements: 6.1, 6.2, 6.3_

- [x] 2. コア型定義

- [x] 2.1 (P) Phase\<T\> enum の実装
  - イベントフェーズとデータを一体化した Rust らしい enum 型を定義する
  - Tunnel/Bubble の2バリアントを持ち、パターンマッチで処理可能にする
  - value(), is_tunnel(), is_bubble() メソッドを実装する
  - Clone, Debug derive を付与する
  - _Requirements: 4.4, 4.5, 8.3_

- [x] 2.2 (P) EventHandler\<T\> 型エイリアスの定義
  - 汎用イベントハンドラの関数ポインタ型を定義する
  - 4引数（world, sender, entity, ev）、戻り値 bool のシグネチャとする
  - PointerEventHandler 型エイリアスを定義する
  - _Requirements: 3.2, 8.1, 8.2, 8.3, 8.4_

- [x] 3. ハンドラコンポーネント群

- [x] 3.1 (P) OnPointerPressed / OnPointerReleased コンポーネント
  - ポインター押下・解放イベントのハンドラを保持するコンポーネントを定義する
  - SparseSet ストレージで少数エンティティに最適化する
  - Clone, Copy derive を付与し、fnポインタ収集を効率化する
  - _Requirements: 3.1, 7.1, 7.2, 7.3, 7.4_

- [x] 3.2 (P) OnPointerEntered / OnPointerExited コンポーネント
  - ポインター進入・退出イベントのハンドラを保持するコンポーネントを定義する
  - SparseSet ストレージで少数エンティティに最適化する
  - Clone, Copy derive を付与する
  - _Requirements: 3.1, 7.1, 7.2, 7.3, 7.4_

- [x] 3.3 (P) OnPointerMoved コンポーネント
  - ポインター移動イベントのハンドラを保持するコンポーネントを定義する
  - SparseSet ストレージで少数エンティティに最適化する
  - Clone, Copy derive を付与する
  - _Requirements: 3.1, 7.1, 7.2, 7.3, 7.4_

- [x] 4. ディスパッチシステム

- [x] 4.1 親チェーン構築ロジック
  - ChildOf を辿って sender から root までのパスを構築する
  - Vec\<Entity\> 形式でバブリング順（sender → root）に格納する
  - _Requirements: 1.2, 1.3_

- [x] 4.2 Tunnel フェーズ実行
  - 親チェーンを逆順（root → sender）で走査しハンドラを呼び出す
  - 各呼び出し前にエンティティ存在チェックを行い、削除済みなら静かに終了する
  - ハンドラが true を返したら伝播停止する
  - _Requirements: 1.4, 1.5, 3.3, 5.5_

- [x] 4.3 Bubble フェーズ実行
  - 親チェーンを順方向（sender → root）で走査しハンドラを呼び出す
  - 各呼び出し前にエンティティ存在チェックを行い、削除済みなら静かに終了する
  - ハンドラが true を返したら伝播停止し、false なら次へ続行する
  - _Requirements: 1.1, 1.4, 1.5, 3.3, 3.4, 5.5_

- [x] 4.4 dispatch_pointer_events システム本体
  - 排他システム（&mut World）として実装する
  - 全 PointerState 保持エンティティを収集し、各々について独立にディスパッチする
  - 2パス方式（収集→実行）で同一フレーム内完結を保証する
  - _Requirements: 5.1, 5.2, 5.3_

- [x] 4.5 スケジュール登録
  - Input スケジュールに dispatch_pointer_events を追加する
  - process_pointer_buffers の後に実行されるよう順序制約を設定する
  - 既存のウィンドウシステムとの統合を確認する
  - _Requirements: 5.4, 6.1, 6.2, 6.4_

- [x] 5. 統合テスト

- [x] 5.1 バブリング・伝播停止テスト
  - 3階層のエンティティ階層でイベントが正しくバブリングすることを確認する
  - ハンドラが true を返した時点で後続ハンドラが呼ばれないことを確認する
  - Tunnel → Bubble の順序が正しいことを確認する
  - _Requirements: 1.1, 1.2, 1.3, 3.3, 3.4_

- [x] 5.2 複数ポインター・削除安全性テスト
  - 複数の PointerState が独立に処理されることを確認する
  - ハンドラ内で親エンティティを削除しても panic せず終了することを確認する
  - _Requirements: 5.2, 5.5_

---

## Notes

- Task 1 は既存コードのリネームであり、他タスクの前提となる
- Task 2, 3 は並列実行可能（型定義のみで相互依存なし）
- Task 4 は Task 1, 2, 3 完了後に実行
- Task 5 は全タスク完了後の統合テスト

---

## 🚧 継続作業コンテキスト (2025-12-04)

### 現在の状態
- **全タスク完了（コード実装済み）**
- **ビルド成功**: `cargo build --example taffy_flex_demo` 通過
- **テスト成功**: `cargo test --all-targets` 通過

### 残課題: PointerイベントのButtonBuffer→PointerState反映問題

#### 問題の症状
- `taffy_flex_demo.rs` でクリックイベントハンドラ（`OnPointerPressed`）が発火しない
- `[ButtonBuffer] record_button_down` ログは出力される（WM_LBUTTONDOWNは受信している）
- しかし `PointerState.left_down` / `right_down` が `false` のまま

#### 根本原因
1. **エンティティIDの不一致**: 
   - `WM_LBUTTONDOWN` 時: `hit_test` で特定されたエンティティ（例: 8v0）に `ButtonBuffer` を記録
   - `WM_MOUSEMOVE` 時: マウス移動で別エンティティ（例: 5v0）に `PointerState` が付与される
   - `process_pointer_buffers`: `PointerState` を持つ 5v0 の `ButtonBuffer` を探すが、ボタンは 8v0 に記録されているので見つからない

2. **タイミング問題**:
   - クリック→マウス移動→PointerState移動 が高速に発生
   - ボタン記録エンティティとPointerState保持エンティティが乖離

#### 現在の修正アプローチ（`dispatch.rs`）
- `dispatch_pointer_events` で `BUTTON_BUFFERS` を直接参照
- `PointerState` がないエンティティへのクリックも処理するよう修正済み

#### 次のステップ
1. **デバッグ**: `dispatch_pointer_events` の `[dispatch_pointer_events] Processing button event without PointerState` ログが出るか確認
2. **修正案A**: `handle_button_message` でボタンイベント時に `PointerState` を強制付与し、既存の `PointerState` を削除
3. **修正案B**: `BUTTON_BUFFERS` をクリア忘れがないか確認（`buf.reset()` 呼び出し）
4. **修正案C**: ボタン押下時に即座にイベントディスパッチ（`dispatch_pointer_events` を同期呼び出し）

#### 関連ファイル
- `crates/wintf/src/ecs/pointer/mod.rs`: `process_pointer_buffers`, `record_button_down`
- `crates/wintf/src/ecs/pointer/dispatch.rs`: `dispatch_pointer_events`
- `crates/wintf/src/ecs/window_proc/handlers.rs`: `handle_button_message`
- `crates/wintf/examples/taffy_flex_demo.rs`: デモ（イベントハンドラ例）

#### デモ起動方法
```powershell
# ログ付き起動（PowerShellから直接実行すると起動しない問題あり）
Start-Process -FilePath ".\target\debug\examples\taffy_flex_demo.exe" -NoNewWindow -Wait
```
