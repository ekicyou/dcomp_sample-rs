# Implementation Plan: GraphicsCore再初期化システム

## Phase 1: Option<T>ラップと基盤準備

- [ ] 1. マーカーコンポーネント定義 (P)
  - HasGraphicsResourcesとGraphicsNeedsInitマーカーコンポーネントを定義
  - Component derive、Default実装を追加
  - 既存のecs/graphics/components.rsに配置
  - _Requirements: 1-2, 5-2_

- [ ] 2. GraphicsCoreをOption<T>でラップ (P)
  - 内部データをGraphicsCoreInner構造体として分離
  - GraphicsCore構造体をinner: Option<GraphicsCoreInner>に変更
  - invalidate()、is_valid()メソッドを実装
  - アクセサメソッド（d2d_factory、d2d_device、dcomp）をOption<&T>戻り値に変更
  - 既存のGraphicsCore::new()をOption::Some()で初期化するよう修正
  - _Requirements: 1-1, 2_

- [ ] 3. WindowGraphicsをOption<T>でラップとgeneration追加
  - 内部データをWindowGraphicsInner構造体として分離
  - WindowGraphics構造体をinner: Option<WindowGraphicsInner>とgeneration: u32に変更
  - invalidate()、is_valid()、generation()メソッドを実装
  - アクセサメソッド（target、context）をOption<&T>戻り値に変更
  - 既存のWindowGraphics::new()をgeneration=0で初期化
  - _Requirements: 1-2, 3-1_

- [ ] 4. VisualとSurfaceをOption<T>でラップ (P)
  - Visual構造体をinner: Option<IDCompositionVisual3>に変更
  - Surface構造体をinner: Option<IDCompositionSurface>に変更
  - 各々にinvalidate()、is_valid()メソッドを実装
  - アクセサメソッド（visual、surface）をOption<&T>戻り値に変更
  - 既存のVisual::new()とSurface::new()をOption::Some()で初期化
  - _Requirements: 1-2, 4-1, 4-2_

## Phase 2: 初期化システム実装

- [ ] 5. init_graphics_coreシステム実装
  - Option<ResMut<GraphicsCore>>でリソースチェック
  - GraphicsCore.innerがNoneの場合、GraphicsCore::new()を実行
  - 初期化成功時、Query<Entity, With<HasGraphicsResources>>で全エンティティを取得
  - Commands::entity(e).insert(GraphicsNeedsInit)で一括マーキング
  - 初期化失敗時、エラーログ出力してNone維持
  - PostLayoutスケジュールに登録
  - _Requirements: 2, 4-3_

- [ ] 6. init_window_graphicsシステム実装
  - Query<(Entity, &WindowHandle, Option<&mut WindowGraphics>), With<GraphicsNeedsInit>>でフィルタ
  - WindowGraphicsがNoneまたはis_valid()==falseの場合に初期化
  - GraphicsCore有効性チェック（is_valid()）
  - 初期化成功時、generation++
  - 初期化失敗時、エラーログ出力してマーカー保持
  - PostLayoutスケジュールでinit_graphics_core.after()に登録
  - _Requirements: 3-1, 3-2, 4-3_

- [ ] 7. init_window_visualシステム実装
  - Query<(Entity, &WindowGraphics, Option<&mut Visual>), With<GraphicsNeedsInit>>でフィルタ
  - WindowGraphics有効性チェック（無効ならスキップ）
  - VisualがNoneまたはis_valid()==falseの場合に初期化
  - IDCompositionVisual3生成、CompositionTargetにSetRoot
  - 初期化失敗時、エラーログ出力してマーカー保持
  - PostLayoutスケジュールでinit_window_graphics.after()に登録
  - _Requirements: 4-1, 4-3_

- [ ] 8. init_window_surfaceシステム実装
  - Query<(Entity, &WindowGraphics, &Visual, Option<&mut Surface>), With<GraphicsNeedsInit>>でフィルタ
  - WindowGraphicsとVisual両方の有効性チェック（いずれか無効ならスキップ）
  - SurfaceがNoneまたはis_valid()==falseの場合に初期化
  - IDCompositionSurface生成、VisualにSetContent
  - 初期化失敗時、エラーログ出力してマーカー保持
  - PostLayoutスケジュールでinit_window_visual.after()に登録
  - _Requirements: 4-2, 4-3_

- [ ] 9. cleanup_graphics_needs_initシステム実装
  - Query<(Entity, &WindowGraphics, &Visual, &Surface), With<GraphicsNeedsInit>>でフィルタ
  - 全コンポーネントのis_valid()チェック（WindowGraphics.is_valid() && Visual.is_valid() && Surface.is_valid()）
  - 全て有効ならCommands::entity(e).remove::<GraphicsNeedsInit>()
  - いずれか無効ならマーカー保持（次フレーム再試行）
  - Drawスケジュールに登録（PostLayout内の他の初期化システムとの競合回避）
  - _Requirements: 4-3, 5-2_

## Phase 3: 破棄検出と依存コンポーネント無効化

- [ ] 10. invalidate_dependent_componentsシステム実装
  - Option<Res<GraphicsCore>>でリソースチェック
  - GraphicsCore.is_valid()==falseの場合に実行
  - Query<&mut WindowGraphics>で全WindowGraphicsにinvalidate()呼び出し
  - Query<&mut Visual>で全Visualにinvalidate()呼び出し
  - Query<&mut Surface>で全Surfaceにinvalidate()呼び出し
  - Updateスケジュールに登録（detect_device_lostの後、PostLayoutの前）
  - _Requirements: 1-1, 5-1_

