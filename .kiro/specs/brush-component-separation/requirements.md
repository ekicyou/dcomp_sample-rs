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
2. When 個別のブラシ値を扱う場合, the wintf library shall 最初からenum型`Brush`を定義しなければならない（初期バリアントは`Solid(D2D1_COLOR_F)`のみ）。
3. When グラデーションブラシ機能が追加される場合, the wintf library shall 既存の`Brush::Solid`を破壊せず、新しいバリアント（`LinearGradient`, `RadialGradient`等）を追加しなければならない。

#### 決定事項

**採用: `Brushes`** - 4つのブラシプロパティ（Foreground/Background/Fill/Stroke）を含むコンテナであることを名前で明示。将来のグラデーション対応時も`Brush`型を拡張すれば`Brushes`コンポーネント名は変更不要。

**Brush型設計: enum（議題1で決定）** - 将来のグラデーション拡張が確定しているため、最初から`enum Brush { Solid(D2D1_COLOR_F) }`として定義。非破壊的拡張を優先。

**プロパティ構成: 3プロパティ（議題2で決定）** - fillを削除しforegroundに統合。foreground/background/strokeの3プロパティ構成でAPIをシンプル化。

---

### Requirement 2: Brushesコンポーネント構造

**Objective:** システム開発者として、4つの描画用途（前景・背景・塗りつぶし・輪郭）を統一的に管理したい。これにより、描画システムの実装を簡素化する。

#### Acceptance Criteria

1. The Brushes component shall `foreground`、`background`、`stroke`の3つのオプショナルなブラシプロパティを含まなければならない。
2. When ブラシプロパティがNoneである場合, the rendering system shall 透明として扱い、描画操作を行わない。
3. The Brushes component shall 効率的な動的追加/削除のためSparseSetストレージ戦略を使用しなければならない。
4. When Brushesコンポーネントが追加される場合, the system shall Visualコンポーネントを自動挿入してはならない（ウィジェットコンポーネント側の責務）。

#### プロパティ定義

| プロパティ | 型 | 用途 | 対応ウィジェット例 |
|-----------|-----|------|-------------------|
| `foreground` | `Option<Brush>` | テキスト色、図形塗りつぶし、前景描画全般 | Label, Typewriter, Rectangle |
| `background` | `Option<Brush>` | 背景色 | Typewriter |
| `stroke` | `Option<Brush>` | 輪郭線、文字縁取り | 将来のShape、テキスト縁取り |

#### 決定事項（議題2, 3）

**fillを削除しforegroundに統合** - テキスト色と図形塗りつぶしは意味的に重複するため、汎用的な「前景色」としてforegroundに統合。3プロパティ構成によりAPIをシンプル化。

**デフォルト値と親継承ルール（議題3）**:
- `Brushes::default()` = 全プロパティNone
- Noneは常に透明として扱う
- Brushesコンポーネントが存在しない場合、描画システムは親ウィジェットのBrushes値を継承する
- ライフタイムイベント（on_add等）でのBrushes自動挿入は本仕様のスコープ外
- 将来拡張: `Brush`型に`Inherit`（親から継承）バリアントを追加する可能性あり

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
4. When ウィジェットコンポーネントが追加される場合 and Brushesコンポーネントが存在しない場合, the rendering system shall 親ウィジェットのBrushes値を継承しなければならない。
5. When ルートウィジェットにBrushesが存在しない場合, the rendering system shall デフォルト色（foreground=黒、その他=透明）を使用しなければならない。

#### スコープ外

- ウィジェットのライフタイムイベント（on_add/on_remove）でのBrushes自動挿入・デフォルト値設定は本仕様のスコープ外とする

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
4. If Brushesコンポーネントが存在しない場合, the rendering system shall 親ウィジェットのBrushes値を継承しなければならない。
5. If ルートウィジェットにBrushesが存在しない場合, the rendering system shall デフォルト色（foreground=黒、その他=透明）を使用しなければならない。
6. The rendering system shall 効率的なダーティ検出のため`Changed<Brushes>`フィルタを使用しなければならない。

---

### Requirement 5: 後方互換性とマイグレーション

**Objective:** 既存ユーザーとして、APIの変更による影響を最小限に抑えたい。これにより、既存コードのマイグレーションを容易にする。

#### Acceptance Criteria

1. The wintf library shall 各ウィジェットに便利なブラシ設定用ビルダーメソッドを提供しなければならない（例: `Rectangle::with_foreground(color)}）。
2. The colors module shall 後方互換性のため`ecs::widget::shapes::rectangle::colors`に維持されなければならない。
3. Where 新しいBrushesベースAPIが導入される場合, the documentation shall 旧APIからのマイグレーション例を含めなければならない。

---

### Requirement 6: 将来拡張性

**Objective:** ライブラリ設計者として、グラデーションブラシなどの将来機能への拡張パスを確保したい。これにより、破壊的変更なく機能追加を可能にする。

#### Acceptance Criteria

1. The Brush type shall 最初から`enum Brush { Solid(D2D1_COLOR_F) }`として定義され、将来的なバリアント追加（LinearGradient, RadialGradient等）が非破壊的に可能でなければならない。
2. The Brushes component structure shall ソリッドカラーのみを前提としてはならない。
3. When shape-brush-system仕様が実装される場合, the Brushes component shall グラデーションブラシとシームレスに統合されなければならない。

---

### Requirement 7: テスト要件

**Objective:** 品質保証担当者として、ブラシコンポーネント分離が正しく機能することを検証したい。

#### Acceptance Criteria

1. The wintf library shall マイグレーション後、`cargo test --all-targets`がテスト失敗なく成功しなければならない。
2. The wintf library shall Brushesコンポーネントの作成とデフォルト値に関するユニットテストを含めなければならない。
3. The wintf library shall RectangleがBrushes.foreground色で描画されることを検証する統合テストを含めなければならない。
4. The wintf library shall LabelがBrushes.foreground色で描画されることを検証する統合テストを含めなければならない。
5. The wintf library shall TypewriterがBrushes.foregroundとBrushes.background色で描画されることを検証する統合テストを含めなければならない。
6. The wintf library shall Brushesコンポーネントが存在しない場合のフォールバック動作のテストを含めなければならない。

