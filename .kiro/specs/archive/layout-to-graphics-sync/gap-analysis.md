# Gap Analysis: layout-to-graphics-sync

**Feature**: layout-to-graphics-sync  
**Analysis Date**: 2025-11-24  
**Status**: Requirements Generated (Not Yet Approved)

---

## 1. 分析概要

### スコープ
Taffyレイアウト計算結果をDirectCompositionグラフィックスリソース（Visual、Surface、WindowPos）に正確に伝播し、ウィンドウシステムとの双方向同期を確立する機能。現在発生している**Surfaceサイズ不整合**（Green rectangleが8pxしか表示されない）の解決が主目的。

### 主要な課題
1. **情報フローの欠如**: `GlobalArrangement` → `Visual.size` → `Surface` → `WindowPos` → `SetWindowPos` の伝播経路が未実装
2. **仮実装への依存**: `GetClientRect`の結果を直接`Visual.size`に代入（レイアウト計算結果が無視される）
3. **無限ループリスク**: ウィンドウリサイズ時のエコーバック検知機構が未実装

### 推奨アプローチ
**Option A（拡張）+ 一部新規システム追加**のハイブリッドアプローチ。既存の`PostLayout`スケジュールに新規システムを追加し、既存コンポーネントを拡張する戦略。

---

## 2. 現状調査

### 2.1 既存のコンポーネント

#### Visual コンポーネント
- **場所**: `crates/wintf/src/ecs/graphics/components.rs:158`
- **現状**:
  ```rust
  #[derive(Component, Debug, Clone)]
  pub struct Visual {
      pub is_visible: bool,
      pub opacity: f32,
      pub transform_origin: Vector2,
      pub size: Vector2,  // ← これをレイアウトから同期する
  }
  ```
- **問題**: `PartialEq`が未実装（変更検知最適化不可）
- **必要な変更**: `#[derive(PartialEq)]`を追加

#### WindowPos コンポーネント
- **場所**: `crates/wintf/src/ecs/window.rs:174`
- **現状**:
  ```rust
  #[derive(Component, Debug, Clone, Copy, PartialEq)]
  pub struct WindowPos {
      pub zorder: ZOrder,
      pub position: Option<POINT>,
      pub size: Option<SIZE>,
      // ... 多数のフラグ
  }
  ```
- **良い点**: 既に`PartialEq`を実装済み
- **必要な変更**: `last_sent_position`と`last_sent_size`フィールドの追加（エコーバック検知用）
- **既存メソッド**: `set_window_pos(&self, hwnd: HWND)`が既に実装されている（line 394）

#### SurfaceGraphics コンポーネント
- **場所**: `crates/wintf/src/ecs/graphics/components.rs:102`
- **現状**:
  ```rust
  #[derive(Component, Debug, Default)]
  pub struct SurfaceGraphics {
      inner: Option<IDCompositionSurface>,
      pub size: (u32, u32),  // ← これをVisual.sizeと同期
  }
  ```
- **良い点**: サイズフィールドが既に存在
- **問題**: `PartialEq`未実装（必要性は低い）

#### GlobalArrangement コンポーネント
- **場所**: `crates/wintf/src/ecs/layout/arrangement.rs:75`
- **現状**:
  ```rust
  #[derive(Component, Debug, Clone, Copy, PartialEq)]
  pub struct GlobalArrangement {
      pub transform: Matrix3x2,
      pub bounds: D2D_RECT_F,  // ← このboundsからVisual.sizeを計算
  }
  ```
- **良い点**: 既に`PartialEq`実装済み、変更検知対応可能

#### BoxSize コンポーネント
- **場所**: `crates/wintf/src/ecs/layout/high_level.rs:299`
- **現状**:
  ```rust
  #[derive(Debug, Clone, Copy, PartialEq, Component)]
  pub struct BoxSize {
      pub width: Option<Dimension>,
      pub height: Option<Dimension>,
  }
  ```
- **良い点**: 既に`PartialEq`実装済み

#### LayoutRoot マーカー
- **場所**: `crates/wintf/src/ecs/layout/mod.rs:111`
- **現状**: `pub struct LayoutRoot;`（マーカーコンポーネント）
- **用途**: Taffyレイアウト計算のルートエンティティを識別

