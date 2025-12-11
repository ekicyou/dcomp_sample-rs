# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | Pasta DSL 宣言的コントロールフロー 要件定義書 |
| **Version** | 1.0 |
| **Date** | 2025-12-11 |
| **Parent Spec** | areka-P0-script-engine (completed) |
| **Priority** | P0 (既存実装の修正) |

---

## Introduction

本仕様書は、現在の`04_control_flow.pasta`サンプルファイルに含まれる誤った実装（命令型プログラミング構文 `if/elif/else/while`）を修正し、元仕様「areka-P0-script-engine」に基づいた正しい宣言的コントロールフロー構文を再定義する。

### 問題の本質

現在の実装には以下の問題がある：

1. **仕様外構文**: `＠if`, `＠elif`, `＠else`, `＠while` は元仕様に存在しない
2. **設計意図の逸脱**: Pasta DSLは宣言的言語であり、命令型の制御構文は本質的に含まない
3. **実装の混乱**: Runeブロック内で実装すべきロジックをDSL構文レベルで提供している

### 元仕様の正しいコントロールフロー

元仕様（`areka-P0-script-engine`およびGRAMMAR.md）で定義された宣言的コントロールフローは以下で構成される：

1. **call** (`＞`): サブルーチン呼び出し（戻り先を記憶）
2. **jump** (`？`): 無条件ジャンプ（戻らない）
3. **ラベル定義**: グローバル(`＊`)とローカル(`ー`)
4. **ランダム選択**: 同名ラベルの複数定義と前方一致選択
5. **キャッシュベース消化**: 選択肢を順に消化する仕組み
6. **発言とコンテキスト**: 会話フローの本質

### スコープ

**含まれるもの:**
- 元仕様に基づいた正しいコントロールフロー構文の要件定義
- `04_control_flow.pasta`の修正実装例
- call/jump/ラベル定義/ランダム選択の正しい使用例
- **トランスパイラー出力仕様**: Pasta DSLからRuneコードへの変換規則

**含まれないもの:**
- 命令型制御構文（`if/elif/else/while`）
- Runeブロック内の条件分岐・ループ（別途Rune機能として実装可能）
- 新しいDSL構文の追加

---

## Requirements

### Requirement 1: ラベルベースのコントロールフロー

**Objective:** スクリプト作成者として、callとjumpを使用した宣言的なコントロールフローを記述できるようにし、会話の流れを自然に表現できるようにする。

#### Acceptance Criteria

1. When スクリプト作成者がグローバルラベルを定義する, the Pasta Engine shall `＊`記号で始まる行をグローバルラベル定義として認識する
2. When スクリプト作成者がローカルラベルを定義する, the Pasta Engine shall 親グローバルラベル内の`ー`記号で始まる行をローカルラベル定義として認識する
3. When スクリプト作成者が`＞ローカル名`構文を使用する, the Pasta Engine shall 現在のグローバルラベル内のローカルラベルを呼び出し、実行後に呼び出し元に戻る
4. When スクリプト作成者が`＞＊グローバル名`構文を使用する, the Pasta Engine shall ファイル全体スコープでグローバルラベルを呼び出し、実行後に呼び出し元に戻る
5. When スクリプト作成者が`？ローカル名`構文を使用する, the Pasta Engine shall 現在のグローバルラベル内のローカルラベルにジャンプし、呼び出し元には戻らない
6. When スクリプト作成者が`？＊グローバル名`構文を使用する, the Pasta Engine shall ファイル全体スコープでグローバルラベルにジャンプし、呼び出し元には戻らない
7. When スクリプト作成者が`＞＊グローバルーローカル`構文を使用する, the Pasta Engine shall グローバルラベル配下のローカルラベルをロングジャンプ形式で呼び出す

### Requirement 2: ランダム選択と前方一致

**Objective:** スクリプト作成者として、同名ラベルの複数定義と前方一致ランダム選択を活用し、会話バリエーションを効率的に記述できるようにする。

#### Acceptance Criteria

