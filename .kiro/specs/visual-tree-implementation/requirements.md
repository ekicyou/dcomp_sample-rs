# Requirements: visual-tree-implementation

**Feature ID**: `visual-tree-implementation`  
**Created**: 2025-11-17  
**Approved**: 2025-11-17  
**Status**: Requirements Approved

---

## Introduction

本要件定義は、ECSのEntity階層（ChildOf/Children）を用いたビジュアルツリー構造の実装を定義する。現在のwintfフレームワークは、基本的なグラフィックスリソース（WindowGraphics、Visual、Surface）を持つが、Entity階層と座標変換システム（Arrangement）は未実装である。

**今回のスコープ**: Entity階層（ChildOf/Children）の導入、Arrangement座標変換システム、階層的Surfaceレンダリング（WindowのSurfaceに全子孫を深さ優先描画）。

**将来のスコープ**: IDCompositionVisualの階層構造（親子関係）を構築し、DirectCompositionの階層的合成機能を活用した効率的な描画とトランスフォーム、部分更新の実現。

### 現状分析

**既存実装**:
- ✅ GraphicsCore: DirectComposition Device統合済み
- ✅ WindowGraphics: Window単位のIDCompositionTarget保持
- ✅ Visual: IDCompositionVisual3のラッパーコンポーネント（単体のみ、階層なし）
- ✅ Surface: IDCompositionSurfaceの管理
- ✅ Rectangle、Label: GraphicsCommandListでフラット描画

**今回実装**:
- ✅ Entity階層（ChildOf/Children）
- ✅ Arrangement座標変換システム（Offset, LayoutScale, GlobalArrangement伝播）
- ✅ 階層的Surfaceレンダリング（WindowのSurfaceに全子孫を深さ優先描画）

**将来実装（今回スコープ外）**:
- ❌ IDCompositionVisualの親子関係（AddVisual）
- ❌ Widget個別のVisual+Surface作成
- ❌ Visual階層に基づく部分更新

### 設計原則

1. **Entity階層の構築**: bevy_ecs::hierarchy::{ChildOf, Children}を使用してEntity階層を管理
2. **Visual/Surfaceの所有ルール**:
   - **今回のスコープ**: ルート(Window)のみがVisual + Surfaceを所有。全Widget描画はWindowのSurfaceに集約
   - **将来の拡張**: 子WidgetもVisual + Surfaceを作成し、IDCompositionVisual階層を構築
     - アニメーションが存在する
     - スクロールが存在する
     - その他、階層でSurfaceを生成する理由がある
3. **変更検知による自動更新**: Changed<Arrangement>で座標変換を自動更新
4. **型安全性**: unsafeコードはCOMラッパー層に隔離

---

## Requirements

### Requirement 1: Window用IDCompositionVisualとSurfaceの作成

**Objective:** システム開発者として、Window Entity用のIDCompositionVisual3とSurfaceを作成したい。これにより、DirectComposition描画の基盤が確立される。

**今回のスコープ:** Window EntityのみがVisual+Surfaceを持つ。Rectangle/Label等の子Widget EntityはVisual/Surfaceを持たない（将来実装）。

#### Acceptance Criteria

1. When Window EntityにWindowGraphicsコンポーネントが追加される時、wintfシステムはIDCompositionVisual3インスタンスとIDCompositionSurfaceを作成しなければならない
2. The wintfシステムは、WindowエンティティのVisualをルートVisualとして識別しなければならない
3. When Visualコンポーネントが作成された時、wintfシステムはCOM参照カウントを適切に管理しなければならない
4. When Window Visualが作成された時、wintfシステムはIDCompositionTarget::SetRootを呼び出してルートVisualとして設定しなければならない

---

### Requirement 2: Entity階層構築（ChildOf/Children）

**Objective:** システム開発者として、Entity階層（ChildOf/Children）を構築したい。これにより、Widget間の親子関係が管理され、座標変換の伝播と階層的描画が可能になる。

**今回のスコープ:** bevy_ecs::hierarchy::{ChildOf, Children}を使用してEntity階層を管理。DirectComposition Visual階層構築（AddVisual）は将来実装。

#### Acceptance Criteria (R2)

