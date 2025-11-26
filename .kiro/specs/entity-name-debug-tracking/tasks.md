# Implementation Plan

## Task 1: visual_hierarchy_sync_systemのName対応

- [ ] 1.1 (P) format_entity_nameヘルパー関数の実装
  - エンティティとNameコンポーネントを受け取り、ログ用文字列を返す関数を作成
  - Nameがある場合は名前をそのまま返す
  - Nameがない場合は`Entity(0v1)`形式でEntity IDを返す
  - _Requirements: 4.4_

- [ ] 1.2 visual_hierarchy_sync_systemのクエリ拡張
  - 子エンティティクエリに`Option<&Name>`を追加
  - 親エンティティクエリに`Option<&Name>`を追加
  - 未同期エンティティ収集時にNameも一緒に取得
  - _Requirements: 3.1_

- [ ] 1.3 ログ出力フォーマットの変更
  - add_visual成功時のログを新フォーマットに変更: `child="ChildName" -> parent="ParentName"`
  - add_visual失敗時のログを新フォーマットに変更: `child="ChildName", parent="ParentName", error={:?}`
  - Visual階層ルート検出時のログを新フォーマットに変更: `Visual hierarchy root: name="RootName"`
  - Nameがない場合はformat_entity_nameでEntity IDにフォールバック
  - _Requirements: 2.1, 2.2, 2.3, 3.2, 3.3, 4.1, 4.2, 4.3_

## Task 2: taffy_flex_demoへのName付与

- [ ] 2.1 (P) bevy_ecs::name::Nameのインポート追加
  - taffy_flex_demoファイルにNameコンポーネントのuseステートメントを追加
  - _Requirements: 1.1_

- [ ] 2.2 各エンティティへのName付与
  - Windowエンティティに`Name::new("FlexDemo-Window")`を追加
  - FlexContainerエンティティに`Name::new("FlexDemo-Container")`を追加
  - RedBoxエンティティに`Name::new("RedBox")`を追加
  - GreenBoxエンティティに`Name::new("GreenBox")`を追加
  - BlueBoxエンティティに`Name::new("BlueBox")`を追加
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

## Task 3: 動作確認とテスト

- [ ] 3.1 taffy_flex_demoの実行確認
  - taffy_flex_demoを実行してエラーがないことを確認
  - visual_hierarchy_sync_systemのログにエンティティ名が出力されることを確認
  - 親子関係が正しくログに表示されることを確認（例: `child="RedBox" -> parent="FlexDemo-Container"`）
  - _Requirements: 2.1, 2.2, 2.3, 4.1, 4.2, 4.3_
