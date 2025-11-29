# Gap Analysis: ECS Component Grouping Refactoring

**Feature**: ecs-component-grouping  
**Phase**: Gap Analysis  
**Date**: 2025-11-22

## 分析概要

### スコープ
既存のECSコンポーネント（`crates/wintf/src/ecs/`配下）を5つの機能グループに再編成し、サブフォルダ構造を導入するリファクタリング。API互換性を維持しながら、コードベースの可読性と保守性を向上させる。

### 主要な課題
1. **レイアウトシステムの統合**: 現在`layout.rs`（517行）、`arrangement.rs`（60行）、`tree_system.rs`（371行）に分散している関連機能を`ecs/layout/`サブフォルダに統合
2. **Transform の実験的位置付け**: WinUI3模倣として実装された`transform.rs`（191行）を`ecs/transform/`に隔離し、非推奨化
3. **共通インフラの明確化**: `tree_system.rs`の汎用関数群を`ecs/common/`に移動し、ドメイン非依存の基盤として確立
4. **API互換性の維持**: `pub use`による再エクスポートで外部利用者のインポートパスを保護

### 推奨アプローチ
**Option C: Hybrid Approach** - 既存ファイルの分割（Layout System）と新規ディレクトリ作成（Common Infrastructure、Transform Deprecation）を組み合わせた段階的リファクタリング。

---

## 1. 現状分析

### 1.1 既存アセット

#### ディレクトリ構造
```
crates/wintf/src/ecs/
├── mod.rs                 # モジュール定義・再エクスポート（25行）
├── window.rs              # Windowグループ（423行）
├── layout.rs              # Layout + Arrangement（517行）
├── arrangement.rs         # Arrangement伝播システム（60行）
├── transform.rs           # Transform実験的実装（191行）
├── tree_system.rs         # 汎用階層伝播関数（371行）
├── graphics/              # Graphicsグループ（既存サブフォルダ）
│   ├── mod.rs
│   ├── components.rs      # WindowGraphics, VisualGraphics等
│   ├── core.rs
│   ├── systems.rs
│   ├── command_list.rs
│   └── visual_manager.rs
└── widget/                # Widgetグループ（既存サブフォルダ）
    ├── mod.rs
    ├── text/
    └── shapes/
```

#### 主要コンポーネント一覧

**Windowグループ** (`window.rs`):
- `Window`, `WindowHandle`, `WindowStyle`, `WindowPos`, `DpiTransform`, `ZOrder`
- Component hooks (`on_window_handle_add`, `on_window_handle_remove`)

**Graphicsグループ** (`graphics/components.rs`):
- `WindowGraphics`, `VisualGraphics`, `SurfaceGraphics`, `TextLayoutResource`
- `HasGraphicsResources`, `GraphicsNeedsInit`, `SurfaceUpdateRequested`

**Layout System** (複数ファイルに分散):
- **taffy連携** (`layout.rs`): `BoxStyle`, `BoxComputedLayout`
- **メトリクス** (`layout.rs`): `Size`, `Offset`, `LayoutScale`, `TextLayoutMetrics`
- **配置** (`layout.rs`): `Arrangement`, `GlobalArrangement`, `ArrangementTreeChanged`
- **矩形操作** (`layout.rs`): `Rect`型エイリアス, `D2DRectExt`トレイト, `transform_rect_axis_aligned`関数
- **伝播システム** (`arrangement.rs`): `sync_simple_arrangements`, `mark_dirty_arrangement_trees`, `propagate_global_arrangements`

**Transform（実験的）** (`transform.rs`):
- `Translate`, `Scale`, `Rotate`, `Skew`, `TransformOrigin`
- `Transform`, `GlobalTransform`, `TransformTreeChanged`

**Common Infrastructure** (`tree_system.rs`):
- 汎用関数: `sync_simple_transforms<L, G, M>`, `mark_dirty_trees<L, G, M>`, `propagate_parent_transforms<L, G, M>`
- サポート型: `WorkQueue`, `NodeQuery<L, G, M>`

**Widget** (`widget/`):
- `widget/text/`: `Label`等
- `widget/shapes/`: `Rectangle`等

### 1.2 アーキテクチャパターン

#### レイヤード分離
- **COM Wrapper Layer** (`com/`): Windows APIラッパー
- **ECS Component Layer** (`ecs/`): コンポーネント定義とシステム
- **Message Handling** (ルート): ウィンドウプロシージャとスレッド管理

