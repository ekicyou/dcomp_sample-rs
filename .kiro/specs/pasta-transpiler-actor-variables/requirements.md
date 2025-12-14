# Requirements Document: pasta-transpiler-actor-variables

| 項目 | 内容 |
|------|------|
| **Document Title** | Pasta トランスパイラー アクター変数参照修正 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-12-14 |
| **Priority** | P1 (Correctness) |
| **Status** | Requirements Generated |

---

## Introduction

本要件定義書は、Pasta DSLトランスパイラーがアクター代入時に文字列リテラルではなく、変数参照を生成するように修正する。現在の実装では `ctx.actor = "さくら"` のように文字列として出力しているが、正しくは `ctx.actor = さくら` のようにアクター変数（オブジェクト）を参照すべきである。

### Background

**現在の実装（誤り）:**

```rune
pub fn __start__(ctx, args) {
    ctx.actor = "さくら";        // 文字列リテラル
    yield Actor("さくら");       // 文字列リテラル
    yield Talk("こんにちは");
}
```

**参照実装（正しい）:**

```rune
// main.rn で定義
pub const さくら = #{
    name: "さくら",
    id: "sakura",
};

pub const うにゅう = #{
    name: "うにゅう",
    id: "unyuu",
};

// トランスパイル出力
pub mod メイン_1 {
    use super::{さくら, うにゅう};
    
    pub fn __start__(ctx) {
        ctx.actor = さくら;          // 変数参照（オブジェクト）
        yield #{ type: "Actor", actor: ctx.actor };
        yield #{ type: "Talk", text: "こんにちは" };
    }
}
```

### Problem Statement

**課題1: 型の不一致**

- **現在**: 文字列 `"さくら"` を代入
- **期待**: オブジェクト `#{ name: "さくら", id: "sakura" }` を代入

**課題2: 拡張性の欠如**

文字列では以下が不可能：
- アクターID（内部識別子）の管理
- アクター属性（表示名、画像パス、声優情報など）の拡張
- 実行時のアクター情報参照

**課題3: 設計仕様との乖離**

参照実装（`comprehensive_control_flow.rn`）では変数参照が使用されており、トランスパイラー出力が設計意図に反する。

**課題4: 発見の経緯**

`pasta-transpiler-pass2-output` 仕様の検証中、Rune VMコンパイル検証で発見。Pass 2出力は正常だが、Pass 1（アクター代入）に問題があることが判明。

### Scope

**含まれるもの：**

1. **Pass 1 の Statement::Speech 処理修正**
   - `ctx.actor = "さくら"` → `ctx.actor = さくら` に変更
   - `yield Actor("さくら")` → `yield Actor(さくら)` または `yield #{ type: "Actor", actor: ctx.actor }` に変更

2. **モジュールレベルの use 文生成**
   - `use super::{さくら, うにゅう};` などのインポート文を生成
   - スクリプト内で使用される全アクターを自動検出してインポート

3. **テスト更新**
   - トランスパイラー出力の検証パターン更新
   - Rune VMコンパイル検証（アクター変数の解決確認）

**含まれないもの：**

- `main.rn` のアクター定義生成（手動またはプロジェクト初期化で対応）
- アクター情報の動的登録機能
- Pass 2 の修正（`pasta-transpiler-pass2-output` で完了済み）

---

## Requirements

### Requirement 1: アクター変数参照の生成

**Objective:** トランスパイラー開発者として、アクター代入時に文字列ではなく変数を参照することで、型安全性と拡張性を確保する。

#### Acceptance Criteria

1. When 会話文（`Statement::Speech`）をトランスパイルする, the Pasta Transpiler shall `ctx.actor = さくら;` のように変数名（識別子）を出力する
2. When アクター名に日本語が含まれる, the Pasta Transpiler shall そのまま識別子として使用する（`さくら`, `うにゅう`, `ななこ` など）
3. When アクター名に記号が含まれる, the Pasta Transpiler shall サニタイズせず、そのまま出力する（Rune識別子として有効な場合）
4. When 文字列リテラルを出力しない, the Pasta Transpiler shall ダブルクォートで囲まない
5. When トランスパイラーがエラーを検出しない, the Pasta Transpiler shall アクターが `main.rn` で定義されていることは検証しない（Rune VMコンパイル時に検証）

#### 現在のコード（修正対象）

**場所**: `crates/pasta/src/transpiler/mod.rs:353`

```rust
// Generate speaker change (store as string)
writeln!(writer, "        ctx.actor = \"{}\";", speaker)
    .map_err(|e| PastaError::io_error(e.to_string()))?;
```

#### 修正後のコード

```rust
// Generate speaker change (store as actor variable)
writeln!(writer, "        ctx.actor = {};", speaker)
    .map_err(|e| PastaError::io_error(e.to_string()))?;
```

