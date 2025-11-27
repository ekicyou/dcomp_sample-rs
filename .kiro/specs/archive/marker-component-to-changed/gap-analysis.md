# ギャップ分析: マーカーコンポーネントからChanged検出への移行

## 1. 現状調査

### 1.1 影響を受けるファイル/モジュール

| ファイル | 役割 | 影響度 |
|---------|------|--------|
| `ecs/graphics/components.rs` | コンポーネント定義 | 高 |
| `ecs/graphics/systems.rs` | メインシステム群 | 高 |
| `ecs/graphics/visual_manager.rs` | Visual管理 | 中 |
| `ecs/graphics/mod.rs` | モジュールエクスポート | 低 |
| `ecs/mod.rs` | クレートエクスポート | 低 |
| `tests/surface_optimization_test.rs` | テスト | 高 |

### 1.2 既存パターン（マーカー方式）

**SurfaceUpdateRequested**:
```rust
// 定義（components.rs:203）
#[derive(Component, Default)]
pub struct SurfaceUpdateRequested;

// 挿入パターン
commands.entity(entity).insert(SurfaceUpdateRequested);

// 検出パターン
Query<..., With<SurfaceUpdateRequested>>

// 削除パターン
commands.entity(entity).remove::<SurfaceUpdateRequested>();
```

**GraphicsNeedsInit**:
```rust
// 定義（components.rs:16）
#[derive(Component, Default)]
pub struct GraphicsNeedsInit;

// 同様のinsert/With/removeパターン
```

### 1.3 insert vs Changed の本質的な違い

**insert()の問題点**:
- `commands.entity(entity).insert(Marker)`はCommandsキューに積まれる
- 同じスケジュール内の後続システムには変更が伝搬しない（Commands適用は各スケジュール終了時）
- アーキタイプ変更が発生（高コスト）

**Changed<T>の利点**:
- コンポーネントの値変更（`&mut T`経由）は**システム実行中に直ちに反映**
- 同一スケジュール内の後続システムで`Changed<T>`として即座に検出可能
- アーキタイプ変更なし（低コスト）

これが本移行の**最大の動機**である。

### 1.4 既存Changedパターン（参考）

コードベースではすでに`Changed<T>`を多数使用:
- `Changed<Arrangement>` - レイアウト変更検出
- `Changed<GlobalArrangement>` - グローバル配置変更検出
- `Changed<GraphicsCommandList>` - 描画コマンド変更検出
- `Changed<SurfaceGraphics>` - Surface変更検出

**重要な発見**: `mark_dirty_surfaces`システム（systems.rs:825）のコメントに「将来的にはSurfaceUpdateRequestedを廃止し、Changed<GraphicsCommandList>をrender_surfaceのフィルターとして直接使用する予定」と記載あり。

### 1.5 公開API状況

```rust
// ecs/graphics/mod.rs
pub use components::*;  // 全コンポーネントが公開

// ecs/mod.rs
pub use graphics::*;    // graphicsモジュール全体が公開

```

`SurfaceUpdateRequested`と`GraphicsNeedsInit`は`pub`として公開されており、外部から使用可能（テストで実際に使用中）。

---

## 2. 要件実現可能性分析

### 2.1 データモデル変更

| 要件 | 技術要件 | 実現可能性 | 備考 |
|------|---------|-----------|------|
| R1: SurfaceRenderTrigger定義 | 新規struct追加 | ✅ 容易 | Defaultで`requested_frame: 0` |
| R3: GraphicsInitState定義 | 新規struct + impl | ✅ 容易 | メソッド3つ追加 |
| R5: コンポーネント初期化統合 | spawn時の挿入追加 | ⚠️ 調査必要 | 既存spawnポイント特定必要 |

### 2.2 システム変更

