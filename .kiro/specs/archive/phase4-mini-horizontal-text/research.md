# Research & Design Decisions: phase4-mini-horizontal-text

---
**Purpose**: Capture discovery findings, architectural investigations, and rationale that inform the technical design.

**Usage**:
- Log research activities and outcomes during the discovery phase.
- Document design decision trade-offs that are too detailed for `design.md`.
- Provide references and evidence for future audits or reuse.
---

## Summary
- **Feature**: `phase4-mini-horizontal-text`
- **Discovery Scope**: Extension (既存システムへのテキスト描画機能追加)
- **Key Findings**:
  - DirectWriteファクトリーはGraphicsCoreに既に統合済み（IDWriteFactory2）
  - Rectangleウィジットが成熟した実装パターンを提供
  - 必要なAPI拡張: CreateTextLayout, DrawTextLayout
  - 新規モジュール: `ecs/widget/text/`
  - 統合ポイント明確: `Draw`スケジュールへの追加のみ

## Research Log

### DirectWrite API Integration Status
- **Context**: DirectWriteをwintfフレームワークに統合し、テキストレンダリングを実現する必要がある
- **Sources Consulted**:
  - 既存コード: `com/dwrite.rs`, `ecs/graphics/core.rs`
  - gap-analysis.md
  - Microsoft DirectWrite documentation
- **Findings**:
  - ✅ IDWriteFactory2は既にGraphicsCoreに統合済み
  - ✅ CreateTextFormat APIは実装済み（DWriteFactoryExt trait）
  - ❌ CreateTextLayout APIは未実装
  - ❌ DrawTextLayout APIは未実装
  - IDWriteFactory2はIDWriteFactory7と後方互換性あり
- **Implications**:
  - CreateTextLayout/DrawTextLayout APIの追加のみで実装可能
  - IDWriteFactory7へのアップグレードは不要（横書きテキストに必要な機能は全てIDWriteFactory2で利用可能）

### Existing Widget Pattern Analysis
- **Context**: Labelウィジットの実装パターンを既存コードから学習
- **Sources Consulted**:
  - `ecs/widget/shapes/rectangle.rs`
  - `ecs/graphics/core.rs`
  - gap-analysis.md
- **Findings**:
  - Rectangleウィジットパターン:
    1. Componentトレイト実装
    2. on_remove hook（GraphicsCommandListクリア）
    3. draw_rectanglesシステム（Changed検知 + CommandList生成）
    4. GraphicsCommandListコンポーネント（キャッシング）
  - WindowGraphicsコンポーネントとの依存関係
  - Drawスケジュールでの実行
- **Implications**:
  - Labelウィジットは同パターンを踏襲
  - draw_labelsシステムをDrawスケジュールに追加
  - TextLayoutコンポーネントでキャッシング実装

### Performance Optimization Strategy
- **Context**: 60fpsを維持するためのパフォーマンス最適化
- **Sources Consulted**:
  - `ecs/widget/shapes/rectangle.rs`
  - requirements.md (Requirement 9)
- **Findings**:
  - Changed<T>検知により不要な再描画を回避
  - GraphicsCommandListキャッシングでGPU効率最大化
  - Rectangleで60fps安定動作実績（Vsync同期環境）
- **Implications**:
  - LabelにChanged検知を適用
  - TextLayoutコンポーネントでキャッシング
  - DrawTextLayout呼び出しをCommandListにバッチング

### Vertical Text Extensibility
- **Context**: Phase 7での縦書き対応を前提とした設計
- **Sources Consulted**:
  - requirements.md (Introduction, Non-Functional Requirements)
  - gap-analysis.md (Section 6.1.1)
- **Findings**:
  - DirectWriteのIDWriteTextLayoutは縦書きをネイティブサポート
  - READING_DIRECTION/FLOW_DIRECTIONパラメータで制御可能
  - API命名は方向に依存しない汎用的な名称を使用すべき
- **Implications**:
  - `Label`であり`HorizontalLabel`ではない
  - `draw_labels`であり`draw_horizontal_labels`ではない
  - Phase 7で`writing_mode: WritingMode`フィールド追加を想定
  - デフォルト値は横書きとすることで後方互換性維持

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| Option A: Extend Existing | shapesモジュールにtext.rs追加 | 既存モジュール活用 | 関心の分離違反、メンテナンス困難 | 不適切 |
| Option B: Create New | `ecs/widget/text/`モジュール新規作成 | 明確な責務分離、拡張性 | COM API拡張が含まれない | 部分的に適切 |
| Option C: Hybrid | COM拡張 + 新規モジュール | レイヤー分離維持、一貫性 | 複数ファイル調整必要 | 推奨（gap-analysis.md Section 3推奨） |

**Selected**: Option C - 既存のレイヤー構造を維持しつつ、新機能を独立モジュールとして実装

## Design Decisions

### Decision: IDWriteFactory2継続使用
- **Context**: 要件ではIDWriteFactory7を指定しているが、既存実装はIDWriteFactory2を使用
- **Alternatives Considered**:
  1. IDWriteFactory7へアップグレード
  2. IDWriteFactory2を継続使用