---

### Requirement 2: Actor イベント生成の修正

**Objective:** スクリプト実行者として、アクター変更イベントが正しいオブジェクト構造で生成されることを保証する。

#### Acceptance Criteria

1. When アクター変更を出力する, the Pasta Transpiler shall `yield #{ type: "Actor", actor: ctx.actor };` の形式で出力する
2. When アクター情報を参照する, the Pasta Transpiler shall `ctx.actor` から現在のアクターを取得する
3. When 文字列リテラルを使用しない, the Pasta Transpiler shall `yield Actor("さくら")` の形式を使用しない
4. When オブジェクトリテラルを生成する, the Pasta Transpiler shall Rune構文 `#{ key: value }` を使用する
5. When type フィールドを含める, the Pasta Transpiler shall 固定値 `"Actor"` を使用する

#### 現在のコード（修正対象）

**場所**: `crates/pasta/src/transpiler/mod.rs:355`

```rust
writeln!(writer, "        yield Actor(\"{}\");", speaker)
    .map_err(|e| PastaError::io_error(e.to_string()))?;
```

#### 修正後のコード

```rust
writeln!(writer, "        yield #{{ type: \"Actor\", actor: ctx.actor }};")
    .map_err(|e| PastaError::io_error(e.to_string()))?;
```

**注記**: `yield Actor(さくら)` の形式も可能だが、参照実装に合わせてオブジェクトリテラル形式を推奨。

---

### Requirement 3: モジュールレベル use 文の生成

**Objective:** 開発者として、生成されたモジュールが必要なアクター変数をインポートし、Rune VMコンパイルが成功することを保証する。

#### Acceptance Criteria

1. When モジュール（`pub mod ラベル名_N`）を生成する, the Pasta Transpiler shall モジュール冒頭に `use super::{アクター1, アクター2, ...};` を出力する
2. When アクターリストを収集する, the Pasta Transpiler shall 当該モジュール内の全 `Statement::Speech` から `speaker` を抽出する
3. When 重複を排除する, the Pasta Transpiler shall 同じアクター名は1回のみインポートする
4. When アクター名をソートする, the Pasta Transpiler shall 決定論的な順序（アルファベット順）でインポートする
5. When アクターが0個の場合, the Pasta Transpiler shall use 文を出力しない

#### 生成例

```rune
pub mod メイン_1 {
    use pasta_stdlib::*;
    use crate::actors::*;  // アクター変数のインポート

    pub fn __start__(ctx, args) {
        ctx.actor = さくら;
        yield #{ type: "Actor", actor: ctx.actor };
        // ...
    }
}
```

---

### Requirement 4: テスト出力の検証

**Objective:** 開発者として、トランスパイラーが正しいコードを生成し、Rune VMでコンパイルが成功することを確認する。

#### Acceptance Criteria

1. When トランスパイラーテストを実行する, the Test Suite shall 生成コードに `ctx.actor = さくら;` が含まれることを検証する（文字列リテラルでないこと）
2. When トランスパイラーテストを実行する, the Test Suite shall 生成コードに `use super::{さくら, ...};` が含まれることを検証する
3. When Rune VMコンパイルテストを実行する, the Test Suite shall `main.rn` と統合してコンパイルが成功することを確認する
4. When 生成コードを検証する, the Test Suite shall ダブルクォートで囲まれたアクター名（`"さくら"`）が存在しないことを確認する
5. When 全テストを実行する, the Test Suite shall 既存の単体・統合テストが全てパスすることを確認する

---

### Requirement 5: 後方互換性の考慮

**Objective:** 開発者として、既存のテストやフィクスチャが新しい出力形式に対応することを保証する。

#### Acceptance Criteria

1. When フィクスチャを更新する, the Development Team shall `comprehensive_control_flow.transpiled.rn` を新しい形式で再生成する
2. When 参照実装と一致させる, the Development Team shall 生成コードが `comprehensive_control_flow.rn` と同じ構造を持つことを確認する
3. When テストケースを更新する, the Development Team shall 検証パターンを新しい出力に合わせて修正する
4. When 既存機能を維持する, the Pasta Transpiler shall Pass 2 出力（`__pasta_trans2__`, `pasta` モジュール）は変更しない
5. When 全テストがパスする, the Test Suite shall 268以上のテストが成功することを確認する

---

## Technical Context

### 現在の実装

**ファイル**: `crates/pasta/src/transpiler/mod.rs`

**関数**: `transpile_statement_to_writer()` (334-424行目)

**修正箇所**: Statement::Speech の処理（346-362行目）

