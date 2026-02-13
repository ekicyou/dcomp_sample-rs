# Implementation Plan

## Task Breakdown

### Phase 1: コンポーネント・リソース整備

- [ ] 1. ECS インフラ構築
- [ ] 1.1 (P) WindowDragContextResource の追加
  - Arc<Mutex<WindowDragContext>> 構造体を定義（hwnd, initial_window_pos, move_window, constraint フィールド）
  - ECS Resource として登録可能にする
  - ドラッグ開始時の Window 情報をスレッド間転送する契約を実装
  - _Requirements: 2.3, 2.5_

- [ ] 1.2 (P) WindowDragging マーカーコンポーネントの追加
  - ドラッグ中 Window のマーカーとして機能するコンポーネントを定義
  - Added<WindowDragging> / RemovedComponents<WindowDragging> で状態検知可能にする
  - _Requirements: 2.7_

### Phase 2: WndProc ハンドラ変更

- [ ] 2. WM_WINDOWPOSCHANGED ハンドラの変更
- [ ] 2.1 (P) サイズ変更判定ロジックの実装
  - entity_ref.get::<BoxStyle>() で現在サイズを読み取り、新サイズと比較
  - サイズ変更時のみ get_mut で BoxStyle.size を更新する分岐を追加
  - WindowPos は bypass_change_detection で常時更新（現行維持）
  - _Requirements: 1.2, 1.3, 1.4_

- [ ] 2.2 (P) BoxStyle.inset 書き込みの除去
  - WM_WINDOWPOSCHANGED ハンドラから BoxStyle.inset への書き込みコードを完全削除
  - Window entity の BoxStyle.inset が常に Auto/Px(0.0) であることを保証
  - _Requirements: 1.1, 1.5_

### Phase 3: ドラッグシステム変更

- [ ] 3. ドラッグシステムの WndProc レベル移行
- [ ] 3.1 DragState enum の拡張
  - Dragging バリアントに hwnd, initial_window_pos, move_window, constraint フィールドを追加
  - update_dragging で JustStarted → Dragging 遷移時に WindowDragContextResource から読み取る処理を実装
  - _Requirements: 2.3, 2.5_

- [ ] 3.2 dispatch_drag_events の変更
  - DragTransition::Started 処理で WindowHandle.hwnd, WindowPos.position, DragConfig.move_window, DragConstraint を取得
  - WindowDragContextResource に書き込む処理を追加
  - Window entity に WindowDragging マーカーを insert
  - DragTransition::Ended 処理で WindowPos.position を DerefMut 更新し Changed<WindowPos> を発火
  - WindowDragging マーカーを remove
  - _Requirements: 2.2, 2.3, 2.4, 2.5, 2.7_

- [ ] 3.3 (P) apply_window_drag_movement システムの削除
  - apply_window_drag_movement システムを完全削除
  - Input スケジュールから該当システムを除去
  - DragEvent 発行は dispatch_drag_events で継続されることを確認
  - _Requirements: 2.2_

### Phase 4: WM_MOUSEMOVE ハンドラ変更

- [ ] 4. WM_MOUSEMOVE ハンドラでの直接 SetWindowPos 実装
- [ ] 4.1 Dragging 状態での WndProc レベルウィンドウ移動の実装
  - DRAG_STATE から Dragging 状態の hwnd, initial_window_pos, constraint を取得
  - move_window == true の場合のみ新座標を計算し、DragConstraint を適用
  - guarded_set_window_pos を呼び出してウィンドウを直接移動
  - DragAccumulatorResource への delta 蓄積は維持（DragEvent 発行用）
  - _Requirements: 2.1, 2.2, 2.5_

### Phase 5: レイアウトシステム変更

- [ ] 5. update_arrangements_system の変更
- [ ] 5.1 (P) Window entity の offset スキップロジックの追加
  - クエリに Option<&Window> を追加
  - Window entity の場合、taffy 結果の location を Arrangement.offset に書き込まないロジックを実装
  - Window の Arrangement.size と scale は引き続き taffy 結果で更新
  - _Requirements: 3.5_

### Phase 6: Examples 変更

- [ ] 6. サンプルアプリケーションの初期位置指定変更
- [ ] 6.1 Examples の初期位置を WindowPos に移行
  - taffy_flex_demo.rs, dcomp_demo.rs, areka.rs 等で BoxStyle.inset による初期位置指定を WindowPos.position に変更
  - BoxStyle.inset は Auto または Px(0.0) に統一
  - _Requirements: 1.5_

### Phase 7: テスト実装

- [ ] 7. 統合テストおよび E2E テストの実装
- [ ] 7.1* BoxStyle.inset 不変性テスト
  - WM_WINDOWPOSCHANGED シミュレーション後に Window entity の BoxStyle.inset が変更されていないことを検証
  - _Requirements: 1.1, 1.5_

- [ ] 7.2* Changed<BoxStyle> 発火タイミングテスト
  - 位置のみ変更メッセージで Changed<BoxStyle> が発火しないことを検証
  - サイズ変更メッセージで Changed<BoxStyle> が発火することを検証
  - _Requirements: 1.3, 1.4_

- [ ] 7.3* ドラッグ終了同期テスト
  - ドラッグ終了後に WindowPos.position が最終位置に更新されることを検証
  - sync_window_arrangement_from_window_pos 経由で Arrangement.offset が整合していることを検証
  - _Requirements: 2.4_

- [ ] 7.4 (P) WindowDragging ライフサイクルテスト
  - ドラッグ開始で Added<WindowDragging> が検知されることを検証
  - ドラッグ終了で RemovedComponents<WindowDragging> が検知されることを検証
  - _Requirements: 2.7_

- [ ] 7.5* update_arrangements Window offset スキップテスト
  - taffy レイアウト計算後に Window entity の Arrangement.offset が taffy 結果で上書きされず、WindowPos 由来の値を保持することを検証
  - _Requirements: 3.5_

## Requirements Coverage

| Requirement | Tasks |
|-------------|-------|
| 1.1 | 2.2, 7.1 |
| 1.2 | 2.1 |
| 1.3 | 2.1, 7.2 |
| 1.4 | 2.1, 7.2 |
| 1.5 | 2.2, 6.1, 7.1 |
| 2.1 | 4.1 |
| 2.2 | 3.2, 3.3, 4.1 |
| 2.3 | 1.1, 3.1, 3.2 |
| 2.4 | 3.2, 7.3 |
| 2.5 | 1.1, 3.1, 3.2, 4.1 |
| 2.6 | (Req 1 に依存) |
| 2.7 | 1.2, 3.2, 7.4 |
| 3.1 | (既存) |
| 3.2 | (既存) |
| 3.3 | (既存) |
| 3.4 | (既存) |
| 3.5 | 5.1, 7.5 |
| 4.1 | (変更なし) |
| 4.2 | (変更なし) |

## Task Progression

1. **Phase 1-2 (並列実行可能)**: インフラ整備（WindowDragContextResource, WindowDragging）と WM_WINDOWPOSCHANGED ハンドラ変更、update_arrangements_system 変更は並列実行可能
2. **Phase 3**: ドラッグシステム変更（Phase 1 の WindowDragContextResource に依存）
3. **Phase 4**: WM_MOUSEMOVE ハンドラ変更（Phase 3 の DragState 拡張に依存）
4. **Phase 5**: Phase 1-4 完了後に Examples 変更
5. **Phase 6**: すべての実装完了後にテスト実装

---

**Note**: `- [ ]*` マークは MVP 後に延期可能なテストを示す。
