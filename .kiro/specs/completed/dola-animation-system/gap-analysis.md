# Dola Animation System — ギャップ分析レポート v2.0

| 項目 | 内容 |
|------|------|
| **対象仕様** | dola-animation-system |
| **要件バージョン** | v1.6 |
| **分析日** | 2026-02-14 |
| **分析種別** | グリーンフィールド新規設計 + 既存コードベース規約調査 + 依存クレート評価 |

---

## 1. 分析サマリー

- **完全なグリーンフィールド**: `crates/dola/` は未作成。serde・シリアライズクレート・interpolation いずれもワークスペースに未登録。既存コードベースとの直接的な依存・衝突は一切ない
- **スコープ明確化済み**: v1.6 で要件が 7 件 / 42 受け入れ基準に絞り込まれ、ランタイム（再生エンジン・値補間計算・プラグインIF・時間管理）はすべてスコープ外。純粋なデータモデル+バリデーション+ビルダーの設計に集中可能
- **WAM COM ラッパーとの責務分離は明確**: `com/animation.rs` は薄い COM trait 拡張のみ（2 種のトランジション）。Dola はプラットフォーム非依存層として完全独立。将来の統合層が橋渡しを担う
- **技術的リスクは低い**: serde derive ベースのデータモデル定義が中心。主要な設計判断は「イージング列挙型の粒度」と「TOML/JSON/YAML 3 フォーマット互換構造」の 2 点
- **ワークスペース統合は容易**: `members = ["crates/*"]` グロブにより `crates/dola/` 作成で自動包含

---

## 2. 既存コードベース調査

### 2.1 ワークスペース構成

| 項目 | 現状 | Dola に必要なもの |
|------|------|-------------------|
| ワークスペースメンバー | `members = ["crates/*"]` グロブ | `crates/dola/` 作成で自動包含 |
| Rust Edition | 2024（`workspace.package`） | 同一 edition を継承 |
| `serde` | ワークスペース依存に未登録 | 必須依存として追加 |
| `serde_json` | 未登録 | feature `json`（デフォルト有効）で追加 |
| `toml` (serde_toml) | 未登録 | feature `toml` で追加 |
| `serde_yaml` | 未登録 | feature `yaml` で追加 |
| `interpolation` | 未登録・未使用 | コンパイル時依存なし（命名準拠のみ） |

### 2.2 既存アニメーション関連コード

#### `com/animation.rs` — WAM COM ラッパー

薄い trait 拡張メソッドのみ。ビジネスロジックや抽象化は一切なし。

| WAM COM インターフェース | ラップ済み操作 | Dola 対応概念 |
|---|---|---|
| `IUIAnimationTimer` | `create_animation_timer()`, `get_time()` | スコープ外（ランタイム） |
| `IUIAnimationManager2` | `create_animation_variable()`, `update()`, `create_storyboard()` | `AnimationVariableDef` / `Storyboard` のデータモデル |
| `IUIAnimationTransitionLibrary2` | AccelerateDecelerate, Cubic の 2 種のみ | `TransitionDef` の 30+ イージング |
| `IUIAnimationStoryboard2` | `schedule()`, `add_transition()`, `add_keyframe_after_transition()`, `add_transition_at_keyframe()` | `StoryboardEntry` / キーフレーム配置 |
| `IUIAnimationVariable2` | `get_value()`, `get_curve()` (DirectComposition連携) | スコープ外（ランタイム） |

**所見**: WAM ラッパーにはイージング関数 2 種しかなく、Dola の 30 種 + ベジェとは大きな差がある。ただし Dola はランタイム補間を行わないため、これはギャップではなく設計上の差異。

#### ECS レイヤー

アニメーション固有の ECS コンポーネントは**一切存在しない**。`transition` のヒットはすべてドラッグ操作の状態遷移（`DragTransition`）であり無関係。

#### serde 使用実績

