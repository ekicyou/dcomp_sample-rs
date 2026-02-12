# Research & Design Decisions: wintf-fix4-feedback-loop-simplify

## Summary
- **Feature**: `wintf-fix4-feedback-loop-simplify`
- **Discovery Scope**: Extension（既存リファクタリング）
- **Key Findings**:
  1. `WM_WINDOWPOSCHANGED` は Sent メッセージ（同期）であり、`SWP_ASYNCWINDOWPOS`（クロススレッド時のみ）を使わない限り非同期にならない
  2. `SetWindowPos` 呼び出しサイトは ECS パス内に3箇所（flush、set_window_pos メソッド、WM_DPICHANGED ハンドラ）のみ
  3. `WindowPosChanged` の影響範囲は handlers.rs, systems.rs, world.rs に限定。テストファイル（feedback_loop_convergence_test.rs）には直接参照なし

## Research Log

### RI-1: SetWindowPos → WM_WINDOWPOSCHANGED の同期保証

- **Context**: ラッパー方式の前提条件として、`SetWindowPos` 呼び出しが `WM_WINDOWPOSCHANGED` を同期的に発火することを MSDN で確認する必要がある
- **Sources Consulted**:
  - [SetWindowPos (MSDN)](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos)
  - [WM_WINDOWPOSCHANGED (MSDN)](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-windowposchanged)
  - [WM_WINDOWPOSCHANGING (MSDN)](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-windowposchanging)
- **Findings**:
  - `WM_WINDOWPOSCHANGED` は "**Sent** to a window" と明記 — Posted ではなく同期ディスパッチ
  - `WM_WINDOWPOSCHANGING` も同様に "**Sent** to a window" — SetWindowPos 実行前に同期で送信
  - `SWP_ASYNCWINDOWPOS` (0x4000) フラグ: "If the calling thread and the thread that owns the window are **attached to different input queues**, the system **posts** the request" — 非同期化は**クロススレッド時のみ**
  - `SWP_NOSENDCHANGING` (0x0400): `WM_WINDOWPOSCHANGING` の抑制のみ。`WM_WINDOWPOSCHANGED` には影響なし
- **Implications**:
  - wintf はシングルスレッド UI アーキテクチャ → `SWP_ASYNCWINDOWPOS` は無関係
  - `SetWindowPos` → `WM_WINDOWPOSCHANGING` → `WM_WINDOWPOSCHANGED` は全て同一コールスタック内で完結
  - **TLS ラッパーフラグは安全に動作する**
  - L3（`RefCell` 再入保護）が万が一の保険として引き続き存在

### コードベース統合ポイントの確認

- **Context**: ラッパー方式の影響範囲を正確に特定
- **Sources Consulted**: codebase grep（`WindowPosChanged`, `is_echo`, `last_sent`, `SetWindowPos`, `bypass_change_detection`, `flush`）
- **Findings**:

  **SetWindowPos Win32 API 直接呼び出し（ECS パス内）:**
  | # | ファイル | 行 | コンテキスト | ラッパー対象 |
  |---|---------|-----|-------------|-------------|
  | 1 | `ecs/window.rs` | L169 | `SetWindowPosCommand::flush()` 内 | YES |
  | 2 | `ecs/window.rs` | L931 | `WindowPos::set_window_pos()` メソッド | YES（使用箇所要確認） |
  | 3 | `handlers.rs` | L410 | `WM_DPICHANGED` ハンドラ | YES |

  **`WindowPosChanged` 実質的な操作箇所（コメント除く）:**
  | 操作 | ファイル | 行 |
  |------|---------|-----|
  | 構造体定義 | `ecs/window.rs` | L207 |
  | `true` 設定 | `handlers.rs` | L175 |
  | `false` リセット | `handlers.rs` | L316 |
  | Query + ガード | `systems.rs` | L708, L722 |

  **`is_echo` / `last_sent_*` 操作箇所（テスト除く）:**
  | 操作 | ファイル | 行 |
  |------|---------|-----|
  | `is_echo()` 定義 | `ecs/window.rs` | L936 |
  | `is_echo()` 呼出 | `systems.rs` | L734 |
  | `last_sent_*` フィールド | `ecs/window.rs` | L687-688 |
  | `last_sent_*` 書込（DerefMut） | `handlers.rs` | L204-206 |
  | `last_sent_*` 書込（bypass） | `handlers.rs` | L229-230 |
  | `last_sent_*` 記録（bypass） | `systems.rs` | L808-809 |

  **テストへの影響:**
  - `feedback_loop_convergence_test.rs`（8テスト）: `WindowPosChanged`, `is_echo`, `last_sent` への**直接参照なし** → 修正不要
  - `layout_graphics_sync_test.rs`: `is_echo`, `last_sent_*`, `bypass_change_detection` を直接使用 → **修正必要**

