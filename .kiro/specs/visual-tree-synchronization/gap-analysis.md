# Gap Analysis: visual-tree-synchronization

**作成日**: 2025-11-25  
**ステータス**: Gap Analysis Complete

---

## 1. 現状調査

### 1.1 関連ファイル・モジュール

| ファイル/モジュール | 役割 | 要件との関連 |
|-------------------|------|-------------|
| `com/dcomp.rs` | DirectComposition APIラッパー | R1 (RemoveVisual) |
| `ecs/graphics/components.rs` | Visual, VisualGraphics, SurfaceGraphics定義 | R2, R5 |
| `ecs/graphics/visual_manager.rs` | Visual生成システム | R2, R3, R4 |
| `ecs/graphics/systems.rs` | 描画・同期システム | R5, R5a, R8 |
| `ecs/widget/text/label.rs` | Labelコンポーネント | R4, R4a |
| `ecs/widget/shapes/rectangle.rs` | Rectangleコンポーネント | R4 |
| `ecs/layout/arrangement.rs` | Arrangement, GlobalArrangement | R5a, R8 |
| `ecs/layout/metrics.rs` | TextLayoutMetrics定義 | R4a |
| `ecs/world.rs` | スケジュール・システム実行順序 | R10 |

### 1.2 既存実装パターン

#### Component Hook パターン
- `Arrangement` → `on_add`: `GlobalArrangement`と`ArrangementTreeChanged`を自動挿入
- `SurfaceGraphics` → `on_add`/`on_replace`: `SurfaceUpdateRequested`を挿入
- `Label` → `on_remove`: `GraphicsCommandList`をクリア
- `Rectangle` → `on_remove`: `GraphicsCommandList`をクリア
- `Window` → `on_add`: `HasGraphicsResources`等を挿入

#### 階層伝播パターン
- `propagate_parent_transforms<L, G, M>()`: 汎用的な親→子伝播システム
- `sync_simple_transforms<L, G, M>()`: 孤立エンティティ用
- `mark_dirty_trees<L, G, M>()`: ダーティビット伝播

#### GPUリソース命名規則
- `XxxGraphics`サフィックス: `WindowGraphics`, `VisualGraphics`, `SurfaceGraphics`
- `invalidate()`メソッドと`generation`フィールドでデバイスロスト対応

### 1.3 統合ポイント

| 統合ポイント | 現状 | 備考 |
|-------------|------|------|
| bevy_ecs階層 | `ChildOf`/`Children`で実装済み | R6, R7で活用 |
| taffy連携 | `BoxStyle` → `TaffyStyle` → レイアウト計算 | R4aでテキストサイズ反映必要 |
| DComp Visual | `DCompositionVisualExt`トレイト存在 | R1でRemoveVisual追加必要 |
| 描画パイプライン | `draw_recursive`で子孫集約描画 | R5aで自己描画に変更 |

---

## 2. 要件別 実現可能性分析

### R1: RemoveVisual APIラッパー

| 技術的ニーズ | 現状 | ギャップ |
|-------------|------|---------|
| `RemoveVisual` API | `AddVisual`のみ存在 | **Missing**: メソッド追加必要 |
| `RemoveAllVisuals` API | 未実装 | **Missing**: メソッド追加必要 |
| エラーハンドリング | `AddVisual`でパターン確立済み | None |

**複雑度シグナル**: Simple CRUD（既存パターンの拡張）

### R2: Visual追加時のVisualGraphics自動作成

| 技術的ニーズ | 現状 | ギャップ |
|-------------|------|---------|
| `Added<Visual>`検知 | `visual_resource_management_system`で実装済み | None |
| `IDCompositionDevice3::CreateVisual` | `create_visual_resources()`で実装済み | None |
| 自動`SurfaceGraphics`作成 | `create_visual_resources()`で実装済み | **Constraint**: Surface遅延作成に変更必要 (R5) |

**複雑度シグナル**: 既存コード修正（遅延作成への変更）

### R3: insert_visual ヘルパー関数

| 技術的ニーズ | 現状 | ギャップ |
|-------------|------|---------|
| `DeferredWorld`対応 | `on_add`フックで`DeferredWorld`使用パターン確立 | None |
| `Visual::default()`挿入 | `commands.entity().insert()`パターン確立 | None |
| 公開API | 内部関数のみ | **Missing**: 公開関数追加必要 |

**複雑度シグナル**: Simple（新規ヘルパー関数作成）

### R4: 既存ウィジットへのVisual自動追加

| 技術的ニーズ | 現状 | ギャップ |
|-------------|------|---------|
| `Label`の`on_add`フック | `on_remove`フックのみ存在 | **Missing**: `on_add`追加必要 |
| `Rectangle`の`on_add`フック | `on_remove`フックのみ存在 | **Missing**: `on_add`追加必要 |
| Surface遅延作成 | R5と統合 | **Constraint**: R5と連携必要 |

