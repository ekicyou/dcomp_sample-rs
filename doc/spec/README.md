# bevy_ecsによるUI管理システム - 設計ドキュメント

このディレクトリには、bevy_ecsを使用したUIフレームワークのコンセプト設計ドキュメントがあります。

## 目次

1. [Entityツリー構造](01-widget-tree.md) - 基本的な考え方とEntity定義
2. [bevy_ecsコンポーネント管理](02-ecs-components.md) - コンポーネントシステムと変更検知
3. [レイアウトシステム](03-layout-system.md) - レイアウトコンポーネントの定義
4. [Visual: DirectComposition統合](04-visual-directcomp.md) - 描画システムとの統合
5. [システム統合と更新フロー](05-update-flow.md) - フレーム更新の流れ
6. [イベントシステム](06-event-system.md) - マウス・キーボードイベント処理
7. [基本的なUI要素](07-ui-elements.md) - Container、TextBlock、Imageなど
8. [レイアウトシステム詳細](08-layout-details.md) - Measure/Arrangeパス
9. [ヒットテストシステム](09-hit-test.md) - 座標からEntityを検索
10. [使用例](10-usage-examples.md) - サンプルコード
11. [ビジュアルツリーの最適化](11-visual-optimization.md) - Visual作成の最適化
12. [bevy_ecsと依存関係プロパティの関係性](12-dependency-properties.md) - WPFとの比較
13. [システム設計](13-system-separation.md) - 各システムの責務と統合

## 設計の要点

- **bevy_ecs完全採用**: Componentベースの柔軟なプロパティ管理
- **動的Visual作成**: 描画が必要なEntityのみがVisualコンポーネントを持つ
- **システム分離**: 関心事を明確に分離（Layout、Visual、Interaction等）
- **自動変更検知**: `Changed<T>`による効率的な更新
- **並列実行**: データ競合のないシステムは自動的に並列実行
- **2パスレイアウト**: Measure/Arrangeで効率的なレイアウト計算
- **マルチウィンドウ対応**: WindowもEntityとして統一的に管理

## まとめ

このUI構造設計の要点：

1. **bevy_ecsによる管理**: `#[derive(Component)]`でデータを定義、システム関数でロジックを実装
2. **Entity中心の設計**: すべてのUI要素はEntityとして存在（親子関係は`Parent`/`Children`）
3. **動的Visual作成**: 描画が必要なEntityのみがVisualコンポーネント（DirectComposition）を持つ
4. **オプショナルコンポーネント**: Layout、TextContent、ImageContent、InteractionStateなど必要に応じて追加
5. **自動変更追跡**: `Changed<T>`/`Added<T>`で変更を自動検知、手動ダーティフラグ不要
6. **リアクティブ更新**: システムチェーンで変更を伝播（TextContent → Layout → Visual → 再描画）
7. **イベントシステム**: Resourceとクエリで柔軟なイベント処理
8. **2パスレイアウト**: Measure/Arrangeで効率的なレイアウト計算
9. **ヒットテスト**: Entityツリーを使った深さ優先探索（Visualの有無に依存しない）
10. **基本UI要素**: Container、TextBlock、Image、Button、StackPanelをEntityとして提供
11. **効率的なメモリ使用**: スパースセットで不要なコンポーネントは存在しない
12. **並列処理**: データ競合のないシステムはbevy_ecsが自動的に並列実行
