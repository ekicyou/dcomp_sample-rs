# Implementation Gap Analysis

| 項目 | 内容 |
|------|------|
| **Document Title** | Pasta DSL 宣言的コントロールフロー ギャップ分析 |
| **Version** | 1.0 |
| **Date** | 2025-12-11 |
| **Feature** | pasta-declarative-control-flow |
| **Phase** | Gap Analysis |

---

## 分析サマリー

**スコープ**: 現在のトランスパイラー実装（`crates/pasta/src/transpiler/mod.rs`）を、要件定義で規定された正しい宣言的コントロールフロー構文に適合させるための全面的な再設計。

**主要な課題**:
- 現在の実装は要件5（トランスパイラー出力仕様）と4つの主要な乖離点を持つ
- `04_control_flow.pasta`に仕様外の命令型構文（`＠if`/`＠elif`/`＠else`/`＠while`）が含まれている
- Pastaランタイム（`ctx.pasta.call()`/`ctx.pasta.jump()`）が未実装
- 親仕様（areka-P0-script-engine）はcall/jump/ラベルのみを定義しており、命令型構文は存在しない

**推奨アプローチ**: Option B（新規コンポーネント作成）- トランスパイラーとランタイムの両方を再設計し、Pastaランタイム層を新規追加

**複雑度**: M（3-7日）- トランスパイラーロジックとRuneコード生成パターンの全面的な変更が必要

**リスク**: Medium - 既存のAST定義は再利用可能だが、出力形式と実行時動作の両方を変更する必要がある

---

## 1. 現状調査

### 1.1. 既存資産

#### トランスパイラー（`crates/pasta/src/transpiler/mod.rs`）

**現在の構造**:
- `Transpiler::transpile()`: PastaFile AST → Runeコード文字列の変換
- グローバルラベル → フラット関数生成（`pub fn ラベル名_番号(ctx)`）
- ローカルラベル → フラット化された関数（`pub fn 親名__子名_番号(ctx)`）
- Call文 → 直接関数呼び出し（`ラベル名()`）
- Jump文 → `return ラベル名()`

**問題点（要件5との乖離）**:
1. **モジュール化なし**: グローバルラベルがモジュールとして生成されていない（要件AC 2）
2. **`__start__`関数なし**: グローバルラベルの最初のスコープが専用関数として生成されていない（要件AC 3）
3. **ローカルラベルのフラット化**: 親モジュール内に配置されず、`親名__子名`形式でフラット化されている（要件AC 4）
4. **直接関数呼び出し**: `ctx.pasta.call()`/`ctx.pasta.jump()`ではなく、直接関数を呼び出している（要件AC 7-9）

**再利用可能な部分**:
- ✅ AST定義（`Statement`, `JumpTarget`, `LabelDef`等）
- ✅ 基本的なトランスパイルロジックの枠組み（`transpile_statement()`, `transpile_expr()`）
- ✅ 識別子サニタイズ（`sanitize_identifier()`）
- ✅ 文字列エスケープ（`escape_string()`）

#### AST定義（`crates/pasta/src/parser/ast.rs`）

**現在の定義**:
```rust
pub enum Statement {
    Speech { speaker, content, span },
    Call { target, filters, args, span },
    Jump { target, filters, span },
    VarAssign { name, scope, value, span },
    RuneBlock { content, span },
}

pub enum JumpTarget {
    Local(String),
    Global(String),
    LongJump { global, local },
    Dynamic(String),
}
```

**状態**: ✅ 要件1-4に適合している。変更不要。

**注**: `Statement`に`If`/`Elif`/`Else`/`While`バリアントは存在しない。これらは`04_control_flow.pasta`に誤って記述されているだけで、パーサーでは認識されていない。

#### ランタイム（`crates/pasta/src/runtime/`）

**現在の構成**:
- `labels.rs`: ラベルテーブル、ランダム選択、属性フィルタリング
- `variables.rs`: 変数管理（グローバル/ローカル）
- `random.rs`: ランダム選択器
- `generator.rs`: プレースホルダー（現在は空実装）

