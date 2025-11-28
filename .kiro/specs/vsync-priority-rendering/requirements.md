# Requirements Document

## Introduction

本仕様は、`taffy_flex_demo`でウィンドウをマウスドラッグで移動している間に描画が遅延する問題を解決するための要件を定義する。

### 問題の背景

現在の実装では、VSYNCスレッドが`DwmFlush()`でVSync到来を待機し、`WM_VSYNC`カスタムメッセージをメッセージウィンドウにPostMessageで送信している。メインスレッドの`run()`メソッドのメッセージループは`PeekMessageW`でこのメッセージを受信し、`EcsWorld::try_tick_world()`を呼び出して描画を更新する。

しかし、ウィンドウのタイトルバーをドラッグして移動している間、Windows が`DefWindowProcW`内でモーダルループを実行するため、アプリケーションの`run()`メソッドは制御を奪われ、`WM_VSYNC`メッセージを処理できない。

調査により以下が判明した：
- モーダルループ中、`run()`のメッセージループはブロックされる
- ただし`WndProc`は`DefWindowProcW`の内部ループから呼び出される（`WM_WINDOWPOSCHANGED`等）
- この時点で`EcsWorld`は借用されていないため、`WndProc`内から`try_borrow_mut()`でアクセス可能

### 目標

モーダルループ中に`WndProc`から呼ばれる特定メッセージ（`WM_WINDOWPOSCHANGED`等）の処理時に、VSYNCタイミングでのワールド更新を実行できるようにする。最小限の変更で安全性を確保する。

## Requirements

### Requirement 1: VSYNC Tick Count メカニズム

**Objective:** As a フレームワーク開発者, I want VSYNCシグナルをアトミックカウンターで通知できるようにしたい, so that WndProc内からVSYNC到来を検知できる。

#### Acceptance Criteria

1. The VSYNCスレッド shall `WM_VSYNC`メッセージ送信**に加えて**、アトミックなtick_countカウンターをインクリメントする。
2. The tick_countカウンター shall `static AtomicU64`で実装され、グローバルにアクセス可能とする。
3. When VSYNCスレッドがDwmFlush()から復帰したとき, the VSYNCスレッド shall tick_countをインクリメントしてからWM_VSYNCをPostMessageする。
4. The tick_countの値 shall ラップアラウンドに対応し、u64の範囲で安全に比較できる。

### Requirement 2: WndProc内からのVSYNC駆動tick関数

**Objective:** As a フレームワーク開発者, I want WndProc内から安全にVSYNCタイミングでworld tickを呼び出したい, so that モーダルループ中でも描画を更新できる。

#### Acceptance Criteria

1. The フレームワーク shall `EcsWorld`に`try_tick_on_vsync(&mut self) -> bool`メソッドを追加する。このメソッドは以下の順序で処理を行う：
   1. tick_countの変化を検出（前回値`last_vsync_tick`と比較）
   2. 変化があれば`try_tick_world()`を呼び出す
   3. 前回値を更新する
   4. tickが実行されたかどうかを`bool`で返す
2. The フレームワーク shall `VsyncTick`トレイトを定義し、`Rc<RefCell<EcsWorld>>`に実装する。このトレイトは`try_tick_on_vsync(&self) -> bool`メソッドを提供する：
   1. `try_borrow_mut()`でEcsWorldの借用を試みる
   2. 借用成功時、`EcsWorld::try_tick_on_vsync()`を呼び出す
   3. 借用失敗時（再入時）は安全にスキップしてfalseを返す
3. The `EcsWorld` shall `last_vsync_tick: u64`フィールドを持ち、前回処理したtick_count値を保持する。
4. When `try_borrow_mut()`が失敗したとき（再入時）, the トレイト実装 shall 安全にスキップしてfalseを返す。

### Requirement 3: WM_WINDOWPOSCHANGED でのtick呼び出し

**Objective:** As a アプリケーションユーザー, I want ウィンドウをドラッグ移動している間も描画が継続して欲しい, so that スムーズなユーザー体験を得られる。

#### Acceptance Criteria

1. The `ecs_wndproc` shall `WM_WINDOWPOSCHANGED`メッセージ処理の**冒頭**で`try_tick_on_vsync()`を呼び出す。
2. The tick処理 shall 既存のWindowPos/BoxStyleコンポーネント更新処理より**前に**実行される。
3. When モーダルループ中（ウィンドウドラッグ中）, the WM_WINDOWPOSCHANGED shall VSYNCタイミングでworld tickを駆動する。
4. When 通常時（モーダルループ外）, the WM_WINDOWPOSCHANGED shall tick_countが変化していなければスキップする（run()のWM_VSYNCで処理済み）。

### Requirement 4: 既存WM_VSYNC処理の維持

**Objective:** As a フレームワーク開発者, I want 既存のWM_VSYNC処理を変更せず維持したい, so that 安定性を確保できる。

#### Acceptance Criteria

1. The `run()`メソッドのWM_VSYNC処理 shall 従来通り`borrow_mut()`で`try_tick_world()`を呼び出す。
2. The WM_VSYNCメッセージ shall 引き続きVSYNCスレッドからPostMessageで送信される。
3. When 通常動作時（モーダルループ外）, the world tick shall 主にWM_VSYNC処理で実行される。
4. The `try_tick_on_vsync()`とWM_VSYNC処理 shall tick_count値の比較により重複実行を防ぐ。

### Requirement 5: 既存動作との互換性

**Objective:** As a フレームワーク開発者, I want 既存のアプリケーションコードに変更を加えずに新機能を適用したい, so that 移行コストがかからない。

#### Acceptance Criteria

1. The 外部API（`WinThreadMgr::run()`等） shall 既存の呼び出し方法で動作する。
2. The `EcsWorld::try_tick_world()` shall 既存の動作と同等のスケジュール実行を行う。
3. The 既存の`WM_WINDOWPOSCHANGED`処理（WindowPos/BoxStyle更新） shall 影響を受けない。
4. The VSYNCスレッドの停止処理 shall 既存のDrop実装と同様に安全に行われる。

### Requirement 6: デバッグ・診断サポート

**Objective:** As a フレームワーク開発者, I want VSYNC処理の状況を監視できるようにしたい, so that 問題発生時に原因を特定できる。

#### Acceptance Criteria

1. When デバッグビルドのとき, the `try_tick_on_vsync()` shall tickが実行された際にログを出力する（オプション）。
2. The フレームワーク shall 既存のフレームレート計測（`measure_and_log_framerate`）を維持する。
3. Where 診断機能が有効な場合, the フレームワーク shall WndProc経由のtick回数とrun()経由のtick回数を区別して計測できる。

### Requirement 7: 拡張性

**Objective:** As a フレームワーク開発者, I want 今後同様の問題が見つかった際に容易に対応したい, so that 保守性を確保できる。

#### Acceptance Criteria

1. The `try_tick_on_vsync()` shall 汎用的な関数として実装され、他のメッセージ処理からも呼び出し可能とする。
2. When 他のモーダルループ関連メッセージで同様の問題が発見されたとき, the 開発者 shall 該当メッセージ処理に`try_tick_on_vsync()`呼び出しを追加するだけで対応できる。
3. The 実装 shall コメントで設計意図を明記し、今後の拡張ポイントを示す。
