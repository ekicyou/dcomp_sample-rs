# Implementation Plan

## タスクフォーマットテンプレート

このドキュメントは、event-drag-systemの実装タスクを定義する。

---

## 実装状況メモ

### 現在の問題（2025-12-08）
- thread_local DragStateがwndprocスレッドとECSスレッドで共有されない問題が発覚
- wndprocでPreparing→JustStarted遷移してもECSスレッドからは見えない
- 結果：ドラッグイベントがとびとびにしか発火しない（イベントロス）

### 新設計方針
- **wndprocスレッド**: thread_local DragStateで状態管理 + デルタを累積
- **ECSスレッド**: 累積量をflushしてイベント配信
- **データ転送**: DragAccumulatorResourceをECSワールドリソースとして共有（Arc<Mutex>）

### 更新されたタスク
- Phase 2を「ドラッグ累積器とスレッド間転送」に変更
- wndprocでの累積処理とECS側でのflush処理を明確化

---

## タスク一覧

### Phase 1: Phase<T>ジェネリック化とリグレッション対策

- [x] 1. Phase<T>ジェネリック関数への変換とPointerEvent回帰テスト
- [x] 1.1 pointer/dispatch.rsをPhase<T>ジェネリック関数化
  - 既存dispatch_pointer_eventsをPhase<T>ジェネリック関数に変換
  - OnPointerDown/OnPointerMove/OnPointerUp/OnPointerEnter/OnPointerLeaveの5種類でPhase<T>配信を確認
  - 型推論エラーは型注釈で解決
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5_
  - **STATUS: 完了（コミット済み）**
  
- [x] 1.2 PointerEventの既存動作リグレッションテスト
  - areka.rsサンプルで既存PointerEvent配信が正常動作することを確認
  - Tunnel/Bubbleフェーズログ出力が正しい順序であることを確認
  - dcomp_demo.rsサンプルで既存動作が維持されることを確認
  - コミット前に全サンプルが正常実行できることを保証
  - _Requirements: 7.1, 7.2, 7.3, 7.4_
  - **STATUS: 完了（コミット済み）**

### Phase 2: ドラッグ累積器とスレッド間転送

- [ ] 2. ドラッグ累積器の実装とスレッド間データ転送
- [ ] 2.1 DragAccumulatorとDragAccumulatorResource定義
  - DragAccumulator構造体（accumulated_delta, pending_transition）を定義
  - DragTransition enum（Started/Ended）を定義
  - DragAccumulatorResource（Arc<Mutex<DragAccumulator>>）をECSリソースとして定義
  - accumulate_delta(), set_transition(), flush()メソッドを実装
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 13.1, 13.2_
  - **FILE: crates/wintf/src/ecs/drag/accumulator.rs (新規作成)**

- [ ] 2.2 DragStateとPhysicalPoint定義（thread_local専用）
  - DragState enum（Idle/Preparing/Dragging）をthread_local! + RefCellで実装
  - PhysicalPoint構造体を定義（x, y: i32）
  - DragStateがEntity、開始位置、現在位置を保持する構造を設計
  - 単一DragState（複数ボタン同時ドラッグ禁止）を明確化
  - update_drag_state(), read_drag_state()ヘルパー関数を実装
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.7, 13.1, 13.2, 13.3_
  - **FILE: crates/wintf/src/ecs/drag/state.rs (既存)**

- [ ] 2.3 DragConfigコンポーネント定義
  - DragConfigコンポーネント構造体を定義（enabled, threshold, buttons）
  - デフォルト値（enabled: true, threshold: 5px, buttons: 左ボタンのみ）を実装
  - ボタンごとの有効/無効フラグを実装
  - _Requirements: 1.6, 2.5, 2.6, 2.8_
  - **FILE: crates/wintf/src/ecs/drag/config.rs (既存)**

- [ ] 2.4 WM_LBUTTONDOWNハンドラでドラッグ準備開始
  - WM_LBUTTONDOWNメッセージハンドラにドラッグ準備ロジックを追加
  - hit_testでEntity取得、DragConfigの有効性チェック
  - thread_local DragState::Preparing遷移とSetCapture呼び出し
  - 開始位置（PhysicalPoint）と開始時刻を記録
  - 既にPreparing/Dragging状態の場合は早期リターン（複数ボタンドラッグ禁止）
  - _Requirements: 1.2, 1.7, 2.2, 14.1_
  - **FILE: crates/wintf/src/ecs/window_proc/handlers.rs**

