# Implementation Plan

## Task 1: 座標変換メソッドの実装

- [x] 1.1 WindowPosにto_window_coordsメソッドを追加
  - クライアント領域の座標・サイズをウィンドウ全体座標に変換するメソッドを実装
  - HWNDを引数に取り、変換後の(x, y, width, height)タプルをResult型で返す
  - メソッドシグネチャ: `pub fn to_window_coords(&self, hwnd: HWND) -> Result<(i32, i32, i32, i32), String>`
  - _Requirements: 1_

- [x] 1.2 (P) Win32 APIによるスタイル・DPI情報取得の実装
  - GetWindowLongPtrWでウィンドウスタイル(GWL_STYLE)を取得
  - GetWindowLongPtrWで拡張スタイル(GWL_EXSTYLE)を取得
  - GetDpiForWindowでウィンドウのDPI値を取得
  - 各API呼び出し失敗時はエラーメッセージを含むErrを返す
  - _Requirements: 2_

- [x] 1.3 AdjustWindowRectExForDpiによる座標変換の実装
  - クライアント領域座標からRECT構造体を構築
  - AdjustWindowRectExForDpiを呼び出してウィンドウ全体矩形を計算
  - 変換後のRECTから(x, y, width, height)を抽出して返す
  - API失敗時はエラーメッセージを返す
  - _Requirements: 1, 2_

## Task 2: システム統合

- [x] 2.1 apply_window_pos_changesシステムへの座標変換統合
  - SetWindowPos呼び出し前にto_window_coordsを呼び出す
  - 変換成功時は変換後の座標・サイズでSetWindowPosを実行
  - 変換失敗時は元の座標・サイズでSetWindowPosを実行（フォールバック）
  - _Requirements: 1, 3_

- [x] 2.2 エコーバックメカニズムの更新
  - last_sent_positionとlast_sent_sizeを変換後の値で更新
  - 既存のis_echo判定ロジックが正常に動作することを確認
  - WM_WINDOWPOSCHANGEDで受信する座標と照合できるようにする
  - _Requirements: 3_

- [x] 2.3 (P) CW_USEDEFAULT特殊値のハンドリング
  - position/sizeにCW_USEDEFAULTが含まれる場合は座標変換をスキップ
  - 既存のCW_USEDEFAULTチェックロジックを変換前に配置
  - スキップ時は元の値でSetWindowPosを呼び出す
  - _Requirements: 4_

- [x] 2.4 (P) エラーログ出力の実装
  - 座標変換失敗時にeprintln!でエラーメッセージを出力
  - エラーメッセージにはHWND情報と失敗理由を含める
  - フォーマット: "Failed to transform window coordinates: {詳細}. Using original values."
  - _Requirements: 3_

## Task 3: テスト実装

- [x] 3.1 WS_OVERLAPPEDWINDOWスタイルでの座標変換テスト
  - 標準ウィンドウスタイルでto_window_coordsの動作を検証
  - 入力: クライアント領域(100, 100, 800, 600)
  - 期待: タイトルバー・ボーダー分が加算された座標が返る
  - _Requirements: 1, 5_

- [x] 3.2 WS_POPUPスタイルでの座標変換テスト【必須】
  - ボーダーレスウィンドウでの変換動作を検証
  - 入力: クライアント領域(100, 100, 800, 600)、スタイル=WS_POPUP
  - 期待: 装飾なしのため入力座標と同一の値が返る
  - スタイル・拡張スタイルの組み合わせによる領域差異を確認
  - _Requirements: 1, 2_

- [x] 3.3 (P) エラーハンドリングテスト
  - 無効なHWND(HWND(0))を渡した場合のテスト
  - 期待: Errが返され、適切なエラーメッセージが設定される
  - _Requirements: 3_

- [x] 3.4 エコーバックメカニズムの統合テスト
  - WindowPosを更新後、WM_WINDOWPOSCHANGED受信時の挙動を検証
  - is_echo判定が正常に動作し、重複SetWindowPos呼び出しが発生しないことを確認
  - last_sent_*が変換後の値と一致することを確認
  - _Requirements: 3_

## Task 4: E2E検証

- [x] 4.1 taffy_flex_demoでの動作確認
  - サンプルアプリケーションを起動してウィンドウ配置を目視確認
  - クライアント領域が指定座標(100, 100)に配置されることを確認
  - ウィンドウサイズが指定サイズ(800x600)のクライアント領域を持つことを確認
  - タイトルバーが画面外に出ないことを確認
  - _Requirements: 5_
  - **検証結果**: E2E実行で `location=(100, 100), size=(800, 600)` を確認

## Task 5: デバッグ（Taffyが(0,0)を返す問題の調査）- SKIPPED

> **Note**: この問題は LayoutRoot 初期化をワールド作成時に移動することで解決済み。
> 根本原因: Taffy はルートノードの location を (0,0) として返す仕様。
> 解決策: Window を LayoutRoot の子として配置し、BoxInset で位置を指定。

- [~] 5.1 デバッグログの追加 - SKIPPED (問題解決済み)
- [~] 5.2 問題箇所の特定と修正 - SKIPPED (問題解決済み)
- [~] 5.3 デバッグログの削除 - SKIPPED (問題解決済み)
