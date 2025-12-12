# Implementation Tasks

| 項目 | 内容 |
|------|------|
| **Document Title** | Pasta DSL 宣言的コントロールフロー 実装タスク |
| **Version** | 1.0 |
| **Date** | 2025-12-12 |
| **Status** | tasks-generated |

---

## 🎯 必達条件（Critical Success Criteria）

**本タスクリスト完了時に以下が必ず達成されること：**

### 1. `comprehensive_control_flow.pasta` → `comprehensive_control_flow.rn` トランスパイル成功

- ✅ `crates/pasta/tests/fixtures/comprehensive_control_flow.pasta` を正しくトランスパイル
- ✅ 期待される `comprehensive_control_flow.rn` と一致する出力を生成
- ✅ 包括的なトランスパイルテストが合格（`assert_eq!` で厳密一致）

### 2. P0範囲の完全実装

- ✅ モジュール構造生成（グローバルラベル → `pub mod ラベル名_番号`）
- ✅ `__start__`関数生成
- ✅ ローカルラベル関数の親モジュール内配置
- ✅ call/jump文の for-loop + yield パターン生成
- ✅ `mod pasta {}` と `label_selector()` 生成
- ✅ 完全一致ラベル解決（`pasta_stdlib::select_label_to_id`）
- ✅ 単語展開、変数代入、発言者切り替えの正しい生成
- ✅ Runeブロックの適切な配置

### 3. P0 Validation Criteria（9項目）すべて合格

（詳細は本ドキュメント末尾の Validation Criteria セクション参照）

---

## Task List

### Phase 1: テストファースト基盤構築

- [x] 1. 包括的なリファレンス実装とテストスイート作成
- [x] 1.1 包括的なPasta DSLサンプルファイル作成
  - グローバルラベル定義とローカルラベル定義を含む
  - call文（引数なし、引数あり）とjump文を含む
  - ロングジャンプ構文（`＞＊グローバルーローカル`）を含む
  - 動的call/jump（`＞＠変数名`）を含む
  - 変数代入（`＄変数＝値`）を含む
  - Runeブロック内での条件分岐例を含む
  - 発言者切り替え（`さくら：`）を含む
  - 単語定義（グローバル/ローカル）と単語展開（`＠単語名`）を含む
  - **P0範囲**: 同名ラベルなし、完全一致のみ
  - ファイル配置: `crates/pasta/tests/fixtures/comprehensive_control_flow_simple.pasta`
  - _Requirements: 7.1, 7.2_

- [x] 1.2 期待されるRune出力ファイル作成
  - 要件5の出力例（リファレンス実装）に厳密に準拠
  - モジュール構造: グローバルラベル → `pub mod ラベル名_番号`
  - `__start__`関数: グローバルラベルの最初のスコープを関数化
  - ローカルラベル関数: 親モジュール内に配置（`pub fn ラベル名_番号(ctx)`）
  - call/jump文: for-loop + yieldパターン（`for a in pasta::call(...) { yield a; }`）
  - `mod pasta {}`: Pass 2で生成される予約関数群（`jump`, `call`, `label_selector`）
  - 引数配列の正確な生成
  - ファイル配置: `crates/pasta/tests/fixtures/comprehensive_control_flow_simple.expected.rn`
  - _Requirements: 7.3, 7.4, 5.1, 5.2, 5.3, 5.4, 5.7, 5.8, 5.9_

- [x] 1.3 包括的なトランスパイルテスト作成
  - トランスパイル結果と期待出力を厳密比較（`assert_eq!`）
  - モジュール構造の正確性を検証
  - `__start__`関数生成を検証
  - ローカルラベル関数の親モジュール内配置を検証
  - call/jump文のfor-loop + yieldパターン生成を検証
  - 引数配列の正確な生成を検証
  - `pasta::call()`/`pasta::jump()`呼び出し形式を検証
  - ファイル配置: `crates/pasta/tests/transpiler_comprehensive_test.rs`
  - _Requirements: 7.5, 7.6, 7.7_

