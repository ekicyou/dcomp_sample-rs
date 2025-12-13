# Implementation Report: Phase 3 - Pass 1 モジュール構造生成（部分実装）

| 項目 | 内容 |
|------|------|
| **実装日** | 2025-12-12 |
| **実装者** | GitHub Copilot CLI |
| **フェーズ** | Phase 3 - Pass 1 モジュール構造生成（部分実装） |
| **ステータス** | 🚧 部分完了 |

---

## 実装概要

Phase 3「Pass 1 - モジュール構造生成」の基本機能（Task 4.1-4.3の一部）を実装しました。
`comprehensive_control_flow_simple.pasta`のトランスパイルテストが合格し、2パストランスパイラーの基本動作が確認できました。

---

## 完了した機能

### ✅ Task 4.1-4.3: モジュール構造生成（基本実装）

**ファイル**: `crates/pasta/src/transpiler/mod.rs`

**実装内容**:

1. **グローバルラベル → モジュール生成**
   - グローバルラベル1つにつき`pub mod ラベル名_番号 { ... }`生成
   - LabelRegistryによる連番管理（P0: 全て`_1`）
   
2. **`__start__`関数生成**
   - グローバルラベルの最初のスコープを`pub fn __start__(ctx)`として生成
   - ローカルラベル定義前のステートメントを含む
   
3. **ローカルラベル関数生成**
   - 各ローカルラベルを親モジュール内の`pub fn ラベル名_番号(ctx)`として生成
   - 親モジュールと同じスコープに配置

### ✅ Task 5: ContextCodegen実装（基本実装）

**実装済みステートメント変換**:

1. **Speech文（発言）**
   ```rust
   Statement::Speech { speaker, content, .. } => {
       writeln!(writer, "        ctx.actor = {};", speaker);
       writeln!(writer, "        yield Actor(\"{}\");", speaker);
       // content parts...
   }
   ```

2. **Call文**
   ```rust
   Statement::Call { target, filters, args, .. } => {
       let search_key = transpile_jump_target_to_search_key(target);
       let args_str = transpile_exprs_to_args(args, &context)?;
       let filters_str = transpile_attributes_to_map(filters); // P0: #{} 固定
       writeln!(writer, "        for a in pasta::call(ctx, \"{}\", {}, [{}]) {{ yield a; }}",
           search_key, filters_str, args_str);
   }
   ```

3. **Jump文**
   ```rust
   Statement::Jump { target, filters, .. } => {
       let search_key = transpile_jump_target_to_search_key(target);
       let filters_str = transpile_attributes_to_map(filters); // P0: #{} 固定
       writeln!(writer, "        for a in pasta::jump(ctx, \"{}\", {}, []) {{ yield a; }}",
           search_key, filters_str);
   }
   ```

4. **変数代入**
   ```rust
   Statement::VarAssign { name, scope, value, .. } => {
       let value_expr = transpile_expr(value, &context)?;
       match scope {
           VarScope::Local => writeln!(writer, "        let {} = {};", name, value_expr),
           VarScope::Global => writeln!(writer, "        ctx.var.{} = {};", name, value_expr),
       }
   }
   ```

5. **Runeブロック**
   ```rust
   Statement::RuneBlock { content, .. } => {
       // インデントを保持しながらそのまま出力
       for line in content.lines() {
           writeln!(writer, "        {}", line.trim_start());
       }
   }
   ```

**実装済みSpeechPart変換**:

1. **Text**: `yield Talk("text");`
2. **VarRef**: `yield Talk(\`${ctx.var.変数名}\`);`
3. **FuncCall (単語展開)**: `for a in pasta::word(ctx, "word", [args]) { yield a; }`
4. **SakuraScript**: `yield SakuraScript("script");`

---

## 合格したテスト

### ✅ transpiler_comprehensive_test.rs

**テストケース**: `test_comprehensive_control_flow_simple_transpile`

**検証項目**:
- ✅ グローバルラベル「会話」が`pub mod 会話_1`として生成
- ✅ `pub fn __start__(ctx)`関数が生成
- ✅ グローバルラベル「別会話」が`pub mod 別会話_1`として生成
- ✅ `pub mod pasta`が生成
- ✅ `pub fn label_selector(label, filters)`が生成
- ✅ `match id {`のmatch文が生成
- ✅ `1 => crate::会話_1::__start__`のマッピング
- ✅ `2 => crate::別会話_1::__start__`のマッピング
- ✅ `ctx.actor = さくら`の発言者切り替え
- ✅ `yield Actor("さくら")`の生成
- ✅ `yield Talk("おはよう！")`の生成

