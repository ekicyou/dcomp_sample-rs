# Feature Specification: pasta-label-resolution-runtime

## Metadata
- **Feature Name**: pasta-label-resolution-runtime
- **Priority**: P1 (pasta-declarative-control-flow の後続)
- **Status**: Specification Only (未実装)
- **Parent Feature**: pasta-declarative-control-flow
- **Created**: 2025-12-12

## Overview

pasta-declarative-control-flow (P0) で生成される `pasta::label_selector()` 関数の実行時ラベル解決機能を実装する。

現在、P0実装では `label_selector()` は常に固定ID（`id = 1`）を返す仮実装となっており、実際のラベル名からIDへの動的解決は行われない。本仕様では、ランタイムでの前方一致検索、フィルタリング、ランダム選択を実装する。

## Problem Statement

### 現状の仮実装（P0）

```rune
pub mod pasta {
    pub fn label_selector(label, filters) {
        let id = 1; // 仮実装: 常に固定ID
        match id {
            1 => crate::会話_1::__start__,
            2 => crate::会話_1::選択肢_1,
            3 => crate::会話_1::選択肢_2,
            _ => panic!("Unknown label id: {}", id),
        }
    }
}
```

### 必要な機能

1. **ラベル名→ID解決**: `label` 文字列から適切なIDを検索
2. **前方一致検索**: `"会話"` → `"会話_1::__start__"` などの前方一致
3. **フィルタリング**: `filters` パラメータによる属性絞り込み
4. **ランダム選択**: 複数候補からの選択（RandomSelector使用）
5. **キャッシュベース消化**: 同じ検索キーでの選択肢の順次消化

## Technical Context

### LabelTable とのインターフェース

P0で定義された `LabelTable` インターフェース:

```rust
pub struct LabelTable {
    labels: HashMap<String, Vec<LabelInfo>>,
    history: HashMap<String, Vec<usize>>,
    random_selector: Box<dyn RandomSelector>,
}

impl LabelTable {
    /// ラベル名→ID解決（P1実装対象）
    pub fn resolve_label_id(
        &mut self,
        label: &str,
        filters: &HashMap<String, String>,
    ) -> Result<usize, PastaError>;
}
```

### PastaApi モジュール登録

P0で実装される Rust → Rune ブリッジ:

```rust
pub struct PastaApi;

impl PastaApi {
    pub fn create_module(
        label_table: Arc<Mutex<LabelTable>>,
    ) -> Result<Module, ContextError> {
        let mut module = Module::with_item(["pasta"])?;
        
        // Rust関数をRuneから呼び出し可能に
        let lt = Arc::clone(&label_table);
        module.function("resolve_label_id", move |label: &str, filters: HashMap<String, String>| -> Result<usize, String> {
            lt.lock().unwrap()
                .resolve_label_id(label, &filters)
                .map_err(|e| e.to_string())
        })?;
        
        Ok(module)
    }
}
```

### データ構造

```rust
pub struct LabelInfo {
    pub id: usize,                              // match文で使うID
    pub name: String,                           // 完全修飾名 ("会話" or "会話::選択肢")
    pub attributes: HashMap<String, String>,    // フィルタ属性 (＆time:morning など)
    pub fn_path: String,                        // 相対パス ("会話_1::__start__")
}
```

### 検索キー生成規則

- **グローバル検索**: `"会話"` → `"会話_1::__start__"` で前方一致
- **ローカル検索**: `"会話_1::選択肢"` → `"会話_1::選択肢_*"` で前方一致

## Requirements

### Requirement 1: 前方一致検索

**Objective**: ラベル名（検索キー）から前方一致する全候補を抽出する

**Acceptance Criteria**:
1. When `resolve_label_id("会話", {})` が呼ばれる, the LabelTable shall `"会話"` で始まり `"::__start__"` で終わる fn_path を持つ全LabelInfoを返す
2. When `resolve_label_id("会話_1::選択肢", {})` が呼ばれる, the LabelTable shall `"会話_1::選択肢"` で始まる fn_path を持つ全LabelInfoを返す
3. When 候補が0件の場合, the LabelTable shall `PastaError::LabelNotFound` を返す

### Requirement 2: 属性フィルタリング

**Objective**: filters パラメータによる候補の絞り込み

