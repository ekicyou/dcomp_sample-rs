# 実装ギャップ分析: pasta-word-definition-dsl

## 分析サマリー

- **スコープ**: Pasta DSLに単語定義機能を追加（`＠単語名＝「単語1」「単語2」...`構文）
- **主要な課題**:
  - 既存のStatementパース/トランスパイルフローへの統合
  - グローバル/ローカルスコープの単語辞書管理（HashMap構造）
  - フォールバック検索機構（Runeスコープ → 単語辞書 → 前方一致ラベル）の実装
  - 前方一致検索とランダム選択ロジックの追加
- **推奨アプローチ**: Option A（既存コンポーネント拡張） - AST/Parser/Transpilerへの段階的な機能追加

---

## 1. 現状調査

### 既存の関連資産

#### Parser Layer (`crates/pasta/src/parser/`)
- **`ast.rs`**: AST型定義
  - `Statement` enum: 現在5種類のvariant（Speech, Call, Jump, VarAssign, RuneBlock）
  - `LabelDef`: ラベル定義（Global/Local）、attributes保持
  - `SpeechPart` enum: 会話内容の構成要素（Text, VarRef, FuncCall, SakuraScript）
  
- **`mod.rs`**: パーサー実装（pest PEGパーサー使用）
  - `parse_statement()`: Statement variantごとにパース関数を振り分け
  - `parse_speech_content()`: 会話内容のパース（VarRef, FuncCallをサポート）
  
- **`pasta.pest`**: PEG文法定義
  - 現在の構文: speech_line, call_stmt, jump_stmt, var_assign, rune_block
  - `at_marker`, `equals`, `colon`などの既存トークン定義あり

#### Transpiler Layer (`crates/pasta/src/transpiler/mod.rs`)
- **Statement処理**: `transpile_statement()`でmatchによる振り分け
  - 各Statement variantに対応するRune IRコード生成
  - `transpile_speech_part()`: SpeechPart→Rune IR変換
  
#### Runtime Layer (`crates/pasta/src/runtime/`)
- **変数管理** (`variables.rs`): `HashMap<String, VariableValue>`でグローバル/ローカル変数を管理
- **ラベル管理** (`labels.rs`): `HashMap<String, Vec<LabelInfo>>`で前方一致検索をサポート

### 既存のパターンと制約

#### 命名規則
- Files: `snake_case.rs`
- Types: `PascalCase`
- Functions: `snake_case`

#### アーキテクチャパターン
- **3層構造**: Parser (AST生成) → Transpiler (Rune IR生成) → Runtime (実行)
- **Error Handling**: `Result<T, PastaError>`パターン、panicは禁止
- **データ構造**: `HashMap`での辞書管理パターンが確立

#### 統合ポイント
- `Statement` enumへの新variant追加が標準パターン
- `SpeechPart`への新要素追加で会話内参照をサポート可能
- Transpilerの`match stmt`パターンに新ケースを追加

---

## 2. 要件実現性分析

### 技術要件とギャップ

#### 2.1 AST拡張
**要件**: Statement::WordDef variantの追加

**現状**:
- `Statement` enumは5つのvariantを持つ
- 各variantは`span: Span`フィールドを含む

**ギャップ**:
- ✅ 新variant追加のパターンは確立済み（RuneBlockが最近の追加例）
- **Missing**: WordDef構造体定義
  ```rust
  Statement::WordDef {
      name: String,
      words: Vec<String>,
      scope: WordScope,  // Global or Local
      span: Span,
  }
  ```

#### 2.2 Parser拡張
**要件**: `＠単語名＝「単語1」「単語2」`構文のパース

**現状**:
- pest PEGパーサーで文法定義
- `at_marker`, `equals`, `ident`, `string_literal`などの基礎トークンは存在
- 全角スペース・タブを区切り文字として認識するロジック有り

