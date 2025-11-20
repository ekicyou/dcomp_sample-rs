# Research & Design Decisions: Visual Auto Component Refactor

---
**Purpose**: Capture discovery findings, architectural investigations, and rationale that inform the technical design.
---

## Summary
- **Feature**: `visual-auto-component-refactor`
- **Discovery Scope**: Extension / Refactor（拡張 / リファクタリング）
- **Key Findings**:
  - 既存の`VisualGraphics`と`SurfaceGraphics`は手動ラッパーである。
  - `GraphicsCore`はBevy Resourceとして利用可能である。
  - 要件により、リソース生成における「Systems」と「Hooks」の比較が求められている。
  - ツリー同期はスコープ外である。

## Research Log

### GPU Resource Generation Strategy
- **Context**: 要件の調査項目1にて、HooksとSystemsの比較が求められている。
- **Sources Consulted**: Bevy ECSドキュメント、既存の`systems.rs`。
- **Findings**:
  - **Hooks (`on_add`)**:
    - `HookContext`経由で`World`にアクセス可能。
    - コンポーネント追加時に即座にリソース生成が可能。
    - **Risk**: Hook内でのCOMオブジェクト生成は、スレッドセーフ性の問題や、重い処理によるメインスレッドのブロックのリスクがある。
  - **Systems (`Query<Entity, Added<Visual>>`)**:
    - 標準的なBevyパターン。
    - `Update`または`PostUpdate`スケジュールで実行される。
    - 並列化が容易（`GraphicsCore`が許容すれば）。
    - 1フレームの遅延が発生するが許容範囲内。
- **Implications**: 設計は両方の戦略の切り替え、またはテストをサポートする必要がある。機能フラグや設定で切り替えるか、実験のために別々のモジュールとして実装する。

### Existing Code Analysis
- **Context**: `VisualGraphics`の現在の使用状況の把握。
- **Findings**:
  - `systems.rs`内の`create_visual_for_target`と`create_surface_for_window`が現在の手動生成ポイントである。
  - `VisualGraphics`は`IDCompositionVisual3`をラップしている。
  - `SurfaceGraphics`は`IDCompositionSurface`をラップしている。
- **Implications**: これらのヘルパー関数は、新しい自動化ロジックに置き換えるためにリファクタリングまたは非推奨にする必要がある。

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| **Systems Approach** | `Update`スケジュールでの`Query<Added<Visual>>` | 標準的なBevy、予測可能な実行順序、容易なエラーハンドリング | 1フレームの遅延 | 安定性のため推奨 |
| **Hooks Approach** | `#[component(on_add = ...)]` | 即時の一貫性 | COMのスレッド問題の可能性、デバッグが困難 | 比較のために実装する |

## Design Decisions

### Decision: Dual Implementation for Resource Generation
- **Context**: ユーザーよりSystemsとHooksの両方を実装して比較するよう要望があった。
- **Selected Approach**: `systems_impl`と`hooks_impl`の2つの分離されたモジュールを実装する。
- **Rationale**: パフォーマンスと安定性を実証的に比較するため。
- **Trade-offs**: 実験フェーズにおいて保守すべきコードがわずかに増える。

### Decision: Logical `Visual` Component Structure
- **Context**: ビジュアルノードの論理的な表現が必要。
- **Selected Approach**: `struct Visual { is_visible: bool, opacity: f32, transform: Matrix3x2 }`
- **Rationale**: R1で要求される基本的なプロパティをカバーしている。
- **Trade-offs**: とくになし。

## Risks & Mitigations
- **Risk**: HooksにおけるCOMのスレッド問題。
  - **Mitigation**: `GraphicsCore`へのアクセスがスレッドセーフであることを確認するか、可能であればHookの実行をメインスレッドに制限する（Bevyのhookはコマンドが適用される場所で実行される）。
- **Risk**: リソースリーク。
  - **Mitigation**: COMオブジェクトを解放するために`Drop`または`on_remove`フックを実装する。
