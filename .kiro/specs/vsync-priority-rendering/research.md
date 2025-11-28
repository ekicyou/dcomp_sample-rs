# Research & Design Decisions

## Summary
- **Feature**: vsync-priority-rendering
- **Discovery Scope**: Extension（既存システムの拡張）
- **Key Findings**:
  1. `WM_WINDOWPOSCHANGED`処理でworld借用が可能（`try_borrow_mut()`パターン既存）
  2. VSYNCスレッドとメインスレッドの通信は`PostMessageW`でWM_VSYNCを送信
  3. `static AtomicU64`によるロックフリーカウンター方式が最適

## Research Log

### Rust AtomicU64 Ordering選択
- **Context**: VSYNC_TICK_COUNTとLAST_VSYNC_TICKの安全なアトミック更新方式
- **Sources Consulted**: 
  - Rust std::sync::atomic::Ordering ドキュメント
  - 既存コードの`DEBUG_*`カウンターでの`Ordering::Relaxed`使用パターン
- **Findings**:
  - `VSYNC_TICK_COUNT`: VSYNCスレッドのみがインクリメント、メインスレッドは読み取りのみ
  - `LAST_VSYNC_TICK`: メインスレッドのみが更新・読み取り
  - 厳密な順序保証は不要（多少の遅延は許容される）
  - `Relaxed`で十分だが、メインスレッド内の一貫性のため`Acquire/Release`も検討可能
- **Implications**: 
  - `VSYNC_TICK_COUNT`: VSYNCスレッドで`Release`、メインスレッドで`Acquire`
  - `LAST_VSYNC_TICK`: メインスレッドのみなので`Relaxed`で十分
  - シンプルさと既存パターンとの一貫性から、すべて`Relaxed`を採用

### tick_count比較ロジック
- **Context**: VSYNC_TICK_COUNTとLAST_VSYNC_TICKの比較方法
- **Sources Consulted**: 既存コード分析
- **Findings**:
  - 単純な不等価比較（`!=`）で十分
  - u64は64ビット整数で、秒間60回のインクリメントでも約97億年でオーバーフロー
  - ラップアラウンド対応は実質不要だが、`wrapping_sub`を使用しても安全
- **Implications**: `current != last`の単純比較を採用

### WM_WINDOWPOSCHANGED処理パターン
- **Context**: 既存の`try_borrow_mut()`パターンの確認
- **Sources Consulted**: `window_proc.rs`
- **Findings**:
  - 既存コード（90-95行目）で`try_borrow_mut()`パターンを使用済み
  - 借用失敗時は処理をスキップする安全な実装
  - Entity取得 → World借用 → コンポーネント更新の流れ
- **Implications**: tick処理をこの流れの最初に挿入可能

### VsyncTickトレイト設計
- **Context**: `Rc<RefCell<EcsWorld>>`に対するtick機能の拡張方法
- **Sources Consulted**: Rust拡張トレイトパターン
- **Findings**:
  - 拡張トレイトで`Rc<RefCell<EcsWorld>>`に新しいメソッドを追加可能
  - `world.rs`に配置することで`win_thread_mgr.rs`と`window_proc.rs`両方から参照可能
  - メソッド名は`EcsWorld`と同じ`try_tick_on_vsync`で一貫性を保つ
- **Implications**: トレイトを`world.rs`に定義し、pubで公開

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| tick_count + WndProc | アトミックカウンターでVSYNC検知、WndProcでtick実行 | ロックフリー、シンプル、既存パターン活用 | 新規static変数追加 | 採用 |
| WM_TIMER | タイマーでtick駆動 | Windows標準 | モーダルループでブロック（問題解決しない） | 却下 |
| SetWindowsHookEx | フックでメッセージ監視 | 柔軟 | オーバーヘッド、複雑性増大 | 却下 |

## Design Decisions

### Decision: アトミックカウンター方式
- **Context**: VSYNCスレッドからメインスレッドへのVSYNC到来通知
- **Alternatives Considered**:
  1. WM_VSYNC依存（現状）— モーダルループでブロック
  2. Mutex — ロック競合リスク
  3. AtomicU64カウンター — ロックフリー
- **Selected Approach**: `static AtomicU64`による2つのカウンター
- **Rationale**: ロックフリーで安全、既存の`DEBUG_*`カウンターと同様のパターン
- **Trade-offs**: static変数追加、メモリオーダリングの理解が必要
- **Follow-up**: デバッグログでtick実行元（run/WndProc）を区別可能にする

### Decision: Ordering::Relaxedの採用
- **Context**: AtomicU64のメモリオーダリング選択
- **Alternatives Considered**:
  1. `Relaxed` — 最小オーバーヘッド
  2. `Acquire/Release` — より強い順序保証
  3. `SeqCst` — 最も厳格な順序保証
- **Selected Approach**: すべて`Ordering::Relaxed`
- **Rationale**: 
  - VSYNCスレッドとメインスレッドの通信は1方向（インクリメント→読み取り）
  - 多少の遅延（1フレーム未満）は許容される
  - 既存の`DEBUG_*`カウンターと同じパターン
- **Trade-offs**: 理論上はリオーダリングの可能性があるが、実用上問題なし
- **Follow-up**: パフォーマンス問題が発生した場合に再検討

### Decision: VsyncTickトレイトの配置
- **Context**: トレイトをどのモジュールに配置するか
- **Alternatives Considered**:
  1. `world.rs` — world tick関連機能の集約
  2. `window_proc.rs` — 使用箇所に近い
  3. 新規`vsync.rs` — 関連機能を分離
- **Selected Approach**: `world.rs`に配置
- **Rationale**: 
  - world tick関連機能として意味的に適切
  - `win_thread_mgr.rs`と`window_proc.rs`両方から参照
  - 新規ファイル追加の複雑性を回避
- **Trade-offs**: `world.rs`のサイズが若干増加
- **Follow-up**: なし

## Risks & Mitigations
- **Risk 1**: WndProc内でのtick実行によるパフォーマンス影響
  - **Mitigation**: カウンター比較が高速（数ナノ秒）、tick実行は必要時のみ
- **Risk 2**: 再入時の借用失敗
  - **Mitigation**: `try_borrow_mut()`で安全にスキップ、既存パターン踏襲
- **Risk 3**: デバッグ困難性
  - **Mitigation**: tick実行元をログで区別可能にする（Req 6.3）

## References
- [Rust Ordering documentation](https://doc.rust-lang.org/std/sync/atomic/enum.Ordering.html)
- 既存コード: `win_thread_mgr.rs` (DEBUG_*カウンター)
- 既存コード: `window_proc.rs` (try_borrow_mut()パターン)
