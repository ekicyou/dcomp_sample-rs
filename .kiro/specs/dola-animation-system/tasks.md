# Implementation Tasks

| 項目 | 内容 |
|------|------|
| **Feature Name** | dola-animation-system |
| **Version** | 1.0 |
| **Date** | 2026-02-14 |
| **Requirements Base** | v1.7 |
| **Design Base** | v2.0 |

---

## Task List

### 1. クレート基盤構築とモジュールスケルトン作成

Dola クレートの基本構成を確立し、全モジュールの骨組みを準備する。

- [ ] 1.1 Cargo.toml セットアップ
  - ワークスペースメタデータ参照設定（version, edition, authors, license, publish）
  - serde 1.x 必須依存、derive feature 有効化（Req 5.6, 7.2）
  - serde_json/toml/serde_yaml をoptional dependencies として追加（Req 7.3）
  - feature gates 定義: `default = ["json"]`, `json`, `toml`, `yaml`（Req 5.1-5.3, 7.3）
  - description/package metadata 記述（Req 7.1）
- [ ] 1.2 モジュール構成とスケルトンファイル作成
  - `crates/dola/src/` 配下に9モジュールファイル生成: `lib.rs`, `document.rs`, `variable.rs`, `transition.rs`, `easing.rs`, `storyboard.rs`, `playback.rs`, `value.rs`, `validate.rs`, `error.rs`, `builder.rs`
  - `lib.rs` に public re-exports 骨組み作成（モジュール宣言 + `pub use` 準備）
  - 各モジュールに「TODO: Implement」マーカー配置
  - Rust 2024 Edition準拠確認（Req 7.1, 7.5）

**Requirements**: 5.6, 7.1, 7.2, 7.3, 7.5

---

### 2. (P) コアデータ型定義（Document/Variable/Value/Error）

Dola ドキュメントのルートコンテナ、変数定義、動的値型、エラー型を実装する。

- [ ] 2.1 (P) DolaDocument 構造体定義
  - `schema_version: String`, `variable: BTreeMap<String, AnimationVariableDef>`, `transition: BTreeMap<String, TransitionDef>`, `storyboard: BTreeMap<String, Storyboard>` フィールド
  - `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]`
  - BTreeMap で決定的なキー順序を保証（diff-friendly）（Req 1.5, 2.8, 4.1, 4.2, 5.1-5.7）
- [ ] 2.2 (P) AnimationVariableDef enum 定義
  - `#[serde(tag = "type")]` 内部タグ方式
  - Float/Integer/Object バリアント（Req 1.1, 1.2, 1.3）
  - Float: `initial: f64, min: Option<f64>, max: Option<f64>`（Req 1.1, 1.6）
  - Integer: `initial: i64, min: Option<i64>, max: Option<i64>, typewriter: Option<String>`（Req 1.2, 1.4, 1.6）
  - Object: `initial: DynamicValue`（Req 1.3, 1.7）
  - `#[serde(rename = "f64")]`, `#[serde(rename = "i64")]`, `#[serde(rename = "object")]`
- [ ] 2.3 (P) DynamicValue enum 定義
  - `#[serde(untagged)]` + 7バリアント: Null, Bool, Integer, Float, String, Array, Map
  - Integer を Float より前に定義（TOML整数/浮動小数点区別保持）（Req 1.3, 1.7）
  - Map は `BTreeMap<String, DynamicValue>` で決定的順序
- [ ] 2.4 (P) DolaError enum 定義
  - 全13エラーバリアント: SchemaVersionMismatch, DuplicateKeyframe, ReservedKeyframeName, UndefinedVariable, UndefinedTransition, UndefinedKeyframe, InvalidEntry, ObjectTransitionViolation, MutuallyExclusive, ValueOutOfRange, TypeMismatch（Req 5.5, design.md V1-V13）
  - 各バリアントに必要なコンテキストフィールド（storyboard名, entry_index, 変数名等）
  - `#[derive(Debug, Clone, PartialEq)]` + Display trait 実装（エラーメッセージ）
