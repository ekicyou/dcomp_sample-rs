# Implementation Gap Analysis: pasta-label-resolution-runtime

| 項目 | 内容 |
|------|------|
| **Feature Name** | pasta-label-resolution-runtime |
| **Analysis Date** | 2025-12-14 |
| **Analyzer** | GitHub Copilot |
| **Parent Spec** | pasta-declarative-control-flow (completed) |
| **Priority** | P1 |

---

## 分析サマリー

**スコープ:** Pasta DSLの実行時ラベル解決機能（前方一致検索、属性フィルタリング、ランダム選択、キャッシュベース消化）を実装し、Rust側の `LabelTable::resolve_label_id()` とRune側の `pasta_stdlib::select_label_to_id()` を統合する。

**主要な課題:**
- 現在の `LabelTable::find_label()` は完全一致検索（HashMap）のみをサポート、前方一致検索が未実装
- 履歴管理が配列インデックスベースで脆弱（フィルタ変動や候補順序変更に非対応）
- Rune ↔ Rust の型変換（HashMap、Object型）が未定義

**推奨アプローチ:** 
- **Option A（推奨）:** 既存の `LabelTable` を拡張し、`resolve_label_id()` メソッドを追加。HashMap + フルスキャンで前方一致検索を実装し、履歴管理をラベルIDベースに変更。
- **実装難易度:** M (3-7日) - 既存パターンの拡張、適度な複雑度
- **リスク:** Low - 既存アーキテクチャとの整合性が高く、明確な統合ポイントあり

---

## 1. Current State Investigation

### 1.1 既存アセット

#### 核心モジュール

| モジュール | パス | 責務 | 再利用可能性 |
|-----------|------|------|------------|
| **LabelTable** | `crates/pasta/src/runtime/labels.rs` | ラベル管理、完全一致検索、属性フィルタリング、ランダム選択 | ✅ 高 - 拡張ベース |
| **RandomSelector** | `crates/pasta/src/runtime/random.rs` | ランダム選択の抽象化、MockRandomSelector | ✅ 完全再利用 |
| **LabelRegistry** | `crates/pasta/src/transpiler/label_registry.rs` | トランスパイル時のラベル収集とID割り当て | ✅ 完全再利用 |
| **PastaStdlib** | `crates/pasta/src/stdlib/mod.rs` | Rune標準ライブラリ、`select_label_to_id()` スタブ実装 | ✅ 拡張必要 |
| **PastaError** | `crates/pasta/src/error.rs` | エラー型定義 | ✅ 新規エラー追加 |

#### データ構造

**LabelTable（既存）:**
```rust
pub struct LabelTable {
    labels: HashMap<String, Vec<LabelInfo>>,  // ⚠️ 完全一致のみ
    history: HashMap<String, Vec<usize>>,     // ⚠️ インデックス記録（問題あり）
    random_selector: Box<dyn RandomSelector>,
}
```

**LabelInfo（既存）:**
```rust
pub struct LabelInfo {
    pub name: String,              // DSL上のラベル名
    pub scope: LabelScope,
    pub attributes: HashMap<String, String>,
    pub fn_name: String,           // Rune関数名（"会話_1::__start__"）
    pub parent: Option<String>,
}
```

**重要:** 現在の `LabelInfo` には `id` フィールドが存在しない。`LabelRegistry::LabelInfo` には `id: i64` フィールドがあるが、`runtime::LabelInfo` には欠落している。本実装では `runtime::LabelInfo` に `pub id: usize` フィールドを追加する。

#### アーキテクチャパターン

1. **トレイトベースの抽象化**: `RandomSelector` トレイトでテスタビリティを確保
2. **2パストランスパイラー**: Pass 1でラベル収集 → Pass 2で `mod pasta {}` 生成
3. **Rust ↔ Rune ブリッジ**: `create_module()` でRune関数を登録
4. **エラー処理**: `thiserror::Error` を使用した構造化エラー
5. **テストパターン**: `MockRandomSelector` で決定論的テスト、`tests/common/` で共通ユーティリティ

