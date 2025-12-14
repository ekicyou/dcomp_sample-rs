# Requirements Document: pasta-engine-doctest-fix

| 項目 | 内容 |
|------|------|
| **Document Title** | PastaEngine Doctest 修正 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-12-14 |
| **Priority** | P2 (Documentation Quality) |
| **Status** | Requirements Generated |

---

## Introduction

本要件定義書は、`crates/pasta/src/engine.rs` に含まれる4つのdoctestが、古いAPI仕様（1引数の `new(script)`）を参照しており、現在のAPI（2引数の `new(script_root, persistence_root)`）と不一致のため失敗している問題を修正する。

### Background

**pasta-transpiler-pass2-output** 仕様の実装中に発見された既知の問題。

#### 現在のAPI（実装済み）
```rust
pub fn new(
    script_root: impl AsRef<Path>,
    persistence_root: impl AsRef<Path>
) -> Result<Self>
```

#### Doctest内の古い呼び出し（4箇所）
```rust
let engine = PastaEngine::new(script)?;  // ❌ 1引数
```

### Problem Statement

**課題1: API仕様の不一致**

doctestが古いAPI仕様を参照しており、以下のコンパイルエラーが発生：

```
error[E0061]: this function takes 2 arguments but 1 argument was supplied
```

**課題2: ドキュメントの信頼性低下**

- ユーザーがdoctestを参考にした際、動作しないコードを提供してしまう
- `cargo test --doc` が失敗するため、CIパイプラインで警告が出る

**課題3: 該当箇所の特定**

失敗しているdoctestは以下の4箇所：

1. `crates/pasta/src/engine.rs:35` - `PastaEngine` 構造体のドキュメント
2. `crates/pasta/src/engine.rs:435` - `find_event_handlers()` メソッド
3. `crates/pasta/src/engine.rs:478` - `on_event()` メソッド
4. `crates/pasta/src/engine.rs:550` - `execute_label_chain()` メソッド

### Scope

**含まれるもの：**

1. **Doctest修正**
   - 4箇所のdoctestを現在のAPI仕様に合わせて修正
   - `no_run` 属性の適切な使用（ファイルシステムアクセスが必要なため）

2. **テスト実行の確認**
   - `cargo test --doc` が成功することを確認

**含まれないもの：**

- APIの変更（現在の2引数APIは正しい）
- 実装コードの変更（doctestのみ修正）
- 他のドキュメントの更新（READMEなど）

---

## Requirements

### Requirement 1: PastaEngine構造体のdoctest修正

**Objective:** ユーザーとして、`PastaEngine` 構造体のドキュメント例が現在のAPIに準拠し、正しい使用方法を理解できるようにする。

**Location:** `crates/pasta/src/engine.rs:35-51`

#### Acceptance Criteria

1. When doctestをコンパイルする, the Doctest shall 2引数の `new(script_root, persistence_root)` を使用する
2. When テストディレクトリを使用する, the Doctest shall 一時ディレクトリまたは適切なテストパスを `script_root` として指定する
3. When 永続化ディレクトリを指定する, the Doctest shall 一時ディレクトリまたは適切なテストパスを `persistence_root` として指定する
4. When `execute_label` を呼び出す, the Doctest shall 2引数の `execute_label(label_name, args)` を使用する（現在のAPI確認が必要）
5. When doctestを実行する, the Doctest shall エラーなくコンパイルが通る

#### 現在のコード

