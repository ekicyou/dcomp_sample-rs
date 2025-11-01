# bevy_ecsによるUI管理システム - 設計ドキュメント

このディレクトリには、bevy_ecsを使用したUIフレームワークの設計ドキュメントがあります。

## 目次

### 第1部: bevy_ecs基礎

1. [bevy_ecsコンポーネント管理](01-ecs-components.md) - ECS概念、Entity、Component、System
2. [Entityツリー構造](02-widget-tree.md) - UIツリーの表現、Parent/Children
3. [システム設計](03-system-separation.md) - 各システムの責務と統合

### 第2部: UIシステム実装

4. [レイアウトシステム](04-layout-system.md) - レイアウトコンポーネントの定義
5. [レイアウトシステム詳細](05-layout-details.md) - Measure/Arrangeパス
6. [Visual: DirectComposition統合](06-visual-directcomp.md) - 描画システムとの統合
7. [システム統合と更新フロー](07-update-flow.md) - フレーム更新の流れ

### 第3部: インタラクション

8. [イベントシステム](08-event-system.md) - マウス・キーボードイベント処理
9. [ヒットテストシステム](09-hit-test.md) - 座標からEntityを検索

### 第4部: UI要素と使用例

10. [基本的なUI要素](10-ui-elements.md) - Container、TextBlock、Imageなど
11. [使用例](11-usage-examples.md) - サンプルコード

### 第5部: 最適化（参考）

12. [ビジュアルツリーの最適化](12-visual-optimization.md) - Visual作成の最適化

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
11. **効率的なメモリ使用**: コンポーネントベース設計で不要なデータを持たない
12. **並列処理**: データ競合のないシステムはbevy_ecsが自動的に並列実行

## 除外されたドキュメント

以下のドキュメントは検討段階のものであり、設計文書から除外されました：

- **12-dependency-properties.md** - WPFとの比較検討（参考資料として保持）
