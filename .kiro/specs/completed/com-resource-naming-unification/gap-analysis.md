# Gap Analysis: com-resource-naming-unification

**Feature ID**: `com-resource-naming-unification`  
**Analysis Date**: 2025-11-17  
**Language**: ja

---

## Analysis Summary

- **スコープ**: COMリソースコンポーネント（`Visual`、`Surface`、`TextLayout`）の命名を統一し、GPU/CPUリソースの区別を明確化する改名作業
- **主な課題**: 既存の3つのコンポーネント名を変更し、すべての参照箇所（システム、クエリ、テスト）を更新する必要がある
- **推奨アプローチ**: オプションA（既存ファイル拡張）- コンポーネント定義ファイルを直接編集し、参照箇所を一括更新
- **実装複雑度**: **Small (1-3日)**
- **リスク**: **Low** - 型システムによる安全性、既存パターンの維持、明確なコンパイルエラー

---

## 1. Current State Investigation

### 1.1 Key Files and Directory Layout

#### COMリソースコンポーネント定義
- **`crates/wintf/src/ecs/graphics/components.rs`** (120行)
  - `WindowGraphics` (22-65行) - ウィンドウレベル、GPU、既に命名規則準拠
  - `Visual` (67-93行) - ウィジェットレベル、GPU、改名対象: `VisualGraphics`
  - `Surface` (95-120行) - ウィジェットレベル、GPU、改名対象: `SurfaceGraphics`
  - マーカーコンポーネント: `HasGraphicsResources`, `GraphicsNeedsInit`（改名対象外）

- **`crates/wintf/src/ecs/widget/text/label.rs`** (95行)
  - `Label` (22-50行) - 論理コンポーネント（改名対象外）
  - `TextLayout` (65-88行) - ウィジェットレベル、CPU、改名対象: `TextLayoutResource`

#### システム実装ファイル
- **`crates/wintf/src/ecs/graphics/systems.rs`** (568行)
  - `create_window_graphics_for_hwnd()` - `WindowGraphics`生成
  - `create_visual_for_target()` - `Visual`生成（改名影響: 戻り値型）
  - `create_surface_for_window()` - `Surface`生成（改名影響: 引数型、戻り値型）
  - `render_surface()` - `Surface`を使用（改名影響: クエリ型）
  - `init_window_visual()` - `Visual`初期化（改名影響: クエリ型、挿入型）
  - `init_window_surface()` - `Surface`初期化（改名影響: クエリ型、挿入型）

- **`crates/wintf/src/ecs/widget/text/draw_labels.rs`** (176行)
  - `draw_labels()` - `TextLayout`を生成・挿入（改名影響: 型、構築、挿入）

#### モジュールエクスポート
- **`crates/wintf/src/ecs/graphics/mod.rs`**
  - `Visual`, `Surface`をpubエクスポート（改名必須）
- **`crates/wintf/src/ecs/widget/text/mod.rs`**
  - `TextLayout`をpubエクスポート（改名必須）

#### スケジュール登録
- **`crates/wintf/src/ecs/world.rs`** (300行)
  - `init_window_visual`, `init_window_surface`のスケジュール登録（関数名は維持）
  - システムコメントで`IDCompositionVisual`, `IDCompositionSurface`を参照（ドキュメント更新不要）

#### テストファイル
- **`crates/wintf/tests/graphics_core_ecs_test.rs`**
  - `WindowGraphics`使用のみ（改名対象外）
- **`crates/wintf/tests/graphics_reinit_unit_test.rs`**
  - `WindowGraphics`使用のみ（改名対象外）
- **`crates/wintf/tests/lazy_reinit_pattern_test.rs`**
  - テスト用`WindowGraphics`構造体を定義（本体とは別）
- **他テスト**: `transform_test.rs`, `component_state_pattern_test.rs`等は改名対象コンポーネントを使用せず

### 1.2 Architectural Patterns and Constraints

#### レイヤードアーキテクチャ
- **COM Wrapper層** (`com/`) → **ECS Component層** (`ecs/`) → **Message Handling層** (ルート)
- 依存方向の厳守: 上位層が下位層に依存
- 改名は**ECS Component層**のみに影響、COM層への影響なし