- [ ] 2.5 WM_MOUSEMOVEハンドラで閾値判定とデルタ累積
  - WM_MOUSEMOVEメッセージハンドラにドラッグ閾値判定を追加
  - Preparing状態でユークリッド距離計算（√(dx²+dy²)）
  - 閾値（デフォルト5px）到達でDragging状態に遷移
  - DragAccumulatorResource.set_transition(Started)を呼び出し
  - Dragging状態では current_pos - prev_pos を計算してDragAccumulatorResource.accumulate_delta()
  - thread_local DragState.prev_posを更新
  - _Requirements: 1.3, 2.1, 2.7, 13.2_
  - **FILE: crates/wintf/src/ecs/window_proc/handlers.rs**

- [ ] 2.6 WM_LBUTTONUPハンドラでドラッグ終了
  - WM_LBUTTONUPメッセージハンドラにドラッグ終了ロジックを追加
  - Dragging状態からIdle遷移
  - DragAccumulatorResource.set_transition(Ended)を呼び出し
  - ReleaseCapture呼び出し
  - 最終位置の記録
  - _Requirements: 1.4, 4.1, 4.2, 4.6, 14.3_
  - **FILE: crates/wintf/src/ecs/window_proc/handlers.rs**

- [ ] 2.7 WM_KEYDOWNハンドラでESCキーキャンセル
  - WM_KEYDOWNメッセージハンドラにESCキー検知を追加
  - ESCキー押下でDragging→Idle遷移
  - DragAccumulatorResource.set_transition(Ended { cancelled: true })
  - ReleaseCapture呼び出し
  - _Requirements: 5.1, 5.2, 5.3_
  - **FILE: crates/wintf/src/ecs/window_proc/handlers.rs**

- [ ] 2.8 WM_CANCELMODEハンドラで強制キャンセル
  - WM_CANCELMODEメッセージハンドラを実装
  - ドラッグ状態クリーンアップ（Dragging→Idle）
  - DragAccumulatorResource.set_transition(Ended { cancelled: true })
  - DefWindowProcWへの委譲（None返却）でReleaseCapture自動実行
  - _Requirements: 5.4, 14.4_
  - **FILE: crates/wintf/src/ecs/window_proc/handlers.rs**

### Phase 3: ドラッグイベント配信とECS統合

- [ ] 3. ドラッグイベント定義とPhase<T>配信
- [ ] 3.1 DragStartEvent/DragEvent/DragEndEvent定義
  - DragStartEvent構造体（target, position, is_primary, timestamp）
  - DragEvent構造体（target, delta, position, is_primary, timestamp）
  - DragEndEvent構造体（target, position, cancelled, is_primary, timestamp）
  - 各イベントにEntity、PhysicalPoint、時刻情報を含める
  - _Requirements: 2.3, 2.4, 3.1, 3.2, 3.3, 3.4, 3.5, 4.3, 4.4, 4.5_
  - **FILE: crates/wintf/src/ecs/drag/events.rs (既存)**

- [ ] 3.2 dispatch_drag_events SystemでDragAccumulator flush
  - dispatch_drag_events()関数を実装（毎ECSフレーム実行）
  - DragAccumulatorResource.flush()で累積量と遷移を取得
  - pending_transitionがStartedなら:
    - DragStartEvent配信（Phase<T>ジェネリック関数）
    - DraggingStateコンポーネント挿入（target entityに）
  - accumulated_deltaが非ゼロなら:
    - DragEvent配信（Phase<T>ジェネリック関数、deltaに累積量設定）
    - DraggingState.prev_frame_posを更新
  - pending_transitionがEndedなら:
    - DragEndEvent配信（Phase<T>ジェネリック関数）
    - DraggingStateコンポーネント削除
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 13.4_
  - **FILE: crates/wintf/src/ecs/drag/dispatch.rs (既存)**

- [ ] 3.3 OnDragStart/OnDrag/OnDragEndハンドラコンポーネント定義
  - OnDragStartコンポーネント（Phase<DragStartEvent>ハンドラ）
  - OnDragコンポーネント（Phase<DragEvent>ハンドラ）
  - OnDragEndコンポーネント（Phase<DragEndEvent>ハンドラ）
  - SparseSet storageで効率的な管理
  - _Requirements: 7.1, 7.2, 7.5_
  - **FILE: crates/wintf/src/ecs/drag/handlers.rs (既存)**

