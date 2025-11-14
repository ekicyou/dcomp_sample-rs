# Gap Analysis: Phase 2 Milestone 2 - WindowGraphics + Visual作成

**Feature ID**: `phase2-m2-window-graphics`  
**Analysis Date**: 2025-11-14  
**Analysis Framework**: `.kiro/settings/rules/gap-analysis.md`

---

## Executive Summary

本分析では、Phase 2 Milestone 2の要件と現在のコードベースを比較し、実装ギャップを特定しました。

### 主要な発見事項

- ✅ **COM APIラッパーは完備**: `create_target_for_hwnd`, `create_visual`, `set_root`のAPIラッパーが既に実装済み
- ✅ **ECSシステムパターンが確立**: `create_windows`システムが参照可能なパターンを提供
- ⚠️ **新規コンポーネント作成が必要**: WindowGraphicsとVisualコンポーネントは未実装
- ⚠️ **D2D DeviceContext作成手段**: 既存のD2D1DeviceExtにcreate_device_contextが存在するが、ウィンドウ単位での管理方法を設計する必要あり
- ✅ **システム登録パターンが明確**: `EcsWorld::add_systems`と`schedules.add_systems`のパターンが既存

### 推奨アプローチ

**Option A: 既存graphics.rsを拡張** - GraphicsCoreと同じファイルに追加（推奨）

---

## 1. Current State Investigation

### 1.1 既存の関連資産

#### ECS Component Layer (`crates/wintf/src/ecs/`)

**graphics.rs** (101行):
- `GraphicsCore` Resource: グローバルグラフィックスリソース（D3D, D2D, DWrite, DComp）
- `ensure_graphics_core()` システム: GraphicsCore未作成時に初期化
- パターン: Resource + 初期化システムの組み合わせ

**window.rs** (423行):
- `Window` Component: ウィンドウ作成パラメータ
- `WindowHandle` Component: 作成済みhwnd保持（SparseSet storage, lifecycle hooks付き）
- `DpiTransform` Component: DPI変換行列
- パターン: Component + lifecycle hooks（on_add/on_remove）

**window_system.rs** (約100行):
- `create_windows()` システム: `Query<(Entity, &Window, ...), Without<WindowHandle>>`でウィンドウ作成
- パターン: `Without<T>`フィルタで未作成エンティティを検出し、`commands.entity(entity).insert(T)`でコンポーネント追加

**world.rs** (224行):
- スケジュール定義: Input, Update, PreLayout, Layout, PostLayout, UISetup, Draw, RenderSurface, Composition
- `EcsWorld::add_systems()`: システム登録用API
- デフォルトシステム登録: `ensure_graphics_core`と`create_windows`をUISetupステージに登録

#### COM Wrapper Layer (`crates/wintf/src/com/`)

**dcomp.rs** (289行):
- `DCompositionDesktopDeviceExt::create_target_for_hwnd()`: 実装済み ✅
- `DCompositionDeviceExt::create_visual()`: 実装済み ✅
- `DCompositionTargetExt::set_root()`: 実装済み ✅
- `DCompositionSurfaceExt::begin_draw()`: ID2D1DeviceContext3を返す

**d2d/mod.rs** (162行):
- `D2D1DeviceExt::create_device_context()`: ID2D1DeviceContextを作成 ✅
- `D2D1DeviceContextExt`: 描画APIラッパー（clear, draw_text, draw_bitmap等）

### 1.2 既存のコーディング規約

#### コンポーネント定義パターン
```rust
#[derive(Component, Debug, ...)]
#[component(storage = "SparseSet")] // オプション
pub struct MyComponent {
    pub field: Type,
}

unsafe impl Send for MyComponent {}
unsafe impl Sync for MyComponent {}
```

#### システム定義パターン
```rust
pub fn my_system(
    query: Query<(&A, &B), Without<C>>,
    mut commands: Commands,
) {
    for (entity, a, b) in query.iter() {
        // 処理
        commands.entity(entity).insert(C { ... });
    }
}
```

#### エラーハンドリング
- `windows::core::Result`を使用
- エラー時は`eprintln!`でログ出力し、そのエンティティの処理をスキップ
- パニックは避ける（アプリケーション全体をクラッシュさせない）

#### ログ出力
- `eprintln!`マクロを使用
- フォーマット: `[Component/System名] メッセージ`
- 例: `eprintln!("[GraphicsCore] 初期化開始");`

### 1.3 統合ポイント

- **GraphicsCore依存**: `Res<GraphicsCore>`でdeskopとdcompデバイスにアクセス
- **WindowHandle依存**: `&WindowHandle`でhwndを取得
- **システム登録**: `world.add_systems(UISetup, system)`でUISetupステージに追加
- **実行順序制御**: `.before()`/`.after()`チェーンで依存関係を明示

