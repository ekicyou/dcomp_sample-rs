# Research & Design Decisions

---
**Feature**: ecs-component-grouping  
**Discovery Scope**: Extension（既存システムのリファクタリング）  
**Key Findings**:
- 既存の`graphics/`, `widget/`サブフォルダパターンを踏襲することで設計判断を最小化
- `tree_system.rs`の汎用ジェネリック関数は型パラメータ`<L, G, M>`で複数ドメインに対応
- API互換性は`ecs/mod.rs`の`pub use`による再エクスポートで維持可能

---

## Summary

このリファクタリングは、既存のECSコンポーネント構造を5つの機能グループに再編成し、サブフォルダ構造を導入することで保守性を向上させる**Extension**タイプの変更です。新規ライブラリの追加はなく、既存のbevy_ecs 0.17.2とtaffy 0.9.1を引き続き使用します。

**Discovery Type**: Light Discovery（既存パターン分析中心）

**主要な発見事項**:
1. **既存サブフォルダパターンの存在**: `graphics/`, `widget/`がすでにサブフォルダ化されており、同様のパターンを`layout/`, `common/`, `transform/`に適用可能
2. **汎用伝播システムの再利用**: `tree_system.rs`のジェネリック関数（`sync_simple_transforms<L, G, M>`等）は型パラメータにより、Layout SystemとTransform Systemの両方で利用可能
3. **API互換性維持の実績**: `ecs/mod.rs`の`pub use`パターンにより、外部利用者はインポートパス変更なしに新構造を利用可能

## Research Log

### 既存サブフォルダパターンの分析

**Context**: Layout System、Common Infrastructure、Transformのサブフォルダ化に際し、既存の成功パターンを調査。

**Sources Consulted**:
- `crates/wintf/src/ecs/graphics/mod.rs` - 既存サブフォルダ構造
- `crates/wintf/src/ecs/widget/mod.rs` - 既存サブフォルダ構造
- `crates/wintf/src/ecs/mod.rs` - 再エクスポートパターン

**Findings**:
- `graphics/`は6ファイル構成（mod.rs, components.rs, core.rs, systems.rs, command_list.rs, visual_manager.rs）
- `widget/`は3ファイル構成（mod.rs, text/, shapes/）のサブディレクトリ階層
- 両者とも`ecs/mod.rs`で`pub use graphics::*;`, `pub use widget::*;`により再エクスポート
- `Component`トレイトの実装はサブモジュール内で完結し、外部からは`use wintf::ecs::*;`で透過的にアクセス可能

**Implications**:
- Layout Systemは`graphics/`と同様の5ファイル構成（taffy.rs, metrics.rs, arrangement.rs, rect.rs, systems.rs）を採用
- Transformは`widget/`と同様のシンプルな2ファイル構成（mod.rs, components.rs）で隔離
- Common Infrastructureは2ファイル構成（mod.rs, tree_system.rs）で汎用性を維持

### tree_system.rsの汎用性分析

**Context**: `tree_system.rs`を`ecs/common/`に移動する際、複数ドメインでの再利用性を検証。

**Sources Consulted**:
- `crates/wintf/src/ecs/tree_system.rs` - 汎用関数の実装（371行）
- `crates/wintf/src/ecs/arrangement.rs` - Layout Systemでの使用例（60行）

**Findings**:
- `sync_simple_transforms<L, G, M>()`, `mark_dirty_trees<L, G, M>()`, `propagate_parent_transforms<L, G, M>()`の3関数は型パラメータにより完全に汎用化
- 型制約：`L: Component + Copy + Into<G>`, `G: Component + Copy + PartialEq + Mul<L, Output = G>`, `M: Component`
- 現在の利用例：
  - Layout: `Arrangement`（L）, `GlobalArrangement`（G）, `ArrangementTreeChanged`（M）
  - Transform: `Transform`（L）, `GlobalTransform`（G）, `TransformTreeChanged`（M）
- `WorkQueue`と`NodeQuery<L, G, M>`型エイリアスも汎用的に設計

**Implications**:
- `ecs/common/tree_system.rs`として配置することで、ドメイン非依存の共通インフラとして明確化
- 将来的に他のドメイン（例: Visibility伝播、Input伝播）でも再利用可能
- インポートパス変更（`use crate::ecs::tree_system::*;` → `use crate::ecs::common::tree_system::*;`）は機械的に実施可能

### doctestの配置戦略

**Context**: `layout.rs`内の4つのdoctestを分割後のファイルに適切に配置する必要性。

**Sources Consulted**:
- `cargo test`出力 - 現在の4つのdoctest実行結果
- `crates/wintf/src/ecs/layout.rs` - doctestの配置箇所（lines 83, 319, 368周辺）

**Findings**:
- 現在のdoctest配置:
  1. `D2DRectExt`トレイト（line 83周辺） - 矩形操作の例
  2. `GlobalArrangement`コンポーネント（line 319周辺） - 配置情報の例
  3. `transform_rect_axis_aligned`関数（line 368周辺） - 矩形変換の例
  4. 4つ目はSize等のメトリクス関連と推測
