# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | メタ仕様ロードマップ管理システム 要件定義書 |
| **Version** | 1.3 |
| **Date** | 2025-11-29 |
| **Parent Specification** | ukagaka-desktop-mascot |
| **Author** | Claude Opus 4.5 + えちょ |

---

## Introduction

本仕様書は、`ukagaka-desktop-mascot` のような**メタ仕様**（多数の子仕様を統括する親仕様）を効果的に駆動するための**ロードマップ管理システム**の要件を定義する。

### 背景

`ukagaka-desktop-mascot` メタ仕様は32件の子仕様（31機能 + 1プロセス支援）を持ち、P0〜P3の優先度で年単位の開発を駆動する。このような大規模仕様を管理するにあたり、以下の課題が浮上した：

1. **AIエージェントのコンテキスト制限**: 長期プロジェクトでは、セッションをまたいでAIがプロジェクト全体を把握することが困難
2. **進捗追跡の複雑さ**: 32件の子仕様それぞれが独自のライフサイクルを持つため、全体の進捗把握が困難
3. **優先度と依存関係の管理**: P0→P1→P2→P3の順序と、Tier依存関係の両方を考慮した実行計画が必要
4. **steering ドキュメントの役割**: 現在の開発フォーカスをAIに伝える仕組みが不明確

### 本仕様の位置づけ

本仕様は「プロセス支援仕様」であり、技術的な機能実装ではなく、**開発プロセスとAIエージェント連携の仕組み**を定義する。成果物はドキュメントとsteeringファイルの形で提供される。

### スコープ

- **対象**: kiro-style Spec Driven Development を採用したプロジェクト
- **目的**: メタ仕様の長期駆動を支援する仕組みの構築
- **成果物**: ROADMAP.md、steeringドキュメント（`/kiro-steering-custom` で生成）
- **完了条件**: 設計承認後、`/kiro-steering-custom` コマンドで成果物を生成して完了

---

## Requirements

### 1. ロードマップ駆動ドキュメント

**Objective:** 開発者/AIとして、プロジェクトの長期計画と現在のフォーカスを一目で把握したい。それにより効率的に開発を進められる。

#### Acceptance Criteria

1. **The** Roadmap System **shall** 親仕様ディレクトリ（`.kiro/specs/{meta-spec}/ROADMAP.md`）にロードマップを配置する
2. **The** Roadmap System **shall** アクティブな子仕様の一覧と優先度を単一ドキュメントで表示する
3. **The** Roadmap System **shall** 各子仕様の現在のフェーズ（requirements-draft / design-approved / implementing / completed 等）を表示する
4. **The** Roadmap System **shall** 現在の開発フォーカス（今取り組むべき子仕様）を明示する
5. **When** 子仕様のフェーズが変更された時, **the** Roadmap System **shall** ドキュメント更新手段を提供する
6. **The** Roadmap System **shall** Tier依存関係（技術的な実装順序）を視覚的に表現する

---

### 2. Steering ドキュメント拡張（focus.md）

**Objective:** AIエージェントとして、ROADMAP.mdの参照・更新タイミングを知りたい。それにより適切なタイミングで進捗情報を確認・更新できる。

#### Acceptance Criteria

1. **The** focus.md **shall** ROADMAP.mdの参照タイミング（セッション開始時、次仕様決定時等）を簡潔に記載する
2. **The** focus.md **shall** ROADMAP.mdの更新タイミング（フェーズ変更時、仕様作成時等）を簡潔に記載する
3. **The** focus.md **shall** 仕様フォルダの配置ルール（specs直下、backlog、completed）を記載する
4. **The** focus.md **shall** コンテキスト消費を最小化するため、詳細情報はROADMAP.mdへの参照とする
5. **The** focus.md **shall** `/kiro-steering-custom focus` で生成可能とする

---

### 3. 仕様フォルダ管理

**Objective:** 開発者/AIとして、仕様の状態に応じた適切なフォルダ配置を維持したい。それによりアクティブな仕様に集中できる。

#### Acceptance Criteria

1. **The** Folder System **shall** アクティブな仕様（現在はP0）を `.kiro/specs/` 直下に配置する
2. **The** Folder System **shall** 当面駆動しない仕様（P1-P3）を `.kiro/specs/backlog/` に配置する
3. **The** Folder System **shall** 完了した仕様を `.kiro/specs/completed/` に移動する
4. **The** Folder System **shall** メタ仕様（ukagaka-desktop-mascot）を常に `.kiro/specs/` 直下に配置する
5. **When** 仕様が完了した時, **the** AI Agent **shall** 該当仕様を completed フォルダに移動する
6. **When** P1仕様がアクティブ化される時, **the** 開発者 **shall** 該当仕様を backlog から specs 直下に移動する

---

### 4. 実行計画と進捗追跡

