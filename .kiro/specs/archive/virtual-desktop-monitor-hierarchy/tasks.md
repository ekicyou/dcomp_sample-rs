# Implementation Plan

## Phase 0: 名称変更

- [ ] 1. BoxStyleとBoxComputedLayoutをTaffy統合名称に変更
- [ ] 1.1 (P) 既存レイアウトコンポーネント名を一括変更
  - `BoxStyle` → `TaffyStyle`に変更（layout/mod.rs, layout/systems.rs, layout/taffy.rs）
  - `BoxComputedLayout` → `TaffyComputedLayout`に変更
  - 全テストファイルとexamples/の名称更新
  - 推定変更箇所: 100-150箇所
  - _Requirements: 3_

- [ ] 1.2 (P) 全テスト実行と動作確認
  - `cargo test --workspace`実行
  - 全テストがパスすることを確認
  - コンパイルエラーがゼロであることを確認
  - examples/が正常に動作することを確認
  - _Requirements: 3, 8_

- [ ] 1.3 (P) ドキュメント更新
  - `doc/spec/`配下のMarkdownファイルの名称更新
  - コード例の名称を新しい名称に更新
  - _Requirements: 3_

## Phase 1a: レイアウト論理コンポーネント追加

- [ ] 2. 絶対配置用の論理コンポーネントを定義
- [ ] 2.1 (P) BoxPositionとBoxInsetコンポーネントを追加
  - `BoxPosition` enum追加（Relative/Absolute）
  - `BoxInset` struct追加（left/top/right/bottom座標）
  - layout/high_level.rsに実装
  - layout/mod.rsにre-export追加
  - _Requirements: 4_

- [ ] 2.2 build_taffy_styles_systemを拡張して論理コンポーネント変換を実装
  - Queryに`Option<&BoxPosition>`と`Option<&BoxInset>`追加
  - BoxPosition → taffy::Position変換ロジック実装
  - BoxInset → taffy::Rect<LengthPercentageAuto>変換ロジック実装
  - 既存のBoxSize/BoxMargin/BoxPaddingの動作が変わらないことを確認
  - _Requirements: 4_

## Phase 1b: Monitor基本実装

- [ ] 3. Monitorコンポーネントとenumerate_monitors関数を実装
- [ ] 3.1 (P) Monitorコンポーネント定義
  - `Monitor` struct定義（handle, bounds, work_area, dpi, is_primary）
  - `Monitor::from_hmonitor()`実装（GetMonitorInfoW + GetDpiForMonitor）
  - `Monitor::physical_size()`実装
  - `Monitor::top_left()`実装
  - `MonitorError`定義
  - ecs/monitor.rs作成、ecs/mod.rsにモジュール追加
  - _Requirements: 1_

- [ ] 3.2 (P) enumerate_monitors関数を実装
  - EnumDisplayMonitors API統合
  - GetMonitorInfoWでモニター情報取得
  - GetDpiForMonitorでDPI取得
  - 全モニターのVec<Monitor>を返却
  - エラーハンドリング（空のVec返却、ログ出力）
  - _Requirements: 1_

- [ ] 4. Appリソース拡張とディスプレイ構成変更フラグ管理
- [ ] 4.1 (P) Appリソースにdisplay_configuration_changedフィールド追加
  - `display_configuration_changed: bool`フィールド追加
  - `mark_display_change()`メソッド実装
  - `reset_display_change()`メソッド実装
  - _Requirements: 1, 6_

- [ ] 5. initialize_layout_root_systemを実装してLayoutRoot Singleton管理
- [ ] 5.1 LayoutRoot Singleton生成とMonitorエンティティ初期化
  - `Query<Entity, With<LayoutRoot>>`でLayoutRoot存在チェック
  - 未存在時にLayoutRootエンティティ生成
  - `enumerate_monitors()`呼び出し
  - 各Monitorエンティティ生成（Monitor, ChildOf(LayoutRoot), BoxPosition::Absolute, BoxSize::default(), BoxInset::default()）
  - TaffyLayoutResource::create_node()で各Monitorのノード作成
  - 既存LayoutRoot存在時は生成スキップ
  - Startupスケジュールに登録
  - _Requirements: 1, 2_

## Phase 2: Taffy統合

- [ ] 6. update_monitor_layout_systemを実装してMonitor情報を論理コンポーネントに変換
- [ ] 6.1 Monitor.boundsからBoxSizeとBoxInsetを計算
  - `Query<(&Monitor, &mut BoxSize, &mut BoxInset), Changed<Monitor>>`
  - Monitor.bounds → BoxSize計算（physical_size()使用）
  - Monitor.bounds → BoxInset計算（top_left()使用）
  - BoxPosition::Absoluteは既にPhase 1bで設定済みのため変更不要
  - Updateスケジュールに登録（detect_display_change_systemの後）
  - _Requirements: 2, 4_

- [ ] 7. sync_taffy_tree_systemにMonitor対応を追加
- [ ] 7.1 MonitorエンティティのChildOf処理を追加
  - Query<&Monitor>追加（Monitor存在確認用）
  - Monitor用ChildOf処理追加（既存のWindow処理と同様）
  - ECS階層（ChildOf）→ Taffyツリー同期
  - Entity↔NodeIdマッピング確認
  - _Requirements: 4, 5_