### 2.2 既存のシステム

#### propagate_global_arrangements システム
- **場所**: `crates/wintf/src/ecs/layout/systems.rs:55`
- **スケジュール**: `PostLayout`（line 202 in world.rs）
- **機能**: 親から子へ`GlobalArrangement`を階層的に伝播
- **重要**: 新規システムはこの後に実行する必要がある

#### init_window_surface システム
- **場所**: `crates/wintf/src/ecs/graphics/systems.rs:391`
- **スケジュール**: `PostLayout`
- **機能**: `Visual.size`からSurfaceを作成・リサイズ
- **現状の問題**:
  ```rust
  let (width, height) = visual_component
      .map(|v| (v.size.X as u32, v.size.Y as u32))
      .unwrap_or((800, 600));
  ```
  - `Visual.size`を読み取っているが、`Visual.size`が`GetClientRect`で設定されている（仮実装）
  - 本来は`GlobalArrangement`から`Visual.size`を設定すべき

#### create_windows システム
- **場所**: `crates/wintf/src/ecs/window_system.rs:81`
- **スケジュール**: `UISetup`
- **問題のコード**:
  ```rust
  let mut rect = RECT::default();
  unsafe {
      let _ = GetClientRect(hwnd, &mut rect);
  }
  let width = (rect.right - rect.left) as f32;
  let height = (rect.bottom - rect.top) as f32;
  
  commands.entity(entity).insert((
      // ...
      crate::ecs::graphics::Visual {
          size: Vector2 { X: width, Y: height },
          ..Default::default()
      },
  ));
  ```
  - **削除対象**: `GetClientRect`呼び出しと、その結果を`Visual.size`に設定する処理
  - **代替**: `BoxSize`から初期`Visual.size`を設定

### 2.3 メッセージハンドリング

#### WM_WINDOWPOSCHANGED ハンドラ
- **場所**: `crates/wintf/src/win_message_handler.rs:1052`
- **現状**: 空実装（`None`を返すだけ）
  ```rust
  fn WM_WINDOWPOSCHANGED(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
      None
  }
  ```
- **必要な実装**: `WINDOWPOS`構造体を解析し、エコーバック検知とECS更新

#### WM_MOVE, WM_SIZE ハンドラ
- **場所**: `crates/wintf/src/win_message_handler.rs:616, 932`
- **現状**: 空実装（`None`を返すだけ）
- **推奨**: 削除（`WM_WINDOWPOSCHANGED`で代替）

### 2.4 スケジュール構成
- **場所**: `crates/wintf/src/ecs/world.rs:12-47`
- **実行順序**: `Input → Update → PreLayout → Layout → PostLayout → UISetup → Draw → ...`
- **PostLayoutの現在の登録**:
  1. `init_graphics_core`
  2. `cleanup_command_list_on_reinit`
  3. `init_window_graphics`
  4. `visual_resource_management_system`
  5. `visual_reinit_system`
  6. `window_visual_integration_system`
  7. `init_window_surface`
  8. `init_window_arrangement`
  9. `propagate_global_arrangements`（最後）

**重要な洞察**: 現在、`propagate_global_arrangements`が`PostLayout`の**最後**に実行されている。新規システムはこの後に追加する必要がある。

---

## 3. 要件とアセットのマッピング

### Requirement 1: Visualサイズ同期システム
- **必要な機能**: `GlobalArrangement.bounds` → `Visual.size`の同期
- **既存アセット**:
  - ✅ `GlobalArrangement`コンポーネント（既存）
  - ✅ `Visual`コンポーネント（既存、要PartialEq追加）
  - ✅ `LayoutRoot`マーカー（既存）
  - ✅ `propagate_global_arrangements`システム（依存先）
- **ギャップ**: 🆕 新規システム`sync_visual_from_layout_root`が必要

### Requirement 2: Surfaceリサイズシステム
- **必要な機能**: `Visual.size`変更時の`SurfaceGraphics`再作成
- **既存アセット**:
  - ✅ `Visual`コンポーネント（既存）
  - ✅ `SurfaceGraphics`コンポーネント（既存）
  - ✅ `create_surface_for_window`関数（既存、line 47 in systems.rs）
  - ⚠️ `init_window_surface`システム（既存、要リファクタリング）
