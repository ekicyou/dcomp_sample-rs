# Research & Design Decisions

---
**Feature**: ukagaka-desktop-mascot
**Discovery Scope**: Complex Integration（新規アプリケーション × 既存フレームワーク拡張）
**Key Findings**:
1. wintfはDirectComposition/D2D/bevy_ecs基盤として十分な成熟度を持つ
2. MCPプロトコルをプラットフォーム↔ゴースト間通信に採用することで、LLM連携が自然に実現可能
3. パッケージアーキテクチャ（頭脳/シェル/バルーン分離）は責務分離の観点で必須
---

## Research Log

### wintf既存アーキテクチャ分析

- **Context**: 基盤フレームワークとして採用するwintfの現状能力を把握
- **Sources Consulted**: 
  - `crates/wintf/src/` ソースコード
  - `doc/spec/` 設計ドキュメント群
  - `.kiro/steering/` プロジェクト方針
- **Findings**:
  - **ECSアーキテクチャ**: bevy_ecs 0.17.2採用、Component/System分離が確立
  - **レイアウト**: taffy 0.9.1統合済み、Flexbox対応、DPI aware
  - **描画**: DirectComposition + Direct2D、透過ウィンドウ対応
  - **テキスト**: DirectWrite統合、縦書き対応（Label widget）
  - **ウィンドウ**: Win32 API、マルチモニター対応、メッセージループ統合
  - **現状のWidget**: Rectangle（色塗り）、Label（テキスト）のみ
- **Implications**:
  - 画像表示（Image widget）は未実装 → MVP必須機能
  - イベントシステムは設計ドキュメントのみ、実装は部分的
  - ヒットテストはドキュメント化されているが実装状況要確認

### MCPプロトコル調査

- **Context**: プラットフォーム↔ゴースト間通信の標準プロトコル選定
- **Sources Consulted**: 
  - https://modelcontextprotocol.io/
  - MCP仕様書
- **Findings**:
  - MCPは「AIアプリケーションと外部システムを接続する標準プロトコル」
  - サーバー/クライアントモデル、ツール呼び出し、リソースアクセスを標準化
  - JSONベースのRPC、stdioまたはHTTP/SSE転送
  - Rust実装（rmcp等）が存在
- **Implications**:
  - プラットフォーム = MCPサーバー（描画、イベント、ゴースト間通信を提供）
  - ゴースト（頭脳） = MCPクライアント（LLM、人格、記憶を管理）
  - 既存のLLMエコシステム（Claude、ChatGPT等）との連携が容易

### SHIORIプロトコル調査

- **Context**: 伺か互換性のためのレガシープロトコル理解
- **Sources Consulted**: 
  - ukadoc（一部アクセス不可）
  - 既存の伺か実装知識
- **Findings**:
  - SHIORIはシェル↔辞書（ゴースト頭脳）間の通信プロトコル
  - GET/NOTIFY/LOADなどのリクエストタイプ
  - DLL形式が主流（32bit）、互換性問題あり
- **Implications**:
  - 完全互換は労力に見合わない（requirements.mdで明示済み）
  - SHIORIのイベント体系は参考になる
  - MCP上でSHIORIライクなイベントマッピングを提供可能

### taffyレイアウトエンジン

- **Context**: wintfで採用済みのレイアウトエンジン能力確認
- **Sources Consulted**: 
  - https://docs.rs/taffy/latest/taffy/
  - wintf内のtaffy統合コード
- **Findings**:
  - Flexbox、Grid、Block layoutをサポート
  - High-level API（TaffyTree）とLow-level APIの両方を提供
  - measure functionによるカスタムサイズ計算（テキスト、画像等）
  - wintfはLow-level APIを使用している模様
- **Implications**:
  - 現代的レイアウトシステムとして十分な能力
  - 旧伺かの絶対座標ベースからの変換が必要

### スクリプトエンジン選定

- **Context**: ゴースト対話スクリプトの記述言語
- **Sources Consulted**: 
  - 里々（Satori）の設計思想
  - Luaエンジン（mlua等）
  - tree-sitter parser
- **Findings**:
  - 里々は「会話を自然に書ける」構文が支持された理由
  - Luaは軽量で組み込み実績豊富
  - カスタムDSLはパーサー開発コストが高い
- **Implications**:
  - MVP: 里々インスパイアのカスタムDSL（簡易版）
  - 拡張: Luaバインディング
  - LLM連携時はスクリプトとLLM応答のハイブリッド

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| **ECS + MCP** | bevy_ecs基盤 + MCPプロトコル | wintf既存資産活用、LLM連携容易、責務分離明確 | MCPの学習コスト | **採用** |
| Actorモデル | ゴーストごとにActor | 並行性、分離性 | 複雑性増大、wintfとの統合困難 | 不採用 |
| モノリシック | 全機能を単一プロセス | シンプル | スケーラビリティ、プラグイン困難 | 不採用 |