**ギャップ**:
- **Missing**: pasta.pestに`word_def_stmt`ルール追加
  ```pest
  word_def_stmt = {
      at_marker ~ word_name ~ equals ~ word_list ~ NEWLINE
  }
  word_list = { quoted_word ~ (WHITESPACE+ ~ quoted_word)* }
  quoted_word = { quote_open ~ word_content ~ quote_close }
  quote_open = { "「" }
  quote_close = { "」" }
  ```
- **Missing**: `parse_word_def_stmt()`関数（mod.rsに追加）
- **Missing**: 二重引用符エスケープ処理（`「「` → `「`）

#### 2.3 配置制約の検証
**要件**: 
- グローバル単語定義: ファイル内どこでも可（インデントなし）
- ローカル単語定義: グローバルラベル直後の宣言部のみ（インデントあり）

**現状**:
- `global_label`ルール: attribute_line, rune_block, local_label, statementを許可
- `local_label`ルール: attribute_line, rune_block, statementを許可

**決定事項**（議題2で確定）:
- **宣言部/実行部の境界判定**: 方式C（トリガー検出）を採用
- **実行部トリガー**: 以下のstatement出現で実行部開始と判断
  - `speech_line` (speaker: content)
  - `local_label` (ーlabel_name)
  - `jump_stmt` (？target)
  - `call_stmt` (＞target)
- **トリガーにならない要素**:
  - `attribute_line` - 宣言部
  - `rune_block` - 宣言部
  - `var_assign` - 宣言部・実行部の両方で可（トリガーにならない）
  - 空行・コメント - トリガーにならない
- **実装方針**:
  - パーサーで`ParsePhase`状態管理（Declaration/Execution）
  - 実行部開始後にword_def出現 → 構文エラー
  - エラーメッセージ: "Local word definition must be in declaration block (before speech lines, local labels, jumps, or calls)"
- **文法レベル**: pasta.pestの`file`ルールに`word_def_stmt`を追加（グローバルレベル配置）

#### 2.4 Transpiler拡張
**要件**: 単語定義をRune静的変数に変換

**現状**:
- `transpile_statement()`は各Statement variantに対応
- グローバル変数は`set_global()`関数呼び出しで設定

**決定事項**（議題4で確定）:
- **前処理実装場所**: Option A（Transpiler内実装）を採用
- **実装方針**:
  - `transpiler/mod.rs`に以下の関数を追加:
    - `extract_word_defs(ast: &PastaFile) -> Vec<WordDef>` - AST全体からWordDefを抽出
    - `merge_word_defs(defs: Vec<WordDef>) -> WordDefRegistry` - 同名定義をマージ
  - `transpile_file()`冒頭で前処理実行
  - マージ後の辞書データからRune静的変数コード生成
- **生成コード例**:
  ```rune
  static GLOBAL_WORD_DICT = #{
      "場所": ["東京", "大阪", "名古屋", "カナダ"],  // マージ済み
      // ...
  };
  static LOCAL_WORD_DICT_会話 = #{
      "天気": ["晴れ", "雨", "曇り", "晴れか、　あめ"],
      // ...
  };
  ```
- **Statement::WordDef処理**: `transpile_statement()`でスキップ（既に辞書初期化済み）
- **将来的なリファクタリング**: 複雑化した際にpreprocessor.rsへ分離可能（Option C移行）

#### 2.5 Runtime拡張
**要件**: フォールバック検索（Runeスコープ → 単語辞書 → 前方一致ラベル）

**現状**:
- Rune IR実行はrune crateに委託
- ラベル検索は`labels.rs`で前方一致をサポート

**決定事項**（議題3で確定）:
- **実装アプローチ**: Transpiler生成Rune関数方式
- **Runeスコープ検索は不使用**: Rune crateのAPI調査不要
  - 理由: Runeの変数スコープ解決は自動（Rune言語仕様）
  - `@名前`が既存変数ならRune側で自動解決
  - 変数が未定義なら単語辞書・ラベル検索に進む
