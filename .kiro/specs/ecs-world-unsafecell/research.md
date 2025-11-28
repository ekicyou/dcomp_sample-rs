# Research & Design Decisions: ecs-world-unsafecell

## Summary
- **Feature**: ecs-world-unsafecell
- **Discovery Scope**: Extension（既存システムの拡張）
- **Key Findings**:
  - 現在のRefCellパターンは6箇所で使用され、うち2箇所が`try_borrow_mut()`による防御的コード
  - `thread_local!`マクロによる`!Send`/`!Sync`の自動保証が利用可能
  - プロジェクト内に`unsafe impl Send/Sync`のパターンが既に複数存在し、SAFETYコメントの書き方が確立済み

## Research Log

### UnsafeCellの安全性保証

- **Context**: RefCellをUnsafeCellに置き換える際の安全性根拠を確認
- **Sources Consulted**: 
  - Rust標準ライブラリドキュメント (`std::cell::UnsafeCell`)
  - プロジェクト内既存パターン（`SendWeak`, `GraphicsCore`など）
- **Findings**:
  - `UnsafeCell`は内部可変性の最も低レベルなプリミティブで、ランタイムチェックなし
  - `thread_local!`マクロ内で使用すると自動的にスレッドローカルになり、`!Send`/`!Sync`が保証される
  - Windowsメッセージループはシングルスレッドで動作するため、再入は同一スレッド上で発生
  - 同一スレッド上での再入は、コールスタックが分離されるため論理的に安全
- **Implications**: 
  - `UnsafeCell<Option<EcsWorld>>`と`thread_local!`の組み合わせで安全に実装可能
  - 明示的な`!Send`/`!Sync`実装は不要（thread_local!が保証）

### 現在のアクセスパターン分析

- **Context**: 移行対象となるコードの特定
- **Sources Consulted**: 
  - `grep_search`による`.borrow_mut()`使用箇所の調査
- **Findings**:
  - **win_thread_mgr.rs:93** - 初期化時（`set_message_window`）
  - **win_thread_mgr.rs:173** - WM_VSYNC（フレーム更新 `try_tick_world`）
  - **window_proc.rs:57** - WM_NCDESTROY（Entity削除）
  - **window_proc.rs:91** - WM_WINDOWPOSCHANGED（`try_borrow_mut`で防御）
  - **window_proc.rs:190** - WM_DISPLAYCHANGE（`try_borrow_mut`で防御）
  - **window_proc.rs:230** - WM_DPICHANGED_DEFERRED（遅延処理）
- **Implications**: 
  - `try_borrow_mut()`の2箇所は直接アクセスに変更可能
  - examplesの`world.borrow_mut()`呼び出しもAPI変更が必要

### WM_DPICHANGED_DEFERRED維持の根拠

- **Context**: PostMessage遅延を維持する技術的理由の確認
- **Sources Consulted**:
  - bevy_ecs 0.17.2の`Changed<T>`フィルター実装
  - プロジェクトのスケジュール実行順序
- **Findings**:
  - `Changed<T>`フィルターは全スケジュール完了時にフラッシュ
  - スケジュール順序: Layout → UISetup（DPI変更はUISetupのSetWindowPosで発生）
  - 同一フレーム内でDPI変更しても、既に実行済みのLayoutでは検出不可
  - PostMessageは次フレーム開始前にDPIを更新するため、Changed<DPI>が機能
- **Implications**: 
  - WM_DPICHANGED_DEFERREDはUnsafeCell化とは独立した必要性がある
  - 要件R4として維持を明文化

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| UnsafeCell + thread_local! | thread_local!内にUnsafeCell<Option<EcsWorld>>を配置 | 自動的な!Send/!Sync、軽量、再入安全 | 初期化前アクセスでpanic | **採用** |
| RefCell維持 + フラグ管理 | 再入フラグで手動管理 | 安全性チェック維持 | 複雑、オーバーヘッド | 却下 |
| Mutex | std::sync::Mutexを使用 | 標準的なパターン | デッドロックリスク、不要なオーバーヘッド | 却下 |

## Design Decisions

### Decision: `EcsWorldCell`の配置場所

