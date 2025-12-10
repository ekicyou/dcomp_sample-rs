# Research: pasta-serialization

## R1: Rune VM Context Passing Implementation

### 調査目的
Rust側から構造体またはハッシュマップをRune VM実行時に引数として渡し、Rune関数内でアクセスする実装方法を確定する。

### 調査内容

#### 1. Rune VM Execution API

**`vm.execute(hash, args)` の型シグネチャ**:
```rust
pub fn execute<A, N>(
    &mut self,
    hash: Hash,
    args: A
) -> Result<N, VmError>
where
    A: Args,        // Args トレイトを実装した型
    N: FromValue,   // 戻り値の型変換
```

**`Args`トレイトの実装**:
- タプル型（`()`, `(T,)`, `(T1, T2)`, ...）が`Args`トレイトを自動実装
- 各要素は`ToValue`トレイトを実装している必要がある

#### 2. Option A: Struct を引数として渡す

**POC実装**:

```rust
use rune::{Context, Vm, Sources, to_value};
use std::sync::Arc;

// コンテキスト構造体定義
#[derive(Debug)]
struct ExecutionContext {
    persistence_path: String,
}

// Rune script
let script = r#"
pub fn test_label(ctx) {
    // ctx はタプルの第1要素としてアクセス
    let path = ctx.persistence_path;
    yield path;
}
"#;

// Rune準備
let mut context = Context::with_default_modules()?;
let runtime = Arc::new(context.runtime()?);
let mut sources = Sources::new();
sources.insert(Source::new("test", script)?)?;
let unit = rune::prepare(&mut sources).with_context(&context).build()?;

// VM実行
let mut vm = Vm::new(runtime, Arc::new(unit));
let hash = rune::Hash::type_hash(&["test_label"]);

// コンテキスト構築
let ctx = ExecutionContext {
    persistence_path: "/path/to/persistence".to_string(),
};

// ⚠️ 問題: ExecutionContext は ToValue を実装していない
// 解決策: rune::Any derive を使用
```

**改良版 - `rune::Any` derive使用**:

```rust
use rune::{Any, ContextError, Module};

#[derive(Any, Debug, Clone)]
#[rune(item = ::execution_context)]  // Rune側でのモジュールパス
struct ExecutionContext {
    #[rune(get)]  // getter自動生成
    persistence_path: String,
}

// モジュール登録が必要
fn create_context_module() -> Result<Module, ContextError> {
    let mut module = Module::new();
    module.ty::<ExecutionContext>()?;
    Ok(module)
}

// Context初期化時にモジュール登録
context.install(create_context_module()?)?;

// VM実行
let ctx = ExecutionContext {
    persistence_path: "/path/to/persistence".to_string(),
};

// タプルとして渡す（Argsトレイト実装済み）
let execution = vm.execute(hash, (ctx,))?;

// Rune側でのアクセス
// pub fn test_label(ctx) {
//     let path = ctx.persistence_path;  // getter経由
//     yield path;
// }
```

**課題**:
- カスタム構造体は`rune::Any` deriveとモジュール登録が必要
- `#[rune(get)]`でgetter自動生成、またはメソッド手動実装が必要

#### 3. Option B: HashMap を引数として渡す

**POC実装**:

```rust
use std::collections::HashMap;
use rune::to_value;

// HashMap構築
let mut ctx = HashMap::new();
ctx.insert("persistence_path".to_string(), "/path/to/persistence".to_string());

// rune::Value に変換
let ctx_value = to_value(ctx)?;

// タプルとして渡す
let execution = vm.execute(hash, (ctx_value,))?;

// Rune側でのアクセス
// pub fn test_label(ctx) {
//     let path = ctx["persistence_path"];  // インデックスアクセス
//     yield path;
// }
```

**利点**:
- カスタム型定義・モジュール登録不要
- `HashMap<String, String>`は`ToValue`を自動実装
- 動的フィールド追加が容易

**欠点**:
- 型安全性が低い（Rune側でのキー名ミス）
- フィールド名がコンパイル時に検証されない

#### 4. Option C: 単純な値（String）を引数として渡す

**POC実装**:

```rust
let persistence_path = "/path/to/persistence".to_string();

// String は ToValue を実装済み
let execution = vm.execute(hash, (persistence_path,))?;

// Rune側でのアクセス
// pub fn test_label(path) {
//     yield path;
// }
```

**利点**:
- 最もシンプル
- 型変換不要

**欠点**:
- 拡張性なし（将来的に複数フィールド追加時に引数増加）