```rust
Statement::Speech {
    speaker,
    content,
    span: _,
} => {
    // Generate speaker change (store as string)
    writeln!(writer, "        ctx.actor = \"{}\";", speaker)  // ← 修正1
        .map_err(|e| PastaError::io_error(e.to_string()))?;
    writeln!(writer, "        yield Actor(\"{}\");", speaker)  // ← 修正2
        .map_err(|e| PastaError::io_error(e.to_string()))?;

    // Generate talk content
    for part in content {
        Self::transpile_speech_part_to_writer(writer, part, &context)?;
    }
}
```

**追加箇所**: `transpile_global_label()` / `transpile_local_label()` 内

モジュール生成時にアクターリストを収集し、use 文を出力する。

### 修正戦略

#### Option A: 即座に変数参照に変更（推奨）

**利点**:
- 参照実装との完全な一致
- 型安全性の向上
- 拡張性の確保

**欠点**:
- テストフィクスチャの更新が必要
- 検証パターンの修正が必要

#### Option B: 設定フラグで切り替え

**利点**:
- 後方互換性の維持
- 段階的な移行が可能

**欠点**:
- 複雑性の増加
- メンテナンスコストの増加

**推奨**: Option A（即座に修正）

理由:
- 文字列形式は設計ミスであり、維持する理由がない
- 早期に修正することで技術的負債を回避
- テスト更新のコストは限定的

### 影響範囲

| コンポーネント | 影響 | 対応 |
|--------------|------|------|
| `transpile_statement_to_writer()` | 修正必要 | Statement::Speech の処理を変更 |
| `transpile_global_label()` | 修正必要 | use 文の生成を追加 |
| `transpile_local_label()` | 修正必要 | use 文の生成を追加 |
| `comprehensive_control_flow.transpiled.rn` | 更新必要 | トランスパイラーで再生成 |
| テスト検証パターン | 更新必要 | 文字列リテラル検証を変数参照検証に変更 |
| Pass 2 出力 | 影響なし | `__pasta_trans2__`, `pasta` は不変 |

---

## Testing Strategy

### Unit Tests

| テストケース | 入力 | 期待される出力 |
|-------------|------|--------------|
| **アクター変数参照** | `さくら：こんにちは` | `ctx.actor = さくら;` |
| **Actorイベント生成** | 同上 | `yield #{ type: "Actor", actor: ctx.actor };` |
| **use文生成** | 2人のアクター | `use super::{さくら, うにゅう};` |
| **重複排除** | 同じアクター2回 | アクター名は1回のみ |
| **アクター0個** | アクターなし | use 文なし |

### Integration Tests

1. **Rune VMコンパイル検証**:
   - `comprehensive_control_flow.pasta` のトランスパイル
   - `main.rn` と統合してRune VMコンパイル
   - アクター変数の解決確認

2. **参照実装との一致**:
   - 生成コードと `comprehensive_control_flow.rn` の構造比較
   - アクター代入形式の一致確認

3. **既存テストの継続**:
   - 268テスト全てがパス
   - Pass 2 出力の不変性確認

---

## Implementation Notes

### 修正優先順位

1. **Phase 1: Statement::Speech 処理の修正**（必須）
   - 文字列リテラルから変数参照に変更
   - Actor イベント生成の修正

2. **Phase 2: use 文生成の追加**（必須）
   - アクターリスト収集ロジック
   - モジュール冒頭への use 文出力

3. **Phase 3: テスト更新**（必須）
   - フィクスチャ再生成
   - 検証パターン更新
   - Rune VMコンパイル検証

### コード変更見積もり

- **修正箇所**: 3箇所（Statement::Speech 処理、use 文生成 ×2）
- **追加行数**: 約30行（use 文生成ロジック）
- **修正行数**: 約5行（文字列リテラル → 変数参照）
- **テスト更新**: 約10テストケース

---

## Dependencies

| 依存仕様/コンポーネント | 理由 | 状態 |
|---------------------|------|------|
| `pasta-transpiler-pass2-output` | Pass 2 出力は変更しない | ✅ Completed |
| `main.rn` (手動作成) | アクター定義が必要 | 既存 |
| `LabelRegistry` | ラベル情報管理 | 既存、変更不要 |

---

## Future Work

- **アクター定義の自動生成**: スクリプト解析からアクターリストを抽出し、`main.rn` のテンプレートを生成
- **アクター属性の拡張**: 画像パス、声優情報、表示名（ローカライズ）などの追加
- **動的アクター登録**: 実行時のアクター追加機能

---

## References

- **発見元**: `.kiro/specs/pasta-transpiler-pass2-output/KNOWN-ISSUES.md`
- **参照実装**: `crates/pasta/tests/fixtures/comprehensive_control_flow.rn` (1-12行目: アクター定義、14-36行目: 使用例)
- **現在の実装**: `crates/pasta/src/transpiler/mod.rs` (346-362行目: Statement::Speech 処理)
- **関連仕様**: `pasta-transpiler-pass2-output` (Pass 2 出力修正、完了済み)
