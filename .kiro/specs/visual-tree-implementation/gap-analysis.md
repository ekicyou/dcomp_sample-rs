# Implementation Gap Analysis: visual-tree-implementation

**Feature ID**: `visual-tree-implementation`  
**Analyzed**: 2025-11-17  
**Approved**: 2025-11-17  
**Status**: Gap Analysis Approved  
**Language**: 日本語

---

## Analysis Summary

**スコープ**: Entity階層（ChildOf/Children）の導入、Arrangementコンポーネントによる座標変換、階層的Surfaceレンダリング（WindowのSurfaceに全子孫を深さ優先描画）

**主要な課題**:
- bevy_ecs::hierarchyのChildOf/Childrenをwintfに統合
- Rectangle/Labelのx/yフィールド廃止とArrangement移行
- render_surfaceの深さ優先階層描画への拡張（Entity階層を辿って全子孫をWindowのSurfaceに描画）
- Arrangement伝播システムの実装

**推奨アプローチ**: Hybrid（Option C）
- 既存: VisualGraphics/SurfaceGraphics（Windowのみ）、tree_system.rs、bevy_ecs::hierarchy (ChildOf/Children)
- 新規: Arrangement/GlobalArrangement/ArrangementTreeChanged、階層的render_surface、arrangement.rs

---

## 1. Current State Investigation

### 既存アセット

#### ECSコンポーネント層 (`crates/wintf/src/ecs/`)
- **graphics/components.rs**:
  - `VisualGraphics` - IDCompositionVisual3ラッパー（単体のみ、階層構築なし）
  - `SurfaceGraphics` - IDCompositionSurfaceラッパー
  - `WindowGraphics` - IDCompositionTarget保持
- **layout.rs**:
  - `BoxStyle` - taffyレイアウト入力
  - `BoxComputedLayout` - taffyレイアウト出力（未使用）
