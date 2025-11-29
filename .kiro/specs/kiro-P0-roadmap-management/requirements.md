# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | メタ仕様ロードマップ管理システム 要件定義書 |
| **Version** | 1.2 |
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
2. **The** Roadmap System **shall** 全子仕様の一覧と優先度（P0/P1/P2/P3）を単一ドキュメントで表示する
3. **The** Roadmap System **shall** 各子仕様の現在のフェーズ（requirements-draft / design-approved / implementing / completed 等）を表示する
4. **The** Roadmap System **shall** 現在の開発フォーカス（今取り組むべき子仕様）を明示する
5. **When** 子仕様のフェーズが変更された時, **the** Roadmap System **shall** ドキュメント更新手段を提供する
6. **The** Roadmap System **shall** Tier依存関係（技術的な実装順序）を視覚的に表現する

---

### 2. Steering ドキュメント拡張

**Objective:** AIエージェントとして、セッション開始時にプロジェクトの現状を即座に把握したい。それにより継続的な開発支援が可能になる。

#### Acceptance Criteria

1. **The** Steering Extension **shall** `/kiro-steering-custom` コマンドで生成可能なテンプレートを提供する
2. **The** Focus Document **shall** 現在取り組み中の子仕様名を明記する
3. **The** Focus Document **shall** 直近の目標（次のマイルストーン）を記載する
4. **Where** ブロッカー（未解決の問題・依存関係）が存在する場合, **the** Focus Document **shall** それを記載する
5. **When** AIエージェントが新しいセッションを開始した時, **the** Agent **shall** `.kiro/steering/` を読み込んでコンテキストを構築する
6. **The** Focus Document **shall** 人間による手動更新とAIによる更新提案の両方をサポートする

---

### 3. AIエージェント連携ガイドライン

**Objective:** AIエージェントとして、メタ仕様を効果的に駆動するためのベストプラクティスを知りたい。それにより一貫性のある開発支援が可能になる。

#### Acceptance Criteria

1. **The** Guidelines **shall** `/kiro-steering-custom` コマンドで生成可能なテンプレートを提供する
2. **The** Guidelines **shall** メタ仕様の構造（親仕様 → 子仕様の関係）を説明する
3. **The** Guidelines **shall** 優先度（P0-P3）とTier（0-6）の使い分けを説明する
4. **The** Guidelines **shall** セッション開始時のコンテキスト構築手順を定義する
5. **The** Guidelines **shall** 子仕様の作成・更新手順を定義する
6. **The** Guidelines **shall** ロードマップドキュメントの更新タイミングを定義する
7. **Where** 複数のAIセッションで作業が継続される場合, **the** Guidelines **shall** 引き継ぎ方法を定義する

---

### 4. 実行計画と進捗追跡

**Objective:** 開発者/AIとして、依存関係を考慮した実行順序と進捗を把握したい。それにより手戻りのない効率的な開発が可能になる。

#### Acceptance Criteria

1. **The** ROADMAP.md **shall** P0子仕様のTier順実行リストを含む
2. **The** ROADMAP.md **shall** 並行実行可能な子仕様を識別可能な形式で記載する
3. **The** ROADMAP.md **shall** 優先度別（P0/P1/P2/P3）の進捗サマリーを含む
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
| `ROADMAP.md` | Markdown | `.kiro/specs/{meta-spec}/` | 設計に基づき手動作成 |
| `focus.md` | Markdown | `.kiro/steering/` | `/kiro-steering-custom focus` |
| `ai-guidelines.md` | Markdown | `.kiro/steering/` | `/kiro-steering-custom ai-guidelines` |

### B. 配置場所の設計方針

**親仕様集約の原則**: ロードマップ情報は親仕様（メタ仕様）ディレクトリに集約する。

- **`ROADMAP.md`**: 人間が確認する進捗・計画情報 → 親仕様ディレクトリ
- **`focus.md`**: AIがセッション開始時に読む現在フォーカス → steering（`/kiro-steering-custom`で生成）
- **`ai-guidelines.md`**: AIエージェント向けガイドライン → steering（`/kiro-steering-custom`で生成）
- **`spec.json`**: 機械可読なメタデータ → 各仕様ディレクトリ

この分離により、人間向け情報とAI向け情報を明確に区別する。

### C. 仕様の完了フロー

本仕様はプロセス支援仕様のため、通常のtasks→implementation フローではなく、以下で完了とする：

1. **要件承認**: この requirements.md を承認
2. **設計作成**: design.md でドキュメント構造とテンプレートを設計
3. **設計承認**: design.md を承認
4. **成果物生成**: 設計で定義されたコマンドを実行
   - `ROADMAP.md` を親仕様ディレクトリに作成
   - `/kiro-steering-custom focus` を実行
   - `/kiro-steering-custom ai-guidelines` を実行
5. **完了**: spec.json の phase を `completed` に更新

### D. 本仕様の経緯

本仕様は `ukagaka-desktop-mascot` メタ仕様の設計レビュー（Issue #3）において、以下の議論から生まれた：

> **Issue #3: 進捗追跡の仕組み**
> 
> 32件の子仕様を数か月〜年単位で追跡する仕組みが必要。kiro-spec-statusコマンドは単一仕様向けであり、メタ仕様の全体管理には不十分。

この議論を受けて、プロセス支援仕様 `kiro-P0-roadmap-management` が追加された。

### E. 優先度とTierの関係

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

### F. 関連仕様

- **親仕様**: `ukagaka-desktop-mascot` - 本仕様の親メタ仕様
- **参照**: `AGENTS.md` - kiro-style Spec Driven Development の概要
- **参照**: `.kiro/steering/kiro-spec-schema.md` - spec.json のスキーマ定義