**欠落機能**:
- ❌ **Pastaランタイムオブジェクト（`ctx.pasta`）**: 未実装
- ❌ **`ctx.pasta.call()`**: callロジック、引数保存・復元、yield伝播
- ❌ **`ctx.pasta.jump()`**: jumpロジック、yield伝播
- ❌ **`ctx.pasta.word()`**: 単語展開（要件5で言及されているが詳細は後続仕様）
- ❌ **`ctx.pasta.add_words()`/`commit_words()`**: 単語定義（要件5 AC 5）

**再利用可能な部分**:
- ✅ `LabelTable`: ラベル解決、前方一致選択、キャッシュベース消化
- ✅ `RandomSelector`: ランダム選択ロジック
- ✅ `VariableManager`: 変数管理

#### サンプルスクリプト（`crates/pasta/examples/scripts/04_control_flow.pasta`）

**現在の状態**: ❌ 仕様外構文を含む
- `＠if：{条件}` / `＠elif：{条件}` / `＠else` - 命令型条件分岐（親仕様に存在しない）
- `＠while：{条件}` - 命令型ループ（親仕様に存在しない）
- `＠jump：ラベル` - 正しい構文だが、誤った文脈で使用されている

**問題の本質**: GRAMMAR.mdに`＠if`等が記載されているが、親仕様（areka-P0-script-engine）には存在しない。これは実装時の誤解による追加と思われる。

### 1.2. アーキテクチャパターン

**現在のパターン**:
1. **フラットな関数生成**: すべてのラベルをトップレベル関数として生成
2. **直接関数呼び出し**: call/jumpを直接関数呼び出し・returnに変換
3. **ジェネレーター関数シグネチャ**: `pub fn ラベル名(ctx) { yield ... }`

**要件が求めるパターン**:
1. **モジュール化**: グローバルラベル → Runeモジュール（`pub mod ラベル名_番号 { ... }`）
2. **`__start__`関数**: グローバルラベルの最初のスコープ専用関数
3. **ローカルラベル関数**: 親モジュール内の個別関数
4. **ランタイム経由の呼び出し**: `ctx.pasta.call(ctx, "親", "子", [args])` + while-let-yield

**パターン比較**:

| 側面 | 現在 | 要件 |
|------|------|------|
| グローバルラベル | フラット関数 | モジュール + `__start__` |
| ローカルラベル | `親__子`関数 | 親モジュール内関数 |
| call | `子()` | `ctx.pasta.call(ctx, "親", "子", [])` |
| jump | `return 子()` | `ctx.pasta.jump(ctx, "親", "子", [])` |
| 引数 | 未対応 | `ctx.args`配列経由 |
| yield伝播 | 直接yield | while-let-yield |

### 1.3. 統合サーフェス

**Runeランタイムとの統合**:
- ✅ 現在のエンジン（`crates/pasta/src/engine.rs`）はRune VMを使用してRuneコードを実行
- ✅ スタンダードライブラリ（`crates/pasta/src/stdlib/`）でRune関数を登録
- ❌ `ctx`オブジェクト構造が未定義（要件5で構造が規定されている）

**必要な統合ポイント**:
1. **`ctx`オブジェクトの構築**: Rune Object型として、6つのフィールド（pasta/actor/scope/save/args）を持つ
2. **`ctx.pasta`の実装**: Rustで実装したPastaランタイムメソッドをRune関数として登録
3. **発言者オブジェクト（`ctx.actor`）**: グローバル変数として定義される発言者オブジェクト
4. **スコープ情報（`ctx.scope`）**: 現在のグローバル/ローカルラベル名を保持

---

## 2. 要件実現可能性分析

### 2.1. 技術的要求

**要件1: ラベルベースのコントロールフロー（7 AC）**
- ✅ AC 1-2: ラベル定義 - AST定義済み、パーサー実装済み
- ⚠️ AC 3-7: call/jump - トランスパイラーとランタイムの再設計が必要
  - **Gap**: `ctx.pasta.call()`/`ctx.pasta.jump()`の実装
  - **Gap**: 引数配列（4th parameter）の実装
  - **Gap**: while-let-yieldパターンの生成