- [ ] 3.4 DraggingStateコンポーネント定義
  - DraggingState構造体（drag_start_pos, prev_frame_pos）をSparseSetで定義
  - ドラッグ中エンティティの識別を可能にする
  - Query<(Entity, &DraggingState)>でアプリからアクセス可能
  - _Requirements: 1.5, 10.2_
  - **FILE: crates/wintf/src/ecs/drag/components.rs (既存)**

### Phase 4: ウィンドウ移動とドラッグ制約

- [ ] 4. ウィンドウ移動システムとドラッグ制約
- [ ] 4.1 apply_window_drag_movement Systemでウィンドウ位置更新
  - apply_window_drag_movement()関数を実装（DragEvent購読）
  - DragEventのdeltaを累積してウィンドウOffset更新
  - SetWindowPosCommand::enqueue()でWorld借用競合を回避
  - event.targetの親階層からWindowコンポーネント探索
  - Windowが見つからなければスキップ（将来の非ウィンドウドラッグ対応）
  - _Requirements: 6.1, 6.2, 6.3, 6.6, 13.5_

- [ ] 4.2 (P) DragConstraintコンポーネント定義と制約適用
  - DragConstraint構造体（min_x, max_x, min_y, max_y: Option<i32>）
  - apply()メソッドで制約適用後の座標を返す
  - apply_window_drag_movement内でDragConstraint適用
  - 軸ごとの制約（水平のみ、垂直のみ）をサポート
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7_

- [ ] 4.3 cleanup_drag_state Systemでマーカー削除
  - cleanup_drag_state()関数を実装（DragEndEvent購読）
  - DragEndEventのtargetエンティティからDraggingMarkerを削除
  - エンティティ削除時の自動クリーンアップ確認
  - _Requirements: 4.6, 10.5_

- [ ] 4.4 SetWindowPosCommand::flush()のスケジュール統合
  - dispatch_drag_events → apply_window_drag_movement → SetWindowPosCommand::flush順序を確保
  - 既存window.rsのflush呼び出しタイミングを確認
  - ドラッグ処理とウィンドウ更新の一貫性を保証
  - _Requirements: 6.3, 13.5_

### Phase 5: マルチモニター対応と高DPI

- [ ] 5. マルチモニター座標系とDPI対応
- [ ] 5.1 GlobalArrangement仮想スクリーン座標系確認
  - GlobalArrangement.boundsが既に仮想スクリーン座標系であることを確認
  - 負の座標値（プライマリモニタより左/上）が正しく処理されることを確認
  - WM_MOUSEMOVEのlParamがスクリーン座標であることを確認
  - SetWindowPosが仮想スクリーン座標系を受け付けることを確認
  - _Requirements: 8.1, 8.2, 8.4_

- [ ] 5.2* マルチモニター環境でのE2Eテスト（オプショナル）
  - マルチモニター環境でのウィンドウドラッグ動作確認
  - モニター境界をまたぐドラッグの正常動作確認
  - 高DPI環境での座標変換の正確性確認
  - 画面外配置時の可視領域補正動作確認（Requirement 8.3）
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_

### Phase 6: taffy_flex_demo統合とサンプル実装

- [ ] 6. taffy_flex_demoへのドラッグ機能統合
- [ ] 6.1 FlexDemoContainerにドラッグハンドラ登録
  - FlexDemoContainerエンティティにOnDragStart/OnDrag/OnDragEndを登録
  - 各ハンドラでイベント種別、sender/entityのName、座標、移動量をログ出力
  - "[Drag]" プレフィックスで既存Tunnel/Bubbleログと区別
  - DragConfigでドラッグ有効化（enabled: true, threshold: 5px）
  - _Requirements: 12.1, 12.2, 12.3, 12.4, 12.6_

- [ ] 6.2 taffy_flex_demoのコメントとドキュメント更新
  - サンプルコード冒頭にドラッグ可能であることをコメント記載
  - ドラッグハンドラの登録例を明確に記述
  - 既存の階層構造とレイアウトシステムとの統合を説明
  - _Requirements: 12.5_