**Objective:** 開発者/AIとして、依存関係を考慮した実行順序と進捗を把握したい。それにより手戻りのない効率的な開発が可能になる。

#### Acceptance Criteria

1. **The** ROADMAP.md **shall** アクティブ仕様のTier順実行リストを含む
2. **The** ROADMAP.md **shall** 並行実行可能な子仕様を識別可能な形式で記載する
3. **The** ROADMAP.md **shall** 進捗サマリー（完了数/進行中/未着手）を含む
4. **The** ROADMAP.md **shall** 親仕様の各要件に対応する子仕様のマッピングを含む
5. **When** 依存先の子仕様が未完了の場合, **the** ROADMAP.md **shall** その依存関係を明示する

---

## Non-Functional Requirements

### NFR-1: 軽量性

**The** Roadmap Management System **shall** Markdownドキュメントのみで実装し、追加の依存関係（スクリプト等）を必要としない。

### NFR-2: 拡張性

**The** Roadmap Management System **shall** 新しい子仕様の追加や優先度の変更に柔軟に対応できる構造を持つ。

### NFR-3: 透明性

**The** Roadmap Management System **shall** すべての進捗情報をテキストベースで保存し、Git履歴で変更を追跡可能にする。

### NFR-4: AI親和性

**The** Roadmap Management System **shall** ドキュメントをMarkdown形式で記述し、AIエージェントが解析・生成しやすい構造とする。

### NFR-5: 既存ワークフロー互換性

**The** Roadmap Management System **shall** 既存のkiro-styleコマンド（`/kiro-spec-*`, `/kiro-steering-custom`）と統合し、既存ワークフローを活用する。

---

## Appendix

### A. 想定される成果物

| 成果物 | 形式 | 配置場所 | 生成方法 |
|--------|------|----------|----------|
| `ROADMAP.md` | Markdown | `.kiro/specs/{meta-spec}/` | 設計に基づき作成 |
| `focus.md` | Markdown | `.kiro/steering/` | `/kiro-steering-custom focus` |

### B. 仕様フォルダ構成

```
.kiro/specs/
├── ukagaka-desktop-mascot/   # メタ仕様（常にアクティブ）
│   └── ROADMAP.md            # ロードマップ
├── {active-specs}/           # アクティブな子仕様（P0等）
├── backlog/                  # 当面駆動しない仕様（P1-P3）
└── completed/                # 完了仕様
```

### C. focus.md の役割

`focus.md` はsteeringファイルとしてAIエージェントに常に読み込まれる。コンテキスト消費を最小化するため、以下の指示のみを簡潔に記載する：

- **参照タイミング**: いつROADMAP.mdを読むべきか
- **更新タイミング**: いつROADMAP.mdを更新すべきか
- **フォルダ配置ルール**: specs直下/backlog/archiveの使い分け

詳細な進捗情報はROADMAP.mdに集約し、focus.mdからは参照のみとする。

### D. 仕様の完了フロー

本仕様はプロセス支援仕様のため、通常のtasks→implementation フローではなく、以下で完了とする：

1. **要件承認**: この requirements.md を承認
2. **設計作成**: design.md でドキュメント構造を設計
3. **設計承認**: design.md を承認
4. **成果物生成**: 
   - `ROADMAP.md` を親仕様ディレクトリに作成
   - `/kiro-steering-custom focus` を実行
   - P1-P3仕様を backlog フォルダに移動
5. **完了**: spec.json の phase を `completed` に更新

### E. 本仕様の経緯

本仕様は `ukagaka-desktop-mascot` メタ仕様の設計レビュー（Issue #3）において、以下の議論から生まれた：

> **Issue #3: 進捗追跡の仕組み**
> 
> 32件の子仕様を数か月〜年単位で追跡する仕組みが必要。kiro-spec-statusコマンドは単一仕様向けであり、メタ仕様の全体管理には不十分。

この議論を受けて、プロセス支援仕様 `kiro-P0-roadmap-management` が追加された。

### F. 優先度とTierの関係

```
優先度（ビジネス観点）
├── P0: MVP必須（内作試作に必要）
├── P1: リリース必須（外部公開に必要）
├── P2: 差別化（競合優位性）
└── P3: 将来（長期ロードマップ）

Tier（技術的依存関係）
├── Tier 0: 基盤（他に依存しない）
├── Tier 1-2: コア（Tier 0に依存）
├── Tier 3: 参照実装（コアを使用）
└── Tier 4-6: 拡張（オプション機能）
```

実装順序は「優先度で大枠を決定 → Tierで詳細順序を決定」の2段階で決まる。

### G. 関連仕様

- **親仕様**: `ukagaka-desktop-mascot` - 本仕様の親メタ仕様
- **参照**: `AGENTS.md` - kiro-style Spec Driven Development の概要
- **参照**: `.kiro/steering/kiro-spec-schema.md` - spec.json のスキーマ定義

