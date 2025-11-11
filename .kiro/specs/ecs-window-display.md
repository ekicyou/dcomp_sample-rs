# ECS Window Display

**Status:** Completed  
**Created:** 2025-11-11  
**Last Updated:** 2025-11-11

## Requirements

### 目的
ECSシステムを使用してWindowを表示し、閉じるまでの完全な実装を行う。ユーザーがWindowを閉じたらアプリケーションが正常に終了するようにする。

### 要件

#### 機能要件
1. **Window表示**
   - ECSの`Window`コンポーネントを持つEntityを作成するだけでウィンドウが表示される
   - Windowのタイトル、サイズ、位置、スタイルを設定可能

2. **Window終了処理**
   - ユーザーがXボタンをクリックしたら`WM_CLOSE` → `DestroyWindow`
   - `WM_DESTROY`で`PostQuitMessage(0)`を呼び出し
   - メッセージループがWM_QUITを受け取ってクリーンに終了

3. **ECS統合**
   - `Window`コンポーネント: ウィンドウ作成パラメータ（title, style, size等）
   - `WindowHandle`コンポーネント: 作成後の確定情報（hwnd, instance, initial_dpi）
   - `create_windows`システムが自動的にウィンドウを作成

4. **実行可能なサンプル**
   - `simple_window.rs`: 最小限のECSウィンドウサンプル
   - 実行すると空のWindowが表示される
   - Windowを閉じるとアプリケーションが終了する

#### 非機能要件
- メインスレッドでウィンドウ作成（Windows APIの制約）
- メモリリークなし
- エラーハンドリングの適切な実装
- 既存コード（dcomp_demo）への影響なし

### 成功基準
- ✅ `cargo run --example simple_window` でWindowが表示される
- ✅ Windowのタイトルバーに指定した文字列が表示される
- ✅ WindowのXボタンをクリックするとアプリケーションが終了する
- ✅ メモリリークやクラッシュが発生しない
- ✅ cargo buildで警告が出ない

### 除外事項
- 描画機能（フェーズ2で実装）
- 複数Window対応（当面は単一Window）
- リサイズやDPI変更の完全な対応（基盤のみ）

## Design

### アーキテクチャ概要

**コンポーネント設計：**
- **`Window`コンポーネント**: ウィンドウ作成パラメータ（title, style, ex_style, x, y, width, height, parent）
- **`WindowHandle`コンポーネント**: 作成後の情報（hwnd, instance, initial_dpi）
  - システムによって自動的に追加される
  - `Window`があるが`WindowHandle`がない = 未作成
  - 両方ある = 作成済み

**システム設計：**
- **`create_windows`システム**: 
  - クエリ: `Query<(Entity, &Window), Without<WindowHandle>>`
  - 未作成のウィンドウを検出して`CreateWindowExW`で作成
  - `WindowHandle`コンポーネントを追加
  - `ShowWindow`で表示

**ウィンドウプロシージャ：**
- **`ecs_wndproc`**: ECS専用のウィンドウプロシージャ
  - `hwnd → Entity`のマッピングを保持
  - `WM_NCCREATE`でマッピング登録
  - `WM_CLOSE` → `DestroyWindow`
  - `WM_DESTROY` → `PostQuitMessage(0)`
  - `WM_PAINT`, `WM_ERASEBKGND`等の基本メッセージ処理

**重要な実装決定：**
1. **シングルスレッド実行**: `ExecutorKind::SingleThreaded`
   - Bevy ECSはデフォルトでマルチスレッド並列実行
   - Windowsのウィンドウは作成したスレッドのメッセージキューに紐付けられる
   - メインスレッド以外で作成されたウィンドウはメッセージループで処理できない
   - **解決策**: Scheduleをシングルスレッドに設定してメインスレッドで実行

2. **ウィンドウクラスの分離**:
   - 既存: `wintf_window_class` → 古い`wndproc`（dcomp_demo等）
   - 新規: `wintf_ecs_window_class` → `ecs_wndproc`（ECS方式）

### データフロー

```
main()
  → WinThreadMgr::new()
  → world.spawn(Window { ... })
  → mgr.run()
    → メッセージループ開始
    → WM_TIMER → ECS tick
      → create_windows システム実行（メインスレッド）
        → CreateWindowExW (with Entity ID)
        → WM_NCCREATE → hwnd→Entity マッピング登録
        → WindowHandle追加
        → ShowWindow
    → ユーザーがXボタンをクリック
      → WM_CLOSE → DestroyWindow
      → WM_DESTROY → PostQuitMessage(0)
      → WM_QUIT → ループ終了
  ← アプリケーション終了
```

## Tasks

### タスク1: Windowコンポーネントの設計 ✅
**ファイル**: `crates/wintf/src/ecs/window.rs`
- `Window`コンポーネント: ウィンドウ作成パラメータを保持
- `WindowHandle`コンポーネント: 作成後の情報（hwnd, instance, initial_dpi）
- マーカーではなく、実際のデータを保持

