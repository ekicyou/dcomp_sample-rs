# Implementation Tasks: com-resource-naming-unification

**Feature ID**: `com-resource-naming-unification`  
**Status**: Tasks Generated  
**Language**: ja

---

## Task Overview

COMリソースコンポーネントの命名規則統一を実施する。GPU資源は`XxxGraphics`、CPU資源は`XxxResource`サフィックスを適用し、型システムによる安全なリファクタリングを行う。

**改名対象**:
- `Visual` → `VisualGraphics` (GPUリソース)
- `Surface` → `SurfaceGraphics` (GPUリソース)
- `TextLayout` → `TextLayoutResource` (CPUリソース)

**影響範囲**: 約90箇所の参照更新（コンポーネント定義、システム実装、モジュールエクスポート）

---

## Implementation Plan

### Phase 1: Component Definitions

- [ ] 1. GPU資源コンポーネントの改名
- [ ] 1.1 (P) Visual → VisualGraphics への改名
  - `crates/wintf/src/ecs/graphics/components.rs`内の`pub struct Visual`を`pub struct VisualGraphics`に変更
  - コンポーネント属性、メソッド実装、ドキュメントコメントを維持
  - `visual()`アクセスメソッド名は変更しない
  - `invalidate()`メソッドと`generation`フィールドを維持（GPU資源パターン）
  - `#[component(storage = "SparseSet")]`属性を維持
  - `unsafe impl Send/Sync`実装を維持
  - _Requirements: 2.1_

- [ ] 1.2 (P) Surface → SurfaceGraphics への改名
  - `crates/wintf/src/ecs/graphics/components.rs`内の`pub struct Surface`を`pub struct SurfaceGraphics`に変更
  - コンポーネント属性、メソッド実装、ドキュメントコメントを維持
  - `surface()`アクセスメソッド名は変更しない
  - `invalidate()`メソッドと`generation`フィールドを維持（GPU資源パターン）
  - `#[component(storage = "SparseSet")]`属性を維持
  - `unsafe impl Send/Sync`実装を維持
  - _Requirements: 2.1_

- [ ] 1.3 (P) TextLayout → TextLayoutResource への改名
  - `crates/wintf/src/ecs/widget/text/label.rs`内の`pub struct TextLayout`を`pub struct TextLayoutResource`に変更
  - コンポーネント属性、メソッド実装、ドキュメントコメントを維持
  - `get()`アクセスメソッド名は変更しない
  - `invalidate()`メソッドや`generation`フィールドは存在しないことを確認（CPU資源のため不要）
  - `#[component(storage = "SparseSet")]`属性を維持
  - _Requirements: 2.2_

### Phase 2: Module Exports

- [ ] 2. モジュールエクスポートの更新
- [ ] 2.1 (P) graphics モジュールエクスポート更新
  - `crates/wintf/src/ecs/graphics/mod.rs`の`pub use`文を更新
  - `Visual` → `VisualGraphics`、`Surface` → `SurfaceGraphics`に変更
  - 他のエクスポート（`WindowGraphics`、マーカーコンポーネント等）は維持
  - _Requirements: 2.1_

- [ ] 2.2 (P) widget/text モジュールエクスポート更新
  - `crates/wintf/src/ecs/widget/text/mod.rs`の`pub use`文を更新
  - `TextLayout` → `TextLayoutResource`に変更
  - `Label`コンポーネントのエクスポートは維持
  - _Requirements: 2.2_

### Phase 3: System References

- [ ] 3. システム実装の型参照更新
- [ ] 3.1 graphics/systems.rs の Visual 参照更新
  - `crates/wintf/src/ecs/graphics/systems.rs`内の`Visual`型参照を`VisualGraphics`に更新
  - 関数シグネチャ: `create_visual_for_target()`の戻り値型
  - システム関数: `init_window_visual`のクエリ型とコンポーネント挿入
  - コンストラクタ呼び出し: `Visual::new()`→`VisualGraphics::new()`
  - 関数名（`init_window_visual`、`create_visual_for_target`）は変更しない
  - _Requirements: 2.1, 5.1_

