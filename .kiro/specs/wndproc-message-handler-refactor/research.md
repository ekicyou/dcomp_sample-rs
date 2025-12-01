# Research & Design Decisions: wndproc-message-handler-refactor

## Summary
- **Feature**: `wndproc-message-handler-refactor`
- **Discovery Scope**: Extension（既存モジュールのリファクタリング）
- **Key Findings**:
  - 現在の`window_proc.rs`は366行で、単一の大きな`match`式にすべてのメッセージ処理が含まれている
  - `WM_WINDOWPOSCHANGED`が最大のハンドラ（約180行）で、複雑な処理を含む
  - 外部APIへの依存変更なし、既存パターンの適用のみ

## Research Log

### Rustの命名規則と`#[allow]`属性
- **Context**: ハンドラ関数名を`WM_NCCREATE`のようにWindowsメッセージ定数と同名にする要件
- **Sources Consulted**: Rust公式ドキュメント、clippy lint一覧
- **Findings**:
  - Rust標準では関数名はsnake_caseを推奨
  - `#[allow(non_snake_case)]`でlint警告を抑制可能
  - モジュールレベル`#![allow(non_snake_case)]`で一括抑制も可能
- **Implications**: `handlers.rs`にモジュールレベルで`#![allow(non_snake_case)]`を配置

### `#[inline]`属性の効果
- **Context**: 関数分離によるパフォーマンス影響の懸念
- **Sources Consulted**: Rust公式ドキュメント、LLVM最適化ドキュメント
- **Findings**:
  - `#[inline]`はコンパイラへのヒント（強制ではない）
  - Release最適化（`opt-level >= 2`）では小さな関数は自動インライン化される
  - `extern "system"` ABI関数はインライン化不可（関数ポインタとして使用されるため）
- **Implications**: ハンドラ関数には`#[inline]`を付与、`ecs_wndproc`には付与しない

### ディレクトリモジュール構造への変換
- **Context**: `window_proc.rs`を`window_proc/`ディレクトリに変換
- **Sources Consulted**: Rust公式ドキュメント、既存のecsモジュール構造
- **Findings**:
  - `window_proc.rs` → `window_proc/mod.rs`への変換はRustの標準パターン
  - 親モジュール`ecs/mod.rs`の`mod window_proc;`宣言は変更不要
  - サブモジュールは`mod handlers;`で宣言し、`use handlers::*;`でインポート
- **Implications**: 既存の`ecs/mod.rs`の変更は不要

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| 同一ファイル内分離 | `window_proc.rs`内に関数を追加 | 最小変更、シンプル | ファイル肥大化 | 短期的には有効 |
| ディレクトリ分離 | `window_proc/mod.rs` + `handlers.rs` | 将来の拡張に対応、保守性向上 | 追加のファイル構造 | **採用** |
| カテゴリ別分離 | `handlers/lifecycle.rs`等 | 高度な組織化 | 現時点では過剰 | 将来検討 |

## Design Decisions

### Decision: ディレクトリベースのモジュール構造
- **Context**: 将来的に多数のメッセージハンドラが追加される見込み
- **Alternatives Considered**:
  1. 同一ファイル内で関数分離のみ
  2. ディレクトリ構造（`window_proc/mod.rs` + `handlers.rs`）
- **Selected Approach**: Option 2（ディレクトリ構造）
- **Rationale**: 要件R8で明示的に指定、将来の拡張性を確保
- **Trade-offs**: 初期コストは若干増加するが、保守性が向上
- **Follow-up**: 800行を超えた時点でカテゴリ別分離を検討

### Decision: `Option<LRESULT>`戻り値パターン
- **Context**: `DefWindowProcW`呼び出しの一元化
- **Alternatives Considered**:
  1. 各ハンドラが直接`LRESULT`を返す（現在の実装）
  2. `Option<LRESULT>`を返し、`None`で`DefWindowProcW`に委譲
- **Selected Approach**: Option 2
- **Rationale**: `DefWindowProcW`呼び出しを`ecs_wndproc`内の1箇所に集約し、一貫性を確保
- **Trade-offs**: 若干の抽象化追加、ただし可読性は向上
- **Follow-up**: なし

### Decision: `pub(super)`可視性の使用
- **Context**: ハンドラ関数の可視性設計
- **Alternatives Considered**:
  1. `pub(crate)` - crate全体から参照可能
  2. `pub(super)` - 親モジュール（`window_proc`）からのみ参照可能
  3. private - サブモジュール内のみ
- **Selected Approach**: `pub(super)`
- **Rationale**: ハンドラは`mod.rs`の`ecs_wndproc`からのみ呼び出される、最小権限の原則
- **Trade-offs**: なし
- **Follow-up**: なし

## Risks & Mitigations
- **Risk 1**: 関数分離による回帰バグ — `cargo test`で既存テスト通過を確認
- **Risk 2**: インライン展開が効かない — Releaseビルドでベンチマーク確認（必要に応じて）
- **Risk 3**: 名前衝突（`WM_*`定数と関数名） — `windows`クレートの定数はuse宣言でスコープに入れない

## References
- [Rust Reference: Inline attributes](https://doc.rust-lang.org/reference/attributes/codegen.html#the-inline-attribute)
- [Rust Reference: Modules](https://doc.rust-lang.org/reference/items/modules.html)
- [windows-rs crate documentation](https://docs.rs/windows/latest/windows/)
