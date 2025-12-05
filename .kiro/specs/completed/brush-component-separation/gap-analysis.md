# ギャップ分析: brush-component-separation

## 概要

本ドキュメントは、`brush-component-separation`仕様の要件と既存コードベースとの差分（ギャップ）を分析し、実装の影響範囲、リスク、推奨オプションを明確にする。

---

## 1. 現状分析

### 1.1 既存の色プロパティ配置

| ファイル | コンポーネント | 現在のプロパティ | 行番号 |
|----------|----------------|------------------|--------|
| `shapes/rectangle.rs` | `Rectangle` | `pub color: Color` | L15 |
| `text/label.rs` | `Label` | `pub color: Color` | L16 |
| `text/typewriter.rs` | `Typewriter` | `pub foreground: Color`, `pub background: Option<Color>` | L62-63 |

### 1.2 既存の色定数（colorsモジュール）

**場所**: `shapes/rectangle.rs` L27-38

```rust
pub mod colors {
    pub const TRANSPARENT: Color = D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
    pub const BLACK: Color = D2D1_COLOR_F { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const WHITE: Color = D2D1_COLOR_F { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
    pub const RED: Color = D2D1_COLOR_F { r: 1.0, g: 0.0, b: 0.0, a: 1.0 };
    pub const GREEN: Color = D2D1_COLOR_F { r: 0.0, g: 1.0, b: 0.0, a: 1.0 };
    pub const BLUE: Color = D2D1_COLOR_F { r: 0.0, g: 0.0, b: 1.0, a: 1.0 };
}
```

### 1.3 描画システムの色参照箇所

| システム | ファイル | 参照パターン | 行番号 |
|----------|----------|--------------|--------|
| `draw_rectangles` | `shapes/rectangle.rs` | `rectangle.color` | L188 |
| `draw_labels` | `text/draw_labels.rs` | `label.color` | L149 |
| `draw_typewriters` | `text/typewriter_systems.rs` | `typewriter.foreground` | L360, L518 |
| `draw_typewriter_backgrounds` | `text/typewriter_systems.rs` | `typewriter.background` | L375 |

### 1.4 Visual on_add フック

**場所**: `graphics/components.rs` L247-265

```rust
fn on_visual_add(mut world: DeferredWorld<'_>, ctx: HookContext) {
    let entity = ctx.entity();
    if !world.contains::<Arrangement>(entity) {
        world.commands().entity(entity).insert(Arrangement::default());
    }
    if !world.contains::<VisualGraphics>(entity) {
        world.commands().entity(entity).insert(VisualGraphics::new());
    }
    // ... SurfaceGraphics, SurfaceGraphicsDirty も挿入
}
```

**ギャップ**: Brushesコンポーネントの自動挿入が未実装。

### 1.5 既存のexamples使用パターン

**typewriter_demo.rs**:
```rust
Rectangle { color: D2D1_COLOR_F { r: 0.95, g: 0.95, b: 0.95, a: 1.0 } }
Typewriter { foreground: D2D1_COLOR_F { ... }, background: Some(D2D1_COLOR_F { ... }) }
```

---

## 2. 要件ごとのギャップ分析

### 要件1: コンポーネント命名規則

| 項目 | 現状 | 要件 | ギャップ |
|------|------|------|----------|
| コンポーネント名 | 存在しない | `Brushes` | 🔴 新規作成 |
| ブラシ型 | `D2D1_COLOR_F`直接使用 | `enum Brush { Solid(D2D1_COLOR_F) }` | 🔴 新規作成 |

**影響度**: 低（新規追加のみ）

### 要件2: Brushesコンポーネント構造

| 項目 | 現状 | 要件 | ギャップ |
|------|------|------|----------|
| foregroundプロパティ | 分散 | `Option<Brush>` | 🔴 新規 |
| backgroundプロパティ | Typewriterのみ | `Option<Brush>` | 🔴 新規 |
| ストレージ戦略 | N/A | SparseSet | 🔴 実装必要 |
| Visual自動挿入 | 未実装 | on_addで挿入 | 🔴 追加必要 |

**影響度**: 中（既存Visual hookの修正）

### 要件3: 既存ウィジェットからの色プロパティ除去

| ウィジェット | 除去対象 | 影響システム | ギャップ |
|--------------|----------|--------------|----------|
| Rectangle | `color` | draw_rectangles | 🔴 破壊的変更 |
| Label | `color` | draw_labels | 🔴 破壊的変更 |
| Typewriter | `foreground`, `background` | draw_typewriters | 🔴 破壊的変更 |