---

## 2. Requirements Feasibility Analysis

### 2.1 要件から抽出される技術的ニーズ

#### データモデル

1. **WindowGraphics コンポーネント**
   - `IDCompositionTarget`: hwndに紐付くCompositionターゲット
   - `ID2D1DeviceContext`: ウィンドウ単位の描画コンテキスト
   - 状態: 未実装 ❌

2. **Visual コンポーネント**
   - `IDCompositionVisual3`: Visualツリーのルートノード
   - 状態: 未実装 ❌

#### システム/サービス

1. **create_window_graphics システム**
   - 検出クエリ: `Query<(Entity, &WindowHandle), Without<WindowGraphics>>`
   - 処理: IDCompositionTarget作成 + ID2D1DeviceContext作成
   - 状態: 未実装 ❌

2. **create_window_visual システム**
   - 検出クエリ: `Query<(Entity, &WindowHandle, &WindowGraphics), Without<Visual>>`
   - 処理: IDCompositionVisual3作成 + set_root()実行
   - 状態: 未実装 ❌

#### API/統合

- ✅ `GraphicsCore::desktop.create_target_for_hwnd()` - 利用可能
- ✅ `GraphicsCore::dcomp.create_visual()` - 利用可能
- ✅ `IDCompositionTarget::set_root()` - 利用可能
- ⚠️ `ID2D1Device::create_device_context()` - 利用可能だが、ウィンドウ単位での管理設計が必要

### 2.2 ギャップと制約

#### Missing（未実装）

1. **WindowGraphicsコンポーネント**
   - 新規作成が必要
   - COMオブジェクトのライフタイム管理（Drop実装）が必要
   - Send + Sync実装が必要

2. **Visualコンポーネント**
   - 新規作成が必要
   - COMオブジェクトのライフタイム管理（Drop実装）が必要
   - Send + Sync実装が必要

3. **create_window_graphicsシステム**
   - 新規作成が必要
   - `create_windows`システムのパターンを踏襲可能

4. **create_window_visualシステム**
   - 新規作成が必要
   - WindowGraphics依存を明示的に表現

5. **システム登録**
   - world.rsまたはアプリケーション側でadd_systems呼び出しが必要

#### Constraints（制約）

1. **ID2D1DeviceContext管理**
   - GraphicsCoreが保持する単一のID2D1DeviceからDeviceContextを作成
   - ウィンドウごとに独立したDeviceContextが必要
   - 既存のGraphicsCore設計では単一デバイスのみ保持
   - **Research Needed**: DeviceContextをウィンドウ単位で作成・管理する最適な方法（毎回作成 vs キャッシュ）

2. **システム実行順序**
   - ensure_graphics_core → create_window_graphics → create_window_visual の順序保証が必要
   - 既存の`.before()`/`.after()`パターンで対応可能

3. **COMオブジェクトライフタイム**
   - windows-rsのスマートポインタを使用（既存パターン）
   - Dropトレイトで自動解放（明示的なDrop実装は不要の可能性）

#### Unknown（調査が必要）

1. **ID2D1DeviceContext作成タイミング**
   - 毎フレーム作成 vs 初回作成してキャッシュ
   - DirectCompositionのベストプラクティスを確認する必要あり

2. **CompositionTarget複数ウィンドウでの動作**
   - 複数ウィンドウでの並行動作の検証
   - 既存のdcomp_demo.rsで単一ウィンドウのみ使用している

### 2.3 複雑性シグナル

- **複雑度**: 中程度
  - 新規コンポーネント2つ + 新規システム2つ
  - COMオブジェクトのライフタイム管理は既存パターン踏襲で対応可能
  - ECSパターンは既存システムが参考になる
  
- **統合複雑度**: 低
  - 既存のGraphicsCoreとWindowHandleに依存するのみ
  - COM APIラッパーは完備済み
  - システム登録も既存パターンで対応可能

---

## 3. Implementation Approach Options

### Option A: 既存graphics.rsを拡張【推奨】

#### 変更対象ファイル
- `crates/wintf/src/ecs/graphics.rs` (101行 → 約300-350行)

#### 追加内容
1. WindowGraphicsコンポーネント定義（約40行）
2. Visualコンポーネント定義（約30行）
3. create_window_graphicsシステム（約50行）
4. create_window_visualシステム（約40行）

#### 互換性評価
- ✅ 既存のGraphicsCoreに影響なし（Resourceとして独立）
- ✅ 既存のensure_graphics_coreシステムに影響なし
- ✅ 新規コンポーネントは既存コンポーネントから完全に独立

