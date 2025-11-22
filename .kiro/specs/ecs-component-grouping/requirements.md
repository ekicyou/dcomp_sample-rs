# Requirements Document

## Project Description (Input)
ECSのコンポーネントが乱雑になってきたのでグループ別に整理したい。まずはどんな機能グループがあるかを要件定義し、要件に従ってリファクタリングしたい。

## Introduction
wintfライブラリのECSコンポーネントは、現在`crates/wintf/src/ecs/`配下に分散配置されている。コンポーネント数の増加に伴い、責務や用途に応じた明確なグルーピングと構造化が必要となっている。本仕様では、既存コンポーネントを機能グループごとに分類し、モジュール構造を整理するためのリファクタリング方針を定義する。

## Requirements

### Requirement 1: コンポーネント機能グループの定義
**Objective:** 開発者として、ECSコンポーネントの責務と用途が一目で理解できるよう、機能グループを明確に定義したい。そうすることで、新規コンポーネントの配置場所が自明になり、コードベースの保守性が向上する。

#### Acceptance Criteria
1. wintfシステムは、以下の5つの主要な機能グループを定義すること:
   - **ウィンドウ管理グループ** (`ecs/window.rs`): ウィンドウの生成、プロパティ、ライフサイクルに関連するコンポーネント
   - **グラフィックスリソースグループ** (`ecs/graphics/`): GPU/CPUリソース、DirectComposition/Direct2D/DirectWriteリソースのラッパー
   - **レイアウトシステムグループ** (`ecs/layout/`): 位置・サイズ計算、taffyレイアウトエンジン連携、配置情報、階層的伝播
   - **ウィジェットグループ** (`ecs/widget/`): UIエレメントの論理表現（Label、Rectangle、Button等）
   - **共通インフラグループ** (`ecs/common/`): 汎用的な階層構造伝播システム、ワークキュー
2. 各機能グループには、グループの責務と含まれるコンポーネントタイプが文書化されること
3. wintfシステムは、命名規則として`XxxGraphics`（GPUリソース）、`XxxResource`（CPUリソース）サフィックスを使用すること
4. When サブシステムが大規模化する時、wintfシステムは該当グループをサブフォルダ構造に分割すること（例: `ecs/layout/`）

### Requirement 2: ウィンドウ管理グループの分類
**Objective:** 開発者として、ウィンドウ関連のコンポーネントが`window.rs`に集約されていることを確認したい。そうすることで、ウィンドウプロパティやライフサイクル管理のコードを素早く特定できる。

#### Acceptance Criteria
1. wintfシステムは、以下のコンポーネントをウィンドウ管理グループに分類すること:
   - `Window`: ウィンドウ作成パラメータ
   - `WindowHandle`: 作成済みウィンドウのHWND、HINSTANCE、初期DPI
   - `WindowStyle`: ウィンドウスタイル（WS_*、WS_EX_*）
   - `WindowPos`: ウィンドウの位置・サイズ・Z-order
   - `DpiTransform`: DPI変換行列
   - `ZOrder`: Z-order設定方法（列挙型）
2. When 新しいウィンドウ関連機能が追加される時、wintfシステムは該当コンポーネントを`ecs/window.rs`に配置すること

### Requirement 3: グラフィックスリソースグループの分類
**Objective:** 開発者として、GPU/CPUリソースコンポーネントが`graphics/components.rs`に配置されていることを確認したい。そうすることで、デバイスロスト対応やリソース初期化ロジックを一元管理できる。

#### Acceptance Criteria
1. wintfシステムは、以下のコンポーネントをグラフィックスリソースグループに分類すること:
   - `WindowGraphics`: ウィンドウレベルGPUリソース（IDCompositionTarget、ID2D1DeviceContext）
   - `VisualGraphics`: ビジュアルノードGPUリソース（IDCompositionVisual3）
   - `SurfaceGraphics`: 描画サーフェスGPUリソース（IDCompositionSurface）
   - `TextLayoutResource`: テキストレイアウトCPUリソース（IDWriteTextLayout）
   - `HasGraphicsResources`: 静的マーカー
   - `GraphicsNeedsInit`: 動的マーカー
   - `SurfaceUpdateRequested`: サーフェス更新要求マーカー
   - `Visual`: ビジュアルツリー論理表現
2. wintfシステムは、GPUリソースコンポーネント（`XxxGraphics`）に`invalidate()`、`is_valid()`、`generation()`メソッドを実装すること
3. wintfシステムは、コマンドリスト関連コンポーネント（`GraphicsCommandList`）を`graphics/command_list.rs`に配置すること