#### 命名規則
- **GPU Resources**: `XxxGraphics`サフィックス（例: `WindowGraphics`）
- **CPU Resources**: `XxxResource`サフィックス（例: `TextLayoutResource`）
- **Logical Components**: サフィックスなし（例: `Label`, `Rectangle`）
- **Markers**: 用途に応じた名前（例: `HasGraphicsResources`, `GraphicsNeedsInit`）

#### Component Hooks
`WindowHandle`コンポーネントは`on_add`, `on_remove`フックを使用してライフサイクル管理を実装：
```rust
#[component(storage = "SparseSet", on_add = on_window_handle_add, on_remove = on_window_handle_remove)]
```

#### 汎用伝播システム
`tree_system.rs`は型パラメータ`<L, G, M>`（Local, Global, Marker）を使用して、異なるドメイン（Layout、Transform）で再利用可能な階層伝播関数を提供：
```rust
pub fn sync_simple_transforms<L, G, M>(...) where
    L: Component + Copy + Into<G>,
    G: Component<Mutability = Mutable> + Copy + PartialEq + Mul<L, Output = G>,
    M: Component<Mutability = Mutable>,
```

### 1.3 統合サーフェス

#### 公開API (`ecs/mod.rs`)
```rust
pub use app::*;
pub use arrangement::*;
pub use bevy_ecs::hierarchy::{ChildOf, Children};
pub use graphics::*;
pub use layout::*;
pub use transform::*;
pub use tree_system::*;
pub use window::{Window, WindowHandle, WindowPos, WindowStyle, ZOrder};
```

**重要**: リファクタリング後もこの`pub use`構造を維持し、外部利用者のインポートパスを保護する必要がある。

#### テスト
- `cargo test`で4つのdoctestが実行される（`layout.rs`内）
- サンプルアプリケーション: `cargo run --example areka`, `cargo run --example dcomp_demo`

---

## 2. 要件実現可能性分析

### 2.1 技術的要求

#### Requirement 1: 5グループ定義
- **Window**: ✅ 既に`window.rs`に集約済み（移動不要）
- **Graphics**: ✅ 既に`graphics/`サブフォルダ化済み（移動不要）
- **Layout System**: ⚠️ `layout.rs`, `arrangement.rs`を`ecs/layout/`に再編成必要
- **Widget**: ✅ 既に`widget/`サブフォルダ化済み（移動不要）
- **Common Infrastructure**: ⚠️ `tree_system.rs`を`ecs/common/`に移動必要

#### Requirement 4: Layout System統合
**目標サブモジュール構造**:
```
ecs/layout/
├── mod.rs              # 再エクスポート
├── taffy.rs            # BoxStyle, BoxComputedLayout
├── metrics.rs          # Size, Offset, LayoutScale, TextLayoutMetrics
├── arrangement.rs      # Arrangement, GlobalArrangement, ArrangementTreeChanged
├── rect.rs             # Rect型エイリアス, D2DRectExt, transform_rect_axis_aligned
└── systems.rs          # sync_simple_arrangements等のシステム関数
```

**分割方針**:
- 現在の`layout.rs`（517行）を5ファイルに分割
- 現在の`arrangement.rs`（60行）を`layout/systems.rs`にマージ

**依存関係**:
- `layout/systems.rs`は`ecs/common/tree_system.rs`の汎用関数を使用
- `layout/arrangement.rs`はマーカーコンポーネント定義のみ（システムは`systems.rs`へ）

#### Requirement 5: Transform非推奨化
**目標構造**:
```
ecs/transform/
├── mod.rs              # 非推奨警告コメント + 再エクスポート
└── components.rs       # Transform関連コンポーネント（既存transform.rsを改名）
```

**非推奨警告テンプレート** (requirements.mdより):
```rust
//! ⚠️ **Experimental / Deprecated Module**
//!
//! This module contains 2D transform components that were designed to mimic WinUI3's
//! transform system. However, wintf's layout system is based on `Arrangement` and does
//! not require separate transform components in most cases.
//!
//! **Recommendation**: Use `Arrangement`-based layout instead of explicit transforms.
//! This module may be removed in future versions.
```

#### Requirement 6: Common Infrastructure分離
**目標構造**:
```
ecs/common/
├── mod.rs              # 再エクスポート
└── tree_system.rs      # 汎用階層伝播関数（既存ファイルを移動）
```

**影響範囲**:
- `arrangement.rs`の`use crate::ecs::tree_system::*;`を`use crate::ecs::common::tree_system::*;`に変更
- その他のインポート文も同様に修正

### 2.2 ギャップと制約