- **ギャップ**: 🔄 既存システムのロジック変更（`Changed<Visual>`対応）

### Requirement 3: WindowPos同期システム
- **必要な機能**: `GlobalArrangement`/`Visual` → `WindowPos`の同期
- **既存アセット**:
  - ✅ `WindowPos`コンポーネント（既存、要フィールド追加）
  - ✅ `Window`コンポーネント（既存）
- **ギャップ**: 🆕 新規システム`sync_window_pos`が必要

### Requirement 4: エコーバック検知機構
- **必要な機能**: 送信値キャッシュと比較ロジック
- **既存アセット**:
  - ✅ `WindowPos`コンポーネント（既存）
- **ギャップ**: 
  - 🔄 `WindowPos`に`last_sent_position`/`last_sent_size`フィールド追加
  - 🆕 `is_echo()`メソッド追加

### Requirement 5: SetWindowPos実行システム
- **必要な機能**: `WindowPos`変更時の`SetWindowPos`呼び出し
- **既存アセット**:
  - ✅ `WindowPos::set_window_pos()`メソッド（既存、line 394）
- **ギャップ**: 🆕 新規システム`apply_window_pos_changes`が必要

### Requirement 6: WM_WINDOWPOSCHANGEDハンドリング
- **必要な機能**: 外部変更の検知とECS更新
- **既存アセット**:
  - ✅ ハンドラ関数シグネチャ（既存、空実装）
- **ギャップ**: 🔄 ハンドラ実装の追加

### Requirement 7: レガシーコード削除
- **削除対象**:
  - ❌ `GetClientRect`呼び出し（window_system.rs:81）
  - ❌ `WM_MOVE`ハンドラ（win_message_handler.rs:616）
  - ❌ `WM_SIZE`ハンドラ（win_message_handler.rs:932）

### Requirement 8: 変更検知最適化
- **必要な機能**: `PartialEq` derive
- **既存アセット**:
  - ✅ `WindowPos`（既に実装済み）
  - ⚠️ `Visual`（未実装）
- **ギャップ**: 🔄 `Visual`に`#[derive(PartialEq)]`追加

### Requirement 9: 双方向同期の確立
- **必要な機能**: 上記すべての統合
- **ギャップ**: 統合テストの追加

### Requirement 10: テスタビリティ
- **既存アセット**:
  - ✅ テストインフラ（`tests/`ディレクトリ）
  - ✅ サンプルアプリ（`examples/taffy_flex_demo.rs`）
- **ギャップ**: 🆕 新規テストの追加

---

## 4. 実装アプローチの選択肢

### Option A: 既存コンポーネント拡張 + 新規システム追加（推奨）

#### 戦略
既存のコンポーネント（`Visual`, `WindowPos`, `SurfaceGraphics`）を最小限拡張し、`PostLayout`スケジュールに新規システムを4つ追加する。

#### 拡張対象
1. **`Visual`コンポーネント**:
   - `#[derive(PartialEq)]`を追加
   - 変更なし（フィールド追加不要）

2. **`WindowPos`コンポーネント**:
   - `last_sent_position: (i32, i32)`フィールド追加
   - `last_sent_size: (i32, i32)`フィールド追加
   - `is_echo(position, size)`メソッド追加

3. **`init_window_surface`システム**（既存）:
   - ロジック変更: `Changed<Visual>`フィルタを使用してリサイズ検知
   - または、新規システム`resize_surface_from_visual`を作成し、既存システムは初期化専用に

#### 新規追加システム
1. **`sync_visual_from_layout_root`**: `GlobalArrangement` → `Visual.size`
2. **`resize_surface_from_visual`**: `Visual.size` → `SurfaceGraphics`（`init_window_surface`から分離）
3. **`sync_window_pos`**: `GlobalArrangement`/`Visual` → `WindowPos`
4. **`apply_window_pos_changes`**: `WindowPos` → `SetWindowPos`

