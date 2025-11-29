# Requirements Document

## Project Description (Input)
レイアウト計算結果をグラフィックスリソース（Visual、Surface、WindowPos）に正しく伝播させ、双方向の同期とループ回避を実現する。

## Introduction
本機能は、wintfフレームワークにおいて、Taffyレイアウトエンジンの計算結果をDirectCompositionのグラフィックスリソースに正確に伝播し、ウィンドウシステムとの双方向同期を確立します。

### 現在の問題
1. **Surfaceサイズ不整合**: Taffy計算は`BoxSize (800×600)`を使用するが、実際のSurfaceは`Visual.size (782×553)`で作成され、Green rectangleが8pxしか表示されない
2. **仮実装への依存**: `window_system.rs`で`GetClientRect`の結果を`Visual.size`に直接代入しており、レイアウト計算結果が無視される
3. **情報フローの欠如**: レイアウト → Visual → Surface → WindowPos の伝播経路が未実装
4. **無限ループリスク**: ウィンドウリサイズ時のエコーバックが検知されず、無限ループの可能性がある

### 解決アプローチ
- **正しい情報フロー**: `GlobalArrangement` → `Visual.size` → `Surface` → `WindowPos` → `SetWindowPos` の一方向伝播
- **エコーバック検知**: `WindowPos`に送信値キャッシュを追加し、`WM_WINDOWPOSCHANGED`でエコーバックをスキップ
- **双方向同期**: レイアウト起点とユーザー操作起点の両方向を正しく処理
- **変更検知最適化**: `PartialEq` deriveによる自動最適化（Bevy ECS 0.14+）

## Requirements

### Requirement 1: Visual サイズ同期システム
**Objective:** 開発者として、Taffyレイアウト計算結果が正確にDirectComposition Visualのサイズに反映されることを保証したい。これにより、レイアウトシステムがグラフィックス描画の信頼できる唯一の情報源となる。

#### Acceptance Criteria
1. When `GlobalArrangement`コンポーネントが変更され、かつエンティティが`LayoutRoot`マーカーを持つ場合、wintfシステムは`GlobalArrangement.bounds`から幅と高さを計算し、`Visual.size`を更新しなければならない
2. The wintfシステムは`sync_visual_from_layout_root`システムを`PostLayout`スケジュールに配置し、`propagate_global_arrangements`システムの後に実行しなければならない
3. When `Visual.size`を更新する際、wintfシステムは以下の計算式を使用しなければならない:
   - `visual.size.X = (bounds.right - bounds.left) as f32`
   - `visual.size.Y = (bounds.bottom - bounds.top) as f32`
4. The wintfシステムは`Query<(&GlobalArrangement, &mut Visual), (With<LayoutRoot>, Changed<GlobalArrangement>)>`クエリを使用し、変更されたエンティティのみを処理しなければならない
5. When `LayoutRoot`を持たないエンティティの`GlobalArrangement`が変更された場合、wintfシステムは`Visual.size`を更新してはならない

### Requirement 2: Surface リサイズシステム
**Objective:** 開発者として、Visual サイズの変更時に Surface が自動的に再作成されることを保証したい。これにより、描画領域が常にレイアウト計算結果と一致する。

#### Acceptance Criteria
1. When `Visual`コンポーネントのサイズが変更された場合、wintfシステムは対応する`SurfaceGraphics`を持つエンティティの Surface を再作成しなければならない
2. The wintfシステムは`resize_surface_from_visual`システムを`PostLayout`スケジュールに配置し、`sync_visual_from_layout_root`システムの後に実行しなければならない
3. When Surface を再作成する際、wintfシステムは`Visual.size`と`SurfaceGraphics.size`を比較し、異なる場合のみ`create_surface_for_window`を呼び出さなければならない
4. When Surface 再作成が完了した場合、wintfシステムは`SurfaceGraphics.size`を新しい`Visual.size`で更新しなければならない
5. If Surface 再作成に失敗した場合、wintfシステムはエラーをログに記録し、`SurfaceGraphics`を無効な状態にマークしなければならない

