# Requirements: com-resource-naming-unification

**Feature ID**: `com-resource-naming-unification`  
**Created**: 2025-11-17  
**Status**: Requirements Generated

---

## Introduction

本要件定義は、COMリソース（Direct2D、DirectComposition、DirectWrite）を保持するECSコンポーネントの命名規則を統一する`com-resource-naming-unification`機能の詳細要件を定義する。

現在、COMリソースを保持するコンポーネントには以下のような命名の不統一が存在する：

- `WindowGraphics` (D2D1DeviceContext + IDCompositionTarget) - ウィンドウレベル
- `Visual` (IDCompositionVisual3) - ウィジェットレベル（現在はWindow直下だが将来は個別Entity）
- `Surface` (IDCompositionSurface) - ウィジェットレベル（現在はWindow直下だが将来は個別Entity）

この不統一は、将来追加されるテキスト関連コンポーネント（`TextLayout`、`TextFormat`など）やブラシ・ジオメトリなどの描画リソースコンポーネントとの整合性を欠く。統一的な命名規則により、コードベースの可読性と保守性を向上させる。

### 現在の実装状況と将来の設計

**現状**: ビジュアルツリーが未実装のため、`Visual`と`Surface`はWindowエンティティに直接アタッチされている。

**将来の設計**: 
- 各ウィジェット（TextBlock、Image、Container等）が独立したEntityとして存在
- 各ウィジェットEntityが独自の`Visual`/`Surface`コンポーネントを持つ
- Windowはルートエンティティとして`WindowGraphics`のみを保持
- ビジュアルツリーは論理ツリーと独立して構築される（1:1対応ではない）

本仕様での改名は、この将来設計を前提とした命名規則への移行である。

### 命名規則の原則

**論理コンポーネント vs COMリソースコンポーネント**:
- **論理コンポーネント**: アプリケーションレベルの概念（例: `Label`, `Rectangle`, `Button`）
- **COMリソースコンポーネント**: Windows COMオブジェクトのラッパー（例: `LabelGraphics`, `RectangleGraphics`）

**GPUリソース vs CPUリソース**:
- **GPUリソース**: Direct3D/Direct2D/DirectCompositionデバイスに依存し、デバイスロスト時に再生成が必要
  - 例: `ID2D1DeviceContext`, `IDCompositionTarget`, `IDCompositionVisual3`, `IDCompositionSurface`, `ID2D1Bitmap`, `ID2D1SolidColorBrush`
  - 特徴: `invalidate()`メソッドと`generation`フィールドを持ち、再初期化システムで管理される
- **CPUリソース**: デバイスに依存せず、一度生成すれば永続的に使用可能
  - 例: `IDWriteTextFormat`, `IDWriteTextLayout`, `ID2D1PathGeometry`（デバイス非依存ジオメトリ）
  - 特徴: 通常の参照カウント管理のみで十分、再初期化の対象外

本仕様では、以下の命名規則を適用する：

1. **GPUリソース**: `XxxGraphics`サフィックス（デバイスロスト対応が必要）
2. **CPUリソース**: `XxxResource`サフィックス（永続的なリソース）

これにより：

1. 論理コンポーネントとCOMリソースの対応関係が明確になる
2. GPUリソースとCPUリソースの区別が命名から判断できる
3. デバイスロスト時の再初期化対象が一目で識別できる
4. 将来の機能拡張（テキスト、ブラシ、ジオメトリなど）でも一貫した命名が可能

---

## Requirements

### Requirement 1: ウィンドウレベルCOMリソースの命名維持

**Objective:** システム開発者として、ウィンドウ単位で管理されるCOMリソースコンポーネントの命名が規則に準拠していることを確認したい。これにより、ウィンドウ関連グラフィックスリソースの責務が明確になる。

#### Acceptance Criteria (Window Level)

1. The wintfシステムは、`WindowGraphics`コンポーネント名を維持しなければならない（すでに規則に準拠）
2. The wintfシステムは、`WindowGraphics`がウィンドウレベルのGPUリソース（D2D1DeviceContext + IDCompositionTarget）を保持することを明確にしなければならない
3. The wintfシステムは、`WindowGraphics`コンポーネントがWindowエンティティにのみアタッチされることを保証しなければならない
4. The wintfシステムは、改名後もCOMオブジェクトへのアクセスメソッド名（`target()`, `device_context()`）を維持しなければならない