### 1.2 統合ポイント

#### 呼び出しフロー

```
Rune: label_selector(label, filters)
  ↓
Rune: pasta_stdlib::select_label_to_id(label, filters)  ← スタブ実装
  ↓
Rust: LabelTable::resolve_label_id(label, filters)      ← 未実装
  ↓
Rust: 前方一致検索 (fn_path.starts_with(search_key))
  ↓
Rust: フィルタリング (属性マッチ + "::__start__"サフィックスフィルタ)
  ↓
Rust: ランダム選択 (RandomSelector::select_index)
  ↓
Rust: 履歴記録 (history[search_key].push(selected.id))
  ↓
Return: ラベルID (i64)
  ↓
Rune: match id { ... } → 関数ポインタ取得 → 実行
```

**検索キー例:**
- `JumpTarget::Global("会話")` → `"会話"` → `"会話_1::__start__"`, `"会話_2::__start__"` にマッチ
- `JumpTarget::Local("選択肢")` → `"選択肢"` → 親ラベルコンテキスト内で検索
- `JumpTarget::LongJump{"会話", "選択肢"}` → `"会話::選択肢"` → `"会話_1::選択肢_1"` にマッチ

#### 現在のスタブ実装

```rust
// crates/pasta/src/stdlib/mod.rs (L56-67)
fn select_label_to_id(_label: String, _filters: rune::runtime::Value) -> i64 {
    // P0: Always return 1 for basic testing
    1
}
```

**統合課題:**
- Runeの `Value` 型から Rust の `HashMap<String, String>` への変換が必要
- `LabelTable` への参照を `select_label_to_id()` に渡す仕組みが未定義

### 1.3 命名規則と慣例

- **Rust型**: `PascalCase` (構造体、列挙型、トレイト)
- **Rust関数**: `snake_case`
- **Runeモジュール**: `snake_case` (`pasta_stdlib`)
- **エラー型**: `PastaError::<Variant>` 形式
- **テスト**: `test_<機能>` または `test_<ケース>`
- **モックオブジェクト**: `Mock<Original>` (例: `MockRandomSelector`)

---

## 2. Requirements Feasibility Analysis

### 2.1 要件からの技術ニーズ

| 要件 | 技術ニーズ | 既存実装 | Gap |
|------|-----------|---------|-----|
| **Req 1: 前方一致検索** | `fn_path` のプレフィックスマッチング | `HashMap::get()` 完全一致のみ | ❌ Missing |
| **Req 2: 属性フィルタリング** | `LabelInfo::attributes` のANDフィルタ | ✅ `find_label()` で実装済み | ✅ Reuse |
| **Req 3: ランダム選択** | `RandomSelector::select_index()` | ✅ 実装済み | ✅ Reuse |
| **Req 4: キャッシュベース消化** | ラベルIDベース履歴管理 | ⚠️ インデックスベース履歴 | ⚠️ Constraint |
| **Req 5: Rust ↔ Rune ブリッジ** | Rune Value → HashMap 変換、`LabelTable` 参照渡し | ⚠️ スタブのみ | ❌ Missing |
| **Req 6: Registry → Table 変換** | `LabelRegistry` → `LabelTable` 変換時にIDフィールド追加 | ⚠️ IDフィールド欠落 | ⚠️ Constraint |

### 2.2 ギャップと制約

#### Gap 1: 前方一致検索の未実装

**現状:**
```rust
// labels.rs L94-105
pub fn find_label(&mut self, name: &str, ...) -> Result<String, PastaError> {
    let candidates = self.labels.get(name)?;  // ← HashMap::get() 完全一致のみ
    // ...
}
```

**必要な機能:**
- `fn_path` が検索キーで始まるすべての `LabelInfo` を抽出
- 例: 検索キー `"会話"` → `"会話_1::__start__"`, `"会話_2::__start__"` にマッチ
- グローバルラベル: `"::__start__"` で終わるものをフィルタ

**採用決定: HashMap + フルスキャン (Option A)**
```rust
let candidates: Vec<&LabelInfo> = labels_by_path
    .iter()
    .filter(|(path, _)| path.starts_with(search_key))
    .map(|(_, info)| info)
    .collect();
```