### タスク2: ECS専用のwndprocを実装 ✅
**ファイル**: `crates/wintf/src/ecs/window.rs`
- `ecs_wndproc`を作成
- `hwnd → Entity`マッピングの管理
- `WM_NCCREATE`でマッピング登録
- `WM_CLOSE`, `WM_DESTROY`, `WM_PAINT`等の処理

### タスク3: Window作成システムを実装 ✅
**ファイル**: `crates/wintf/src/ecs/window.rs`
- `create_windows`システム
- `Query<(Entity, &Window), Without<WindowHandle>>`で未作成を検出
- `CreateWindowExW`でウィンドウ作成（EntityのIDを渡す）
- `WindowHandle`を追加
- `ShowWindow`で表示

### タスク4: シングルスレッド実行を設定 ✅
**ファイル**: `crates/wintf/src/ecs/world.rs`
- `schedule.set_executor_kind(ExecutorKind::SingleThreaded)`
- メインスレッドでウィンドウ作成を保証

### タスク5: ECS用ウィンドウクラスを登録 ✅
**ファイル**: `crates/wintf/src/process_singleton.rs`
- `wintf_ecs_window_class`を追加
- `ecs_wndproc`を使用
- 既存のウィンドウクラスと共存

### タスク6: サンプル実装の作成 ✅
**ファイル**: `crates/wintf/examples/simple_window.rs` (新規作成)
- ECS方式でWindowを作成
- タイトル: "wintf - ECS Window"
- サイズ: 800x600

### タスク7: 動作確認とデバッグ ✅
- スレッドIDのログ出力でマルチスレッド問題を発見
- シングルスレッド実行で解決
- Xボタンでの正常終了を確認
- 既存の`dcomp_demo`への影響がないことを確認

## Implementation Log

### 実装完了 (2025-11-11)

#### 変更ファイル

**1. `crates/wintf/src/ecs/window.rs`**
- `Window`コンポーネントを再設計: マーカーではなくパラメータを保持
- `WindowHandle`コンポーネントを追加: 作成後の情報
- `ecs_wndproc`を実装: ECS専用のウィンドウプロシージャ
- `hwnd → Entity`マッピングの管理（グローバルHashMap）
- `create_windows`システムを実装

**2. `crates/wintf/src/ecs/world.rs`**
- `ExecutorKind::SingleThreaded`を設定
- デフォルトで`create_windows`システムを登録

**3. `crates/wintf/src/process_singleton.rs`**
- `wintf_ecs_window_class`を追加
- `ecs_wndproc`を使用
- 既存のウィンドウクラスと共存

**4. `crates/wintf/src/win_ecs.rs`**
- `EcsWindow::set_hwnd`を更新: `WindowHandle`を使用

**5. `crates/wintf/examples/simple_window.rs`** (新規作成)
- ECS方式の最小限サンプル

#### 主要な技術的課題と解決策

**課題1: メッセージがwndprocに届かない**
- **原因**: Bevy ECSがマルチスレッドで実行し、ウィンドウが別スレッドで作成されていた
- **調査**: スレッドIDをログ出力して発見
  - SYSTEM: ThreadId(11), ThreadId(12) 等
  - MSG LOOP: ThreadId(1)
  - WNDPROC: ThreadId(11)
- **解決**: `schedule.set_executor_kind(ExecutorKind::SingleThreaded)`ですべてメインスレッドで実行

**課題2: `&World`パラメータでメインスレッド固定できるか？**
- **検証**: `&World`を引数に追加してテスト
- **結果**: 排他的になるだけで、スレッドは固定されない
- **結論**: `ExecutorKind::SingleThreaded`が正解

#### 動作確認結果
- ✅ `cargo run --example simple_window`でWindowが表示される
- ✅ Windowのタイトルバーに"wintf - ECS Window"が表示される
- ✅ WindowのXボタンをクリックするとアプリケーションが正常に終了する
- ✅ `dcomp_demo`への影響なし（既存コードは正常にビルド・動作）
- ✅ フレームレート: 約62 FPS
- ✅ メモリリークなし
- ✅ ビルド警告なし

#### 学んだこと
1. **Windowsのウィンドウはスレッドに紐付けられる**
   - 作成したスレッドのメッセージキューにメッセージが送られる
   - メインスレッド以外で作成すると、メインのメッセージループで処理できない

2. **Bevy ECSのマルチスレッド実行**
   - デフォルトで並列実行される
   - Windows GUIには不向き
   - `ExecutorKind::SingleThreaded`で解決

3. **`&World`パラメータは排他的にするだけ**
   - スレッド固定の効果はない
   - 並列実行を制限するだけ

#### 今後の拡張ポイント
- イベント処理システム（マウス、キーボード等）
- 複数ウィンドウ対応
- DPI変更の完全対応
- リサイズ処理