#### ECSパターン
- **bevy_ecs 0.17.2**使用
- コンポーネント定義: `#[derive(Component)]` + `#[component(storage = "SparseSet")]`
- GPU資源パターン: `Option<T>` + `invalidate()` + `generation`フィールド
- CPU資源パターン: `Option<T>`のみ（再初期化不要）

#### デバイスロスト対応
- GPU資源: `invalidate()`で`None`化、`generation`カウンタでバージョン管理
- CPU資源: デバイス非依存のため`invalidate()`/`generation`実装なし
- 改名後もこのパターンを維持（要件5で明記）

#### 命名規則（現状）
- **Files**: `snake_case.rs`
- **Types**: `PascalCase`
- **Functions**: `snake_case`
- **COM wrapper**: アクセスメソッド名は内部COM型に対応（例: `visual()`, `surface()`, `get()`）

### 1.3 Integration Surfaces

#### COMオブジェクトラップ
- `Visual` → `IDCompositionVisual3`
- `Surface` → `IDCompositionSurface`
- `TextLayout` → `IDWriteTextLayout`
- アクセスメソッド名は改名後も維持（要件で明示）

#### システム間依存
- `init_window_visual` → `create_visual_for_target()`
- `init_window_surface` → `create_surface_for_window()`
- `draw_labels` → `TextLayout::new()`構築
- `render_surface` → `Surface`クエリ
- デバイスロスト検出システム → GPU資源の`invalidate()`呼び出し

---

## 2. Requirements Feasibility Analysis

### 2.1 Technical Needs Mapping

| Requirement | 既存資産 | Gap | 実装アプローチ |
|------------|---------|-----|--------------|
| **R1: WindowGraphics維持** | `WindowGraphics`既存 | なし | 変更不要（既に準拠） |
| **R2: Visual→VisualGraphics** | `Visual`定義済み | 改名のみ | 型名変更、参照箇所更新 |
| **R2: Surface→SurfaceGraphics** | `Surface`定義済み | 改名のみ | 型名変更、参照箇所更新 |
| **R2: TextLayout→TextLayoutResource** | `TextLayout`定義済み | 改名のみ | 型名変更、参照箇所更新 |
| **R3: 将来の命名規則** | なし | ルール定義 | ドキュメント記載 |
| **R4: 共有リソース命名規則** | なし | ルール定義 | ドキュメント記載 |
| **R5: 移行安全性** | テスト基盤存在 | 検証手順 | `cargo test`で確認 |
| **R6: ドキュメント更新** | `structure.md`存在 | セクション追加 | 命名規則セクション追記 |
| **R7: 一貫性検証** | なし | レビュープロセス | 設計フェーズで定義 |

### 2.2 Missing Capabilities

#### 改名対象の明確なマッピング
- ✅ **既存**: 3コンポーネントの定義位置が明確
- ❌ **Gap**: 全参照箇所のリスト化が未実施 → grep検索で特定済み（下記参照）

#### 参照箇所の網羅（grep結果より）
- `Visual`: 30+ マッチ（主に`com/dcomp.rs`のCOM wrapper、`ecs/graphics/systems.rs`のシステム）
- `Surface`: 30+ マッチ（主に`ecs/graphics/systems.rs`、`ecs/world.rs`のスケジュール）
- `TextLayout`: 29マッチ（主に`ecs/widget/text/`配下、`com/dwrite.rs`、`com/d2d/mod.rs`）

#### テスト影響範囲
- ✅ **確認済み**: 既存テストは`WindowGraphics`のみ使用
- ✅ **安全性**: 改名対象3コンポーネントを直接使用するテストなし
- ⚠️ **注意**: `lazy_reinit_pattern_test.rs`がテスト用`WindowGraphics`を定義（別物）

### 2.3 Complexity Signals

#### Simple Renaming Task
- 型名変更のみで、ロジック変更なし
- Rustの型システムによりコンパイルエラーで漏れ検出
- 既存のCOMラッパーメソッド名（`visual()`, `surface()`, `get()`）は維持
- デバイスロスト対応パターン（`invalidate()`, `generation`）も維持