1. When スクリプト作成者が同一名のラベルを複数定義する, the Pasta Engine shall すべての定義を内部的に連番付きで管理する（例: `挨拶_1`, `挨拶_2`, `挨拶_3`）
2. When スクリプト作成者が前方一致するラベル名でcall/jumpを実行する, the Pasta Engine shall 前方一致するすべてのラベルからランダムに1つを選択して実行する
3. When スクリプト作成者がロングジャンプ構文（`＊グローバルーローカル`）を使用する, the Pasta Engine shall グローバルとローカルの組み合わせをフラット化し、すべての組み合わせから1つをランダム選択する
4. When Pasta Engineが同じ選択キーワードで2回目以降の呼び出しを受ける, the Pasta Engine shall キャッシュから未消化の選択肢を順に返す
5. When キャッシュ内の選択肢がすべて消化される, the Pasta Engine shall キャッシュをクリアし、次回は再構築する

### Requirement 3: 動的call/jump

**Objective:** スクリプト作成者として、変数の値をラベル名として動的に解決し、実行時に柔軟なフロー制御を実現できるようにする。

#### Acceptance Criteria

1. When スクリプト作成者が`＞＠変数名`構文を使用する, the Pasta Engine shall 変数の値をラベル名として解決し、該当ラベルを呼び出す
2. When スクリプト作成者が`？＠変数名`構文を使用する, the Pasta Engine shall 変数の値をラベル名として解決し、該当ラベルにジャンプする
3. If 変数の値が存在しないラベル名を示す, the Pasta Engine shall エラーメッセージを生成し、スクリプト実行を中断する

### Requirement 4: 宣言的な会話フロー表現

**Objective:** スクリプト作成者として、命令型構文なしで条件分岐やメニュー選択を表現し、宣言的なコントロールフローのみで会話を記述できるようにする。

#### Acceptance Criteria

1. When スクリプト作成者が条件に応じた会話分岐を実装する, the Pasta Engine shall Runeブロック内で条件評価を行い、結果に応じて変数に適切なラベル名を設定し、動的jumpで分岐を実現できる
2. When スクリプト作成者がメニュー選択機能を実装する, the Pasta Engine shall 各選択肢に対応するローカルラベルを定義し、選択結果に応じてcallまたはjumpで処理を分岐できる
3. When スクリプト作成者がループ的な処理を実装する, the Pasta Engine shall jumpで自身または別のラベルに戻ることで繰り返し実行を実現できる

### Requirement 5: トランスパイラー出力仕様

**Objective:** 開発者として、Pasta DSLからRuneコードへのトランスパイル規則を明確に定義し、宣言的コントロールフロー構文が正しくRuneのyield文とwhile-let-yieldパターンに変換されることを保証する。

#### トランスパイラー出力の基本原則

1. **モジュール構造**: グローバルラベル1つにつきRuneモジュール1つを生成（`pub mod ラベル名_番号 { ... }`）
2. **`__start__`関数**: グローバルラベルの最初のスコープ（ローカルラベル定義前の処理）は必ず`pub fn __start__(ctx)`関数として生成
3. **ローカルラベル関数**: 各ローカルラベルは親モジュール内の個別関数（`pub fn ラベル名_番号(ctx)`）として生成
4. **環境引数**: すべての関数は`ctx`（コンテキストオブジェクト、Rune Object型）を第一引数として受け取るジェネレーター関数
5. **`ctx`オブジェクト構造**:
   - `ctx.pasta`: Pastaランタイムが提供するローカル処理関数（`call`, `jump`, `word`など）
   - `ctx.actor`: 現在の発言者オブジェクト（グローバル変数として定義された発言者）
   - `ctx.actor.name`: 発言者名（例: `"さくら"`）
   - `ctx.scope`: スコープ情報オブジェクト
   - `ctx.scope.global`: 現在のグローバルスコープラベル名
   - `ctx.scope.local`: 現在のローカルスコープラベル名
   - `ctx.save`: `＄変数＝値`で設定される永続化変数のオブジェクト
   - `ctx.args`: 関数呼び出し時に渡される引数配列（`＞ラベル（引数1　引数2）`で指定）
6. **単語定義スコープ**: 
   - グローバル単語定義（`＠グローバル単語：...`）→ モジュール外で`add_words()`を直接呼び出し
   - ローカル単語定義（`＠単語：...`）→ 関数内で`ctx.pasta.add_words()` + `ctx.pasta.commit_words()`

