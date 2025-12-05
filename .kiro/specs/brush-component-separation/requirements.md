# Requirements Document

## Project Description (Input)
色指定（ブラシ）のコンポーネント分離を行いたい。現在、矩形やラベル、タイプライターウィジットなど、独立してcolorやフォアグラウンド、バックグラウンドがあるが、これを独立したコンポーネントにしたい。フォアグラウンド・バックグラウンド・フィル・ストロークの４パラメーターとする。Noneの時は透明。既存のウィジットから色関係プロパティを除去し、色を利用しているシステムはリファクタリング。コンポーネントの名前は将来のことを考え、ブラシまたはブラシの複数形とするが、複数形は分かりにくいかもしれないので名前についても要件定義で検討。

## Introduction

本仕様は、wintfライブラリにおける色・ブラシ関連プロパティを既存ウィジェットから分離し、独立したECSコンポーネントとして統合する要件を定義する。現在、`Rectangle`、`Label`、`Typewriter`などの各ウィジェットは個別に`color`、`foreground`、`background`といったプロパティを保持しているが、これを単一の統合ブラシコンポーネントに集約することで、コードの一貫性・再利用性・保守性を向上させる。

## Requirements

### Requirement 1: コンポーネント命名規則

**Objective:** ライブラリ利用者として、直感的で将来拡張性のあるコンポーネント名を使用したい。これにより、APIの学習コストを低減し、コードの可読性を向上させる。

#### 背景と選択肢

| 候補 | メリット | デメリット |
|------|----------|------------|
| `Brush` | 単数形で明快、WPF/WinUIと類似 | 4つのプロパティを含むため単数形が適切か疑問 |
| `Brushes` | 複数のブラシを含むことを明示 | 英語の複数形は非ネイティブには分かりにくい |
| `BrushSet` | 集合であることを明示 | やや冗長 |
| `Paint` | 描画系で一般的（Skia等） | DirectXエコシステムとの一貫性が低い |
| `ColorStyle` | 意味が明確 | 将来グラデーション対応時に不適切 |

#### Acceptance Criteria

1. When 統合ブラシコンポーネントを定義する場合, the wintf library shall ECSコンポーネント名として`Brushes`を使用しなければならない。
2. When 個別のブラシ値を扱う場合, the wintf library shall enum型`Brush`を定義しなければならない（バリアント: `Inherit`, `Solid(D2D1_COLOR_F)`）。
3. When グラデーションブラシ機能が追加される場合, the wintf library shall 既存の`Brush::Inherit`/`Brush::Solid`を破壊せず、新しいバリアント（`LinearGradient`, `RadialGradient`等）を追加しなければならない。

#### 決定事項

**採用: `Brushes`** - 2つのブラシプロパティ（Foreground/Background）を含むコンテナであることを名前で明示。将来のグラデーション対応時も`Brush`型を拡張すれば`Brushes`コンポーネント名は変更不要。

**Brush型設計: enum（議題1, 6で決定）** - `enum Brush { Inherit, Solid(D2D1_COLOR_F) }`として定義。`Inherit`は親継承マーカー、描画前に解決される。将来のグラデーション拡張も非破壊的に追加可能。

---

### Requirement 2: Brushesコンポーネント構造

**Objective:** システム開発者として、4つの描画用途（前景・背景・塗りつぶし・輪郭）を統一的に管理したい。これにより、描画システムの実装を簡素化する。

#### Acceptance Criteria

1. The Brushes component shall `foreground`、`background`の2つのオプショナルなブラシプロパティを含まなければならない。
2. When ブラシプロパティがNoneである場合, the rendering system shall 透明として扱い、描画操作を行わない。
3. The Brushes component shall 効率的な動的追加/削除のためSparseSetストレージ戦略を使用しなければならない。
4. When Brushesコンポーネントが追加される場合, the system shall Visualコンポーネントを自動挿入してはならない（ウィジェットコンポーネント側の責務）。