- [ ] 3.2 graphics/systems.rs の Surface 参照更新
  - `crates/wintf/src/ecs/graphics/systems.rs`内の`Surface`型参照を`SurfaceGraphics`に更新
  - 関数シグネチャ: `create_surface_for_window()`の戻り値型と引数型
  - システム関数: `init_window_surface`、`render_surface`のクエリ型とコンポーネント挿入
  - コンストラクタ呼び出し: `Surface::new()`→`SurfaceGraphics::new()`
  - 関数名（`init_window_surface`、`render_surface`、`create_surface_for_window`）は変更しない
  - _Requirements: 2.1, 5.1_

- [ ] 3.3 widget/text/draw_labels.rs の TextLayout 参照更新
  - `crates/wintf/src/ecs/widget/text/draw_labels.rs`内の`TextLayout`型参照を`TextLayoutResource`に更新
  - システム関数: `draw_labels`のクエリ型とコンポーネント挿入
  - コンストラクタ呼び出し: `TextLayout::new()`→`TextLayoutResource::new()`、`TextLayout::empty()`→`TextLayoutResource::empty()`
  - 関数名（`draw_labels`）は変更しない
  - _Requirements: 2.2, 5.1_

### Phase 4: Compilation and Testing

- [ ] 4. ビルドとテストによる検証
- [ ] 4.1 コンパイルエラーの解消
  - `cargo build --all-targets`を実行し、すべてのコンパイルエラーを解消
  - 型不一致エラーから未更新の参照箇所を特定し、型名を更新
  - 警告なしでビルド成功することを確認
  - _Requirements: 5.1_

- [ ] 4.2 既存テストの成功確認
  - `cargo test`を実行し、すべてのテストが成功することを確認
  - テスト対象ファイル（改名影響なし）:
    - `tests/graphics_core_ecs_test.rs` - WindowGraphics使用のみ
    - `tests/graphics_reinit_unit_test.rs` - WindowGraphics使用のみ
    - `tests/lazy_reinit_pattern_test.rs` - 独立したテスト用WindowGraphics定義
    - その他テストファイル - 改名対象コンポーネント不使用
  - 既存テストのロジックやアサーションは変更しない
  - _Requirements: 5.2_

- [ ] 4.3 サンプルアプリケーションの実行確認
  - `cargo run --example areka`を実行し、正常動作を確認
  - ウィンドウ表示、描画、インタラクションが改名前と同じ動作をすることを確認
  - 他のサンプル（`dcomp_demo`、`simple_window`等）の実行も確認
  - _Requirements: 5.1, 5.2_

- [ ] 4.4 残存参照の検索確認
  - `grep -r "\bVisual\b" crates/wintf/src/`で旧型名`Visual`の残存を確認
  - `grep -r "\bSurface\b" crates/wintf/src/`で旧型名`Surface`の残存を確認（ただしIDCompositionSurfaceは除外）
  - `grep -r "\bTextLayout\b" crates/wintf/src/`で旧型名`TextLayout`の残存を確認（ただしIDWriteTextLayoutは除外）
  - コメント、docstring、テストメッセージ内での旧型名の有無を確認
  - COMインターフェイス名（`IDCompositionVisual3`、`IDCompositionSurface`、`IDWriteTextLayout`）は変更対象外
  - _Requirements: 5.1_

### Phase 5: Documentation

