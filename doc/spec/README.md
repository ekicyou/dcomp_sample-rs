# slotmapによるプロパティ管理システム - 設計ドキュメント

このディレクトリには、UIフレームワークのコンセプト設計ドキュメントがあります。

## 目次

1. [Widgetツリー構造](01-widget-tree.md) - 基本的な考え方とWidget定義
2. [ECSプロパティ管理](02-ecs-components.md) - コンポーネントシステムと依存関係管理
3. [レイアウトシステム](03-layout-system.md) - レイアウトプロパティの定義
4. [Visual: DirectComposition統合](04-visual-directcomp.md) - 描画システムとの統合
5. [システム統合と更新フロー](05-update-flow.md) - フレーム更新の流れ
6. [イベントシステム](06-event-system.md) - マウス・キーボードイベント処理
7. [基本的なUI要素](07-ui-elements.md) - Container、TextBlock、Imageなど
8. [レイアウトシステム詳細](08-layout-details.md) - Measure/Arrangeパス
9. [ヒットテストシステム](09-hit-test.md) - 座標からWidgetを検索
10. [使用例](10-usage-examples.md) - サンプルコード
11. [ビジュアルツリーの最適化](11-visual-optimization.md) - Visual作成の最適化
12. [ECSと依存関係プロパティの関係性](12-dependency-properties.md) - WPFとの比較
13. [ECSシステム分離設計](13-system-separation.md) - 各システムの責務と統合

## 設計の要点

- **ECS的な管理**: SlotMapとSecondaryMapで柔軟なプロパティ管理
- **動的Visual作成**: 描画が必要なWidgetのみがVisualを持つ
- **システム分離**: 関心事を明確に分離（Widget、Layout、Visual、Interaction等）
- **2パスレイアウト**: Measure/Arrangeで効率的なレイアウト計算
- **マルチウィンドウ対応**: WindowもWidgetとして統一的に管理

## まとめ

このUI構造設計の要点：

1. **ECS的な管理**: SlotMapとSecondaryMapで柔軟なプロパティ管理
2. **必須コンポーネント**: すべてのWidgetはWidget（ツリー構造）を持つ
3. **動的Visual作成**: 描画が必要なWidgetのみがVisual（DirectComposition）を持つ
4. **オプショナルコンポーネント**: Layout、TextContent、ImageContent、InteractionStateなど必要に応じて追加
5. **イベントシステム**: ハンドラベースで柔軟なイベント処理
6. **2パスレイアウト**: Measure/Arrangeで効率的なレイアウト計算
7. **ヒットテスト**: Widgetツリーを使った深さ優先探索（Visualの有無に依存しない）
8. **基本UI要素**: Container、TextBlock、Image、Button、StackPanelを提供
9. **効率的なメモリ使用**: 不要なVisualを作成しない
10. **段階的な分離**: 現在は`WidgetSystem`で統合管理、将来的にシステム分離を検討