### Requirement 3: WindowPos 同期システム
**Objective:** 開発者として、レイアウト計算結果がウィンドウの位置とサイズに反映されることを保証したい。これにより、ECSエンティティとウィンドウシステムの整合性が保たれる。

#### Acceptance Criteria
1. When `GlobalArrangement`または`Visual`が変更され、かつエンティティが`Window`コンポーネントを持つ場合、wintfシステムは`WindowPos`コンポーネントを更新しなければならない
2. The wintfシステムは`sync_window_pos`システムを`PostLayout`スケジュールに配置し、`resize_surface_from_visual`システムの後に実行しなければならない
3. When `WindowPos`を更新する際、wintfシステムは以下の値を設定しなければならない:
   - `window_pos.position = Some(POINT { x: bounds.left as i32, y: bounds.top as i32 })`
   - `window_pos.size = Some(SIZE { cx: visual.size.X as i32, cy: visual.size.Y as i32 })`
4. The wintfシステムは`Query<(&GlobalArrangement, &Visual, &mut WindowPos), (With<Window>, Or<(Changed<GlobalArrangement>, Changed<Visual>)>)>`クエリを使用しなければならない
5. When `Window`コンポーネントを持たないエンティティの場合、wintfシステムは`WindowPos`を更新してはならない

### Requirement 4: エコーバック検知機構
**Objective:** 開発者として、`SetWindowPos`の呼び出しによって生成される`WM_WINDOWPOSCHANGED`メッセージをエコーバックとして検知し、無限ループを防止したい。

#### Acceptance Criteria
1. The wintfシステムは`WindowPos`コンポーネントに`last_sent_position: (i32, i32)`と`last_sent_size: (i32, i32)`フィールドを追加しなければならない
2. When `apply_window_pos_changes`システムが`SetWindowPos`を呼び出す際、wintfシステムは送信した位置とサイズを`bypass_change_detection()`を使用して`last_sent_position`と`last_sent_size`に記録しなければならない
3. When `WM_WINDOWPOSCHANGED`メッセージを受信した際、wintfシステムは受信した位置とサイズを`last_sent_position`と`last_sent_size`と比較しなければならない
4. If 受信した位置とサイズが`last_sent_position`と`last_sent_size`に一致する場合、wintfシステムは`WindowPos`コンポーネントの更新をスキップし、処理を終了しなければならない
5. If 受信した位置とサイズが`last_sent_position`と`last_sent_size`と異なる場合、wintfシステムは`WindowPos`を更新し、レイアウト再計算をトリガーしなければならない

### Requirement 5: SetWindowPos 実行システム
**Objective:** 開発者として、ECS内の`WindowPos`の変更が実際のウィンドウシステムに反映されることを保証したい。エコーバック検知機構により無限ループを防ぐ。

#### Acceptance Criteria
1. When `WindowPos`コンポーネントが変更された場合、wintfシステムは対応するウィンドウハンドル(HWND)に対して`SetWindowPos`を呼び出さなければならない
2. The wintfシステムは`apply_window_pos_changes`システムを`PostLayout`スケジュールに配置し、`sync_window_pos`システムの後に実行しなければならない
3. When `SetWindowPos`を呼び出す際、wintfシステムは`WindowPos.position`と`WindowPos.size`の値を使用しなければならない
4. When `SetWindowPos`呼び出しが完了した際、wintfシステムは`WindowPos.bypass_change_detection()`を使用して`last_sent_position`と`last_sent_size`を更新しなければならない
5. If `SetWindowPos`が失敗した場合、wintfシステムはエラーをログに記録し、`last_sent_position`と`last_sent_size`を更新してはならない

### Requirement 6: WM_WINDOWPOSCHANGED メッセージハンドリング
**Objective:** 開発者として、外部からのウィンドウサイズ変更（ユーザー操作、システムコマンド）がECSレイアウトシステムに正しく伝播されることを保証したい。

