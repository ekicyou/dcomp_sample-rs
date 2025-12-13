# Implementation Report: Phase 2 - トランスパイラー基盤リファクタリング

| 項目 | 内容 |
|------|------|
| **実装日** | 2025-12-12 |
| **実装者** | GitHub Copilot CLI |
| **フェーズ** | Phase 2 - トランスパイラー基盤リファクタリング |
| **ステータス** | ✅ 完了 |

---

## 実装概要

Phase 2「トランスパイラー基盤リファクタリング」の全タスク（2.1, 2.2, 3.1, 3.2）を完了しました。
2パストランスパイラー戦略の基盤実装とLabelRegistryによるラベル管理システムを構築しました。

---

## 完了したタスク

### ✅ Task 2.1: LabelRegistry実装

**ファイル**: `crates/pasta/src/transpiler/label_registry.rs`

**実装内容**:

1. **LabelInfo構造体**:
   - `id`: ラベルの一意なID（1から開始）
   - `name`: ラベル名（接頭辞なし）
   - `attributes`: 属性情報（P1用、P0では空）
   - `fn_path`: Rune関数パス（例: `crate::会話_1::__start__`）
   - `parent`: 親ラベル名（ローカルラベルのみ）

2. **LabelRegistry構造体**:
   - `labels`: ID→LabelInfoマップ
   - `next_id`: 次に割り当てるID（自動採番）
   - `name_counters`: ラベル名→カウンターマップ（同名ラベル管理）

3. **主要メソッド**:
   - `register_global()`: グローバルラベル登録（例: `会話` → ID 1, counter 1, path `crate::会話_1::__start__`）
   - `register_local()`: ローカルラベル登録（例: `会話::選択肢` → ID 2, counter 1, path `crate::会話_1::選択肢_1`）
   - `all_labels()`: 全ラベルをIDソート順で取得
   - `get_label()`: IDからラベル情報取得

**設計の特徴**:
- **P0実装**: 同名ラベルに対応（自動的に `_1`, `_2`, `_3` と連番付与）
- **P0範囲**: comprehensive_control_flow.pastaは同名ラベルを使用しないため、全て `_1` になる
- **拡張性**: P1で前方一致検索やランダム選択を実装する際の基盤

**要件カバレッジ**: Requirements 5.2, 5.3, 5.4

---

### ✅ Task 2.2: LabelRegistry単体テスト

**ファイル**: 
- `crates/pasta/src/transpiler/label_registry.rs` (内部テスト)
- `crates/pasta/tests/label_registry_test.rs` (統合テスト)

**テストケース**:

1. **test_register_global_label**: グローバルラベル登録とID採番
2. **test_register_multiple_global_labels**: 複数グローバルラベル
3. **test_register_duplicate_global_labels**: 同名ラベルの連番管理
4. **test_register_local_label**: ローカルラベル登録とfn_path生成
5. **test_sanitize_name**: 識別子サニタイズ
6. **test_label_registry_basic**: 基本的な登録と取得
7. **test_label_registry_with_local_labels**: ローカルラベルの階層構造
8. **test_label_registry_duplicate_names**: 同名ラベルの連番テスト

**テスト結果**: ✅ 全テスト合格（8/8）

**要件カバレッジ**: Requirements 5.2, 5.3, 5.4

---

### ✅ Task 3.1: Transpilerインターフェースのリファクタリング

**ファイル**: `crates/pasta/src/transpiler/mod.rs`

**実装内容**:

1. **Pass 1 メソッド**: `transpile_pass1<W: Write>(file, registry, writer)`
   - PastaFileをパースしてLabelRegistryに登録
   - グローバルラベル → `pub mod ラベル名_番号 { pub fn __start__(ctx) { ... } }`
   - ローカルラベル → 親モジュール内の関数として生成
   - `use pasta_stdlib::*;` をファイル先頭に追加
   - **重要**: `mod pasta {}` は生成しない（Pass 2で生成）

2. **Pass 2 メソッド**: `transpile_pass2<W: Write>(registry, writer)`
   - `mod pasta {}` を生成:
     - `pub fn jump(ctx, label, filters, args)`
     - `pub fn call(ctx, label, filters, args)`
     - `pub fn label_selector(label, filters)`: ID→関数パスのmatch文
   - LabelRegistryから全ラベル情報を取得してmatch腕を生成
   - デフォルト腕でエラー生成

3. **ヘルパーメソッド**: `transpile_to_string(file)` (テスト専用)
   - Pass 1とPass 2を連続実行して文字列を返す
   - **注意**: 単一ファイル専用、本番コードでは使用禁止

4. **補助メソッド**:
   - `transpile_global_label()`: グローバルラベルの変換
   - `transpile_local_label()`: ローカルラベルの変換
   - `transpile_statement_to_writer()`: ステートメントの変換（P0: Speech文のみ）

**Writeトレイト対応**:
- `std::io::Write`トレイトを使用
- `Vec<u8>`, `File`, `Cursor<Vec<u8>>`など柔軟な出力先に対応
- 将来的なキャッシュ実装の基盤

**既存APIの保持**:
- `transpile(file)`: 既存の単一パストランスパイラー（非推奨）
- 後方互換性のために残してあるが、新しいコードでは `transpile_pass1` + `transpile_pass2` を使用

