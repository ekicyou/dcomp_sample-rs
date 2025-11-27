# Implementation Plan

## Phase 1: SurfaceGraphicsDirty 移行

- [x] 1. SurfaceGraphicsDirty コンポーネント定義
- [x] 1.1 (P) SurfaceGraphicsDirty コンポーネントを追加
  - `ecs/graphics/components.rs` に新コンポーネントを定義
  - `requested_frame: u64` フィールドを追加
  - `Default` トレイト実装（初期値 0）
  - `Component` derive マクロ追加
  - _Requirements: 1.1, 1.2, 1.3_

- [x] 1.2 Visual の on_add フックで SurfaceGraphicsDirty を自動挿入
  - Visual コンポーネントの `on_add` フック実装を追加
  - `SurfaceGraphicsDirty::default()` を自動挿入するロジック追加
  - 既存エンティティが SurfaceGraphicsDirty を持つ場合はスキップ
  - _Requirements: 5.1, 5.2, 5.3_

- [x] 1.3 mark_dirty_surfaces システムを Changed パターンに変更
  - `insert(SurfaceUpdateRequested)` を `dirty.requested_frame = current_frame` に置換
  - `FrameCount` リソースからフレーム番号を取得
  - クエリを `&mut SurfaceGraphicsDirty` に変更
  - _Requirements: 1.4, 2.3_

- [x] 1.4 render_surface システムを Changed パターンに変更
  - `With<SurfaceUpdateRequested>` を `Changed<SurfaceGraphicsDirty>` に置換
  - `remove::<SurfaceUpdateRequested>()` 呼び出しを削除
  - _Requirements: 2.1, 2.2_

- [x] 1.5 deferred_surface_creation_system の描画トリガーを変更
  - Surface 作成後の `insert(SurfaceUpdateRequested)` を削除
  - `SurfaceGraphicsDirty` のフレーム更新に置換
  - _Requirements: 2.4_

- [x] 1.6 on_surface_graphics_changed フックと SafeInsertSurfaceUpdateRequested を削除
  - `SafeInsertSurfaceUpdateRequested` カスタムコマンド定義を削除
  - `on_surface_graphics_changed` フックを削除または SurfaceGraphicsDirty 更新に変更
  - _Requirements: 2.5, 2.6_

- [x] 1.7 SurfaceUpdateRequested コンポーネントを削除
  - `SurfaceUpdateRequested` 定義を削除
  - 関連する import 文を削除
  - _Requirements: 1.5_

## Phase 2: HasGraphicsResources 拡張

- [x] 2. HasGraphicsResources 拡張
- [x] 2.1 (P) HasGraphicsResources に世代番号フィールドとメソッドを追加
  - `needs_init_generation: u32` フィールド追加
  - `processed_generation: u32` フィールド追加
  - `Default` 実装を更新（両フィールド 0 初期化）
  - `request_init()` メソッド実装（世代番号インクリメント）
  - `needs_init() -> bool` メソッド実装（世代番号比較）
  - `mark_initialized()` メソッド実装（世代番号同期）
  - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5_

- [x] 2.2 init_graphics_core システムを Changed パターンに変更
  - `insert(GraphicsNeedsInit)` を `res.request_init()` に置換
  - 再初期化時と初期化時の両方の呼び出し箇所を変更
  - _Requirements: 4.1_

- [x] 2.3 init_window_graphics システムを Changed パターンに変更
  - `With<GraphicsNeedsInit>` を `Changed<HasGraphicsResources>` に置換
  - `res.needs_init()` 条件チェックを追加
  - _Requirements: 4.2_

- [x] 2.4 init_window_visual システムを Changed パターンに変更
  - `With<GraphicsNeedsInit>` を `Changed<HasGraphicsResources>` に置換
  - `res.needs_init()` 条件チェックを追加
  - _Requirements: 4.3_

- [x] 2.5 cleanup_graphics_needs_init システムを Changed パターンに変更
  - `With<GraphicsNeedsInit>` を `needs_init()` 条件に置換
  - `remove::<GraphicsNeedsInit>()` を `res.mark_initialized()` に置換
  - _Requirements: 4.4_

- [x] 2.6 cleanup_command_list_on_reinit システムを Changed パターンに変更
  - `With<GraphicsNeedsInit>` を `needs_init()` 条件に置換
  - _Requirements: 4.5_

- [x] 2.7 create_visuals_for_init_marked システムを Changed パターンに変更
  - `With<GraphicsNeedsInit>` を `Changed<HasGraphicsResources>` + `needs_init()` に置換
  - _Requirements: 4.6_

- [x] 2.8 GraphicsNeedsInit コンポーネントを削除
  - `GraphicsNeedsInit` 定義を削除
  - 関連する import 文を削除
  - _Requirements: 3.6_

## Phase 3: テストと検証

- [x] 3. テスト更新と統合検証
- [x] 3.1 (P) HasGraphicsResources メソッドのユニットテストを追加
  - `request_init()` が世代番号をインクリメントすることを検証
  - `needs_init()` が世代番号不一致時に true を返すことを検証
  - `mark_initialized()` が世代番号を同期することを検証
  - ラッピング動作の検証
  - _Requirements: 6.4_

- [x] 3.2 surface_optimization_test.rs のテストを更新
  - `test_surface_update_requested_component_exists` を SurfaceGraphicsDirty 用に変更
  - `test_mark_dirty_surfaces_propagation` を新パターンに更新
  - `test_surface_update_requested_on_add_hook` を更新または削除
  - マーカー存在検証を状態検証に置換
  - _Requirements: 6.1, 6.2, 6.3, 6.5_

- [x] 3.3 全テスト実行と動作確認
  - `cargo test --all-targets` で全テスト成功を確認
  - サンプルアプリケーション `areka.rs` の動作確認
  - `taffy_flex_demo.rs` の動作確認
  - _Requirements: 6.6_

## Phase 4: API 公開と文書化

- [x] 4. 公開 API の整理
- [x] 4.1 新コンポーネントの公開設定
  - `SurfaceGraphicsDirty` を `pub` として公開
  - `wintf::ecs` モジュールからアクセス可能に設定
  - `HasGraphicsResources` の新メソッドを `pub` で公開
  - _Requirements: 7.3_

- [ ] 4.2* 移行ガイドの作成（オプション）
  - `SurfaceUpdateRequested` → `SurfaceGraphicsDirty` の移行手順
  - `GraphicsNeedsInit` → `HasGraphicsResources.needs_init()` の移行手順
  - 削除コンポーネントと新 API のマッピング表
  - _Requirements: 7.1, 7.2, 7.4_