- [ ] 5. 命名規則ドキュメントの更新
- [ ] 5.1 structure.md への命名規則セクション追加
  - `.kiro/steering/structure.md`の既存「Naming Conventions」セクションの後に「Component Naming Conventions」セクションを追加
  - **GPUリソース (`XxxGraphics`)**: デバイス依存、`invalidate()`/`generation`実装、例: `WindowGraphics`, `VisualGraphics`, `SurfaceGraphics`
  - **CPUリソース (`XxxResource`)**: デバイス非依存、永続的、例: `TextLayoutResource`, `TextFormatResource`（将来）, `PathGeometryResource`（将来）
  - **レベル分類**: ウィンドウレベル（`WindowGraphics`）、ウィジェットレベル（`VisualGraphics`, `TextLayoutResource`）、共有リソース（将来の`BrushGraphics`等）
  - **非COMコンポーネント**: 論理コンポーネント（`Label`, `Rectangle`等）、マーカーコンポーネント（`HasGraphicsResources`等）はサフィックスなし
  - デバイスロスト対応の有無による区別を明記
  - `TextLayoutResource`の再利用性（Label、TextBlock、Button等での使用）を記載
  - _Requirements: 6.1, 6.2, 6.3, 7.9_

- [ ] 5.2 将来の拡張例の記載
  - 将来追加予定のGPUリソース例: `LabelBrushGraphics`, `SolidColorBrushGraphics`, `BitmapGraphics`
  - 将来追加予定のCPUリソース例: `TextFormatResource`, `PathGeometryResource`, `GeometryGroupResource`
  - ウィジェット固有リソースと共有リソースの命名パターンを説明
  - COMリソースコンポーネント内部のアクセスメソッド命名規則（COMインターフェイス型に対応）
  - _Requirements: 2.3, 3.1, 3.2, 3.3, 3.4, 3.5_

---

## Task Summary

- **合計**: 5メジャータスク、14サブタスク
- **並列実行可能**: 6タスク（Phase 1全体、Phase 2全体は並列実行可能）
- **要件カバレッジ**: 全9要件（R1-R7）を完全にカバー
- **推定工数**: 1-3日（Small規模）

### Requirements Coverage

| Requirement | Covered by Tasks |
|-------------|------------------|
| R1 (WindowGraphics維持) | 検証のみ（4.1, 4.2で確認） |
| R2.1 (Visual→VisualGraphics) | 1.1, 2.1, 3.1 |
| R2.1 (Surface→SurfaceGraphics) | 1.2, 2.1, 3.2 |
| R2.2 (TextLayout→TextLayoutResource) | 1.3, 2.2, 3.3 |
| R2.3 (将来の命名規則) | 5.2 |
| R3 (共有リソース命名) | 5.2 |
| R5 (移行安全性) | 1.1-1.3 (パターン維持), 4.1-4.4 (検証) |
| R6 (ドキュメント更新) | 5.1, 5.2 |
| R7 (一貫性検証) | 5.1, 5.2 |

### Parallel Execution Notes

- **Phase 1 (1.1-1.3)**: 3コンポーネントの定義ファイルは独立しており、並列改名可能
- **Phase 2 (2.1-2.2)**: モジュールエクスポートファイルも独立しており並列更新可能
- **Phase 3 (3.1-3.3)**: Phase 1, 2完了後に実施（コンパイルエラーベースの更新のため順次実行推奨）
- **Phase 4**: Phase 3完了後に実施（検証フェーズ）
- **Phase 5**: Phase 4完了後に実施（ドキュメント更新）

---

## Success Criteria

- ✅ すべてのGPUリソースコンポーネントが`XxxGraphics`命名規則に準拠
- ✅ すべてのCPUリソースコンポーネントが`XxxResource`命名規則に準拠
- ✅ `cargo build --all-targets`が警告なしで成功
- ✅ `cargo test`がすべて成功
- ✅ サンプルアプリケーションが正常動作
- ✅ `.kiro/steering/structure.md`に命名規則（GPU/CPU区別含む）が明記
- ✅ 旧型名の残存なし（grep検索で確認）
- ✅ COMオブジェクトアクセスメソッド名維持
- ✅ デバイスロスト対応パターン維持
- ✅ ストレージタイプ・スレッド安全性維持

---

_Tasks generated following Kiro-style Spec-Driven Development workflow_