### Phase 2: トランスパイラー基盤リファクタリング

- [ ] 2. LabelRegistry実装（ラベル収集とID割り当て）
- [ ] 2.1 LabelInfo構造体とLabelRegistry実装
  - LabelInfo構造体: id, name, attributes, fn_path フィールド
  - LabelRegistry: ラベル収集、ID自動採番（1から開始）
  - 同名ラベルに連番付与（`会話_1`, `会話_2`）
  - 各ラベルのRune関数パス生成（`crate::会話_1::__start__`）
  - グローバル/ローカルラベル登録メソッド（`register_global`, `register_local`）
  - 全ラベル情報取得メソッド（`all_labels`）
  - **P0範囲**: 同名ラベルなし、連番は全て`_1`
  - _Requirements: 5.2, 5.3, 5.4_

- [ ] 2.2 (P) LabelRegistry単体テスト作成
  - グローバルラベル登録とID採番を検証
  - ローカルラベル登録とfn_path生成を検証
  - 連番管理ロジックを検証（P0: 常に`_1`）
  - _Requirements: 5.2, 5.3, 5.4_

- [ ] 3. 2パストランスパイラー統合（Writeトレイト対応）
- [ ] 3.1 Transpilerインターフェースのリファクタリング
  - **主要インターフェース**:
    - `pub fn transpile_pass1<W: Write>(file: &PastaFile, registry: &mut LabelRegistry, writer: &mut W)`
    - `pub fn transpile_pass2<W: Write>(registry: &LabelRegistry, writer: &mut W)`
  - **テスト用便利メソッド**: `#[doc(hidden)] pub fn transpile_to_string(file: &PastaFile) -> Result<String>`
    - **注意**: 本番コードでは使用しない（複数ファイル非対応）
    - 単体テスト専用の便利関数
  - Pass 1は複数回呼び出し可能（各PastaFileごとにregistryに蓄積）
  - Pass 2は全ファイル処理後に1回のみ呼び出し
  - **重要**: Pass 1とPass 2は文字列生成のみ、Runeコンパイルは最後に1回
  - キャッシュ機能の基盤（オプショナル）: `persistence_root/cache/pass1/`
  - _Requirements: 5.1, 5.2_

- [ ] 3.2 (P) 2パストランスパイラー統合テスト
  - Writeトレイトへの出力を検証（String, File, Vecなど）
  - 複数PastaFileの処理を検証（Pass 1を複数回呼び出し）
  - Pass 1出力のみの検証テスト追加
  - Pass 2出力（mod pasta）の検証テスト追加
  - transpile_to_string()の単体テスト（テスト専用）
  - 既存テストケースが引き続きパスすることを確認
  - _Requirements: 5.1, 5.2_

### Phase 3: Pass 1 - モジュール構造生成

- [ ] 4. ModuleCodegen実装（グローバルラベル→モジュール変換）
- [ ] 4.1 グローバルラベルのモジュール生成
  - グローバルラベル1つにつきRuneモジュール1つ生成（`pub mod ラベル名_番号 { ... }`）
  - LabelRegistryから取得した連番とパスを使用
  - グローバル単語定義をモジュール外部で`add_words()`呼び出しとして生成
  - _Requirements: 5.1, 5.2_

- [ ] 4.2 `__start__`関数生成
  - グローバルラベルの最初のスコープ（ローカルラベル定義前）を`pub fn __start__(ctx)`関数として生成
  - すべての関数を`ctx`（コンテキストオブジェクト）を第一引数とするジェネレーター関数として生成
  - _Requirements: 5.3_

- [ ] 4.3 ローカルラベル関数生成
  - 各ローカルラベルを親モジュール内の個別関数（`pub fn ラベル名_番号(ctx)`）として生成
  - すべての関数を`pub fn 名前(ctx)`シグネチャで統一
  - _Requirements: 5.4, 5.11_