| システム | 現状 | 変更内容 | 複雑度 |
|---------|------|---------|--------|
| `render_surface` | `With<SurfaceUpdateRequested>` | `Changed<SurfaceRenderTrigger>` | 低 |
| `mark_dirty_surfaces` | `insert(SurfaceUpdateRequested)` | フィールド更新 | 低 |
| `deferred_surface_creation_system` | `insert(SurfaceUpdateRequested)` | フィールド更新 | 低 |
| `on_surface_graphics_changed`フック | `SafeInsertSurfaceUpdateRequested`コマンド | 削除または変更 | 中 |
| `init_graphics_core` | `insert(GraphicsNeedsInit)` | `request_init()` | 低 |
| `init_window_graphics` | `With<GraphicsNeedsInit>` | `Changed<GraphicsInitState>` + 条件 | 中 |
| `init_window_visual` | `With<GraphicsNeedsInit>` | 同上 | 中 |
| `cleanup_graphics_needs_init` | `remove::<GraphicsNeedsInit>()` | `mark_initialized()` | 低 |
| `cleanup_command_list_on_reinit` | `With<GraphicsNeedsInit>` | 条件変更 | 低 |
| `create_visuals_for_init_marked` | `With<GraphicsNeedsInit>` | 条件変更 | 低 |

### 2.3 ギャップと制約

#### ギャップ0: 状態追跡コンポーネントの事前登録（重要）

**課題**: `Changed<T>`パターンでは、対象コンポーネントがエンティティ生成時に**事前に存在している必要がある**。後から`insert()`した場合、そのフレームでは`Changed`として検出されるが、タイミング依存の問題が発生しうる。

**現状の問題点**:
- 現在のマーカー方式は「必要になったら`insert()`」という遅延挿入パターン
- 新方式では「最初から存在し、値を変更して`Changed`を発火」というパターン
- エンティティ生成時にコンポーネントが欠落していると`Changed`検出が機能しない

**解決策**:
1. **ライフサイクルフックによる自動登録**: 関連コンポーネント（`HasGraphicsResources`, `Visual`等）の`on_add`フックで状態追跡コンポーネントを挿入
2. **集約関数の提供**: `ensure_graphics_state_components()`のような関数で一括管理
3. **Bundle定義**: `GraphicsStateBundle`として状態追跡コンポーネントをまとめる

**影響範囲**:
- `HasGraphicsResources`を持つエンティティ → `GraphicsInitState`が必要
- `Visual`を持つエンティティ → `SurfaceRenderTrigger`が必要

#### ギャップ1: on_surface_graphics_changedフックの変更方法

**現状**:
```rust
fn on_surface_graphics_changed(mut world: DeferredWorld, context: HookContext) {
    let mut commands = world.commands();
    commands.queue(SafeInsertSurfaceUpdateRequested {
        entity: context.entity,
    });
}
```

**課題**: フック内から`SurfaceRenderTrigger`のフィールドを更新するには、`world.get_mut::<SurfaceRenderTrigger>()`が必要。しかしフック発火時点でコンポーネントが存在するか不明。

**解決策オプション**:
- A: フックを維持し、カスタムCommandで`SurfaceRenderTrigger`を更新
- B: フックを削除し、`Added<SurfaceGraphics>`でシステム検出
- C: `SurfaceGraphics`にフラグを内包（設計変更）
- **D（推奨）**: `Visual`の`on_add`フックで`SurfaceRenderTrigger`を事前登録し、`SurfaceGraphics`フックでは値更新のみ

#### ギャップ2: Changed<T>の初期フレーム挙動

`Changed<T>`はコンポーネント追加時にも`true`を返す（`Added<T>`と同様）。これは既存の動作と互換性がある。

#### ギャップ3: GraphicsInitStateの初期状態

**要件**: 初期状態で`needs_init() == false`であること

**解決策**: `Default::default()`で`needs_init_generation: 0, processed_generation: 0`となり、`needs_init()`は`false`を返す ✅

#### ギャップ4: フレームカウントリソースへのアクセス

`SurfaceRenderTrigger.requested_frame`更新には`FrameCount`リソースが必要。

**現状**: 多くのシステムで`Res<FrameCount>`を既に使用 ✅

---

