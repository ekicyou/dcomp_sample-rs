# Design Document: deps-update

## Overview

**Purpose**: wintf ワークスペースの全依存パッケージを最新安定バージョンに更新し、破壊的変更に対応するコード修正を実施する。セキュリティ修正・バグ修正・最新機能の恩恵を受けるとともに、技術的負債の蓄積を防止する。

**Users**: wintf の開発者が、最新の bevy_ecs 0.18・ambassador 0.5・rand 0.10 等のAPIを利用して開発を継続できる。

**Impact**: `Cargo.toml`（ワークスペースルート＋クレート固有）のバージョン指定変更、および bevy 0.18 / rand 0.10 のAPI変更に伴うソースコード修正。アーキテクチャ変更は伴わない。

### Goals
- ワークスペース全依存パッケージを最新安定バージョンに更新する
- 破壊的変更に対応するコード修正を実施し、ビルド・テスト・サンプルを全て通す
- ステアリングドキュメント（`tech.md`）のバージョン情報を整合させる

### Non-Goals
- 新機能の追加や既存APIの変更（依存更新に必要な最小限の修正のみ）
- bevy 0.18 の新機能（Immutable Components 等）の積極的採用
- submodules/pasta の依存更新（別リポジトリで管理）

## Architecture

### Existing Architecture Analysis

本プロジェクトは bevy の ECS サブシステム（bevy_ecs, bevy_app, bevy_tasks, bevy_utils）をスタンドアロンで使用しており、bevy のレンダリング・UI・アセット等のサブシステムは一切使用していない。この特性により、bevy 0.18 の破壊的変更の大部分（30+項目中25+項目）は本プロジェクトに影響しない。

**維持すべきパターン**:
- bevy_ecs のスタンドアロンECS使用（`bevy_ecs::prelude::*` 経由）
- ambassador によるトレイト委譲パターン（`delegatable_trait`, `Delegate`）
- Rust 2024 Edition のモジュール解決規則

**既存の技術的制約**:
- bevy_ecs の内部モジュールパスを一部直接参照（`lifecycle::HookContext`, `component::Mutable` 等）
- テストコードで `IntoSystem::into_system()` を明示的に使用

### Architecture Pattern & Boundary Map

アーキテクチャパターンの変更なし。既存のECSベースアーキテクチャを維持したまま、依存パッケージのバージョンのみを更新する。

```mermaid
graph LR
    subgraph "変更対象"
        CT["Cargo.toml<br/>(ワークスペース + クレート)"]
        SRC["src/*.rs<br/>(bevy API修正)"]
        TEST["tests/*.rs<br/>(IntoSystem修正)"]
        EX["examples/*.rs<br/>(rand API修正)"]
        TECH["tech.md<br/>(バージョン情報)"]
    end

    subgraph "更新パッケージ"
        BEVY["bevy_ecs 0.18<br/>bevy_app 0.18<br/>bevy_tasks 0.18<br/>bevy_utils 0.18"]
        AMB["ambassador 0.5"]
        RAND["rand 0.10"]
        SAFE["taffy 0.9.2<br/>human-panic 2.0.6<br/>async-io 2.6"]
    end

    CT --> BEVY
    CT --> AMB
    CT --> RAND
    CT --> SAFE
    BEVY --> SRC
    BEVY --> TEST
    RAND --> EX
    CT --> TECH
```

### Technology Stack

| Layer | Choice / Version | Role in Feature | Notes |
|-------|------------------|-----------------|-------|
| ECS | bevy_ecs 0.17.2 → **0.18.0** | コアECSフレームワーク | 破壊的変更あり（影響限定的） |
| App Framework | bevy_app 0.17.2 → **0.18.0** | アプリケーションランナー | bevy_ecs に追従 |
| Task Runtime | bevy_tasks 0.17.2 → **0.18.0** | 非同期タスク実行 | bevy_ecs に追従 |
| Utilities | bevy_utils 0.17.2 → **0.18.0** | ユーティリティ集 | bevy_ecs に追従 |
| Trait Delegation | ambassador 0.4.2 → **0.5.0** | トレイト委譲マクロ | API互換の見込み |
| Layout | taffy 0.9.1 → **0.9.2** | Flexboxレイアウト | パッチ更新 |
| Error Handling | human-panic 2.0.3 → **2.0.6** | パニックハンドラ | パッチ更新 |
| Async I/O | async-io 2.3 → **2.6** | 非同期I/O | マイナー更新 |
| Random (dev) | rand 0.9.2 → **0.10.0** | サンプルコード用乱数 | 破壊的変更あり |

## Requirements Traceability