- **Transpiler生成コード**:
  ```rune
  // グローバル単語辞書（静的変数）
  static GLOBAL_WORD_DICT = #{
      "場所": ["東京", "大阪", "名古屋"],
      // ...
  };
  
  // ローカル単語辞書（ラベルごと）
  static LOCAL_WORD_DICT_会話 = #{
      "天気": ["晴れ", "雨", "曇り"],
      // ...
  };
  
  // 参照解決ヘルパー関数
  fn resolve_ref(name) {
      // 1. Rune変数は自動解決済み（この関数が呼ばれる = 未定義）
      // 2. ローカル単語辞書検索（前方一致）
      if let Some(words) = prefix_search(LOCAL_WORD_DICT, name) {
          return random_choice(words);
      }
      // 3. グローバル単語辞書検索（前方一致）
      if let Some(words) = prefix_search(GLOBAL_WORD_DICT, name) {
          return random_choice(words);
      }
      // 4. ラベル前方一致検索
      return call_label_prefix(name);
  }
  ```
- **VarRef変換**: `SpeechPart::VarRef("名前")` → Rune IR `resolve_ref("名前")`
- **前方一致検索**: `prefix_search(dict, key)`ヘルパー実装
- **ランダム選択**: `random_choice(words)`ヘルパー実装

#### 2.6 会話内参照
**要件**: `＠単語名`で会話行内から単語参照

**現状**:
- `SpeechPart::VarRef`: 変数参照をサポート
- `SpeechPart::FuncCall`: 関数呼び出しをサポート

**決定事項**（議題1で確定）:
- **採用**: Option A（VarRef拡張）
- **実装方針**: 
  - AST構造変更なし、`SpeechPart::VarRef(String)`をそのまま使用
  - Runtime側でフォールバック検索を実装（Rune変数 → 単語辞書 → ラベル）
  - パーサー変更不要、Transpiler最小変更
- **VarRefの意味拡張**:
  - AST段階: `＠名前`形式の参照全般を表現（変数、単語、ラベルを区別しない）
  - Runtime段階: フォールバック検索で実際の対象を決定
  - 名称は変更せず、ドキュメントで意味を明記
- **利点**:
  - ✅ ユーザー透過性（`＠名前`と書くだけで自動解決）
  - ✅ 要件との整合性（フォールバック検索の自然な実現）
  - ✅ 既存コードへの影響最小
- **Option Bを却下**:
  - 新SpeechPart::WordRef variantは追加しない
  - 理由: パーサー複雑化、ユーザーが変数と単語を意識する必要が生じる

#### 2.7 エラーハンドリング
**要件**: 
- パース時: 空定義検出、全エラー収集後にResult::Err
- 実行時: 未定義参照はエラーログ、空文字列で継続

**現状**:
- `PastaError` enum: 各種エラー型定義
- `Result<T, PastaError>`パターン確立

**ギャップ**:
- ✅ 既存エラーハンドリングパターンで対応可能
- **Missing**: 空定義検出ロジック（パーサーで`word_list`が空の場合をチェック）
- **Missing**: エラーログ出力機構（Runtime側）

---

## 3. 実装アプローチオプション

### Option A: 既存コンポーネント拡張（推奨）

#### 拡張対象ファイル
1. **`crates/pasta/src/parser/ast.rs`**
   - `Statement` enumに`WordDef` variant追加
   - `WordScope` enum定義（Global/Local）

2. **`crates/pasta/src/parser/pasta.pest`**
   - `word_def_stmt`ルール追加
   - `file`ルールに`word_def_stmt`を追加
   - `global_label`, `local_label`ルールに配置制約を反映

3. **`crates/pasta/src/parser/mod.rs`**
   - `parse_word_def_stmt()`関数追加
   - `parse_statement()`に新ケース追加