### 選定理由（ECS + MCP）

1. **wintfとの一貫性**: 既存のbevy_ecsアーキテクチャを継承
2. **責務分離**: プラットフォーム（MCPサーバー）とゴースト（MCPクライアント）の明確な境界
3. **拡張性**: MCPツールとしてプラグイン機能を自然に実装可能
4. **LLM統合**: MCPは元々LLM連携のためのプロトコル

## Design Decisions

### Decision: プラットフォーム↔ゴースト通信にMCPを採用

- **Context**: ゴースト（頭脳）とプラットフォーム間の通信プロトコルが必要
- **Alternatives Considered**:
  1. SHIORI互換プロトコル — 32bit DLL問題、レガシー負債
  2. カスタムRPCプロトコル — 車輪の再発明
  3. MCP — 標準化済み、LLM連携との親和性
- **Selected Approach**: MCP
- **Rationale**: 
  - 2025年の技術としてLLM連携は必須
  - MCPはLLMとツール連携のために設計されたプロトコル
  - 標準化により、サードパーティゴーストの開発が容易
- **Trade-offs**: 
  - 学習コスト（新しいプロトコル）
  - SHIORI完全互換は断念
- **Follow-up**: MCP Rust実装（rmcp）の評価

### Decision: パッケージ分離（頭脳/シェル/バルーン）

- **Context**: ゴースト資産の配布・再利用性
- **Alternatives Considered**:
  1. 一体型パッケージ — シンプルだが再利用困難
  2. 分離型パッケージ — 複雑だが柔軟
- **Selected Approach**: 分離型（頭脳/シェル/バルーン独立）
- **Rationale**:
  - requirements.md Requirement 27で明示された要件
  - コミュニティ創作活動の促進
  - 「着せ替え」「バルーン交換」は伺かエコシステムの重要機能
- **Trade-offs**:
  - 依存関係管理の複雑さ
  - マニフェスト仕様の設計が必要
- **Follow-up**: manifest.toml仕様の詳細設計

### Decision: レンダリングパイプラインの責務

- **Context**: 描画の責務をどこに置くか
- **Alternatives Considered**:
  1. シェルが描画コマンドを生成 — 柔軟だがセキュリティリスク
  2. プラットフォームが描画を完全制御 — 安全だが表現力制限
- **Selected Approach**: プラットフォーム主導、シェルは素材提供のみ
- **Rationale**:
  - セキュリティ（シェルに任意コード実行を許さない）
  - 責務境界表（requirements.md）で明示
  - DirectComposition APIの直接操作はプラットフォーム責務
- **Trade-offs**:
  - カスタム描画エフェクトの制限（プラグインで対応）
- **Follow-up**: シェルフォーマット（サーフェス定義、アニメーション定義）の詳細設計

### Decision: スクリプトエンジンの段階的実装

- **Context**: ゴースト対話スクリプトの実行環境
- **Alternatives Considered**:
  1. 里々完全互換 — 実装コスト大、レガシー負債
  2. Lua/Wasm汎用スクリプト — 柔軟だが学習コスト
  3. カスタムDSL — 「会話を自然に書ける」に最適化
- **Selected Approach**: 段階的実装
  - Phase 1: カスタムDSL（里々インスパイア簡易版）
  - Phase 2: Luaバインディング
  - Phase 3: Wasmプラグイン
- **Rationale**:
  - MVPでは「会話を書ける」最小限のDSLで十分
  - 拡張性はLua/Wasmで担保
- **Trade-offs**:
  - 複数のスクリプト環境サポートの複雑さ
- **Follow-up**: カスタムDSL文法の設計

## Risks & Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| MCPの成熟度不足 | 中 | 低 | 標準仕様に準拠、必要に応じてフォールバック |
| パフォーマンス問題（描画） | 高 | 中 | DirectComposition活用、プロファイリング |
| スクリプトエンジンの複雑さ | 中 | 高 | MVPは最小限DSL、段階的拡張 |
| プラグインセキュリティ | 高 | 中 | サンドボックス設計、明示的権限要求 |
| wintfの機能不足 | 中 | 中 | MVP必須機能を優先実装（Image、イベント） |

## References

- [wintf設計ドキュメント](../../../doc/spec/README.md) — ECS/Visual/Layout設計
- [MCP公式サイト](https://modelcontextprotocol.io/) — プロトコル仕様
- [taffy](https://docs.rs/taffy/latest/taffy/) — レイアウトエンジン
- [bevy_ecs](https://docs.rs/bevy_ecs/) — ECSフレームワーク
- [DirectComposition](https://learn.microsoft.com/en-us/windows/win32/directcomp/directcomposition-portal) — Windows描画API