- Rustdocはファイル単位でdoctestを実行し、モジュールパスは自動的に調整

**Implications**:
- `D2DRectExt`および`transform_rect_axis_aligned`のdoctestは`layout/rect.rs`に配置
- `GlobalArrangement`のdoctestは`layout/arrangement.rs`に配置
- `Size`, `Offset`等のメトリクスdoctestは`layout/metrics.rs`に配置
- 分割後も`cargo test`で4つのdoctestが実行されることを検証

### API互換性維持の検証

**Context**: 外部利用者（サンプルアプリケーション、tests/）が既存のインポートパスで動作し続けることを保証。

**Sources Consulted**:
- `crates/wintf/src/ecs/mod.rs` - 現在の`pub use`パターン
- `crates/wintf/examples/areka.rs`, `examples/dcomp_demo.rs` - 外部利用例

**Findings**:
- 現在の`pub use`構造:
  ```rust
  pub use arrangement::*;
  pub use layout::*;
  pub use transform::*;
  pub use tree_system::*;
  ```
- サンプルアプリケーションでは`use wintf::ecs::*;`または`use wintf::ecs::{具体的な型名};`でインポート
- `pub use`による再エクスポートにより、モジュール構造変更は外部から透過

**Implications**:
- リファクタリング後も`ecs/mod.rs`の`pub use`を維持:
  ```rust
  pub use layout::*;          // layout/配下のすべてを再エクスポート
  pub use transform::*;       // transform/配下のすべてを再エクスポート
  pub use common::tree_system::*; // common/tree_system配下のすべてを再エクスポート
  ```
- 外部利用者は`use wintf::ecs::Arrangement;`のようにサブフォルダを意識せずアクセス可能
- コンパイルエラーは発生せず、API互換性が完全に維持される

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| Flat Structure（現状） | すべてのコンポーネントを`ecs/`直下に配置 | シンプル、学習コストなし | ファイル数増加で可読性低下（layout.rs 517行） | 現状の課題を解決するためリファクタリング実施 |
| Domain Subfolder | 機能グループごとにサブフォルダ化（`layout/`, `common/`等） | 関連コンポーネントの明確な集約、ファイルサイズ適正化 | ディレクトリ階層が1段階深くなる | **採用**。既存の`graphics/`, `widget/`パターンと一貫性あり |
| Module per Component | 各コンポーネントを個別ファイル化 | 最大の分離、単一責任原則の徹底 | ファイル数過多、小ファイル管理の煩雑さ | 過剰な分割。関連コンポーネントはグループ化が適切 |

**選択理由**: Domain Subfolderパターンは、既存の成功事例（`graphics/`, `widget/`）があり、ファイルサイズの適正化（100-200行/ファイル）と可読性向上を両立。wintfのレイヤードアーキテクチャ哲学とも整合。

## Design Decisions

### Decision: Layout Systemのサブモジュール分割粒度

**Context**: 現在の`layout.rs`（517行）を複数ファイルに分割する際の粒度決定。

**Alternatives Considered**:
1. **3ファイル分割** - taffy.rs, metrics.rs, arrangement+rect+systems.rs
2. **5ファイル分割** - taffy.rs, metrics.rs, arrangement.rs, rect.rs, systems.rs
3. **7ファイル分割** - 上記5ファイル + taffy_systems.rs + arrangement_systems.rs

**Selected Approach**: 5ファイル分割（Option 2）

**Rationale**:
- 各ファイルが明確な責務を持つ（Single Responsibility Principle）
- ファイルサイズが100-200行程度に収まり、可読性が向上
- taffyレイアウトエンジン連携、メトリクス定義、配置伝播、矩形操作、システム関数という5つの明確なドメインに対応
- 既存の`graphics/`（6ファイル）と同程度の粒度で一貫性あり

**Trade-offs**:
- **Benefits**: 関連コンポーネントの発見が容易、テスト対象の明確化、doctest配置の自然な分散
- **Compromises**: インポート文が増加（各ファイル間の依存関係が明示的になる）、ファイル数増加によるディレクトリ構造の複雑化

**Follow-up**: 実装時に各ファイルの行数を確認し、200行を大幅に超える場合は再分割を検討

### Decision: Transformの非推奨化アプローチ

**Context**: Transform系コンポーネントは実験的機能（WinUI3模倣）であり、wintfの主要なレイアウトシステムである`Arrangement`と重複。

**Alternatives Considered**:
1. **即座に削除** - transform.rsを完全に除去
2. **非推奨警告のみ** - 現状維持でdocコメントに警告追加
3. **サブフォルダ隔離+非推奨警告** - ecs/transform/配下に移動し、明確な警告とMigration Guideを提供

**Selected Approach**: サブフォルダ隔離+非推奨警告（Option 3）