### Phase 7: 統合テストとドキュメント

- [ ] 7. 統合テストと最終検証
- [ ] 7.1 ドラッグ操作の統合テスト
  - taffy_flex_demoで全ドラッグフローの動作確認（開始→移動→終了）
  - ESCキーキャンセルの動作確認
  - WM_CANCELMODEキャンセルの動作確認（Alt+Tab、モーダルダイアログ等）
  - ドラッグ閾値（5px）の動作確認
  - 単一クリック（閾値未到達）が正常動作することを確認
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 5.1, 5.2, 5.3_

- [ ] 7.2* Phase::Tunnel/Bubbleイベント伝播の統合テスト（オプショナル）
  - 親エンティティでのイベント横取り動作確認
  - 子エンティティでのstopPropagation動作確認
  - Tunnelフェーズでの早期停止確認
  - Bubbleフェーズでの親への伝播停止確認
  - _Requirements: 7.2, 7.3, 7.4, 7.6_

- [ ] 7.3* ドラッグ制約の統合テスト（オプショナル）
  - DragConstraintを設定してウィンドウ移動範囲を制限
  - 軸ごとの制約（水平のみ、垂直のみ）動作確認
  - 境界到達時のウィンドウ停止動作確認
  - 制約の動的切り替え動作確認
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7_

- [ ] 7.4 パフォーマンス検証
  - ドラッグイベント処理が16ms以内（60fps維持）であることを確認
  - ウィンドウ位置更新の遅延がないことを確認（1フレーム以内）
  - ドラッグ状態管理オーバーヘッドが無視できるレベル（< 0.1ms）であることを確認
  - _Requirements: NFR-1, NFR-2_

- [ ] 7.5 エラーハンドリングとエッジケース検証
  - エンティティ削除時のドラッグ自動キャンセル確認
  - マウスキャプチャ予期しない解放時の動作確認
  - 複数ボタン同時押下時の拒否動作確認
  - フォーカス喪失時のキャンセル動作確認（オプション機能）
  - _Requirements: 1.7, 5.4, 5.5, 5.6, 14.4_

---

## 要件カバレッジ