4. **`crates/pasta/src/transpiler/mod.rs`**
   - `transpile_statement()`に`Statement::WordDef`ケース追加
   - 前処理フェーズでWordDef抽出・マージ
   - 静的変数初期化コード生成

5. **`crates/pasta/src/runtime/`**
   - 新規ファイル`words.rs`（単語辞書管理）
   - フォールバック検索関数（Rune側で実装）

#### 互換性評価
- ✅ 既存のStatement処理フローに自然に統合
- ✅ 後方互換性維持（新構文は既存スクリプトに影響なし）
- ✅ テストカバレッジ拡張で対応可能

#### 複雑度と保守性
- **複雑度**: 中程度
  - AST拡張: 単純（新variant追加）
  - Parser拡張: 中（新ルール＋配置制約検証）
  - Transpiler拡張: 中〜高（マージロジック＋静的変数生成）
  - Runtime拡張: 高（フォールバック検索＋キャッシュ機構）
- **保守性**: 良好
  - 既存パターンに沿った実装
  - ファイルサイズは適度（各ファイル500行前後に収まる見込み）

#### トレードオフ
- ✅ 最小限の新規ファイル、既存フローに統合
- ✅ 既存パターン活用、学習コスト低
- ❌ Transpilerの前処理フェーズが複雑化
- ❌ Runtime検索ロジックが多段階に（Runeスコープ→単語辞書→ラベル）

---

### Option B: 新規コンポーネント作成

#### 新規作成候補
1. **`crates/pasta/src/words/`**
   - `mod.rs`: 単語辞書管理のコアロジック
   - `parser.rs`: 単語定義専用パーサー
   - `resolver.rs`: フォールバック検索ロジック

2. **`crates/pasta/src/preprocessor.rs`**
   - AST前処理（WordDef抽出・マージ）
   - 宣言部/実行部の境界検証

#### 統合ポイント
- Parser: `parse_file()`から`preprocessor::process_word_defs()`を呼び出し
- Transpiler: 前処理済み単語辞書を受け取り、静的変数コード生成
- Runtime: `words::resolver`モジュールをRune側から呼び出し

#### 責務境界
- `words/`: 単語定義の解析・管理・検索に特化
- `preprocessor`: AST変換・検証に特化
- 既存Parser/Transpiler: 従来の責務を維持

#### トレードオフ
- ✅ 明確な責務分離、テスト容易性向上
- ✅ 単語定義ロジックが独立、再利用可能
- ❌ ファイル数増加（ナビゲーション複雑化）
- ❌ インターフェース設計が必要（開発初期の overhead）

---

### Option C: ハイブリッドアプローチ

#### 組み合わせ戦略
- **Phase 1**: Option A（既存拡張）で基本機能を実装
  - Statement::WordDef追加
  - 基本的なパース・トランスパイル
  - 単純なランダム選択

- **Phase 2**: 複雑なロジックを新規モジュールに分離
  - `words/resolver.rs`: フォールバック検索
  - `words/cache.rs`: シャッフルキャッシュ機構
  - Refactoring: Transpilerの前処理を`preprocessor.rs`に抽出

#### 段階的実装
1. **Minimal Viable**: 単語定義の基本構文サポート
2. **Enhanced Search**: フォールバック検索追加
3. **Optimization**: キャッシュ機構追加

#### リスク軽減
- 段階的リリースで早期フィードバック
- Phase 1で既存パターン踏襲、Phase 2で最適化
- 各フェーズでテストカバレッジ拡充

#### トレードオフ
- ✅ バランスの取れたアプローチ
- ✅ 反復的な改善が可能
- ❌ 計画の複雑性増加
- ❌ Phase間の一貫性維持に注意必要

---

## 4. 研究が必要な項目

以下の項目は設計フェーズで詳細調査が必要：

1. **Runeヘルパー関数実装**
   - `prefix_search(dict, key)`の効率的な実装
   - `random_choice(words)`のRNG選択
   - `call_label_prefix(name)`の既存labels.rs統合