**wintf クレート内で serde の直接使用はゼロ**。Dola がプロジェクト内初の serde 直接利用クレートとなる。

### 2.3 規約抽出

| 規約カテゴリ | 既存パターン | Dola への適用 |
|---|---|---|
| ファイル命名 | `snake_case.rs` | 同一 |
| 型命名 | `PascalCase` | 同一 |
| モジュール構造 | `com/` → `ecs/` のレイヤー分離 | Dola は単層（データモデル + バリデーション） |
| エラー型 | `windows::core::Result` | Dola 独自の `DolaError` enum |
| テスト配置 | `tests/` ディレクトリ（integration tests） | 同一パターン + 単体テスト |
| ログ | `tracing` クレート | トレースは必要に応じて（データモデルクレートなので最小限） |

---

## 3. 要件対アセットマップ

### Requirement 1: アニメーション変数

| AC | 技術要素 | 既存アセット | ギャップ |
|----|----------|-------------|---------|
| 1.1 f64 連続値 | `AnimationVariableDef` enum variant | **Missing** | enum + serde derive |
| 1.2 i64 離散値 | `AnimationVariableDef` enum variant | **Missing** | 同上（f64 補間→i64 丸め はランタイム） |
| 1.3 Object 型 | `AnimationVariableDef` enum variant + `DynamicValue` | **Missing** | DynamicValue 型設計が必要 |
| 1.4 タイプライター属性 | i64 変数のメタデータ | **Missing** | 属性フィールドの設計 |
| 1.5 一意な名前 | `HashMap<String, _>` | **Missing** | バリデーションで重複チェック |
| 1.6 初期値・値域 | 構造体フィールド | **Missing** | f64/i64 の min/max + initial |
| 1.7 Object 初期値 | `DynamicValue` | **Missing** | DynamicValue 型設計に依存 |

**複雑性**: S（Small） — 標準的な enum + serde derive パターン。

### Requirement 2: トランジション

| AC | 技術要素 | 既存アセット | ギャップ |
|----|----------|-------------|---------|
| 2.1 パラメータ定義 | `TransitionDef` struct | **Missing** | from/to/relative_to/easing/delay/duration |
| 2.2 Linear イージング | `EasingFunction` enum | **Missing** | Linear variant |
| 2.3 組み込み30種 | `EasingFunction` enum | **Missing** | interpolation 命名準拠の自前 enum |
| 2.4 二次ベジェ | `EasingFunction::QuadraticBezier` | **Missing** | 3 制御点パラメトリック variant |
| 2.5 三次ベジェ | `EasingFunction::CubicBezier` | **Missing** | 4 制御点パラメトリック variant |
| 2.6 Object 型制限 | バリデーション | **Missing** | to のみ許可、他フィールド禁止の検証 |
| 2.7 serde 実装 | `#[derive(Serialize, Deserialize)]` | **Missing** | 自前 enum で直接 serde 対応 |
| 2.8 名前付きテンプレート | `HashMap<String, TransitionDef>` | **Missing** | ドキュメントレベルの名前参照 |
| 2.9 ハイブリッド参照 | `TransitionRef` (String | Object) | **Missing** | `#[serde(untagged)]` パターン（**Research Finding 2**） |
| 2.10 総時間計算 | delay + duration | **Missing** | バリデーション/ビルダー |

**複雑性**: M（Medium） — イージング列挙型の定義量が多い。`#[serde(untagged)]` によるハイブリッド参照の設計が要注意。

### Requirement 3: キーフレーム

| AC | 技術要素 | 既存アセット | ギャップ |
|----|----------|-------------|---------|
| 3.1 予約 "start" | 定数/バリデーション | **Missing** | 予約名チェック |
| 3.2 名前付き KF | `keyframe` フィールド | **Missing** | StoryboardEntry のフィールド |
| 3.3 SBローカルスコープ | バリデーション | **Missing** | SB 単位の名前解決 |
| 3.4 "start" 予約チェック | バリデーション | **Missing** | ユーザー定義禁止の検証 |
| 3.5 重複禁止 | バリデーション | **Missing** | SB 内名前重複チェック |
| 3.6 暗黙的 KF 生成 | 内部キーフレーム名生成 | **Missing** | v1.5 追加。未命名エントリへの内部名付与ロジック |

