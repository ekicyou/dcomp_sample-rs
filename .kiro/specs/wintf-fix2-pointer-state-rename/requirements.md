# Requirements Document

## Project Description (Input)
`PointerState.screen_point` フィールドの命名を `client_point` に修正する。現在のフィールド名は `screen_point` だが、実際に保持している値はクライアント座標（物理 px）であり、名前と内容が不一致。値の変更は行わず、純粋なリネーム作業。

## 親仕様からの参照情報

### 調査レポート参照
- **親仕様**: `dpi-coordinate-transform-survey`
- **根拠セクション**: report.md §1.2 項目2, §2.1（PointerState行）, §6.4.2, §8.2 Step 2, §7.1 G6

### 調査結果サマリ
- `ecs/pointer/mod.rs` L116-145 の `PointerState.screen_point` は、名前が `screen_point`（スクリーン座標を示唆）だが、実際にはクライアント座標（物理 px）を保持している
- ドラッグ処理は別経路で正しいスクリーン座標を使用しているため、現在の命名不整合による **実害は限定的** だが、可読性・保守性の観点から修正すべき
- 値そのものは変更しない（クライアント座標・物理 px のまま）

### 関連するコード箇所
| ファイル | 行番号 | 内容 |
|---------|--------|------|
| `crates/wintf/src/ecs/pointer/mod.rs` | L116-145 | `PointerState` 構造体定義 — `screen_point` フィールド |
| （参照箇所） | — | `screen_point` を参照する全コード（リネーム対象） |

### 依存関係
- **前提条件**: なし（Step 1 と独立して実施可能）
- **後続仕様**: なし

### コスト見積もり
- Low（数時間）

### 検証基準（調査レポートより）
- コンパイル通過
- 既存動作に変更なし

## Introduction

`PointerState` コンポーネントの `screen_point` フィールドを `client_point` にリネームする。
このフィールドは名前が「スクリーン座標」を示唆しているが、実際に保持しているのはクライアント座標（物理 px）であり、名前と内容が不一致である。
本仕様は純粋なリネーム作業であり、値の計算ロジックやデータフローには一切変更を加えない。

## Requirements

### Requirement 1: PointerState フィールド名の変更

**Objective:** 開発者として、`PointerState.screen_point` フィールドが実際に保持する値（クライアント座標・物理 px）と名前を一致させたい。それにより、コードの可読性と保守性を向上させる。

#### Acceptance Criteria

1. The wintf library shall `PointerState` 構造体の `screen_point` フィールドを `client_point` にリネームする
2. The wintf library shall リネーム後も `client_point` フィールドの型を `PhysicalPoint` のまま維持する
3. The wintf library shall `client_point` フィールドの doc コメントを「クライアント座標（物理ピクセル）」に修正する

### Requirement 2: 全参照箇所の一括更新

**Objective:** 開発者として、`screen_point` を参照する全コードが新名称 `client_point` に更新されることで、コンパイルエラーなく一貫した命名を保ちたい。

#### Acceptance Criteria

1. The wintf library shall `PointerState` の初期化箇所（`handlers.rs` 内の構造体リテラル・4箇所）のフィールド名を `client_point` に更新する
2. The wintf library shall `PointerState` の `Default` 実装内のフィールド名を `client_point` に更新する
3. The wintf library shall `pointer/mod.rs` 内の `screen_point` フィールドアクセス箇所（約12箇所）を `client_point` に更新する
4. The wintf library shall サンプルコード（`taffy_flex_demo.rs`）内の `screen_point` フィールドアクセス箇所（約10行）を `client_point` に更新する
5. The wintf library shall `pointer/mod.rs` 内のユニットテスト（L878）の `screen_point` 参照を `client_point` に更新する

### Requirement 3: リネーム対象外の明確化

**Objective:** 開発者として、別概念である `screen_point` 変数・パラメータが誤ってリネームされないことで、正しい座標系の説明を維持したい。

#### Acceptance Criteria

1. The wintf library shall `nchittest_cache.rs` 内の `screen_point`（WM_NCHITTEST 用・実際のスクリーン座標）を変更しない
2. The wintf library shall `hit_test.rs` 内の `screen_point` パラメータ（ヒットテスト用変数名）を変更しない
3. The wintf library shall `PointerState.screen_point` フィールド以外の同名ローカル変数やパラメータを変更しない

### Requirement 4: コメント・ドキュメント整合性

**Objective:** 開発者として、リネーム後のコードコメントが新しいフィールド名 `client_point` と整合していることで、混乱を防ぎたい。

#### Acceptance Criteria

1. The wintf library shall `PointerState` 構造体の `client_point` フィールドの doc コメントに「クライアント座標（物理ピクセル）」と記述する
2. When `screen_point` を参照するコメントが `PointerState` のフィールドを指している場合（例: `pointer/mod.rs` L509「Phase 1ではscreen_pointと同じ」）, the wintf library shall そのコメントを `client_point` に修正する
3. The wintf library shall `PointerState` のフィールドを指していないコメント内の `screen_point`（`nchittest_cache.rs`, `hit_test.rs` 等）は変更しない

### Requirement 5: ビルド・動作検証

**Objective:** 開発者として、リネーム後にコンパイルが通り既存動作に影響がないことを確認したい。

#### Acceptance Criteria

1. When リネーム完了後, the wintf library shall `cargo build` が警告なしで成功する
2. When リネーム完了後, the wintf library shall `cargo test` が全テストパスする
3. The wintf library shall リネーム前後で `client_point`（旧 `screen_point`）に格納される値を変更しない
