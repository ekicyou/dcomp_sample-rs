# Tasks: transform_system.rs → tree_system.rs への変更

**Status**: tasks_defined  
**Updated**: 2025-11-14  
**Feature**: transform-to-tree-refactor

## タスク概要

このリファクタリングは6つの独立したタスクに分解される。各タスクは順次実行される必要がある。

## タスクリスト

### Task 1: ファイルのリネーム
**ID**: TASK-001  
**優先度**: 高  
**依存関係**: なし  
**推定時間**: 1分

#### 説明
`transform_system.rs` を `tree_system.rs` にリネームする。ファイル内容は完全に保持。

#### 実装手順
```powershell
# ファイルをリネーム（Gitの追跡を保持）
Move-Item -Path "crates\wintf\src\ecs\transform_system.rs" -Destination "crates\wintf\src\ecs\tree_system.rs"
```

#### 成果物
- `crates/wintf/src/ecs/tree_system.rs` - 新しいファイル（381行）
- `crates/wintf/src/ecs/transform_system.rs` - 削除

#### 検証方法
```powershell
# ファイルが存在することを確認
Test-Path "crates\wintf\src\ecs\tree_system.rs"
# 元のファイルが削除されていることを確認
Test-Path "crates\wintf\src\ecs\transform_system.rs"
```

#### 完了条件
- [ ] `tree_system.rs` が存在する
- [ ] `transform_system.rs` が存在しない
- [ ] ファイル内容が381行で変更なし

---

### Task 2: ecs/mod.rs のモジュール宣言を更新
**ID**: TASK-002  
**優先度**: 高  
**依存関係**: TASK-001  
**推定時間**: 1分

#### 説明
`ecs/mod.rs` の5行目にあるモジュール宣言を `transform_system` から `tree_system` に変更する。

#### 実装手順
ファイル: `crates/wintf/src/ecs/mod.rs`

```rust
// 変更前（5行目）
pub mod transform_system;

// 変更後
pub mod tree_system;
```

#### 成果物
- `crates/wintf/src/ecs/mod.rs` - 1行変更

#### 検証方法
```powershell
# 変更内容を確認
Get-Content "crates\wintf\src\ecs\mod.rs" | Select-String "pub mod tree_system"
```

#### 完了条件
- [ ] `pub mod tree_system;` が存在する
- [ ] `pub mod transform_system;` が存在しない
- [ ] 他の行は変更されていない

---

### Task 3: ecs/mod.rs の再エクスポートを更新
**ID**: TASK-003  
**優先度**: 高  
**依存関係**: TASK-002  
**推定時間**: 1分

#### 説明
`ecs/mod.rs` の15行目にある再エクスポート文を `transform_system` から `tree_system` に変更する。

#### 実装手順
ファイル: `crates/wintf/src/ecs/mod.rs`

```rust
// 変更前（15行目）
pub use transform_system::*;

// 変更後
pub use tree_system::*;
```

#### 成果物
- `crates/wintf/src/ecs/mod.rs` - 1行変更（TASK-002と合わせて計2行）

#### 検証方法
```powershell
# 変更内容を確認
Get-Content "crates\wintf\src\ecs\mod.rs" | Select-String "pub use tree_system"
```

#### 完了条件
- [ ] `pub use tree_system::*;` が存在する
- [ ] `pub use transform_system::*;` が存在しない
- [ ] 他の行は変更されていない

---

### Task 4: tests/transform_test.rs のインポート文を更新
**ID**: TASK-004  
**優先度**: 高  
**依存関係**: TASK-003  
**推定時間**: 1分

#### 説明
`tests/transform_test.rs` の4行目にあるインポート文を `transform_system` から `tree_system` に変更する。

#### 実装手順
ファイル: `crates/wintf/tests/transform_test.rs`

```rust
// 変更前（4行目）
use wintf::ecs::transform_system::*;

// 変更後
use wintf::ecs::tree_system::*;
```

#### 成果物
- `crates/wintf/tests/transform_test.rs` - 1行変更

#### 検証方法
```powershell
# 変更内容を確認
Get-Content "crates\wintf\tests\transform_test.rs" | Select-Object -First 10 | Select-String "tree_system"
```