**代替案 (Phase 3):**
- **Trie構造**: `prefix_tree` クレート使用（O(M)の検索、M = キー長）、ラベル数1000以上時に検討

#### Gap 2: ラベルIDフィールドの欠落

**現状:**
- `LabelRegistry::LabelInfo` には `id: i64` フィールドあり
- `LabelTable::LabelInfo` には `id` フィールドなし

**影響:**
- 履歴管理でIDを記録できない（現在はインデックスを記録）
- `resolve_label_id()` がIDを返す要件を満たせない

**解決策:**
- `runtime/labels.rs` の `LabelInfo` に `pub id: usize` フィールドを追加（または `i64` で統一）
- `from_label_registry()` で `registry_info.id` を `runtime_info.id` にコピー
- **注記:** `LabelRegistry` は `i64` を使用しているが、ランタイムでは配列インデックスとの親和性を考慮して `usize` に変換することを推奨

#### Gap 3: 履歴管理の脆弱性

**現状:**
```rust
// labels.rs L143-149
self.history
    .entry(name.to_string())
    .or_insert_with(Vec::new)
    .push(
        candidates.iter().position(|l| l.fn_name == selected.fn_name).unwrap(),
        // ↑ 配列のインデックスを記録（問題あり）
    );
```

**問題:**
- フィルタが異なる呼び出しで候補順序が変わる → インデックスが無効化
- ラベル追加・削除でインデックスがずれる

**解決策:**
- `history: HashMap<String, Vec<usize>>` の `Vec<usize>` をラベルIDのリストに変更
- フィルタごとに履歴を分離: `history: HashMap<String, Vec<usize>>` → `history: HashMap<(String, HashMap<String, String>), Vec<usize>>`
  - または履歴キーを `format!("{}:{:?}", label, filters)` に変更

#### Gap 4: Rune ↔ Rust 型変換の未定義

**現状:**
```rust
fn select_label_to_id(_label: String, _filters: rune::runtime::Value) -> i64 {
    // filters は rune::runtime::Value 型（未変換）
}
```

**必要な機能:**
- Runeの `Value` 型から Rust の `HashMap<String, String>` への変換
- Rune側で空のHashMapが渡された場合の処理

**調査項目 (Research Needed):**
- Runeの `Value::into_object()` または `Value::into_any()` の使用方法
- Rune Object → Rust HashMap の変換パターン（Rune公式ドキュメント）

#### Gap 5: LabelTableへの参照渡し

**現状:**
- `select_label_to_id()` は引数として `label` と `filters` のみを受け取る
- `LabelTable` への参照がない

**既存パターン:**
```rust
// engine.rs L58
pub struct PastaEngine {
    label_table: LabelTable,  // ← エンジンが所有
    // ...
}
```

**実装オプション:**
1. **グローバルスレッドローカル変数**: `thread_local!` で `LabelTable` を保持（推奨しない）
2. **Runeモジュール生成時に注入**: `create_module()` で `Arc<Mutex<LabelTable>>` をキャプチャ
3. **ctx経由で渡す**: Runeコードで `pasta_stdlib::select_label_to_id(ctx, label, filters)` とし、`ctx` から `LabelTable` を取得

**推奨:** Option 2 - `create_module()` でクロージャーキャプチャ（既存の `persistence` モジュールと同じパターン）

### 2.3 複雑性シグナル

- **アルゴリズム複雑度**: 中程度 - 前方一致検索（線形走査）、フィルタリング（ネストループ）
- **外部統合**: Rune VM統合（既存パターンあり）
- **データ構造変更**: `LabelInfo` へのIDフィールド追加、履歴管理の変更
- **並行性**: `Arc<Mutex<LabelTable>>` でスレッドセーフ性は確保済み

---

## 3. Implementation Approach Options

### Option A: Extend Existing LabelTable（推奨）

**概要:** 既存の `LabelTable` に `resolve_label_id()` メソッドを追加し、`find_label()` と並存させる。