1. The wintfシステムは、bevy_ecs::hierarchy::{ChildOf, Children}をインポートして使用しなければならない
2. When アプリケーション開発者がWidget Entityを作成する時、ChildOfコンポーネントを設定して親Entityを指定できなければならない
3. The wintfシステムは、ChildOfコンポーネントの変更を検知してChildrenコンポーネントを自動更新しなければならない（bevy_ecs::hierarchyの標準機能）
4. When Window EntityにChildrenコンポーネントが追加された時、wintfシステムは子Widget Entityのリストを保持しなければならない
5. The wintfシステムは、Entity階層を使用してGlobalArrangement伝播と階層的Surfaceレンダリングを実装しなければならない

---

### Requirement 3: Window Visual/Surfaceのライフサイクル管理

**Objective:** システム開発者として、Window VisualとSurfaceの作成と破棄を適切に管理したい。これにより、リソースリークを防止できる。

**今回のスコープ:** Window EntityのみがVisual+Surfaceを持つため、Windowのライフサイクル管理のみ実装。

#### Acceptance Criteria (R3)

1. When WindowGraphicsコンポーネントが作成された時、wintfシステムは対応するWindow EntityにVisualコンポーネントとSurfaceコンポーネントを追加しなければならない（既存実装: init_window_visual, init_window_surface）
2. When Visualコンポーネントが削除された時、wintfシステムはIDCompositionVisualのCOM参照を解放しなければならない
3. When Window Entityがdespawnされる時、wintfシステムはbevy_ecsのon_removeフックでVisualとSurfaceリソースをクリーンアップしなければならない
4. The wintfシステムは、COM参照カウント管理により、Visualの二重解放を防止しなければならない

---

### Requirement 4: レイアウト座標変換の実装（Arrangementコンポーネント）

**Objective:** システム開発者として、レイアウト計算結果をDirectCompositionのVisual座標（SetOffsetX/Y）に反映したい。これにより、Widget階層の座標変換が実現される。

**設計方針:**
- **layout.rsに追加**: Offset、LayoutScale、Arrangement、GlobalArrangement、ArrangementTreeChangedコンポーネント
- **座標変換フロー**: 
  ```
  Arrangement (ローカルレイアウト位置+スケール)
    ↓
  GlobalArrangement (親から伝播した累積レイアウト変換)
    ↓
  Transform (ローカル視覚効果変換: 回転、傾斜等)
    ↓
  最終変換 = GlobalArrangement * Transform
  ```
- **Arrangementの構成**: Offset（親からの相対位置）+ LayoutScale（DPIスケール、ViewBox等）
- **Transformとの区別**: 
  - Arrangement（layout.rs）: レイアウト層の座標変換、親から子へ累積伝播
  - Transform（transform.rs）: 視覚効果層の変換（WPF RenderTransform相当）、累積伝播なし

**重要な設計原則:**
- **taffyレイアウトエンジンは今回のスコープ外**: BoxComputedLayout統合は将来実装
- **Arrangementは直接設定**: Rectangle/Label等のEntityにArrangementコンポーネントを直接設定（決め打ち座標）
- **Rectangle/Labelのx/yフィールド廃止**: 座標はArrangementコンポーネントで管理
- **GlobalArrangementの伝播**: bevy_ecsのChildOf/Childrenを使用して親から子へ累積変換を伝播

**今回のスコープ:** Window EntityはArrangement { offset: (0,0), scale: (1,1) }固定。子Widget EntityではArrangementコンポーネントを直接設定（決め打ち座標）。

#### Acceptance Criteria (R4)

1. The wintfシステムは、layout.rsにOffset構造体（x: f32, y: f32）とLayoutScale構造体（x: f32, y: f32）を定義しなければならない
2. The wintfシステムは、layout.rsにArrangementコンポーネント（offset: Offset, scale: LayoutScale）を定義しなければならない
3. The wintfシステムは、layout.rsにGlobalArrangementコンポーネント（Matrix3x2）を定義しなければならない
4. The wintfシステムは、layout.rsにArrangementTreeChangedマーカーコンポーネントを定義しなければならない
5. When Window Visualが作成される時、wintfシステムはArrangement { offset: (0.0, 0.0), scale: (1.0, 1.0) }を設定しなければならない
6. When Rectangle/Label Entityが作成される時、アプリケーション開発者はArrangementコンポーネントを直接設定しなければならない（例: Arrangement { offset: (10.0, 20.0), scale: (1.0, 1.0) }）
7. The wintfシステムは、Rectangle構造体とLabel構造体からx/yフィールドを削除しなければならない
8. The wintfシステムは、propagate_global_arrangementシステムでChildOf/Childrenを使用して親のGlobalArrangementを子に伝播しなければならない
9. When Changed<Arrangement>またはChanged<GlobalArrangement>である時、wintfシステムはArrangementTreeChangedマーカーを追加しなければならない
10. The wintfシステムは、ArrangementからMatrix3x2への変換（offset.x/y、scale.x/y → 行列）を実装しなければならない
11. The wintfシステムは、将来の描画時に `final_transform = GlobalArrangement * Transform` で最終座標変換を計算する設計でなければならない