**Rationale**:
- 即座の削除は既存コード（もしあれば）への影響が大きく、移行時間を確保できない
- 現状維持は実験的機能としての位置付けが不明確で、新規開発者が誤用する可能性
- サブフォルダ隔離により「この機能は特殊」というシグナルを明確化
- 非推奨警告+Migration Guideにより、ユーザーは代替手段（`Arrangement`）を理解可能

**Trade-offs**:
- **Benefits**: 段階的な移行が可能、既存コードの破壊を回避、明確な非推奨メッセージによる新規使用の抑止
- **Compromises**: 完全削除までの移行期間中、コードベースに非推奨コードが残存、`ecs/mod.rs`での再エクスポートが必要（API互換性維持のため）

**Follow-up**: 
- リファクタリング完了後、wintfの実際の使用状況を確認
- 6ヶ月程度の猶予期間後、Transform削除の可否を再評価
- 削除時は`ecs/transform/`ディレクトリごと除去し、`ecs/mod.rs`の`pub use transform::*;`を削除

### Decision: Common Infrastructureのスコープ

**Context**: `tree_system.rs`を`ecs/common/`に配置する際、将来的な拡張可能性を考慮。

**Alternatives Considered**:
1. **tree_system.rs専用ディレクトリ** - `ecs/tree_system/`として独立配置
2. **汎用インフラディレクトリ** - `ecs/common/`として将来の汎用システム追加を想定
3. **layoutおよびtransform配下** - 各ドメインに`tree_system.rs`をコピー配置

**Selected Approach**: 汎用インフラディレクトリ（Option 2）

**Rationale**:
- `tree_system.rs`の型パラメータ`<L, G, M>`は完全に汎用的で、Layout、Transform以外のドメインでも利用可能
- 将来的に他の汎用システム（例: 並列処理ユーティリティ、イベント伝播システム）を追加する可能性
- `ecs/common/`という命名により、「ドメイン非依存の共通基盤」という意図が明確

**Trade-offs**:
- **Benefits**: 将来の拡張性、汎用性の明確化、コード重複の回避
- **Compromises**: 現時点では`tree_system.rs`のみのディレクトリとなる、`common`という抽象的な名前がやや曖昧

**Follow-up**:
- 将来的にワークキュー拡張、汎用イベントシステム等が必要になった場合、`ecs/common/`配下に追加
- `tree_system.rs`以外のファイルが追加されない場合でも、汎用性を示す命名として維持

## Risks & Mitigations

**Risk 1: Layout Systemファイル分割時の依存関係ミス**
- **影響**: コンパイルエラー、doctestの失敗
- **Mitigation**: 
  - 分割前に依存グラフを作成（taffy → metrics → arrangement → rect → systems）
  - Phase 2完了時に`cargo check`および`cargo test`で即座に検証
  - 各ファイルのインポート文を明示的に記述（`use crate::ecs::layout::metrics::*;`等）

**Risk 2: インポートパス変更漏れ**
- **影響**: `arrangement.rs`や他モジュールで`tree_system`のインポートパス変更が漏れるとコンパイルエラー
- **Mitigation**:
  - Phase 1（Common Infrastructure分離）完了時に`cargo check`で即座に検出
  - 変更箇所は`arrangement.rs`の1ヶ所のみと特定済み
  - `use crate::ecs::tree_system::*;` → `use crate::ecs::common::tree_system::*;`の置換を確実に実施

**Risk 3: doctest配置ミスによるテスト失敗**
- **影響**: 分割後にdoctestが実行されない、またはモジュールパスエラー
- **Mitigation**:
  - 分割時に各doctestをコピー&ペーストではなく、元ファイルからカット&ペーストで移動
  - Phase 2完了時に`cargo test`で4つのdoctestすべてが成功することを確認
  - doctestのモジュールパス（`use wintf::ecs::*;`等）が正しいことを検証

**Risk 4: API互換性の見落とし**
- **影響**: 外部利用者（サンプルアプリ、tests/）のコードが破壊
- **Mitigation**:
  - `ecs/mod.rs`の`pub use`パターンを厳密に維持
  - 最終検証で`cargo run --example areka`および`cargo run --example dcomp_demo`を実行
  - エラーメッセージが「型が見つからない」等のインポート関連であれば、即座に`pub use`を修正

## References

- [bevy_ecs 0.17.2 Documentation](https://docs.rs/bevy_ecs/0.17.2/bevy_ecs/) - ECS architecture foundation
- [taffy 0.9.1 Documentation](https://docs.rs/taffy/0.9.1/taffy/) - Layout engine
- [Rust API Guidelines - Re-exports](https://rust-lang.github.io/api-guidelines/flexibility.html#c-reexport) - `pub use`パターンのベストプラクティス
- wintf Steering Context:
  - `.kiro/steering/structure.md` - レイヤードアーキテクチャ哲学、命名規則
  - `.kiro/steering/tech.md` - ECSアーキテクチャ詳細、Component hooks
  - `.kiro/steering/product.md` - wintf概要、ターゲットユースケース