| 要件ID | タスク番号 | 説明 |
|--------|-----------|------|
| 1.1 | 2.1, 7.1 | ドラッグ状態管理（非ドラッグ、ドラッグ準備、ドラッグ中） |
| 1.2 | 2.3, 7.1 | マウスボタン押下でドラッグ準備状態遷移 |
| 1.3 | 2.4, 7.1 | 閾値超過でドラッグ中状態遷移 |
| 1.4 | 2.5, 7.1 | マウスボタン解放で非ドラッグ状態遷移 |
| 1.5 | 3.4 | ドラッグ中エンティティとボタン種別の一意識別 |
| 1.6 | 2.2 | ボタンごとのドラッグ有効/無効設定 |
| 1.7 | 2.1, 2.3, 7.5 | 同時複数ボタンドラッグ禁止 |
| 2.1 | 2.4, 7.1 | 閾値超過でDragStartイベント発火 |
| 2.2 | 2.3 | DragStartイベント発火時にSetCapture |
| 2.3 | 3.1 | ドラッグ開始位置とボタン種別をイベント情報に含める |
| 2.4 | 3.1 | ドラッグ対象エンティティをイベント情報に含める |
| 2.5 | 2.2 | エンティティごとのドラッグ閾値設定 |
| 2.6 | 2.2 | ドラッグ閾値0設定で即座にDragStart発火 |
| 2.7 | 2.4 | ドラッグ開始判定にユークリッド距離使用 |
| 2.8 | 2.2 | ドラッグ閾値を物理ピクセル単位で管理 |
| 3.1 | 3.1 | ドラッグ中マウス移動でDragイベント継続発火 |
| 3.2 | 3.1 | 現在マウス位置をイベント情報に含める |
| 3.3 | 3.1 | ドラッグ開始位置からの差分をイベント情報に含める |
| 3.4 | 3.1 | 前フレームからの移動量をイベント情報に含める |
| 3.5 | 3.1 | ドラッグ経過時間をイベント情報に含める |
| 4.1 | 2.5 | マウスボタン解放でDragEndイベント発火 |
| 4.2 | 2.5 | DragEndイベント発火時にReleaseCapture |
| 4.3 | 3.1 | 最終マウス位置をイベント情報に含める |
| 4.4 | 3.1 | 総移動量をイベント情報に含める |
| 4.5 | 3.1 | 正常終了かキャンセルかを識別 |
| 4.6 | 2.5, 4.3 | DragEndイベント発火後にドラッグ状態クリア |
| 5.1 | 2.6, 7.1 | ESCキーでドラッグキャンセル |
| 5.2 | 2.6, 7.1 | キャンセル時にDragEndイベント（cancelledフラグ付き）発火 |
| 5.3 | 2.6, 7.1 | キャンセル時にReleaseCapture |
| 5.4 | 2.7, 7.5 | プログラムからのドラッグキャンセル要求受付 |
| 5.5 | 7.5 | エンティティ削除時の自動ドラッグキャンセル |
| 5.6 | 7.5 | フォーカス喪失時のドラッグキャンセルオプション |
| 6.1 | 4.1 | ウィンドウエンティティドラッグでWindowPos更新 |
| 6.2 | 4.1 | ドラッグ中マウスカーソル追従でリアルタイム更新 |
| 6.3 | 4.1, 4.4 | SetWindowPos APIでウィンドウ位置更新 |
| 6.4 | 2.2 | ウィンドウ移動有効/無効設定 |
| 6.5 | 4.1 | ウィンドウエンティティのみドラッグ移動対象 |
| 6.6 | 4.1 | イベントキャプチャなしでウィンドウ移動なし |
| 7.1 | 1.1 | Phase enumでTunnel/Bubble配信 |
| 7.2 | 1.1, 3.2, 7.2* | Tunnel/Bubbleの2フェーズ配信 |
| 7.3 | 1.1, 3.2, 7.2* | TunnelフェーズでのstopPropagation |
| 7.4 | 1.1, 3.2, 7.2* | BubbleフェーズでのstopPropagation |
| 7.5 | 1.1, 3.2, 3.3 | ハンドラにsenderとentityを引数として渡す |
| 7.6 | 7.2* | 子ウィジェットがイベント消費時にウィンドウ移動なし |
| 8.1 | 5.1, 5.2* | モニター境界越えの正常動作 |
| 8.2 | 5.1, 5.2* | 仮想スクリーン座標系での位置計算 |
| 8.3 | 5.2* | 画面外配置時の可視領域補正オプション |
| 8.4 | 5.1, 5.2* | 高DPI環境の座標変換 |
| 8.5 | 5.2* | モニター構成変更時の位置再計算 |
| 9.1 | 4.2, 7.3* | GlobalArrangementバウンディングボックスで制約 |
| 9.2 | 4.2, 7.3* | デフォルト制約なし |
| 9.3 | 4.2, 7.3* | 制約範囲外で境界停止 |
| 9.4 | 4.2, 7.3* | スクリーン物理座標系バウンディングボックス使用 |
| 9.5 | 4.2, 7.3* | 軸ごとのドラッグ制約 |
| 9.6 | 4.2, 7.3* | 制約の動的切り替え |
| 9.7 | 4.2, 7.3* | 制約適用後の位置をイベント情報に含める |
| 10.1 | 全般 | ECSシステムとして実装 |
| 10.2 | 3.4 | ドラッグ状態をECSコンポーネントで管理 |
| 10.3 | 3.2 | ドラッグイベントをECSリソースで配信 |
| 10.4 | 3.2 | 親仕様イベントシステムと統合 |
| 10.5 | 4.3 | エンティティ削除時のドラッグ状態クリーンアップ |
| 11.1 | 3.1 | ドラッグ終了時の最終位置イベント通知 |
| 11.2 | 3.1 | 最終位置の画面座標と仮想スクリーン座標提供 |
| 11.3 | 3.1 | 総移動量提供 |
| 11.4 | 3.1 | 配置モニター情報提供 |
| 11.5 | 3.1 | キャンセル時は位置変更通知なし |
| 12.1 | 6.1 | FlexDemoContainerドラッグ可能化 |
| 12.2 | 6.1 | FlexDemoContainer左ボタンドラッグでウィンドウ移動 |
| 12.3 | 6.1 | OnDragStart/OnDrag/OnDragEndハンドラ実装 |
| 12.4 | 6.1 | ドラッグイベントログ出力 |
| 12.5 | 6.2 | サンプルコードコメント記載 |
| 12.6 | 6.1 | "[Drag]"プレフィックスログ出力 |
| 13.1 | 2.1 | wndproc内でドラッグ状態管理と閾値判定 |
| 13.2 | 2.1, 2.4 | 物理ピクセル座標で閾値判定 |
| 13.3 | 2.1 | Win32メッセージの物理ピクセル座標を直接使用 |
| 13.4 | 3.2 | DragStartイベントをECSリソースで配信 |
| 13.5 | 4.1, 4.4 | wndprocから直接SetWindowPos実行 |
| 14.1 | 2.3 | ドラッグ開始時にSetCaptureでマウスキャプチャ取得 |
| 14.2 | 2.3 | マウスキャプチャ中にウィンドウ外でもイベント受信 |
| 14.3 | 2.5 | ドラッグ終了時に必ずReleaseCapture |
| 14.4 | 2.7, 7.5 | WM_CANCELMODEで自動キャンセル |
| 14.5 | 2.3 | マウスキャプチャで統一的実装基盤提供 |
| NFR-1 | 7.4 | ドラッグイベント処理16ms以内、状態管理<0.1ms |
| NFR-2 | 7.4 | リアルタイム追従、1フレーム以内 |
| NFR-3 | 7.1, 7.5 | イベント取りこぼしなし、状態整合性保証、正確な座標計算 |

