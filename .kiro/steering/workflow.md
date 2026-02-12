# Workflow - 開発ワークフロー

Kiro仕様駆動開発における作業フローと完了時アクション。

---

## 実装完了時のアクション

仕様の実装が完了した際、以下の手順を **この順序で** 実行すること。

### Step 1. 実装コミット & プッシュ

ソースコード変更をコミットし、リモートへプッシュする。

```bash
git add -A
git commit -m "<type>(<scope>): <summary>

<body>

Spec: <spec-name>"
git push origin <branch>
```

**コミットタイプ**: `feat` / `fix` / `refactor` / `docs` / `test`

### Step 2. 仕様フォルダを `completed/` に移動

**移動を先に行い、移動後に `spec.json` を更新する。**
（VS Code の不具合により、移動前にファイルを更新すると、エディタの確定操作で移動元に復活する場合がある）

```bash
mv .kiro/specs/<spec-name> .kiro/specs/completed/
```

### Step 3. `spec.json` の `phase` を更新

**移動後のパスで** `spec.json` を編集する。

- `phase` → `"implementation-complete"`
- `updated_at` → 現在日時

### Step 4. ロードマップ更新（該当する場合）

ROADMAPが存在する場合（メタ仕様配下の仕様など）：
- Progress Summary を更新
- Phase 列を `implementation-complete` に更新
- 📍 参照: `focus.md` のROADMAP更新タイミング

### Step 5. 完了コミット & プッシュ

仕様移動とメタデータ更新をコミットする。

```bash
git add -A
git commit -m "chore(specs): <spec-name> を完了フォルダに移動"
git push origin <branch>
```

### 完了チェックリスト

すべての Step を実行した後、以下を確認する：

- [ ] 全テストがパス（`cargo test`）
- [ ] スペックフォルダが `.kiro/specs/completed/<spec-name>/` に存在
- [ ] `spec.json` の `phase` が `"implementation-complete"`
- [ ] 移動元（`.kiro/specs/<spec-name>/`）にファイルが残っていない
- [ ] ロードマップ更新済み（該当する場合）

## 仕様フェーズフロー

```
requirements → design → tasks → implementation → implementation-complete
```

各フェーズ移行時に進捗を確認し、完了時は上記アクションを実行。

---
_Document patterns, not every workflow variation_
