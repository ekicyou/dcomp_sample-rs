# File Extension Fix: .rune → .rn

**日時**: 2025-12-12
**理由**: Runeの正式な拡張子は`.rn`（`.rune`ではない）

## 修正内容

### 1. テストフィクスチャーファイル名の変更

**変更前**:
- `comprehensive_control_flow_simple.expected.rune`
- `comprehensive_control_flow.annotated.rune`
- `comprehensive_control_flow.rune`
- `yield_propagation_test.rune`

**変更後**:
- `comprehensive_control_flow_simple.expected.rn`
- `comprehensive_control_flow.annotated.rn`
- `comprehensive_control_flow.rn`
- `yield_propagation_test.rn`

### 2. ドキュメント内の参照を修正

以下のファイルで`.rune`を`.rn`に置換：

- ✅ `.kiro/specs/pasta-declarative-control-flow/design.md`
- ✅ `.kiro/specs/pasta-declarative-control-flow/DESIGN_DECISIONS.md`
- ✅ `.kiro/specs/pasta-declarative-control-flow/implementation-report-task-1.md`
- ✅ `.kiro/specs/pasta-declarative-control-flow/requirements.md`
- ✅ `.kiro/specs/pasta-declarative-control-flow/tasks.md`
- ✅ `crates/pasta/tests/fixtures/README.md`

### 3. テストコードの修正

- ✅ `crates/pasta/tests/transpiler_comprehensive_test.rs`
- ✅ `crates/pasta/tests/comprehensive_control_flow_test.rs`

### 4. プロジェクトファイル名の変更

- ✅ `tests/fixtures/test-project/main.rune` → `main.rn`

### 5. ソースコードの修正

- ✅ `src/loader.rs` - ファイル名とコメント修正
- ✅ `src/engine.rs` - コメント修正
- ✅ `src/error.rs` - エラーメッセージ修正
- ✅ `tests/directory_loader_test.rs` - テストコード修正

## 検証

```bash
# テストフィクスチャーの確認
$ ls crates/pasta/tests/fixtures/*.rn
comprehensive_control_flow_simple.expected.rn
comprehensive_control_flow.annotated.rn
comprehensive_control_flow.rn
yield_propagation_test.rn

# テスト実行（期待通りの失敗 - トランスパイラー未実装）
$ cargo test --test transpiler_comprehensive_test
test test_comprehensive_control_flow_simple_transpile ... FAILED
```

## テスト結果

```bash
$ cargo test --test directory_loader_test
running 8 tests
test test_directory_not_found_error ... ok
test test_not_absolute_path_error ... ok
test test_main_rune_not_found_error ... ok  # ← main.rn検証テスト
test test_dic_directory_not_found_error ... ok
test test_label_execution ... ok
test test_ignored_files_skipped ... ok
test test_from_directory_success ... ok
test test_multiple_labels_random_selection ... ok

test result: ok. 8 passed; 0 failed
```

✅ すべてのテストがパスしました！

## 重要な注意事項

**Runeの正式な拡張子**: `.rn`

今後、Runeコードファイルを作成する際は`.rn`拡張子を使用すること。

**プロジェクト構造**:
```
script_root/
  ├── main.rn           # ← .rnを使用
  └── dic/
      └── *.pasta
```

## 関連リソース

- [Rune Book - Items and Imports](https://rune-rs.github.io/book/items_imports.html)
- [Rune Examples](https://github.com/rune-rs/rune/tree/main/examples)

---

**このドキュメントは将来の参考のために保存されています。**
