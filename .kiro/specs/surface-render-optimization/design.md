# Design: surface-render-optimization

**Feature**: Surface Render Optimization
**Status**: Approved

## 1. Architecture Overview

### 1.1 Concept: Push Model (Dirty Flag)
従来の「毎フレーム全探索・全描画」から、「変更検知 → ダーティフラグ設定 → 必要な場合のみ描画」というプッシュ型モデルへ移行する。

### 1.2 Data Flow
1. **Change Detection**: `mark_dirty_surfaces` システムがコンポーネントの変更（`GraphicsCommandList`, `Children` 等）を検知。
2. **Propagation**: 変更箇所から親方向へツリーを遡り、描画責任を持つ `SurfaceGraphics` を特定。
3. **Marking**: 特定されたサーフェスエンティティに `SurfaceUpdateRequested` マーカーを付与。
4. **Synchronization**: `apply_deferred` により、同一フレーム内でマーカーの付与を確定させる。
5. **Rendering**: `render_surface` システムがマーカーを持つサーフェスのみを抽出し、再帰的に描画コマンドを発行する。
   - **Nested Surface Skipping**: 再帰描画中に子要素が `SurfaceGraphics` を持っている場合、そのサブツリーの描画はスキップする（そのサーフェス自身の責務であるため）。

## 2. Component Design

### 2.1 `SurfaceUpdateRequested`
描画更新が必要なサーフェスを示すマーカーコンポーネント。

```rust
#[derive(Component, Default)]
pub struct SurfaceUpdateRequested;
```

- **定義場所**: `crates/wintf/src/ecs/graphics/components.rs`
- **ライフサイクル**: `mark_dirty_surfaces` で付与され、`render_surface` の処理完了時に削除される。

## 3. System Design

### 3.1 `mark_dirty_surfaces`
変更を検知し、ダーティフラグを立てるシステム。

- **Input**:
  - `Changed<GraphicsCommandList>`: 描画内容の変更
  - `Changed<GlobalArrangement>`: レイアウト/配置の変更
  - `Changed<Children>`: 階層構造の変更
  - `Parent`: 親への遡及用
  - `SurfaceGraphics`: 到達目標
- **Logic**:
  1. 変更があったエンティティを収集。
  2. 各エンティティについて、`Parent` を辿って親へ移動。
  3. `SurfaceGraphics`コンポーネントを持つエンティティに到達したら、`Commands`を使用して`SurfaceUpdateRequested`をinsertし、探索を終了。
  4. ルートに到達しても見つからない場合は無視（通常ありえないが）。

### 3.2 `render_surface` (Refactoring)
マーカーが付いたサーフェスのみを描画するよう修正。

- **Input**:
  - Query: `(Entity, &SurfaceGraphics, &WindowGraphics, &mut VisualGraphics), With<SurfaceUpdateRequested>`
- **Logic**:
  1. クエリにマッチしたエンティティ（更新が必要なサーフェス）に対してループ。
  2. `BeginDraw` を呼び出し。
  3. ヘルパー関数 `draw_recursive(entity, ...)` を呼び出し。
  4. `EndDraw` を呼び出し。
  5. `commit_composition` を呼び出し。
  6. `Commands`を使用して`SurfaceUpdateRequested`をremove。

### 3.3 `draw_recursive` (Helper)
再帰的に描画コマンドを発行する。

- **Arguments**: `entity`, `offset`, `command_list_query`, `children_query`, `surface_query`
- **Logic**:
  1. **Visual Update**:
     - 現在のエンティティの `Visual` に対して、位置や変形（Offset等）を適用する（親サーフェスの責務）。
  2. **Nested Surface Check**:
     - 現在の `entity` が `SurfaceGraphics` を持っており、かつ **ルート（呼び出し元）でない** 場合、ここでリターン（コンテンツ描画と子要素への再帰をスキップ）。
  3. **Draw**:
     - `GraphicsCommandList` を持っていれば `DrawImage` 等を実行。
  4. **Recurse**:
     - `Children` を取得し、各子エンティティに対して `draw_recursive` を呼び出す。

## 4. Scheduling

Bevyのスケジュール順序を明示的に制御し、変更検知から描画までを1フレーム内で完結させる。

```rust
// crates/wintf/src/ecs/world.rs または graphics/mod.rs

app.add_systems(Update, (
    mark_dirty_surfaces,
    apply_deferred, // コマンド(addComponent)を即時反映
    render_surface
).chain());
```

- **理由**: `mark_dirty_surfaces` は `Commands` 経由でコンポーネントを追加するため、その効果は次の `apply_deferred`（通常はフレームの終わり）まで反映されない。`render_surface` が同じフレーム内でマーカーを認識するためには、間に明示的な `apply_deferred` が必要。

## 5. Implementation Steps

1. **Define Component**: `SurfaceUpdateRequested` を `components.rs` に追加。
2. **Implement `mark_dirty_surfaces`**: `systems.rs` に実装。
3. **Refactor `render_surface`**:
   - `draw_recursive` 関数の分離と実装（ネストスキップロジック含む）。
   - クエリフィルタの追加。
   - マーカー削除処理の追加。
4. **Update Scheduling**: `world.rs` でシステム登録順序を修正。
5. **Add Tests**: `crates/wintf/tests/surface_optimization_test.rs` を作成し、ロジックを検証。

## 6. Test Plan

### 6.1 Unit Testing (Logic Verification)
実際の描画を行わず、ECSのロジックのみを検証するテストを作成する。

- **File**: `crates/wintf/tests/surface_optimization_test.rs`
- **Scenarios**:
  1. **Propagation**: 子要素の変更が親サーフェスに伝播し、マーカーが付与されること。
  2. **Nested Surface Isolation**: ネストされたサーフェスがある場合、変更が親サーフェス（外側）に伝播せず、該当するサーフェス（内側）で止まること。
  3. **Marker Cleanup**: 描画システム実行後にマーカーが削除されること。

### 6.2 Regression Testing (Visual Verification)
既存のサンプルアプリを使用して、描画崩れがないことと、静止時の負荷軽減を確認する。

- **Target**: `examples/simple_window.rs`
- **Verification**:
  - アプリ起動後、描画が正常に行われること。
  - ログ出力を一時的に有効化し、静止時に `render_surface` が実行されていない（ログが出ない）ことを確認。
  - ウィンドウリサイズ等の操作時に、適切に再描画が行われることを確認。
  - **Note**: 現状のサンプルでは `SurfaceGraphics` のネスト構造は存在しないため、ネストの検証はユニットテストに依存する。
