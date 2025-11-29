# Requirements Document

## Project Description (Input)
bevy_ecsのNameコンポーネントを使って、taffy_flex_demoで作っている各エンティティに「Red-Rectangle」などと一意の名前を付け、エンティティ間の相互作用について追跡したい。特に、visual_hierarchy_sync_systemが正しく親ビジュアルにadd_visual出来ているかを追跡したい。add_visualしている親子の名前をログに残したい。

## Introduction
本仕様は、bevy_ecsの`Name`コンポーネントを活用してエンティティにわかりやすい名前を付け、デバッグログでエンティティ間の相互作用（特にVisual階層構築）を追跡可能にする機能を定義する。Entity IDだけでは識別困難な状況を改善し、開発効率を向上させる。

## Requirements

### Requirement 1: taffy_flex_demoへのName付与
**Objective:** 開発者として、taffy_flex_demoの各エンティティに人間が読める名前を付けたい。これによりログ出力でエンティティを即座に識別できるようになる。

#### Acceptance Criteria
1. When taffy_flex_demoがWindowエンティティを生成するとき, the taffy_flex_demoは `Name::new("FlexDemo-Window")` コンポーネントを付与しなければならない
2. When taffy_flex_demoがFlexContainerエンティティを生成するとき, the taffy_flex_demoは `Name::new("FlexDemo-Container")` コンポーネントを付与しなければならない
3. When taffy_flex_demoがRedBoxエンティティを生成するとき, the taffy_flex_demoは `Name::new("RedBox")` コンポーネントを付与しなければならない
4. When taffy_flex_demoがGreenBoxエンティティを生成するとき, the taffy_flex_demoは `Name::new("GreenBox")` コンポーネントを付与しなければならない
5. When taffy_flex_demoがBlueBoxエンティティを生成するとき, the taffy_flex_demoは `Name::new("BlueBox")` コンポーネントを付与しなければならない

### Requirement 2: visual_hierarchy_sync_systemのログ拡張
**Objective:** 開発者として、visual_hierarchy_sync_systemでadd_visualが呼ばれた際に、親子両方のエンティティ名をログに出力したい。これによりVisual階層構築の正しさを検証できる。

#### Acceptance Criteria
1. When visual_hierarchy_sync_systemがadd_visualを成功させたとき, the wintfシステムは親エンティティのName（存在する場合）と子エンティティのName（存在する場合）をログに出力しなければならない
2. When visual_hierarchy_sync_systemがadd_visualを成功させたとき and 親または子にNameがないとき, the wintfシステムはEntity ID をフォールバックとして使用しなければならない
3. When visual_hierarchy_sync_systemがVisual階層のルートを検出したとき, the wintfシステムはルートエンティティのName（存在する場合）をログに出力しなければならない

### Requirement 3: Nameコンポーネントのクエリ対応
**Objective:** 開発者として、visual_hierarchy_sync_systemがNameコンポーネントを参照できるようにしたい。これによりエンティティ名をログに含めることができる。

#### Acceptance Criteria
1. The visual_hierarchy_sync_systemは、クエリに `Option<&Name>` を含めてNameコンポーネントを取得できなければならない
2. While Nameコンポーネントが存在しないとき, the wintfシステムは `Entity={:?}` 形式でEntity IDを出力しなければならない
3. While Nameコンポーネントが存在するとき, the wintfシステムは `Name="xxx"` 形式でエンティティ名を出力しなければならない

### Requirement 4: ログ出力フォーマット
**Objective:** 開発者として、一貫したログ出力フォーマットでVisual階層構築を追跡したい。これによりログ解析が容易になる。

#### Acceptance Criteria
1. The visual_hierarchy_sync_systemのadd_visual成功ログは、 `[visual_hierarchy_sync] AddVisual success: child="ChildName" -> parent="ParentName"` フォーマットを使用しなければならない
2. The visual_hierarchy_sync_systemのadd_visual失敗ログは、 `[visual_hierarchy_sync] AddVisual failed: child="ChildName", parent="ParentName", error={:?}` フォーマットを使用しなければならない
3. The visual_hierarchy_sync_systemのルート検出ログは、 `[visual_hierarchy_sync] Visual hierarchy root: name="RootName"` フォーマットを使用しなければならない
4. If Nameが存在しないとき, the wintfシステムはEntity IDを `"Entity(0v1)"` 形式で代替表示しなければならない

## Non-Goals
- 全システムへのName対応拡張（本仕様はvisual_hierarchy_sync_systemのみ）
- Nameコンポーネントの自動生成
- ログレベルの動的制御
- ログのファイル出力

## Success Criteria
1. taffy_flex_demoの全エンティティにNameコンポーネントが付与されている
2. visual_hierarchy_sync_systemのログにエンティティ名が表示される
3. Visual階層構築時の親子関係がログで明確に追跡できる

## Dependencies
- **bevy_ecs 0.17.2**: `bevy_ecs::name::Name` コンポーネント
- **visual-tree-synchronization仕様**: visual_hierarchy_sync_systemの基本実装