- [ ] 2.5* (P) Unit tests — DolaDocument/AnimationVariableDef/DynamicValue serde round-trip
  - DolaDocument の最小構成（schema_version のみ）のJSON serialize/deserialize
  - AnimationVariableDef 3バリアント（Float/Integer/Object）のJSON round-trip
  - DynamicValue 全7バリアントのJSON round-trip
  - BTreeMap の順序保証テスト（キー挿入順とシリアライズ順の独立性）

**Requirements**: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 2.8, 4.1, 4.2, 5.1-5.7

---

### 3. (P) Easing型定義とイージング関数列挙

イージング列挙型（名前付き + パラメトリック）を実装する。

- [ ] 3.1 (P) EasingFunction enum 定義
  - `#[serde(untagged)]` + 2バリアント: Named(EasingName), Parametric(ParametricEasing)
  - 文字列デシリアライズ → Named、オブジェクト → Parametric の順で試行（Req 2.7）
- [ ] 3.2 (P) EasingName enum 定義
  - `#[serde(rename_all = "snake_case")]` でシリアライズ時 snake_case 変換
  - 31バリアント: Linear（Req 2.2）、Quadratic/Cubic/Quartic/Quintic/Sine/Circular/Exponential/Elastic/Back/Bounce の In/Out/InOut（Req 2.3, 7.4）
  - interpolation::EaseFunction 命名準拠（コンパイル時依存なし）（Req 7.4）
- [ ] 3.3 (P) ParametricEasing enum 定義
  - `#[serde(tag = "type", rename_all = "snake_case")]` 内部タグ + snake_case
  - QuadraticBezier: `x0, x1, x2: f64`（Req 2.4）
  - CubicBezier: `x0, x1, x2, x3: f64`（Req 2.5）
- [ ] 3.4* (P) Unit tests — EasingFunction/EasingName/ParametricEasing serde round-trip
  - EasingName 全31バリアントのJSON round-trip（snake_case確認: `"quadratic_in"`, `"cubic_bezier"`）
  - ParametricEasing 2バリアントのJSON/TOML round-trip（内部タグ "type" 確認）
  - EasingFunction のNamed/Parametric両方のuntaggedデシリアライズ（文字列 → Named、オブジェクト → Parametric）

**Requirements**: 2.2, 2.3, 2.4, 2.5, 2.7, 7.4

---

### 4. (P) Transition型定義とハイブリッド参照

トランジション定義、値型、参照型を実装する。

- [ ] 4.1 (P) TransitionValue enum 定義
  - `#[serde(untagged)]` + 2バリアント: Scalar(f64), Dynamic(DynamicValue)
  - 数値 → Scalar、構造 → Dynamic の順で試行（Req 2.1, 2.6）
  - TOML整数の吸収戦略検討（カスタムデシリアライザで整数→Scalar変換）
- [ ] 4.2 (P) TransitionDef 構造体定義
  - `from: Option<TransitionValue>, to: Option<TransitionValue>, relative_to: Option<f64>, easing: Option<EasingFunction>, delay: f64, duration: Option<f64>`（Req 2.1, 2.10）
  - `delay` に `#[serde(default)]` でデフォルト 0.0
  - 不変条件コメント: to/relative_to排他、Object型制限、型整合性（design.md参照）
- [ ] 4.3 (P) TransitionRef enum 定義
  - `#[serde(untagged)]` + 2バリアント: Named(String), Inline(TransitionDef)
  - 文字列 → Named、オブジェクト → Inline の順で試行（Req 2.8, 2.9）
- [ ] 4.4* (P) Unit tests — TransitionValue/TransitionDef/TransitionRef serde round-trip
  - TransitionValue: Scalar(5.0), Dynamic({ path: "img.png" }) のJSON round-trip
  - TransitionDef: to/relative_to/easing/delay/duration 全フィールド組み合わせ
  - TransitionRef: Named("fade_in"), Inline(TransitionDef) のJSON/TOML round-trip

**Requirements**: 2.1, 2.6, 2.8, 2.9, 2.10

---