#### プロパティ定義

| プロパティ | 型 | 用途 | 対応ウィジェット例 |
|-----------|-----|------|-------------------|
| `foreground` | `Option<Brush>` | テキスト色、図形塗りつぶし、前景描画全般 | Label, Typewriter, Rectangle |
| `background` | `Option<Brush>` | 背景色 | Typewriter |

#### スコープ外

- `stroke`（輪郭線）は本仕様のスコープ外。ストロークは色以外の関心事項（幅、破線設定等）があるため、将来的に独立した`Stroke`コンポーネントとして設計する。

#### 決定事項（議題2, 3, 6）

**fillを削除しforegroundに統合** - テキスト色と図形塗りつぶしは意味的に重複するため、汎用的な「前景色」としてforegroundに統合。2プロパティ構成によりAPIをシンプル化。

**デフォルト値と親継承ルール（議題3, 6）**:
- `Brush`型に`Inherit`バリアントを含める: `enum Brush { Inherit, Solid(D2D1_COLOR_F) }`
- `Brushes::default()` = 全プロパティ`Brush::Inherit`
- Visualコンポーネントのon_addでBrushesデフォルト値を自動挿入

**ハイブリッド継承方式（議題6: C+A採用、議題8: BrushInheritマーカー採用）**:
- on_add時: `BrushInherit`マーカーコンポーネントを挿入（Brushesは挿入しない）
- Drawスケジュール: `resolve_inherited_brushes`システムで`With<BrushInherit>`をクエリし、親を辿って解決
- ルートまでBrushesがない場合のデフォルト色:
  - foreground → `Brush::BLACK`
  - background → `Brush::TRANSPARENT`
- ユーザーがspawn時に`Brushes`を明示指定すれば、そのInheritフィールドのみ解決
- **静的解決**: 継承は初回描画時のみ解決し、一度確定後の親変更には追従しない（別仕様スコープ）
- **効率性**: 解決後はBrushInheritマーカーを除去。以降はO(0)で処理対象外

---

### Requirement 3: 既存ウィジェットからの色プロパティ除去

**Objective:** ライブラリ保守者として、色関連プロパティを各ウィジェットから除去し、Brushesコンポーネントに統一したい。これにより、コードの重複を排除し保守性を向上させる。

#### 対象ウィジェット

| ウィジェット | 現在のプロパティ | 移行先 |
|-------------|-----------------|--------|
| `Rectangle` | `color: Color` | `Brushes.foreground` |
| `Label` | `color: Color` | `Brushes.foreground` |
| `Typewriter` | `foreground: Color`, `background: Option<Color>` | `Brushes.foreground`, `Brushes.background` |

#### Acceptance Criteria

1. The Rectangle component shall マイグレーション後、色関連プロパティを一切含んではならない。
2. The Label component shall マイグレーション後、色関連プロパティを一切含んではならない。
3. The Typewriter component shall マイグレーション後、色関連プロパティを一切含んではならない。
4. When Visualコンポーネントが追加される場合, the Visual on_add hook shall `BrushInherit`マーカーコンポーネントを自動挿入しなければならない（Brushesは挿入しない）。
5. When ウィジェットに特定の色を設定する場合, the user shall Brushesコンポーネントを明示的にspawnバンドルに含めて上書きできなければならない（例: `world.spawn((Widget::new(), Brushes::with_foreground(...)));`）。

---

### Requirement 4: 描画システムのリファクタリング

**Objective:** システム開発者として、Brushesコンポーネントから色情報を取得するよう描画システムを更新したい。これにより、統一された色管理を実現する。

#### 対象システム

- `draw_rectangles` / `update_rectangle_command_list`
- `draw_labels` / Label描画関連システム
- `draw_typewriters` / `draw_typewriter_backgrounds` / Typewriter描画関連システム

#### Acceptance Criteria

