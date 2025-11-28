# Implementation Plan

## Task 1: Surface生成システムの基盤準備

- [ ] 1.1 (P) デバッグ統計リソースを追加する
  - Surface生成の統計情報を収集するリソースを作成
  - 作成数、スキップ数、削除数、リサイズ数のカウンターを実装
  - デバッグビルド時のみ統計を更新する条件を追加
  - _Requirements: 5.3_

- [ ] 1.2 (P) 物理ピクセルサイズ計算ヘルパーを実装する
  - GlobalArrangement.boundsから幅・高さを計算するロジックを実装
  - サイズ0の場合はNoneを返す境界チェックを追加
  - 小数点以下の切り上げ処理を実装
  - _Requirements: 3.1, 3.2, 3.3_

## Task 2: Surface生成システムの改修

- [ ] 2.1 deferred_surface_creation_systemを拡張する
  - クエリ条件をGlobalArrangementベースに変更
  - サイズ計算にGlobalArrangement.boundsを使用
  - サイズ0の場合はスキップし、スキップ理由をログ出力
  - 既存SurfaceGraphicsとのサイズ比較による再作成判定を追加
  - 作成時にエンティティ名と物理ピクセルサイズをログ出力
  - 統計リソースの更新処理を追加
  - _Requirements: 1.1, 1.2, 2.2, 2.3, 3.1, 3.2, 3.3, 3.4, 5.1, 5.2_

- [ ] 2.2 sync_surface_from_arrangementシステムを廃止する
  - スケジュール登録からシステムを削除
  - システム関数自体は残し、deprecated属性を付与
  - _Requirements: 2.1, 2.4_

## Task 3: Surface削除システムの実装

- [ ] 3.1 cleanup_surface_on_commandlist_removedシステムを新規作成する
  - RemovedComponents<GraphicsCommandList>を検出するシステムを実装
  - 対象EntityからSurfaceGraphicsコンポーネントを削除
  - VisualGraphicsのSetContent(null)呼び出しを実装
  - 削除時のログ出力を追加
  - 統計リソースの削除カウンター更新を追加
  - _Requirements: 1.3, 1.4_

- [ ] 3.2 スケジュールにSurface削除システムを登録する
  - Drawスケジュールのdeferred_surface_creation_systemの後に配置
  - システム間の依存関係を設定
  - _Requirements: 1.4_

## Task 4: 統合テストと動作確認

- [ ] 4.1 GraphicsCommandList追加時のSurface作成を確認する
  - 描画コマンドを持つエンティティにSurfaceが作成されることを確認
  - 物理ピクセルサイズで作成されることを確認
  - ログ出力が正しいことを確認
  - _Requirements: 1.1, 3.1, 3.2, 5.2_

- [ ] 4.2 GraphicsCommandListなしのエンティティでスキップを確認する
  - レイアウトコンテナにSurfaceが作成されないことを確認
  - スキップ理由のログ出力を確認
  - _Requirements: 1.2, 2.4, 5.1_

- [ ] 4.3 既存サンプルアプリケーションでの動作確認
  - areka.rsサンプルで描画が正常であることを確認
  - taffy_flex_demo.rsで複雑な階層でも正常動作することを確認
  - DPI 100%/150%環境での描画確認（可能であれば）
  - _Requirements: 4.1, 4.2, 4.3, 4.4_

- [ ]* 4.4 ユニットテストを追加する
  - サイズ計算ロジックの境界ケーステスト（0, 小数点, 負値）
  - SurfaceCreationStatsの初期化・更新テスト
  - _Requirements: 3.3, 5.3_