#### 既存機能のギャップ
- ❌ **Gap**: サブフォルダ構造が未整備（Layout System、Common Infrastructure、Transform）
- ❌ **Gap**: 非推奨警告が未記載（Transform）
- ✅ **Existing**: Component hooks、汎用伝播システム、既存サブフォルダ（Graphics、Widget）

#### アーキテクチャ制約
- ✅ **Constraint**: API互換性維持必須（`pub use`による再エクスポート）
- ✅ **Constraint**: テスト成功維持必須（`cargo test`, サンプルアプリ動作確認）
- ⚠️ **Constraint**: doctestの配置変更（`layout.rs`から分割後の各ファイルへ）

#### 未調査項目（Research Needed）
- ⚠️ **Research**: `arrangement.rs`内のシステム関数は`layout/systems.rs`と`layout/arrangement.rs`のどちらに配置すべきか？
  - **暫定方針**: `layout/systems.rs`に統合（システム関数とコンポーネント定義を分離）
- ⚠️ **Research**: Transform deprecation後の移行ガイドは必要か？
  - **暫定方針**: モジュールレベルdocコメントで`Arrangement`ベースの代替案を提示

---

## 3. 実装アプローチ評価

### Option A: Extend Existing Components ❌
**適用不可**: 既存ファイルの肥大化（`layout.rs` 517行）を解消するのが目的であり、ファイル拡張は逆効果。

### Option B: Create New Components 🔺
**部分的に適用**: 新規ディレクトリ作成（`ecs/layout/`, `ecs/common/`, `ecs/transform/`）は必要だが、既存コンポーネントの移動も伴うため単独では不十分。

### Option C: Hybrid Approach ✅ 推奨
**組み合わせ戦略**:

#### Phase 1: Common Infrastructure分離（リスク: Low）
1. `ecs/common/`ディレクトリ作成
2. `tree_system.rs`を`ecs/common/tree_system.rs`に移動
3. `ecs/common/mod.rs`作成（`pub mod tree_system;`）
4. インポートパス更新（`arrangement.rs`等）
5. `ecs/mod.rs`で`pub use common::tree_system::*;`再エクスポート
6. テスト確認: `cargo test`

**Trade-offs**:
- ✅ 影響範囲が明確（`tree_system.rs`のみ）
- ✅ 他のフェーズと独立
- ❌ `arrangement.rs`のインポート修正が必要

#### Phase 2: Layout System統合（リスク: Medium）
1. `ecs/layout/`ディレクトリ作成
2. `layout.rs`を分割:
   - `layout/taffy.rs`: `BoxStyle`, `BoxComputedLayout`
   - `layout/metrics.rs`: `Size`, `Offset`, `LayoutScale`, `TextLayoutMetrics`
   - `layout/arrangement.rs`: `Arrangement`, `GlobalArrangement`, `ArrangementTreeChanged`
   - `layout/rect.rs`: `Rect`, `D2DRectExt`, `transform_rect_axis_aligned`
3. `arrangement.rs`の内容を`layout/systems.rs`に統合
4. `ecs/layout/mod.rs`作成（各サブモジュール再エクスポート）
5. `ecs/mod.rs`で`pub use layout::*;`維持（APIパス変更なし）
6. doctestの動作確認

**Trade-offs**:
- ✅ レイアウト関連ロジックの一元管理
- ✅ ファイルサイズの適正化（各モジュール100-200行程度）
- ❌ 複数ファイル間の依存関係管理が必要
- ❌ doctest配置の見直し必要

#### Phase 3: Transform非推奨化（リスク: Low）
1. `ecs/transform/`ディレクトリ作成
2. `transform.rs`を`transform/components.rs`に改名・移動
3. `ecs/transform/mod.rs`作成（非推奨警告コメント + 再エクスポート）
4. `ecs/mod.rs`で`pub use transform::*;`維持
5. 将来的な削除候補として`.kiro/steering/tech.md`に記載

**Trade-offs**:
- ✅ 実験的機能の明確な隔離
- ✅ 新規コードでの使用抑止
- ❌ 既存コードへの影響は限定的（警告のみ）

### リスク軽減策
- **Incremental rollout**: Phase 1 → Phase 2 → Phase 3 の順で実施
- **Testing checkpoints**: 各Phase後に`cargo test`, `cargo run --example areka`で動作確認
- **Rollback strategy**: Git履歴で各Phaseをコミット分離し、問題発生時は該当Phaseのみrevert

---

## 4. 実装複雑度とリスク

### Effort: **M (Medium, 3-7 days)**