```rust
/// ```no_run
/// use pasta::PastaEngine;
///
/// let script = r#"
/// ＊挨拶
///     さくら：こんにちは！
///     うにゅう：やあ！
/// "#;
///
/// let mut engine = PastaEngine::new(script)?;  // ❌ 1引数
/// let events = engine.execute_label("挨拶")?;  // ❌ 引数数不明
///
/// for event in events {
///     println!("{:?}", event);
/// }
/// # Ok::<(), pasta::PastaError>(())
/// ```
```

#### 修正方針

- スクリプト文字列をファイルに書き出してディレクトリ構造を作成
- または、既存のテストフィクスチャディレクトリを参照
- `execute_label` の正しいシグネチャを確認して修正

---

### Requirement 2: find_event_handlers() メソッドのdoctest修正

**Objective:** 開発者として、`find_event_handlers()` メソッドの使用例が現在のAPIに準拠していることを確認する。

**Location:** `crates/pasta/src/engine.rs:435-449`

#### Acceptance Criteria

1. When doctestをコンパイルする, the Doctest shall 2引数の `new(script_root, persistence_root)` を使用する
2. When `find_event_handlers` を呼び出す, the Doctest shall 正しいメソッドシグネチャを使用する
3. When doctestを実行する, the Doctest shall エラーなくコンパイルが通る

#### 現在のコード

```rust
/// ```no_run
/// # use pasta::PastaEngine;
/// let script = r#"
/// ＊OnClick
///     さくら：クリックされました！
///
/// ＊OnDoubleClick
///     さくら：ダブルクリック！
/// "#;
/// let engine = PastaEngine::new(script)?;  // ❌ 1引数
///
/// let handlers = engine.find_event_handlers("Click");
/// assert!(handlers.contains(&"OnClick".to_string()));
/// # Ok::<(), pasta::PastaError>(())
/// ```
```

---

### Requirement 3: on_event() メソッドのdoctest修正

**Objective:** 開発者として、`on_event()` メソッドの使用例が現在のAPIに準拠していることを確認する。

**Location:** `crates/pasta/src/engine.rs:478-489`

#### Acceptance Criteria

1. When doctestをコンパイルする, the Doctest shall 2引数の `new(script_root, persistence_root)` を使用する
2. When `on_event` を呼び出す, the Doctest shall 正しいメソッドシグネチャを使用する
3. When doctestを実行する, the Doctest shall エラーなくコンパイルが通る

#### 現在のコード

```rust
/// ```no_run
/// # use pasta::PastaEngine;
/// # use std::collections::HashMap;
/// let script = r#"
/// ＊OnClick
///     さくら：クリックされました！
/// "#;
/// let mut engine = PastaEngine::new(script)?;  // ❌ 1引数
///
/// let events = engine.on_event("Click", HashMap::new())?;
/// # Ok::<(), pasta::PastaError>(())
/// ```
```

---

### Requirement 4: execute_label_chain() メソッドのdoctest修正

**Objective:** 開発者として、`execute_label_chain()` メソッドの使用例が現在のAPIに準拠していることを確認する。

**Location:** `crates/pasta/src/engine.rs:550-565`

#### Acceptance Criteria

1. When doctestをコンパイルする, the Doctest shall 2引数の `new(script_root, persistence_root)` を使用する
2. When `execute_label` を呼び出す, the Doctest shall 正しいメソッドシグネチャ（引数数を確認）を使用する
3. When doctestを実行する, the Doctest shall エラーなくコンパイルが通る

#### 現在のコード

```rust
/// ```no_run
/// # use pasta::PastaEngine;
/// let script = r#"
/// ＊挨拶
///     さくら：おはよう！
///
/// ＊挨拶_続き
///     さくら：今日も元気だね！
/// "#;
/// let mut engine = PastaEngine::new(script)?;  // ❌ 1引数
///
/// // Execute chain manually
/// let mut all_events = engine.execute_label("挨拶")?;  // ❌ 引数数不明
/// all_events.extend(engine.execute_label("挨拶_続き")?);  // ❌ 引数数不明
/// # Ok::<(), pasta::PastaError>(())
/// ```
```

---

### Requirement 5: Doctest実行の検証

**Objective:** 開発者として、全てのdoctestが正常にコンパイル・実行され、ドキュメント品質が保証されることを確認する。

#### Acceptance Criteria

1. When `cargo test --doc --package pasta` を実行する, the Test Suite shall 全6 doctestがパスする（現在2 passed, 4 failed）
2. When doctestが失敗する, the Test Suite shall エラーメッセージを表示しない
3. When CIパイプラインを実行する, the CI shall doctestの失敗によるビルドエラーを報告しない

---

## Technical Context

### 現在のAPI（実装済み）

#### PastaEngine::new()

```rust
pub fn new(
    script_root: impl AsRef<Path>,
    persistence_root: impl AsRef<Path>
) -> Result<Self>
```

**パラメータ:**
- `script_root`: スクリプトルートディレクトリ（絶対パス）
  - `dic/` ディレクトリを含む
  - `main.rn` を含む
- `persistence_root`: 永続化ルートディレクトリ（絶対または相対パス）
  - `variables.toml` を保存

#### execute_label() (要確認)

```rust
pub fn execute_label(&mut self, label_name: &str, args: Vec<Value>) -> Result<Vec<ScriptEvent>>
```

**パラメータ確認が必要:**
- 現在のシグネチャが `execute_label(label_name)` か `execute_label(label_name, args)` かを確認
- doctestで正しい引数数を使用

### テスト戦略

#### Doctest修正パターン

**Option A: テストフィクスチャディレクトリを使用**
```rust
/// ```no_run
/// use pasta::PastaEngine;
/// use std::path::PathBuf;
///
/// let script_root = PathBuf::from("tests/fixtures/test-project");
/// let persistence_root = PathBuf::from("tests/fixtures/test-project/persistence");
/// let mut engine = PastaEngine::new(&script_root, &persistence_root)?;
/// # Ok::<(), pasta::PastaError>(())
/// ```
```

**Option B: 一時ディレクトリを使用（推奨）**
```rust
/// ```no_run
/// use pasta::PastaEngine;
/// use std::fs;
/// use std::path::PathBuf;
///
/// // Create temporary directories
/// let temp_dir = std::env::temp_dir().join("pasta_doctest");
/// let script_root = temp_dir.join("script");
/// let persistence_root = temp_dir.join("persistence");
/// fs::create_dir_all(&script_root)?;
/// fs::create_dir_all(&persistence_root)?;
///
/// let mut engine = PastaEngine::new(&script_root, &persistence_root)?;
/// # Ok::<(), pasta::PastaError>(())
/// ```
```

**Option C: no_run属性を維持（簡潔）**
```rust
/// ```no_run
/// use pasta::PastaEngine;
/// use std::path::PathBuf;
///
/// let script_root = PathBuf::from("/path/to/script");
/// let persistence_root = PathBuf::from("/path/to/persistence");
/// let mut engine = PastaEngine::new(&script_root, &persistence_root)?;
/// # Ok::<(), pasta::PastaError>(())
/// ```
```

**推奨:** Option C（`no_run`属性付き）
- doctestは構文例を示すことが目的
- 実際の実行は単体テストで行う
- 簡潔で読みやすい

---

## Implementation Notes

### 修正優先順位

1. **Phase 1: API仕様の調査**（必須）
   - `execute_label()` の正しいシグネチャを確認
   - 他のメソッドの引数も確認

2. **Phase 2: Doctest修正**（必須）
   - 4箇所のdoctestを修正
   - `no_run` 属性を維持

3. **Phase 3: テスト実行**（必須）
   - `cargo test --doc --package pasta` で検証

### 影響範囲

- **変更ファイル**: `crates/pasta/src/engine.rs` のみ
- **変更箇所**: docコメント4箇所（約20行）
- **影響**: なし（ドキュメント修正のみ）

---

## Testing Strategy

### Doctest検証

| テストケース | 期待結果 |
|-------------|---------|
| `cargo test --doc --package pasta` | 全6 doctest pass |
| `PastaEngine` 構造体のdoctest | コンパイル成功 |
| `find_event_handlers()` のdoctest | コンパイル成功 |
| `on_event()` のdoctest | コンパイル成功 |
| `execute_label_chain()` のdoctest | コンパイル成功 |

---

## References

- **関連仕様:** `.kiro/specs/pasta-transpiler-pass2-output/` (この仕様実装中に発見)
- **Implementation Report:** `.kiro/specs/pasta-transpiler-pass2-output/implementation-report.md` (既知の問題セクション)
- **ソースコード:** `crates/pasta/src/engine.rs`
