# Requirements Document

## Project Description (Input)
Visual作成時に自動作成されるコンポーネントの整理

## Terminology (用語定義)
本ドキュメントでは、混乱を避けるために以下の用語定義を使用する。

- **Widget Tree (ウィジットツリー)**: ECSのエンティティ親子関係（`ChildOf` / `Children`）によって構築される論理的な階層構造。アプリケーションの論理的な構造を表す。
- **Visual Tree (ビジュアルツリー)**: DirectCompositionのVisualオブジェクトによって構築される描画用の階層構造。`Visual` コンポーネントを持つエンティティによって構成され、実際に画面に描画される構造を表す。

## Requirements

### Requirement 1: 論理Visualコンポーネントの導入と自動管理
**目的:** 開発者が下位のGPUリソースを意識することなく、`Visual` コンポーネントに基づいてGPUリソース（`VisualGraphics`, `SurfaceGraphics`）を自動的に管理する仕組みを構築する。

#### R1 Acceptance Criteria
1. システムは、ビジュアルツリーのノードを表す `Visual` コンポーネントを導入しなければならない。
2. `Visual` コンポーネントは、可視性、不透明度、変形基準点（transform-origin）などの論理プロパティを保持しなければならない。
3. システムは、ウィンドウのルートエンティティに対して、自動的に `Visual` コンポーネントを付与しなければならない。
4. **Scope Exclusion (スコープ外事項):** アニメーションや頻繁な更新に基づく `Visual` コンポーネントの自動付与ロジックは、本仕様のスコープ外とする（将来的な拡張として想定する）。
5. システムは、`Visual` コンポーネントの有無に基づいて、`VisualGraphics`（GPUリソース）を自動的に作成・破棄しなければならない。
6. `Visual` コンポーネントを持たないエンティティの描画内容は、現状の描画パイプラインと同様に、親の `Visual` のSurfaceに統合して（吸収されて）描画されなければならない。

### Requirement 2: DirectComposition仕様に準拠した柔軟なVisual構成
**目的:** DirectCompositionの柔軟な設計（Surfaceの有無、子Visualの有無が任意）をそのまま活かし、WinUI3のような厳格な型分け（ContainerVisual/SpriteVisual等）を行わず、シンプルかつ柔軟な構成を実現する。

#### R2 Acceptance Criteria
1. `Visual` コンポーネントは、構造上は `SurfaceGraphics` を持たないことも許容される設計とする（DirectCompositionの仕様に準拠し、将来的な最適化を妨げないため）。
2. しかし、本仕様のスコープでは、`Visual` コンポーネントを持つすべてのエンティティに対して、無条件で自動的に `SurfaceGraphics` を作成しなければならない。
3. **Scope Exclusion (スコープ外事項):** 本来は、自身または子孫の `GraphicsCommandList` の有無や要求サイズを集約して `SurfaceGraphics` の生成要否やサイズを決定すべきであるが、この最適化処理は複雑であるため、本仕様ではスコープ外とし、一律作成とする。

### Requirement 3: Visual階層の同期 [Out of Scope]
**目的:** 開発者がVisualの親子関係を手動で管理しなくて済むように、ビジュアルツリーをウィジットツリー（ECSエンティティ階層）に自動的に反映させる。

#### R3 Acceptance Criteria
1. **Scope Exclusion (スコープ外事項):** 本要件は今回のリファクタリングのスコープ外とする。Visualツリーの構築仕様にて別途検討を行う。
2. （参考）`Visual` を持つエンティティの子エンティティ（ウィジットツリー上の子）に `Visual` コンポーネントが追加された場合、システムは自動的に子Visualを親Visualの子リスト（ビジュアルツリー）に追加しなければならない。
3. （参考）`Visual` を持つエンティティがウィジットツリー内で親を変更された場合、システムはビジュアルツリーを一致するように更新しなければならない。
4. （参考）システムは、階層内の「ギャップ」（例: 別のVisualエンティティの孫にあたるVisualエンティティで、間にVisualを持たないエンティティが存在する場合）を処理し、ウィジットツリー上の構造に基づいてもっとも近いVisualの祖先を正しく解決しなければならない。

### Requirement 4: リソースの初期化と復旧
**目的:** 初期化時やデバイスロスト時にアプリケーションが安定して動作するように、基盤となるCOMリソースの堅牢なハンドリングを提供する。

#### R4 Acceptance Criteria
1. `Visual` が追加された際、システムは現在の `GraphicsCore` から作成された有効な `IDCompositionVisual3` インスタンスで `VisualGraphics` を初期化しなければならない。
2. グラフィックスデバイスがロストした場合、システムは `VisualGraphics` および `SurfaceGraphics` を無効としてマークしなければならない。
3. グラフィックスデバイスが復旧した際、システムは論理的な `Visual` およびコンテンツコンポーネントに基づいてCOMリソースを再作成しなければならない。

### Requirement 5: 既存実装の移行と互換性維持
**目的:** 既存のコードベースにおいて直接 `VisualGraphics` を生成している箇所を、新しい `Visual` コンポーネントを使用するようにリファクタリングし、既存の描画動作を維持する。

#### R5 Acceptance Criteria
1. システム初期化時（ウィンドウ作成時など）に `VisualGraphics` を直接生成している既存のコードを特定し、`Visual` コンポーネントを付与する形式に変更しなければならない。
2. この変更により、Requirement 1および2のプロセスを通じて `VisualGraphics` と `SurfaceGraphics` が間接的に生成され、描画パイプラインが正常に機能することを確認しなければならない。
3. リファクタリング前後で、アプリケーションの描画結果や挙動に変化がないこと（リグレッションがないこと）を保証しなければならない。

## Investigation Items (調査項目)

### Investigation 1: GPUリソース生成戦略（ライフタイムイベント vs システム）
**背景:** `Visual` コンポーネントの追加をトリガーとして `VisualGraphics` / `SurfaceGraphics` を生成する際、Bevy ECSのコンポーネントライフタイムイベント（Hooks/Observers）を利用して即座に反映するのが理想的である。しかし、GPUリソースの生成には `GraphicsCore` へのアクセスが必須となる。

**調査内容:**
1. Bevy ECSのコンポーネントライフタイムイベント（`on_add` 等）内で、`GraphicsCore` リソースに安全にアクセスし、GPUリソースを生成することが技術的に可能か、また設計として適切か調査する。
2. `Visual` コンポーネントの追加を検知してGPUリソースを生成する専用のシステム（System）を設計・実装する。
3. **比較検討:** 上記1（Hooks/Observers）と2（Systems）の両方の方式を実装し、スレッド安全性、パフォーマンス、コードの複雑さの観点から比較検討を行う。実装フェーズにて実験的に両方を試し、最適なアプローチを選定する。
