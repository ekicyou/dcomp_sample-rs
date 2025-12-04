# Implementation Plan

## Task Overview

| 項目 | 内容 |
|------|------|
| **Total Tasks** | 6 major tasks, 15 sub-tasks |
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

- [ ] 6. GlobalArrangement.bounds と DPI スケールの整合性修正

- [ ] 6.1 スケール適用タイミングの設計見直し
  - 現状: Window の bounds.left が (80, 80) になる（期待値: 125, 125）
  - LayoutRoot は物理ピクセル座標系（スケール 1.0）
  - Window の **内部** に入って初めて DPI スケールが適用されるべき
  - 「移動してからスケール」の考え方で bounds 計算を再設計する

- [ ] 6.2 GlobalArrangement::mul の bounds 計算修正
  - 現在の修正: `offset × parent_scale` で scaled_offset を計算
  - 問題: Window の場合、parent(LayoutRoot).scale = 1.0 なので offset がスケールされない
  - しかし Window 自身の scale (1.25) を適用する必要がある
  - 解決策: `offset × child.scale` を使うか、スケール適用のセマンティクスを再検討

- [ ] 6.3 hierarchical_bounds_test.rs の期待値調整
  - 新しいスケール適用ロジックに合わせてテスト期待値を更新
  - 全テストが通ることを確認

---

## Notes

- Task 1 は既存コードのリネームであり、他タスクの前提となる
- Task 2, 3 は並列実行可能（型定義のみで相互依存なし）
- Task 4 は Task 1, 2, 3 完了後に実行
- Task 5 は全タスク完了後の統合テスト

---

## ✅ 完了 (2025-12-04)

### 最終状態
- **全タスク完了（コード実装済み）**
- **ビルド成功**: `cargo build --example taffy_flex_demo` 通過
- **テスト成功**: `cargo test --all-targets` 通過
- **動作確認成功**: `taffy_flex_demo.exe` でクリックイベントが正常に発火

### 解決した課題: PointerイベントのButtonBuffer→PointerState反映問題

#### 問題の症状（解決済み）
- `taffy_flex_demo.rs` でクリックイベントハンドラ（`OnPointerPressed`）が発火しなかった
- 原因: `process_pointer_buffers` が `buf.reset()` した後に `dispatch_pointer_events` が実行されていた

#### 解決策
1. **スケジュール順序変更** (`world.rs`):
   - `dispatch_pointer_events` → `process_pointer_buffers` の順に変更
   - イベントディスパッチがボタンバッファ処理の前に実行されるように

2. **dispatch_pointer_events 修正** (`dispatch.rs`):
   - BUTTON_BUFFERS から直接ボタンイベントを取得
   - ディスパッチ完了後に BUTTON_BUFFERS をリセット
   - PointerState の有無に関わらず OnPointerPressed をディスパッチ

3. **process_pointer_buffers 修正** (`mod.rs`):
   - BUTTON_BUFFERS のリセットを削除（dispatch_pointer_events が担当）

#### デモ起動方法
```powershell
$env:RUST_LOG="info"; .\target\debug\examples\taffy_flex_demo.exe
```

---

## 🔴 未解決課題: ヒットテスト座標ずれ問題 (2025-12-04)

### 問題の症状
- BlueBoxの**見た目の位置**と**hit_testで判定される位置**がずれている
- 青の左上しか反応しない（右側や中央をクリックしてもContainerにヒットする）
- DPIスケール 125% (1.25) 環境で約77ピクセルのずれが発生

### 調査結果

#### 座標系の整理
1. **WM_LBUTTONDOWN の lparam**: クライアント座標（物理ピクセル）
2. **WindowPos.position**: クライアント領域左上のスクリーン座標（物理ピクセル）
3. **GlobalArrangement.bounds**: スクリーン座標（物理ピクセル）
4. **Arrangement.offset**: DIP座標（論理ピクセル）

#### 問題箇所の特定

**Visual offset と GlobalArrangement.bounds の不一致**:

```
Container:
  visual_offset_x = 12.5  (10 DIP × 1.25 scale)
  bounds_left = 135.0     (Window 125 + Container 10)

BlueBox:
  visual_offset_x = 375.0 (300 DIP × 1.25 scale)
  bounds_left = 435.0
```

**計算の差異**:
- Visual は親Visualからの相対オフセット（DirectComposition が階層処理）
- BlueBox の実際のスクリーン位置 = Container位置 + BlueBox offset = 137.5 + 375 = **512.5**
- しかし bounds_left = 435.0
- **差 = 512.5 - 435 = 77.5 ピクセル** ← これがずれの原因

#### 根本原因

`Arrangement` → `Matrix3x2` 変換（arrangement.rs 行177-184）:

```rust
impl From<Arrangement> for Matrix3x2 {
    fn from(arr: Arrangement) -> Self {
        let scale: Matrix3x2 = arr.scale.into();
        let translation: Matrix3x2 = arr.offset.into();
        // 現在: translation * scale
        translation * scale
    }
}
```