#### 完了条件
- [ ] `use wintf::ecs::tree_system::*;` が存在する
- [ ] `use wintf::ecs::transform_system::*;` が存在しない
- [ ] 他の668行は変更されていない

---

### Task 5: ビルド検証
**ID**: TASK-005  
**優先度**: 高  
**依存関係**: TASK-004  
**推定時間**: 2-3分

#### 説明
`cargo build` を実行し、全てのコードが正しくコンパイルされることを確認する。

#### 実装手順
```powershell
# プロジェクトをビルド
cargo build --package wintf
```

#### 成果物
- ビルド成功の確認

#### 検証方法
- ビルドが成功すること（exit code 0）
- 新しい警告が追加されていないこと
- コンパイルエラーが発生しないこと

#### 完了条件
- [ ] `cargo build --package wintf` が成功する
- [ ] 警告数が変わっていない（既存の2つのみ）
- [ ] `tree_system.rs` が正しくコンパイルされる

---

### Task 6: テスト実行と検証
**ID**: TASK-006  
**優先度**: 高  
**依存関係**: TASK-005  
**推定時間**: 3-5分

#### 説明
全てのテストを実行し、変更が既存の動作に影響を与えていないことを確認する。

#### 実装手順
```powershell
# 全テスト実行
cargo test --package wintf --test transform_test

# シナリオテストのみ実行（オプション）
cargo test test_scenario_ --package wintf
```

#### 成果物
- 全テストのパス確認

#### 検証方法
テストが以下の項目をカバーすること:
- 単体テスト7個（transform変換 + sync_simple_transforms）
- シナリオテスト8個（階層伝播の包括的テスト）

#### 完了条件
- [ ] 全15個のテストがパスする
- [ ] テスト失敗がない
- [ ] 特に以下の重要シナリオがパス:
  - test_scenario_1_deep_wide_hierarchy_propagation
  - test_scenario_2_partial_subtree_change
  - test_scenario_7_isolation_and_tree_reconstruction
  - test_scenario_8_dirty_mark_optimization

---

## タスク実行順序

```
TASK-001: ファイルリネーム
    ↓
TASK-002: mod.rs モジュール宣言更新
    ↓
TASK-003: mod.rs 再エクスポート更新
    ↓
TASK-004: テストのインポート文更新
    ↓
TASK-005: ビルド検証
    ↓
TASK-006: テスト実行
```

## リスク管理

### 高リスクポイント
1. **ファイル移動の失敗**: Git履歴が途切れる可能性
   - 対策: `Move-Item` または `git mv` を使用
   
2. **インポート文の更新漏れ**: コンパイルエラー
   - 対策: `cargo build` で即座に検出

3. **テスト失敗**: 予期しない動作変更
   - 対策: ファイル内容を変更していないため、発生しないはず
   - 万が一発生した場合: `git restore .` でロールバック

### ロールバック手順
```powershell
# コミット前の状態に戻す
git restore .

# または個別にファイルを戻す
git restore crates/wintf/src/ecs/
git restore crates/wintf/tests/transform_test.rs
```

## 検証チェックリスト

### 変更前の確認
- [ ] 現在のコードが正常にビルドできる
- [ ] 現在のテストが全てパスする
- [ ] Gitの作業ディレクトリがクリーン

### 変更後の確認
- [ ] 3つのファイルのみが変更されている
- [ ] `cargo build --package wintf` が成功
- [ ] `cargo test --package wintf --test transform_test` が全てパス
- [ ] 警告数が増えていない
- [ ] Git差分が予期したものと一致

### コミット前の確認
- [ ] `git status` で変更内容を確認
- [ ] `git diff` で差分が意図通り
- [ ] コミットメッセージが明確

## 推定総所要時間

- **タスク実行**: 10-15分
- **検証**: 5-10分
- **合計**: 15-25分

## 次のステップ

タスク完了後:
```bash
# 実装フェーズへ進む
/kiro-spec-impl transform-to-tree-refactor
```

または、手動実装の場合:
1. TASK-001から順次実行
2. 各タスクの完了条件を確認
3. 最終的にビルドとテストを実行
4. Gitコミット

---

**タスク定義完了** - 次のコマンド: `/kiro-spec-impl transform-to-tree-refactor`
