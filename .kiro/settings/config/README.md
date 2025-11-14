# Kiro Specs 構造設定ガイド

## 設定ファイル

`.kiro/settings/config/specs.json` で仕様書の構造を管理します。

## デフォルト構造: サブフォルダ

```
.kiro/specs/{feature-name}/
  ├── spec.md       # 仕様書本体
  └── init.json     # メタデータ（phase, approvals等）
```

### 例
```
.kiro/specs/transform_system_test/
  ├── spec.md
  └── init.json
```

## 設定の変更方法

`.kiro/settings/config/specs.json` を編集：

```json
{
  "structure": "subfolder",  // または "single_file"
  "spec_filename": "spec.md"
}
```

### オプション

- **`subfolder`** (デフォルト): `.kiro/specs/{feature}/spec.md`
- **`single_file`**: `.kiro/specs/{feature}.md`

## Kiroコマンドへの影響

`/kiro-spec-init` は自動的にこの設定を参照して、適切な構造で仕様ファイルを作成します。

---
*このガイドは `.kiro/settings/config/specs.json` と連動しています*