#### No External Integrations
- 外部APIや非同期処理への影響なし
- 純粋なコードベース内のリファクタリング

---

## 3. Implementation Approach Options

### Option A: Extend Existing Components (推奨)
**Rationale**: 既存のコンポーネント定義ファイルを直接編集し、型名のみを変更する。

#### 影響ファイル
1. **コンポーネント定義**:
   - `crates/wintf/src/ecs/graphics/components.rs`
     - `pub struct Visual` → `pub struct VisualGraphics`
     - `pub struct Surface` → `pub struct SurfaceGraphics`
   - `crates/wintf/src/ecs/widget/text/label.rs`
     - `pub struct TextLayout` → `pub struct TextLayoutResource`

2. **モジュールエクスポート**:
   - `crates/wintf/src/ecs/graphics/mod.rs`
     - `pub use components::{..., Visual, Surface, ...}` → `VisualGraphics, SurfaceGraphics`
   - `crates/wintf/src/ecs/widget/text/mod.rs`
     - `pub use label::{..., TextLayout}` → `TextLayoutResource`

3. **システム実装**:
   - `crates/wintf/src/ecs/graphics/systems.rs`
     - 関数シグネチャ、クエリ型、コンストラクタ呼び出しを更新
   - `crates/wintf/src/ecs/widget/text/draw_labels.rs`
     - クエリ型、コンストラクタ呼び出しを更新

4. **ドキュメント**:
   - `.kiro/steering/structure.md`
     - 命名規則セクション追加（GPU: `XxxGraphics`, CPU: `XxxResource`）

#### 互換性評価
- ✅ **破壊的変更**: 型名変更のため既存コードは一括更新必須
- ✅ **型安全性**: Rustコンパイラが未更新箇所を検出
- ✅ **アクセスメソッド維持**: `visual()`, `surface()`, `get()`は変更なし
- ✅ **ストレージ維持**: `#[component(storage = "SparseSet")]`変更なし
- ✅ **スレッド安全性維持**: `Send`/`Sync`実装維持

#### 実装手順
1. コンポーネント定義の型名変更（3ファイル）
2. モジュールエクスポートの更新（2ファイル）
3. システム実装の型参照更新（grep結果に基づき一括置換）
4. `cargo build`でコンパイルエラー解消
5. `cargo test`で既存テスト成功確認
6. ドキュメント更新（`structure.md`）

#### Trade-offs
- ✅ 最小限のファイル編集（既存ファイルの型名のみ）
- ✅ 既存パターン・構造の完全維持
- ✅ テスト影響なし（改名対象コンポーネントを使用するテストなし）
- ✅ Rustの型チェックにより漏れなし
- ❌ 一時的にコンパイルエラー大量発生（設計フェーズで段階的更新戦略を定義）

### Option B: Create New Components
**Rationale**: 新規に`VisualGraphics`等を定義し、旧名を`#[deprecated]`化して段階的移行。

#### 非推奨の理由
- ❌ コンポーネント数増加（一時的に6コンポーネント存在）
- ❌ エンティティへの二重アタッチリスク（旧新両方が存在）
- ❌ デバイスロスト対応システムが両方を認識する必要
- ❌ 実装コストが高い（エイリアス管理、移行期間の管理）
- ⚠️ ECSでは型がコンポーネントの識別子のため、別型は別コンポーネント扱い

### Option C: Hybrid Approach
**Rationale**: コンポーネント改名とドキュメント整備を段階的に実施。

#### 実装可能性
- Phase 1: GPU資源（`Visual`, `Surface`）改名
- Phase 2: CPU資源（`TextLayout`）改名
- Phase 3: ドキュメント整備

#### 非推奨の理由
- ⚠️ 段階分けのメリットが少ない（改名は機械的作業）
- ⚠️ 一度に実施した方が一貫性が高い
- ✅ **ただし採用可能**: リスク分散が必要な場合は検討価値あり

---

## 4. Implementation Complexity & Risk

### Effort: **Small (S: 1-3日)**