#### システム実行順序（PostLayout内）
```
... (既存のシステム)
↓
propagate_global_arrangements
↓
sync_visual_from_layout_root           ← 新規
↓
resize_surface_from_visual             ← 新規（または既存をリファクタ）
↓
sync_window_pos                        ← 新規
↓
apply_window_pos_changes               ← 新規
```

#### トレードオフ
- ✅ 既存パターンに沿った設計（Bevy ECSの標準的なシステム分割）
- ✅ 段階的な実装が可能（システムごとにテスト）
- ✅ 既存コンポーネントへの影響が最小限
- ✅ `PostLayout`スケジュールの既存インフラを活用
- ❌ 新規ファイルまたは既存ファイルへの追加が必要
- ❌ システム登録コードの更新が必要（world.rs）

### Option B: 統合同期システム（非推奨）

#### 戦略
すべての同期処理を1つの大きなシステム`sync_layout_to_graphics`にまとめる。

#### トレードオフ
- ✅ ファイル数が少ない
- ✅ システム登録が1回で済む
- ❌ 単一責任原則に違反（テストが困難）
- ❌ 変更検知の粒度が粗くなる（パフォーマンス低下）
- ❌ デバッグが困難
- ❌ Bevy ECSのベストプラクティスに反する

### Option C: イベント駆動アプローチ（オーバーエンジニアリング）

#### 戦略
`LayoutChangedEvent`などのイベントを発行し、イベントリスナーで処理する。

#### トレードオフ
- ✅ 疎結合な設計
- ❌ Bevy ECSの変更検知機構と重複（不要な複雑さ）
- ❌ パフォーマンスオーバーヘッド
- ❌ 既存パターンと不整合

---

## 5. 実装の複雑さとリスク評価

### 工数見積もり: **M (Medium: 3-7日)**

#### 内訳
- **コンポーネント拡張**: 0.5日
  - `Visual`に`PartialEq`追加
  - `WindowPos`にフィールドとメソッド追加
- **新規システム実装**: 2-3日
  - `sync_visual_from_layout_root`: 0.5日
  - `resize_surface_from_visual`: 0.5-1日（既存コードのリファクタ含む）
  - `sync_window_pos`: 0.5日
  - `apply_window_pos_changes`: 0.5日
- **メッセージハンドラ実装**: 1日
  - `WM_WINDOWPOSCHANGED`の実装
  - ECSとの統合
- **レガシーコード削除**: 0.5日
  - `GetClientRect`削除
  - `WM_MOVE`/`WM_SIZE`削除
  - 初期化ロジック修正
- **テスト**: 1-2日
  - 単体テスト（`WindowPos::is_echo`等）
  - 統合テスト（完全フロー）
  - 手動テスト（taffy_flex_demo.rs）
- **ドキュメント更新**: 0.5日

**根拠**: 既存のシステムパターンが確立されており、新規システムは既存の`propagate_global_arrangements`と同様の構造。エコーバック検知は新規概念だが、WinUI3の実装パターンが明確。

### リスク: **Medium（中）**

#### リスク要因
1. **エコーバック検知の正確性** (Medium)
   - `bypass_change_detection()`の使用が必須
   - WinUI3のパターンを踏襲することでリスク低減
   
2. **システム実行順序の依存性** (Low)
   - `PostLayout`内の順序が重要
   - `.after()`チェインで明示的に制御可能

3. **既存コードとの互換性** (Low)
   - `GetClientRect`削除による影響
   - 初期化ロジックの変更が必要
   - 影響範囲は`window_system.rs`の1箇所のみ

4. **変更検知の動作** (Low)
   - `PartialEq` deriveによる自動最適化
   - Bevy ECS 0.14+の標準機能

#### リスク軽減策
- WinUI3の実装パターンを詳細に参照
- 段階的な実装とテスト（システムごと）
- デバッグログの充実（各ステップで状態をログ出力）

---

## 6. 技術的な調査項目

以下は設計フェーズで調査・決定すべき項目：

### 6.1 WindowPosフィールドの詳細設計
- **Question**: `last_sent_*`フィールドは`Option<T>`にすべきか？
- **Recommendation**: 非Option（`(i32, i32)`と`(0, 0)`で初期化）
- **Rationale**: エコーバック判定で常に値が必要、Noneチェックが不要