#### 複雑性と保守性
- ✅ グラフィックス関連が1ファイルに集約され、責務が明確
- ✅ GraphicsCoreとWindowGraphics/Visualの関係が同一ファイルで理解しやすい
- ⚠️ ファイルサイズが350行程度になるが、Rust標準では許容範囲
- ✅ モジュール分割の必要性は低い（コンポーネント定義 + システム定義のシンプルな構造）

#### Trade-offs
- ✅ 最小限のファイル変更で実装可能
- ✅ グラフィックス関連の責務が明確に1箇所に集約
- ✅ 既存パターン（GraphicsCore + ensure_graphics_core）と一貫性がある
- ✅ インポート構造がシンプル（`use crate::ecs::graphics::*`で全てアクセス可能）
- ⚠️ ファイルサイズがやや大きくなるが、機能的な凝集度は高い
- ✅ 将来的なリファクタリング（モジュール分割）も容易

### Option B: 新規window_graphics.rsを作成

#### 新規ファイル
- `crates/wintf/src/ecs/window_graphics.rs` (約200行)

#### 内容
- WindowGraphicsコンポーネント + Visualコンポーネント
- create_window_graphicsシステム + create_window_visualシステム

#### 統合ポイント
- `ecs/mod.rs`にpub mod window_graphics追加
- GraphicsCoreへの依存は`use crate::ecs::graphics::GraphicsCore`
- WindowHandleへの依存は`use crate::ecs::window::WindowHandle`

#### 責務境界
- **新規ファイル**: ウィンドウ単位のグラフィックスリソース管理
- **既存graphics.rs**: グローバルグラフィックスリソース管理（GraphicsCore）
- **既存window.rs**: ウィンドウ自体の管理（WindowHandle）

#### Trade-offs
- ✅ graphics.rsのサイズを抑えられる
- ✅ 責務が物理的に分離される
- ❌ ファイル数が増える（ナビゲーションの複雑化）
- ❌ GraphicsCoreとWindowGraphicsの関係が物理的に離れる
- ❌ インポートが増える（複数ファイルからインポート必要）
- ⚠️ 「ウィンドウ単位のグラフィックス」という責務が独立する十分な理由があるか不明確

### Option C: ハイブリッドアプローチ（段階的実装）

#### Phase 1: graphics.rsに最小実装
- WindowGraphicsとVisualコンポーネントのみ定義（データ構造）

#### Phase 2: システム実装
- create_window_graphicsとcreate_window_visualシステムを追加

#### Phase 3: 必要に応じてリファクタリング
- ファイルサイズが500行を超えたら分割を検討

#### Trade-offs
- ✅ 段階的なリスク管理
- ✅ 初期は最小限の変更で済む
- ❌ 複数フェーズの管理が必要
- ⚠️ Phase 1とPhase 2の境界が不明瞭（同時実装が自然）

---

## 4. Requirement-to-Asset Mapping

| Requirement | Existing Asset | Gap Status | Implementation |
|-------------|----------------|------------|----------------|
| WindowGraphicsコンポーネント | - | ❌ Missing | 新規作成（約40行） |
| Visualコンポーネント | - | ❌ Missing | 新規作成（約30行） |
| create_window_graphicsシステム | create_windowsパターン | ❌ Missing | パターン踏襲で実装（約50行） |
| create_window_visualシステム | - | ❌ Missing | 新規実装（約40行） |
| IDCompositionTarget作成 | dcomp.rs::create_target_for_hwnd | ✅ Ready | そのまま使用可能 |
| IDCompositionVisual3作成 | dcomp.rs::create_visual | ✅ Ready | そのまま使用可能 |
| set_root()実行 | dcomp.rs::set_root | ✅ Ready | そのまま使用可能 |
| ID2D1DeviceContext作成 | d2d/mod.rs::create_device_context | ⚠️ Constraint | ウィンドウ単位管理を設計 |
| システム登録（UISetup） | world.rs::add_systems | ✅ Ready | 既存パターンで対応 |
| 実行順序制御 | .before()/.after() | ✅ Ready | 既存パターンで対応 |
| COMライフタイム管理 | windows-rsスマートポインタ | ✅ Ready | 既存パターンで対応 |
| エラーハンドリング | Result + eprintln! | ✅ Ready | 既存パターンで対応 |
| ログ出力 | eprintln!マクロ | ✅ Ready | 既存パターンで対応 |

---

## 5. Implementation Complexity & Risk

### Effort Estimation: **M (Medium: 3-7 days)**

#### 根拠
- 新規コンポーネント2つ: 各1-2時間（パターン明確）
- 新規システム2つ: 各2-3時間（既存パターン踏襲、COM API呼び出し）
- システム登録と統合: 1-2時間
- テスト・デバッグ: 2-3日（複数ウィンドウでの動作確認、COMエラーハンドリング検証）
- ドキュメント: 0.5日

