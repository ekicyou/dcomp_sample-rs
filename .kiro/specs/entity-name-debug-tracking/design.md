# Design Document: entity-name-debug-tracking

## Overview

**Purpose**: bevy_ecsの`Name`コンポーネントを活用してエンティティに人間が読める名前を付与し、`visual_hierarchy_sync_system`のログ出力でエンティティ間の親子関係を明確に追跡可能にする。

**Users**: wintfライブラリの開発者がVisual階層構築のデバッグに使用する。

**Impact**: 既存の`visual_hierarchy_sync_system`のログ出力を拡張し、`taffy_flex_demo`サンプルにNameコンポーネントを追加する。

### Goals
- taffy_flex_demoの全エンティティに識別可能な名前を付与
- visual_hierarchy_sync_systemのログにエンティティ名を含める
- Visual階層構築時の親子関係をログで追跡可能にする

### Non-Goals
- 全システムへのName対応拡張（本仕様はvisual_hierarchy_sync_systemのみ）
- Nameコンポーネントの自動生成
- ログレベルの動的制御
- ログのファイル出力

## Architecture

### Existing Architecture Analysis

現在の`visual_hierarchy_sync_system`は以下の構造を持つ:

1. **ParamSetによるクエリ分離**: 子エンティティと親エンティティを別々のクエリで取得
2. **2パス処理**: まず未同期エンティティを収集し、その後処理
3. **ログ出力**: Entity IDのみを使用（`{:?}`フォーマット）

本変更は、既存のクエリにNameコンポーネントを追加し、ログ出力フォーマットを拡張するのみ。アーキテクチャの変更は不要。

### Technology Stack

| Layer | Choice / Version | Role in Feature | Notes |
|-------|------------------|-----------------|-------|
| ECS | bevy_ecs 0.17.2 | `Name`コンポーネント提供 | `bevy_ecs::name::Name` |
| Example | taffy_flex_demo | Name付与対象 | 5エンティティに追加 |
| System | visual_hierarchy_sync_system | ログ拡張対象 | クエリ変更とログフォーマット変更 |

## Requirements Traceability

| Requirement | Summary | Components | Interfaces | Flows |
|-------------|---------|------------|------------|-------|
| 1.1-1.5 | taffy_flex_demoへのName付与 | taffy_flex_demo | spawn() | - |
| 2.1-2.3 | visual_hierarchy_sync_systemログ拡張 | visual_hierarchy_sync_system | eprintln! | Visual階層同期フロー |
| 3.1-3.3 | Nameコンポーネントのクエリ対応 | visual_hierarchy_sync_system | Query | - |
| 4.1-4.4 | ログ出力フォーマット | visual_hierarchy_sync_system | eprintln! | - |

## Components and Interfaces

| Component | Domain/Layer | Intent | Req Coverage | Key Dependencies | Contracts |
|-----------|--------------|--------|--------------|------------------|-----------|
| taffy_flex_demo | Example | Name付与デモ | 1.1-1.5 | bevy_ecs::name::Name (P0) | - |
| visual_hierarchy_sync_system | ECS/Graphics | Visual階層同期とログ | 2.1-2.3, 3.1-3.3, 4.1-4.4 | bevy_ecs::name::Name (P1) | Service |
| format_entity_name | ECS/Graphics | 名前フォーマット | 4.4 | - | Service |

### ECS/Graphics Layer

#### visual_hierarchy_sync_system

| Field | Detail |
|-------|--------|
| Intent | ECS階層とDirectComposition Visual階層を同期し、親子関係をログ出力 |
| Requirements | 2.1, 2.2, 2.3, 3.1, 3.2, 3.3, 4.1, 4.2, 4.3, 4.4 |

**Responsibilities & Constraints**
- 未同期のVisualGraphicsを検出し、親Visualに追加
- 親子エンティティのNameを取得してログに出力
- Nameがない場合はEntity IDをフォールバック

**Dependencies**
- Inbound: なし
- Outbound: なし
- External: bevy_ecs::name::Name — エンティティ名取得 (P1)

**Contracts**: Service [x]

##### Service Interface

```rust
/// visual_hierarchy_sync_systemのシグネチャ変更
pub fn visual_hierarchy_sync_system(
    mut vg_queries: ParamSet<(
        // 子エンティティクエリ: Option<&Name>を追加
        Query<(Entity, &ChildOf, &mut VisualGraphics, Option<&Name>)>,
        // 親エンティティクエリ: Option<&Name>を追加
        Query<(&VisualGraphics, Option<&Name>)>,
    )>,
)
```

- Preconditions: WorldにNameコンポーネントが登録されていること
- Postconditions: ログ出力にエンティティ名が含まれる
- Invariants: Nameがない場合はEntity IDで代替

#### format_entity_name ヘルパー関数

| Field | Detail |
|-------|--------|
| Intent | エンティティ名のフォーマット統一 |
| Requirements | 4.4 |

**Contracts**: Service [x]

##### Service Interface

```rust
/// エンティティ名をログ用にフォーマットする
/// 
/// # Arguments
/// * `entity` - エンティティID
/// * `name` - Nameコンポーネント（オプション）
/// 
/// # Returns
/// Nameがあれば `"Name"` 形式、なければ `"Entity(0v1)"` 形式
fn format_entity_name(entity: Entity, name: Option<&Name>) -> String {
    match name {
        Some(n) => n.to_string(),
        None => format!("Entity({:?})", entity),
    }
}
```

- Preconditions: なし
- Postconditions: 常に有効な文字列を返す
- Invariants: なし

### Example Layer

#### taffy_flex_demo

| Field | Detail |
|-------|--------|
| Intent | Flexboxレイアウトのデモとエンティティ名付与 |
| Requirements | 1.1, 1.2, 1.3, 1.4, 1.5 |

**Responsibilities & Constraints**
- 各エンティティにNameコンポーネントを付与
- 名前は一意で識別可能であること

**Dependencies**
- External: bevy_ecs::name::Name — エンティティ名付与 (P0)

**Implementation Notes**
- 既存のspawn()呼び出しにName::new()を追加
- 名前規則: `FlexDemo-Window`, `FlexDemo-Container`, `RedBox`, `GreenBox`, `BlueBox`

## Data Models

### Domain Model

本仕様で新規データモデルの追加はなし。既存の`bevy_ecs::name::Name`コンポーネントを使用。

```rust
// bevy_ecs::name::Name の定義（参考）
pub struct Name(Cow<'static, str>);
```

## Error Handling

### Error Strategy

本仕様ではエラー処理の変更なし。Nameコンポーネントはオプショナルであり、存在しない場合はEntity IDにフォールバック。

## Testing Strategy

### Unit Tests
1. `format_entity_name`関数のテスト: Nameあり/なしの両ケース
2. ログフォーマットの検証: 期待されるフォーマットとの一致

### Integration Tests
1. taffy_flex_demoの実行: 全エンティティにNameが付与されていることを確認
2. visual_hierarchy_sync_systemのログ出力: エンティティ名が含まれることを確認

### E2E Tests
1. taffy_flex_demo実行時のログ確認: Visual階層構築時に親子名が出力される
