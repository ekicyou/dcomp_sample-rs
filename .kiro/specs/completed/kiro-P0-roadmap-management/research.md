# Research & Design Decisions

---

| 項目 | 内容 |
|------|------|
| **Feature** | kiro-P0-roadmap-management |
| **Discovery Scope** | Simple Addition（ドキュメント生成のみ、技術実装なし） |

---

## Summary

本仕様はプロセス支援仕様であり、技術的な実装を伴わない。成果物は Markdown ドキュメント（ROADMAP.md、focus.md）のみ。

### Key Findings

1. **既存 steering 構造との統合**: `.kiro/steering/` 配下のファイルは AI セッション開始時に自動読み込みされるため、focus.md は軽量に保つ必要がある
2. **メタ仕様の配置パターン**: 親仕様ディレクトリ（`.kiro/specs/ukagaka-desktop-mascot/`）に ROADMAP.md を配置することで、メタ仕様と進捗情報を一元化
3. **フォルダ構成の明確化**: specs直下（アクティブ）、backlog（待機）、completed（完了）の3階層で仕様ライフサイクルを管理

---

## Research Log

### Topic: Steering ファイルのコンテキスト消費

- **Context**: focus.md に詳細情報を含めると、全セッションでコンテキストを消費する
- **Sources Consulted**: `.kiro/steering/` 配下の既存ファイル、AGENTS.md
- **Findings**:
  - steering ファイルは AI セッション開始時に自動読み込み
  - 現在の steering ファイル（product.md, tech.md, structure.md）は合計約5KB
  - focus.md は参照指示のみに留め、詳細は ROADMAP.md へ誘導すべき
- **Implications**: focus.md は 20-30 行以内に収め、ROADMAP.md への参照を主体とする

### Topic: ROADMAP.md の配置場所

- **Context**: ロードマップ情報をどこに配置するか
- **Sources Consulted**: 要件定義での議論
- **Findings**:
  - steering に置くとコンテキストを常に消費
  - 親仕様ディレクトリに置くとオンデマンド参照が可能
  - メタ仕様と関連ドキュメントの一元化が図れる
- **Implications**: `.kiro/specs/ukagaka-desktop-mascot/ROADMAP.md` に配置決定

---

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| Steering 集約 | すべてを steering に配置 | 自動読み込み | コンテキスト消費大 | 却下 |
| 親仕様集約 | ROADMAP を親仕様配下に配置 | オンデマンド参照、一元化 | 手動参照が必要 | **採用** |
| 分散配置 | 各仕様に進捗記載 | 情報の近接性 | 全体把握が困難 | 却下 |

---

## Design Decisions

### Decision: focus.md の責務を最小化

- **Context**: AI エージェントのコンテキスト制限への対応
- **Alternatives Considered**:
  1. focus.md にロードマップ情報を埋め込む — コンテキスト消費大
  2. focus.md は参照指示のみ — コンテキスト消費最小
- **Selected Approach**: Option 2（参照指示のみ）
- **Rationale**: steering は常に読み込まれるため、最小限の指示で必要時に ROADMAP.md を参照させる方が効率的
- **Trade-offs**: AI が自発的に ROADMAP を参照しない可能性があるが、focus.md の指示で補う

### Decision: フォルダ3階層構成

- **Context**: 仕様のライフサイクル管理
- **Alternatives Considered**:
  1. specs 直下のみ — 管理が煩雑
  2. 2階層（active/archive）— P1-P3 の区別不明確
  3. 3階層（直下/backlog/completed）— ライフサイクル明確
- **Selected Approach**: Option 3（3階層）
- **Rationale**: アクティブ（P0）、待機（P1-P3）、完了を明確に分離
- **Trade-offs**: フォルダ移動の手間が増えるが、フォーカスの明確化というメリットが大きい

---

## Risks & Mitigations

- **Risk 1**: AI が ROADMAP.md を参照し忘れる → focus.md で明確に参照タイミングを指示
- **Risk 2**: ROADMAP.md の更新忘れ → フェーズ変更時の更新ルールを focus.md に明記
- **Risk 3**: フォルダ移動の漏れ → 完了時は AI が移動、アクティブ化は開発者が移動という責務分担を明確化

---

## References

- `.kiro/steering/` - 既存 steering ファイル群
- `AGENTS.md` - kiro-style Spec Driven Development 概要
- `.kiro/steering/kiro-spec-schema.md` - spec.json スキーマ定義