### 5. (P) Storyboard型定義とキーフレーム参照

ストーリーボード、エントリ、キーフレーム参照型、割り込みポリシーを実装する。

- [ ] 5.1 (P) InterruptionPolicy enum 定義
  - `#[serde(rename_all = "snake_case")]` + 5バリアント: Cancel, Conclude, Trim, Compress, Never（Req 4.9）
  - マルチプロセス協調設計のコメント記述（research.md Decision 10参照）
- [ ] 5.2 (P) Storyboard 構造体定義
  - `time_scale: f64, loop_count: Option<u32>, interruption_policy: InterruptionPolicy, entry: Vec<StoryboardEntry>`（Req 4.1, 4.3, 4.7, 4.8, 6.2, 4.9）
  - `time_scale` に `#[serde(default = "default_time_scale")]` でデフォルト 1.0
  - `interruption_policy` に `#[serde(default = "default_interruption_policy")]` でデフォルト Conclude
  - helper関数 `fn default_time_scale() -> f64 { 1.0 }`, `fn default_interruption_policy() -> InterruptionPolicy { InterruptionPolicy::Conclude }`
- [ ] 5.3 (P) StoryboardEntry 構造体定義
  - `variable: Option<String>, transition: Option<TransitionRef>, at: Option<KeyframeRef>, between: Option<BetweenKeyframes>, keyframe: Option<String>`（Req 3.2, 4.3, 4.4, 4.5, 4.6）
  - 4配置パターンのコメント記述: 前エントリ連結、KF起点、KF間、純粋KF
- [ ] 5.4 (P) KeyframeNames enum 定義
  - `#[serde(untagged)]` + 2バリアント: Single(String), Multiple(Vec<String>)（Req 4.4）
- [ ] 5.5 (P) KeyframeRef enum 定義
  - `#[serde(untagged)]` + 3バリアント: Single(String), Multiple(Vec<String>), WithOffset { keyframes: KeyframeNames, offset: f64 }（Req 4.4, 4.5）
  - `offset` に `#[serde(default)]` でデフォルト 0.0
- [ ] 5.6 (P) BetweenKeyframes 構造体定義
  - `from: String, to: String`（Req 4.4）
- [ ] 5.7* (P) Unit tests — Storyboard/StoryboardEntry/KeyframeRef serde round-trip
  - KeyframeRef 4形式: `"visible"`, `["a", "b"]`, `{ keyframes: "visible", offset: 0.5 }`, `{ keyframes: ["a", "b"], offset: 1.0 }` のJSON/TOML round-trip
  - StoryboardEntry 4配置パターン（前エントリ連結、at="visible"、between={from,to}、keyframe only）のJSON round-trip
  - Storyboard: time_scale/loop_count/interruption_policy のデフォルト値確認

**Requirements**: 3.1, 3.2, 3.3, 4.1, 4.3, 4.4, 4.5, 4.6, 4.7, 4.8, 4.9, 6.2

---

### 6. (P) Playback型定義と再生制御データモデル

再生状態列挙型とスケジューリング指示型を実装する。

- [ ] 6.1 (P) PlaybackState enum 定義
  - 5バリアント: Idle, Playing, Paused, Completed, Cancelled（Req 6.1）
  - `#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]`
- [ ] 6.2 (P) ScheduleRequest 構造体定義
  - `storyboard: String, start_time: f64`（Req 6.3, 6.5）
  - `#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]`
- [ ] 6.3* (P) Unit tests — PlaybackState/ScheduleRequest serde round-trip
  - PlaybackState 全5バリアントのJSON round-trip
  - ScheduleRequest のJSON round-trip（Req 6.4）

**Requirements**: 6.1, 6.3, 6.4, 6.5

---

### 7. バリデーション基盤と基本構造検証（V1-V6）

Validate trait とバリデーションルール V1-V6 を実装する。

- [ ] 7.1 Validate trait 定義
  - `trait Validate { fn validate(&self) -> Result<(), Vec<DolaError>>; }`
  - `impl Validate for DolaDocument`（Req 1.5, 1.6, cross）
  - エラー収集戦略: すべてのエラーを `Vec<DolaError>` に蓄積して一括返却