#### 根拠
- 型名変更のみで、ロジック変更なし
- grep検索で参照箇所が明確（Visual: 30+, Surface: 30+, TextLayout: 29）
- 既存パターンの完全維持（アクセスメソッド、ストレージ、スレッド安全性）
- テスト影響なし（改名対象コンポーネントを使用するテストが存在しない）

#### 作業内訳
- Day 1: コンポーネント定義変更（3ファイル）+ 参照箇所更新（システム実装）
- Day 2: コンパイルエラー解消 + テスト実行 + ドキュメント更新
- Day 3: レビュー + 最終検証

### Risk: **Low**

#### 低リスクの根拠
1. **型システムによる安全性**
   - Rustコンパイラが未更新箇所をすべて検出
   - 実行時エラーのリスクなし

2. **既存パターンの維持**
   - `invalidate()`/`generation`パターン維持
   - COMオブジェクトアクセスメソッド名維持
   - ストレージタイプ・スレッド安全性維持

3. **テスト基盤の存在**
   - `cargo test`で動作確認可能
   - 既存テストに改名影響なし

4. **明確なスコープ**
   - 改名のみ、機能追加なし
   - COM層・メッセージ層への影響なし

#### 潜在的リスク
- ⚠️ **一時的コンパイルエラー大量発生**: 改名中にビルド不可（対策: ブランチ作業、一括更新）
- ⚠️ **ドキュメント記載漏れ**: 命名規則の説明が不十分（対策: 設計フェーズで詳細定義）

---

## 5. Recommendations for Design Phase

### Preferred Approach
**Option A (Extend Existing Components)** - 既存ファイルの型名を直接変更し、参照箇所を一括更新

### Key Decisions for Design Phase

1. **改名順序の決定**
   - 推奨: 1ブランチで全コンポーネント一括改名（一貫性重視）
   - 代替: GPU資源→CPU資源の段階的改名（リスク分散）

2. **参照箇所更新の戦略**
   - grep検索結果に基づく一括置換リスト作成
   - IDEのリファクタリング機能活用（可能な場合）
   - コンパイルエラーベースの逐次修正

3. **ドキュメント命名規則セクション内容**
   - GPU資源（`XxxGraphics`）: デバイスロスト対応、`invalidate()`/`generation`
   - CPU資源（`XxxResource`）: 永続的、再初期化不要
   - ウィンドウレベル vs ウィジェットレベル vs 共有リソース
   - 具体例: `WindowGraphics`, `VisualGraphics`, `TextLayoutResource`

4. **テスト戦略**
   - 既存テスト: 改名影響なし（`WindowGraphics`のみ使用）
   - 統合テスト: サンプルアプリ実行で確認（`examples/areka.rs`等）
   - コンパイル確認: `cargo build --all-targets`

### Research Items

なし（改名作業のため新規調査不要）

---

## Appendix: Reference Search Results

### Visual References (30+)
- `com/dcomp.rs`: COM wrapper実装（`create_visual()`, `add_visual()`等）
- `ecs/graphics/components.rs`: コンポーネント定義
- `ecs/graphics/systems.rs`: システム実装（`create_visual_for_target()`, `init_window_visual()`）
- `ecs/world.rs`: スケジュール登録

### Surface References (30+)
- `ecs/graphics/components.rs`: コンポーネント定義
- `ecs/graphics/systems.rs`: システム実装（`create_surface_for_window()`, `render_surface()`, `init_window_surface()`）
- `ecs/world.rs`: スケジュール登録（`RenderSurface`）

### TextLayout References (29)
- `ecs/widget/text/label.rs`: コンポーネント定義
- `ecs/widget/text/draw_labels.rs`: システム実装（`draw_labels()`）
- `ecs/widget/text/mod.rs`: モジュールエクスポート
- `com/dwrite.rs`: COM wrapper（`create_text_layout()`）
- `com/d2d/mod.rs`: Direct2D wrapper（`draw_text_layout()`）

---

**Analysis Approach**: gap-analysis.mdフレームワークに準拠した徹底調査

**Next Steps**: `/kiro-spec-design com-resource-naming-unification`を実行して技術設計フェーズに進む