#### 変更対象ファイル

1. **`crates/pasta/src/runtime/labels.rs`**
   - `LabelInfo` に `pub id: usize` フィールド追加
   - `LabelTable::resolve_label_id()` メソッド追加
   - 履歴管理を `Vec<usize>` （ラベルID）に変更
   - 前方一致検索ロジック実装（HashMap + フルスキャン）

2. **`crates/pasta/src/stdlib/mod.rs`**
   - `select_label_to_id()` のスタブを実装版に置き換え
   - `create_module()` で `Arc<Mutex<LabelTable>>` をキャプチャ
   - Rune Value → HashMap 変換処理

3. **`crates/pasta/src/error.rs`**
   - 新規エラー型追加:
     - `PastaError::NoMatchingLabel`
     - `PastaError::InvalidLabel`
     - `PastaError::RandomSelectionFailed`
     - `PastaError::DuplicateLabelPath`

4. **`crates/pasta/src/engine.rs`**
   - `PastaEngine::new()` で `LabelTable` を `Arc<Mutex<>>` でラップ
   - `create_module()` に `LabelTable` を渡す

#### 互換性評価

- ✅ **後方互換性**: `find_label()` は変更せず、`resolve_label_id()` を追加するため既存コードは動作継続
- ✅ **インターフェース整合性**: `RandomSelector` トレイトをそのまま使用
- ⚠️ **破壊的変更**: `LabelInfo` 構造体にフィールド追加（デシリアライゼーション時の注意が必要）

#### 複雑性とメンテナビリティ

- **ファイルサイズ**: `labels.rs` は現在314行 → 約450行に増加（許容範囲）
- **単一責務原則**: ラベル解決という単一ドメインに集中（✅ 維持）
- **認知負荷**: `find_label()` と `resolve_label_id()` の2つのエントリーポイント（中程度）

#### Trade-offs

- ✅ 既存のテストコード（`tests/` 配下の `LabelTable` 使用箇所）が継続動作
- ✅ `from_label_registry()` の変更のみで統合可能
- ✅ アーキテクチャ変更なし
- ❌ `LabelInfo` へのIDフィールド追加が破壊的変更の可能性
- ❌ 履歴管理ロジックが2系統存在（`find_label()` はインデックス、`resolve_label_id()` はID）

**実装手順:**
1. `LabelInfo` に `id` 追加、`from_label_registry()` を修正
2. `resolve_label_id()` の骨格実装（前方一致検索なし、常に最初の候補を返す）
3. 前方一致検索ロジック追加（HashMap + フルスキャン）
4. 履歴管理をIDベースに変更
5. Rust ↔ Rune ブリッジ実装
6. テストケース追加

---

### Option B: Create New LabelResolver Component

**概要:** `LabelTable` は変更せず、新しい `LabelResolver` コンポーネントを作成し、前方一致検索専用とする。

#### 責務分離

- **LabelTable**: 既存の完全一致検索、属性フィルタリング、履歴管理（変更なし）
- **LabelResolver**: 前方一致検索、IDベース履歴、`resolve_label_id()` の実装

#### 統合ポイント

```rust
// 新規ファイル: crates/pasta/src/runtime/label_resolver.rs
pub struct LabelResolver {
    labels_by_path: HashMap<String, LabelInfo>,  // fn_path → LabelInfo
    history: HashMap<String, Vec<usize>>,        // 検索キー → ラベルIDリスト
    random_selector: Box<dyn RandomSelector>,
}

impl LabelResolver {
    pub fn new(label_table: &LabelTable, random_selector: Box<dyn RandomSelector>) -> Self {
        // LabelTableから変換
    }
    
    pub fn resolve_label_id(&mut self, label: &str, filters: &HashMap<String, String>) -> Result<usize, PastaError> {
        // 前方一致検索 + フィルタリング + ランダム選択
    }
}
```

#### Trade-offs