---

## 実装順序の根拠

1. **Phase 1（Phase<T>ジェネリック化）**: 設計レビュー議題1の決定に従い、最初のタスクで既存PointerEvent配信をPhase<T>ジェネリック関数化し、リグレッションテストで動作保証。全OK後にコミットし、後続タスクはリスクフリー。

2. **Phase 2（ドラッグ状態管理）**: DragState（thread_local!）とWin32メッセージハンドラを実装。wndproc層での閾値判定、SetCapture/ReleaseCapture、ESC/WM_CANCELMODEキャンセルを確立。

3. **Phase 3（イベント配信）**: Phase<T>でDragStartEvent/DragEvent/DragEndEventを配信。ハンドラコンポーネント（OnDragStart等）とDraggingMarkerを定義し、ECS層との統合を完了。

4. **Phase 4（ウィンドウ移動）**: apply_window_drag_movementでSetWindowPosCommand::enqueue()を使用し、World借用競合を回避。DragConstraintでドラッグ範囲制約、cleanup_drag_stateでマーカー削除。

5. **Phase 5（マルチモニター）**: GlobalArrangementの仮想スクリーン座標系確認。既存実装で対応済みのため、検証のみ。高DPI環境のE2Eテストはオプショナル。

6. **Phase 6（taffy_flex_demo統合）**: 実装完了後、サンプルアプリケーションで実際の動作確認とログ出力。ドキュメント更新。

7. **Phase 7（統合テスト）**: 全フロー検証（開始→移動→終了）、キャンセル動作確認、パフォーマンス検証、エラーハンドリング確認。オプショナルテスト（Phase::Tunnel/Bubble、DragConstraint）は時間に応じて実施。

---

## 並列実行可能タスク

- タスク2.1（DragState定義）とタスク2.2（DragConfig定義）は並列実行可能（データ依存なし）
- タスク3.1（イベント定義）、タスク3.3（ハンドラコンポーネント）、タスク3.4（DraggingMarker）は並列実行可能（ファイル競合なし）
- タスク4.2（DragConstraint定義）は他タスクと並列実行可能（apply_window_drag_movementとは独立）

---

## 注意事項

- **オプショナルタスク**（`- [ ]*`マーク）は、MVP後に時間があれば実施する受入基準特化型テスト。実装の正確性検証よりもAcceptance Criteria網羅を目的とする。
- **Phase 1のリグレッションテスト（タスク1.2）は必須**。Phase<T>ジェネリック化が既存PointerEvent配信を破壊しないことを保証。
- **SetWindowPosCommand::flush()のスケジュール統合（タスク4.4）**は、既存window.rsのflush呼び出しタイミングを確認し、ドラッグ処理との順序を保証する重要タスク。
- **DraggingMarkerのsenderフィールド（タスク3.4）**は、議題4の決定に従い、最初にOnDragStartを処理した子エンティティを記録。
