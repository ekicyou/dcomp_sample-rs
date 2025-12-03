# Requirements Document

## Introduction

本仕様は、`taffy_flex_demo`サンプルアプリケーションを`EcsWorld::spawn`による非同期コマンド発行パターンにリファクタリングするものである。現行の`thread::spawn` + `mpsc::channel`による同期的なコマンド送信方式を、wintfフレームワークが提供する`EcsWorld::spawn` APIを活用した非同期パターンに置き換え、コードをスリム化する。

## Project Description (Input)

taffy_flex_demoデモで、EcsWorld::spawnの非同期実行に改変する。
EcsWorld::spawnでは、async fnが登録可能であり、更にワールドに対してコマンド発行可能である。
そこで、0秒時の処理後に非同期に5秒待機し、、、と繰り返して10秒までの処理をするように
taffy_flex_demoを変更してスリム化する。

最初に「taffy_flex_demo_old.rs」いう名前でオリジナルを残し、
新たに「taffy_flex_demo.rs」を開発する。

## Requirements

### Requirement 1: オリジナルファイルの保存

**Objective:** As a 開発者, I want オリジナルの`taffy_flex_demo.rs`を別名で保存する, so that 既存の実装を参照・比較できる

#### Acceptance Criteria

1. The demo shall オリジナルの`taffy_flex_demo.rs`を`taffy_flex_demo_old.rs`としてコピー保存する
2. The demo shall `taffy_flex_demo_old.rs`は変更せずそのまま維持する

### Requirement 2: 非同期パターンへの移行

**Objective:** As a 開発者, I want `thread::spawn` + `mpsc::channel`を`EcsWorld::spawn`に置き換える, so that wintfの非同期コマンド発行パターンを活用できる

#### Acceptance Criteria

1. When デモが起動された時, the demo shall `EcsWorld::spawn`を使用して非同期タスクを登録する
2. The demo shall `std::thread::spawn`と`std::sync::mpsc::channel`を使用しない
3. The demo shall `CommandSender`を通じてECSコマンドを送信する
4. The demo shall 専用のUpdateシステム登録（コマンド処理用）を削除する

### Requirement 3: タイムライン制御

**Objective:** As a 開発者, I want 0秒→5秒→10秒のタイムライン処理を非同期で実行する, so that シーケンシャルな処理フローを維持する

#### Acceptance Criteria

1. When デモが起動された時, the demo shall 0秒時点でFlexboxデモウィンドウを作成する
2. When 0秒の処理が完了した後, the demo shall 非同期に5秒待機する
3. When 5秒経過した時, the demo shall レイアウトパラメーターを変更する
4. When 5秒の処理が完了した後, the demo shall 非同期に5秒待機する
5. When 10秒経過した時, the demo shall ウィンドウを閉じる

### Requirement 4: 非同期待機の実装

**Objective:** As a 開発者, I want `async_io::Timer`等の非同期タイマーを使用する, so that メインスレッドをブロックせずに待機できる

#### Acceptance Criteria

1. The demo shall `std::thread::sleep`の代わりに非同期タイマーを使用する
2. The demo shall 非同期待機中もECSのtick処理を継続可能とする
3. While 非同期タスク実行中, the demo shall メインスレッドのメッセージループをブロックしない

### Requirement 5: コードのスリム化

**Objective:** As a 開発者, I want 不要なボイラープレートコードを削除する, so that コードの可読性が向上する

#### Acceptance Criteria

1. The demo shall `WorldCommand`型エイリアスを削除する
2. The demo shall `Mutex<Receiver<WorldCommand>>`を削除する
3. The demo shall タイマースレッドのspawn/joinロジックを削除する
4. The demo shall `add_systems`によるコマンド処理システム登録を削除する

### Requirement 6: 機能の維持

**Objective:** As a 開発者, I want リファクタリング後も同じ視覚的動作を維持する, so that 機能の退行が発生しない

#### Acceptance Criteria

1. The demo shall Flexboxレイアウトのウィンドウを表示する（灰色背景、マージン10px）
2. The demo shall 赤・緑・青の3つの矩形をFlexコンテナ内に表示する
3. The demo shall 赤ボックスの子として画像（BitmapSource）を表示する
4. When 5秒経過した時, the demo shall ウィンドウ位置・サイズを変更する（-500,400 / 600x400）
5. When 5秒経過した時, the demo shall FlexContainerの方向をColumnに変更する
6. When 5秒経過した時, the demo shall 各矩形のサイズ・grow値を変更する
7. When 10秒経過した時, the demo shall Windowエンティティをdespawnしてアプリを終了する

