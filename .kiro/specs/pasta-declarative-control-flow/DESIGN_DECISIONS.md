# Design Decisions - Pasta Declarative Control Flow

このドキュメントは、実装者がコンテキストクリア後も設計意図を理解できるよう、重要な設計決定を記録します。

---

## 🎯 Critical Design Decision: 2パストランスパイル戦略

### 決定内容

トランスパイラーは**2パス戦略**を採用し、**Writeトレイト**を出力先として受け取ります。

```rust
impl Transpiler {
    pub fn transpile<W: std::io::Write>(
        file: &PastaFile, 
        writer: &mut W
    ) -> Result<(), PastaError>;
}
```

### 理由

#### 1. Runeのコンパイルモデルとの適合

**問題**: Runeは**コンパイル時に全ての名前を解決**する必要がある。

```rune
// Pass 1で生成されるコード（不完全）
pub mod 会話_1 {
    pub fn __start__(ctx) {
        for a in pasta::call(ctx, "ラベル", #{}, []) { yield a; }
        //       ^^^^^^^^^^^^^ この時点では未定義
    }
}
// mod pasta {} はまだ存在しない ← コンパイル不可
```

**解決策**: Pass 2で`mod pasta {}`を追加してから、初めてRuneコンパイルを実行。

```
Pass 1: Pasta AST → 中間Rune文字列（pasta::call参照、mod pastaなし）
  ↓
Pass 2: 中間Rune文字列 + mod pasta {} → 完全なRune文字列
  ↓
Runeコンパイル: 完全なRune文字列 → 実行可能Unit（1回のみ）
```

#### 2. なぜ「2パス」なのか

- **Pass 1**: ラベル収集が必要（AST全体を走査してLabelRegistryを構築）
- **Pass 2**: 収集したラベル情報から`mod pasta {}`のID→関数パスマッピングを生成

**重要**: 「2パス」とは**トランスパイラー内部の処理段階**であり、Runeコンパイルは最後に1回だけ。

#### 3. Writeトレイトを採用した理由

**問題**: 最初の設計では`String`を返していたが、以下の課題があった：
- デバッグ時にPass 1の出力を確認できない
- 大きなファイルでメモリを圧迫する可能性
- キャッシュファイルへの出力が非効率

**解決策**: `std::io::Write`を出力先として受け取る。

#### 4. Runeのモジュール解決とファイル拡張子

**重要な発見**: Runeの正式な拡張子は`.rn`（`.rune`ではない）

**モジュール解決の仕組み**:
```rust
// main.rnのパス: /path/to/script/main.rn
sources.insert(rune::Source::from_path("/path/to/script/main.rn"))?;

// main.rn内で "mod foo;" と書くと:
// → /path/to/script/foo.rn または
// → /path/to/script/foo/mod.rn を自動ロード
```

**現在の設計での扱い**:
```rust
// トランスパイル済みコードは仮想ソース（ファイルパスなし）
sources.insert(rune::Source::new("entry", transpiled_code))?;

// main.rnはパスから読み込み
sources.insert(rune::Source::from_path("script_root/main.rn"))?;

// 一括コンパイル（全ソースが同時に見える）
let unit = rune::prepare(&mut sources).build()?;
```

**なぜパス解決を気にしなくてよいか**:
- トランスパイル済みコードは完全に自己完結（全モジュールを含む）
- main.rnは単なるエントリーポイント
- main.rnからトランスパイル済みモジュールを参照する必要はない
- 実行はRust側の`PastaEngine::execute_label()`から呼び出される

**メリット**:
- ✅ 柔軟な出力先: String、File、Stderr、Vec<u8>など
- ✅ Pass 1とPass 2を別ファイルに出力可能（デバッグ用）
- ✅ メモリ効率的（ストリーミング書き込み）
- ✅ テスト容易性向上

### 使用例

```rust
// ✅ 本番コード: PastaEngine::new()での使用
pub fn new(script_root: impl AsRef<Path>) -> Result<Self> {
    let loaded = DirectoryLoader::load(script_root)?;
    let mut registry = LabelRegistry::new();
    let mut output = String::new();
    
    // Pass 1: 各pastaファイルを処理
    for pasta_file in &loaded.pasta_files {
        let ast = parse_file(pasta_file)?;
        Transpiler::transpile_pass1(&ast, &mut registry, &mut output)?;
    }
    
    // Pass 2: mod pasta {} を生成（1回のみ）
    Transpiler::transpile_pass2(&registry, &mut output)?;
    
    // Runeコンパイル（1回のみ）
    let unit = rune::prepare(&output).build()?;
    Ok(Self { unit, ... })
}

// ✅ オプション: 個別ファイルキャッシュ（デバッグ用）
let cache_dir = persistence_root.join("cache/pass1");
std::fs::create_dir_all(&cache_dir)?;

for pasta_file in &loaded.pasta_files {
    let ast = parse_file(pasta_file)?;
    let file_name = pasta_file.file_stem().unwrap();
    let cache_path = cache_dir.join(format!("{}.rn", file_name));
    let mut cache_file = File::create(&cache_path)?;
    
    Transpiler::transpile_pass1(&ast, &mut registry, &mut cache_file)?;
}

// ✅ テストコード: 単一ファイルの便利メソッド（テスト専用）
#[test]
fn test_simple_transpile() {
    let ast = parse_str("＊会話\n　さくら：こんにちは", "test.pasta")?;
    // transpile_to_string()はテスト専用（本番コードでは使わない）
    let output = Transpiler::transpile_to_string(&ast)?;
    assert!(output.contains("pub mod 会話_1"));
}
```