**影響度**: 高（API破壊的変更、example修正必須）

### 要件4: 描画システムのリファクタリング

| システム | 現在の参照 | 変更後の参照 | 追加処理 |
|----------|------------|--------------|----------|
| draw_rectangles | `rectangle.color` | `brushes.foreground` | 親継承フォールバック |
| draw_labels | `label.color` | `brushes.foreground` | 親継承フォールバック |
| draw_typewriters | `typewriter.foreground/background` | `brushes.foreground/background` | 親継承フォールバック |

**追加要件**:
- `Changed<Brushes>`フィルタの追加
- 親ウィジェットからのBrushes継承ロジック

**影響度**: 高（描画ロジック全面改修）

### 要件5: 後方互換性とマイグレーション

| 項目 | 現状 | 要件 | ギャップ |
|------|------|------|----------|
| ビルダーメソッド | なし | `with_foreground(brush)` | 🔴 新規 |
| 色定数 | `colors::BLACK` | `Brush::BLACK` | 🔴 移行 |
| モジュール配置 | `shapes/rectangle.rs` | `ecs/widget/brushes.rs` | 🔴 新規・移行 |

**影響度**: 中（API追加、ドキュメント更新）

### 要件6: 将来拡張性

| 項目 | 現状 | 要件 | ギャップ |
|------|------|------|----------|
| Brush型設計 | 型なし | enum with Solid variant | 🔴 新規 |
| 拡張パス | N/A | 非破壊的バリアント追加 | 設計で対応 |

**影響度**: 低（新規設計）

### 要件7: テスト要件

| テスト種別 | 現状 | 要件 | ギャップ |
|------------|------|------|----------|
| Brushes単体テスト | なし | 必須 | 🔴 新規 |
| Rectangle統合テスト | color使用 | foreground使用 | 🔴 修正 |
| Label統合テスト | color使用 | foreground使用 | 🔴 修正 |
| Typewriter統合テスト | 直接参照 | Brushes参照 | 🔴 修正 |
| フォールバックテスト | なし | 必須 | 🔴 新規 |

**影響度**: 中（テスト追加・修正）

---

## 3. 修正対象ファイル一覧

### 3.1 新規作成

| ファイル | 目的 | 工数 |
|----------|------|------|
| `ecs/widget/brushes.rs` | Brush enum, Brushes component, 色定数 | M |
| `tests/brushes_component_test.rs` | 単体テスト | S |
| `tests/brushes_inheritance_test.rs` | 継承動作テスト | M |

### 3.2 既存ファイル修正

| ファイル | 修正内容 | 工数 | リスク |
|----------|----------|------|--------|
| `ecs/widget/mod.rs` | brushesモジュールexport | S | 低 |
| `graphics/components.rs` | on_visual_addでBrushes挿入 | S | 中 |
| `shapes/rectangle.rs` | color除去, with_foreground追加, colors削除 | M | 高 |
| `text/label.rs` | color除去, with_foreground追加 | M | 高 |
| `text/typewriter.rs` | foreground/background除去, ビルダー追加 | M | 高 |
| `text/draw_labels.rs` | Brushes参照に変更 | M | 中 |
| `text/typewriter_systems.rs` | Brushes参照に変更 | L | 中 |
| `examples/typewriter_demo.rs` | 新API使用に移行 | M | 低 |
| `examples/taffy_flex_demo.rs` | 新API使用に移行 | S | 低 |

### 3.3 テスト修正

| ファイル | 修正内容 | 工数 |
|----------|----------|------|
| 既存Rectangle関連テスト | 新API使用 | S |
| 既存Label関連テスト | 新API使用 | S |
| 既存Typewriter関連テスト | 新API使用 | S |

---

## 4. 実装オプション

### Option A: 一括移行（推奨）

**アプローチ**: 全ウィジェットを同時に移行し、一貫性を確保

**手順**:
1. `brushes.rs`を作成（Brush enum, Brushes component）
2. Visual on_addにBrushes挿入を追加
3. 全ウィジェットからcolor系プロパティを除去
4. 全描画システムをBrushes参照に変更
5. 全examples/testsを更新

**メリット**:
- コードベース全体で一貫性が保たれる
- 中間状態がない

**デメリット**:
- 大規模な変更が一度に発生
- 問題発生時の切り分けが難しい