- [ ] ] 7.2 V1: スキーマバージョン検証
  - `schema_version` が期待値（例: "1.0"）と完全一致するか検証（Req 5.5）
  - 不一致時は `DolaError::SchemaVersionMismatch` 追加
- [ ] 7.3 V2-V3: キーフレーム名制約検証
  - V2: 同一SB内でキーフレーム名重複検出（明示的 `keyframe` フィールドのみ対象、暗黙的KFは除外）（Req 3.5）
  - V3: ユーザー定義キーフレーム名に "start" 使用禁止検証（Req 3.4）
  - 違反時は `DolaError::DuplicateKeyframe` / `DolaError::ReservedKeyframeName` 追加
- [ ] 7.4 V4-V5: 参照先存在確認
  - V4: 各エントリの `variable` が `document.variable` に存在するか（cross）
  - V5: TransitionRef::Named の名前が `document.transition` に存在するか（Req 2.8）
  - 違反時は `DolaError::UndefinedVariable` / `DolaError::UndefinedTransition` 追加
- [ ] 7.5 V6: キーフレーム参照検証（前方参照許可 + 暗黙的KF追跡）
  - **第1パス**: 各SBのentry配列を走査し、すべての`keyframe`フィールド（明示的）+ 暗黙的KF名（`keyframe`省略時に`__implicit_{index}`生成）を収集（Req 3.6, cross）
  - **第2パス**: `at`/`between` で参照されるKF名がすべて第1パスの集合に存在するか検証（前方参照許可: 宣言順序不問）
  - 違反時は `DolaError::UndefinedKeyframe` 追加
  - 暗黙的KF生成ロジック: entry index i で `keyframe` 省略 → `__implicit_{i}` を内部名として使用（`{i}` は0-based、例: 配列0番目→`__implicit_0`）
- [ ] 7.6* Unit tests — V1-V6バリデーション正常系・異常系
  - V1: スキーマバージョン "1.0" OK / "2.0" NG
  - V2: KF名 "visible" 重複検出
  - V3: KF名 "start" 使用禁止
  - V4: 未定義変数 "undefined_var" 参照エラー
  - V5: 未定義トランジション "undefined_trans" 参照エラー
  - V6: 前方参照OK（entry[2]でat="kf_from_entry_5"、entry[5]でkeyframe="kf_from_entry_5"）、未定義KF参照NG、暗黙的KF前方参照OK（entry[2]でat="__implicit_5"、entry[5]でkeyframe省略）

**Requirements**: 1.5, 1.6, 2.8, 3.4, 3.5, 3.6, 5.5, cross

---

### 8. (P) バリデーション高度制約（V7-V13）

エントリ構成、Object型制限、排他性、値域、型整合性の検証ルールを実装する。

- [ ] 8.1 (P) V7-V9: エントリ構成バリデーション
  - V7: `transition` あり → `variable` 必須（Req 4.5）
  - V8: `at` と `between` は排他（同時指定不可）（Req 4.5）
  - V9: 純粋KFエントリ（variable/transition なし）→ `keyframe` 必須（Req 4.6）
  - 違反時は `DolaError::InvalidEntry` 追加（reason フィールドに詳細）
- [ ] 8.2 (P) V10: Object型トランジション制限
  - 変数型が Object → トランジションは `to` のみ許可（`from`/`relative_to`/`easing` 不可）、`to` は `TransitionValue::Dynamic` のみ（Req 2.6）
  - 違反時は `DolaError::ObjectTransitionViolation` 追加
- [ ] 8.3 (P) V11: to/relative_to 排他性
  - TransitionDef で `to` と `relative_to` 同時指定禁止（Req 2.1）
  - 違反時は `DolaError::MutuallyExclusive` 追加
- [ ] 8.4 (P) V12: f64/i64 変数の値域検証
  - AnimationVariableDef の `initial` が `min`/`max` 範囲内か（Req 1.6）
  - TransitionValue::Scalar の値が変数の `min`/`max` 範囲内か（`from`/`to` 対象）
  - 違反時は `DolaError::ValueOutOfRange` 追加