**複雑性**: S-M — バリデーションロジックが中心。暗黙的 KF 生成は設計の要点。

### Requirement 4: ストーリーボード

| AC | 技術要素 | 既存アセット | ギャップ |
|----|----------|-------------|---------|
| 4.1 複数 SB 定義 | `HashMap<String, Storyboard>` | **Missing** | ドキュメントレベル |
| 4.2 一意な名前 | バリデーション | **Missing** | HashMap キーで暗黙保証 |
| 4.3 エントリ配列 | `Vec<StoryboardEntry>` | **Missing** | 配列構造 |
| 4.4 エントリフィールド | `StoryboardEntry` struct | **Missing** | v1.6: at がキーフレーム名配列に変更 |
| 4.5 3種配置 | 配置方式の判別 | **Missing** | v1.5/v1.6: 前エントリ連結(暗黙KF)、at(配列+全KF完了待機)、between |
| 4.6 純粋 KF エントリ | `StoryboardEntry` (variable/transition省略) | **Missing** | Option フィールド |
| 4.7 メタ情報 | `Storyboard` のメタフィールド | **Missing** | loop/timescale 等 |
| 4.8 同一タイムライン | データモデル制約 | **Missing** | 構造的に保証 |

**複雑性**: M-L — 3 種配置方式の表現と暗黙的 KF 連結がデータモデル設計の核心。`at` の配列化 (v1.6) で `KeyframeRef` の設計にも影響。

### Requirement 5: シリアライズフォーマット

| AC | 技術要素 | 既存アセット | ギャップ |
|----|----------|-------------|---------|
| 5.1 JSON | `serde_json` | **Missing** | feature `json` + デシリアライズ関数 |
| 5.2 TOML | `toml` クレート | **Missing** | feature `toml` + TOML 構造制約への対応 |
| 5.3 YAML | `serde_yaml` | **Missing** | feature `yaml` |
| 5.4 スキーマバージョン | `DolaDocument::schema_version` | **Missing** | バージョン文字列フィールド |
| 5.5 バージョン不一致エラー | バリデーション | **Missing** | デシリアライズ後チェック |
| 5.6 serde 使用 | `#[derive(Serialize, Deserialize)]` | **Missing** | 全型に適用 |
| 5.7 全データモデル対応 | 全型にシリアライズ対応 | **Missing** | derive で網羅 |

**複雑性**: S — serde derive で大部分対応。TOML のネスト構造制約 (Research Finding 3) への対応が唯一の注意点。

### Requirement 6: 再生制御データモデル

| AC | 技術要素 | 既存アセット | ギャップ |
|----|----------|-------------|---------|
| 6.1 再生状態 enum | `PlaybackState` | **Missing** | 5 状態の列挙型 |
| 6.2 タイムスケール | f64 フィールド | **Missing** | デフォルト 1.0 |
| 6.3 時間表現 f64秒 | 型設計 | **Missing** | 全時間値を f64 で統一 |
| 6.4 シリアライズ対応 | serde derive | **Missing** | derive で対応 |
| 6.5 スケジュール指示 | `ScheduleRequest` | **Missing** | SB 名 + 開始時刻 |

**複雑性**: S — 単純な enum + struct 定義。

### Requirement 7: クレート構成

| AC | 技術要素 | 既存アセット | ギャップ |
|----|----------|-------------|---------|
| 7.1 独立クレート | `crates/dola/` | **Missing** | ディレクトリ + Cargo.toml 作成 |
| 7.2 serde 必須 | Cargo.toml 依存 | **Missing** | `serde = { version = "1", features = ["derive"] }` |
| 7.3 feature フラグ | Cargo.toml features | **Missing** | json/toml/yaml の feature gate |
| 7.4 interpolation 命名準拠 | EasingFunction enum 名 | **Missing** | 自前 enum で命名合わせ（コンパイル依存なし） |
| 7.5 no_std 不要 | N/A | N/A | 制約なし |