### 6.2 init_window_surfaceのリファクタリング戦略
- **Option 1**: 既存システムを分割（初期化とリサイズを別システムに）
- **Option 2**: 既存システムに`Changed<Visual>`フィルタを追加
- **Recommendation**: Option 1（責任の分離、テスト容易性）

### 6.3 WM_WINDOWPOSCHANGEDハンドラからのECSアクセス
- **Challenge**: メッセージハンドラからECS Worldへのアクセス方法
- **Existing Pattern**: `WinState` traitを通じてWorldにアクセス可能か？
- **Research Needed**: 既存のメッセージハンドラ実装パターンを確認

### 6.4 エコーバック検知の精度
- **Question**: 位置とサイズの完全一致で十分か？浮動小数点誤差は？
- **Recommendation**: `i32`の完全一致（浮動小数点なし）
- **Rationale**: WindowsのSIZE/POINTは整数型

### 6.5 PartialEqのフィールド除外
- **Question**: `Visual`の全フィールドを比較すべきか？`size`のみでよいか？
- **Recommendation**: 全フィールド比較（derive時のデフォルト動作）
- **Rationale**: 他のフィールド（`opacity`, `is_visible`等）も変更検知対象

---

## 7. 設計フェーズへの推奨事項

### 7.1 優先的に実装すべき順序
1. **Phase 1**: コンポーネント拡張とVisual同期
   - `Visual`に`PartialEq`追加
   - `WindowPos`拡張
   - `sync_visual_from_layout_root`システム実装
   - テスト: Visual.sizeが正しく更新されるか確認

2. **Phase 2**: Surface同期
   - `resize_surface_from_visual`システム実装（または既存システムリファクタ）
   - テスト: Surfaceが正しくリサイズされるか確認

3. **Phase 3**: WindowPos同期とSetWindowPos
   - `sync_window_pos`システム実装
   - `apply_window_pos_changes`システム実装
   - テスト: ウィンドウサイズが変更されるか確認

4. **Phase 4**: エコーバック検知とWM_WINDOWPOSCHANGED
   - `WM_WINDOWPOSCHANGED`ハンドラ実装
   - テスト: 無限ループが発生しないか確認

5. **Phase 5**: レガシーコード削除とクリーンアップ
   - `GetClientRect`削除
   - `WM_MOVE`/`WM_SIZE`削除
   - 統合テスト

### 7.2 主要な設計決定事項
1. **システムファイル配置**: 
   - 新規ファイル`crates/wintf/src/ecs/layout/sync_systems.rs`を作成
   - または既存の`systems.rs`に追加

2. **エラーハンドリング**: 
   - `SetWindowPos`失敗時のログ出力
   - `create_surface_for_window`失敗時のリトライ戦略

3. **デバッグサポート**:
   - 各システムでerrln!()によるログ出力
   - フレームカウンタの活用

### 7.3 キー統合ポイント
- **world.rs**: `PostLayout`スケジュールへのシステム登録
- **window_system.rs**: `GetClientRect`削除と初期化ロジック修正
- **win_message_handler.rs**: `WM_WINDOWPOSCHANGED`実装

---

## 8. まとめ

### 推奨実装アプローチ
**Option A: 既存コンポーネント拡張 + 新規システム追加**

- 既存のBevy ECSパターンに従った設計
- 段階的な実装とテストが可能
- リスクは中程度だが、軽減策が明確
- 工数は3-7日（Medium）

### クリティカルパス
1. `Visual`に`PartialEq`追加（必須、他のすべてに影響）
2. `propagate_global_arrangements`の後に実行される新規システムの実装
3. エコーバック検知機構（無限ループ防止の要）
4. `WM_WINDOWPOSCHANGED`実装（双方向同期の要）

### 次のステップ
設計フェーズへ進み、以下を詳細化：
- 各システムの具体的な実装
- エラーハンドリング戦略
- テスト計画
- システム実行順序の最終確定

コマンド: `/kiro-spec-design layout-to-graphics-sync`

---

**Gap Analysis Complete** - 既存コードベースとの統合ポイント、実装アプローチ、リスクを明確化しました。