**合計**: 3-5日（開発者の既存コードベース理解度による）

### Risk Assessment: **Medium**

#### 根拠
- ✅ **技術的不確実性は低**: COM APIラッパー完備、ECSパターン確立
- ⚠️ **ID2D1DeviceContext管理**: ウィンドウ単位での作成方法のベストプラクティス確認が必要
- ⚠️ **複数ウィンドウでの動作未検証**: 既存のサンプルは単一ウィンドウのみ
- ✅ **統合リスクは低**: 既存コンポーネントへの影響なし、後方互換性維持
- ⚠️ **パフォーマンス検証**: DeviceContext作成がウィンドウ初期化時のボトルネックにならないか確認

#### リスク軽減策
1. DeviceContext作成方法について、DirectCompositionの公式ドキュメントまたはサンプルを確認
2. 複数ウィンドウでの動作を早期にテスト（simple_window.rsを拡張）
3. エラーハンドリングを徹底（COM APIの全呼び出しでResult処理）
4. 段階的実装（WindowGraphics → Visual → システム統合）

---

## 6. Recommendations for Design Phase

### 推奨アプローチ: **Option A - 既存graphics.rsを拡張**

#### 主な理由
1. **責務の凝集度**: グラフィックスリソース管理が1箇所に集約
2. **既存パターンとの一貫性**: GraphicsCore（グローバル）とWindowGraphics（ウィンドウ単位）の関係が明確
3. **実装コストの最小化**: 最小限のファイル変更で実装可能
4. **保守性**: 関連コードが1ファイルに集約され、理解しやすい

#### 設計フェーズで決定すべき事項

1. **ID2D1DeviceContext管理戦略**
   - **Research Needed**: 毎フレーム作成 vs 初回作成してキャッシュ
   - 推奨: 初回作成してWindowGraphicsに保持（パフォーマンス優先）
   - 根拠: DirectCompositionでは通常DeviceContextを再利用する設計が一般的

2. **WindowGraphicsコンポーネント構造**
   ```rust
   pub struct WindowGraphics {
       pub target: IDCompositionTarget,
       pub device_context: ID2D1DeviceContext,  // または ID2D1DeviceContext3
   }
   ```

3. **Visualコンポーネント構造**
   ```rust
   pub struct Visual {
       pub visual: IDCompositionVisual3,
   }
   ```

4. **システム実行順序**
   ```rust
   schedules.add_systems(
       UISetup,
       create_window_graphics.after(ensure_graphics_core)
   );
   schedules.add_systems(
       UISetup,
       create_window_visual.after(create_window_graphics)
   );
   ```

5. **エラーハンドリング詳細**
   - COM API各呼び出しでResult<T>を処理
   - エラー時はeprintln!でHRESULT含む詳細ログ出力
   - そのエンティティの処理をスキップ（continueで次へ）

#### 実装順序の推奨

1. **Phase 1**: WindowGraphicsコンポーネント定義
2. **Phase 2**: create_window_graphicsシステム実装
3. **Phase 3**: Visualコンポーネント定義
4. **Phase 4**: create_window_visualシステム実装
5. **Phase 5**: world.rsまたはexampleでシステム登録
6. **Phase 6**: 複数ウィンドウでのテスト・検証

---

## 7. Research Items for Design Phase

1. **DirectComposition DeviceContext管理のベストプラクティス**
   - 参考URL: Microsoft Docs - DirectComposition Samples
   - 確認事項: DeviceContextのライフサイクル管理、再利用パターン

2. **複数ウィンドウでのCompositionTarget動作**
   - 検証方法: simple_window.rsを拡張して2つのウィンドウを作成
   - 確認事項: Targetの独立性、リソース競合の有無

3. **ID2D1DeviceContext vs ID2D1DeviceContext3**
   - 確認事項: どちらを使用すべきか（windows-rsの型互換性）
   - 推奨: ID2D1DeviceContextで十分（必要に応じてcastで対応）

---

## 8. Out-of-Scope for This Analysis

以下は設計フェーズまたは実装フェーズで詳細化：

- 具体的なDeviceContext作成オプション（D2D1_DEVICE_CONTEXT_OPTIONS）
- VisualのプロパティやTransform設定（Milestone 3以降）
- 描画処理の実装（Milestone 3）
- 子Visual管理（Milestone 4）
- パフォーマンス最適化の詳細

---

## Conclusion

本マイルストーンは、既存のCOM APIラッパーとECSパターンを活用することで、**中程度の工数（3-7日）**かつ**中程度のリスク**で実装可能です。**Option A（既存graphics.rsを拡張）**を推奨アプローチとし、設計フェーズでID2D1DeviceContext管理戦略を明確化することで、スムーズな実装が期待できます。

---

_Gap Analysis completed. Ready for design phase._
