# Research & Design Decisions

| 項目 | 内容 |
|------|------|
| **Feature** | dola-animation-system |
| **Discovery Scope** | New Feature（グリーンフィールド） |

## Summary

- **Key Finding 1**: `interpolation` クレート v0.3.0 の `EaseFunction` は serde 未対応。Dola では同一命名の自前列挙型を定義し、コンパイル時依存を排除する
- **Key Finding 2**: serde `#[serde(untagged)]` によるハイブリッド参照（文字列 | オブジェクト）は TransitionRef・EasingFunction・KeyframeRef の3箇所で適用可能
- **Key Finding 3**: TOML の `[table]` と `[[array_of_tables]]` の共存制約により、ストーリーボードはネスト構造（`[storyboard.X]` + `[[storyboard.X.entry]]`）を採用
- **Key Finding 4**: Object 型変数のためのフォーマット非依存値型 `DynamicValue` を自前定義する必要がある
- **Key Finding 5**: v1.6/v1.7 の `at` フィールド変更により、`KeyframeRef` は 3 バリアント（Single/Multiple/WithOffset）+ `KeyframeNames` ヘルパー enum の多層 `#[serde(untagged)]` 設計が必要
- **Key Finding 6**: Object 型トランジションの `to` を自然に表現するため、`TransitionValue` に `Dynamic(DynamicValue)` バリアントを追加。`#[serde(untagged)]` で Scalar → Dynamic の順に試行

---

## Research Log

### interpolation クレート API 分析