## 3. 実装アプローチオプション

### Option A: 段階的移行（推奨）

**概要**: 2フェーズに分けて移行。各フェーズ後にテスト確認。

**Phase 1: SurfaceUpdateRequested移行**
1. `SurfaceRenderTrigger`定義追加
2. 既存システム変更
3. フック変更（Option B-2採用推奨）
4. テスト更新
5. 旧コンポーネント削除

**Phase 2: GraphicsNeedsInit移行**
1. `GraphicsInitState`定義追加
2. 既存システム変更
3. テスト更新
4. 旧コンポーネント削除

**Trade-offs**:
- ✅ リスク分散（問題発生時の切り分けが容易）
- ✅ 各フェーズでテスト実行可能
- ❌ 作業期間が長くなる

### Option B: 一括移行

**概要**: すべてのマーカーを一度に移行。

**Trade-offs**:
- ✅ 作業期間が短い
- ✅ コードベース全体の一貫性が即座に確保
- ❌ 問題発生時の切り分けが困難
- ❌ テスト失敗時の原因特定が難しい

### Option B-1: フック維持（on_surface_graphics_changed）

```rust
struct SafeUpdateSurfaceRenderTrigger {
    entity: Entity,
}

impl Command for SafeUpdateSurfaceRenderTrigger {
    fn apply(self, world: &mut World) {
        let frame = world.resource::<FrameCount>().0;
        if let Some(mut trigger) = world.get_mut::<SurfaceRenderTrigger>(self.entity) {
            trigger.requested_frame = frame;
        }
    }
}
```

**Trade-offs**:
- ✅ 既存のフック構造を維持
- ❌ カスタムCommandが残る

### Option B-2: フック削除（推奨）

フックを削除し、`mark_dirty_surfaces`システムで`Added<SurfaceGraphics>`も検出。

```rust
pub fn mark_dirty_surfaces(
    mut query: Query<
        &mut SurfaceRenderTrigger,
        Or<(
            Changed<GraphicsCommandList>,
            Changed<SurfaceGraphics>,
            Added<SurfaceGraphics>,  // フック代替
            Changed<GlobalArrangement>,
        )>,
    >,
    frame_count: Res<FrameCount>,
) {
    for mut trigger in query.iter_mut() {
        trigger.requested_frame = frame_count.0;
    }
}
```

**Trade-offs**:
- ✅ カスタムCommand削除でコード簡素化
- ✅ 全トリガーロジックが一箇所に集約
- ⚠️ `SurfaceRenderTrigger`が`SurfaceGraphics`より先に存在する必要あり

---

## 4. 複雑度とリスク評価

### 工数見積もり

| タスク | 見積もり |
|--------|---------|
| コンポーネント定義 | S (1日) |
| SurfaceRenderTriggerシステム変更 | M (2-3日) |
| GraphicsInitStateシステム変更 | M (2-3日) |
| テスト更新 | S (1日) |
| ドキュメント | S (0.5日) |
| **合計** | **M (1週間程度)** |

### リスク評価: **Medium**

**リスク要因**:
- Changed<T>の挙動がマーカー方式と微妙に異なる可能性
- フック変更時の副作用
- テストでカバーされていないエッジケース

**緩和策**:
- 段階的移行で問題を早期発見
- 既存の`Changed<T>`パターンを参考に実装
- `cargo test --all-targets`で全テスト確認

---

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ

**Option A（段階的移行）** + **Option B-2（フック削除）**

### 設計フェーズでの調査事項

1. **エンティティ生成ポイントの特定**: `SurfaceRenderTrigger`と`GraphicsInitState`を挿入すべき全箇所
2. **システム実行順序の確認**: `Changed`検出が正しく機能する順序
3. **フック削除の副作用確認**: `Added<SurfaceGraphics>`で完全に代替可能か

### 次のステップ

1. `/kiro-spec-design marker-component-to-changed` で設計ドキュメント生成
2. 設計で詳細なシステム実行順序とコンポーネント配置を定義
