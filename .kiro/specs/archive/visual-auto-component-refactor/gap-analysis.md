# Gap Analysis: Visual Auto Component Refactor

## 1. Analysis Summary

現在のコードベースには、基本的な構成要素（`VisualGraphics`, `SurfaceGraphics`, `GraphicsCore`）は存在しますが、仕様で求められている**自動化**と**階層同期**の仕組みが欠けています。ECSの「Widget Tree」からDirectCompositionの「Visual Tree」を自動構築するロジックは現在実装されていません。

*   **論理コンポーネントの欠如**: `Visual` コンポーネントが存在しません。
*   **ツリー同期の欠如**: ECSの階層変更をDirectCompositionに反映する（`IDCompositionVisual::AddVisual`を呼び出す）システムが存在しません。
*   **手動リソース管理**: GPUリソースは現在、コンポーネントに基づいてリアクティブに生成されるのではなく、ヘルパー関数内で手動生成されています。
*   **描画ロジック**: 既存の `draw_recursive` 関数は、「Surface境界で再帰を停止する」という要件と互換性があり、そのまま活用できる可能性が高いです。

## 2. Document Status

*   [x] **Context Loaded**: Spec, Requirements, Steering files (Product, Structure, Tech) を確認済み。
*   [x] **Codebase Investigated**: `crates/wintf/src/ecs/graphics/` を調査。`VisualGraphics` と `SurfaceGraphics` の使用状況を分析済み。
*   [x] **Gap Identified**: アクティブなコード内で `AddVisual` が呼び出されていないことを確認（ツリー同期ロジックの欠如）。
* [x] **Approaches Evaluated**:「Systems vs Hooks」のトレードオフを分析済み。

## 3. Current State Investigation

### Existing Components
*   **`VisualGraphics`**: `IDCompositionVisual3` のラッパー。現在は手動で生成されている。
*   **`SurfaceGraphics`**: `IDCompositionSurface` のラッパー。現在は手動で生成されている。
*   **`GraphicsCore`**: D3D/D2D/DCompデバイスを保持する中心的なリソース。
*   **`draw_recursive`**: 子がSurfaceを持つ場合に再帰を停止するロジックを実装しており、要件と合致している。

### Missing Capabilities
*   **`Visual` Component**: ビジュアルノードを表す論理マーカーが存在しない。
*   **Automatic Resource Creation**: `Visual` を監視して `VisualGraphics`/`SurfaceGraphics` を生成するシステムが存在しない。
*   **Visual Tree Synchronization**: `ChildOf` の変更を監視して `AddVisual`/`RemoveVisual` を呼び出すシステムが存在しない。

## 4. Requirements Feasibility Analysis

### R1: 論理Visualコンポーネント
*   **実現可能性**: 高。`#[derive(Component)] struct Visual` を定義するのは容易。
*   **制約**: 不透明度や変形などの論理プロパティを保持し、`VisualGraphics` と同期させる必要がある。

### R2: 柔軟なVisual構成
*   **実現可能性**: 高。「常にSurfaceを作成する」という要件により、実装が大幅に簡素化される。
*   **制約**: `SurfaceGraphics` の生成には `GraphicsCore` が必要。

### R3: Visual階層の同期
*   **ステータス**: **Out of Scope**（今回のスコープ外）
*   **備考**: Visualツリーの構築仕様にて別途検討を行う。

### R4: リソース初期化・復旧
*   **実現可能性**: 高。`GraphicsCore` はすでに `is_valid()` を持っており、システム側でチェック可能。

### R5: 移行
*   **実現可能性**: 中。既存の初期化コード（`create_visual_for_target` 等）を、単に `Visual` コンポーネントを付与するだけの処理に変更する必要がある。

## 5. Investigation Item: GPU Resource Generation Strategy

**問い**: Bevy ECSのHooks/ObserversとSystems、どちらを使うべきか？

**分析**:
*   **Hooks (`on_add`)**:
    *   **メリット**: 即座に反応できる。
    *   **デメリット**: Hook内で `GraphicsCore` (Resource) にアクセスするには `world.get_resource()` が必要。可能だが、DCompリソース生成（COM割り当て）をHook内で行うと、メインスケジュールのブロックやスレッドの問題が発生するリスクがある。Hooksは軽量な論理状態の変更に向いている。
*   **Systems (`Query<Entity, Added<Visual>>`)**:
    *   **メリット**: Bevyの慣用的な方法。並列実行が可能（`GraphicsCore` の競合がなければ）。「リソース準備未完了」状態のハンドリングが容易（次のフレームにリトライすればよい）。
    *   **デメリット**: 1フレームの遅延が発生する（このユースケースでは許容範囲）。

**推奨**: **Systems** を使用する。
*   `visual_resource_system` を作成し、`Update` または `PostUpdate` で実行する。
*   `Added<Visual>`（または `VisualGraphics` を持たない `Visual`）をクエリする。
*   `GraphicsCore` が有効かチェックし、リソースを生成する。

## 6. Implementation Approach Options

### Option A: New Systems (推奨)
新しいモジュール `crates/wintf/src/ecs/graphics/visual_manager.rs`（仮）を作成する。

1.  **`Visual` Component**: `components.rs` で定義。
2.  **`visual_lifecycle_system`**:
    *   `Added<Visual>` を検知。
    *   `VisualGraphics` と `SurfaceGraphics` を生成。
3.  **`visual_tree_sync_system`**:
    *   Visualエンティティの `Changed<Children>` や `Added<Parent>` を検知。
    *   DCompツリーの接続を再構築。
4.  **`root_visual_system`**:
    *   Windowルートエンティティに `Visual` があることを保証する。

### Option B: Extend Existing Systems
`systems.rs` を修正してロジックを追加する。
*   **メリット**: ファイル分散が少ない。
*   **デメリット**: `systems.rs` はすでに肥大化している（600行以上）。
*   **結論**: 採用しない。関心の分離のため、新規モジュールが望ましい。

## 7. Next Steps

1.  **Design Phase**: `/kiro-spec-design visual-auto-component-refactor` を実行。
2.  **Focus Areas**:
    *   `Visual` 構造体のフィールド定義。
    *   ツリー同期における「ギャップ解決」アルゴリズムの詳細化。
    *   システムの実行順序設計（リソース生成 -> ツリー同期 -> 描画）。