- [ ] 8. システムスケジュール統合とシステム実行順序確立
- [ ] 8.1 新規システムをECSスケジュールに登録
  - `initialize_layout_root_system`をStartupスケジュールに登録
  - `update_monitor_layout_system`をUpdateスケジュールに登録
  - 実行順序: Startup → Update（detect_display_change → update_monitor_layout） → Layout（build_taffy_styles → sync_taffy_tree → compute_taffy_layout → distribute_computed_layouts） → Render
  - 循環依存が存在しないことを確認
  - _Requirements: 7_

## Phase 3: 動的更新

- [ ] 9. WM_DISPLAYCHANGEメッセージハンドラを実装
- [ ] 9.1 (P) WM_DISPLAYCHANGEハンドラでフラグ設定
  - win_message_handler.rsにWM_DISPLAYCHANGE処理追加
  - `App::mark_display_change()`呼び出し
  - _Requirements: 6_

- [ ] 10. detect_display_change_systemを実装してモニター構成変更を検知
- [ ] 10.1 ディスプレイ構成変更検知とMonitorエンティティ更新
  - `App.display_configuration_changed`監視（フラグがtrueの場合のみ処理実行）
  - `enumerate_monitors()`再実行
  - 新旧モニターリスト比較（handle基準）
  - 新規Monitor: エンティティ生成、ChildOf設定、BoxSize/BoxInset/BoxPosition追加、TaffyLayoutResource::create_node()呼び出し
  - 削除Monitor: TaffyLayoutResource::remove_node()、エンティティ削除
  - 変更Monitor: Monitor.bounds/dpi更新（Changed<Monitor>イベント発火）
  - すべてのMonitor更新完了後、最後に`App::reset_display_change()`呼び出し
  - Updateスケジュールに登録（update_monitor_layout_systemの前）
  - _Requirements: 6_

## テストとバリデーション

- [ ] 11. Monitor階層構築テストを実装
- [ ] 11.1 (P) LayoutRoot Singleton生成とMonitor列挙テスト
  - LayoutRootが一度だけ生成されることを検証
  - 既存LayoutRootがある場合は生成されないことを検証
  - enumerate_monitors()が全モニターを列挙することを検証
  - MonitorエンティティがLayoutRootの子として生成されることを検証
  - Monitor.bounds/dpi/is_primaryが正確に取得されることを検証
  - _Requirements: 1, 9_

- [ ] 11.2 (P) LayoutRoot → {Monitor, Window} → Widget階層構築テスト
  - LayoutRoot → {Monitor, Window} → Widget階層が正しく構築されることを検証
  - ChildOfコンポーネントが正しく設定されることを検証
  - _Requirements: 2, 9_

- [ ] 12. Monitor→TaffyStyle変換テストを実装
- [ ] 12.1 (P) Monitor.boundsからTaffyStyle変換テスト
  - Monitor.boundsがTaffyStyle.size/insetに正しく変換されることを検証
  - Position::Absoluteが設定されることを検証
  - 複数モニターの座標が正しくinsetに反映されることを検証
  - _Requirements: 4, 9_

- [ ] 13. Taffyレイアウト計算テストを実装
- [ ] 13.1 (P) Taffyツリー同期とレイアウト計算テスト
  - ECS階層がTaffyツリーに正しく同期されることを検証
  - Entity↔NodeIdマッピングが正確であることを検証
  - LayoutRootをルートとしてレイアウト計算が実行されることを検証
  - TaffyComputedLayoutが各エンティティに配布されることを検証
  - Monitor/Window/WidgetのTaffyComputedLayoutが親子関係に基づき正しく計算されることを検証
  - _Requirements: 5, 9_

- [ ] 14. ディスプレイ構成変更テストを実装
- [ ] 14.1 (P) DisplayConfigurationChangedフラグテスト
  - App.mark_display_change()でフラグがtrueになることを検証
  - App.reset_display_change()でフラグがfalseになることを検証
  - _Requirements: 6, 9_

- [ ] 14.2 (P) モニター追加・削除・更新テスト
  - 新規モニター検出時にMonitorエンティティが生成されることを検証
  - ChildOfが正しく設定されることを検証
  - モニター削除時にMonitorエンティティが削除されることを検証
  - TaffyLayoutResourceのマッピングがクリーンアップされることを検証
  - モニター解像度変更時にMonitor.boundsが更新されることを検証
  - _Requirements: 6, 9_

- [ ] 15. 既存システム互換性テストを実行
- [ ] 15.1 (P) 既存テストの実行と後方互換性確認
  - 全既存テストを実行（名称変更除く）
  - LayoutRootまたはMonitorエンティティが存在しない場合でも既存レイアウトシステムが正常動作することを確認
  - GlobalArrangement → WindowPos変換処理が維持されていることを確認
  - Surface最適化機能が継続してサポートされていることを確認
  - _Requirements: 8, 9_