| Requirement | Summary | Components | Interfaces | Flows |
|-------------|---------|------------|------------|-------|
| R1 (1.1-1.3) | ワークスペース依存の最新化 | Phase 1: Cargo.toml 更新 | `[workspace.dependencies]`, crate-level `Cargo.toml` | Phase 1 → Phase 2 → Phase 3 |
| R2 (2.1-2.3) | ビルド成功の保証 | Phase 2: bevy API修正, Phase 3: ambassador/rand修正 | src/, tests/, examples/ のRustコード | コンパイル駆動修正ループ |
| R3 (3.1-3.3) | テスト・サンプル通過の保証 | Phase 2-3: テスト・サンプル修正 | tests/*.rs, examples/*.rs | 各Phase末のテスト検証 |
| R4 (4.1) | ステアリング情報の整合性維持 | Phase 4: tech.md更新 | `.kiro/steering/tech.md` | 最終Phase |

## Components and Interfaces

| Component | Domain/Layer | Intent | Req Coverage | Key Dependencies | Contracts |
|-----------|--------------|--------|--------------|------------------|-----------|
| Phase 1: Cargo.toml 更新 | Build Config | 全パッケージバージョンを最新に変更 | R1 | なし | Cargo.toml |
| Phase 2: bevy 0.18 コード修正 | ECS/Core | bevy API変更に対応するコード修正 | R2, R3 | Phase 1 (P0) | src/, tests/ |
| Phase 3: ambassador/rand 修正 | Macro/Dev | ambassador 0.5, rand 0.10 対応コード修正 | R2, R3 | Phase 1 (P0) | src/, examples/ |
| Phase 4: ドキュメント更新 | Docs | ステアリング tech.md のバージョン同期 | R4 | Phase 1-3 (P0) | tech.md |

### Phase 1: Cargo.toml 更新（Build Config）

| Field | Detail |
|-------|--------|
| Intent | 全依存パッケージのバージョン指定を最新安定バージョンに変更 |
| Requirements | R1 (1.1, 1.2, 1.3) |

**Responsibilities & Constraints**
- ワークスペースルート `Cargo.toml` の `[workspace.dependencies]` セクションを更新
- クレート固有 `Cargo.toml` のバージョン指定を更新
- bevy 系クレート（bevy_ecs, bevy_app, bevy_tasks, bevy_utils）は同一バージョンで統一
- `already_latest` パッケージ（windows, windows-core, tracing 等）は変更不要

**更新対象マトリクス**

| パッケージ | 現行 | 更新先 | 分類 | 更新箇所 |
|---|---|---|---|---|
| bevy_ecs | 0.17.2 | 0.18.0 | 破壊的 | workspace deps |
| bevy_app | 0.17.2 | 0.18.0 | 破壊的 | workspace deps |
| bevy_tasks | 0.17.2 | 0.18.0 | 破壊的 | workspace deps |
| bevy_utils | 0.17.2 | 0.18.0 | 破壊的 | workspace deps |
| ambassador | 0.4.2 | 0.5.0 | 破壊的（互換見込み） | workspace deps |
| taffy | 0.9.1 | 0.9.2 | パッチ | workspace deps |
| human-panic | 2.0.3 | 2.0.6 | パッチ | workspace deps |
| async-io | 2.3 | 2.6 | マイナー | crate deps |
| rand | 0.9.2 | 0.10.0 | 破壊的 | crate deps (dev) |

### Phase 2: bevy 0.18 コード修正（ECS/Core）

| Field | Detail |
|-------|--------|
| Intent | bevy 0.18 のAPI変更に対応するソースコード修正 |
| Requirements | R2 (2.1, 2.2, 2.3), R3 (3.1, 3.3) |

**修正戦略**: コンパイル駆動修正（Cargo.toml 更新後に `cargo build` → エラーに従い修正を反復）

**想定される修正カテゴリ**

| カテゴリ | 影響範囲 | 修正内容 | 確度 |
|---|---|---|---|
| Import パス変更 | src/ 全般 | `use bevy_ecs::xxx` のパス修正 | 中（コンパイラが指摘） |
| `DetectChangesMut` | 3 src + 3 test ファイル | インポートパスの調整（必要な場合） | 低（パス変更なしの可能性） |
| `HookContext` | 7 src ファイル | `lifecycle::HookContext` パス確認 | 低（パス変更なしの可能性） |
| `IntoSystem` ジェネリクス | 3 test ファイル、14箇所 | 新ジェネリックパラメータ `In` への対応 | 中（テストのみ） |
| `Message`/`Messages` | 3 src ファイル | メッセージAPI パス確認 | 低（0.17で導入、安定の見込み） |
| `Mutable` | 1 ファイル | `component::Mutable` パス確認 | 低 |
| `lifetimeless` | 1 ファイル | `system::lifetimeless::*` パス確認 | 低 |
| `ExecutorKind` | 1 ファイル (world.rs) | enum variant の名称変更可能性 | 低 |

**影響なし確認済み（修正不要）**:
- `Entity::row()` / `EntityRow` — 未使用
- `Tick` 型インポート — 未使用
- `clear_children` — 未使用
- `SimpleExecutor` — 未使用
- `Bundle` derive — カスタム impl なし
- `Event` / `EntityEvent` derive — 未使用
- States API — 未使用

### Phase 3: ambassador/rand コード修正

| Field | Detail |
|-------|--------|
| Intent | ambassador 0.5 と rand 0.10 のAPI変更に対応するコード修正 |
| Requirements | R2 (2.3), R3 (3.2, 3.3) |

#### ambassador 0.5

**影響ファイル**: 5箇所の `#[delegatable_trait]` + 1箇所の `#[derive(Delegate)]` + 4ファイルの `use ambassador::*`

**修正見込み**: API互換のため修正不要と推定。コンパイルで確認し、エラーがあれば対応。

#### rand 0.10

**影響ファイル**: `examples/dcomp_demo.rs`（1ファイルのみ）

**修正対象の使用パターン**:

| 現行コード | 0.10 での状態 | 修正要否 |
|---|---|---|
| `use rand::{seq::*, *}` | glob import で `RngExt` が入る | 要確認 |
| `rand::rng()` | 維持（0.9で導入） | 不要 |
| `rng.random_range(b'A'..=b'Z')` | `RngExt` のメソッド | glob import で解決の見込み |
| `values.shuffle(&mut rng)` | `SliceRandom` のメソッド | `seq::*` で解決 |

**修正方針**: glob import（`use rand::*`）で `RngExt` が自動インポートされるため、そのままコンパイルが通る可能性が高い。通らない場合は `use rand::RngExt;` を明示追加。

### Phase 4: ドキュメント更新

| Field | Detail |
|-------|--------|
| Intent | ステアリングドキュメントのバージョン情報を実態に合わせる |
| Requirements | R4 (4.1) |

**対象**: `.kiro/steering/tech.md` の `Key Libraries` セクション

**更新内容**: Phase 1 で更新した全パッケージのバージョン番号を反映

## Error Handling

### Error Strategy

本フィーチャーはビルド時エラー（コンパイルエラー）への対処が中心。ランタイムエラー処理の変更は伴わない。

**コンパイル駆動修正フロー**:
1. Cargo.toml のバージョンを更新
2. `cargo build` を実行
3. コンパイルエラーを分析・修正
4. エラーがなくなるまで 2-3 を反復
5. `cargo test` で回帰確認
6. `cargo build --examples` でサンプル確認

**ロールバック戦略**: 各Phase完了時に Git commit することで、問題発生時に特定Phaseまで戻れる状態を維持。

## Testing Strategy

### Build Verification（R2 対応）
- `cargo build` — debug ビルド成功を確認
- `cargo build --release` — release ビルド成功を確認
- `cargo build --examples` — 全サンプルのビルド成功を確認

### Unit / Integration Tests（R3 対応）
- `cargo test` — 既存テスト全件パスを確認
- テストコード自体のAPI修正（`IntoSystem` ジェネリクス等）が必要な場合は修正を実施

### Sample Verification（R3 対応）
- `cargo build --examples` で全サンプルがビルドできることを確認
- `dcomp_demo` サンプル（rand 0.10 影響対象）の動作確認

### Regression Checklist
- [ ] `cargo build` 成功
- [ ] `cargo build --release` 成功
- [ ] `cargo test` 全件パス
- [ ] `cargo build --examples` 成功
- [ ] `cargo clippy` — 新規警告なし（ベストエフォート）

## Migration Strategy

```mermaid
flowchart TD
    P1["Phase 1: Cargo.toml バージョン更新"] --> C1{"cargo build?"}
    C1 -- "エラー" --> P2["Phase 2: bevy 0.18 コード修正"]
    C1 -- "成功" --> P3
    P2 --> C2{"cargo build?"}
    C2 -- "エラー" --> P2
    C2 -- "成功" --> P3["Phase 3: ambassador/rand 修正"]
    P3 --> C3{"cargo build + test?"}
    C3 -- "エラー" --> P3
    C3 -- "成功" --> P4["Phase 4: tech.md 更新"]
    P4 --> V["最終検証: build + test + examples"]
    V --> DONE["完了"]
```

**Phase 1**: Cargo.toml の全バージョン指定を一括更新（5分）
**Phase 2**: bevy 0.18 のコンパイルエラーを反復修正（1-3時間 ※影響限定的と判明）
**Phase 3**: ambassador 0.5 / rand 0.10 のコンパイルエラーを修正（30分）
**Phase 4**: tech.md のバージョン情報を更新（5分）

**Rollback Triggers**: 各Phase で修正不可能なAPIの根本的非互換が発見された場合、該当パッケージのみ前バージョンに戻す。ただし、research.md の調査結果から全パッケージ更新可能と判断。