**要件2: ランダム選択と前方一致（5 AC）**
- ✅ AC 1-5: すべてランタイムレイヤーに実装済み（`LabelTable`、`RandomSelector`）
- ✅ キャッシュベース消化も実装済み

**要件3: 動的call/jump（3 AC）**
- ✅ AC 1-2: AST定義済み（`JumpTarget::Dynamic`）
- ⚠️ AC 3: エラーハンドリング - ランタイムでの実装が必要

**要件4: 宣言的な会話フロー表現（3 AC）**
- ✅ AC 1-3: Runeブロック内での条件評価、変数設定、動的jumpで実現可能
- ✅ 現在のトランスパイラーはRuneブロックをそのまま出力（AC完全適合）

**要件5: トランスパイラー出力仕様（13 AC）**
- ❌ AC 2-4: モジュール構造、`__start__`関数、ローカルラベル関数 - 全面的な再設計が必要
- ❌ AC 7-9: call/jump文の生成 - while-let-yieldパターンの実装が必要
- ❌ AC 11: 引数アクセス - `ctx.args`の実装が必要
- ❌ AC 12-13: `ctx.args`保存・復元、yield伝播 - ランタイム実装が必要

**要件6: サンプルファイルの修正（5 AC）**
- ⚠️ AC 1-5: `04_control_flow.pasta`の書き直し
  - **Gap**: 現在の内容をすべて削除し、call/jump/ラベルベースの実装例に置き換える
  - **Gap**: ランダム選択、動的call/jump、メニュー選択の実装例を追加

### 2.2. ギャップと制約

**主要ギャップ**:

1. **モジュール生成ロジック**: 
   - **Missing**: グローバルラベル → Runeモジュール変換ロジック
   - **Constraint**: Runeのモジュール構文に従う必要がある（`pub mod 名前 { ... }`）
   - **Impact**: トランスパイラーの出力構造全体を変更

2. **Pastaランタイムオブジェクト**:
   - **Missing**: `ctx.pasta`オブジェクトとそのメソッド（call/jump/word/add_words/commit_words）
   - **Constraint**: Rust側でRune関数として実装し、Rune VMに登録
   - **Impact**: 新規ランタイムモジュール（`runtime/pasta_api.rs`など）の追加

3. **`ctx`オブジェクト構造**:
   - **Missing**: 6フィールド（pasta/actor/scope/save/args）を持つRune Object型
   - **Constraint**: Rune VMでのObject型構築
   - **Impact**: エンジン初期化ロジックの変更

4. **引数保存・復元メカニズム**:
   - **Missing**: `ctx.pasta.call()`での`ctx.args`保存・復元ロジック
   - **Constraint**: Rune VMでのミュータブルなObject操作
   - **Impact**: call/jumpランタイム実装の設計

5. **while-let-yield伝播パターン**:
   - **Missing**: トランスパイラーでのパターン生成ロジック
   - **Constraint**: Runeのジェネレーター構文に従う
   - **Impact**: call/jump文のトランスパイル処理変更

**制約条件**:
- ✅ Rune VM 0.14のジェネレーター機能（yield式サポート済み）
- ✅ 既存のAST定義は変更不要（要件に適合）
- ⚠️ Runeのモジュール構文とジェネレーター構文の学習が必要
- ⚠️ `04_control_flow.pasta`の全面的な書き直しが必要

### 2.3. 複雑度シグナル

**アルゴリズム的複雑性**:
- **Medium**: モジュール生成ロジック（AST走査とネスト構造生成）
- **Medium**: while-let-yieldパターン生成（文字列テンプレート操作）
- **Low**: 引数配列生成（単純な式リスト変換）

**統合的複雑性**:
- **Medium**: Pastaランタイムオブジェクトの実装（Rust ↔ Rune FFI）
- **Medium**: `ctx`オブジェクト構築（Rune Object型操作）
- **Low**: 既存LabelTableとの統合（APIは明確）

**ワークフロー的複雑性**:
- **Low**: トランスパイラー単体のテスト（入力DSL → 出力Runeコードの検証）
- **Medium**: エンドツーエンドテスト（DSL → Rune実行 → IR出力の検証）

