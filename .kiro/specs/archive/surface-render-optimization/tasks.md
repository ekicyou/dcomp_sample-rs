# Implementation Plan

## Task Format Template

### Major + Sub-task structure

- [x] 1. 基盤セットアップ (Core Infrastructure Setup)
- [x] 1.1 マーカーコンポーネント定義とテストスケルトン作成
  - `crates/wintf/src/ecs/graphics/components.rs` に `SurfaceUpdateRequested` コンポーネントを定義する。
  - `crates/wintf/tests/surface_optimization_test.rs` を作成し、初期テスト構造を構築する。
  - _Requirements: 1_

- [x] 2. 変更検知システムの実装 (Change Detection System Implementation)
- [x] 2.1 ダーティマーキングロジックの実装
  - `crates/wintf/src/ecs/graphics/systems.rs` に `mark_dirty_surfaces` システムを実装する。
  - `GraphicsCommandList`, `GlobalArrangement`, `Children` の変更検知クエリを追加する。
  - 親要素を遡り、直近の `SurfaceGraphics` 所有者を特定するロジックを実装する。
  - `Commands` を使用して `SurfaceUpdateRequested` マーカーを付与する。
  - `surface_optimization_test.rs` に伝播ロジックを検証する単体テストを追加する。
  - _Requirements: 1, 2_

- [x] 3. 描画システムのリファクタリング (Render System Refactoring)
- [x] 3.1 再帰描画とネスト分離の実装
  - ツリー走査を処理する `draw_recursive` ヘルパー関数を実装する。
  - ネストされた `SurfaceGraphics` エンティティに遭遇した場合に処理をスキップするロジック（分離）を追加する。
  - `SurfaceUpdateRequested` を持つエンティティのみを反復処理するように `render_surface` をリファクタリングする。
  - `draw_recursive` をメイン描画ループに統合する。
  - 描画完了後に `SurfaceUpdateRequested` を削除するロジックを追加する。
  - 描画が発生しない場合にデバッグログが抑制されることを確認する。
  - `surface_optimization_test.rs` にネストされたサーフェスの分離を検証する単体テストを追加する。
  - _Requirements: 1, 2, 3_

- [x] 4. スケジューリングと統合 (Scheduling & Integration)
- [x] 4.1 システムスケジューリングの設定
  - `crates/wintf/src/ecs/world.rs` を更新し、`mark_dirty_surfaces` と `render_surface` を登録する。
  - マーカーが即座に反映されるよう、間に `apply_deferred` を挟んで `.chain()` を使用してスケジュールを設定する。
  - `examples/simple_window.rs` を実行し、表示崩れや過剰なログ出力がないか確認して実装を検証する。
  - _Requirements: 1, 2, 3_