---

### Requirement 2: ウィジェットレベルCOMリソースの命名統一

**Objective:** システム開発者として、ウィジェット（ビジュアルツリー要素）レベルで管理されるCOMリソースコンポーネントの命名を統一したい。これにより、将来のビジュアルツリー実装に備えた一貫性が確保される。

#### Acceptance Criteria (Widget Level)

1. The wintfシステムは、`Visual`コンポーネントを`VisualGraphics`に改名しなければならない
2. The wintfシステムは、`Surface`コンポーネントを`SurfaceGraphics`に改名しなければならない
3. When コンポーネント改名が実施された時、wintfシステムはすべての参照箇所（システム、クエリ、型注釈）を更新しなければならない
4. The wintfシステムは、改名後も`VisualGraphics`と`SurfaceGraphics`がウィジェットレベルのGPUリソースであることをドキュメントで明記しなければならない
5. The wintfシステムは、改名後もCOMオブジェクトへのアクセスメソッド名（`visual()`, `surface()`）を維持しなければならない

**Objective:** システム開発者として、将来追加されるウィジェット固有のCOMリソースコンポーネント（Label、Rectangle等）の命名規則を定義したい。これにより、一貫した拡張が可能になる。

#### Acceptance Criteria (Widget Resources)

1. When 新しいウィジェット用GPUリソースコンポーネントが追加される時、wintfシステムは`{LogicalComponent}Graphics`命名規則に従わなければならない
2. The wintfシステムは、以下のGPUリソース命名例に従わなければならない:
   - Labelウィジェット用ブラシ → `LabelBrushGraphics`
   - Rectangleウィジェット用ブラシ → `RectangleBrushGraphics`（存在する場合）
   - Buttonウィジェット用ビットマップ → `ButtonBitmapGraphics`（将来追加時）
3. When 新しいウィジェット用CPUリソースコンポーネントが追加される時、wintfシステムは`{LogicalComponent}Resource`命名規則に従わなければならない
4. The wintfシステムは、以下のCPUリソース命名例に従わなければならない:
   - LabelウィジェットのTextLayout → `LabelTextLayoutResource`
   - LabelウィジェットのTextFormat → `LabelTextFormatResource`
   - 図形のPathGeometry → `ShapeGeometryResource`
5. The wintfシステムは、COMリソースコンポーネント内部のアクセスメソッド名を、ラップするCOMインターフェイス型に対応させなければならない（例: `text_layout()`, `brush()`）

---

### Requirement 3: 共有リソースの命名規則

**Objective:** システム開発者として、複数のウィジェットで共有されるCOMリソース（ブラシ、ジオメトリなど）の命名規則を定義したい。これにより、リソース管理の意図が明確になる。

#### Acceptance Criteria (Shared Resources)

1. When 共有可能なGPUリソースがエンティティに保持される時、wintfシステムは`{ResourceType}Graphics`命名規則に従わなければならない
2. The wintfシステムは、以下のGPUリソース命名例に従わなければならない:
   - ID2D1SolidColorBrush → `SolidColorBrushGraphics`
   - ID2D1LinearGradientBrush → `LinearGradientBrushGraphics`
   - ID2D1Bitmap → `BitmapGraphics`
3. When 共有可能なCPUリソースがエンティティに保持される時、wintfシステムは`{ResourceType}Resource`命名規則に従わなければならない
4. The wintfシステムは、以下のCPUリソース命名例に従わなければならない:
   - IDWriteTextFormat → `TextFormatResource`
   - ID2D1PathGeometry → `PathGeometryResource`
   - ID2D1GeometryGroup → `GeometryGroupResource`
5. The wintfシステムは、共有リソースコンポーネントが独立して再利用可能であることを、命名から推測可能にしなければならない

---

### Requirement 5: 既存コードの移行安全性

**Objective:** システム開発者として、既存のコードが改名により破壊されないようにしたい。これにより、段階的な移行が可能になる。

#### Acceptance Criteria (Migration Safety)