- ✅ `LabelTable` を一切変更しない（完全に既存動作を保護）
- ✅ 責務が明確に分離
- ✅ テストが独立
- ❌ 2つのラベル管理コンポーネントが並存（混乱の可能性）
- ❌ `PastaEngine` が2つのコンポーネントを保持する必要
- ❌ `LabelTable` → `LabelResolver` の変換オーバーヘッド

**非推奨理由:**
- ラベル解決は本質的に同一ドメイン（分離する必要性が低い）
- 既存の `LabelTable` が既にフィルタリングやランダム選択を実装済み（重複）

---

### Option C: Hybrid Approach（段階的移行）

**概要:** Phase 1で Option A の実装を行い、Phase 2で `find_label()` を非推奨化し、`resolve_label_id()` に統一する。

#### Phase 1: 並存期間
- `resolve_label_id()` を追加（Option A と同じ）
- `find_label()` は変更せず維持
- 新規コードは `resolve_label_id()` を使用

#### Phase 2: 段階的移行（将来）
- `find_label()` に `#[deprecated]` 属性を追加
- 全ての呼び出し箇所を `resolve_label_id()` に置き換え
- 履歴管理を統一（IDベースのみ）

#### Trade-offs

- ✅ 段階的な移行でリスク低減
- ✅ 既存コードの動作保証
- ✅ 将来的に単一のAPIに統一可能
- ❌ Phase 2の実装が不確実（技術的負債の可能性）
- ❌ 移行期間中は2つのAPIが並存（ドキュメント負荷）

---

## 4. Recommended Approach & Key Decisions

### 推奨: Option A（Extend Existing LabelTable）

**理由:**
1. **既存パターンとの整合性**: `LabelTable` は既にラベル解決の中核であり、拡張が自然
2. **実装効率**: 既存の `RandomSelector`, `LabelInfo`, `attributes` を再利用
3. **テストカバレッジ**: 既存の `MockRandomSelector` とテストユーティリティをそのまま使用可能
4. **統合容易性**: `from_label_registry()` の変更のみで統合完了

### 主要な設計決定

#### Decision 1: 前方一致検索の実装方式

**選択: HashMap + フルスキャン（Phase 1）**

**理由:**
- ラベル数が典型的に100～500程度でパフォーマンス問題なし
- 外部クレート依存なし（シンプル）
- O(N)の走査コストは許容範囲（10ms以下）

**将来の移行パス:**
- Phase 2でパフォーマンス問題が発生した場合、Trie構造に移行
- データ構造の変更は `LabelTable` 内部のみで完結

#### Decision 2: 履歴管理のキー生成

**選択: `format!("{}:{:?}", label, filters)` でキーを生成**

**理由:**
- フィルタごとに履歴を分離（要件4のAcceptance Criteria 3）
- シンプルな実装（HashMap<String, Vec<usize>>をそのまま使用）

**代替案:**
- `(String, HashMap<String, String>)` をキーとする → Hash実装が必要

#### Decision 3: LabelTable参照の渡し方

**選択: `create_module()` で `Arc<Mutex<LabelTable>>` をキャプチャ**

**理由:**
- 既存の `persistence` モジュールと同じパターン（`stdlib/persistence.rs` 参照）
- Runeのシグネチャを変更不要（`select_label_to_id(label, filters)` のまま）

**実装例:**
```rust
// stdlib/mod.rs
pub fn create_module(label_table: Arc<Mutex<LabelTable>>) -> Result<Module, ContextError> {
    let mut module = Module::with_crate("pasta_stdlib")?;
    
    let lt = Arc::clone(&label_table);
    module.function("select_label_to_id", move |label: String, filters: rune::runtime::Value| -> Result<i64, String> {
        // Rune Value → HashMap 変換
        let filters_map = parse_rune_filters(filters)?;
        
        // LabelTableを呼び出し
        let mut table = lt.lock().unwrap();
        let id = table.resolve_label_id(&label, &filters_map)
            .map_err(|e| e.to_string())?;
        
        Ok(id as i64)
    }).build()?;
    
    Ok(module)
}
```

#### Decision 4: エラーハンドリング戦略

**選択: 構造化エラー追加 + Rune側でpanic**