- **Implications**:
  - ラッパーで囲むべき箇所は3箇所と少なく、影響範囲は限定的
  - `WindowPos::set_window_pos()` メソッド（L931）の使用箇所の調査が必要（設計時に確認）
  - `bypass_change_detection()` は `last_sent_*` 削除に伴い、用途が変わる可能性がある

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| ラッパー方式（採用） | `SetWindowPos` 呼び出しを TLS フラグ管理ラッパーで囲む | 3つの既存メカニズムを一挙に置換、4→3ステップ化、工数 S | 同期保証への依存（MSDN で確認済み）| `DpiChangeContext` と同じ TLS パターン |
| 旧 Option A: TLS ゲート統合 | `WindowPosGate` TLS 構造体でキュー+フラグ管理 | World 第2借用削除 | `is_echo()` が残る、構造体やや複雑 | ラッパー方式で上位互換 |
| 旧 Option B: ECS Resource 化 | `WindowPosChanged` → Resource 化 | bevy_ecs イディオム準拠 | 4ステップ維持、二重管理残存 | 簡素化効果が小さい |
| 旧 Option C: TLS + flush 集約 | Option A + flush 呼び出し整理 | 最大簡素化 | TLS 構造体が3責務、テスト困難 | ラッパー方式で `is_echo` 不要化によりさらにシンプルに |

## Design Decisions

### Decision: ラッパー関数の API 設計 — TLS bool vs RAII Drop guard

- **Context**: ラッパー関数が TLS フラグを ON/OFF する方式の選択
- **Alternatives Considered**:
  1. 単純 TLS bool — `set(true)` → `SetWindowPos()` → `set(false)`
  2. RAII Drop guard — `let _guard = SetWindowPosGuard::new()` でスコープ管理
- **Selected Approach**: **RAII Drop guard**（設計レビュー後の最終決定）
- **Rationale**: `SetWindowPos` が `Err` を返す場合、`?` による early return で `set(false)` が実行されず TLS フラグが `true` のまま残留する危険性がある。RAII Drop guard により、`?` 使用時もパニック時も確実にフラグをリセットできる。Rust の慣用的なパターンであり、実装コストも軽微。
- **Trade-offs**: わずかな複雑性の増加（Drop trait 実装）、ただし Rust では標準的パターン
- **Follow-up**: `SetWindowPosGuard` 構造体を `ecs/window.rs` に追加、Drop trait で `IS_SELF_INITIATED.set(false)` を実装

### Decision: echo 判定時の `WM_WINDOWPOSCHANGED` ハンドラ動作 — スキップ vs bypass 更新

- **Context**: ラッパーフラグ ON 時（自アプリ由来の通知）のハンドラ動作
- **Alternatives Considered**:
  1. 値更新を完全スキップ — `WindowPos` に一切触れない
  2. `bypass_change_detection()` で更新しつつ `Changed` 抑制
- **Selected Approach**: **(b) bypass_change_detection() で更新**
- **Rationale**: `flush()` が実行した `SetWindowPos` の結果として OS がわずかに異なる値を返す可能性がある（スナップ、DPI 丸め等）。`WindowPos` コンポーネントは常に OS の実際の状態を反映すべきであり、bypass で更新すれば `Changed` は発火せずデータ整合性を維持できる。
- **Trade-offs**: bypass 更新のオーバーヘッドは無視できるレベル。データ整合性 > パフォーマンス
- **Follow-up**: CW_USEDEFAULT ガード（G3）は維持。DpiChangeContext 処理も echo 時でも実行

### Decision: flush 呼び出しポイント — 3箇所維持

- **Context**: 冗長な flush 呼び出しを削減可能か
- **Alternatives Considered**:
  1. 2箇所に削減（VsyncTick + WM_VSYNC のみ、WM_WINDOWPOSCHANGED ③ 削除）
  2. 3箇所維持（冪等保証のため安全策）
- **Selected Approach**: **(b) 3箇所維持**
- **Rationale**: flush は冪等操作であり、冗長呼び出しのコストはほぼゼロ。一方、特定パスでの flush 欠落は致命的（SetWindowPos キューが蓄積）。安全性を優先。
- **Trade-offs**: 若干の冗長性はあるが、正確性の保険として価値がある
- **Follow-up**: 将来的に flush 呼び出しを整理する場合は別仕様で対応

## Risks & Mitigations
- **同期保証への依存** — MSDN で Sent メッセージであることを確認済み。L3（RefCell 再入保護）が保険として存在
- **`WindowPos::set_window_pos()` の見落とし** — 設計時に全呼び出しサイトを grep で確認してラッパー対象に含める
- **テスト修正漏れ** — `layout_graphics_sync_test.rs` が `is_echo`/`last_sent` を使用。削除に伴う修正が必要

## References
- [SetWindowPos (MSDN)](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos) — 同期/非同期動作、SWP_ASYNCWINDOWPOS の定義
- [WM_WINDOWPOSCHANGED (MSDN)](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-windowposchanged) — Sent メッセージ（同期）の確認
- [WM_WINDOWPOSCHANGING (MSDN)](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-windowposchanging) — Sent メッセージ（同期）の確認
- `.kiro/specs/wintf-fix4-feedback-loop-simplify/gap-analysis.md` — 詳細なコードベース調査結果
