# Implementation Checklist - 実装前確認事項

このドキュメントは、実装を開始する前に確認すべき重要な設計決定をまとめています。

---

## ✅ 設計原則の確認

### 1. トランスパイラーの2パス戦略

- [ ] **Pass 1**: 文字列生成のみ（Runeコンパイルなし）
- [ ] **Pass 2**: 文字列生成のみ（Runeコンパイルなし）
- [ ] **Runeコンパイル**: 最後に1回だけ実行

**重要**: Pass 1とPass 2は単なる文字列操作。Runeコンパイラーは一切呼ばない。

### 2. インターフェース設計

```rust
impl Transpiler {
    // ✅ 主要API: 複数ファイル対応
    pub fn transpile_pass1<W: Write>(
        file: &PastaFile,
        registry: &mut LabelRegistry,
        writer: &mut W
    ) -> Result<(), PastaError>;
    
    pub fn transpile_pass2<W: Write>(
        registry: &LabelRegistry,
        writer: &mut W
    ) -> Result<(), PastaError>;
    
    // ⚠️ テスト専用: 本番コードでは使わない
    #[doc(hidden)]
    pub fn transpile_to_string(file: &PastaFile) -> Result<String, PastaError>;
}
```

### 3. 使用方法

```rust
// ✅ 正しい使い方（本番コード）
let mut registry = LabelRegistry::new();
let mut output = String::new();

for pasta_file in &files {
    let ast = parse_file(pasta_file)?;
    Transpiler::transpile_pass1(&ast, &mut registry, &mut output)?;
}

Transpiler::transpile_pass2(&registry, &mut output)?;

// ❌ 間違った使い方
let output = Transpiler::transpile_to_string(&ast)?; // テスト専用
```

---

## ✅ Rune関連の重要事項

### 1. ファイル拡張子

- **正しい**: `.rn`
- **間違い**: `.rune`

**影響範囲**:
- プロジェクト構造: `script_root/main.rn`
- テストフィクスチャー: `*.rn`
- コード内の全参照

### 2. モジュール解決の仕組み

```rust
// Source::from_path() で読み込んだファイルのディレクトリが基準
sources.insert(rune::Source::from_path("/path/to/main.rn"))?;

// main.rn 内で "mod foo;" と書くと:
// → /path/to/foo.rn または
// → /path/to/foo/mod.rn を自動ロード
```

### 3. 現在の設計での扱い

```rust
// トランスパイル済みコード = 仮想ソース（ファイルパスなし）
sources.insert(rune::Source::new("entry", transpiled_code))?;

// main.rn = 実ファイル（パスあり）
sources.insert(rune::Source::from_path("script_root/main.rn"))?;

// 一括コンパイル
let unit = rune::prepare(&mut sources).build()?;
```

**なぜパス解決を気にしなくてよいか**:
- トランスパイル済みコードは完全に自己完結
- main.rnは単なるエントリーポイント
- main.rnからトランスパイル済みモジュールを参照しない

---

## ✅ PastaFileとLabelRegistry

### PastaFileとは

```rust
pub struct PastaFile {
    pub path: PathBuf,              // ファイルパス
    pub labels: Vec<LabelDef>,      // ファイル内の全グローバルラベル
    pub span: Span,                 // ソース位置情報
}
```

- **1つの`.pasta`ファイル** = **1つの`PastaFile`**
- 複数ファイルを処理する場合は、各`PastaFile`ごとにPass 1を呼び出す

### LabelRegistryの役割

```rust
let mut registry = LabelRegistry::new();

// Pass 1を複数回呼び出してラベル情報を蓄積
for pasta_file in &files {
    Transpiler::transpile_pass1(&ast, &mut registry, &mut output)?;
    // registry にラベル情報が追加される
}

// Pass 2で registry から mod pasta {} を生成
Transpiler::transpile_pass2(&registry, &mut output)?;
```

---

## ✅ 必達条件（Critical Success Criteria）

### P0実装で必ず達成すること

1. ✅ `comprehensive_control_flow.pasta` → `comprehensive_control_flow.rn` トランスパイル成功
2. ✅ トランスパイル結果が期待される `.rn` ファイルと厳密一致
3. ✅ P0 Validation Criteria（11項目）すべて合格

**重要**: `comprehensive_control_flow.pasta` は同名ラベルを使用していないため、P0実装（完全一致検索）で完全にサポート可能。

### テストファイルの管理

**クリーン版（テスト用）**:
- `comprehensive_control_flow.rn` - コメントなし、厳密比較用

**注記付き版（リファレンス）**:
- `comprehensive_control_flow.annotated.rn` - 詳細コメント付き、参考資料

**改変禁止**: テストを通すために`.rn`ファイルを変更してはいけない。トランスパイラーを修正して正しい出力を生成すること。

---

## ✅ 実装の順序

### Phase 1: テスト基盤
- [x] Task 1.1: comprehensive_control_flow_simple.pasta作成
- [x] Task 1.2: 期待される出力.rn作成
- [x] Task 1.3: トランスパイルテスト作成

### Phase 2: トランスパイラー基盤
- [ ] Task 2.1: LabelRegistry実装
- [ ] Task 2.2: LabelRegistry単体テスト

### Phase 3: Pass 1実装
- [ ] Task 3.1: Transpilerインターフェース実装
- [ ] Task 4.1: ModuleCodegen実装（グローバルラベル → mod）
- [ ] Task 5.1: ContextCodegen実装（call/jump/word変換）

### Phase 4: Pass 2実装
- [ ] Task 6.1: ReservedFunctionResolver実装（mod pasta生成）

### Phase 5: ランタイム統合
- [ ] Task 7.1: PastaApi実装（select_label_to_id関数）
- [ ] Task 8.1-8.2: Send trait実装（LabelTable/WordDictionary）

### Phase 6: エンジン統合
- [ ] Task 9.1: PastaEngine統合

### Phase 7: サンプル修正
- [ ] Task 10.1: 04_control_flow.pasta修正

### Phase 8: 最終検証
- [ ] Task 11: 包括的統合テスト（必達条件検証）

---

## ✅ よくある質問

### Q1: なぜPass 1とPass 2を分けるのか？

**A**: ラベル収集が必要だから。

- Pass 1: 全PastaFileを走査してラベル情報をLabelRegistryに蓄積
- Pass 2: 蓄積されたラベル情報から`mod pasta {}`のID→関数パスマッピングを生成

### Q2: なぜRuneを2回コンパイルしないのか？

**A**: Pass 1の出力は不完全だから。

- Pass 1で生成されるコードは`pasta::call()`を参照
- しかし`mod pasta {}`はPass 2で生成される
- したがってPass 1の出力はコンパイル不可
- Pass 2で完全なコードを作ってから初めてコンパイル可能

### Q3: transpile_to_string()を本番コードで使ってはいけない理由は？

**A**: 複数ファイルに対応していないから。

- transpile_to_string()は1つのPastaFileしか処理できない
- 本番では複数の.pastaファイルを処理する必要がある
- transpile_pass1()を複数回呼び出す必要がある

---

## 📚 関連ドキュメント

- [requirements.md](./requirements.md) - 要件定義
- [design.md](./design.md) - 技術設計書
- [tasks.md](./tasks.md) - 実装タスクリスト
- [DESIGN_DECISIONS.md](./DESIGN_DECISIONS.md) - 重要な設計決定
- [FILE_EXTENSION_FIX.md](./FILE_EXTENSION_FIX.md) - ファイル拡張子の修正記録

---

**最終更新**: 2025-12-12

**次のステップ**: Phase 2 (Task 2.1: LabelRegistry実装) から開始