**工数**: L（3-5日）
**リスク**: 中

---

### Option B: 段階的移行

**アプローチ**: ウィジェットごとに段階的に移行

**手順**:
1. `brushes.rs`を作成
2. Rectangle → Label → Typewriter の順で個別移行
3. 各段階でテスト・動作確認

**メリット**:
- リスク分散
- 問題の早期発見

**デメリット**:
- 中間状態でAPIが不統一
- 移行期間中のメンテナンスコスト

**工数**: XL（5-7日）
**リスク**: 低

---

### Option C: 並行運用期間付き移行

**アプローチ**: 旧APIを非推奨として残しつつ新APIを導入

**手順**:
1. `brushes.rs`を作成
2. 旧プロパティに`#[deprecated]`を付与
3. 新旧両方をサポートする描画システム
4. 次バージョンで旧API削除

**メリット**:
- 既存ユーザーへの影響最小
- 移行猶予期間あり

**デメリット**:
- コード複雑化
- 保守コスト増大
- 実装工数大

**工数**: XL（7-10日）
**リスク**: 低

---

## 5. 推奨事項

### 推奨: Option A（一括移行）

**理由**:
1. **コンパイラによる網羅的チェック**: Rustの型システムにより、colorプロパティ除去後は全参照箇所でコンパイルエラーとなり、修正漏れがない
2. **ウィジェット数が限定的**: Rectangle/Label/Typewriterの3種のみで管理可能
3. **内部ライブラリ**: 外部ユーザーへの後方互換性考慮は不要
4. **テストカバレッジ**: 既存テストにより動作確認可能

### リスク軽減策

1. **フィーチャーブランチでの作業**: mainブランチへの影響を分離
2. **段階的コミット**: 論理単位でコミットし、問題時のrevert容易化
3. **CI確認**: 各段階で`cargo test --all-targets`を実行

---

## 6. 依存関係・前提条件

### 前提条件

- 要件定義完了（✅ 完了）
- 6つの議題決定事項（✅ 完了）
- stroke除外決定（✅ 完了）
- ハイブリッド継承方式（C+A）決定（✅ 完了）

### 技術的依存

| 依存先 | 状態 | 影響 |
|--------|------|------|
| bevy_ecs 0.17.2 | ✅ 導入済み | SparseSet, on_add hook使用可 |
| D2D1_COLOR_F | ✅ 導入済み | Brush::Solid内部型 |
| Visual on_add | ✅ 実装済み | 拡張ポイント確認済み |
| Parent/ChildOf | ✅ 導入済み | 親継承解決に使用 |

### 外部依存

なし（内部リファクタリング）

---

## 7. 工数見積もり

| フェーズ | 工数 | 内訳 |
|----------|------|------|
| 設計ドキュメント | S | design.md作成 |
| brushes.rs実装 | M | Brush enum (Inherit/Solid), Brushes component, 定数 |
| Visual hook修正 | S | on_visual_add拡張 |
| resolve_inherited_brushes | M | 継承解決システム |
| ウィジェット修正 | L | 3ウィジェット + ビルダー |
| 描画システム修正 | M | 3システム（継承ロジック分離済み） |
| テスト更新 | M | 既存テスト修正 + 新規テスト |
| examples更新 | S | 2-3ファイル |
| **合計** | **L** | **3-5日** |

---

## 8. 結論

### ギャップサマリー

- **新規作成**: 3ファイル（brushes.rs, resolve_inherited_brushes, テスト）
- **修正**: 9ファイル（widget 3, drawing 3, examples 2, hook 1）
- **削除**: colorsモジュール（brushesに統合）

### 追加決定事項（議題6）

**ハイブリッド継承方式（C+A）**:
- `Brush::Inherit`バリアントを追加
- on_add時: `Brushes::default()`（全プロパティ`Inherit`）を挿入
- 描画前: `resolve_inherited_brushes`システムで親を辿って解決
- ルートまでInheritの場合: foreground=BLACK, background=TRANSPARENT
- **静的解決**: 初回描画時のみ解決、親変更追従は別仕様スコープ

### 次のステップ

1. 本ギャップ分析のレビュー・承認
2. `/kiro-spec-design brush-component-separation` でデザインドキュメント作成
3. `/kiro-spec-tasks brush-component-separation` でタスク分割
4. Option A（一括移行）で実装開始

---

*Generated: 2025-12-05*
*Spec: brush-component-separation*
