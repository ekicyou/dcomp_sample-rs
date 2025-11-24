# Implementation Plan

## Task Overview

本実装では、4つの新規ECSシステムをPostLayoutスケジュールに追加し、レイアウト計算結果をDirectCompositionグラフィックスリソースとウィンドウシステムに伝播させる完全な同期機構を確立します。

## Tasks

### Phase 1: 準備とクリーンアップ

- [ ] 1. コンポーネント拡張とレガシーコード削除
- [ ] 1.1 (P) Visualコンポーネントに変更検知最適化を追加
  - `Visual`構造体に`#[derive(PartialEq)]`を追加
  - すべてのフィールド（`is_visible`, `opacity`, `transform_origin`, `size`）が比較対象となることを確認
  - 同じ値を設定した場合に`Changed<Visual>`フラグが立たないことを検証
  - _Requirements: 8_

- [ ] 1.2 (P) WindowPosコンポーネントにエコーバック検知フィールドを追加
  - `WindowPos`構造体に`last_sent_position: Option<(i32, i32)>`を追加
  - `WindowPos`構造体に`last_sent_size: Option<(i32, i32)>`を追加
  - `is_echo(position: POINT, size: SIZE) -> bool`メソッドを実装
  - エコーバック判定ロジック（送信値と受信値の厳密一致）を実装
  - _Requirements: 4, 10_

- [ ] 1.3 冗長なinit_window_surfaceシステムを削除
  - `crates/wintf/src/ecs/world.rs`のPostLayoutスケジュール登録からinit_window_surfaceを削除
  - `crates/wintf/src/ecs/graphics/systems.rs`のinit_window_surface関数定義を削除
  - create_surface_for_windowヘルパー関数は保持（resize_surface_from_visualで再利用）
  - 既存テストスイートを実行し、regression確認
  - _Requirements: 7_

- [ ] 1.4 (P) レガシーなウィンドウサイズ取得コードを削除
  - `window_system.rs`のcreate_windowsシステム内のGetClientRect呼び出しを削除
  - Visual.sizeへの直接代入コードを削除
  - Visualコンポーネントをデフォルト値で作成（実際のサイズはsync_visual_from_layout_rootで計算）
  - _Requirements: 7_

### Phase 2: 順方向同期システム実装

- [ ] 2. Layout→Graphics→Windowの伝播チェーンを構築
- [ ] 2.1 GlobalArrangementからVisualへのサイズ同期を実装
  - sync_visual_from_layout_rootシステムを実装
  - LayoutRootマーカーを持つエンティティのみ処理
  - GlobalArrangement.boundsから幅と高さを計算しVisual.sizeを更新
  - `Changed<GlobalArrangement>`クエリで変更検知最適化を活用
  - PostLayoutスケジュールのpropagate_global_arrangements直後に登録
  - _Requirements: 1, 9_

- [ ] 2.2 Visualサイズ変更時のSurface再作成を実装
  - resize_surface_from_visualシステムを実装
  - Visual.sizeとSurfaceGraphics.sizeを比較し、異なる場合のみ再作成
  - create_surface_for_windowヘルパーを活用してSurface生成
  - Surface再作成失敗時はinvalidate()で無効化
  - PostLayoutスケジュールのsync_visual_from_layout_root直後に登録
  - _Requirements: 2, 9_

- [ ] 2.3 レイアウト結果をWindowPosに同期
  - sync_window_posシステムを実装
  - Windowマーカーを持つエンティティのみ処理
  - GlobalArrangement.boundsからpositionを設定
  - Visual.sizeからsizeを設定
  - `Or<(Changed<GlobalArrangement>, Changed<Visual>)>`クエリで両方の変更を検知
  - PostLayoutスケジュールのresize_surface_from_visual直後に登録
  - _Requirements: 3, 9_

- [ ] 2.4 WindowPos変更をSetWindowPosに反映
  - apply_window_pos_changesシステムを実装
  - エコーバックチェック（is_echo()メソッド使用）でスキップ判定
  - 既存のWindowPos::set_window_pos()メソッドを呼び出し
  - SetWindowPos成功時にbypass_change_detection()でlast_sent値を記録
  - 失敗時はエラーログのみ（last_sent更新なし）
  - PostLayoutスケジュールのsync_window_pos直後に登録（最後のシステム）
  - _Requirements: 5, 9_