この行列積の順序では、**offset（DIP座標）に scale が適用されない**。

- `translation * scale` = 先に scale 適用、次に translation 適用
- しかし translation（DIPオフセット）自体にはスケールがかからない
- 結果として bounds 計算で DIP offset がそのまま使われる

**一方 Visual offset 計算**（graphics/systems.rs）:

```rust
let offset_x = arrangement.offset.x * scale_x;
```

こちらは正しく DIP × scale = 物理ピクセル に変換している。

#### 試みた修正と結果

1. **行列順序を `scale * translation` に変更**
   - Window bounds.left が 64 になった（125 が期待値）
   - LayoutRoot の仮想デスクトップ座標が影響している可能性
   - 単純な順序変更では解決しない

2. **`sync_window_arrangement_from_window_pos` システム追加**
   - WindowPos.position → Arrangement.offset の同期を試みた
   - WindowPosChanged フラグを使ってもタイミング問題で機能しない
   - 毎フレーム実行にすると Window の offset が DIP に戻されてしまう
   - 一旦無効化して元に戻した

### 現在の状態 (2025-12-04 21:40)
- **✅ 問題解決**: hit_test のクリック判定が正しく動作するようになった
- 行列順序: `translation * scale`（変更なし）
- `GlobalArrangement::mul` を修正: bounds 計算で子の offset に親の scale を適用
- 全テスト成功: `cargo test --all-targets` パス

### 修正内容

#### `GlobalArrangement::mul` (arrangement.rs)

修正前:
```rust
let child_matrix: Matrix3x2 = rhs.into();
let result_transform = self.transform * child_matrix;
let child_bounds = rhs.local_bounds();
let result_bounds = transform_rect_axis_aligned(&child_bounds, &result_transform);
```

修正後:
```rust
// transform計算（元のオフセットを使用）
let child_matrix: Matrix3x2 = rhs.into();
let result_transform = self.transform * child_matrix;

// bounds計算
// 子のオフセットに親のスケールを適用してからローカル座標を変換
let parent_scale_x = self.transform.M11;
let parent_scale_y = self.transform.M22;
let scaled_offset = Offset {
    x: rhs.offset.x * parent_scale_x,
    y: rhs.offset.y * parent_scale_y,
};

// bounds.left = parent.bounds.left + scaled_offset.x
// bounds.right = bounds.left + size * result_scale
let result_bounds = D2DRect {
    left: self.bounds.left + scaled_offset.x,
    top: self.bounds.top + scaled_offset.y,
    right: self.bounds.left + scaled_offset.x + rhs.size.width * result_transform.M11,
    bottom: self.bounds.top + scaled_offset.y + rhs.size.height * result_transform.M22,
};
```

### 残課題 (2025-12-04 21:46)

現在の状態:
- **クリック判定は動作する**: BlueBox のクリックイベントは正しく発火
- **Window の bounds.left が (80, 80)**: 期待値は (125, 125)
- **テストは全て通る**: `cargo test --all-targets` パス

#### 問題の核心

`offset × parent_scale` のロジックでは:
- Window: `100 × 1.0 = 100` (LayoutRoot.scale = 1.0)
- しかし実際は `100 × 1.25 = 125` になるべき

**スケール適用のセマンティクス**:
- LayoutRoot は物理ピクセル座標系（マルチモニター環境でモニターごとに DPI が異なる）
- Window の **内部に入って初めて** DPI スケールが適用される
- 「移動してからスケール」の順序で考えるべき

#### 解決方針

`offset × child.scale` を使うべきか？
- Window.offset = 100 DIP × Window.scale = 1.25 → 125 物理ピクセル
- Container.offset = 10 DIP × Container.scale = 1.0 だが、親(Window)のスケールが既に適用済み

より正確には:
- **Window**: `offset × self.scale`（DIP を物理ピクセルに変換）
- **Window の子**: `offset × parent_scale`（親座標系で既にスケール済み）

次回セッションで Task 6 を実装する際に検討。

### 関連ファイル
- `crates/wintf/src/ecs/layout/arrangement.rs` - Matrix3x2 変換、GlobalArrangement::mul
- `crates/wintf/src/ecs/layout/rect.rs` - transform_rect_axis_aligned
- `crates/wintf/src/ecs/layout/systems.rs` - sync_window_arrangement_from_window_pos (追加済み、無効化中)
- `crates/wintf/src/ecs/graphics/systems.rs` - visual_property_sync_system
- `crates/wintf/src/ecs/layout/hit_test.rs` - hit_test_in_window
- `crates/wintf/tests/hierarchical_bounds_test.rs` - bounds 計算テスト

### デバッグログ追加済み
- `handle_button_message`: client_x, client_y, screen_x, screen_y, bounds をログ出力
- `visual_property_sync_system`: visual_offset と bounds の比較ログ（現在コメントアウト）
- `mark_dirty_arrangement_trees`: changed_count ログ（現在コメントアウト）