1. When Rectangleを描画する場合, the rendering system shall Brushesコンポーネントからforeground色を読み取らなければならない。
2. When Labelを描画する場合, the rendering system shall Brushesコンポーネントからforeground色を読み取らなければならない。
3. When Typewriterを描画する場合, the rendering system shall Brushesコンポーネントからforeground色とbackground色を読み取らなければならない。
4. The wintf library shall Drawスケジュール内で`resolve_inherited_brushes`システムを実行し、`BrushInherit`マーカーを持つエンティティのBrushesを解決しなければならない（bevy_ecsの順序最適化に委ねる）。
5. If ルートまでBrushesコンポーネントがない場合, the resolve system shall デフォルト色（foreground=BLACK、background=TRANSPARENT）を適用しなければならない。
6. The rendering system shall 効率的なダーティ検出のため`Changed<Brushes>`フィルタを使用しなければならない。

---

### Requirement 5: 後方互換性とマイグレーション

**Objective:** 既存ユーザーとして、APIの変更による影響を最小限に抑えたい。これにより、既存コードのマイグレーションを容易にする。

#### Acceptance Criteria

1. The wintf library shall 各ウィジェット型にブラシ設定を含むインスタンス生成メソッドを提供しなければならない（例: `Rectangle::new().with_foreground(brush)`）。
2. The Brush type shall 基本色定数を関連定数として提供しなければならない（例: `Brush::BLACK`, `Brush::WHITE`, `Brush::TRANSPARENT`等）。
3. The Brush/Brushes types shall `ecs::widget::brushes`モジュールに配置されなければならない。
4. Where 新しいBrushesベースAPIが導入される場合, the documentation shall 旧APIからのマイグレーション例を含めなければならない。

#### 決定事項（議題4, 5）

**ウィジェット起点のビルダーパターン** - ユーザーの関心順序（「どのウィジェットか」→「どのブラシか」）に沿い、各ウィジェット型にブラシ設定メソッドを提供。今後ウィジェット属性が増加した際も同様のパターンで拡張可能。

**モジュール配置とcolors統合（議題5）**:
- `Brush`/`Brushes`は`ecs::widget::brushes`に配置（論理階層、GPUリソースではない）
- `colors`モジュールは廃止し`brushes`に統合
- 色定数は`Brush::BLACK`等の関連定数として提供

---

### Requirement 6: 将来拡張性

**Objective:** ライブラリ設計者として、グラデーションブラシなどの将来機能への拡張パスを確保したい。これにより、破壊的変更なく機能追加を可能にする。

#### Acceptance Criteria

1. The Brush type shall `enum Brush { Inherit, Solid(D2D1_COLOR_F) }`として定義され、将来的なバリアント追加（LinearGradient, RadialGradient等）が非破壊的に可能でなければならない。
2. The Brushes component structure shall ソリッドカラーのみを前提としてはならない。
3. When shape-brush-system仕様が実装される場合, the Brushes component shall グラデーションブラシとシームレスに統合されなければならない。

---

### Requirement 7: テスト要件

**Objective:** 品質保証担当者として、ブラシコンポーネント分離が正しく機能することを検証したい。

#### Acceptance Criteria

1. The wintf library shall マイグレーション後、`cargo test --all-targets`がテスト失敗なく成功しなければならない。
2. The wintf library shall Brushesコンポーネントの作成とデフォルト値（`Brush::Inherit`）に関するユニットテストを含めなければならない。
3. The wintf library shall RectangleがBrushes.foreground色で描画されることを検証する統合テストを含めなければならない。
4. The wintf library shall LabelがBrushes.foreground色で描画されることを検証する統合テストを含めなければならない。
5. The wintf library shall TypewriterがBrushes.foregroundとBrushes.background色で描画されることを検証する統合テストを含めなければならない。
6. The wintf library shall `resolve_inherited_brushes`システムが`Brush::Inherit`を正しく解決することを検証するテストを含めなければならない。
7. The wintf library shall ルートまでInheritの場合にデフォルト色が適用されることを検証するテストを含めなければならない。