2. **シャッフルキャッシュ機構**
   - 既存実装の有無確認（里々の実装を参考にできる可能性）
   - キャッシュのライフサイクル管理

4. **前方一致検索の最適化**
   - HashMapでのprefix searchの効率
   - Trieなどの代替データ構造の検討

5. **エラーメッセージの日本語化**
   - 既存のエラーメッセージパターン確認
   - 多言語対応の方針

---

## 5. 実装複雑度とリスク

### 工数見積もり: **M (中規模: 3-7日)**

#### 内訳
- AST/Parser拡張: 1-2日（新variant追加、pest文法定義）
- Transpiler拡張: 2-3日（マージロジック、静的変数生成）
- Runtime拡張: 2-3日（フォールバック検索、ランダム選択）
- テスト作成: 1-2日

#### 根拠
- 既存パターンが確立されており、新機能追加の precedent あり
- 中程度の複雑度（マージロジック、多段階検索）
- 単一クレート内での完結、外部依存なし

### リスク評価: **Medium (中リスク)**

#### リスク要因
- **Rune統合**: Runeスコープ検索APIの調査が必要
  - 軽減策: Rune crateドキュメント調査、既存のFuncCall実装を参考
  
- **前処理フェーズの複雑化**: AST全体スキャン・マージロジック
  - 軽減策: 段階的実装（Phase 1で単純実装、Phase 2で最適化）
  
- **配置制約の検証**: 宣言部/実行部の境界判定
  - 軽減策: 保守的な実装（明確なエラーメッセージ）

#### 成功要因
- ✅ 既存のStatement/SpeechPartパターンが明確
- ✅ HashMap管理パターンが確立（variables.rs, labels.rs）
- ✅ pest PEGパーサーの拡張が直感的

---

## 6. 設計フェーズへの推奨事項

### 推奨アプローチ
**Option A（既存コンポーネント拡張）** を推奨

#### 理由
1. 既存のStatement処理フローとの自然な統合
2. パーサー変更が最小限（pest文法拡張のみ）
3. 開発初期段階で新規モジュール作成の overhead を回避
4. 後にOption Cへの移行も容易（リファクタリングの余地）

### 重要な設計判断
1. **SpeechPart拡張 vs WordRef追加**
   - VarRefの Runtime解決でフォールバック検索を適用（Option A推奨）
   - 理由: 既存コードへの影響最小、ユーザーには透過的

2. **前処理フェーズの設計**
   - Transpiler内でWordDef抽出・マージ（初期実装）
   - 将来的に`preprocessor.rs`へ分離可能

3. **エラーハンドリング戦略**
   - パース時: 空定義を即座にエラー収集
   - Runtime時: ログ出力＋空文字列フォールバック

### 次フェーズでの研究項目
1. Rune crateのスコープ検索API詳細調査
2. 宣言部/実行部境界判定の実装詳細
3. シャッフルキャッシュ機構の設計
4. 前方一致検索の最適化手法

### 成功メトリクス
- パース成功率: 100%（正常な単語定義構文）
- エラー検出率: 100%（空定義、配置違反）
- Runtime検索正確性: 100%（フォールバック順序の正確性）
- パフォーマンス: 単語検索 < 1ms（前方一致含む）

---

## 次のステップ

ギャップ分析が完了しました。次のフェーズに進むには：

1. **設計フェーズへ移行**: `/kiro-spec-design pasta-word-definition-dsl`
   - 詳細なAPI設計、データ構造設計
   - Rune統合の技術調査
   - テスト戦略の策定

2. **要件の自動承認と設計**: `/kiro-spec-design pasta-word-definition-dsl -y`
   - 要件を自動承認して直接設計フェーズへ

推奨: 研究項目（特にRune API調査）を設計フェーズで実施してから実装に進む。