- **Context**: 新しい型`EcsWorldCell`をどのモジュールに配置するか
- **Alternatives Considered**:
  1. `ecs/world.rs` - EcsWorldと同じファイル
  2. `ecs/world_cell.rs` - 新規ファイル
  3. `ecs/mod.rs` - ecsモジュールルート
- **Selected Approach**: `ecs/world.rs`に配置
- **Rationale**: 
  - `EcsWorld`と密接に関連するため同一ファイルが自然
  - ファイル数増加を抑制
  - 既存のpub exportを最小限の変更で維持
- **Trade-offs**: ファイルが長くなるが、責務の一貫性を重視
- **Follow-up**: 将来的にファイルが肥大化した場合は分割を検討

### Decision: グローバル変数の配置と初期化API

- **Context**: `thread_local!`をどこに配置し、どのようなAPIで初期化するか
- **Alternatives Considered**:
  1. `ecs/world.rs`にthread_local!、init_world() + with_world()
  2. `ecs/window_proc.rs`にthread_local!、set_ecs_world()を拡張
  3. 専用モジュール`ecs/global.rs`
- **Selected Approach**: `ecs/world.rs`にthread_local!、`init_world()` + `with_world()`
- **Rationale**: 
  - `EcsWorld`の管理責務を一箇所に集約
  - 既存の`set_ecs_world`/`get_ecs_world`を削除してAPI簡素化
- **Trade-offs**: window_proc.rsからのインポートパスが変更
- **Follow-up**: pub(crate)で内部公開に限定

### Decision: 初期化前アクセスの処理

- **Context**: `with_world()`が初期化前に呼ばれた場合の挙動
- **Alternatives Considered**:
  1. `panic!` - 早期失敗
  2. `Option<R>`を返す - 呼び出し側で処理
  3. デフォルトEcsWorldを自動作成
- **Selected Approach**: `panic!`で早期失敗
- **Rationale**: 
  - 初期化順序のバグは致命的であり、早期検出が望ましい
  - `WinThreadMgr::new()`で必ず初期化されるため、正常フローでは発生しない
  - Option戻り値は全呼び出し箇所でunwrap/matchが必要になり煩雑
- **Trade-offs**: ランタイムパニックの可能性があるが、開発時にのみ発生
- **Follow-up**: テストで初期化順序を検証

### Decision: wndproc内での`with_world()`使用制約

- **Context**: `&mut EcsWorld`の重複によるUB（未定義動作）の回避
- **Issue**: wndproc再入時に`with_world()`が重複呼び出しされると、`&mut`が同時に2つ存在しUBになる可能性
- **Analysis**:
  - WM_DPICHANGEDはPostMessageで遅延されるため再入しない
  - 一般的な再入シナリオでも、一時変数にworld内変数を保持しなければ問題ない
- **Selected Approach**: wndproc内の`with_world()`使用を以下に限定
  1. **マーカーコンポーネント投入**
  2. **イベント投入**
- **Rationale**: 
  - いずれの処理もWindows APIを呼び出さない
  - Windows API呼び出しがなければ再入は原理的に発生しない
  - したがって`&mut EcsWorld`の重複は発生せず、UBは回避される
- **Future Direction**: wndproc内の処理は最終的にECSイベント投入方式に統一予定
- **Follow-up**: SAFETYコメントにこの制約を明記

## Risks & Mitigations

- **Risk 1: unsafeコードによるメモリ安全性の懸念**
  - Mitigation: SAFETYコメントの徹底、thread_local!による静的保証、コードレビュー
  - Mitigation: wndproc内での`with_world()`使用をマーカー/イベント投入に限定し、Windows API呼び出しを禁止
- **Risk 2: examples/testsの移行漏れ**
  - Mitigation: grep_searchで全使用箇所を特定、コンパイルエラーで検出
- **Risk 3: 将来のマルチスレッド化時の問題**
  - Mitigation: SAFETYコメントにシングルスレッド前提を明記、doc/に設計判断を文書化

## References

- [Rust std::cell::UnsafeCell](https://doc.rust-lang.org/std/cell/struct.UnsafeCell.html) - 内部可変性の基本ドキュメント
- [Rust thread_local!](https://doc.rust-lang.org/std/macro.thread_local.html) - スレッドローカルストレージのマクロ
- プロジェクト内パターン: `src/ecs/window_proc.rs` の `SendWeak` 実装