**Acceptance Criteria**:
1. When `resolve_label_id("会話", {"time": "morning"})` が呼ばれる, the LabelTable shall 前方一致した候補のうち `attributes["time"] == "morning"` のもののみを返す
2. When 複数のフィルタが指定される, the LabelTable shall すべてのフィルタ条件を満たす候補のみを返す
3. When フィルタ適用後の候補が0件の場合, the LabelTable shall `PastaError::NoMatchingLabel` を返す

### Requirement 3: ランダム選択

**Objective**: 複数候補から1つをランダムに選択

**Acceptance Criteria**:
1. When 前方一致候補が複数存在する, the LabelTable shall RandomSelector を使用して1つを選択する
2. When 候補が1つのみの場合, the LabelTable shall その候補を直接返す

### Requirement 4: キャッシュベース消化

**Objective**: 同じ検索キーでの選択肢の順次消化

**Acceptance Criteria**:
1. When 同じ検索キーで2回目の呼び出しが発生する, the LabelTable shall 1回目とは異なる候補を返す
2. When すべての候補が消化される, the LabelTable shall キャッシュをクリアし、次回は再選択する
3. When history に記録された候補を除外する, the LabelTable shall 残りの候補から選択する

## Implementation Notes

### Prefix-Tree (Trie) 検索

効率的な前方一致検索のため、`prefix-tree` クレートを使用:

```rust
use prefix_tree::PrefixTree;

pub struct LabelTable {
    trie: PrefixTree<String, LabelInfo>,  // fn_path → LabelInfo
    history: HashMap<String, Vec<usize>>,
    random_selector: Box<dyn RandomSelector>,
}

impl LabelTable {
    pub fn resolve_label_id(
        &mut self,
        label: &str,
        filters: &HashMap<String, String>,
    ) -> Result<usize, PastaError> {
        // 1. 前方一致検索
        let candidates: Vec<&LabelInfo> = self.trie
            .search_by_prefix(label)
            .collect();
        
        // 2. フィルタリング
        let filtered: Vec<&LabelInfo> = candidates.iter()
            .filter(|info| self.matches_filters(info, filters))
            .copied()
            .collect();
        
        // 3. 履歴除外
        let history = self.history.entry(label.to_string()).or_default();
        let available: Vec<&LabelInfo> = filtered.iter()
            .filter(|info| !history.contains(&info.id))
            .copied()
            .collect();
        
        // 4. 全消化後のリセット
        if available.is_empty() {
            history.clear();
            return self.resolve_label_id(label, filters); // 再帰
        }
        
        // 5. ランダム選択
        let selected = self.random_selector.select(&available)?;
        history.push(selected.id);
        
        Ok(selected.id)
    }
}
```

### LabelRegistry からの変換

P0で生成された LabelRegistry を LabelTable に変換:

```rust
impl LabelRegistry {
    pub fn into_label_table(self, random_selector: Box<dyn RandomSelector>) -> LabelTable {
        let mut trie = PrefixTree::new();
        
        for info in self.labels {
            let label_info = LabelInfo {
                id: info.id,
                name: info.name,
                attributes: info.attributes,
                fn_path: info.fn_path,
            };
            trie.insert(label_info.fn_path.clone(), label_info);
        }
        
        LabelTable {
            trie,
            history: HashMap::new(),
            random_selector,
        }
    }
}
```

## Testing Strategy

### Unit Tests

1. **前方一致テスト**: 様々な検索キーでの候補抽出確認
2. **フィルタテスト**: 単一/複数フィルタでの絞り込み確認
3. **ランダム選択テスト**: 複数候補からの選択動作確認
4. **キャッシュテスト**: 履歴管理と自動リセット確認

### Integration Tests

1. **エンドツーエンド**: DSL → Rune生成 → 実行時解決の全体フロー
2. **エラーハンドリング**: 存在しないラベル、フィルタ不一致のケース

## Dependencies

- **Prerequisite**: pasta-declarative-control-flow (P0) の完了
- **Crates**: `prefix-tree` または同等のTrie実装

## Future Work

- **パフォーマンス最適化**: Trie検索の高速化
- **拡張フィルタ構文**: 正規表現、範囲指定など
- **デバッグ支援**: ラベル解決のトレースログ機能

## References

- Parent Design: `.kiro/specs/pasta-declarative-control-flow/design.md`
- GRAMMAR.md: `crates/pasta/GRAMMAR.md` (属性構文)
- Current LabelTable: `crates/pasta/src/runtime/labels.rs`