**注**: `ctx`オブジェクトのフィールド構造は上記の通り。詳細な型定義、`ctx.pasta`の完全なメソッドシグネチャ、内部実装メカニズムは設計フェーズで定義する。

#### 現在の実装との差異

**⚠️ 重要**: 現在のトランスパイラー実装（`crates/pasta/src/transpiler/mod.rs`）は以下の点で要件と乖離している：

1. グローバルラベルがモジュール化されず、フラットな関数として生成されている
2. `__start__`関数が生成されていない
3. ローカルラベルが親モジュール内に配置されず、`親名__子名`形式でフラット化されている
4. `ctx.pasta.call()`/`ctx.pasta.jump()`形式ではなく、直接関数呼び出しになっている

これらは設計フェーズで全面的に修正する必要がある。

#### Acceptance Criteria

1. When トランスパイラーがグローバル単語定義（`＠グローバル単語：値1 値2`）を処理する, the Pasta Transpiler shall `add_words("単語名", ["値1", "値2"])`をモジュール外部で生成する
2. When トランスパイラーがグローバルラベルを処理する, the Pasta Transpiler shall `pub mod ラベル名_番号 { ... }`形式のRuneモジュールを生成する
3. When トランスパイラーがグローバルラベルの最初のスコープ（ローカルラベルなし）を処理する, the Pasta Transpiler shall `pub fn __start__(ctx) { ... }`関数を生成する
4. When トランスパイラーがローカルラベルを処理する, the Pasta Transpiler shall `pub fn ラベル名_番号(ctx) { ... }`関数を生成する
5. When トランスパイラーがローカル単語定義（`＠単語：値1 値2`）を処理する, the Pasta Transpiler shall `ctx.pasta.add_words("単語", ["値1", "値2"]); ctx.pasta.commit_words();`を生成する
6. When トランスパイラーが変数代入（`＄変数＝値`）を処理する, the Pasta Transpiler shall `ctx.save.変数 = 値;`を生成する
7. When トランスパイラーが引数なしcall文（`＞ラベル名`）を処理する, the Pasta Transpiler shall `while let Some(a) = ctx.pasta.call(ctx, "親ラベル", "ラベル名", []).next() { yield a; }`を生成する
8. When トランスパイラーが引数付きcall文（`＞ラベル名（引数1　引数2）`）を処理する, the Pasta Transpiler shall `while let Some(a) = ctx.pasta.call(ctx, "親ラベル", "ラベル名", [引数1, 引数2]).next() { yield a; }`を生成する
9. When トランスパイラーがjump文（`？ラベル名`）を処理する, the Pasta Transpiler shall call文と同様に引数配列を第4引数として渡す（`ctx.pasta.jump(ctx, "親ラベル", "ラベル名", [...])`）
10. When トランスパイラーが発言者切り替え（`さくら：`）を処理する, the Pasta Transpiler shall `ctx.actor = さくら; yield Actor("さくら");`を生成する（`さくら`はグローバル変数として定義された発言者オブジェクト）
11. When トランスパイラーがローカルラベル関数を生成する, the Pasta Transpiler shall すべての関数を`pub fn 名前(ctx)`シグネチャで統一し、引数は`ctx.args`経由でアクセスする
10. When トランスパイラーが発言内容を処理する, the Pasta Transpiler shall `yield Talk("発言内容");`を生成する
11. When トランスパイラーが単語展開（`＠単語名`）を発言内で処理する, the Pasta Transpiler shall `while let Some(a) = ctx.pasta.word(ctx, "単語名").next() { yield a; };`を生成する
12. When トランスパイラーがRuneブロックを検出する, the Pasta Transpiler shall Runeコードをそのままモジュール内に展開する

#### 出力例（リファレンス実装）

**入力 Pasta DSL:**
```pasta
＠グローバル単語：はろー　わーるど

＊会話
　＠場所：東京　大阪
　＄変数＝１０
　＞コール１
　＞コール２
　？ジャンプ

　ージャンプ
　さくら：良い天気だね。
　うにゅう：せやね。

　ージャンプ
　さくら：＠場所　では雨が降ってる。
　うにゅう：ぐんにょり。

　ーコール１
　さくら：はろー。

　ーコール２
　うにゅう：わーるど。
```