- [ ] 8.5 (P) V13: 変数型とトランジション値型の整合性
  - f64/i64 変数 → `from`/`to` は `TransitionValue::Scalar` のみ（Dynamic 不可）（Req 2.1, 2.6）
  - Object 変数 → `to` は `TransitionValue::Dynamic` のみ（Scalar 不可）
  - 違反時は `DolaError::TypeMismatch` 追加
- [ ] 8.6* (P) Unit tests — V7-V13バリデーション正常系・異常系
  - V7: transition あり + variable なし → エラー
  - V8: at と between 同時指定 → エラー
  - V9: variable/transition なし + keyframe なし → エラー
  - V10: Object型変数 + from 指定 → エラー、Object型 + to=Scalar → エラー
  - V11: to と relative_to 同時指定 → エラー
  - V12: initial=1.5, max=1.0 → エラー、to=200.0, max=100.0 → エラー
  - V13: f64変数 + to=Dynamic → エラー、Object変数 + to=Scalar → エラー

**Requirements**: 1.6, 2.1, 2.6, 4.5, 4.6, cross

---

### 9. ビルダーAPI — DolaDocument構築ヘルパー

プログラマティックにDolaDocumentを構築するビルダーを実装する。

- [ ] 9.1 DolaDocumentBuilder 実装
  - `new(schema_version)`, `variable(name, def)`, `transition(name, def)`, `storyboard(name, sb)`, `build() -> Result<DolaDocument, Vec<DolaError>>` メソッド
  - `build()` で自動的に `validate()` を呼び出し、検証済みドキュメントを返す（cross）
  - メソッドチェーン可能な move semantics
- [ ] 9.2* Unit tests — DolaDocumentBuilder
  - 最小構成（schema_version のみ）→ build() → validate OK
  - variable/transition/storyboard 追加 → build() → serialize → deserialize → 一致検証
  - 不正データ（未定義変数参照）→ build() → Err(Vec<DolaError>) 確認

**Requirements**: cross-cutting（利便性）

---

### 10. ビルダーAPI — Storyboard構築ヘルパー

プログラマティックにStoryboardを構築するビルダーを実装する。

- [ ] 10.1 StoryboardBuilder 実装
  - `new()`, `time_scale(f64)`, `loop_count(u32)`, `interruption_policy(InterruptionPolicy)`, `entry(StoryboardEntry)`, `build() -> Storyboard` メソッド（cross）
  - メソッドチェーン可能な move semantics
- [ ] 10.2* Unit tests — StoryboardBuilder
  - デフォルト値（time_scale=1.0, interruption_policy=Conclude）確認
  - entry 追加 → build() → entry配列検証
  - time_scale/loop_count/interruption_policy 設定 → build() → フィールド確認

**Requirements**: cross-cutting（利便性）

---

### 11. (P) Serialization統合とフォーマット別テスト

feature gates の動作確認と全フォーマットの round-trip テストを実施する。

- [ ] 11.1 (P) Feature gates 動作検証
  - `cargo build --no-default-features` でserde_json無効確認
  - `cargo build --features toml` でtoml有効確認
  - `cargo build --features yaml` でserde_yaml有効確認
  - `cargo build --all-features` で全フォーマット有効確認（Req 7.3）
- [ ] 11.2* (P) Integration tests — JSON round-trip
  - 完全なDolaDocument（3変数、2トランジション、2SB、各SBに4配置パターン）のserialize → deserialize → 元データと一致（Req 5.1）
  - EasingName snake_case確認（`"quadratic_in_out"`）
  - InterruptionPolicy snake_case確認（`"conclude"`）
- [ ] 11.3* (P) Integration tests — TOML round-trip（feature `toml`）
  - 同じDolaDocumentのTOML round-trip（Req 5.2）
  - TOML配列インライン vs 改行挙動確認
  - BTreeMap キー順序の決定性確認