**複雑度シグナル**: 既存コンポーネント拡張

### R4a: Labelテキスト測定とBoxStyle反映

| 技術的ニーズ | 現状 | ギャップ |
|-------------|------|---------|
| テキスト測定 | `draw_labels`内で`GetMetrics()`実行済み | **Research Needed**: 分離が必要か検討 |
| `TextLayoutMetrics` | 定義・生成済み | None |
| `BoxStyle`への反映 | 未実装 | **Missing**: 同期システム追加必要 |
| システム実行順序 | `draw_labels`はレイアウト後 | **Constraint**: 測定をレイアウト前に移動 |

**複雑度シグナル**: Medium（システム分離・順序変更）

### R5: SurfaceGraphics遅延作成

| 技術的ニーズ | 現状 | ギャップ |
|-------------|------|---------|
| 遅延作成トリガー | 描画時（`GraphicsCommandList`保持時） | **Missing**: 条件追加必要 |
| `GlobalArrangement.bounds`からサイズ計算 | `Visual.size`使用中 | **Constraint**: サイズ計算ロジック変更 |
| Surface新規作成（リサイズ時） | `resize_surface_from_visual`存在 | 既存ロジック活用可能 |
| `SetContent` | `create_surface_for_window()`内で実行済み | None |

**複雑度シグナル**: Medium（既存システム改修）

### R5a: 描画方式の変更（自己描画方式）

| 技術的ニーズ | 現状 | ギャップ |
|-------------|------|---------|
| `draw_recursive`廃止 | `render_surface`内で再帰描画中 | **Missing**: 自己描画に変更 |
| スケール成分抽出 | `GlobalArrangement.transform`から抽出必要 | **Research Needed**: Matrix3x2分解方法 |
| `Visual.SetOffsetX/Y`呼び出し | `DCompositionVisualExt`に存在 | None |
| `Arrangement.offset`同期 | 未実装 | **Missing**: 同期システム追加 (R8) |

**複雑度シグナル**: Large（描画パイプライン全面変更）

### R6: ウィジットツリー変更の検知

| 技術的ニーズ | 現状 | ギャップ |
|-------------|------|---------|
| `Added<ChildOf>`検知 | `sync_taffy_tree_system`で使用済み | None |
| `Changed<ChildOf>`検知 | `sync_taffy_tree_system`で使用済み | None |
| `RemovedComponents<ChildOf>` | `sync_taffy_tree_system`で使用済み | None |

**複雑度シグナル**: Simple（既存パターン流用）

### R7: ビジュアルツリーへの同期

| 技術的ニーズ | 現状 | ギャップ |
|-------------|------|---------|
| `AddVisual`呼び出し | `DCompositionVisualExt.add_visual()`存在 | None |
| `RemoveVisual`呼び出し | 未実装 | **Missing**: R1で追加後使用 |
| Z-order管理 | `AddVisual`の`insertabove`引数で制御可能 | **Research Needed**: Childrenの順序をどう反映するか |

**複雑度シグナル**: Medium（新規同期システム）

### R8: VisualのOffset同期

| 技術的ニーズ | 現状 | ギャップ |
|-------------|------|---------|
| `SetOffsetX`/`SetOffsetY` | `DCompositionVisualExt`で実装済み | None |
| `Changed<Arrangement>`検知 | 汎用パターン確立済み | None |
| `VisualGraphics`取得 | クエリパターン確立済み | None |

**複雑度シグナル**: Simple（既存パターンで実装可能）

### R9: Visualライフサイクル管理

| 技術的ニーズ | 現状 | ギャップ |
|-------------|------|---------|
| `on_remove`フック | `Label`, `Rectangle`で使用済み | None |
| 親Visualからの削除 | R7の`RemoveVisual`を活用 | **Constraint**: R7実装後 |
| COM参照カウント | `IDCompositionVisual3`はDrop時自動解放 | None |

**複雑度シグナル**: Simple（既存パターン流用）

### R10: システム実行順序

| 技術的ニーズ | 現状 | ギャップ |
|-------------|------|---------|
| スケジュール定義 | `world.rs`で複数スケジュール定義済み | None |
| 依存関係指定 | `.after()`, `.chain()`使用済み | None |
| 新システム追加箇所 | 既存スケジュール構造を拡張 | **Missing**: 新システム配置設計 |

**複雑度シグナル**: Simple（既存構造への追加）

---

## 3. 実装アプローチ選択肢

### Option A: 既存コンポーネント拡張

**適用範囲**: R1, R3, R4, R6, R8, R9, R10