**期待される出力 Rune:**
```rune
use pasta::add_words;

add_words("グローバル単語", ["はろー", "わーるど"]);

pub mod 会話_1 {
    pub fn __start__(ctx) {
        ctx.pasta.add_words("場所", ["東京", "大阪"]); 
        ctx.pasta.commit_words();
        ctx.save.変数 = 10;
        while let Some(a) = ctx.pasta.call(ctx, "会話_1", "コール１", []).next() { yield a; }
        while let Some(a) = ctx.pasta.call(ctx, "会話_1", "コール２", []).next() { yield a; }
        while let Some(a) = ctx.pasta.jump(ctx, "会話_1", "ジャンプ", []).next() { yield a; }
    }

    pub fn ジャンプ_1(ctx) {
        // さくら：良い天気だね。
        ctx.actor = さくら;
        yield Actor("さくら");
        yield Talk("良い天気だね。");
        // うにゅう：せやね。
        ctx.actor = うにゅう;
        yield Actor("うにゅう");
        yield Talk("せやね。");
    }

    pub fn ジャンプ_2(ctx) {
        // さくら：＠場所　では雨が降ってる。
        ctx.actor = さくら;
        yield Actor("さくら");
        while let Some(a) = ctx.pasta.word(ctx, "場所").next() { yield a; };
        yield Talk("では雨が降ってる。");
        // うにゅう：ぐんにょり。
        ctx.actor = うにゅう;
        yield Actor("うにゅう");
        yield Talk("ぐんにょり。");
    }

    pub fn コール１(ctx) {  
        // さくら：はろー。
        ctx.actor = さくら;
        yield Actor("さくら");
        yield Talk("はろー。");
    }

    pub fn コール２(ctx) {
        // うにゅう：わーるど。
        ctx.actor = うにゅう;
        yield Actor("うにゅう");
        yield Talk("わーるど。");
    }
}
```

### Requirement 6: サンプルファイルの修正

**Objective:** 開発者として、`04_control_flow.pasta`を元仕様に準拠した正しい実装例として書き直し、宣言的コントロールフロー構文の使用方法を示す。

#### Acceptance Criteria

1. When 開発者が`04_control_flow.pasta`を修正する, the Pasta Engine shall 現在含まれているすべての`＠if`, `＠elif`, `＠else`, `＠while`構文を削除する
2. When 開発者が`04_control_flow.pasta`を修正する, the Pasta Engine shall call/jump/ラベル定義を使用した宣言的な実装例を提供する
3. When 開発者が`04_control_flow.pasta`を修正する, the Pasta Engine shall ランダム選択とキャッシュベース消化の実装例を含める
4. When 開発者が`04_control_flow.pasta`を修正する, the Pasta Engine shall 動的call/jumpを使用したメニュー選択の実装例を含める
5. When 開発者が`04_control_flow.pasta`を修正する, the Pasta Engine shall ファイル冒頭のコメントを修正し、「宣言的コントロールフロー」を正しく説明する

---

## Related Documentation

- `.kiro/specs/completed/areka-P0-script-engine/requirements.md` - 元仕様の要件定義
- `crates/pasta/GRAMMAR.md` - Pasta DSL文法リファレンス
- `crates/pasta/examples/scripts/` - サンプルスクリプト集

---

## Acceptance Validation

以下の基準をすべて満たす場合、本仕様の実装は成功とみなされる：

1. `04_control_flow.pasta`に命令型構文（`＠if`, `＠elif`, `＠else`, `＠while`）が含まれない
2. call/jump/ラベル定義を使用した宣言的なコントロールフロー例が実装されている
3. ランダム選択と前方一致の動作例が含まれている
4. 動的call/jumpを使用したメニュー選択例が含まれている
5. トランスパイラーが要件5で定義された出力規則に従ってRuneコードを生成する
6. 生成されたRuneコードがwhile-let-yieldパターンを使用してyieldイベントを正しく伝播する
7. すべてのサンプルコードがPasta Engineで正常に実行できる