- [ ] 4.4 (P) ModuleCodegen単体テスト
  - グローバルラベル→モジュール変換を検証
  - `__start__`関数生成を検証
  - ローカルラベル関数の親モジュール内配置を検証
  - _Requirements: 5.2, 5.3, 5.4_

- [ ] 5. ContextCodegen実装（call/jump/word文変換）
- [ ] 5.1 call文のfor-loop + yieldパターン生成
  - 引数なしcall: `for a in pasta::call(ctx, "検索キー", #{}, []) { yield a; }`
  - 引数付きcall: `for a in pasta::call(ctx, "検索キー", #{}, [引数1, 引数2]) { yield a; }`
  - グローバル検索キー: `"会話"`（グローバルラベル名）
  - ローカル検索キー: `"会話_1::選択肢"`（親::子形式）
  - _Requirements: 5.7, 5.8_

- [ ] 5.2 jump文のfor-loop + yieldパターン生成
  - call文と同様の形式で`pasta::jump(ctx, "検索キー", #{}, [...])`を生成
  - グローバル/ローカル検索キーの生成ロジックはcallと同一
  - _Requirements: 5.9_

- [ ] 5.3 単語展開とローカル単語定義の生成
  - 単語展開: `for a in pasta::word(ctx, "単語", []) { yield a; }`
  - ローカル単語定義: `ctx.pasta.add_words("単語", ["値1", "値2"]); ctx.pasta.commit_words();`
  - _Requirements: 5.5_

- [ ] 5.4 変数代入と発言者切り替えの生成
  - 変数代入: `ctx.save.変数 = 値;`
  - 発言者切り替え: `ctx.actor = さくら; yield Actor("さくら");`
  - _Requirements: 5.6, 5.10_

- [ ] 5.5 (P) ContextCodegen単体テスト
  - call/jump文のfor-loop + yield生成を検証
  - 検索キー生成ロジックを検証（グローバル/ローカル）
  - 引数配列の正確な生成を検証
  - 単語展開、変数代入、発言者切り替えの生成を検証
  - _Requirements: 5.5, 5.6, 5.7, 5.8, 5.9, 5.10_

### Phase 4: Pass 2 - 予約関数生成

- [ ] 6. ReservedFunctionResolver実装（mod pasta{}生成）
- [ ] 6.1 `mod pasta {}`予約関数群生成
  - `jump()`, `call()`, `label_selector()`関数を生成
  - `label_selector()`内でID→関数パスマッピングのmatch文を生成
  - LabelRegistryから全ラベル情報を取得してmatch腕を生成
  - デフォルト腕でError生成クロージャを返却（`_ => |ctx, args| { yield Error(...); }`）
  - Pass 1の中間Runeコードに`mod pasta {}`を追加
  - _Requirements: 5.7, 5.8, 5.9_

- [ ] 6.2 (P) ReservedFunctionResolver単体テスト
  - `mod pasta {}`生成を検証
  - `label_selector()`のmatch文生成を検証
  - ID→関数パスマッピングの正確性を検証
  - _Requirements: 5.7, 5.8, 5.9_

### Phase 5: Pastaランタイム実装

- [ ] 7. PastaApi実装（pasta_stdlib::select_label_to_id関数登録）
- [ ] 7.1 P0実装: 完全一致ラベル解決関数
  - `pasta_stdlib::select_label_to_id(label, filters) -> i64`をRune関数として登録
  - P0: 静的HashMapで完全一致検索のみ実装
  - ラベル名→ID マッピングをトランスパイル時に生成
  - 存在しないラベルはエラー返却
  - _Requirements: 3.1, 3.2, 3.3_

- [ ] 7.2 (P) PastaApi単体テスト
  - P0実装の完全一致検索を検証
  - 存在しないラベルのエラー処理を検証
  - _Requirements: 3.1, 3.2, 3.3_

- [ ] 8. (P) LabelTableとWordDictionaryのSend実装
- [ ] 8.1 (P) LabelTableにSend trait実装
  - `unsafe impl Send for LabelTable {}`を追加
  - 内部状態（HashMap等）がSendを満たすことを確認
  - VM::send_execute()での送り込み可能性を保証
  - **注**: P0段階ではLabelTableは未使用（P1実装対象）
  - _Requirements: 8.1, 8.6_