**複雑性**: S — Cargo.toml + lib.rs のスキャフォールド。

---

## 4. 実装アプローチ評価

### 4.1 プロジェクト種別

Dola は**完全なグリーンフィールド**プロジェクト。既存コードベースの拡張ではなく、新規クレートの作成。したがって Option A（既存拡張）は該当しない。

### Option B: 新規クレート作成（推奨）

**根拠**: Dola は wintf に依存しない独立クレート（Req 7.1）。既存コードとの結合点がゼロ。

**構成案**:
```
crates/dola/
├── Cargo.toml
├── src/
│   ├── lib.rs          # 公開 API + re-exports
│   ├── model/          # データモデル定義
│   │   ├── mod.rs
│   │   ├── variable.rs   # AnimationVariableDef
│   │   ├── transition.rs # TransitionDef, TransitionRef, EasingFunction
│   │   ├── storyboard.rs # Storyboard, StoryboardEntry, KeyframeRef
│   │   ├── playback.rs   # PlaybackState, ScheduleRequest
│   │   └── document.rs   # DolaDocument (top-level)
│   ├── validation.rs   # 分離バリデーション
│   ├── builder.rs      # オプショナルビルダー
│   └── error.rs        # DolaError
└── tests/
    ├── serde_roundtrip_test.rs
    ├── validation_test.rs
    └── builder_test.rs
```

**統合ポイント**:
- ワークスペース: `members = ["crates/*"]` で自動包含
- 将来の wintf 統合: `crates/wintf/Cargo.toml` に `dola = { path = "../dola" }` を追加（将来の別仕様）
- feature gate: `json`（default）, `toml`, `yaml`

**トレードオフ**:
- ✅ 完全な責務分離、独立テスト可能
- ✅ 将来の crates.io 公開に対応（名前 "dola" は利用可能確認済み）
- ✅ wintf 以外のプロジェクトからも利用可能
- ❌ wintf との統合層は将来の別仕様で別途設計が必要

### Option C: ハイブリッドアプローチ（不採用）

Dola のスコープは純粋なデータモデル+バリデーションであり、既存コードとの結合が不要なため、ハイブリッドアプローチは不要。

---

## 5. 残存する設計判断事項

以下は要件 v1.6 でもカバーしきれていない設計判断事項。設計フェーズで解決する。

| # | 設計判断事項 | 関連要件 | 影響 |
|---|---|---|---|
| D1 | **値域（上限・下限）超過時の挙動** — クランプ？エラー？無視？WAM はクランプ | Req 1.6 | バリデーション設計 |
| D2 | **TransitionValue と Object 型 `to` の表現** — TransitionValue は Scalar(f64) のみ。Object 型トランジションの `to`（DynamicValue）をどう型安全に表現するか | Req 2.1, 2.6 | 型設計の核心。design.md 再生成時に解決 |
| D3 | **サブストーリーボード（入れ子）** — 再利用のために SB を入れ子にできるか？WAM にはこの概念なし | Req 4 | データモデル拡張性 |
| D4 | **ループ区間指定** — SB 全体ループのみか、部分区間ループもサポートか | Req 4.7 | メタ情報の粒度 |
| D5 | **スキーマバージョン互換性ポリシー** — マイナー後方互換？完全一致のみ？ | Req 5.4, 5.5 | 長期運用設計 |
| D6 | **`at` 配列 + `offset` の型表現** — 単純文字列配列か、構造体か、`#[serde(untagged)]` か | Req 4.4 | KeyframeRef の serde 設計 |

---

## 6. 候補クレート評価（更新版）

### 6.1 serde エコシステム