1. When コンポーネント改名が実施される時、wintfシステムはすべてのRustコンパイルエラーを解消しなければならない
2. When コンポーネント改名が実施される時、wintfシステムは既存のすべてのテストが成功することを確認しなければならない
3. The wintfシステムは、改名前後でCOMオブジェクトのライフタイム管理動作を変更してはならない
4. The wintfシステムは、改名前後でコンポーネントのストレージタイプ（`SparseSet`等）を変更してはならない
5. The wintfシステムは、改名前後でコンポーネントのスレッド安全性（`Send`/`Sync`実装）を変更してはならない
6. The wintfシステムは、GPUリソースの`invalidate()`および`generation`管理を改名後も維持しなければならない

---

### Requirement 6: ドキュメント更新

**Objective:** システム開発者として、改名後の命名規則がドキュメントに反映されることを確認したい。これにより、新規開発者が正しいパターンを学習できる。

#### Acceptance Criteria (Documentation)

1. When コンポーネント改名が完了した時、wintfシステムは`.kiro/steering/structure.md`内の命名規則セクションを更新しなければならない
2. The wintfシステムは、命名規則のセクションに以下の情報を含めなければならない:
   - GPUリソースコンポーネントの命名パターン（`XxxGraphics`）
   - CPUリソースコンポーネントの命名パターン（`XxxResource`）
   - 論理コンポーネントとCOMリソースコンポーネントの対応関係
   - 具体的な命名例（ウィンドウレベル、ウィジェット固有、共有リソース）
   - デバイスロスト対応の有無による区別の説明
3. When 命名規則ドキュメントが更新された時、wintfシステムは将来の開発者が命名規則に従えるように十分な例を提供しなければならない

---

### Requirement 7: 命名規則の一貫性検証

**Objective:** システム開発者として、将来追加されるコンポーネントが命名規則に準拠していることを確認したい。これにより、長期的な一貫性が保たれる。

#### Acceptance Criteria (Consistency)

1. The wintfシステムは、`ecs/graphics/components.rs`内のすべてのGPUリソースコンポーネントが`XxxGraphics`命名規則に従わなければならない
2. When 新しいCOMリソースコンポーネントが追加される時、wintfシステムはレビュー時に命名規則準拠を確認しなければならない
3. If COMオブジェクトを保持しないコンポーネントが存在する場合、wintfシステムは`Graphics`または`Resource`サフィックスを付けてはならない（例: `HasGraphicsResources`、`GraphicsNeedsInit`）
4. The wintfシステムは、マーカーコンポーネント（データを持たないコンポーネント）に対して`Graphics`または`Resource`サフィックスを使用してはならない
5. When デバイスロスト対応が必要なリソースが追加される時、wintfシステムは`Graphics`サフィックスと`invalidate()`/`generation`機能を実装しなければならない
6. When デバイス非依存なリソースが追加される時、wintfシステムは`Resource`サフィックスを使用し、再初期化機構を省略しなければならない

---

## Out of Scope

以下は本要件の対象外とする：

- COMリソースコンポーネント以外のコンポーネント命名（論理コンポーネント、マーカーコンポーネントなど）
- システム関数の命名変更（`draw_rectangles`等）
- モジュール構造の変更（`ecs/graphics/`ディレクトリ構成など）
- COMオブジェクトのライフタイム管理ロジックの変更
- デバイスロスト検出・復旧機構の実装（既存の`invalidate()`/`generation`パターンを維持）
- 新規COMリソースコンポーネントの実装（既存コンポーネントの改名のみ）

---

## Dependencies

- 既存のECSコンポーネント実装（`ecs/graphics/components.rs`）
- 既存のシステム実装（`ecs/graphics/systems.rs`、`ecs/window_system.rs`など）
- 既存のテストコード（`tests/graphics_core_ecs_test.rs`など）
- 既存のデバイスロスト対応システム（`invalidate_dependent_components`、`cleanup_command_list_on_reinit`）

---

## Success Metrics

- すべてのGPUリソースコンポーネントが`XxxGraphics`命名規則に準拠
- すべてのCPUリソースコンポーネントが`XxxResource`命名規則に準拠（将来追加時）
- すべての既存テストが改名後も成功
- `cargo build`が警告なしで成功
- ドキュメント（`structure.md`）が命名規則（GPU/CPU区別含む）を明記