### 推奨アプローチ

**Option B (HashMap)** を推奨

**理由**:
1. **実装の簡潔性**: カスタム型定義・モジュール登録が不要
2. **拡張性**: 将来的なコンテキストフィールド追加が容易
3. **Gap Analysisとの整合性**: "構造体またはハッシュマップ"という要件に合致
4. **Rune側の自然な構文**: `ctx["persistence_path"]` は一般的なパターン

**実装例**:

```rust
// crates/pasta/src/engine.rs

impl PastaEngine {
    fn build_execution_context(&self) -> Result<rune::Value, PastaError> {
        let mut ctx = std::collections::HashMap::new();
        
        // persistence_path フィールド設定
        let path_str = if let Some(ref path) = self.persistence_path {
            path.to_string_lossy().to_string()
        } else {
            String::new()  // 空文字列 = パスなし
        };
        
        ctx.insert("persistence_path".to_string(), path_str);
        
        // 将来的な拡張ポイント（コメントアウト例）
        // ctx.insert("engine_version".to_string(), env!("CARGO_PKG_VERSION").to_string());
        // ctx.insert("script_name".to_string(), self.script_name.clone());
        
        rune::to_value(ctx).map_err(|e| {
            PastaError::RuneRuntimeError(format!("Failed to build context: {}", e))
        })
    }
    
    fn execute_label_with_filters(&mut self, ...) -> Result<Vec<ScriptEvent>> {
        // ... (既存のラベル検索ロジック)
        
        let mut vm = Vm::new(self.runtime.clone(), self.unit.clone());
        let hash = rune::Hash::type_hash(&[fn_name.as_str()]);
        
        // コンテキスト構築
        let context = self.build_execution_context()?;
        
        // VM実行（タプルとして渡す）
        let execution = vm.execute(hash, (context,))
            .map_err(|e| PastaError::VmError(e))?;
        
        // ... (既存のジェネレータ処理)
    }
}
```

### Rune側のアクセスパターン

**トランスパイラ生成コード**:
```rune
pub fn label_name(ctx) {
    // persistence_path 取得
    let path = ctx["persistence_path"];
    
    // パスが設定されているか確認
    if path.is_empty() {
        // エラーハンドリング（永続化機能を使用しない）
    } else {
        // 永続化処理（TOML保存等）
        let full_path = format!("{}/save_data.toml", path);
        // ... (ファイルI/O)
    }
    
    // 既存のラベル処理
    yield emit_text("こんにちは");
}
```

### トランスパイラ変更箇所

**`crates/pasta/src/transpiler/mod.rs`** (Line 155):

```rust
// Before:
output.push_str(&format!("pub fn {}() {{\n", fn_name));

// After:
output.push_str(&format!("pub fn {}(ctx) {{\n", fn_name));
```

**影響範囲**:
- すべてのグローバルラベル・ローカルラベルのシグネチャが変更
- 既存テストへの影響: Rune関数は未使用引数を許容するため、`ctx`を使用しないラベルも正常動作

### 検証テストコード

```rust
#[test]
fn test_context_passing_with_persistence_path() -> Result<()> {
    let script = r#"
＊test
    さくら：こんにちは
"#;

    let temp_dir = tempfile::TempDir::new()?;
    let mut engine = PastaEngine::new_with_persistence(script, temp_dir.path())?;
    
    // 実行時にコンテキストが渡されることを確認
    let events = engine.execute_label("test")?;
    
    // イベント検証（既存ロジック）
    assert_eq!(events.len(), 2);
    
    Ok(())
}

#[test]
fn test_context_passing_without_persistence_path() -> Result<()> {
    let script = r#"
＊test
    さくら：やあ
"#;

    let mut engine = PastaEngine::new(script)?;  // 永続化パスなし
    
    // persistence_path が空文字列で渡されることを確認
    let events = engine.execute_label("test")?;
    
    assert_eq!(events.len(), 2);
    
    Ok(())
}
```

---

## R1 調査結果サマリ

✅ **実装方法確定**: HashMap + `rune::to_value` + タプル引数  
✅ **型要件**: `HashMap<String, String>` は `ToValue` 自動実装済み  
✅ **Rune側構文**: `ctx["persistence_path"]` でアクセス  
✅ **トランスパイラ変更**: シグネチャを `pub fn label_name(ctx)` に変更  
✅ **既存テスト影響**: なし（未使用引数は許容）  

**Status**: 完了 - 設計フェーズで実装可能  
**Next**: R2調査へ進む