#### Acceptance Criteria
1. When `WM_WINDOWPOSCHANGED`メッセージを受信した場合、wintfシステムは`WINDOWPOS`構造体から位置とサイズを抽出しなければならない
2. When 受信した位置とサイズがエコーバック（`last_sent_position`/`last_sent_size`と一致）の場合、wintfシステムは処理をスキップし、`DefWindowProc`を呼び出してはならない
3. When 受信した位置とサイズが外部変更（エコーバックでない）の場合、wintfシステムは`WinState` traitを通じてECS `World`にアクセスし、対応するエンティティの`WindowPos`コンポーネントを更新しなければならない
4. When 外部変更による`WindowPos`更新時、wintfシステムは対応する`BoxSize`コンポーネントも更新し、Taffyレイアウト再計算をトリガーしなければならない
5. The wintfシステムは`WM_WINDOWPOSCHANGED`を処理した後、`DefWindowProc`を呼び出さず、`LRESULT(0)`を返さなければならない

### Requirement 7: レガシーコードの削除と初期化ロジックの改善
**Objective:** 開発者として、仮実装として追加されたレガシーコードを削除し、新しい同期システムに完全移行したい。また、ウィンドウ初期化時の`Visual.size`設定をレイアウトシステムに委譲したい。

#### Acceptance Criteria
1. The wintfシステムは`window_system.rs`の`create_windows`システム内の`GetClientRect`呼び出しと、その結果を`Visual.size`に代入するコードを削除しなければならない
2. When ウィンドウ初期化時、wintfシステムは`Visual`コンポーネントをデフォルト値（または適切な初期値）で作成し、実際のサイズは`sync_visual_from_layout_root`システムに計算させなければならない
3. The wintfシステムは`WM_MOVE`および`WM_SIZE`メッセージハンドラを削除しなければならない（`WM_WINDOWPOSCHANGED`で代替）
4. The wintfシステムは削除後、既存のテストスイートが全て通過することを確認しなければならない
5. The wintfシステムはレガシーコード削除に関するドキュメント（特にREADME.md）を更新しなければならない

### Requirement 8: 変更検知最適化
**Objective:** 開発者として、コンポーネントの値が実際に変化した場合のみ変更フラグが立つことを保証したい。これにより、不要な再計算を防ぎパフォーマンスを向上させる。

#### Acceptance Criteria
1. The wintfシステムは`Visual`コンポーネントに`#[derive(PartialEq)]`を追加しなければならない（`WindowPos`は既に`PartialEq`実装済み）
2. When コンポーネントに同じ値を設定した場合、Bevy ECSは変更フラグを立てず、`Changed<T>`フィルタで検出されてはならない
3. The wintfシステムは`PartialEq` deriveにより、明示的な値比較コードが不要になることを確認しなければならない
4. The wintfシステムは変更検知最適化により、`SetWindowPos`の不要な呼び出しが削減されることをテストで確認しなければならない
5. When `Visual`のすべてのフィールド（`is_visible`, `opacity`, `transform_origin`, `size`）を比較対象とし、いずれかが変更された場合に変更検知が動作しなければならない

### Requirement 9: 双方向同期の確立
**Objective:** 開発者として、レイアウトシステムからウィンドウへの同期と、ウィンドウからレイアウトシステムへの同期が両方とも正しく機能することを保証したい。

#### Acceptance Criteria
1. When レイアウト計算によって`GlobalArrangement`が変更された場合、wintfシステムは変更を`Visual` → `Surface` → `WindowPos` → `SetWindowPos`の順に伝播しなければならない
2. When ユーザーがウィンドウをリサイズした場合、wintfシステムは`WM_WINDOWPOSCHANGED` → `WindowPos` → `BoxSize`の順に更新し、レイアウト再計算をトリガーしなければならない
3. When どちらの方向の同期でも、wintfシステムはエコーバック検知により無限ループを防止しなければならない
4. The wintfシステムは双方向同期が正しく機能することを統合テストで確認しなければならない
5. The wintfシステムは双方向同期の情報フロー図をドキュメントに含めなければならない

### Requirement 10: テスタビリティとデバッグサポート
**Objective:** 開発者として、同期システムの動作を検証し、問題を診断できるテストとデバッグ機能を持ちたい。