### Requirement 4: レイアウトシステムグループの統合
**Objective:** 開発者として、レイアウト計算・配置伝播・矩形操作が`ecs/layout/`配下に統合されていることを確認したい。そうすることで、レイアウトエンジン連携、階層的配置計算、ジオメトリ処理を一箇所で管理できる。

#### Acceptance Criteria
1. wintfシステムは、`ecs/layout/`をサブフォルダ構造として以下のモジュールに分割すること:
   - `layout/taffy.rs`: taffyレイアウトエンジン連携（`BoxStyle`, `BoxComputedLayout`）
   - `layout/metrics.rs`: レイアウトメトリクス（`Size`, `Offset`, `LayoutScale`, `TextLayoutMetrics`）+ doctestを含む
   - `layout/arrangement.rs`: 配置伝播コンポーネント（`Arrangement`, `GlobalArrangement`, `ArrangementTreeChanged`）
   - `layout/rect.rs`: 矩形操作（`Rect`型エイリアス, `D2DRectExt`トレイト, `transform_rect_axis_aligned`関数）+ doctestを含む
   - `layout/systems.rs`: レイアウト計算システム、配置伝播システム（現在の`arrangement.rs`の内容を統合）
2. wintfシステムは、現在の`layout.rs`（517行）を上記5ファイルに分割し、各ファイルを100-200行程度に適正化すること
3. wintfシステムは、現在の`arrangement.rs`（60行）のシステム関数を`layout/systems.rs`に統合すること
4. wintfシステムは、`layout/systems.rs`が`ecs/common/tree_system.rs`の汎用関数（`sync_simple_transforms`, `mark_dirty_trees`, `propagate_parent_transforms`）を使用すること
5. wintfシステムは、`ecs/layout/mod.rs`から各サブモジュールを`pub use`で再エクスポートすること
6. wintfシステムは、doctestを分割後のファイル（`metrics.rs`, `rect.rs`）に配置し、`cargo test`で4つのdoctestが成功すること

### Requirement 5: 変換行列の実験的配置と非推奨化
**Objective:** 開発者として、2D変換コンポーネントが実験的機能であり、将来的な削除候補であることを明確に理解したい。そうすることで、新規コードでの使用を避け、代替手段（`Arrangement`ベースのレイアウト）を優先できる。

#### Acceptance Criteria
1. wintfシステムは、`ecs/transform/`をサブフォルダ構造として以下のように構成すること:
   - `transform/mod.rs`: 非推奨警告コメント + サブモジュール宣言 + 再エクスポート
   - `transform/components.rs`: 変換コンポーネント実装（現在の`transform.rs`を改名）
2. wintfシステムは、現在の`transform.rs`（191行）を`transform/components.rs`に移動すること
3. wintfシステムは、`ecs/transform/mod.rs`の先頭に以下の非推奨警告を記載すること:
   ```rust
   //! ⚠️ **Experimental / Deprecated Module**
   //!
   //! This module contains 2D transform components that were designed to mimic WinUI3's
   //! transform system. However, wintf's layout system is based on `Arrangement` and does
   //! not require separate transform components in most cases.
   //!
   //! **Recommendation**: Use `Arrangement`-based layout instead of explicit transforms.
   //! This module may be removed in future versions.
   //!
   //! # Migration Guide
   //!
   //! Instead of using `Transform` components, use `Arrangement` and `GlobalArrangement`
   //! to position UI elements within the layout tree. The arrangement system integrates
   //! with the taffy layout engine and provides hierarchical position propagation.
   ```
4. When 変換コンポーネントが使用される時、wintfシステムは`TransformTreeChanged`マーカーを伝播させること（後方互換性のため）
5. wintfシステムは、`ecs/mod.rs`から`pub use transform::*;`で再エクスポートすること（API互換性維持）

### Requirement 6: 共通インフラグループの分離
**Objective:** 開発者として、汎用的な階層構造伝播システムが`ecs/common/`配下に配置されていることを確認したい。そうすることで、レイアウト・変換以外のドメインでも階層伝播パターンを再利用できる。

#### Acceptance Criteria
1. wintfシステムは、`ecs/common/`をサブフォルダ構造として以下のように構成すること:
   - `common/mod.rs`: サブモジュール宣言 + 再エクスポート
   - `common/tree_system.rs`: 汎用階層伝播システム（現在の`tree_system.rs`を移動）