**新規エラー型:**
```rust
#[derive(Error, Debug)]
pub enum PastaError {
    #[error("No matching label for '{label}' with filters {filters:?}")]
    NoMatchingLabel {
        label: String,
        filters: HashMap<String, String>,
    },
    
    #[error("Invalid label name: '{label}'")]
    InvalidLabel { label: String },
    
    #[error("Random selection failed")]
    RandomSelectionFailed,
    
    #[error("Duplicate label path: {path}")]
    DuplicateLabelPath { path: String },
}
```

**Rune側での処理:**
```rune
// トランスパイラーが生成（変更なし）
let id = pasta_stdlib::select_label_to_id(label, filters);
match id {
    1 => crate::会話_1::__start__,
    _ => { yield pasta_stdlib::Error(`ラベルID ${id} が見つかりませんでした。`); },
}
```

---

## 5. Implementation Complexity & Risk

### Effort: M（3-7日）

**内訳:**
- **Day 1-2**: `LabelInfo` へのIDフィールド追加、`resolve_label_id()` の骨格実装、単体テスト
- **Day 3-4**: 前方一致検索ロジック、履歴管理の変更、追加テストケース
- **Day 5-6**: Rust ↔ Rune ブリッジ実装、Rune Value変換、統合テスト
- **Day 7**: エンドツーエンドテスト、ドキュメント更新、エッジケース対応

**既存パターンの活用:**
- `RandomSelector` トレイト、`MockRandomSelector` をそのまま使用（Day 0.5削減）
- `from_label_registry()` の変更のみで統合（Day 1削減）
- `tests/common/` のユーティリティ再利用（Day 0.5削減）

### Risk: Low

**理由:**
- ✅ **既知技術**: Rust標準ライブラリ（HashMap, Vec）のみ、外部クレート依存なし
- ✅ **明確な統合ポイント**: `from_label_registry()`, `create_module()` の変更箇所が限定的
- ✅ **テスト容易性**: `MockRandomSelector` で決定論的テスト可能
- ✅ **パフォーマンス懸念小**: O(N)の走査コスト、想定ラベル数で問題なし

**潜在的リスクと対策:**

| リスク | 影響 | 対策 |
|--------|------|------|
| Rune Value → HashMap 変換失敗 | Medium | Rune公式ドキュメント調査、fallback処理実装 |
| 前方一致検索の誤動作（連番処理） | Medium | 包括的なユニットテスト、エッジケース網羅 |
| 履歴管理のメモリリーク | Low | 履歴クリア処理の実装、メモリ使用量テスト |
| LabelInfo構造体変更による互換性問題 | Low | デフォルト値設定、移行テスト |

---

## 6. Research Items for Design Phase

### Research 1: Rune Value → HashMap 変換

**調査項目:**
- `rune::runtime::Value::into_object()` の使用方法
- Rune Object から Rust HashMap への変換パターン
- 空のHashMapが渡された場合の扱い（`Value::Unit` か `Value::Object(empty)` か）

**参照:**
- Rune公式ドキュメント: https://rune-rs.github.io/
- 既存の `persistence` モジュール（`stdlib/persistence.rs`）

### Research 2: 前方一致検索の最適化

**調査項目:**
- ラベル数が1000以上になった場合のパフォーマンス測定
- `prefix_tree` クレートまたは `radix_trie` クレートの評価
- Trie構造導入時のメモリオーバーヘッド

**判断基準:**
- 解決時間が10ms以上かかる場合は最適化を検討
- メモリ使用量が10MB以下であればTrie導入可

### Research 3: 履歴管理のメモリ効率化

**調査項目:**
- 長時間実行時の履歴メモリ使用量の増加傾向
- LRUキャッシュの導入可否
- 履歴の自動クリア戦略（例: N回実行後、またはメモリ閾値超過時）

---

## 7. Test Strategy

### 既存テストパターンの活用

**再利用可能なテストユーティリティ:**
- `tests/common/create_test_script()`: 一時的なスクリプトファイル生成
- `tests/common/create_unique_persistence_dir()`: 独立した永続化ディレクトリ
- `runtime::random::MockRandomSelector`: 決定論的ランダム選択

