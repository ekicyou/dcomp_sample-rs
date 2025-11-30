# Research & Design Decisions: wintf-P0-logging-system

## Summary
- **Feature**: `wintf-P0-logging-system`
- **Discovery Scope**: Extension（既存システムへのログシステム追加）
- **Key Findings**:
  - tracingは安定版0.1.43、MSRV 1.65+でwintfの環境と完全互換
  - ライブラリはログ発行のみ、Subscriber初期化はアプリケーション責任が公式推奨パターン
  - tracing-subscriberの`env-filter`機能でRUST_LOG環境変数対応が標準実装可能

## Research Log

### tracing クレートのライブラリ利用パターン
- **Context**: wintfはライブラリであり、Subscriber初期化をどこで行うべきか確認
- **Sources Consulted**: 
  - https://docs.rs/tracing/latest/tracing/#in-libraries
  - https://docs.rs/tracing/latest/tracing/#in-executables
- **Findings**:
  - 公式ドキュメントで「Libraries should link only to the tracing crate, and use the provided macros」と明記
  - 「In general, libraries should NOT call set_global_default()!」と警告あり
  - Subscriberの初期化はexecutablesの責任
- **Implications**: Req 4の設計方針（ライブラリはログ発行のみ）は公式推奨と完全一致

### tracing-subscriber EnvFilter
- **Context**: RUST_LOG環境変数によるフィルタリング実装方法
- **Sources Consulted**: https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html
- **Findings**:
  - `env-filter`フィーチャーで`EnvFilter`が利用可能
  - `RUST_LOG`環境変数を自動的にパース
  - `EnvFilter::from_default_env()`で簡単に初期化可能
  - ターゲット指定（`wintf=debug`）も可能
- **Implications**: taffy_flex_demoでの参考実装は数行で完結

### 構造化ログのフィールド記法
- **Context**: HWND/Entity IDなどのコンテキスト付与方法
- **Sources Consulted**: https://docs.rs/tracing/latest/tracing/#recording-fields
- **Findings**:
  - `?`シグルでDebug実装を使用: `info!(hwnd = ?hwnd, "...")`
  - `%`シグルでDisplay実装を使用
  - フィールド名にドットを含められる: `entity.id = ...`
- **Implications**: Req 3.3の実装は標準的なマクロ記法で対応可能

### パフォーマンス特性
- **Context**: メッセージループ内でのログ呼び出しがGUIに影響しないか確認
- **Sources Consulted**: https://docs.rs/tracing/latest/tracing/#core-concepts (Subscribers section)
- **Findings**:
  - Subscriber未設定時、イベントは構築すらされない（ゼロコスト）
  - `Subscriber::enabled()`がfalseを返すとSpan/Eventは生成されない
  - コンパイル時レベルフィルタ機能あり（`max_level_*` features）
- **Implications**: Req 5.1は標準動作で満たされる

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| workspace.dependencies追加 | tracing/tracing-subscriberをワークスペースレベルで管理 | バージョン統一、既存パターン準拠 | 特になし | **採用** |
| wintfのみ依存追加 | crates/wintf/Cargo.tomlのみ変更 | 影響範囲最小 | 将来的なクレート追加時に再設定必要 | 不採用 |

## Design Decisions

### Decision: ログレベルマッピング規則
- **Context**: 50+箇所の`eprintln!`をどのログレベルに置換するか
- **Alternatives Considered**:
  1. すべて`debug!`に統一 — 簡単だが粒度が粗い
  2. 内容に基づく分類 — 適切だが判断が必要
- **Selected Approach**: 内容に基づく5段階分類
- **Rationale**: 運用時のノイズ削減と問題調査効率のバランス
- **Trade-offs**: 初期実装に判断コストがかかるが、長期的なメンテナンス性向上
- **Follow-up**: 各ファイルごとにマッピングテーブルを設計文書に記載

### Decision: デフォルトログレベル実装方式
- **Context**: Req 2.3-2.4のビルドモード別デフォルトレベル
- **Alternatives Considered**:
  1. wintfライブラリ内で`#[cfg(debug_assertions)]`判定 — 責務混在
  2. アプリケーション側で判定 — 責務分離
  3. tracing-subscriberのEnvFilter設定 — 標準的
- **Selected Approach**: アプリケーション側で`EnvFilter`を設定
- **Rationale**: ライブラリはログ発行のみという方針と整合
- **Trade-offs**: 利用者が明示的に設定する必要があるが、taffy_flex_demoで参考実装を提供
- **Follow-up**: taffy_flex_demoにビルドモード判定付き初期化コードを実装

## Risks & Mitigations
- **Risk 1**: 50+箇所の置換作業でログレベル判断の一貫性が崩れる可能性
  - **Mitigation**: マッピングテーブルを設計文書に明記し、レビュー時に参照
- **Risk 2**: Subscriber未設定時にログが見えずデバッグ困難
  - **Mitigation**: taffy_flex_demoに参考実装を追加し、READMEで案内

## References
- [tracing crate documentation](https://docs.rs/tracing/latest/tracing/) — 公式ドキュメント
- [tracing-subscriber documentation](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/) — Subscriber実装
- [tracing GitHub repository](https://github.com/tokio-rs/tracing) — ソースコード・例