**補足:** 
- transform.rsのGlobalTransformとTransformTreeChangedは誤った設計であり、今後削除予定（今回のスコープ外）
- Transformは視覚効果のみ（回転、傾斜等）を担当し、階層的な累積伝播を行わない
- **Rectangle/Labelのx/yフィールドは廃止**: 座標管理はArrangementコンポーネントに一本化
- 将来、taffyレイアウトエンジン統合時にBoxComputedLayout → Arrangementの変換を追加

**参考実装パターン（tree_system.rs）:**

tree_system.rsには、GlobalTransform伝播の実装パターンが存在する。これをArrangementTreeChanged伝播に適用する:

1. **sync_simple_transforms**: 階層に属していないEntity（ルートWindow）のGlobalArrangementを更新
   - `Changed<Arrangement>`, `Added<GlobalArrangement>`, `Without<ChildOf>`, `Without<Children>`でクエリ
   - 孤立したEntity（`RemovedComponents<ChildOf>`）の処理

2. **mark_dirty_trees**: 「ダーティビット」を階層の祖先に向かって伝播
   - `Changed<Arrangement>`, `Changed<ChildOf>`, `Added<GlobalArrangement>`でEntityを検出
   - ChildOfを辿って親方向にArrangementTreeChangedマーカーを伝播
   - `tree.is_changed() && !tree.is_added()`で既に処理済みの判定

3. **propagate_parent_transforms**: 親から子へGlobalArrangementを伝播
   - ルートEntity（`Without<ChildOf>`, `Changed<ArrangementTreeChanged>`）から開始
   - 並列処理: `par_iter_mut()`でルートを並列処理
   - 深さ優先探索: `propagate_descendants_unchecked`で子孫に再帰的に伝播
   - 変換計算: `global_arrangement.set_if_neq(parent_global * arrangement)`
   - 静的シーン最適化: `!tree.is_changed() && !parent_global.is_changed()`でスキップ

4. **システム実行順序**:
   ```
   sync_simple_transforms  // ルートのGlobalArrangementを更新
   mark_dirty_trees        // ダーティビットを伝播
   propagate_parent_transforms  // 子孫にGlobalArrangementを伝播
   ```

この実装パターンをArrangement伝播に適用することで、高パフォーマンスな階層的座標変換が実現される。

---

### Requirement 5: ルートVisual管理

**Objective:** システム開発者として、Window Entityをビジュアルツリーのルートとして管理したい。これにより、DirectComposition描画の起点が確立される。

**現時点の範囲:** 単一Window対応。将来、複数Window対応を追加。

#### Acceptance Criteria (R5)

1. When WindowGraphicsコンポーネントが初期化される時、wintfシステムは対応するWindow EntityにVisualコンポーネントとSurfaceコンポーネントを追加しなければならない
2. When Window EntityのVisualが作成された時、wintfシステムはIDCompositionTarget::SetRootを呼び出してルートVisualとして設定しなければならない
3. The wintfシステムは、init_window_visualシステムでWindow Entity(WindowGraphicsコンポーネントを持つ)を検索してルートVisualを作成しなければならない
4. The wintfシステムは、Window EntityがChildOfコンポーネントを持たない(ルートEntity)ことを前提としなければならない
5. When 将来、複数Windowが実装される時、各Window単位で独立したビジュアルツリーを管理する設計でなければならない

---

### Requirement 6: 階層的Surfaceレンダリング

**Objective:** システム開発者として、親Surfaceに自分と子孫のコマンドリストを深さ優先で描画したい。これにより、ビジュアルツリー階層が単一Surfaceに統合される。

