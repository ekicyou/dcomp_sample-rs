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

---

## Risks & Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| `serde(untagged)` のエラーメッセージ不明瞭 | UX | 高 | カスタムバリデーションで具体的なエラーメッセージ提供 |
| TOML i64/f64 変換問題 | 互換性 | 中 | カスタムデシリアライザで吸収 |
| `interpolation` 命名変更への追従遅れ | 互換性 | 低 | バージョンピン + CI 命名一致テスト |
| `DynamicValue` の f64 NaN 等価比較 | 正確性 | 低 | `total_cmp` ベースの比較実装 |
| TOML インライン表の1行制限 | UX | 中 | 名前付きトランジションテンプレートの使用を推奨 |

---

## References

- [interpolation 0.3.0 — EaseFunction](https://docs.rs/interpolation/0.3.0/interpolation/enum.EaseFunction.html)
- [interpolation 0.3.0 — quad_bez](https://docs.rs/interpolation/0.3.0/interpolation/fn.quad_bez.html)
- [interpolation 0.3.0 — cub_bez](https://docs.rs/interpolation/0.3.0/interpolation/fn.cub_bez.html)
- [serde — Enum representations](https://serde.rs/enum-representations.html)
- [TOML v1.0.0 Specification](https://toml.io/en/v1.0.0)
- [Windows Animation Manager — Storyboard Construction](https://learn.microsoft.com/en-us/windows/win32/uianimation/storyboard-construction)