- **Selected Approach**: IDWriteFactory2を継続使用
- **Rationale**:
  - 横書きテキストに必要な機能は全てIDWriteFactory2で利用可能
  - IDWriteFactory7は後方互換性あり（将来的なアップグレードパス確保）
  - 既存コードへの影響を最小化
- **Trade-offs**:
  - ✅ 既存システムとの整合性維持
  - ✅ 実装コスト削減
  - ⚠️ 最新API機能は利用不可（現時点で不要）
- **Follow-up**: 将来的に高度な機能が必要になった場合、IDWriteFactory7へのキャストを実装

### Decision: TextLayoutキャッシング戦略
- **Context**: IDWriteTextLayoutの生成は比較的高コスト、パフォーマンス最適化が必要
- **Alternatives Considered**:
  1. 毎フレーム再生成
  2. Componentとして永続化
  3. グローバルキャッシュ
- **Selected Approach**: Componentとして永続化（TextLayoutコンポーネント）
- **Rationale**:
  - Changed<Label>検知で再生成タイミング制御
  - Rectangleパターンと一貫性
  - エンティティライフサイクルと連動した自動クリーンアップ
- **Trade-offs**:
  - ✅ パフォーマンス最適化
  - ✅ 既存パターンとの一貫性
  - ✅ メモリ管理の単純化
  - ❌ メモリ使用量増加（ただし許容範囲）
- **Follow-up**: on_remove hookでTextLayoutの適切な解放を実装

### Decision: Labelコンポーネントフィールド設計
- **Context**: 横書きテキスト実装だが、Phase 7での縦書き対応を前提とした設計が必要
- **Alternatives Considered**:
  1. 横書き専用フィールド（HorizontalLabel）
  2. 最小フィールドセット（汎用Label）
  3. 縦書きフィールドを先行実装
- **Selected Approach**: 最小フィールドセット（汎用Label）
- **Rationale**:
  - 複雑さを避け、Phase 7での拡張に備える
  - API命名は方向に依存しない汎用的な名称
  - 既存のRectangle（5フィールド）と同レベルの複雑さ
- **Trade-offs**:
  - ✅ 拡張性確保（Phase 7で`writing_mode`追加容易）
  - ✅ シンプルな実装
  - ✅ 後方互換性維持（デフォルト値で横書き）
  - ❌ 縦書き機能は後回し（意図的な設計判断）
- **Follow-up**: Phase 7で`writing_mode: WritingMode`フィールド追加、デフォルト値は`WritingMode::Horizontal`

### Decision: draw_labelsシステム実装順序
- **Context**: COM API拡張、コンポーネント定義、システム実装、統合の依存関係
- **Alternatives Considered**:
  1. トップダウン（システムから）
  2. ボトムアップ（COM APIから）
  3. 並行実装
- **Selected Approach**: ボトムアップ（COM API → コンポーネント → システム → 統合）
- **Rationale**:
  - 各レイヤーを独立してテスト可能
  - 依存関係に沿った実装順序
  - gap-analysis.md Section 6.4の推奨順序に準拠
- **Trade-offs**:
  - ✅ 段階的なテストとデバッグ
  - ✅ 明確な依存関係
  - ❌ エンドツーエンド動作確認が後半
- **Follow-up**: 各フェーズでユニットテスト実装

## Risks & Mitigations

- **Risk 1**: IDWriteFactory2とIDWriteFactory7の非互換性
  - **Mitigation**: IDWriteFactory7は後方互換性あり、必要に応じてcast()で対応可能
  
- **Risk 2**: TextLayoutキャッシングによるメモリ使用量増加
  - **Mitigation**: 10 Labelで問題なし、on_remove hookで適切に解放
  
- **Risk 3**: Phase 7での縦書き対応時の破壊的変更
  - **Mitigation**: API命名とコンポーネント構造を汎用的に設計、`writing_mode`フィールド追加のみで対応可能
  
- **Risk 4**: DrawTextLayout API呼び出しのパフォーマンス
  - **Mitigation**: CommandListバッチング、Changed検知による最適化、Vsync同期で60fps安定

## References

- [Microsoft DirectWrite Documentation](https://learn.microsoft.com/en-us/windows/win32/directwrite/direct-write-portal) — DirectWrite API reference
- [IDWriteFactory2 Interface](https://learn.microsoft.com/en-us/windows/win32/api/dwrite_2/nn-dwrite_2-idwritefactory2) — Factory interface used in current implementation
- [IDWriteTextLayout Interface](https://learn.microsoft.com/en-us/windows/win32/api/dwrite/nn-dwrite-idwritetextlayout) — Text layout interface to be implemented
- [ID2D1DeviceContext::DrawTextLayout](https://learn.microsoft.com/en-us/windows/win32/api/d2d1_1/nf-d2d1_1-id2d1devicecontext-drawtextlayout) — Drawing method to be wrapped
- gap-analysis.md — Comprehensive existing codebase analysis and implementation approach evaluation
- requirements.md — 11 requirements with 78 acceptance criteria