**設計方針:**
- **現在の実装**: 各EntityのSurfaceに自分のコマンドリストのみ描画
- **今回の変更**: 親EntityのSurfaceに、自分と全子孫のコマンドリストを深さ優先で描画
- **描画順序**: 「親」→「第1子」→「第1子の子孫（深さ優先）」→「第2子」→...
- **座標変換**: 各子孫描画時に、そのGlobalArrangementを基準変換として適用
- **Visual構成**: Window EntityのみVisual+Surfaceを持ち、子Widget EntityはVisual作成するがSurfaceは親のものを使用

**今回のスコープ:** Window Surfaceへの階層的描画実装。子Widget独自Surfaceは将来実装（アニメーション/スクロール時）。

#### Acceptance Criteria (R6)

1. The wintfシステムは、render_surfaceシステムで親Entity（Window）のSurfaceに対して、自分と全子孫のコマンドリストを描画しなければならない
2. The wintfシステムは、ChildOf/Childrenを使用して深さ優先探索でコマンドリストを収集しなければならない
3. When 子孫Entityのコマンドリストを描画する時、wintfシステムはそのEntityのGlobalArrangementをID2D1DeviceContext::SetTransformで設定しなければならない
4. The wintfシステムは、描画順序を「親」→「第1子」→「第1子の全子孫（深さ優先）」→「第2子」→...で実行しなければならない
5. When Window EntityにSurfaceコンポーネントとVisualコンポーネントが存在する時、wintfシステムはIDCompositionVisual::SetContentでSurfaceを設定しなければならない
6. The wintfシステムは、render_surfaceシステムでGraphicsCommandListをSurfaceにコミットした後、SetContentを呼び出さなければならない
7. When 将来、子Widget独自Surfaceが実装される時、アニメーション/スクロール等の条件で子Widget EntityにもSurfaceを作成し、階層描画をスキップする設計でなければならない

---

### Requirement 7: 変更検知と効率的更新

**Objective:** システム開発者として、変更があったVisualのみを更新したい。これにより、不要なDirectComposition API呼び出しを削減してパフォーマンスが向上する。

**現時点の範囲:** Window Visual作成/削除の検知のみ。将来、階層変更やオフセット変更の検知を追加。

#### Acceptance Criteria (R7)

1. The wintfシステムは、Added<Visual>フィルターで新規作成されたWindow Visualのみをクエリしなければならない
2. The wintfシステムは、Removed<Visual>フックで削除されたWindow Visualのクリーンアップを実行しなければならない
3. When Window Surfaceが再作成された時のみ、wintfシステムはSetContentを呼び出さなければならない
4. The wintfシステムは、CommitCompositionスケジュールでIDCompositionDevice::Commitを1回呼び出して全変更をコミットしなければならない
5. When 将来、子Widget Visualが実装される時、Changed<ChildOf>による階層変更検知を追加する設計でなければならない

---

### Requirement 8: エラーハンドリング

**Objective:** システム開発者として、Window Visual作成時のエラーを適切に処理したい。これにより、部分的なエラーでもシステム全体が停止しない堅牢性が実現される。

**現時点の範囲:** Window Visual作成のエラーハンドリング。将来、階層構築のエラーハンドリングを追加。

#### Acceptance Criteria (R8)

1. When IDCompositionVisual3作成に失敗した時、wintfシステムはeprintln!でエラーログを出力し、該当Window Entityをスキップしなければならない
2. When IDCompositionTarget::SetRoot呼び出しに失敗した時、wintfシステムはエラーログを出力しなければならない
3. The wintfシステムは、windows::core::Resultのエラーをログ出力しなければならない
4. When Window Visual作成エラーが発生した時、wintfシステムは他のWindowの処理を継続しなければならない
5. When 将来、子Widget Visual階層構築が実装される時、親Entityが存在しない場合の警告ログを追加する設計でなければならない

---

### Requirement 9: サンプルアプリケーション

**Objective:** アプリケーション開発者として、Entity階層に基づくビジュアルツリーの使用例を参照したい。これにより、階層構造の実装方法が理解できる。

**今回のスコープ:** 既存のsimple_window.rsを更新し、Rectangle/Labelを使った親子階層のデモを実装。専用Container Widgetは今回のスコープ外。