- **widget/**:
  - `Rectangle` { x, y, width, height, color } - 矩形Widget
  - `Label` { text, font_family, font_size, color, x, y } - テキストWidget
- **tree_system.rs**:
  - `sync_simple_transforms<L, G, M>()` - ルートEntity変換更新
  - `mark_dirty_trees<L, G, M>()` - ダーティビット伝播
  - `propagate_parent_transforms<L, G, M>()` - 親→子への変換伝播
  - ジェネリック実装で、bevy_ecs::hierarchy::{ChildOf, Children}を利用可能

#### bevy_ecs 0.17.2階層ユーティリティ
- **bevy_ecs::hierarchy**:
  - `ChildOf<E>` - 子→親の参照(Component)
  - `Children<E>` - 親→子のリスト(Component、`RelationshipTarget`トレイトを実装)
- **bevy_ecs::relationship**:
  - `RelationshipTarget` - 関係性のターゲット側を表すトレイト(`Children`が実装)
  - `DescendantDepthFirstIter<'w, 's, D, F, S: RelationshipTarget>` - 深さ優先子孫イテレータ
  - `DescendantIter<'w, 's, D, F, S: RelationshipTarget>` - 一般子孫イテレータ
  - `AncestorIter<'w, 's, D, F, R: Relationship>` - 祖先イテレータ(上向き走査)
- **Query::iter_descendants_depth_first<S: RelationshipTarget>(&self, entity: Entity)**:
  - Queryメソッドとして深さ優先探索を提供
  - `Children`が`RelationshipTarget`を実装しているため、`query.iter_descendants_depth_first::<Children>(root_entity)`で使用可能
  - 型推論により`query.iter_descendants_depth_first(root_entity)`とも書ける

#### COM APIラッパー (`crates/wintf/src/com/dcomp.rs`)
- `DCompositionTargetExt::set_root()` - ルートVisual設定
- `DCompositionVisualExt::add_visual()` - 子Visual追加（今回の実装で使用）
- `DCompositionVisualExt::set_offset_x/y()` - オフセット設定（将来のArrangement適用で使用予定）
- `DCompositionVisualExt::set_content()` - Surface設定（各Entity用Surface設定ですでに使用中）

#### 描画システム (`crates/wintf/src/ecs/`)
- **graphics/systems.rs**:
  - `render_surface()` - 単一EntityのGraphicsCommandListをSurfaceに描画
  - 現在: 1 Entity = 1 Surface、子孫の描画は未実装
- **widget/shapes/rectangle.rs**:
  - `draw_rectangles()` - RectangleをGraphicsCommandListに追加
- **widget/text/draw_labels.rs**:
  - `draw_labels()` - LabelをGraphicsCommandListに追加

#### スケジュール実行順序 (`crates/wintf/src/ecs/world.rs`)
```
PostLayout:
  - init_graphics_core
  - cleanup_command_list_on_reinit
  - init_window_graphics
  - init_window_visual
  - init_window_surface

Draw:
  - cleanup_graphics_needs_init
  - draw_rectangles
  - draw_labels

RenderSurface:
  - render_surface

CommitComposition:
  - commit_composition
```

### アーキテクチャパターン

- **レイヤードアーキテクチャ**: COM → ECS → Message Handling
- **ECSコンポーネント命名規則**:
  - `XxxGraphics`: GPUリソース（DeviceLost対応、invalidate()実装）
  - `XxxResource`: CPUリソース（永続的）
- **変更検知パターン**: `Changed<T>`, `Added<T>`, `RemovedComponents<T>`使用
- **unsafe隔離**: COMラッパー層にunsafeを集約、上位層は安全なAPI

### 統合ポイント

- **bevy_ecs 0.17.2**: ECS基盤、ジェネリック階層伝播システム実装済み
- **windows 0.62.2**: Windows API バインディング、DirectComposition対応
- **taffy 0.9.1**: レイアウトエンジン（今回は未統合、将来実装）

---

## 2. Requirements Feasibility Analysis

### 技術ニーズ（requirements.mdより）

#### R1: Window Visual/Surface作成
- **必要**: Window用のVisualGraphics/SurfaceGraphics作成
- **現状**: 実装済み（init_window_visual, init_window_surface）
- **ギャップ**: なし（要件を満たしている）

#### R2: Entity階層構築（ChildOf/Children）
- **必要**: bevy_ecs::hierarchy::{ChildOf, Children}をwintfに統合
- **現状**: bevy_ecs 0.17.2にChildOf/Children実装済み
- **ギャップ**: wintfへのインポートと使用方法の整備（ドキュメント、サンプル）

#### R3: Window Visual/Surfaceライフサイクル管理
- **必要**: Window EntityのVisual+Surface自動追加、despawn時のクリーンアップ
- **現状**: 実装済み（init_window_visual, init_window_surface, on_removeフック）
- **ギャップ**: なし（要件を満たしている）

#### R4: Arrangementコンポーネント
- **必要**: Offset, LayoutScale, Arrangement, GlobalArrangement, ArrangementTreeChanged
- **現状**: layout.rsにBoxStyle/BoxComputedLayoutのみ、tree_system.rsはChildOf/Childrenを利用可能
- **ギャップ**: Arrangement関連コンポーネントが未実装
- **必要**: tree_system.rsパターンの適用（sync_simple_transforms, mark_dirty_trees, propagate_parent_transforms）

#### R5: ルートVisual管理
- **必要**: Window EntityへのVisual/Surface自動追加、SetRoot呼び出し
- **現状**: 実装済み（init_window_visual, init_window_surface）
- **ギャップ**: なし（要件を満たしている）

#### R6: 階層的Surfaceレンダリング
- **必要**: WindowのSurfaceに自分+全子孫を深さ優先描画、GlobalArrangementをSetTransformで適用
- **現状**: render_surfaceは単一EntityのGraphicsCommandListのみ描画
- **ギャップ**: 各子孫描画時のSetTransform適用が未実装(深さ優先探索は`Query::iter_descendants_depth_first::<Children>`で可能、`Children`は`RelationshipTarget`実装済み)

#### R7: 変更検知と効率的更新
- **必要**: Changed<Arrangement>, ArrangementTreeChangedによる差分更新
- **現状**: Changed/Added/Removedパターンは既存システムで使用中
- **ギャップ**: Arrangement伝播システム（sync_simple_arrangements, mark_dirty_arrangement_trees, propagate_global_arrangements）が未実装

#### R8: エラーハンドリング
- **必要**: Visual作成失敗時のログ出力とスキップ
- **現状**: 既存システムはeprintln!とResult::ok()で対応
- **ギャップ**: なし（既存パターンで対応可能）

#### R9: サンプルアプリケーション
- **必要**: simple_window.rsに複雑なツリー構造（6 Rectangle + 2 Label、最大4階層）
- **現状**: simple_window.rsは単純なWindow+Rectangle+Label
- **ギャップ**: 階層構造、ChildOf設定、Arrangement設定、色指定追加が必要

#### R10: パフォーマンス要件
- **必要**: 50個のRectangle/Labelで60fps維持、変更なしフレームではCommitのみ
- **現状**: 既存システムはChanged検知で差分更新
- **ギャップ**: パフォーマンス測定とチューニングが必要（実装後）

### 制約とギャップ

| 要件 | ギャップ種別 | 詳細 |
|------|--------------|------|
| R1 | OK | 既存実装で要件を満たす（Window Visual/Surface作成済み） |
| R2 | Minimal | bevy_ecs::hierarchyインポートとサンプル整備 |
| R3 | OK | 既存実装で要件を満たす（Window ライフサイクル管理済み） |
| R4 | Missing | Arrangement関連コンポーネント全て、伝播システム |
| R5 | OK | 既存実装で要件を満たす |
| R6 | Missing | 階層的Surfaceレンダリング、Children深さ優先探索、SetTransform適用 |
| R7 | Missing | Arrangement伝播システム（sync/mark/propagate） |
| R8 | OK | 既存パターンで対応可能 |
| R9 | Missing | 複雑なツリー構造サンプル、ChildOf/Arrangement設定 |
| R10 | Unknown | パフォーマンス測定必要（実装後） |

### 複雑性シグナル

- **bevy_ecs標準機能利用**: ChildOf/Children（bevy_ecs::hierarchy）をwintfに統合
- **既存パターン拡張**: tree_system.rsジェネリック関数の具体化
- **統合複雑性**: 深さ優先Surfaceレンダリング（再帰 or スタック管理）
- **ライフサイクル管理**: Visual/Surface作成とEntity階層の同期

---

## 3. Implementation Approach Options

### Option A: Extend Existing Components ❌

**対象外の理由**:
- 新規コンポーネント（ChildOf/Children/Arrangement系）が多数必要
- render_surfaceの単純拡張では深さ優先描画の複雑性に対応困難
- 既存Rectangle/Labelのx/y廃止は構造変更なので「拡張」ではない

### Option B: Create New Components 🔺

**新規作成対象**:
- **layout.rs**: Offset, LayoutScale, Arrangement, GlobalArrangement, ArrangementTreeChanged
- **arrangement.rs** (新規): sync_simple_arrangements, mark_dirty_arrangement_trees, propagate_global_arrangements
- **import**: bevy_ecs::hierarchy::{ChildOf, Children}を使用

**統合ポイント**:
- 既存: VisualGraphics/SurfaceGraphics（コンポーネント）
- 既存: dcomp.rs（DirectComposition APIラッパー）
- 既存: tree_system.rs（ジェネリックパターンの参考実装）

**Trade-offs**:
- ✅ 責務分離が明確（階層管理、レイアウト変換、Visual構築が独立）
- ✅ 既存コードへの影響最小
- ❌ 新規ファイル追加による構成複雑化
- ❌ Rectangle/Label修正（x/y削除）は避けられない

### Option C: Hybrid Approach ✅ 推奨

**拡張対象**:
- **layout.rs**: Offset, LayoutScale, Arrangement, GlobalArrangement, ArrangementTreeChangedを追加
- **graphics/systems.rs**: render_surfaceを階層的描画に拡張
- **widget/shapes/rectangle.rs**: x/yフィールド削除、on_removeフック追加
- **widget/text/label.rs**: x/yフィールド削除、on_removeフック追加
- **world.rs**: スケジュールにbuild_visual_tree、Arrangement伝播システムを追加

**新規作成対象**:
- **ecs/arrangement.rs**: sync_simple_arrangements, mark_dirty_arrangement_trees, propagate_global_arrangementsシステム
- **import**: `use bevy_ecs::hierarchy::{ChildOf, Children};` で標準階層コンポーネントを利用

**段階的実装戦略**:

#### Phase 1: Entity階層とArrangement基盤
1. `use bevy_ecs::hierarchy::{ChildOf, Children};` でbevy_ecs標準階層コンポーネントをインポート
2. layout.rsにArrangement関連コンポーネント追加（Offset, LayoutScale, Arrangement, GlobalArrangement, ArrangementTreeChanged）
3. 既存tree_system.rsパターンをArrangement用にコピー（arrangement.rs作成）

#### Phase 2: Arrangement伝播システム
1. ecs/arrangement.rsにsync_simple_arrangements, mark_dirty_arrangement_trees, propagate_global_arrangementsシステムを実装
2. world.rsのDrawスケジュールに登録（draw_rectangles/draw_labelsの後、render_surfaceの前）
3. 動作確認: 単純な階層（Window → Rectangle1個）でGlobalArrangement伝播をテスト

#### Phase 3: 階層的Surfaceレンダリング
1. graphics/systems.rsのrender_surfaceを拡張（Query::iter_descendants_depth_firstメソッドで深さ優先探索、`query.iter_descendants_depth_first(window_entity)`構文）
2. 各子孫描画前にID2D1DeviceContext::SetTransformでGlobalArrangement適用
3. 動作確認: Rectangle → Label1個で階層的描画をテスト

#### Phase 4: Rectangle/Label移行
1. Rectangle/Labelからx/yフィールド削除
2. Arrangementコンポーネント設定に移行（既存サンプルの移行）

#### Phase 5: サンプル更新
1. simple_window.rsに複雑なツリー構造追加（4階層、6 Rectangle + 2 Label）
2. ChildOf関係設定、Arrangement座標設定、色指定追加

**リスク軽減策**:
- Phase 1-2完了後、単純な階層（Window → Rectangle1個）で動作確認
- Phase 3で階層的描画を検証（Rectangle → Label1個）
- Phase 4前に既存サンプルをバックアップ（areka.rs等）
- 段階的コミット（各Phaseごと）

**Trade-offs**:
- ✅ 段階的実装で動作確認しながら進められる
- ✅ 既存パターン（tree_system.rs）を活用
- ✅ 新規モジュール（arrangement.rs）で責務分離
- ✅ bevy_ecs::hierarchy利用で実装工数削減
- ❌ Rectangle/Label修正は破壊的変更（サンプル更新必須）

---

## 4. Complexity & Risk Assessment

### Effort: **M (1 week)**

**根拠**:
- 新規コンポーネント: 5個（Offset, LayoutScale, Arrangement, GlobalArrangement, ArrangementTreeChanged）+ bevy_ecs::hierarchy利用（ChildOf, Children）
- 新規システム: 4個（sync_simple_arrangements, mark_dirty_arrangement_trees, propagate_global_arrangements, 階層的render_surface拡張）
- 既存修正: Rectangle/Label構造変更、render_surface拡張
- サンプル更新: simple_window.rs複雑化
- テスト: 階層構造、GlobalArrangement伝播、深さ優先描画の検証

**内訳見積もり**:
- Phase 1（Entity階層とArrangement基盤）: 1日（bevy_ecs::hierarchy利用で短縮）
- Phase 2（Arrangement伝播システム）: 1-2日（tree_system.rsパターン流用）
- Phase 3（階層的Surfaceレンダリング）: 1-2日（Query::iter_descendants_depth_firstメソッド利用で実装が簡潔）
- Phase 4（Rectangle/Label移行）: 1日
- Phase 5（サンプル更新）: 1日
- 統合テスト・デバッグ: 1日

### Risk: **Medium**

**根拠**:
- ✅ 低リスク要素:
  - DirectComposition APIラッパーは既に実装済み
  - tree_system.rsが参考実装として存在
  - bevy_ecsの変更検知パターンは既存システムで実証済み
- ⚠️ 中リスク要素:
  - 深さ優先Surfaceレンダリングの正確性（描画順序、座標変換スタック管理）
  - GlobalArrangementとTransformの合成タイミング（今回はTransform未使用だが設計考慮必要）
  - Rectangle/Label修正による既存サンプル（areka.rs等）への影響
  - パフォーマンス（階層深度増加時の描画負荷）

**リスク軽減策**:
- 段階的実装と動作確認（各Phase後にテスト）
- 単純なケース（2階層）から開始
- 既存サンプルのバックアップ
- パフォーマンス測定（実装後にR10検証）

---

## 5. Research Items for Design Phase

### 1. bevy_ecs::hierarchy統合パターン

**質問**: bevy_ecs::hierarchy::{ChildOf, Children}をwintfのtree_system.rsジェネリック関数でどう活用するか？

**✅ 解決済み**: `tests/transform_test.rs`に完全な使用例が存在、具体的な適用方法が明確

**tree_system.rsのジェネリック型パラメータ**:
```rust
pub fn sync_simple_transforms<L, G, M>(...)
pub fn mark_dirty_trees<L, G, M>(...)
pub fn propagate_parent_transforms<L, G, M>(...)
where
    L: Component + Copy + Into<G>,
    G: Component<Mutability = Mutable> + Copy + PartialEq + Mul<L, Output = G>,
    M: Component<Mutability = Mutable>,
```

**Arrangement向けの型パラメータ適用**:
- `L` = `Arrangement` (ローカル変換、Offset × LayoutScale)
- `G` = `GlobalArrangement` (グローバル変換、親からの累積)
- `M` = `ArrangementTreeChanged` (ダーティビットマーカー)
- `ChildOf`/`Children`はbevy_ecs::hierarchyから直接使用（既に関数内で参照されている）

**実際の使用例**: `tests/transform_test.rs`より
```rust
// tests/transform_test.rs:122
schedule.add_systems(
    sync_simple_transforms::<Transform, GlobalTransform, TransformTreeChanged>
);

// tests/transform_test.rs:270
fn create_test_schedule() -> Schedule {
    let mut schedule = Schedule::default();
    schedule.add_systems((
        mark_dirty_trees::<Transform, GlobalTransform, TransformTreeChanged>,
        sync_simple_transforms::<Transform, GlobalTransform, TransformTreeChanged>,
        propagate_parent_transforms::<Transform, GlobalTransform, TransformTreeChanged>,
    ));
    schedule
}
```

**arrangement.rsでの実装パターン**（Transform例を置き換え）:
```rust
use crate::ecs::tree_system::{sync_simple_transforms, mark_dirty_trees, propagate_parent_transforms};

pub fn sync_simple_arrangements(
    query: ParamSet<...>,
    orphaned: RemovedComponents<ChildOf>,
) {
    sync_simple_transforms::<Arrangement, GlobalArrangement, ArrangementTreeChanged>(query, orphaned);
}

pub fn mark_dirty_arrangement_trees(
    changed: Query<Entity, Or<(Changed<Arrangement>, Changed<ChildOf>, Added<GlobalArrangement>)>>,
    orphaned: RemovedComponents<ChildOf>,
    transforms: Query<(Option<&ChildOf>, &mut ArrangementTreeChanged)>,
) {
    mark_dirty_trees::<Arrangement, GlobalArrangement, ArrangementTreeChanged>(changed, orphaned, transforms);
}

pub fn propagate_global_arrangements(
    queue: Local<WorkQueue>,
    roots: Query<(Entity, Ref<Arrangement>, &mut GlobalArrangement, &Children), (Without<ChildOf>, Changed<ArrangementTreeChanged>)>,
    nodes: NodeQuery<Arrangement, GlobalArrangement, ArrangementTreeChanged>,
) {
    propagate_parent_transforms::<Arrangement, GlobalArrangement, ArrangementTreeChanged>(queue, roots, nodes);
}
```

**world.rsでのシステム登録パターン**（`world.rs:150-190`参照）:
```rust
// PostLayoutまたはDraw scheduleに登録
schedules.add_systems(PostLayout, (
    // 既存システム...
    sync_simple_arrangements,
    mark_dirty_arrangement_trees,
    propagate_global_arrangements,
));
```

**重要な発見**: tree_system.rsの関数は現在コードベースで未使用（ジェネリック関数として定義のみ）。`tests/transform_test.rs`が唯一の具体的な使用例であり、Arrangement実装が最初の本格的な適用となる。

**影響範囲**: arrangement.rsの実装、world.rsのシステム登録（Design Phaseで詳細化）

### 2. 深さ優先Surfaceレンダリングの実装方針

**質問**: Query::iter_descendants_depth_firstメソッドを使用するか、DescendantDepthFirstIterを直接構築するか？

**✅ 解決済み**: `Query::iter_descendants_depth_first::<Children>`を推奨、最も簡潔で読みやすい

**実装方針**:
- **推奨**: `Query::iter_descendants_depth_first::<Children>(entity)` - もっとも簡潔、推奨構文
  - `Children`が`RelationshipTarget`を実装しているため使用可能
  - 型推論により`query.iter_descendants_depth_first(root)`とも書ける
  - bevy_ecs 0.17.2公式ドキュメント: <https://docs.rs/bevy_ecs/0.17.2/bevy_ecs/system/struct.Query.html#method.iter_descendants_depth_first>
- **代替案**: `DescendantDepthFirstIter::new(&query, entity)` - より明示的だが冗長
- **非推奨**: 再帰ラッパー - 不要な抽象化

**render_surfaceでの使用例**（擬似コード）:
```rust
pub fn render_surface(
    windows: Query<(Entity, &VisualSurface), With<Window>>,
    widgets: Query<(&GlobalArrangement, &GraphicsCommandList)>,
) {
    for (window_entity, surface) in windows.iter() {
        surface.begin_draw();
        
        // Window自身を描画
        if let Ok((global_arr, cmd_list)) = widgets.get(window_entity) {
            surface.set_transform(global_arr);
            surface.draw(cmd_list);
        }
        
        // 全子孫を深さ優先で描画
        for descendant in widgets.iter_descendants_depth_first::<Children>(window_entity) {
            if let Ok((global_arr, cmd_list)) = widgets.get(descendant) {
                surface.set_transform(global_arr);
                surface.draw(cmd_list);
            }
        }
        
        surface.end_draw();
        surface.commit();
    }
}
```

**ポイント**:
- Entity階層（Children）を辿って、各子孫のGraphicsCommandListをWindowのSurfaceに描画
- GlobalArrangementをSetTransformで適用、描画後にIdentityでリセット不要（次の描画でoverrideされる）
- 深さ優先順序により正しい描画順が保証される（親→子→孫...）

**参考資料**:
- Query::iter_descendants_depth_firstメソッド: <https://docs.rs/bevy_ecs/0.17.2/bevy_ecs/system/struct.Query.html#method.iter_descendants_depth_first>
- RelationshipTargetトレイト: <https://docs.rs/bevy_ecs/0.17.2/bevy_ecs/relationship/trait.RelationshipTarget.html>
- DescendantDepthFirstIter: <https://docs.rs/bevy_ecs/0.17.2/bevy_ecs/relationship/struct.DescendantDepthFirstIter.html>

**影響範囲**: graphics/systems.rs render_surfaceの実装

### 3. GlobalArrangement × Transform合成設計

**質問**: 描画時に `final_transform = GlobalArrangement * Transform` をどこで計算するか？

**決定事項**: **Option A: render_surface内で計算してSetTransform**
- キャッシュ不要、描画時に毎回計算で十分
- 今回はTransform未使用だが、将来のTransform統合時も同じパターン適用可能
- 実装: 各子孫描画前に`SetTransform(GlobalArrangement)`、描画後に`SetTransform(Identity)`でリセット

**理由**:
- 描画頻度（60fps）でも計算コストは無視できる（行列乗算のみ）
- Changed検知とキャッシュ管理の複雑性を回避
- 描画スレッドで完結、システム間の依存関係が単純

**影響範囲**: render_surfaceの実装のみ（arrangement.rsへの影響なし）

### 4. Rectangle/Label修正の影響範囲

**質問**: x/y削除によりareka.rs等の既存サンプルはどこまで影響を受けるか？

**✅ 解決済み**: 既存サンプルへの影響なし

**分析結果**:
- **areka.rs**: 内部が未実装のため、Rectangle/Label使用なし → 影響なし
- **dcomp_demo.rs**: DirectComposition実装確認専用、ECS未使用 → 影響なし
- **simple_window.rs等**: 新規Arrangement導入時に更新（Phase 5で対応）

**確認方法**: `cargo build --all-targets` でビルド確認（全ターゲット含む）

**影響範囲**: サンプル更新工数、移行ドキュメント作成要否

---

## 6. Recommendations for Design Phase

### 推奨アプローチ: **Option C (Hybrid)**

**理由**:
- 既存の堅牢なパターン（tree_system.rs、DirectComposition APIラッパー）を最大限活用
- 新規モジュール（visual_tree.rs, arrangement.rs）で責務を明確に分離、bevy_ecs::hierarchy利用
- 段階的実装により、各Phaseで動作確認しながら進められる
- Rectangle/Label修正は破壊的変更だが、Arrangementへの移行は設計的に正しい

### 重要な設計決定事項

1. **ChildOf/Children実装**: bevy_ecs 0.17.2標準の`bevy_ecs::hierarchy::{ChildOf, Children}`を採用、tree_system.rsジェネリック関数パターンで利用

2. **深さ優先レンダリング**: tree_system.rsのpropagate_descendants_uncheckedパターンを参考に、再帰方式で実装（サンプルは4階層程度でスタックオーバーフローリスクなし）

3. **GlobalArrangement伝播**: tree_system.rsの `propagate_parent_transforms` パターンを適用（sync_simple_arrangements, mark_dirty_arrangement_trees, propagate_global_arrangementsの3システム）

4. **render_surface拡張**: 
   - `Query::iter_descendants_depth_first::<Children>(window_entity)`で子孫を深さ優先探索
   - `Children`は`RelationshipTarget`を実装しているため、このメソッドが使用可能
   - 各子孫描画前にSetTransform(GlobalArrangement)
   - 描画後にSetTransform(Identity)でリセット
   - WindowのSurfaceに全Widget(子孫)のGraphicsCommandListを統合描画

5. **スケジュール順序**:
```
PostLayout:
  - (既存システム)
  - init_window_visual (既存)
  - init_window_surface (既存)

Draw:
  - cleanup_graphics_needs_init
  - draw_rectangles
  - draw_labels
  - sync_simple_arrangements (新規)
  - mark_dirty_arrangement_trees (新規)
  - propagate_global_arrangements (新規)

RenderSurface:
  - render_surface (拡張、Children深さ優先探索による階層的描画)

CommitComposition:
  - commit_composition
```

### Next Steps

1. ✅ Research Items 1-4すべて解決済み
   - Item 1: tree_system.rs統合パターン確認（`tests/transform_test.rs`に実例）
   - Item 2: 深さ優先探索実装方針決定（`Query::iter_descendants_depth_first::<Children>`）
   - Item 3: Transform計算戦略決定（render_surface内で計算、キャッシュなし）
   - Item 4: 既存サンプル影響範囲確認（areka.rs未実装、dcomp_demo.rsはECS未使用）
2. ✅ ビルド確認完了（`cargo build --all-targets` 成功）
3. ⏭️ Design Phase開始準備完了
4. **次のコマンド**: `/kiro-spec-design visual-tree-implementation` でdesign.md生成

---

_Gap analysis completed on 2025-11-17_
_All research items resolved, ready for design phase_