- [ ] 8.2 (P) WordDictionaryにSend trait実装
  - `unsafe impl Send for WordDictionary {}`を追加
  - 内部状態がSendを満たすことを確認
  - VM::send_execute()での送り込み可能性を保証
  - **注**: P0段階ではWordDictionaryは単語展開スタブのみ
  - _Requirements: 8.2, 8.6_

### Phase 6: エンジン統合

- [ ] 9. PastaEngine統合（2パストランスパイルとランタイム登録）
- [ ] 9.1 PastaEngine::new()のリファクタリング
  - 2パストランスパイル実行（`transpile_pass1()` → `transpile_pass2()`）
  - PastaApiモジュール登録（`pasta_stdlib::select_label_to_id`）
  - Rune VMのContext構築とモジュールインストール
  - **P0範囲**: LabelTable/WordDictionary登録は後回し（P1対応）
  - _Requirements: 8.3, 8.4_

- [ ] 9.2 (P) PastaEngine統合テスト
  - 2パストランスパイル結果のRuneコンパイル成功を検証
  - `pasta_stdlib::select_label_to_id`関数の呼び出し可能性を検証
  - comprehensive_control_flow_simple.pastaの実行成功を検証
  - _Requirements: 8.3, 8.4, 8.5_

### Phase 7: サンプルファイル修正

- [ ] 10. 04_control_flow.pastaの修正
  - 命令型構文（`＠if`, `＠elif`, `＠else`, `＠while`）を全削除
  - call/jump/ラベル定義を使用した宣言的な実装例に書き換え
  - 動的call/jumpを使用したメニュー選択例を追加
  - ファイル冒頭のコメントを「宣言的コントロールフロー」に修正
  - **P0範囲**: ランダム選択とキャッシュベース消化は除外（P1対応）
  - _Requirements: 6.1, 6.2, 6.4, 6.5_

### Phase 8: 最終検証

- [ ] 11. 包括的統合テスト実行（必達条件の検証）
  - **🎯 必達**: `comprehensive_control_flow.pasta` → `comprehensive_control_flow.rn` トランスパイル成功
  - **🎯 必達**: トランスパイル結果が期待される `.rn` ファイルと厳密一致（`assert_eq!`）
  - comprehensive_control_flow_simple.pastaのトランスパイルテストがパスすることを確認
  - 既存の全テストケースがパスすることを確認
  - 04_control_flow.pastaが正常に実行できることを確認
  - P0 Validation Criteriaの全9項目を検証
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 2.1, 3.1, 3.2, 3.3, 4.1, 4.2, 4.3, 5.1, 5.2, 5.3, 5.4, 5.5, 5.6, 5.7, 5.8, 5.9, 5.10, 5.11, 5.12, 5.13, 6.1, 6.2, 6.4, 6.5, 7.1, 7.2, 7.3, 7.4, 7.5, 7.6, 7.7, 8.1, 8.2, 8.3, 8.4, 8.5, 8.6, 8.7, 8.8_

---

## Notes

### トランスパイラー設計の重要原則

**2パス戦略の正確な意味**:
1. **Pass 1**: Pasta AST → 中間Runeコード（文字列生成、Runeコンパイルなし）
2. **Pass 2**: 中間Runeコード + `mod pasta {}` → 最終Runeコード（文字列生成）
3. **Runeコンパイル**: 最終Runeコード → 実行可能Unit（**1回のみ実行**）

**Writeトレイト対応**:
- トランスパイラーは`std::io::Write`を出力先として受け取る
- 柔軟な出力先: String（メモリ）、File、Stderr、Vec<u8>など
- キャッシュ機能: `persistence_root/cache/pass1/`, `persistence_root/cache/final/`