---

## 3. 実装アプローチオプション

### Option A: 既存コンポーネントの拡張

**対象ファイル**:
- `crates/pasta/src/transpiler/mod.rs` - トランスパイラー本体を拡張
- `crates/pasta/src/runtime/` - ランタイムモジュールにPasta API追加

**変更内容**:
1. `transpile_label_with_counter()`を変更してモジュール出力に対応
2. `transpile_statement()`でcall/jumpの生成ロジックを変更
3. `runtime/`に`pasta_api.rs`を追加してPastaランタイムメソッド実装

**互換性評価**:
- ⚠️ **Breaking Change**: 既存のトランスパイル結果が全面的に変わる
- ⚠️ Runeコード形式の変更により、既存のテストが全て失敗する可能性
- ✅ AST定義は変更不要なので、パーサーテストは影響を受けない

**複雑度と保守性**:
- ⚠️ 既存の`transpile_label_with_counter()`ロジックを完全に書き換える必要がある
- ⚠️ フラット関数生成ロジックとモジュール生成ロジックは根本的に異なる
- ✅ テストコードは既存のものを修正して再利用可能

**Trade-offs**:
- ✅ ファイル構造の変更が少ない（既存ファイルの修正のみ）
- ❌ 既存ロジックとの整合性が低い（ほぼ全面的な書き換え）
- ❌ トランスパイラーの責務が増大（モジュール生成 + ランタイム呼び出し）

### Option B: 新規コンポーネント作成（推奨）

**新規ファイル**:
- `crates/pasta/src/transpiler/module_codegen.rs` - モジュール生成ロジック
- `crates/pasta/src/transpiler/context_codegen.rs` - ctx関連コード生成
- `crates/pasta/src/runtime/pasta_api.rs` - Pastaランタイムメソッド実装

**既存ファイルの変更**:
- `crates/pasta/src/transpiler/mod.rs` - 新規モジュールを呼び出すように変更
- `crates/pasta/src/engine.rs` - `ctx`オブジェクト構築とPasta API登録
- `crates/pasta/src/stdlib/mod.rs` - Pasta APIのRune関数登録

**責務分離**:
1. **`module_codegen.rs`**: 
   - グローバルラベル → Runeモジュール生成
   - `__start__`関数とローカルラベル関数の生成
   - モジュール構造の整理
2. **`context_codegen.rs`**:
   - call/jump文のwhile-let-yield生成
   - 引数配列の生成
   - `ctx.pasta.*`呼び出しの生成
3. **`pasta_api.rs`**:
   - `call()`: ラベル解決、引数保存・復元、ジェネレーター実行
   - `jump()`: ラベル解決、ジェネレーター実行
   - `word()`: 単語展開（後続仕様で実装）
   - `add_words()`: 単語定義
   - `commit_words()`: 単語コミット

**統合ポイント**:
- `Transpiler::transpile()`から`module_codegen`を呼び出し
- `transpile_statement()`から`context_codegen`を呼び出し
- `PastaEngine::new()`で`pasta_api`をRune VMに登録

**Trade-offs**:
- ✅ クリーンな責務分離（モジュール生成、コンテキスト生成、ランタイム実装）
- ✅ 既存ロジックを保持しながら新機能を追加可能
- ✅ テストの分離（モジュール生成テスト、ランタイムテスト）
- ❌ ファイル数の増加（3ファイル追加）
- ✅ 保守性の向上（各モジュールの責務が明確）

### Option C: ハイブリッドアプローチ

**組み合わせ戦略**:
- **Phase 1**: Option Bで新規モジュールを追加
- **Phase 2**: 既存トランスパイラーを新規モジュールに段階的に移行
- **Phase 3**: 旧ロジックを削除

**フェーズ実装**:
1. **Phase 1（MVP）**:
   - `module_codegen.rs`と`pasta_api.rs`を実装
   - 最小限のトランスパイル（モジュール構造とcall/jump）
   - `04_control_flow.pasta`の修正