**要件カバレッジ**: Requirements 5.1, 5.2

---

### ✅ Task 3.2: 2パストランスパイラー統合テスト

**ファイル**: `crates/pasta/tests/two_pass_transpiler_test.rs`

**テストケース**:

1. **test_two_pass_transpiler_to_vec**: Vec<u8>への出力
   - Pass 1出力の検証（mod pastaなし）
   - Pass 2出力の検証（mod pasta追加）

2. **test_two_pass_transpiler_to_string**: String出力のシミュレーション
   - 複数グローバルラベルの処理
   - LabelRegistryの状態確認
   - label_selectorのマッピング検証

3. **test_transpile_to_string_helper**: ヘルパーメソッドのテスト
   - transpile_to_string()の動作確認
   - Pass 1とPass 2の統合検証

4. **test_multiple_files_simulation**: 複数ファイル処理のシミュレーション
   - Pass 1を2回呼び出し（file1, file2）
   - 共有LabelRegistryの使用
   - Pass 2で全ファイルのラベルをまとめて処理

**テスト結果**: ✅ 全テスト合格（4/4）

**既存テストの確認**:
- `transpiler_comprehensive_test.rs`: ✅ 合格（transpile_to_string()使用に更新）
- `label_registry_test.rs`: ✅ 合格（3/3）

**要件カバレッジ**: Requirements 5.1, 5.2

---

## 技術的な成果

### 2パストランスパイラー戦略の実現

**なぜ2パスが必要か**:
1. Pass 1で全ファイルのラベルを収集する必要がある
2. Pass 2で全ラベルのID→関数パスマッピングを生成する必要がある
3. Pass 1の出力は不完全（`pasta::call()`を参照するが`mod pasta`が未定義）
4. Runeコンパイルは最後に1回だけ実行（Pass 1の出力はコンパイル不可）

**実装の正確性**:
- ✅ Pass 1: 文字列生成のみ（Runeコンパイルなし）
- ✅ Pass 2: 文字列生成のみ（Runeコンパイルなし）
- ✅ Runeコンパイル: 最後に1回だけ（Phase 6で実装予定）

### Writeトレイト対応による柔軟性

**対応可能な出力先**:
- `Vec<u8>`: メモリ内バッファ
- `File`: ファイル直接書き込み
- `Cursor<Vec<u8>>`: 位置付きバッファ
- `Stderr`: 標準エラー出力

**将来の拡張**:
- キャッシュディレクトリへの書き込み: `persistence_root/cache/pass1/`
- 複数ファイルの並列処理
- インクリメンタル再コンパイル

### LabelRegistryによる統一管理

**メリット**:
1. **一貫性**: 全ファイルで一意なID管理
2. **追跡可能性**: ラベル名→ID→関数パスの完全なトレース
3. **拡張性**: P1でのフィルタリング機能追加が容易
4. **デバッグ性**: ラベル情報の可視化が簡単

---

## ファイル構成

```
crates/pasta/src/
├── transpiler/
│   ├── mod.rs                    (更新: 2パスAPI追加)
│   └── label_registry.rs         (新規: ラベル管理)
├── error.rs                      (更新: io_error()追加)

crates/pasta/tests/
├── label_registry_test.rs        (新規: LabelRegistry統合テスト)
├── two_pass_transpiler_test.rs   (新規: 2パス統合テスト)
└── transpiler_comprehensive_test.rs (更新: transpile_to_string()使用)
```

---

## 次のステップ

Phase 3「Pass 1 - モジュール構造生成」に進みます:
- Task 4.1: ModuleCodegen実装（グローバルラベル→モジュール変換）
- Task 4.2: `__start__`関数生成
- Task 4.3: ローカルラベル関数生成
- Task 4.4: ModuleCodegen単体テスト

現状のtranspile_statement_to_writer()はSpeech文の基本的な変換のみ実装しています。
Phase 3-5でcall/jump文、単語展開、変数代入などの変換機能を追加します。

---

## 完了基準の確認

### P0 Validation Criteria (Phase 2関連)

- ✅ LabelRegistry実装完了
- ✅ グローバルラベル登録とID採番が正常動作
- ✅ ローカルラベル登録とfn_path生成が正常動作
- ✅ 連番管理ロジックが正常動作（同名ラベル対応）
- ✅ 2パストランスパイラーインターフェース実装
- ✅ Writeトレイト対応
- ✅ Pass 1とPass 2の分離
- ✅ transpile_to_string()ヘルパー実装
- ✅ 全単体テスト合格
- ✅ 全統合テスト合格

---

## 関連要件

| Requirement | Acceptance Criteria | 実装状況 |
|-------------|---------------------|----------|
| 5.1 | Pass 1とPass 2の分離 | ✅ 完了 |
| 5.2 | LabelRegistry実装 | ✅ 完了 |
| 5.3 | __start__関数生成（基本実装） | ✅ 完了 |
| 5.4 | ローカルラベル関数生成（基本実装） | ✅ 完了 |

**注**: Task 4-5でさらに詳細な変換ロジックを実装します。

---

**最終更新**: 2025-12-12
**次のフェーズ**: Phase 3 - Pass 1 - モジュール構造生成