2. wintfシステムは、現在の`tree_system.rs`（371行）を`common/tree_system.rs`に移動すること
3. wintfシステムは、`ecs/common/tree_system.rs`に以下の汎用システムを維持すること:
   - `sync_simple_transforms<L, G, M>()`: 階層に属さないエンティティの更新
   - `mark_dirty_trees<L, G, M>()`: ダーティビットの祖先への伝播
   - `propagate_parent_transforms<L, G, M>()`: 親から子へのトランスフォーム伝播
   - `WorkQueue`: 並列処理ワークキュー
   - `NodeQuery<L, G, M>`: ノードクエリ型エイリアス
4. wintfシステムは、`tree_system.rs`のジェネリック関数が以下のドメインで再利用されること:
   - レイアウトシステム: `Arrangement`/`GlobalArrangement`の伝播（`layout/systems.rs`から使用）
   - 変換システム: `Transform`/`GlobalTransform`の伝播（実験的、`transform/`から使用）
5. wintfシステムは、`ecs/common/mod.rs`から`tree_system`モジュールを公開すること
6. wintfシステムは、`ecs/mod.rs`から`pub use common::tree_system::*;`で再エクスポートすること（API互換性維持）
7. wintfシステムは、`layout/systems.rs`および他のモジュールのインポートパスを`use crate::ecs::common::tree_system::*;`に更新すること

### Requirement 7: ウィジェットグループの現状維持
**Objective:** 開発者として、UIエレメントの論理表現が`ecs/widget/`配下に既に整理されていることを確認したい。そうすることで、新規ウィジェットの追加場所が明確になる。

#### Acceptance Criteria
1. wintfシステムは、以下のサブモジュールをウィジェットグループに維持すること:
   - `widget/text/`: テキスト関連ウィジェット（`Label`等）
   - `widget/shapes/`: 図形ウィジェット（`Rectangle`等）
2. wintfシステムは、ウィジェットコンポーネントに論理プロパティ（テキスト内容、色、サイズ等）を保持させること
3. When 新しいウィジェットタイプが追加される時、wintfシステムは適切なサブモジュール配下にコンポーネントを配置すること
4. wintfシステムは、既存の`ecs/widget/`構造を変更しないこと（今回のリファクタリング対象外）

### Requirement 8: モジュール構造の一貫性
**Objective:** 開発者として、モジュール構造が`.kiro/steering/structure.md`の組織哲学に準拠していることを確認したい。そうすることで、プロジェクト全体の設計原則との整合性が保たれる。

#### Acceptance Criteria
1. wintfシステムは、ECSコンポーネントレイヤー（`ecs/`）をCOMラッパーレイヤー（`com/`）とメッセージハンドリングレイヤー（ルート）から明確に分離すること
2. wintfシステムは、ファイル名に`snake_case`、型名に`PascalCase`、関数名に`snake_case`を使用すること
3. wintfシステムは、各モジュールが独立してテスト可能な単位として設計されていること

### Requirement 9: リファクタリングの安全性保証
**Objective:** 開発者として、リファクタリング後も既存の動作が維持されることを確認したい。そうすることで、回帰バグのリスクを最小化できる。

#### Acceptance Criteria
1. When コンポーネントがモジュール間で移動される時、wintfシステムは`ecs/mod.rs`の`pub use`による再エクスポートでAPI互換性を維持すること
2. wintfシステムは、以下のフェーズごとにテストを実行し、成功することを確認すること:
   - Phase 1完了後（Common Infrastructure分離）: `cargo check`, `cargo test`
   - Phase 2完了後（Layout System統合）: `cargo check`, `cargo test`（4つのdoctest含む）
   - Phase 3完了後（Transform非推奨化）: `cargo check`, `cargo test`
3. wintfシステムは、最終的に以下の動作確認テストが成功すること:
   - `cargo test`: 全doctestおよび単体テストが成功
   - `cargo run --example areka`: サンプルアプリケーションが正常に起動・動作
   - `cargo run --example dcomp_demo`: デモアプリケーションが正常に起動・動作
4. wintfシステムは、各Phaseを個別のGitコミットとして記録し、問題発生時にロールバック可能にすること

### Requirement 10: ドキュメント更新
**Objective:** 開発者として、コンポーネントグルーピングの方針がドキュメント化されていることを確認したい。そうすることで、チームメンバーが一貫した分類基準に従って開発できる。

#### Acceptance Criteria
1. wintfシステムは、`.kiro/steering/structure.md`に各機能グループの責務と代表的なコンポーネント例を記載すること
2. wintfシステムは、各モジュールファイルの先頭にモジュールレベルのドキュメントコメント（`//!`）を追加すること
3. When 新規コンポーネントが追加される時、開発者はドキュメントを参照して適切な機能グループに配置できること