#### Acceptance Criteria
1. The wintfシステムは`WindowPos::is_echo(position, size)`メソッドを提供し、エコーバック判定ロジックを単体テスト可能にしなければならない
2. The wintfシステムはレイアウト → Visual → Surface → WindowPos の完全フローを検証する統合テストを提供しなければならない
3. The wintfシステムは`taffy_flex_demo.rs`サンプルでウィンドウリサイズ動作を手動テスト可能にしなければならない
4. When デバッグモードが有効な場合、wintfシステムは同期システムの各ステップ（Visual更新、Surface再作成、SetWindowPos呼び出し、エコーバック検知）をログ出力しなければならない
5. The wintfシステムはSuccess Criteriaに記載された全ての項目を検証するテストを提供しなければならない

## In-Scope Items
本仕様で実装する項目：

### 既存システムの削除
#### `init_window_surface`システムの削除
**理由**:

- `visual_resource_management_system`がすでに`Added<Visual>`トリガーでSurfaceを作成している
- `init_window_surface`は`Visual.size`からSurfaceサイズを取得して再作成を試みるが、初回作成時はサイズが一致しているため実質的に何も実行されない
- 役割が完全に重複しており、PostLayoutスケジュールのムダな実行コストとなっている

**削除内容**:

1. `crates/wintf/src/ecs/graphics/systems.rs`の`init_window_surface`関数定義を削除
2. `crates/wintf/src/ecs/world.rs`のPostLayoutスケジュールから登録を削除
3. 関連するヘルパー関数`create_surface_for_window`は`resize_surface_from_visual`で再利用するため保持

**影響分析**:

- ✅ `visual_resource_management_system`が初回Surface作成を担当（既存）
- ✅ `resize_surface_from_visual`が`Changed<Visual>`でリサイズを担当（新規実装）
- ✅ `cleanup_graphics_needs_init`は引き続き機能（SurfaceGraphics存在チェックのみ）
- ⚠️ 削除前に`init_window_surface`が実際に実行されていないことをログで確認すること

## Non-Goals
本仕様の範囲外とする項目：

- アニメーション中のフレーム間補間処理
- 複数モニター環境でのDPI対応（別仕様で対応）
- ウィンドウの最小化・最大化時の特殊処理
- レイアウト計算のパフォーマンス最適化（別仕様: render-dirty-tracking）
- Visual階層の親子関係同期（別仕様: visual-tree-synchronization）

## Success Criteria
以下のすべてが達成された場合、本仕様の実装は成功とみなされる：

1. ✅ Taffy計算結果（例: 800×600）がSurfaceサイズに正確に反映される
2. ✅ Green rectangleが期待通りの高さ（45px）で表示される
3. ✅ ユーザーによるウィンドウリサイズ時に無限ループが発生しない
4. ✅ レイアウトシステム起点のウィンドウリサイズ時に無限ループが発生しない
5. ✅ `WM_WINDOWPOSCHANGED`でエコーバックが正しくスキップされる
6. ✅ 既存のすべてのテストが通過する
7. ✅ 新規追加した統合テストが通過する

## Dependencies
本仕様は以下に依存する：

- **taffy-layout-integration**（必須）: Taffyレイアウトエンジンの統合が完了していること
- **bevy_ecs 0.14+**: `PartialEq` deriveによる変更検知最適化機能
- **Windows API**: `SetWindowPos`, `WM_WINDOWPOSCHANGED`, `WINDOWPOS`構造体

## References
- **WinUI3のループ回避実装**: `microsoft/microsoft-ui-xaml`リポジトリの`DesktopWindowImpl.cpp`
- **Bevy ECS変更検知**: `Mut::bypass_change_detection()`メソッドとPartialEqによるDrop時最適化
- **Windowsメッセージ処理**: `WM_WINDOWPOSCHANGED` (0x0047)の仕様
- **現在の問題詳細**: `.kiro/specs/layout-to-graphics-sync/README.md`

---

**Note**: 本要件定義は、README.mdに記載された詳細な技術仕様と背景知識に基づいて作成されています。実装時はREADME.mdの「Background Knowledge」セクションも参照してください。