- [ ] 11.4* (P) Integration tests — YAML round-trip（feature `yaml`）
  - 同じDolaDocumentのYAML round-trip（Req 5.3）
  - YAMLインデント・構造の可読性確認

**Requirements**: 5.1, 5.2, 5.3, 7.3

---

### 12. (P) E2Eテスト、エッジケース、互換性検証

全体統合、境界条件、フォーマット互換性の包括的テストを実施する。

- [ ] 12.1* (P) E2Eテスト — 全配置パターン統合
  - Builder API → 4配置パターンSB構築 → validate → JSON serialize → deserialize → validate再実行（cross）
  - 暗黙的KF生成（`keyframe`省略）→ 前エントリ連結 → バリデーションOK（Req 3.6）
- [ ] 12.2* (P) エッジケーステスト
  - 空のストーリーボード（entry配列0件）→ serialize/deserialize OK
  - 純粋KFエントリのみのSB → バリデーションOK（Req 4.6）
  - タイプライター変数（`typewriter: "こんにちは"`）+ duration=3.0 トランジション（Req 1.4）
  - ベジェイージング付きインライントランジション（Req 2.4, 2.5）
  - `delay` のみ（`duration` 省略）→ 即時遷移（Req 2.10）
  - `at = "start"` でSB開始時点から配置（Req 3.1）
  - `at = ["a", "b"]` で複数KF完了待機（Req 4.4）
  - `at = { keyframes: "visible", offset: 0.5 }` のオフセット付きKF指定（Req 4.4）
  - Object型トランジション `to = { path: "image.png" }` の Dynamic 値（Req 2.6）
  - 値域超過（initial > max, to < min）→ V12エラー検出（Req 1.6）
  - f64変数にDynamic値指定 → V13エラー検出（Req 2.1, 2.6）
- [ ] 12.3* (P) 互換性テスト
  - TOML整数 `from = 5` vs 浮動小数点 `from = 5.0` の相互変換（カスタムデシリアライザ動作確認）
  - DynamicValue のフォーマット間互換性（JSON → TOML → JSON → 元データと一致）
  - KeyframeRef の4形式 JSON/TOML/YAML 3フォーマット互換性（Single/Multiple/WithOffset-single/WithOffset-multiple）

**Requirements**: 1.4, 1.6, 2.1, 2.4, 2.5, 2.6, 2.10, 3.1, 3.6, 4.4, 4.6, cross

---

## Progress Summary

- **Total Major Tasks**: 12
- **Total Sub-Tasks**: 46 (including 17 test tasks marked with `*`)
- **Parallel-Capable Tasks**: 6 major tasks (Tasks 2-6, 8), 8 sub-tasks with explicit `(P)` markers
- **Requirements Coverage**: All 43 acceptance criteria mapped to implementation tasks

---

## Notes

- **Parallel Execution**: Tasks marked with `(P)` can run concurrently with other tasks once their prerequisites (if any) are satisfied. Task 1 is a prerequisite for all subsequent tasks. Tasks 2-6 are independent core type definitions. Task 7 depends on Tasks 2-6. Task 8 can run in parallel with Task 7. Tasks 9-10 depend on Tasks 2-6 and 7. Task 11 depends on Tasks 2-6. Task 12 can run in parallel with Task 11.
- **Test Tasks**: Sub-tasks marked with `*` are test-focused and can be deferred until after MVP if necessary (as per `tasks-generation.md` optional test coverage rule). However, all tests are recommended for complete AC validation.
- **暗黙的キーフレーム生成**: `{index}` は entry 配列内の 0-based index（例: entry[0] → `__implicit_0`, entry[2] → `__implicit_2`）。設計書 design.md および requirements.md Req 3.6 参照。
- **InterruptionPolicy**: Req 4.9 で追加。マルチプロセス協調設計、デフォルトは Conclude。research.md Decision 10 参照。
- **Easing/InterruptionPolicy snake_case**: design.md Q2-Q3 で明確化。Rust バリアント名は PascalCase、シリアライズ形式は snake_case（`#[serde(rename_all = "snake_case")]`）。