### キャッシュディレクトリ構造

```
persistence_root/
  ├── save/           # セーブデータ
  ├── cache/          # トランスパイルキャッシュ（オプショナル）
  │   ├── pass1/      # Pass 1出力（デバッグ用）
  │   │   └── transpiled.rn
  │   └── final/      # 最終Runeコード
  │       └── transpiled.rn
  └── logs/           # エラーログ
```

---

## 🎯 Critical Design Decision: 期待値ファイルのクリーン版

### 決定内容

`comprehensive_control_flow.rn`（期待値）を2つのバージョンで管理：

1. **クリーン版**: `comprehensive_control_flow.rn`（テスト用）
   - コメントなし
   - トランスパイラーが実際に出力する形式
   - `assert_eq!`による厳密比較用

2. **注記付き版**: `comprehensive_control_flow.annotated.rn`（リファレンス）
   - 詳細な説明コメント付き
   - 実装時の参考資料
   - 不正な改変の証跡

### 理由

**問題**: オリジナルの期待値ファイルには大量のコメントが含まれていたが、トランスパイラーはコメントを出力しない。

**解決策**: 
- テスト用のクリーン版を作成（コメントなし）
- オリジナルは`.annotated.rn`として保存

**メリット**:
- ✅ `assert_eq!`による厳密比較が可能
- ✅ テストを通すための不正な改変を防止
- ✅ オリジナルの設計意図が保存される
- ✅ 実装時の参考資料として利用可能

### 重要な原則

**改変の禁止**: テストを通すために`.rn`ファイルを変更してはいけません。

トランスパイラーの出力が期待と異なる場合：
- ❌ `.rn`ファイルを修正してテストを通す
- ✅ トランスパイラーを修正して正しい出力を生成する

---

## 🎯 Critical Design Decision: P0スコープと必達条件

### 決定内容

**P0範囲**で`comprehensive_control_flow.pasta`の**完全サポート**を必達条件とします。

### 理由

**誤解の防止**: 当初「P0は簡易版のみ、完全版はP1」と誤解されかねない表現があった。

**事実**:
- `comprehensive_control_flow.pasta`は**同名ラベルを使用していない**
- したがって、完全一致検索（P0実装）で完全にサポート可能
- P1で追加されるのは「同名ラベルのランダム選択」などの高度な機能

### P0/P1の正確な違い

| 項目 | P0範囲 | P1範囲 |
|------|--------|--------|
| ラベル解決 | 完全一致のみ | 前方一致検索 |
| 同名ラベル | なし（全て異なる名前） | あり（ランダム選択） |
| テストケース | `comprehensive_control_flow.pasta` | 同名ラベルを使用する高度なケース |
| 属性フィルタリング | なし | あり |
| キャッシュベース消化 | なし | あり |

---

## 🎯 Implementation Guidelines

### トランスパイラー実装時の注意点

1. **Pass 1は複数回呼び出し可能**
   - 各PastaFileごとに呼び出される
   - LabelRegistryに各ファイルのラベル情報が蓄積される
   - 出力先は同じWriterを使い回す

2. **Pass 2は最後に1回だけ呼び出し**
   - 全てのPass 1処理完了後
   - LabelRegistryから`mod pasta {}`を生成

3. **Pass 1とPass 2は文字列生成のみ**
   - Runeコンパイルは呼び出さない
   - 単なる文字列の生成と連結

4. **Writeトレイトを正しく使用**
   - `write!()`, `writeln!()`マクロを使用
   - エラーハンドリングを適切に行う

5. **transpile_to_string()はテスト専用**
   - `#[doc(hidden)]`で隠す
   - 本番コードでは使用しない（複数ファイル非対応）
   - ドキュメントに注意書きを明記

6. **キャッシュは後から追加可能**
   - P0実装ではメモリ上で完結
   - デバッグビルド時は`eprintln!`で出力を確認
   - キャッシュ機能は将来の拡張として実装

7. **Runeファイルの拡張子は`.rn`**
   - `.rune`ではない（これは誤り）
   - プロジェクト全体で統一済み
   - `main.rn`, テストフィクスチャー`*.rn`

8. **PastaFileはAST情報を持つ構造体**
   - 1つの`.pasta`ファイル = 1つの`PastaFile`
   - 複数ファイルの処理：各`PastaFile`ごとにPass 1を呼び出し
   - LabelRegistryに各ファイルのラベル情報が蓄積される

### テスト実装時の注意点

1. **クリーン版の期待値を使用**
   - `comprehensive_control_flow.rn`（コメントなし）
   - `assert_eq!`で厳密比較

2. **注記付き版は参考資料**
   - `comprehensive_control_flow.annotated.rn`
   - 実装時の理解を助ける

3. **テストを通すために期待値を変更しない**
   - トランスパイラーを修正して正しい出力を生成

---

## 📚 関連ドキュメント

- [requirements.md](./requirements.md): 要件定義
- [design.md](./design.md): 技術設計書
- [tasks.md](./tasks.md): 実装タスクリスト
- [tests/fixtures/README.md](../../crates/pasta/tests/fixtures/README.md): テストフィクスチャーの説明

---

**最終更新日**: 2025-12-12
