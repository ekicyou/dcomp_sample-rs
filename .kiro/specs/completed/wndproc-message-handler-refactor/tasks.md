# Implementation Plan

## Task Overview

`ecs_wndproc`のメッセージハンドラをディレクトリベースのモジュール構造に分離するリファクタリング。

---

## Tasks

- [x] 1. モジュール構造の変換
  - [x] 1.1 ディレクトリ構造の作成
    - `window_proc/`ディレクトリを作成し、`mod.rs`を配置する
    - 既存の`window_proc.rs`の内容を`mod.rs`に移動する
    - ECS World管理コード（`SendWeak`, `ECS_WORLD`, `set_ecs_world`, `try_get_ecs_world`）を保持する
    - `mod handlers;`宣言を追加する
    - _Requirements: 8.1, 8.3_
  - [x] 1.2 (P) ハンドラモジュールの作成
    - `handlers.rs`ファイルを作成する
    - モジュールレベルで`#![allow(non_snake_case)]`を設定する
    - 必要なインポート（`windows`, `tracing`）を追加する
    - `HandlerResult`型エイリアスを定義する
    - _Requirements: 8.2, 2.2_

- [x] 2. シンプルなハンドラ関数の実装
  - [x] 2.1 (P) 即値を返すハンドラの実装
    - `WM_ERASEBKGND`: `Some(LRESULT(1))`を返す
    - `WM_PAINT`: `ValidateRect`呼び出し後`Some(LRESULT(0))`を返す
    - `WM_CLOSE`: `DestroyWindow`呼び出し後`Some(LRESULT(0))`を返す
    - 各関数に`#[inline]`属性と`pub(super)`可視性を付与する
    - _Requirements: 1.2, 2.1, 3.1, 4.1, 6.3, 6.4, 6.5, 7.1, 8.4, 8.5_
  - [x] 2.2 (P) Entity管理ハンドラの実装
    - `WM_NCCREATE`: `CREATESTRUCTW`から`lpCreateParams`を取得し`GWLP_USERDATA`に保存、`None`を返す
    - `WM_NCDESTROY`: `get_entity_from_hwnd`でEntity取得、ECS Worldから削除、`GWLP_USERDATA`クリア、`None`を返す
    - 各関数に`#[inline]`属性と`pub(super)`可視性を付与する
    - _Requirements: 1.2, 2.1, 3.1, 4.1, 6.1, 6.2, 7.1, 8.4, 8.5_

- [x] 3. 複雑なハンドラ関数の実装
  - [x] 3.1 WM_WINDOWPOSCHANGEDハンドラの実装
    - ハンドラ内で`try_get_ecs_world()`を呼び出してECS Worldを取得する
    - 第1借用セクション: DPI更新、WindowPosChanged=true、WindowPos/BoxStyle更新
    - `try_tick_on_vsync()`呼び出し
    - `flush_window_pos_commands()`呼び出し
    - 第2借用セクション: WindowPosChanged=false
    - `None`を返す
    - `#[inline]`属性と`pub(super)`可視性を付与する
    - _Requirements: 1.2, 2.1, 3.1, 4.1, 6.6, 7.1, 8.4, 8.5_
  - [x] 3.2 (P) WM_DISPLAYCHANGEハンドラの実装
    - ハンドラ内で`try_get_ecs_world()`を呼び出す
    - `App`リソースの`mark_display_change()`を呼び出す
    - `None`を返す
    - `#[inline]`属性と`pub(super)`可視性を付与する
    - _Requirements: 1.2, 2.1, 3.1, 4.1, 6.7, 7.1, 8.4, 8.5_
  - [x] 3.3 (P) WM_DPICHANGEDハンドラの実装
    - `DPI::from_WM_DPICHANGED`で新DPIを取得する
    - `lparam`から`suggested_rect`を取得する
    - `DpiChangeContext`をスレッドローカルに設定する
    - `SetWindowPos`を呼び出す
    - `Some(LRESULT(0))`を返す
    - `#[inline]`属性と`pub(super)`可視性を付与する
    - _Requirements: 1.2, 2.1, 3.1, 4.1, 6.8, 7.1, 8.4, 8.5_

- [x] 4. ディスパッチャと公開API
  - [x] 4.1 ecs_wndprocのリファクタリング
    - `match`式を各ハンドラ関数呼び出しに変換する
    - ワイルドカードパターンで`None`を返す
    - `unwrap_or_else`で`DefWindowProcW`を一元的に呼び出す
    - `WM_NCHITTEST`の個別パターンを削除（ワイルドカードに委譲）
    - _Requirements: 1.1, 1.3, 3.2, 3.3, 5.1, 5.2, 5.3, 5.4_
  - [x] 4.2 公開API関数の可視性変更
    - `set_ecs_world`を`pub(crate)`に変更し`#[inline]`を付与する
    - `try_get_ecs_world`を`pub(super)`に変更する
    - `get_entity_from_hwnd`を`pub(crate)`に変更し`#[inline]`を付与する
    - `ecs_wndproc`を`pub(crate)`に変更する
    - _Requirements: 9.1, 9.2, 9.3_

- [x] 5. ビルド検証と動作確認
  - [x] 5.1 ビルドとテストの実行
    - `cargo build`でコンパイルエラーがないことを確認する
    - `cargo test --all-targets`で既存テストが通過することを確認する
    - `ecs/mod.rs`の再エクスポート（`pub use window_proc::*`）が正常に動作することを確認する
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7, 6.8, 7.2_
  - [x] 5.2 サンプルアプリケーションの動作確認
    - `cargo run --example taffy_flex_demo`でウィンドウが正常に表示・操作できることを確認する
    - ウィンドウの移動・リサイズ・クローズが正常に動作することを確認する
    - マルチモニター環境でのDPI変更が正常に動作することを確認する（可能な場合）
    - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5, 6.6, 6.7, 6.8_
