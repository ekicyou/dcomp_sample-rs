# Requirements Document

## Introduction
本機能は、`wintf`ライブラリの`Label`コンポーネントにおいて、日本語の縦書き表示をサポートすることを目的とします。DirectWriteの機能を活用し、開発者がラベルのテキスト方向（縦書き・横書き）を容易に切り替えられるようにし、それに伴うレイアウト計算と描画処理を適切に行うための要件を定義します。

## Requirements

### Requirement 1: テキスト方向の設定 (Text Direction Configuration)
**Objective:** 開発者として、ラベルのテキストを縦書きにするか横書きにするかを指定したい。これにより、日本語の伝統的な表現やデザイン要件に対応できる。

#### Acceptance Criteria
1. **[Ubiquitous]** `Label`コンポーネントは、テキストの流れる方向（縦書きまたは横書き）を保持するプロパティを持たなければならない。(The `Label` component shall have a property to hold the text flow direction.)
2. **[Optional]** テキスト方向が明示的に指定されない場合、`Label`コンポーネントは横書き（Horizontal）をデフォルトとしなければならない。(Where the text direction is not explicitly specified, the `Label` component shall default to Horizontal.)
3. **[Event-Driven]** 開発者がテキスト方向を変更した際、システムはレイアウトの再計算をトリガーしなければならない。(When the developer changes the text direction, the system shall trigger a layout recalculation.)

### Requirement 2: 縦書きレイアウト計算 (Vertical Layout Calculation)
**Objective:** システムとして、縦書き指定時に適切なサイズ計算を行いたい。これにより、UI要素が正しく配置される。

#### Acceptance Criteria
1. **[State-Driven]** テキスト方向が縦書きである間、レイアウトシステムは、文字の積み上げ方向を垂直方向、行の進行方向を右から左（または設定依存）としてサイズ（幅・高さ）を計算しなければならない。(While the text direction is vertical, the layout system shall calculate the size assuming characters stack vertically and lines progress from right to left.)
2. **[State-Driven]** テキスト方向が縦書きである間、レイアウトシステムは、フォントの縦書き用メトリクスを使用して行間や文字間を計算しなければならない。(While the text direction is vertical, the layout system shall use vertical font metrics to calculate line spacing and character spacing.)

### Requirement 3: 縦書きレンダリング (Vertical Text Rendering)
**Objective:** ユーザーとして、縦書きのテキストを正しく視認したい。これにより、意図されたデザイン通りに情報が伝わる。

#### Acceptance Criteria
1. **[State-Driven]** テキスト方向が縦書きである間、描画システムはDirectWriteの機能を使用して、グリフを縦書き用に回転または置換して描画しなければならない。(While the text direction is vertical, the rendering system shall render glyphs rotated or substituted for vertical writing using DirectWrite.)
2. **[State-Driven]** テキスト方向が縦書きである間、描画システムは、指定されたアライメント（上揃え、中央揃え、下揃え等）にしたがってテキストを配置しなければならない。(While the text direction is vertical, the rendering system shall align text according to the specified alignment.)

### Requirement 4: レイアウトサイズの出力 (Layout Size Output)
**Objective:** 開発者として、計算されたテキストのサイズがシステム内で利用可能な状態になることを期待する。現状Taffy統合は未実装であるため、計算結果は独立したコンポーネントに保持される必要がある。

#### Acceptance Criteria
1. **[Ubiquitous]** システムは、縦書き・横書きにかかわらず、計算されたテキストのコンテンツサイズ（幅・高さ）を保持するコンポーネント（例: `TextLayoutMetrics`）を出力しなければならない。(The system shall output a component (e.g., `TextLayoutMetrics`) holding the calculated text content size (width/height) regardless of text direction.)
2. **[State-Driven]** テキスト方向が縦書きである間、システムは縦書きとして計算されたバウンディングボックスを、この出力コンポーネントに格納しなければならない。(While the text direction is vertical, the system shall store the bounding box calculated as vertical text into this output component.)

### Requirement 5: 検証用サンプルの更新 (Verification Sample Update)
**Objective:** 開発者として、実装された縦書き機能が正しく動作することを目視で確認したい。

#### Acceptance Criteria
1. **[Ubiquitous]** `simple_window.rs` サンプルアプリケーションは、縦書きラベルを含む新しいUIツリー（コンテナとラベル）を含まなければならない。(The `simple_window.rs` sample application shall include a new UI tree (container and label) containing a vertical label.)
2. **[Ubiquitous]** 追加されるUIツリーは、既存のレイアウトと重ならない位置（例: x=300以降）に配置されなければならない。(The added UI tree shall be placed in a position that does not overlap with the existing layout (e.g., x=300 or later).)