**なぜRuneを2回コンパイルしないのか**:
- Runeは**コンパイル時に全ての名前を解決**する必要がある
- Pass 1で`pasta::call()`を参照するが、`mod pasta {}`はPass 2で生成
- したがって、Pass 1の出力は**不完全なRuneコード**（コンパイル不可）
- Pass 2で完全なRuneコードを生成してから、初めてコンパイル可能になる

**Runeのファイル拡張子とモジュール解決**:
- 正式な拡張子：`.rn`（`.rune`は誤り）
- `main.rn`のディレクトリが`mod foo;`の解決基準
- トランスパイル済みコードは`Source::new("entry", code)`で仮想登録（ファイルパスなし）
- 設計：トランスパイル済みコードは自己完結、main.rnから参照不要

### P0/P1スコープ分離

**P0範囲（本タスクリスト）**:
- 簡易ラベル解決（完全一致のみ）
- 同名ラベルなし（全ラベルに`_1`連番）
- 静的HashMap使用の`select_label_to_id`実装
- **🎯 comprehensive_control_flow.pasta の完全サポート（必達）**
- comprehensive_control_flow_simple.pasta による基礎検証

**重要**: `comprehensive_control_flow.pasta` は同名ラベルを使用していないため、P0実装で完全にサポート可能。

**P1範囲（別仕様: pasta-label-resolution-runtime）**:
- 前方一致検索
- **同名ラベル**のランダム選択
- 属性フィルタリング
- キャッシュベース消化
- 同名ラベルを使用する高度なテストケース

### 並列実行可能タスク

以下のタスクは並列実行可能（`(P)`マーク付き）:
- 2.2: LabelRegistry単体テスト（2.1完了後）
- 3.2: 2パストランスパイラー統合テスト（3.1完了後）
- 4.4: ModuleCodegen単体テスト（4.1-4.3完了後）
- 5.5: ContextCodegen単体テスト（5.1-5.4完了後）
- 6.2: ReservedFunctionResolver単体テスト（6.1完了後）
- 7.2: PastaApi単体テスト（7.1完了後）
- 8.1, 8.2: Send trait実装（互いに独立）
- 9.2: PastaEngine統合テスト（9.1完了後）

### 実装順序の理由

1. **Phase 1**: テストファーストで期待される出力を明確化（要件7に基づく）
2. **Phase 2-4**: トランスパイラー基盤（LabelRegistry → Pass 1 → Pass 2）
3. **Phase 5**: ランタイム実装（P0完全一致のみ）
4. **Phase 6**: エンジン統合
5. **Phase 7**: サンプルファイル修正
6. **Phase 8**: 最終検証

### 既存テストへの影響

トランスパイラー出力形式が全面変更されるため、以下の既存テストは期待値更新が必要:
- `transpiler_tests.rs`: 全テストケースの期待Runeコード更新
- `parser_tests.rs`: 影響なし（ASTレベル）
- `engine_integration_test.rs`: 実行結果は変わらないが、内部動作が変更

---

## Requirements Coverage

全8要件・46 Acceptance Criteriaをカバー:

| Requirement | Acceptance Criteria | Covered by Tasks |
|-------------|---------------------|------------------|
| 1 | 1.1-1.7 | 4.1, 4.2, 4.3, 5.1, 5.2, 11 |
| 2 | 2.1-2.5 | **P1対象**（別仕様） |
| 3 | 3.1-3.3 | 7.1, 7.2, 11 |
| 4 | 4.1-4.3 | 10, 11 |
| 5 | 5.1-5.13 | 1.2, 2.1, 3.1, 4.1, 4.2, 4.3, 5.1, 5.2, 5.3, 5.4, 6.1, 11 |
| 6 | 6.1-6.5 | 10, 11 |
| 7 | 7.1-7.7 | 1.1, 1.2, 1.3, 11 |
| 8 | 8.1-8.8 | 8.1, 8.2, 9.1, 9.2, 11 |

**注**: Requirement 2（ランダム選択と前方一致）はP1実装対象のため、本タスクリストでは除外。