## Phase 4: エンティティspawn時のマーカー付与

- [ ] 11. 既存spawn処理にHasGraphicsResourcesマーカー付与を追加
  - WindowGraphics、Visual、Surface生成時のspawn処理を特定
  - commands.entity(entity).insert(HasGraphicsResources)を追加
  - ウィンドウ生成フローでマーカー付与を確認
  - 新Widget追加時の統合手順を文書化（READMEまたはdoc/）
  - _Requirements: 1-2, 5-2_

## Phase 5: 統合テストと検証

- [ ] 12. GraphicsCore再初期化統合テスト
  - GraphicsCore.invalidate()呼び出しによる破棄シミュレーション
  - init_graphics_coreがNone検出して再初期化することを確認
  - 全HasGraphicsResourcesエンティティへGraphicsNeedsInit一括マーキング確認
  - 順次初期化システム（init_window_graphics → init_window_visual → init_window_surface）実行確認
  - cleanup_graphics_needs_initがマーカー削除することを確認
  - tests/ディレクトリに配置
  - _Requirements: 1-2, 2, 4-3, 7-2_

- [ ] 13. コンポーネント状態遷移テスト
  - WindowGraphics、Visual、Surfaceのinvalidate()呼び出し確認
  - is_valid()メソッドの動作確認（Some→true、None→false）
  - generation番号インクリメント確認
  - アクセサメソッドの戻り値確認（有効時Some(&T)、無効時None）
  - tests/ディレクトリに配置
  - _Requirements: 1-2, 3-1, 4-1, 4-2, 7-2_

- [ ] 14. ECS並列実行最適化テスト (P)
  - 初期化システム（Query<&mut T, With<GraphicsNeedsInit>>）と参照システム（Query<&T, Without<GraphicsNeedsInit>>）が異なるエンティティセットを操作することを確認
  - Bevy ECSのArchetype-levelクエリ最適化が機能することを確認
  - Changed<T>検出機構がinvalidate()と再初期化を検出することを確認
  - 複数エンティティでの並列実行をベンチマーク
  - tests/ディレクトリに配置
  - _Requirements: 3-2, 5-1, 5-2, 7-2_

- [ ]* 15. エラーハンドリングとログ検証テスト
  - GraphicsCore初期化失敗時のエラーログ出力確認（HRESULT、エラーメッセージ）
  - コンポーネント初期化失敗時のエンティティ情報とエラー理由ログ確認
  - 再初期化の各ステップ（開始、進行中、完了、失敗）ログ確認
  - 無効なコンポーネントへのアクセス試行時の警告出力確認
  - tests/ディレクトリに配置
  - _Requirements: 6, 7-2_

- [ ]* 16. パフォーマンスベンチマーク
  - GraphicsCore初期化時間測定（10ms以内目標）
  - コンポーネント初期化時間測定（1ms/エンティティ以内目標）
  - マーカー一括追加時間測定（1ms/1000エンティティ以内目標）
  - 1000エンティティでのArchetype-level最適化効果測定
  - benches/ディレクトリまたはtests/ディレクトリに配置
  - _Requirements: 7-2_

- [ ]* 17. 複数ウィンドウ同時再初期化テスト
  - 複数HasGraphicsResourcesエンティティ（複数ウィンドウ）生成
  - GraphicsCore.invalidate()呼び出し
  - 全エンティティが順次初期化されることを確認
  - 各エンティティのgeneration番号確認
  - tests/ディレクトリに配置
  - _Requirements: 7-2_

## Phase 6: 既存システム削除と移行完了

- [ ] 18. 既存初期化システムの削除と最終検証
  - 既存のcreate_window_graphics、create_window_visual、create_window_surfaceシステムを削除
  - 新初期化システムへの完全移行確認
  - 既存の描画機能（RenderSurface、Compositionスケジュール）が正常動作することを確認
  - examplesアプリケーション（areka.rs、dcomp_demo.rs）で動作確認
  - Phase 1-4の全Validation checkpoints達成確認
  - _Requirements: 1-1, 1-2, 2, 3-1, 3-2, 4-1, 4-2, 4-3, 5-1, 5-2_

---

**Total**: 18タスク（メジャータスク18、サブタスクなし）  
**Estimated Duration**: 6-9日（Phase 1: 1-2日、Phase 2: 2-3日、Phase 3: 0.5日、Phase 4: 0.5日、Phase 5: 2-3日、Phase 6: 0.5日）  
**Requirements Coverage**: 全7要件（1-1, 1-2, 2, 3-1, 3-2, 4-1, 4-2, 4-3, 5-1, 5-2, 6, 7-1, 7-2）

**Parallel Execution Notes**:
- タスク1, 2, 4: Phase 1の独立したコンポーネント変更（マーカー定義、GraphicsCore、Visual/Surface）は並列実行可能
- タスク14: ECS並列実行テストは他のテストと独立して実行可能
- タスク15, 16, 17: エラーハンドリング、パフォーマンス、複数ウィンドウテストは並列実行可能（*マーク: MVP後の追加検証として延期可能）

**Optional Test Coverage**:
- タスク15: エラーハンドリングとログ検証は基本機能確認後に実施可能
- タスク16: パフォーマンスベンチマークはMVP後の最適化フェーズで実施可能
- タスク17: 複数ウィンドウテストは単一ウィンドウ動作確認後に実施可能