| クレート | 用途 | バージョン | 備考 |
|---|---|---|---|
| `serde` | 必須基盤 | 1.x | `features = ["derive"]` |
| `serde_json` | JSON フォーマット | 1.x | feature `json`（default） |
| `toml` | TOML フォーマット | 0.8.x | feature `toml` |
| `serde_yaml` | YAML フォーマット | 0.9.x | feature `yaml` |

### 6.2 イージング計算クレート

**設計判断済み（Research Decision 1）**: `interpolation` クレートの `EaseFunction` 命名に準拠した自前列挙型を定義。コンパイル時依存なし。

理由:
- `interpolation` の `EaseFunction` は serde 未対応。newtype ラッパーか `#[serde(remote)]` が必要で煩雑
- 自前定義なら serde derive を直接付与可能
- Dola はデータモデルのみ（値補間計算はランタイムの責務）なので、補間ロジック自体は不要
- 将来のランタイムが `interpolation` / `keyframe` / 自前実装のいずれかを選択可能

---

## 7. 実装複雑性・リスク評価

| 要件 | 工数 | リスク | 根拠 |
|------|------|--------|------|
| Req 1: アニメーション変数 | S | Low | enum + serde derive。DynamicValue 型設計のみ要注意 |
| Req 2: トランジション | M | Low | 30+ イージング enum 定義量は多いが機械的。untagged 参照の設計が要注意 |
| Req 3: キーフレーム | S | Low | バリデーション中心。暗黙的 KF 生成は自然な拡張 |
| Req 4: ストーリーボード | M | Medium | 3 種配置方式 + at 配列化 + 暗黙 KF 連結がデータモデル設計の核心 |
| Req 5: シリアライズ | S | Low | serde derive で大部分対応。TOML ネスト制約のみ注意 |
| Req 6: 再生制御モデル | S | Low | 単純な enum + struct |
| Req 7: クレート構成 | S | Low | Cargo.toml + lib.rs スキャフォールド |

**総合工数**: M（3-7 日）
**総合リスク**: Low-Medium

> v1.0 分析時の総合工数 L / リスク Medium-High からの下方修正。理由:
> - ランタイム（再生エンジン・値補間・プラグインIF・時間管理）がスコープ外に移行
> - 要件が 9→7 件、受け入れ基準が明確化
> - データモデル + バリデーション + ビルダーの純粋なライブラリ設計に集中可能

---

## 8. 設計フェーズへの推奨事項

### 優先的に解決すべき設計判断

1. **D2: TransitionValue と Object 型 `to` の表現** — 型設計の核心。design.md 再生成時に A/B/C 案から決定
2. **D6: `at` 配列 + `offset` の KeyframeRef 設計** — v1.6 変更の反映
3. **D1: 値域超過挙動** — バリデーション戦略の基盤

### 設計フェーズで確定不要（将来仕様に委譲）

- D3（サブ SB）: v1 では未サポートが妥当。拡張可能な設計にしておくのみ
- D4（ループ区間）: v1 では SB 全体ループのみが現実的
- D5（バージョン互換性）: 初期リリースでは完全一致で十分

### Research Items（設計フェーズへの引き継ぎ）

- `#[serde(untagged)]` による TransitionRef / KeyframeRef のハイブリッド参照パターンの TOML/JSON/YAML 3 フォーマット互換性検証
- DynamicValue の JSON/TOML/YAML 各フォーマットでの表現力差異の調査
- Storyboard の TOML ネスト構造 `[storyboard.X]` + `[[storyboard.X.entry]]` と JSON/YAML フラット構造の互換性確認

---

## 9. 次のステップ

ギャップ分析が完了。設計フェーズに進むには：

```
/kiro-spec-design dola-animation-system -y
```

> **注意**: 既存の design.md は v1.4 要件ベース。v1.5（暗黙的 KF）・v1.6（at 配列化）の変更および本ギャップ分析の結果を反映した再生成が必要。