**対象ファイル修正**:
- `com/dcomp.rs`: `RemoveVisual`追加
- `ecs/graphics/mod.rs`: `insert_visual`公開
- `widget/text/label.rs`: `on_add`追加
- `widget/shapes/rectangle.rs`: `on_add`追加

**トレードオフ**:
- ✅ 既存コードの構造を維持
- ✅ テストが容易（差分小）
- ❌ 一部ファイルの責務増加

### Option B: 新規システムモジュール作成

**適用範囲**: R4a, R5, R5a, R7

**新規ファイル作成**:
- `ecs/graphics/visual_tree_sync.rs`: ビジュアルツリー同期システム
- `ecs/layout/text_measure.rs`: テキスト測定システム（R4a）
- `ecs/graphics/surface_management.rs`: Surface遅延作成・リサイズ管理

**トレードオフ**:
- ✅ 責務分離が明確
- ✅ 新機能の独立テスト可能
- ❌ ファイル数増加
- ❌ モジュール間の依存設計が必要

### Option C: ハイブリッドアプローチ（推奨）

**戦略**:
1. **Phase 1**: API層（R1）と軽量拡張（R3, R4, R6）を既存ファイルに追加
2. **Phase 2**: 新規システム（R4a, R5, R7, R8）を専用モジュールで実装
3. **Phase 3**: 描画パイプライン変更（R5a）を最後に実施

**ファイル配置**:
```
ecs/graphics/
  ├── visual_tree_sync.rs    ← NEW: R6, R7, R9
  ├── visual_offset_sync.rs  ← NEW: R8
  ├── surface_management.rs  ← NEW: R5
  └── (既存ファイル修正)

ecs/layout/
  └── text_measure.rs        ← NEW: R4a
```

**トレードオフ**:
- ✅ 段階的な実装が可能
- ✅ 各フェーズでテスト・検証可能
- ✅ 描画パイプライン変更を最後に行うことでリスク分散
- ❌ 計画が複雑

---

## 4. 工数・リスク評価

| 要件 | 工数 | リスク | 根拠 |
|------|------|--------|------|
| R1 | S (1-2日) | Low | 既存APIラッパーパターンの拡張 |
| R2 | S (1日) | Low | 既存システムの修正（遅延作成対応） |
| R3 | S (1日) | Low | シンプルなヘルパー関数追加 |
| R4 | S (1-2日) | Low | 既存on_addフックパターンの適用 |
| R4a | M (3-5日) | Medium | システム分離・実行順序変更 |
| R5 | M (3-5日) | Medium | 既存Surface作成ロジックの改修 |
| R5a | L (1-2週) | High | 描画パイプライン全面変更 |
| R6 | S (1日) | Low | 既存検知パターン流用 |
| R7 | M (3-5日) | Medium | 新規同期システム、Z-order管理 |
| R8 | S (1-2日) | Low | 既存パターンで実装可能 |
| R9 | S (1日) | Low | 既存on_removeパターン流用 |
| R10 | S (1日) | Low | 既存スケジュール構造への追加 |

**総合工数**: L (2-3週)  
**総合リスク**: Medium（R5aの描画パイプライン変更がクリティカルパス）

---

## 5. 設計フェーズへの推奨事項

### 5.1 推奨アプローチ

**Option C（ハイブリッド）** を推奨。

### 5.2 設計フェーズで検討すべき事項

1. **R4a: テキスト測定のタイミング**
   - 現在の`draw_labels`から測定ロジックを分離する方法
   - `Label.color`変更時の無駄な再測定を避ける戦略

2. **R5a: Matrix3x2からのスケール成分抽出**
   - 軸平行変換の前提で`M11`/`M22`を直接使用可能か確認
   - 回転がない前提の妥当性を設計で明文化

3. **R7: Z-order管理戦略**
   - `Children`の順序とVisual階層の対応
   - `AddVisual`の`insertabove`引数の活用方法

4. **システム実行順序の詳細設計**
   - 新規スケジュール追加 vs 既存スケジュール拡張
   - PostLayoutとRenderの間の新スケジュールが必要か

### 5.3 Research Items（設計フェーズで調査）

| 項目 | 詳細 | 優先度 |
|-----|------|--------|
| Matrix3x2スケール抽出 | 軸平行前提での`M11`/`M22`使用確認 | High |
| `AddVisual` Z-order | `insertabove`引数の詳細動作 | Medium |
| テキスト測定分離 | DirectWrite呼び出しの最適化 | Medium |

---

## 6. 次のステップ

1. 本Gap Analysisのレビュー
2. `/kiro-spec-design visual-tree-synchronization` で設計フェーズを開始
3. Phase 1（API層・軽量拡張）から実装開始を推奨