2. **Phase 2（完全実装）**:
   - `context_codegen.rs`で引数処理を追加
   - yield伝播の完全実装
   - 全受入基準のカバレッジ
3. **Phase 3（リファクタリング）**:
   - 旧トランスパイラーロジックの削除
   - テストの整理
   - ドキュメントの更新

**リスク緩和**:
- ✅ 段階的なロールアウト（フェーズごとにテスト）
- ✅ フィーチャーフラグ不要（新旧ロジックは並存しない）
- ⚠️ Phase 1後に既存テストが全て失敗する（新形式への移行が必要）

**Trade-offs**:
- ✅ リスク分散（段階的な実装）
- ✅ 各フェーズで動作確認可能
- ❌ 複雑な計画管理（3フェーズの調整）
- ⚠️ Phase 1完了時点でBreaking Change（既存出力形式の廃止）

---

## 4. 実装複雑度とリスク

### 4.1. 工数見積もり

**Effort: M（3-7日）**

**内訳**:
- **Phase 1（モジュール生成）**: 2日
  - `module_codegen.rs`実装（1日）
  - トランスパイラー統合とテスト（0.5日）
  - `04_control_flow.pasta`修正（0.5日）
- **Phase 2（ランタイム実装）**: 3日
  - `pasta_api.rs`実装（1.5日）
  - `ctx`オブジェクト構築とRune統合（1日）
  - エンドツーエンドテスト（0.5日）
- **Phase 3（引数処理と伝播）**: 2日
  - `context_codegen.rs`実装（1日）
  - 引数保存・復元ロジック（0.5日）
  - while-let-yield完全実装とテスト（0.5日）

**合計**: 7日（最大見積もり）

**根拠**:
- ✅ AST定義とパーサーは変更不要（工数削減）
- ✅ 既存のLabelTableとRandomSelectorを再利用可能（工数削減）
- ⚠️ Runeのモジュール構文とジェネレーター構文の学習（+1日）
- ⚠️ Rune FFI（Rust ↔ Rune Object操作）の学習（+1日）

### 4.2. リスク評価

**Risk: Medium**

**主要リスク**:

1. **Rune VM API理解不足**（Medium）
   - **内容**: Rune 0.14のモジュール構文、Object型操作、ジェネレーター動作の理解
   - **緩和策**: Runeドキュメント、サンプルコード、既存のstdlib実装を参照
   - **影響**: 実装時間の延長（+1-2日）

2. **yield伝播メカニズム**（Medium）
   - **内容**: while-let-yieldパターンが正しくイベントを伝播するか不明
   - **緩和策**: 単純なテストケースで動作確認、段階的に複雑化
   - **影響**: 設計変更の可能性（AC 13の要件を満たせない場合）

3. **`ctx.args`のミュータビリティ**（Low）
   - **内容**: Rune VMでのObject型フィールドのミュータブル操作
   - **緩和策**: Runeのドキュメントとサンプルで確認、最悪の場合はクローンで回避
   - **影響**: パフォーマンス低下（極小）

4. **既存テストの全面的な書き直し**（Low）
   - **内容**: トランスパイル結果の形式変更により、既存テストが全て失敗
   - **緩和策**: 新形式に合わせてテストを段階的に修正
   - **影響**: テスト修正工数（+1日）

**パフォーマンスリスク**:
- **Low**: モジュール構造による実行速度への影響は無視できる（関数呼び出しオーバーヘッドのみ）
- **Low**: 引数保存・復元のオーバーヘッド（配列操作のみ、毎回のcall/jumpで発生）

**セキュリティリスク**:
- **None**: ユーザー入力を直接実行するわけではない（静的なDSL → Rune変換）

---

## 5. 設計フェーズへの推奨事項

### 5.1. 推奨アプローチ

**Option B（新規コンポーネント作成）を推奨**

**理由**:
1. ✅ クリーンな責務分離（モジュール生成、コンテキスト生成、ランタイム実装）
2. ✅ 既存ロジックとの整合性を保ちながら新機能を追加
3. ✅ テストの分離と並列開発が可能
4. ✅ 保守性の向上（各モジュールの責務が明確）
5. ⚠️ ファイル数の増加は許容範囲（3ファイル追加のみ）

