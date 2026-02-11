# Research & Design Decisions

## Summary
- **Feature**: `wintf-fix2-pointer-state-rename`
- **Discovery Scope**: Simple Addition（純粋なフィールドリネーム）
- **Key Findings**:
  - 変更対象は `PointerState.screen_point` フィールドとその参照のみ（約28箇所）
  - `nchittest_cache.rs` / `hit_test.rs` の `screen_point` は別概念（実スクリーン座標）であり対象外
  - tracing ログの構造化フィールド名 `screen_x`/`screen_y` も意味的に更新すべき

## Research Log

### PointerState.screen_point の参照箇所の完全調査

- **Context**: リネーム対象と非対象を正確に識別する必要がある
- **Sources Consulted**: `grep_search` による codebase 全検索
- **Findings**:
  - **リネーム対象（PointerState フィールド参照）**:
    - `pointer/mod.rs`: 14箇所（定義1, Default1, アクセス10, テスト1, コメント1）
    - `handlers.rs`: 4箇所（構造体リテラル初期化）
    - `taffy_flex_demo.rs`: 10行（`state.screen_point.x/y` アクセス）
  - **リネーム対象外（別概念）**:
    - `nchittest_cache.rs`: 20+箇所（WM_NCHITTEST 用、型は `(i32, i32)`、`PointerState` と無関係）
    - `hit_test.rs`: 5箇所（関数パラメータ名、`PointerState` と無関係）
  - **`tests/` ディレクトリ**: `screen_point` 参照なし
- **Implications**: ファイル単位の選択的置換で安全に実施可能。一括 `sed` は危険

### tracing ログの構造化フィールド名

- **Context**: `pointer/mod.rs` L618-619 に `screen_x = pointer.screen_point.x` のログ出力あり。フィールド名リネーム時にログキーも更新すべきか？
- **Sources Consulted**: `pointer/mod.rs` L506-519, L614-627; ステアリング `logging.md`
- **Findings**:
  - L515-516: `new_x = pointer.screen_point.x`（ログキー: `new_x`/`new_y`）→ フィールド名を含まないキー名、変更不要
  - L618-619: `screen_x = pointer.screen_point.x`（ログキー: `screen_x`/`screen_y`）→ `screen_*` はスクリーン座標を示唆するため不正確
  - ステアリング `logging.md` の座標フィールド規約: `x = pos.x, y = pos.y` のパターン
- **Implications**: `screen_x`/`screen_y` → `client_x`/`client_y` に変更すべき。ログキー名も座標系の名前と一致させるのが正確

## Design Decisions

### Decision: tracing ログの構造化フィールド名も更新する

- **Context**: `pointer/mod.rs` L618-619 で `screen_x`/`screen_y` というログキー名がフィールドのアクセスに使われている
- **Alternatives Considered**:
  1. ログキー名はそのまま残す（ログ集約の互換性を維持）
  2. `client_x`/`client_y` に更新する（意味の正確性を優先）
- **Selected Approach**: Option 2 — `client_x`/`client_y` に更新
- **Rationale**: 
  - このプロジェクトのログは開発デバッグ用で、外部の集約・ダッシュボードなどは未構築
  - フィールド名と同じ座標系の名前を使うことで意味が明確になる
  - steering の `logging.md` は `x`/`y` のシンプルなキー名を推奨しており、`screen_*` よりも `client_*` が正確
- **Trade-offs**: ログキーの名前変更による過去ログとの非互換（実害なし — 開発ログのみ）
- **Follow-up**: 設計レビュー時に開発者の確認を取る

## Risks & Mitigations
- **Risk 1**: リネーム漏れ → Rust コンパイラが未変更のフィールドアクセスをコンパイルエラーとして検出。ミス発見は自動的
- **Risk 2**: 対象外箇所の誤リネーム → ファイル単位での選択的置換で防止（`nchittest_cache.rs` と `hit_test.rs` を除外）
- **Risk 3**: ログキー変更の互換性 → 開発ログのみであり実害なし

## References
- 親仕様: `.kiro/specs/dpi-coordinate-transform-survey/report.md` §1.2, §6.4.2, §8.2
- ステアリング: `.kiro/steering/logging.md` — 構造化ログのフィールド規約
