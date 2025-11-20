# Gap Analysis: surface-render-optimization

**Feature ID**: `surface-render-optimization`  
**Analysis Date**: 2025-11-20  
**Language**: ja

---

## Analysis Summary

- **スコープ**: `render_surface` システムの毎フレーム描画を廃止し、変更検知（ダーティフラグ伝播）に基づくオンデマンド描画へ移行する。
- **主な課題**:
  - 既存の `render_surface` は無条件で全走査・全描画を行っている。
  - 変更検知ロジック（`mark_dirty_surfaces`）の新規実装が必要。
  - ネストされた `SurfaceGraphics` を考慮したツリー走査ロジックの修正が必要。
- **推奨アプローチ**: **Option C (Hybrid Approach)** - 既存コンポーネントを拡張しつつ、新しいマーカーコンポーネントとシステムを追加する。
- **実装複雑度**: **Medium (3-5日)** - ロジック自体は複雑ではないが、再帰的な伝播とスキップ処理の正確性が求められる。
- **リスク**: **Low** - 描画漏れのリスクがあるが、ロジックは明確でありテスト可能。

---

## 1. Current State Investigation

### 1.1 Key Files and Directory Layout

#### 描画システム
- **`crates/wintf/src/ecs/graphics/systems.rs`**
  - `render_surface`: 現在のメイン描画ループ。`Query<(Entity, &SurfaceGraphics), With<Window>>` でウィンドウを取得し、`hierarchy.iter_descendants_depth_first` で全子孫を描画している。
  - **現状の課題**: 変更有無に関わらず毎フレーム `BeginDraw` / `EndDraw` を実行している。

#### コンポーネント定義
- **`crates/wintf/src/ecs/graphics/components.rs`**
  - `SurfaceGraphics`: 描画対象のサーフェス。
  - `WindowGraphics`, `VisualGraphics`: 関連リソース。
- **`crates/wintf/src/ecs/graphics/command_list.rs`**
  - `GraphicsCommandList`: 描画コマンドを保持。変更検知のトリガーの一つ。
- **`crates/wintf/src/ecs/layout.rs`**
  - `GlobalArrangement`: グローバル配置情報。変更検知のトリガーの一つ。

### 1.2 Architectural Patterns and Constraints

#### ECSパターン
- **Bevy ECS 0.17.2**: `Query`, `Commands`, `Res` を使用。
- **Change Detection**: Bevy標準の `Changed<T>` フィルタや `Ref<T>::is_changed()` が利用可能。
- **Hierarchy**: `Children` コンポーネントによる親子関係。

#### 描画フロー
1. `render_surface` が全ウィンドウを走査。
2. 各ウィンドウで `BeginDraw`。
3. 子孫を深さ優先探索し、`GraphicsCommandList` があれば `DrawImage`。
4. `EndDraw`。
5. `commit_composition` で反映。

### 1.3 Integration Surfaces

- **DirectComposition**: `IDCompositionSurface` への描画。
- **Direct2D**: `ID2D1DeviceContext` を使用した描画コマンドの発行。

---

## 2. Requirements Feasibility Analysis

### 2.1 Technical Needs Mapping

| Requirement | 既存資産 | Gap | 実装アプローチ |
|------------|---------|-----|--------------|
| **R1: 変更検知** | `Changed<T>` | 伝播ロジックなし | 新規システム `mark_dirty_surfaces` 実装 |
| **R1: マーカー** | なし | 定義なし | 新規コンポーネント `SurfaceUpdateRequested` 定義 |
| **R2: 走査範囲** | `iter_descendants` | スキップ機能なし | 再帰関数によるカスタム走査の実装 |
| **R2: ネスト除外** | なし | ロジックなし | 走査時の条件分岐追加 |
| **R3: ログ抑制** | `eprintln!` | 制御なし | 条件付きログ出力への変更 |

### 2.2 Missing Capabilities

#### ダーティフラグ伝播システム
- **現状**: 変更を検知して親へ伝播させる仕組みが存在しない。
- **必要機能**:
  - `GraphicsCommandList`, `GlobalArrangement`, `Children` の変更を監視。
  - 親を遡り、直近の `SurfaceGraphics` 持ちエンティティを特定。
  - `SurfaceUpdateRequested` を付与。

