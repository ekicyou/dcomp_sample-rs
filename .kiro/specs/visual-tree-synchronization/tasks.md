# Implementation Plan

## Phase 1: API基盤整備

- [x] 1. RemoveVisual APIラッパーの実装
- [x] 1.1 (P) DCompositionVisualExtトレイトにremove_visual/remove_all_visualsメソッドを追加
  - DirectComposition RemoveVisual APIを呼び出すラッパーメソッドを実装
  - RemoveAllVisuals APIのラッパーも併せて実装
  - 存在しないVisualの削除時のエラーハンドリングを実装
  - _Requirements: R1_

- [x] 2. VisualGraphicsの親キャッシュ拡張
- [x] 2.1 VisualGraphicsコンポーネントにparent_visualフィールドを追加
  - 親Visual参照をキャッシュするフィールドを追加
  - on_removeフックで親Visualからの自動削除を実装
  - エラー発生時は無視して処理を継続
  - _Requirements: R9_

- [x] 3. Visual追加ヘルパー関数の実装
- [x] 3.1 (P) insert_visual/insert_visual_with関数をecs::graphicsモジュールに公開
  - DeferredWorldを受け取りVisual::default()を挿入するヘルパー関数を実装
  - カスタムVisual値を挿入できるinsert_visual_with関数も実装
  - ウィジェットのon_addフックから呼び出し可能な形式で実装
  - _Requirements: R3_

## Phase 2: Visual自動作成と階層同期

- [x] 4. Visual追加時のVisualGraphics自動作成
- [x] 4.1 Added<Visual>を検知してVisualGraphicsを作成するシステムを実装
  - PostLayoutスケジュールでAdded<Visual>クエリを使用
  - GraphicsCoreからCreateVisualを呼び出してVisualGraphics作成
  - SurfaceGraphicsはこの時点では作成しない（遅延作成）
  - Window Entityの既存WindowVisual作成との統合
  - **Note**: 既存のvisual_resource_management_systemで対応済み
  - _Requirements: R2_

- [x] 5. 既存ウィジェットへのVisual自動追加
- [x] 5.1 (P) LabelコンポーネントにVisual自動追加フックを実装
  - on_addフックでinsert_visualを呼び出し
  - _Requirements: R4_

- [x] 5.2 (P) RectangleコンポーネントにVisual自動追加フックを実装
  - on_addフックでinsert_visualを呼び出し
  - _Requirements: R4_

- [x] 6. ウィジェットツリー変更の検知システム
- [x] 6.1 ChildOf変更を検知してVisual階層を同期するシステムを実装
  - Compositionスケジュールで実行
  - Changed<ChildOf>で親変更を検知し、旧親からRemoveVisual→新親にAddVisual
  - RemovedComponents<ChildOf>でChildOf削除を検知
  - parent_visualキャッシュを更新
  - **BLOCKED**: 既存の描画フロー（draw_recursive方式）と競合するため、world.rsでの登録を一時無効化
  - **UNBLOCK条件**: Phase 4（自己描画方式への移行）完了後に有効化
  - _Requirements: R6, R7_

- [x] 6.2 Children順序変更を検知してZ-orderを同期するシステムを実装
  - Changed<Children>で順序変更を検知
  - AddVisualのinsertabove/referencevisualパラメーターでZ-order制御
  - **Note**: visual_hierarchy_sync_systemで基本機能を実装
  - _Requirements: R7_

## Phase 3: テキスト測定とレイアウト統合

- [ ] 7. テキストレイアウト測定システムの実装
- [ ] 7.1 measure_text_sizeシステムをPreLayoutスケジュールに追加
  - Label追加/変更時にDirectWriteでテキストサイズを測定
  - TextLayoutMetricsコンポーネントに結果を保存
  - _Requirements: R4a_

- [ ] 7.2 sync_label_size_to_box_styleシステムを実装
  - TextLayoutMetricsからBoxStyle.sizeに値を反映
  - ユーザー設定のBoxStyleがある場合は優先
  - _Requirements: R4a_

- [ ] 7.3 draw_labelsシステムから測定ロジックを分離
  - 既存のテキスト測定コードをmeasure_text_sizeに移動
  - draw_labelsは描画のみを行うよう変更
  - TextLayoutMetricsが存在する場合は再測定をスキップ
  - _Requirements: R4a_

## Phase 4: Surface遅延作成と描画方式変更

- [ ] 8. SurfaceGraphics遅延作成システムの実装
- [ ] 8.1 deferred_surface_creation_systemを実装
  - PostLayoutスケジュールで実行
  - 描画可能コンテンツ（Label/Rectangle）を持つEntityにのみSurface作成
  - GlobalArrangementのスケール成分を考慮したサイズ計算
  - SetContentでVisualにSurfaceを設定
  - _Requirements: R5_

- [ ] 8.2 Surfaceサイズ変更検知とリサイズシステムを実装
  - GlobalArrangement変更時にSurfaceサイズを再評価
  - サイズ変更時は新しいSurfaceを作成して置き換え
  - _Requirements: R5_

- [ ] 9. 描画方式の変更（自己描画方式への移行）
- [ ] 9.1 render_surfaceシステムを自己描画方式に変更
  - draw_recursive関数を廃止
  - 各Entityが自分のCommandListのみを自分のSurfaceに描画
  - Changed<GraphicsCommandList>で変更検知
  - _Requirements: R5a_

- [ ] 9.2 Visual Offset同期システムを実装
  - Changed<Arrangement>を検知してSetOffsetX/SetOffsetYを呼び出し
  - Arrangementのoffset成分をVisualのOffset値として設定
  - _Requirements: R8_

## Phase 5: 廃止項目の削除とスケジュール統合

- [ ] 10. 廃止項目の削除
- [ ] 10.1 (P) PreRenderSurfaceスケジュールとmark_dirty_surfacesシステムを削除
  - world.rsからスケジュール定義を削除
  - mark_dirty_surfacesシステムを削除
  - SurfaceUpdateRequestedマーカーコンポーネントを削除
  - _Requirements: R5a_

- [ ] 11. システム実行順序の統合
- [ ] 11.1 world.rsに新規システムを登録
  - PreLayout: text_layout_measurement_system
  - PostLayout: create_visual_graphics_system, deferred_surface_creation_system
  - Composition: visual_hierarchy_sync_system, visual_zorder_sync_system, visual_transform_sync_system
  - RenderSurface: render_to_surface_system（Changed<GraphicsCommandList>フィルター）
  - **BLOCKED**: visual_hierarchy_sync_systemは既存描画フローと競合するため登録をコメントアウト
  - **UNBLOCK条件**: Phase 4（自己描画方式への移行）完了後に有効化
  - _Requirements: R10_

## Phase 6: 統合テストと検証

- [x] 12. 統合テストの実装
- [x] 12.1 階層追加/削除シナリオのテストを実装
  - ChildOf追加時のVisual階層同期を検証
  - ChildOf変更時の旧親→新親移動を検証
  - Entity despawn時のVisualクリーンアップを検証
  - **Note**: visual_hierarchy_sync_test.rsで実装済み
  - _Requirements: R6, R7, R9_

- [x] 12.2 Z-order変更シナリオのテストを実装
  - Children順序変更時のZ-order同期を検証
  - **Note**: test_children_order_change_syncs_zorderで実装済み
  - _Requirements: R7_

- [x] 12.3 dcomp_demo.rsでビジュアル回帰テストを実行
  - 既存の描画が正常に動作することを目視確認
  - Label/Rectangleが正しい位置に表示されることを確認
  - **Note**: ビルド確認済み、目視確認は手動で実施
  - _Requirements: R5a_