### Phase 3: 逆方向同期とメッセージハンドリング

- [ ] 3. Window→Layoutの逆方向伝播を実装
- [ ] 3.1 WM_WINDOWPOSCHANGEDメッセージハンドラーを追加
  - WinMessageHandler traitにWM_WINDOWPOSCHANGEDメソッド定義を追加
  - デフォルト実装はNoneを返す（既存パターン準拠）
  - メッセージディスパッチャーに1行追加（WM_WINDOWPOSCHANGED case）
  - _Requirements: 6_

- [ ] 3.2 エコーバック検知と外部変更処理を実装
  - WM_WINDOWPOSCHANGEDハンドラー内で受信値とlast_sent値を比較
  - エコーバック検知時は処理をスキップ（DefWindowProc呼び出しなし）
  - 外部変更時にWindowPosとBoxSizeを更新（ECS World経由）
  - 既存のEntity-HWNDマッピング機構を活用
  - BoxSize更新によりTaffyレイアウト再計算をトリガー
  - _Requirements: 6, 9_

- [ ] 3.3 (P) レガシーなWM_MOVE/WM_SIZEハンドラーを削除
  - WM_MOVEメッセージハンドラーを削除
  - WM_SIZEメッセージハンドラーを削除
  - WM_WINDOWPOSCHANGEDで代替されることを確認
  - _Requirements: 7_

### Phase 4: テストとバリデーション

- [ ] 4. 同期システムの品質を検証
- [ ] 4.1 (P) エコーバック判定ロジックの単体テストを作成
  - WindowPos::is_echo()メソッドの単体テスト
  - 送信値と受信値が一致する場合のテスト
  - 送信値と受信値が異なる場合のテスト
  - _Requirements: 10_

- [ ] 4.2 (P) 座標変換ロジックの単体テストを作成
  - GlobalArrangement.bounds→Visual.sizeの変換テスト
  - GlobalArrangement.bounds→WindowPos.positionの変換テスト
  - Visual.size→WindowPos.sizeの変換テスト
  - _Requirements: 10_

- [ ] 4.3* 順方向フロー統合テストを作成
  - Layout→Visual→Surface→WindowPosの完全フロー検証
  - Changed検知が正しく連鎖することを確認
  - Surface再作成が正しくトリガーされることを確認
  - SetWindowPosが正しく呼び出されることを確認
  - _Requirements: 9, 10_

- [ ] 4.4* エコーバック検知の統合テストを作成
  - 順方向フロー後のエコーバックスキップを検証
  - last_sent値が正しく記録されることを確認
  - エコーバック時にWindowPos更新がスキップされることを確認
  - _Requirements: 9, 10_

- [ ] 4.5* 逆方向フロー統合テストを作成
  - WM_WINDOWPOSCHANGED→WindowPos→BoxSizeの伝播検証
  - 外部変更時にBoxSize更新がトリガーされることを確認
  - レイアウト再計算が正しく動作することを確認
  - _Requirements: 9, 10_

- [ ] 4.6 taffy_flex_demoサンプルで手動検証
  - Taffy計算結果（800×600）がSurfaceサイズに正確に反映されることを確認
  - Green rectangleの高さ（45px）が正しく表示されることを確認
  - ユーザーによるウィンドウリサイズ時に無限ループが発生しないことを確認
  - レイアウトシステム起点のウィンドウリサイズ時に無限ループが発生しないことを確認
  - _Requirements: 9, 10_

- [ ] 4.7* Visual PartialEq最適化の効果測定テストを作成
  - 同じ値を設定した場合に`Changed<Visual>`フラグが立たないことを確認
  - 不要なシステム実行回数が削減されることを測定
  - _Requirements: 8, 10_

## Task Statistics

- **Total**: 4 major tasks, 18 sub-tasks
- **Parallel-capable**: 7 sub-tasks marked with (P)
- **Optional test coverage**: 5 sub-tasks marked with *
- **Requirements coverage**: All 10 requirements mapped

## Notes

- Phase 1とPhase 2の一部タスク（1.1, 1.2, 1.4, 4.1, 4.2, 4.7）は並列実行可能（ファイル競合なし）
- Phase 3は既存のメッセージハンドリング機構への統合が必要（Entity-HWNDマッピング調査）
- テストタスク（4.3-4.5, 4.7）はMVP後に延期可能（オプション）、4.6は手動検証のため必須