#### 最適化された描画システム
- **現状**: `render_surface` は無条件描画。
- **必要機能**:
  - `SurfaceUpdateRequested` を持つエンティティのみを処理対象とするフィルタリング。
  - ネストされた `SurfaceGraphics` をスキップする再帰的描画ロジック。
  - 処理完了後のマーカー削除。

### 2.3 Complexity Signals

#### 再帰的走査とスキップ
- Bevyの `iter_descendants` はフラットなイテレータを返すため、途中で枝刈り（スキップ）することが難しい。
- **対策**: `Children` コンポーネントを手動で辿る再帰関数を実装する必要がある。

#### 親への遡及
- `Parent` コンポーネントを辿る処理は単純だが、階層が深い場合のパフォーマンスに注意が必要（ただしGUIツリーの深さは通常限定的）。

---

## 3. Implementation Approach Options

### Option A: Extend Existing System (非推奨)
**Rationale**: `render_surface` 内ですべての判定を行う（プル型）。
- **Pros**: システムが増えない。
- **Cons**: 毎フレーム全ツリー走査が必要になり、静止時の負荷がゼロにならない。要件（プッシュ型）と不一致。

### Option B: New Systems & Components (推奨 - Hybrid)
**Rationale**: 変更検知システムと描画システムを分離し、マーカーコンポーネントで連携する（プッシュ型）。

#### 新規コンポーネント
- `SurfaceUpdateRequested`: マーカーコンポーネント。

#### 新規システム
- `mark_dirty_surfaces`: 変更を検知し、親サーフェスにマーカーを付ける。
  - Query: `Changed<GraphicsCommandList>`, `Changed<GlobalArrangement>`, `Changed<Children>`
- `render_surface` (改修): マーカー付きのみ処理。
  - Query: `(Entity, &SurfaceGraphics), With<SurfaceUpdateRequested>`

#### 実装手順
1. `SurfaceUpdateRequested` コンポーネント定義 (`crates/wintf/src/ecs/graphics/components.rs`)。
2. `mark_dirty_surfaces` システム実装 (`crates/wintf/src/ecs/graphics/systems.rs`)。
3. `render_surface` システム改修 (`crates/wintf/src/ecs/graphics/systems.rs`)。
   - クエリフィルタ追加。
   - 再帰描画ロジックの刷新（`iter_descendants` から手動再帰へ）。
   - マーカー削除処理追加。

**Trade-offs**:
- ✅ 静止時のCPU負荷を最小化できる。
- ✅ 責務が明確に分離される。
- ❌ システム間の連携が必要になる。

---

## 4. Implementation Complexity & Risk

### Effort: **Medium (3-5日)**

#### 根拠
- ロジック自体は標準的だが、描画システムのコア部分に手を入れるため慎重な実装が必要。
- 再帰ロジックのテストと検証に時間が必要。

### Risk: **Low**

#### 潜在的リスク
- **描画漏れ**: 伝播ロジックやスキップロジックのバグにより、更新されるべき箇所が更新されないリスク。
  - **対策**: ユニットテストまたは目視確認用サンプル (`examples/dcomp_demo.rs`) での徹底検証。
- **無限ループ**: 親子関係の循環（通常ありえないが）による無限ループ。
  - **対策**: Bevyの `Parent`/`Children` は循環を防ぐ仕組みがあるが、念のためループ回数制限などを考慮。

---

## 5. Recommendations for Design Phase

### Preferred Approach
**Option B (New Systems & Components)** - ダーティフラグ伝播方式を採用。

### Key Decisions for Design Phase

1. **マーカーコンポーネントの定義場所**
   - `crates/wintf/src/ecs/graphics/components.rs` に配置。

2. **システム分割**
   - `mark_dirty_surfaces` を独立したシステムとして実装し、`render_surface` の前に実行されるようにスケジューリングする。

3. **再帰描画ロジック**
   - `fn draw_recursive(...)` のようなヘルパー関数を作成し、`render_surface` から呼び出す形にする。
   - 引数で `&Query<...>` を渡して再帰的にエンティティを取得・描画する。

### Research Items
- 特になし。Bevyの標準機能で実現可能。

---

**Analysis Approach**: 既存コードの構造解析と要件とのマッピングによるギャップ特定。

**Next Steps**: `/kiro-spec-design surface-render-optimization` を実行して技術設計フェーズに進む。