**代替案**: Option C（ハイブリッド）も有効だが、Phase 1完了時点で既存出力形式が廃止されるため、段階的実装の利点が限定的。

### 5.2. 主要決定事項

**設計フェーズで決定すべき項目**:

1. **`ctx`オブジェクトの詳細設計**:
   - Rune Object型の構築方法
   - 各フィールド（pasta/actor/scope/save/args）の型定義
   - `ctx.actor`の発言者オブジェクト構造

2. **Pastaランタイムメソッドのシグネチャ**:
   - `call(ctx, parent, label, args) -> Generator`
   - `jump(ctx, parent, label, args) -> Generator`
   - `word(ctx, keyword) -> Generator`（後続仕様で詳細化）
   - `add_words(keyword, values)`
   - `commit_words()`

3. **引数保存・復元の実装戦略**:
   - Rune VMでのObject型フィールドのミュータブル操作方法
   - クローン vs. インプレース更新のパフォーマンス比較

4. **エラーハンドリング戦略**:
   - 存在しないラベル呼び出し時の動作（AC 3）
   - トランスパイルエラー vs. ランタイムエラーの切り分け

5. **モジュール命名規則**:
   - 同名グローバルラベルの連番付け（`挨拶_1`, `挨拶_2`）
   - ローカルラベルの連番付け（親モジュール内で独立した連番）
   - **決定済**: グローバルラベルは現在と同じ連番方式、ローカルラベルは親モジュール内で独立した連番（`pub mod 会話_1 { pub fn ジャンプ_1(ctx) {...} pub fn ジャンプ_2(ctx) {...} }`）

### 5.3. 調査項目

**設計前に調査すべき項目**:

1. **Runeモジュール構文**:
   - ✅ ドキュメント: https://rune-rs.github.io/ → "Modules"セクション
   - ✅ サンプルコード: Rune公式リポジトリの`examples/`

2. **Rune Object型操作**:
   - ✅ ドキュメント: https://rune-rs.github.io/ → "Types"セクション
   - ✅ 既存コード: `crates/pasta/src/stdlib/`のRune関数実装

3. **Runeジェネレーター動作**:
   - ✅ ドキュメント: https://rune-rs.github.io/ → "Generators"セクション
   - ⚠️ while-let-yieldパターンの動作確認（実験コード）

4. **Rune FFI（Rust ↔ Rune）**:
   - ✅ ドキュメント: https://rune-rs.github.io/ → "Embedding Rune"セクション
   - ✅ 既存コード: `crates/pasta/src/engine.rs`のVM初期化処理

### 5.4. 次のステップ

1. **設計ドキュメント生成**: `/kiro-spec-design pasta-declarative-control-flow` を実行
2. **調査項目の完了**: 上記4項目を調査し、設計ドキュメントに反映
3. **詳細設計**: モジュール構造、APIシグネチャ、エラーハンドリングを定義
4. **実装タスク分解**: 設計ドキュメントからタスクリストを生成（`/kiro-spec-tasks`）

---

## 6. サマリー

**要約**:
- 現在のトランスパイラーは要件5と4つの主要な乖離点を持つ（モジュール化なし、`__start__`なし、ローカルラベルのフラット化、直接関数呼び出し）
- Pastaランタイムオブジェクト（`ctx.pasta`）が未実装で、call/jump/wordメソッドを新規追加する必要がある
- AST定義は要件に適合しており、変更不要
- `04_control_flow.pasta`に仕様外の命令型構文が含まれており、全面的な書き直しが必要

**推奨アプローチ**: Option B（新規コンポーネント作成）
- `module_codegen.rs`: モジュール生成ロジック
- `context_codegen.rs`: ctx関連コード生成
- `pasta_api.rs`: Pastaランタイムメソッド実装

**工数**: M（3-7日）、リスク: Medium
- 主要リスク: Rune VM API理解、yield伝播メカニズムの検証

**次のアクション**: `/kiro-spec-design pasta-declarative-control-flow` で詳細設計を開始

---

_このギャップ分析は設計フェーズの入力として使用されます。_