- **Context**: Req 2.3, 2.4, 2.5, 7.4 — イージング関数名を `interpolation::EaseFunction` に準拠させる
- **Sources**: [interpolation 0.3.0 docs](https://docs.rs/interpolation/0.3.0/interpolation/)
- **Findings**:
  - `EaseFunction` 列挙型: 30バリアント（Quadratic/Cubic/Quartic/Quintic/Sine/Circular/Exponential/Elastic/Back/Bounce × In/Out/InOut）
  - derive: `Clone`, `Copy`, `Debug`, `PartialEq`, `StructuralPartialEq` — **serde 未対応**
  - `quad_bez<T: Lerp>(x0: &T, x1: &T, x2: &T, t: &T::Scalar) -> T` — 二次ベジェ補間
  - `cub_bez<T: Lerp>(x0: &T, x1: &T, x2: &T, x3: &T, t: &T::Scalar) -> T` — 三次ベジェ補間
  - `Lerp` trait: `lerp(&self, other: &Self, t: &Self::Scalar) -> Self`
  - `Ease` trait: `ease(self, easing: EaseFunction, t: &T::Scalar) -> T`
- **Implications**:
  - serde 未対応のため、Dola では同一命名の `EasingFunction` 列挙型を自前定義する
  - ランタイム（将来の別仕様）が `EasingFunction` → `interpolation::EaseFunction` へマッピングする
  - ベジェ補間は制御点パラメータをシリアライズ可能な構造体として定義する

### serde ポリモーフィックデシリアライゼーション

- **Context**: Req 2.9（ハイブリッドトランジション参照）、イージング関数の文字列/オブジェクト混在
- **Sources**: [serde enum representations](https://serde.rs/enum-representations.html)
- **Findings**:
  - `#[serde(untagged)]`: バリアントを順番に試行。文字列→オブジェクトの順で定義すれば自然に択一処理
  - エラーメッセージが不明瞭になる既知の問題（"data did not match any variant"）
  - `#[serde(tag = "type")]`: 内部タグ方式。パラメトリックイージングに適用可能
  - TOML での文字列 `"Linear"` とオブジェクト `{ type = "CubicBezier", x0 = 0.0, ... }` の混在は自然に処理可能
- **Implications**:
  - `TransitionRef`: `#[serde(untagged)]` で `Named(String)` | `Inline(TransitionDef)`
  - `EasingFunction`: `#[serde(untagged)]` で `Named(EasingName)` | `Parametric(ParametricEasing)`
  - `KeyframeRef`: `#[serde(untagged)]` で `Simple(String)` | `WithOffset { keyframe, offset }`
  - バリデーションエラーはデシリアライズ後の検証フェーズでカスタムメッセージを提供

### TOML 構造制約とストーリーボード配置

- **Context**: Req 4.7（メタ情報配置は設計フェーズで決定）
- **Sources**: [TOML v1.0.0 Spec](https://toml.io/en/v1.0.0)
- **Findings**:
  - TOML では同一キーパスに `[table]`（スカラー値）と `[[array_of_tables]]` を**混在不可**
  - つまり `[storyboard.greeting]`（メタ）と `[[storyboard.greeting]]`（エントリ配列）は共存できない
  - 解決策: ネスト構造 — `[storyboard.greeting]` にメタ + `[[storyboard.greeting.entry]]` にエントリ配列
  - このパターンは JSON/YAML でも自然に `{ "storyboard": { "greeting": { "time_scale": 1.0, "entry": [...] } } }` に対応
  - TOML インライン表は1行制限あり。複雑なインライントランジションは名前付きテンプレートで回避推奨
- **Implications**:
  - ユーザーの初期スケッチ `[[storyboard.greeting]]` から `[[storyboard.greeting.entry]]` へ変更
  - Storyboard 構造体 = メタフィールド + `entry: Vec<StoryboardEntry>` として一体化

### DynamicValue 型の必要性

- **Context**: Req 1.3, 1.7, 2.6 — Object 型変数の初期値・トランジション終了値
- **Findings**:
  - `serde_json::Value` は JSON 専用（フォーマット非依存ではない）
  - `toml::Value` は TOML 専用
  - フォーマット非依存の動的値型を自前定義する必要がある
  - 構成: Null, Bool, Integer(i64), Float(f64), String, Array, Map(BTreeMap)
  - `#[serde(untagged)]` で各フォーマットのネイティブ型に自然にマッピング
  - Integer を Float より前に定義することで TOML の i64/f64 区別を保持
- **Implications**:
  - `value.rs` モジュールに `DynamicValue` 列挙型を定義
  - `PartialEq` 実装で f64 の NaN 問題を考慮（`total_cmp` 使用を推奨）

### TOML の整数/浮動小数点変換

- **Context**: `from = 5`（TOML 上は i64）を `Option<f64>` フィールドにデシリアライズ
- **Findings**:
  - `serde_json`: 数値は内部的に f64/i64/u64 だが自動変換あり
  - `toml` クレート: i64 → f64 の自動変換は**保証されない**
  - `serde_yaml`: 自動変換あり
- **Implications**:
  - `from` / `relative_to` フィールドにカスタムデシリアライザを実装するか、ラッパー型 `Number(i64|f64)` を使用
  - 設計上は f64 を論理型とし、実装フェーズで TOML 互換デシリアライザを対応

### v1.5/v1.6/v1.7 要件変更の設計影響評価

- **Context**: 要件 v1.5（暗黙的KF生成 Req 3.6）、v1.6（at 配列化 Req 4.4/4.5）、v1.7（at 文字列短縮形 Req 4.4/4.5）の追加による既存設計への影響評価
- **Sources**: requirements.md v1.7, gap-analysis.md v2.0
- **Findings**:
  - Req 3.6（暗黙的KF）: データモデル変更不要。`keyframe: Option<String>` のまま。暗黙的KF名は処理段階（バリデーション/ランタイム）で生成。V6 を更新し暗黙的KFを考慮した参照検証が必要
  - Req 4.4/4.5（at 配列化 + 文字列短縮形）: `KeyframeRef` enum の完全再設計。3バリアント（Single/Multiple/WithOffset）+ `KeyframeNames` ヘルパー enum。`#[serde(untagged)]` の2段階適用
  - gap-analysis D2（TransitionValue + Object）: `TransitionValue::Dynamic(DynamicValue)` バリアント追加で解決
  - gap-analysis D1（値域超過）: バリデーションエラーとして処理。ランタイムクランプはスコープ外
  - gap-analysis D3-D5: 将来仕様に委譲（v1 では未サポート/最小実装で十分）
- **Implications**:
  - KeyframeRef の再設計が design.md v2.0 の最大の変更点
  - バリデーションルール追加: V12（値域チェック）, V13（型整合性チェック）, V6更新（暗黙的KF対応）

---

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| Pure Data Model | Struct + serde のみ | 最小限、テスト容易 | 構築時の不正状態を防げない | シンプルすぎる |
| Domain Model | Struct + 不変条件 + private fields | 型安全な構築 | serde との相性が複雑化 | オーバーエンジニアリング気味 |
| **Hybrid（採用）** | Serde structs + 分離バリデーション + オプショナルビルダー | 柔軟性と安全性のバランス | 二段階（deserialize → validate） | Dola のスコープに最適 |

**選定理由**: Dola はシリアライズ可能なデータフォーマットが主目的。serde の derive マクロとの親和性を最優先しつつ、分離されたバリデーション関数で構造整合性を保証する。ビルダー API はプログラマティックな構築のための補助的インターフェース。

---

## Design Decisions

### Decision 1: EasingFunction 自前定義（interpolation 非依存）

- **Context**: Req 7.4「ランタイム依存の有無は設計フェーズで判断」
- **Alternatives**:
  1. `interpolation` クレートに依存し newtype ラッパーで serde 実装
  2. `interpolation` クレートをフォーク
  3. 同一命名の自前列挙型を定義
- **Selected**: Option 3 — 自前定義
- **Rationale**:
  - Dola はデータモデル専用クレート。値補間ロジックは含まない
  - `interpolation` は serde 未対応。newtype でラップするコストに見合わない
  - 命名準拠により、ランタイム側のマッピングは機械的な 1:1 対応
- **Trade-offs**: `interpolation` クレートのバリアント名変更時に手動追従が必要
- **Follow-up**: CI テストで命名一致を検証する仕組みを推奨

### Decision 2: DynamicValue カスタム型

- **Context**: Object 型変数の値をフォーマット非依存で表現
- **Alternatives**:
  1. `serde_json::Value` を常時依存
  2. `toml::Value` / `serde_yaml::Value` のいずれかをベース型に
  3. カスタム `DynamicValue` 列挙型
- **Selected**: Option 3 — カスタム型
- **Rationale**: フォーマット固有の型に依存すると、他フォーマット使用時に不要な依存が発生。カスタム型は serde の `Serialize`/`Deserialize` を直接実装でき、全フォーマットに対応可能

### Decision 3: ネステッドストーリーボード構造

- **Context**: Req 4.7 のメタ情報配置（設計フェーズ決定事項）
- **Alternatives**:
  1. フラット `[[storyboard.X]]` + 別セクション `[storyboard_meta.X]`
  2. ネスト `[storyboard.X]`（メタ）+ `[[storyboard.X.entry]]`（エントリ）
  3. 最初のエントリにメタ情報を埋め込み
- **Selected**: Option 2 — ネスト構造
- **Rationale**:
  - 単一の `Storyboard` 構造体で完結（名前同期不要）
  - TOML/JSON/YAML すべてで自然な構造
  - Rust の `BTreeMap<String, Storyboard>` に直接マッピング

### Decision 4: EasingFunction 二層構造

- **Context**: 文字列イージング `"Linear"` とパラメトリック `{ type: "CubicBezier", ... }` の混在
- **Selected**: `EasingFunction` = `Named(EasingName)` | `Parametric(ParametricEasing)`
  - `EasingName`: 31バリアント（Linear + 30 EaseFunction 準拠）
  - `ParametricEasing`: `#[serde(tag = "type")]` で QuadraticBezier / CubicBezier
- **Rationale**: `#[serde(untagged)]` で文字列→Named、オブジェクト→Parametric を自然に処理。TOML で `easing = "CubicIn"` と `easing = { type = "CubicBezier", x0 = 0.0, ... }` が共存可能

### Decision 5: フラット StoryboardEntry + 分離バリデーション

- **Context**: エントリは3つの配置パターン + 純粋KF定義を表現する必要がある
- **Alternatives**:
  1. 列挙型 `enum StoryboardEntry { Transition{...}, Keyframe{...} }` で型レベル区別
  2. フラット構造体 `struct StoryboardEntry` + 全フィールド `Option` + バリデーション
- **Selected**: Option 2 — フラット + バリデーション
- **Rationale**:
  - TOML/JSON の表現が自然（各エントリが同じスキーマ）
  - serde(untagged) の列挙型よりエラーメッセージが明確
  - バリデーションで有効な組み合わせを検証

### Decision 6: KeyframeRef の多形 serde 設計（v1.6/v1.7 対応）

- **Context**: Req 4.4/4.5 — v1.6 で `at` が配列に変更、v1.7 で文字列短縮形も許可。既存の `KeyframeRef`（Simple/WithOffset）では配列表現不可
- **Alternatives**:
  1. `at` を常に `Vec<String>` とし、`offset` は別フィールドに分離
  2. `KeyframeRef` を 3 バリアント（Single/Multiple/WithOffset）に再設計 + `KeyframeNames` ヘルパー
  3. `at` を生値として受け取りカスタムデシリアライザで処理
- **Selected**: Option 2 — 3 バリアント + KeyframeNames
- **Rationale**:
  - `#[serde(untagged)]` で String → Vec<String> → Object の順に自然に試行
  - JSON/TOML/YAML すべてで直感的な表現が可能
  - WithOffset 内の `keyframes` も `KeyframeNames`（String | Vec<String>）で多形化
- **Trade-offs**: 2 段階の untagged enum でエラーメッセージが不明瞭になる可能性
- **Follow-up**: バリデーションで空配列・重複KF名チェックを追加

### Decision 7: TransitionValue に Dynamic バリアント追加（Object 型対応）

- **Context**: gap-analysis D2 — TransitionValue は Scalar(f64) のみだが、Object 型変数の `to` に DynamicValue を許容する必要がある
- **Alternatives**:
  1. `to` フィールドの型を `DynamicValue` に変更（すべての値を動的型で扱う）
  2. `TransitionValue` に `Dynamic(DynamicValue)` バリアントを追加
  3. `to` と `to_object` の 2 フィールドに分離
- **Selected**: Option 2 — Dynamic バリアント追加
- **Rationale**:
  - `#[serde(untagged)]` で Scalar(f64) → Dynamic(DynamicValue) の順に試行
  - 数値は Scalar、オブジェクト構造は Dynamic に自然にマッピング
  - 既存の Scalar パスに影響なし
  - TOML: `to = 1.0` → Scalar, `to = { path = "smile.png" }` → Dynamic
- **Trade-offs**: TOML 整数 `to = 5` は Scalar デシリアライズ失敗後 Dynamic(Integer) にフォールバックする可能性 → カスタムデシリアライザで吸収
- **Follow-up**: バリデーション V10/V13 で変数型と TransitionValue バリアントの整合性を検証

### Decision 8: 値域超過はバリデーションエラーとする

- **Context**: gap-analysis D1 — f64/i64 変数の初期値やトランジション to/from が値域（min/max）を超過した場合の挙動
- **Alternatives**:
  1. クランプ（WAM スタイル: 超過値を min/max に丸める）
  2. バリデーションエラー（静的チェックとして報告）
  3. 無視（ランタイムに委譲）
- **Selected**: Option 2 — バリデーションエラー
- **Rationale**:
  - Dola はデータモデル＋バリデーションのクレート。ランタイム挙動はスコープ外
  - 明示的エラー報告で定義ファイル作成者が早期に問題発見可能
  - ランタイムのクランプ実装は別仕様の判断
- **Trade-offs**: イージング曲線によるオーバーシュート等の中間値は静的チェック不可能（ランタイムの関心事）
- **Follow-up**: DolaError に ValueOutOfRange バリアント追加

### Decision 9: EasingName/ParametricEasing は snake_case シリアライズ

- **Context**: 設計分析 Q2 — TOML/JSON で手書きする際の可読性。interpolation は PascalCase だが、Rust enum バリアント名とシリアライズ形式を分離すべきか？
- **Alternatives**:
  1. PascalCase のまま（`"QuadraticInOut"`, `{ type = "CubicBezier" }`） — interpolation と完全一致
  2. Rust は PascalCase、シリアライズのみ snake_case（`"quadratic_in_out"`, `{ type = "cubic_bezier" }`）
- **Selected**: Option 2 — `#[serde(rename_all = "snake_case")]` で自動変換
- **Rationale**:
  - **TOML/JSON 可読性向上** — `easing = "quadratic_in"` は手書きしやすく、Rust エコシステムの慣習と一致
  - **interpolation 準拠維持** — Rust enum 名は PascalCase で interpolation と名前一致、マッピング可能
  - **serde ベストプラクティス** — snake_case シリアライズは tokio, actix-web など主要クレートで標準パターン
  - **警告なし** — Rust enum バリアントは PascalCase のまま、`rustc`/`clippy` の警告を回避
- **Trade-offs**: シリアライズ形式と Rust enum 名が異なるが、これは一般的なパターン
- **Follow-up**: TOML 例を snake_case に更新（`"linear"`, `"quadratic_in_out"`, `"cubic_bezier"`）

### Decision 10: InterruptionPolicy をストーリーボード属性として追加

- **Context**: 設計分析 Q3 — マルチプロセス協調アニメーション環境において、ストーリーボード競合時の終了戦略をどこに持たせるべきか？WAM ではマネージャーのグローバル属性だが、実用上は各ストーリーボードが「自分が中断されたらどう振る舞うか」を宣言する方が自然
- **Alternatives**:
  1. InterruptionPolicy のみ追加（協調的な自己申告）
  2. InterruptionPolicy + priority（競争的優先度）も追加
  3. 見送り（将来仕様に委譲）
- **Selected**: Option 1 — InterruptionPolicy のみ
- **Rationale**:
  - **マルチプロセス協調アニメーション設計**: 各プロセスは「自分の終了方針」を宣言的に自己申告し、オーケストレーション側の指示は絶対とする。priority（競争）ではなく協調的な設計が適切
  - **Never の意味**: 「このSBが未完了なら新SBの開始を待機」という協調的な制約として機能
  - **Dola のスコープ**: データモデル＋バリデーションであり、解決ロジックは将来のランタイム仕様に委譲。InterruptionPolicy は「情報提供」として十分
  - **priority 不採用**: 使いこなせない複雑性を避ける。オーケストレーション側の解決ロジックが単純化される
- **Trade-offs**: 優先度による細かな制御はできないが、シンプルで理解しやすい設計となる
- **Follow-up**: 
  - Req 4.9 追加（interruption_policy 属性）
  - InterruptionPolicy enum 定義（Cancel/Conclude/Trim/Compress/Never）
  - Storyboard に `interruption_policy: InterruptionPolicy` フィールド追加（デフォルト: Conclude）
  - snake_case シリアライズ（`"cancel"`, `"conclude"`, `"never"` など）

---

## Risks & Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| `serde(untagged)` のエラーメッセージ不明瞭 | UX | 高 | カスタムバリデーションで具体的なエラーメッセージ提供 |
| TOML i64/f64 変換問題 | 互換性 | 中 | カスタムデシリアライザで吸収 |
| `interpolation` 命名変更への追従遅れ | 互換性 | 低 | バージョンピン + CI 命名一致テスト |
| `DynamicValue` の f64 NaN 等価比較 | 正確性 | 低 | `total_cmp` ベースの比較実装 |
| TOML インライン表の1行制限 | UX | 中 | 名前付きトランジションテンプレートの使用を推奨 |
| KeyframeRef 2段階 `untagged` のエラーメッセージ劣化 | UX | 中 | バリデーション層でキーフレーム参照形式の具体的エラーを提供 |
| TransitionValue Scalar/Dynamic の TOML 整数フォールバック | 互換性 | 中 | Scalar 用カスタムデシリアライザ（i64→f64 変換）で吸収 |

---

## References

- [interpolation 0.3.0 — EaseFunction](https://docs.rs/interpolation/0.3.0/interpolation/enum.EaseFunction.html)
- [interpolation 0.3.0 — quad_bez](https://docs.rs/interpolation/0.3.0/interpolation/fn.quad_bez.html)
- [interpolation 0.3.0 — cub_bez](https://docs.rs/interpolation/0.3.0/interpolation/fn.cub_bez.html)
- [serde — Enum representations](https://serde.rs/enum-representations.html)
- [TOML v1.0.0 Specification](https://toml.io/en/v1.0.0)
- [Windows Animation Manager — Storyboard Construction](https://learn.microsoft.com/en-us/windows/win32/uianimation/storyboard-construction)