### 新規テストケース

| テストカテゴリ | テストケース数 | ファイル配置案 |
|---------------|--------------|--------------|
| **ユニットテスト** | 15 | `crates/pasta/src/runtime/labels.rs` (`#[cfg(test)]` セクション) |
| **統合テスト** | 5 | `crates/pasta/tests/label_resolution_test.rs` |
| **エンドツーエンド** | 3 | `crates/pasta/tests/comprehensive_control_flow_test.rs` (既存拡張) |

**ユニットテストケース（抜粋）:**
1. `test_resolve_label_id_forward_match`: 前方一致検索の基本動作
2. `test_resolve_label_id_with_filters`: 属性フィルタリング
3. `test_resolve_label_id_cache_exhaustion`: キャッシュ全消化とリセット
4. `test_resolve_label_id_history_by_filter`: フィルタごとの履歴分離
5. `test_resolve_label_id_empty_label`: 空文字列エラー
6. `test_resolve_label_id_no_candidates`: 候補なしエラー
7. `test_resolve_label_id_filter_no_match`: フィルタ不一致エラー

---

## 8. Requirement-to-Asset Map

| 要件 | 既存アセット | Gap/Constraint | 実装アプローチ |
|------|------------|---------------|--------------|
| **Req 1: 前方一致検索** | `LabelTable::find_label()` (完全一致) | ❌ Missing: 前方一致ロジック | HashMap + フルスキャン実装 |
| **Req 2: 属性フィルタリング** | ✅ `LabelTable::find_label()` のフィルタリングロジック | - | 再利用 |
| **Req 3: ランダム選択** | ✅ `RandomSelector` トレイト、`MockRandomSelector` | - | 再利用 |
| **Req 4: キャッシュベース消化** | `LabelTable::history` (インデックスベース) | ⚠️ Constraint: IDベースに変更必要 | 履歴管理を `Vec<usize>` (ID) に変更 |
| **Req 5: Rust ↔ Rune ブリッジ** | `stdlib::select_label_to_id()` (スタブ) | ❌ Missing: 実装版、型変換 | `create_module()` で `Arc<Mutex<LabelTable>>` キャプチャ |
| **Req 6: Registry → Table 変換** | ✅ `LabelTable::from_label_registry()` | ⚠️ Constraint: IDフィールド欠落 | `LabelInfo` に `id` 追加 |

---

## 9. Summary & Next Steps

### 実装推奨事項

1. **Approach:** Option A（Extend Existing LabelTable）を採用
2. **Priority 1:** `LabelInfo` へのIDフィールド追加、`from_label_registry()` 修正
3. **Priority 2:** `resolve_label_id()` の骨格実装（前方一致なし、単純な最初候補返却）
4. **Priority 3:** 前方一致検索ロジック（HashMap + フルスキャン）
5. **Priority 4:** Rust ↔ Rune ブリッジ実装、型変換

### Design Phase への引き継ぎ

**確定事項:**
- 既存 `LabelTable` の拡張方針
- HashMap + フルスキャンでの前方一致検索
- IDベースの履歴管理
- `create_module()` でのクロージャーキャプチャ

**要研究事項:**
- Rune Value → HashMap の具体的変換コード
- エッジケースの網羅的なテストケース設計
- パフォーマンステストの実施計画

### Design Generation へ進む条件

- ✅ Gap分析完了
- ✅ 実装アプローチ決定
- ✅ 主要な設計決定完了

**次のステップ:**
- `/kiro-spec-design pasta-label-resolution-runtime` でデザインフェーズへ進行
- または、Gap分析の結果を踏まえて要件定義を修正

---

## References

- **親仕様:** `.kiro/specs/completed/pasta-declarative-control-flow/`
- **現在の実装:** `crates/pasta/src/runtime/labels.rs`
- **標準ライブラリ:** `crates/pasta/src/stdlib/mod.rs`
- **エラー型:** `crates/pasta/src/error.rs`
- **テストユーティリティ:** `crates/pasta/tests/common/`