**サンプル構成例:**
```
Window Entity (ルート、Surface所有)
  ├─ Rectangle1 Entity (青背景、200x150、Arrangement { offset: (20, 20), scale: (1, 1) })
  │    ├─ Rectangle1-1 Entity (緑背景、80x60、Arrangement { offset: (10, 10), scale: (1, 1) })
  │    │    └─ Label1 Entity (赤文字「Hello」、Arrangement { offset: (5, 5), scale: (1, 1) })
  │    └─ Rectangle1-2 Entity (黄背景、80x60、Arrangement { offset: (10, 80), scale: (1, 1) })
  │         └─ Rectangle1-2-1 Entity (紫背景、60x40、Arrangement { offset: (10, 10), scale: (1, 1) })
  │              └─ Label2 Entity (白文字「World」、Arrangement { offset: (5, 5), scale: (1, 1) })
```

**色指定:**
- Rectangle1: 青 (0x0000FF)
- Rectangle1-1: 緑 (0x00FF00)
- Rectangle1-2: 黄 (0xFFFF00)
- Rectangle1-2-1: 紫 (0xFF00FF)
- Label1: 赤文字 (0xFF0000)、テキスト「Hello」
- Label2: 白文字 (0xFFFFFF)、テキスト「World」

**検証ポイント:**
- 深さ3階層以上のEntity階層（最大4階層）
- LabelがRectangle上に完全に重なる配置（Labelの方が小さい）
- 複数の兄弟Entity（Rectangle1-1とRectangle1-2）による描画順序検証
- GlobalArrangementの累積計算（例: Label2の最終座標 = 20+10+10+5 = 45）
- 各Rectangle/Labelが異なる色で描画され、視覚的に階層構造を確認できる

**重要:** Rectangle/LabelのEntityにx/yフィールドは存在しない。座標はArrangementコンポーネントで直接設定する。

#### Acceptance Criteria (R9)

1. The wintfシステムは、simple_window.rsサンプルを更新してEntity階層構造を追加しなければならない
2. The サンプルは、6個のRectangle Entityと2個のLabel Entityを作成し、最大4階層のツリー構造を構築しなければならない
3. The サンプルは、各EntityにChildOfコンポーネントで親を設定し、Rectangle1にはRectangle1-1とRectangle1-2の2つの子を持たせなければならない
4. The サンプルは、全てのRectangle/Label EntityにArrangementコンポーネントを直接設定して親からの相対座標を指定しなければならない
5. The サンプルは、LabelがRectangle上に完全に重なる配置（Labelの方が小さい）を実装しなければならない
6. The サンプルは、Rectangle構造体とLabel構造体にx/yフィールドが存在しないことを確認できなければならない
7. When サンプルが実行される時、wintfシステムはWindow Surfaceに複数のRectangleとLabelが階層的に描画されることを表示しなければならない
8. The サンプルは、深さ優先探索の描画順序（Rectangle1 → Rectangle1-1 → Label1 → Rectangle1-2 → Rectangle1-2-1 → Label2）を確認できなければならない
9. The サンプルは、Window EntityにVisualコンポーネントが自動作成され、ルートVisualとして設定されることを確認できなければならない
10. The サンプルは、cargo run --example simple_windowコマンドで実行可能でなければならない

---

### Requirement 10: パフォーマンス要件

**Objective:** エンドユーザーとして、滑らかなUI表示を期待する。これにより、快適なユーザー体験が提供される。

**現時点の範囲:** Window Visual作成のパフォーマンス。将来、子Widget Visual階層構築のパフォーマンス要件を追加。

#### Acceptance Criteria (R10)

1. The wintfシステムは、Window Surfaceへの50個のRectangle/Label描画で60fps以上を維持しなければならない
2. When Window Surfaceに変更がないフレームでは、wintfシステムはCommit以外のDirectComposition APIを呼び出してはならない
3. The wintfシステムは、Window Visual作成とSetRootを1フレームあたり1ms以内で完了しなければならない
4. The wintfシステムは、COM参照カウント管理によりメモリリークを発生させてはならない
5. When 将来、子Widget Visualが実装される時、階層構築を1フレームあたり5ms以内で完了する設計でなければならない

---

_Requirements generated on 2025-11-17_