**トランスパイル結果**:
```rune
use pasta_stdlib::*;

pub mod 会話_1 {
    pub fn __start__(ctx) {
        ctx.actor = さくら;
        yield Actor("さくら");
        yield Talk("おはよう！");
    }
}

pub mod 別会話_1 {
    pub fn __start__(ctx) {
        ctx.actor = さくら;
        yield Actor("さくら");
        yield Talk("別の会話です。");
    }
}

pub mod pasta {
    pub fn jump(ctx, label, filters, args) {
        let label_fn = label_selector(label, filters);
        for a in label_fn(ctx, args) { yield a; }
    }

    pub fn call(ctx, label, filters, args) {
        let label_fn = label_selector(label, filters);
        for a in label_fn(ctx, args) { yield a; }
    }

    pub fn label_selector(label, filters) {
        let id = pasta_stdlib::select_label_to_id(label, filters);
        match id {
            1 => crate::会話_1::__start__,
            2 => crate::別会話_1::__start__,
            _ => |ctx, args| {
                yield Error(`ラベルID ${id} が見つかりませんでした。`);
            },
        }
    }
}
```

---

## 未実装の機能

### ❌ 複雑なコントロールフロー（comprehensive_control_flow.pasta）

**comprehensive_control_flow.pasta**は以下の高度な機能を使用:
- ローカル単語定義（`＠場所：東京　大阪　京都`）
- ネストされたcall文
- 引数付きローカルラベル（`ーカウント表示　＄値`）
- Runeブロック内のローカル関数定義
- 単語展開の組み合わせ（`＠場所の天気は＠天気だね`）

これらの機能は、**期待される出力ファイル**（comprehensive_control_flow.rn）を見ると、
各ラベル内で使用されるcall/jump/wordごとにヘルパー関数（`__call_X__`, `__jump_X__`, `__word_X__`）を生成する
非常に複雑な仕組みが必要です。

例: `comprehensive_control_flow.rn`の`メイン_1`モジュール
```rune
pub mod メイン_1 {
    pub fn __start__(ctx) {
        // ...
        for a in __word_挨拶__(ctx, []) { yield a; }  // ヘルパー関数呼び出し
        // ...
        for a in __call_自己紹介__(ctx, []) { yield a; }  // ヘルパー関数呼び出し
        // ...
    }
    
    // ヘルパー関数定義
    fn __word_挨拶__(ctx, args) {
        for value in ctx.pasta.word(ctx, "挨拶", args) {
            yield value;
        }
    }
    
    fn __call_自己紹介__(ctx, args) {
        for value in 自己紹介(ctx) {
            yield value;
        }
    }
    // ...
}
```

この実装には以下が必要:
1. 各ラベルのステートメントを事前スキャンしてcall/jump/word使用を検出
2. 使用されているターゲット名のリストを収集
3. 各ターゲット用のヘルパー関数を生成
4. `__start__`と各ローカルラベル関数内でヘルパー呼び出しに変換

**設計との齟齬**:
tasks.mdやdesign.mdでは、call/jump文を直接`pasta::call(ctx, "label", #{}, [args])`に変換すると
記載されていますが、期待される出力ファイルでは間接呼び出しパターンになっています。

この差分の解決には、設計レビューと実装戦略の再検討が必要です。

---

## 次のステップ

### Option A: シンプルな設計に戻す

設計ドキュメント通り、call/jump文を直接`pasta::call()`/`pasta::jump()`に変換する実装に進む:
- Phase 4: Pass 2完了（既に実装済み）
- Phase 5: PastaApi実装（`pasta_stdlib::select_label_to_id`）
- Phase 6: PastaEngine統合
- Phase 7-8: テストと検証

この場合、`comprehensive_control_flow.rn`を設計に合わせて修正する必要があります。

### Option B: 複雑な設計を実装

期待される出力ファイル（comprehensive_control_flow.rn）に合わせて、ヘルパー関数生成パターンを実装:
- ステートメントスキャナーの実装
- ヘルパー関数生成ロジックの追加
- 各ラベルごとのコンテキスト管理

この場合、実装時間が大幅に増加します（推定: 追加で2-3時間）。

---

## 推奨事項

**Option A（シンプルな設計）を推奨**:

理由:
1. tasks.mdとdesign.mdに明確に記載されている設計
2. P0スコープ（完全一致検索のみ）に適した実装
3. 実装時間の短縮
4. 後でP1実装時に最適化可能

実装手順:
1. ✅ Phase 1-2完了
2. ✅ Phase 3基本完了
3. Phase 4-5: Runtimeオブジェクトの実装（`LabelTable`, `WordDictionary`, `PastaApi`）
4. Phase 6: PastaEngine統合
5. Phase 7: サンプルファイル修正
6. Phase 8: 最終検証

---

## 関連要件

| Requirement | Acceptance Criteria | 実装状況 |
|-------------|---------------------|----------|
| 5.1 | Pass 1とPass 2の分離 | ✅ 完了 |
| 5.2 | LabelRegistry実装 | ✅ 完了 |
| 5.3 | __start__関数生成 | ✅ 完了 |
| 5.4 | ローカルラベル関数生成 | ✅ 基本完了 |
| 5.7-5.9 | call/jump文の生成 | ✅ 基本完了 |
| 5.10 | 発言者切り替え生成 | ✅ 完了 |
| 5.11 | ctx引数の統一 | ✅ 完了 |

**注**: 複雑なケース（引数付きローカルラベル、ネスト、単語展開の組み合わせ）は未実装。

---

**最終更新**: 2025-12-12  
**次のフェーズ**: Phase 4-5 (Runtime実装) または設計レビュー
