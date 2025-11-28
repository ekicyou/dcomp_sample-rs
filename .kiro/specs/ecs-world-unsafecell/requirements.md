# Requirements Document

## Introduction

本仕様は、wintfフレームワークにおけるEcsWorldへのアクセス機構を改善し、wndproc（ウィンドウプロシージャ）の再入問題に対応することを目的とする。

### 背景

現在の実装では`Rc<RefCell<EcsWorld>>`を使用してEcsWorldを管理している。この方式には以下の問題がある：

1. **再入時のパニック**: wndprocは同期的に呼ばれることがあり（例：`WM_DPICHANGED`）、EcsWorld借用中に別のwndproc呼び出しが発生すると`RefCell`の二重借用パニックが発生する
2. **防御的コーディングの必要性**: 現在は`try_borrow_mut()`や`PostMessage`による遅延処理で回避しているが、コードが複雑化している
3. **シングルスレッド保証の未活用**: Windowsメッセージループは本質的にシングルスレッドで動作するため、より軽量な同期機構で十分なはず

### 解決方針

`UnsafeCell`を使用した軽量なアクセス機構を導入し、シングルスレッド環境でのメッセージ処理を自然に行えるようにする。

## Requirements

### Requirement 1: UnsafeCellベースのEcsWorldラッパー

**Objective:** As a フレームワーク開発者, I want EcsWorldへのアクセスをUnsafeCellで管理する, so that wndprocの再入時にパニックせず自然に処理できる

#### Acceptance Criteria

1. The wintf shall `UnsafeCell<EcsWorld>`をラップする新しい型`EcsWorldCell`を提供する
2. The `EcsWorldCell` shall スレッドローカルストレージ（thread_local!）で管理され、メインスレッドからのみアクセス可能とする
3. When wndprocからEcsWorldにアクセスする時, the `EcsWorldCell` shall 可変参照を直接提供する（RefCellのオーバーヘッドなし）
4. The `EcsWorldCell` shall `!Send`かつ`!Sync`を明示し、コンパイル時にスレッド安全性を保証する

### Requirement 2: 再入安全なアクセスAPI

**Objective:** As a フレームワーク開発者, I want 再入を許容するアクセスAPIを持つ, so that ネストしたwndproc呼び出しで自然に動作する

#### Acceptance Criteria

1. The `EcsWorldCell` shall `with_world<R>(f: impl FnOnce(&mut EcsWorld) -> R) -> R`形式のアクセスAPIを提供する
2. When 再入が発生した時, the `EcsWorldCell` shall 同じスレッド上での再入アクセスを許可する
3. If マルチスレッドからアクセスが試みられた場合, then the `EcsWorldCell` shall コンパイルエラーとなる（`!Send`/`!Sync`により）

### Requirement 3: 既存コードの移行

**Objective:** As a フレームワーク開発者, I want 既存の`Rc<RefCell<EcsWorld>>`を新機構に置き換える, so that コードベース全体で一貫したアクセスパターンを使用できる

#### Acceptance Criteria

1. The wintf shall `WinThreadMgrInner`の`world: Rc<RefCell<EcsWorld>>`フィールドを削除し、`EcsWorldCell`に移行する
2. The wintf shall `ecs/window_proc.rs`の`ECS_WORLD`グローバル変数を`EcsWorldCell`のthread_local!に置き換える
3. When `WinThreadMgr::run()`のメッセージループでEcsWorldにアクセスする時, the wintf shall `EcsWorldCell::with_world()`を使用する
4. The wintf shall `try_borrow_mut()`による防御的コードを削除し、直接アクセスに置き換える

### Requirement 4: PostMessage遅延処理の維持

**Objective:** As a フレームワーク開発者, I want DPI変更のPostMessage遅延を維持する, so that bevy_ecsの`Changed<DPI>`フィルターが正常に動作する

#### Acceptance Criteria

1. When `WM_DPICHANGED`を処理する時, the wintf shall 現行通りPostMessageで`WM_DPICHANGED_DEFERRED`を送信する
2. The wintf shall `WM_DPICHANGED_DEFERRED`ハンドラでDPIコンポーネントを更新する（スケジュール外での更新）
3. The wintf shall この遅延が`Changed<DPI>`フィルターの動作保証のためであることをコメントで文書化する

#### 技術的根拠

- `Changed<T>`フィルターは全スケジュール完了時にフラッシュされる
- 同一フレーム内（スケジュール実行中）にDPIを変更しても、Layoutスケジュール（UISetupより前）では検出されない
- PostMessageによる遅延は次フレーム開始前にDPIを更新するため、`Changed<DPI>`が確実に機能する
- 1フレームの遅延は視覚的に許容範囲であり、レイアウト不発よりはるかに軽微

### Requirement 5: 安全性の文書化

**Objective:** As a フレームワーク開発者, I want unsafeコードの安全性根拠を明確に文書化する, so that 将来のメンテナンスが容易になる

#### Acceptance Criteria

1. The wintf shall `EcsWorldCell`の実装に`SAFETY:`コメントを付与し、以下を説明する：
   - シングルスレッド保証の根拠（Windowsメッセージループの性質）
   - 再入時の安全性（同一スレッド上での排他的アクセスは論理的に安全）
   - `!Send`/`!Sync`による静的保証
2. The wintf shall モジュールレベルのドキュメントで使用上の注意を記載する
3. The wintf shall 設計判断の理由をdoc/に文書化する