**内訳**:
- Phase 1 (Common Infrastructure): 0.5日（低リスク、単純移動）
- Phase 2 (Layout System): 3-4日（中リスク、ファイル分割+doctest移行）
- Phase 3 (Transform Deprecation): 0.5日（低リスク、警告追加）
- テスト・検証: 1日（全Phase後の統合テスト）
- ドキュメント更新: 1日（`.kiro/steering/structure.md`、モジュールdocコメント）

**根拠**:
- 既存パターン（`graphics/`, `widget/`サブフォルダ）の踏襲により設計判断は最小限
- API互換性維持により外部利用者への影響なし
- ファイル分割は機械的作業が中心（依存関係分析は必要）

### Risk: **Medium**

**リスク要因**:
1. **Layout System統合の複雑性** (Medium):
   - `layout.rs`（517行）の分割時に依存関係を誤るとコンパイルエラー
   - doctest（4件）の配置ミスで`cargo test`失敗の可能性
   - **軽減策**: 分割前にコンポーネント間の依存グラフを作成、doctestは分割後も元のモジュールパスで動作確認

2. **インポートパス修正漏れ** (Low):
   - `arrangement.rs`等での`tree_system`インポートパス変更漏れ
   - **軽減策**: `cargo check`で即座に検出可能、Phase 1完了時に確認

3. **API互換性維持の見落とし** (Low):
   - `ecs/mod.rs`の`pub use`設定ミスで外部利用者のコードが破壊
   - **軽減策**: サンプルアプリケーション（`areka.rs`, `dcomp_demo.rs`）で動作確認

**全体評価**: 段階的実装とテストチェックポイントにより、リスクはMediumに抑制可能。

---

## 5. 設計フェーズへの推奨事項

### 優先アプローチ
**Option C: Hybrid Approach**（3 Phase戦略）を推奨。

### 主要決定事項
1. **Layout Systemのサブモジュール分割粒度**:
   - `taffy.rs`, `metrics.rs`, `arrangement.rs`, `rect.rs`, `systems.rs`の5ファイル構成
   - 各ファイル100-200行を目安（Single Responsibility Principle遵守）

2. **Transformの非推奨化アプローチ**:
   - `ecs/transform/mod.rs`にRustdoc警告を記載
   - 即座の削除は行わず、ユーザーフィードバック収集後に判断

3. **Common Infrastructureのスコープ**:
   - 現時点では`tree_system.rs`のみを配置
   - 将来的に他の汎用システム（ワークキュー拡張等）追加時の拡張ポイント

### 要調査項目
1. **doctest配置の最終決定**:
   - 分割後のファイル（`layout/metrics.rs`, `layout/rect.rs`）にdoctestを移動
   - または`layout/mod.rs`に集約してパス指定で実行
   - **推奨**: 分割後のファイルに配置（テスト対象コードとの近接性維持）

2. **Transform移行ガイドの必要性**:
   - `Arrangement`ベースのレイアウト例をドキュメント化するか
   - **推奨**: `ecs/transform/mod.rs`のdocコメントに簡単な例を追加

3. **`arrangement.rs`のシステム関数配置**:
   - `layout/systems.rs`に統合（コンポーネント定義と分離）
   - **推奨**: システム関数は`systems.rs`、コンポーネントは`arrangement.rs`に分離

### ドキュメント更新要件
- `.kiro/steering/structure.md`: 5グループ定義とサブフォルダ構造の追記
- 各モジュール先頭の`//!`docコメント: 責務と含まれるコンポーネントの説明
- `ecs/transform/mod.rs`: 非推奨警告と代替手段の明記

---

## 次のステップ

### 設計フェーズへ
Gap Analysisの結果を踏まえ、以下のコマンドで設計ドキュメントを生成してください：

```
/kiro-spec-design ecs-component-grouping
```

または、要件を自動承認して直接設計フェーズに進む場合：

```
/kiro-spec-design ecs-component-grouping -y
```

### 設計フェーズで詳細化すべき項目
1. ファイル分割の具体的な行範囲（`layout.rs`の行1-100は`taffy.rs`へ、等）
2. `pub use`再エクスポートの完全なリスト
3. doctest移行の具体的な手順
4. 各Phase完了時のテスト項目チェックリスト
5. コードレビューポイント（特にAPI互換性検証）

---

**Analysis Status**: ✅ Complete  
**Recommended Approach**: Option C (Hybrid - 3 Phase Implementation)  
**Estimated Effort**: M (3-7 days)  
**Risk Level**: Medium (mitigated by phased approach)
