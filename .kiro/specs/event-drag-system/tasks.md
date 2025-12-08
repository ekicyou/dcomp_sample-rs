# Implementation Plan

## タスクフォーマットテンプレート

このドキュメントは、event-drag-systemの実装タスクを定義する。

---

## 実装状況メモ

### 🔴 **緊急課題：DPIスケール問題（2025-12-08 22:07 JST）**

#### 現象
- **あらゆるDPIスケール環境でウィンドウがマウスに対して約1.25倍速く動く**
- **1.25倍スケールのモニタ**: マウス10px → ウィンドウ12.5px
- **2.0倍スケールのモニタ**: マウス10px → ウィンドウ約12.5px（1.25倍スケール環境と同じ挙動）
- **DPIチェンジによるスケール変更は正しく動作している**（別の問題ではない）
- ログでは正確に動いているように見える：
  - `dx=1` → `client_x` が1ピクセル増加
  - `dx=2` → `client_x` が2ピクセル増加

#### 重要な観察
1. **DPIスケール値に依存しない挙動**：
   - 1.25倍でも2.0倍でも同じ約1.25倍速の動き
   - つまり、DPIスケール値を使った単純な変換の問題ではない
2. **DPIチェンジは正常動作**：
   - モニタ間でのDPI変更時のウィンドウリサイズは正しい
   - つまり、DPI認識機構自体は正常
3. **ログと実際の動作の乖離**：
   - `dx=1`でログ上は1px増加
   - 実際には約1.25px増加している
4. **🔑 重要なヒント：1.25という数字の意味**：
   - **最初のDPIスケールが1.25倍だった**
   - **この値が更新されずに使われ続けている可能性が高い**
   - モニタを2.0倍スケールに移動しても1.25倍速 → スケール値が古いまま
   - プログラム起動時のDPIスケール（1.25倍）が固定されている？

#### 調査済み事項
1. ✅ DPI変換の追加・削除を試行
   - `event.delta * dpi_scale`で変換 → 問題解決せず
   - 変換なし（そのまま加算） → 問題解決せず
2. ✅ `WM_MOUSEMOVE`と`WM_WINDOWPOSCHANGED`のログ確認
   - `dx=1`で`client_x`が正確に1増加
   - `dx=2`で`client_x`が正確に2増加
3. ✅ `DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2`設定済み
   - `process_singleton.rs:64`で設定

#### 未調査事項
- **マウス座標の実際の単位**：
  - `WM_MOUSEMOVE`の`lParam`は物理ピクセルか論理ピクセルか？
  - `DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2`モードでは物理ピクセルのはずだが、実際は？
- **累積器の座標変換**：
  - `DragAccumulator::accumulate_delta`で座標変換が必要か？
  - `WM_MOUSEMOVE`で取得する`prev_pos`と`current_pos`の単位は？
- **BoxStyle.insetの実際の単位**：
  - ドキュメントでは「物理ピクセル」だが、実際にレイアウト計算時に変換されている可能性
  - `apply_window_pos_changes`での`SetWindowPos`呼び出し時の座標系

#### 次回の調査手順
1. **🔍 最優先：初回DPIスケール値（1.25）が固定化されている変数を探す**
   - `apply_window_drag_movement`で使用される可能性のある変数
   - ウィンドウ作成時に初期化され、その後更新されない変数
   - `BoxStyle.inset`から`SetWindowPos`への変換処理で使われる係数
   - `apply_window_pos_changes`システム内の固定DPI係数
2. **マウス座標の生値を確認**
   ```rust
   // WM_MOUSEMOVEハンドラで
   let x_raw = (lparam.0 & 0xFFFF) as i16 as i32;
   tracing::info!("RAW mouse x={}, screen_x={}", x_raw, screen_x);
   ```
3. **WindowPosの生値を確認**
   ```rust
   // WM_WINDOWPOSCHANGEDで
   tracing::info!("WINDOWPOS: x={}, y={} (window coords)", wp.x, wp.y);
   ```
4. **DPIスケールの確認**
   ```rust
   // GlobalArrangementから
   let dpi_scale = ga.scale_x();
   tracing::info!("DPI scale={}", dpi_scale);
   ```
5. **ウィンドウ座標変換の確認**
   - `WindowHandle::window_to_client_coords`の実装を確認
   - 座標変換時にDPIスケールが適用されているか確認
6. **`apply_window_pos_changes`システムを確認**
   - `BoxStyle.inset`から`WindowPos`への変換処理
   - ここで初回DPIスケール（1.25）が固定使用されている可能性が高い

#### 疑わしいポイント
1. **🔥 最有力：初回DPIスケール（1.25）が固定化されている変数**
   - ウィンドウ作成時に1.25倍スケールで初期化
   - その後、モニタ移動してもこの値が更新されない
   - `apply_window_pos_changes`システムで使われている？
   - `BoxStyle.inset` → `WindowPos` 変換時の係数？
2. **`BoxStyle.inset`への書き込みと実際のSetWindowPos間の変換**
   - `apply_window_pos_changes`でのinset→WindowPos変換
   - ここで初回DPIスケール（1.25）が固定使用されている可能性
3. **ログに出ている`client_x`が実は別の座標系**
   - `WindowPos.position`が既に変換済みの座標？
   - 実際のウィンドウ位置とは異なる値？
4. **既存のウィンドウ移動処理との重複適用**
   - `BoxStyle.inset`更新とWindowPos更新の両方が動いている？
   - 2つの更新が重なって1.25倍になっている？
   - ただし、これだと2.0倍環境でも1.25倍になる説明がつかない

### 以前の問題（解決済み）
- ~~thread_local DragStateがwndprocスレッドとECSスレッドで共有されない問題~~
  - → DragAccumulatorResourceで解決
- ~~ドラッグイベントがとびとびにしか発火しない~~
  - → 累積器方式で解決

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
- [x] 2.1 DragAccumulatorとDragAccumulatorResource定義
  - DragAccumulator構造体（accumulated_delta, pending_transition）を定義
  - DragTransition enum（Started/Ended）を定義
  - DragAccumulatorResource（Arc<Mutex<DragAccumulator>>）をECSリソースとして定義
  - accumulate_delta(), set_transition(), flush()メソッドを実装
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 13.1, 13.2_
  - **FILE: crates/wintf/src/ecs/drag/accumulator.rs (新規作成)**
  - **STATUS: 完了**

- [x] 2.2 DragStateとPhysicalPoint定義（thread_local専用）
  - DragState enum（Idle/Preparing/Dragging）をthread_local! + RefCellで実装
  - PhysicalPoint構造体を定義（x, y: i32）
  - DragStateがEntity、開始位置、現在位置を保持する構造を設計
  - 単一DragState（複数ボタン同時ドラッグ禁止）を明確化
  - update_drag_state(), read_drag_state()ヘルパー関数を実装
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.7, 13.1, 13.2, 13.3_
  - **FILE: crates/wintf/src/ecs/drag/state.rs (既存)**
  - **STATUS: 完了（既存実装）**

- [x] 2.3 DragConfigコンポーネント定義
  - DragConfigコンポーネント構造体を定義（enabled, threshold, buttons）
  - デフォルト値（enabled: true, threshold: 5px, buttons: 左ボタンのみ）を実装
  - ボタンごとの有効/無効フラグを実装
  - _Requirements: 1.6, 2.5, 2.6, 2.8_
  - **FILE: crates/wintf/src/ecs/drag/config.rs (既存)**
  - **STATUS: 完了（既存実装）**

- [x] 2.4 WM_LBUTTONDOWNハンドラでドラッグ準備開始
  - WM_LBUTTONDOWNメッセージハンドラにドラッグ準備ロジックを追加
  - hit_testでEntity取得、DragConfigの有効性チェック
  - thread_local DragState::Preparing遷移とSetCapture呼び出し
  - 開始位置（PhysicalPoint）と開始時刻を記録
  - 既にPreparing/Dragging状態の場合は早期リターン（複数ボタンドラッグ禁止）
  - _Requirements: 1.2, 1.7, 2.2, 14.1_
  - **FILE: crates/wintf/src/ecs/window_proc/handlers.rs (handle_button_message)**
  - **STATUS: 完了**

- [x] 2.5 WM_MOUSEMOVEハンドラで閾値判定とデルタ累積
  - WM_MOUSEMOVEメッセージハンドラにドラッグ閾値判定を追加
  - Preparing状態でユークリッド距離計算（√(dx²+dy²)）
  - 閾値（デフォルト5px）到達でDragging状態に遷移
  - DragAccumulatorResource.set_transition(Started)を呼び出し
  - Dragging状態では current_pos - prev_pos を計算してDragAccumulatorResource.accumulate_delta()
  - thread_local DragState.prev_posを更新
  - _Requirements: 1.3, 2.1, 2.7, 13.2_
  - **FILE: crates/wintf/src/ecs/window_proc/handlers.rs (WM_MOUSEMOVE)**
  - **STATUS: 完了**

- [x] 2.6 WM_LBUTTONUPハンドラでドラッグ終了
  - WM_LBUTTONUPメッセージハンドラにドラッグ終了ロジックを追加
  - Dragging状態からIdle遷移
  - DragAccumulatorResource.set_transition(Ended)を呼び出し
  - ReleaseCapture呼び出し
  - 最終位置の記録
  - _Requirements: 1.4, 4.1, 4.2, 4.6, 14.3_
  - **FILE: crates/wintf/src/ecs/window_proc/handlers.rs (handle_button_message + fallback)**
  - **STATUS: 完了（hit_test失敗時のフォールバックも実装）**

- [x] 2.7 WM_KEYDOWNハンドラでESCキーキャンセル
  - WM_KEYDOWNメッセージハンドラにESCキー検知を追加
  - ESCキー押下でDragging→Idle遷移
  - DragAccumulatorResource.set_transition(Ended { cancelled: true })
  - ReleaseCapture呼び出し
  - _Requirements: 5.1, 5.2, 5.3_
  - **FILE: crates/wintf/src/ecs/window_proc/handlers.rs (WM_KEYDOWN)**
  - **STATUS: 完了**

- [x] 2.8 WM_CANCELMODEハンドラで強制キャンセル
  - WM_CANCELMODEメッセージハンドラを実装
  - ドラッグ状態クリーンアップ（Dragging→Idle）
  - DragAccumulatorResource.set_transition(Ended { cancelled: true })
  - DefWindowProcWへの委譲（None返却）でReleaseCapture自動実行
  - _Requirements: 5.4, 14.4_
  - **FILE: crates/wintf/src/ecs/window_proc/handlers.rs (WM_CANCELMODE)**
  - **STATUS: 完了**

### Phase 3: ドラッグイベント配信とECS統合

- [x] 3. ドラッグイベント定義とPhase<T>配信
- [x] 3.1 DragStartEvent/DragEvent/DragEndEvent定義
  - DragStartEvent構造体（target, position, is_primary, timestamp）
  - DragEvent構造体（target, delta, position, is_primary, timestamp）
  - DragEndEvent構造体（target, position, cancelled, is_primary, timestamp）
  - 各イベントにEntity、PhysicalPoint、時刻情報を含める
  - _Requirements: 2.3, 2.4, 3.1, 3.2, 3.3, 3.4, 3.5, 4.3, 4.4, 4.5_
  - **FILE: crates/wintf/src/ecs/drag/dispatch.rs**
  - **STATUS: 完了**

- [x] 3.2 dispatch_drag_events SystemでDragAccumulator flush
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
  - **追加修正**: JustStarted→Dragging遷移を追加（dispatch後にupdate_dragging呼び出し）
  - _Requirements: 7.1, 7.2, 7.3, 7.4, 7.5, 13.4_
  - **FILE: crates/wintf/src/ecs/drag/dispatch.rs**
  - **STATUS: 完了**

- [x] 3.3 OnDragStart/OnDrag/OnDragEndハンドラコンポーネント定義
  - OnDragStartコンポーネント（Phase<DragStartEvent>ハンドラ）
  - OnDragコンポーネント（Phase<DragEvent>ハンドラ）
  - OnDragEndコンポーネント（Phase<DragEndEvent>ハンドラ）
  - SparseSet storageで効率的な管理
  - _Requirements: 7.1, 7.2, 7.5_
  - **FILE: crates/wintf/src/ecs/drag/mod.rs**
  - **STATUS: 完了**

- [x] 3.4 DraggingStateコンポーネント定義
  - DraggingState構造体（drag_start_pos, prev_frame_pos）をSparseSetで定義
  - ドラッグ中エンティティの識別を可能にする
  - Query<(Entity, &DraggingState)>でアプリからアクセス可能
  - _Requirements: 1.5, 10.2_
  - **FILE: crates/wintf/src/ecs/drag/mod.rs**
  - **STATUS: 完了**

### Phase 4: ウィンドウ移動とドラッグ制約

- [ ] 4. ウィンドウ移動システムとドラッグ制約 **【🔴 BLOCKED: DPIスケール問題により未完了】**
- [ ] 4.1 apply_window_drag_movement Systemでウィンドウ位置更新 **【🔴 1.25倍速バグあり】**
  - apply_window_drag_movement()関数を実装（DragEvent購読）
  - DragEventのdeltaを累積してウィンドウOffset更新
  - SetWindowPosCommand::enqueue()でWorld借用競合を回避
  - event.targetの親階層からWindowコンポーネント探索
  - Windowが見つからなければスキップ（将来の非ウィンドウドラッグ対応）
  - **重要**: BoxStyle.insetを直接更新（物理ピクセル単位）
  - **🔴 致命的バグ**: あらゆるDPIスケール環境で1.25倍速く動く
  - _Requirements: 6.1, 6.2, 6.3, 6.6, 13.5_
  - **FILE: crates/wintf/src/ecs/drag/systems.rs (apply_window_drag_movement)**
  - **STATUS: 実装完了だが、致命的バグにより使用不可**

- [x] 4.2 DragConstraintコンポーネント定義と制約適用
  - DragConstraint構造体（min_x, max_x, min_y, max_y: Option<i32>）
  - apply()メソッドで制約適用後の座標を返す
  - apply_window_drag_movement内でDragConstraint適用
  - 軸ごとの制約（水平のみ、垂直のみ）をサポート
  - _Requirements: 9.1, 9.2, 9.3, 9.4, 9.5, 9.6, 9.7_
  - **FILE: crates/wintf/src/ecs/drag/mod.rs**
  - **STATUS: 完了（構造体定義済み、apply_window_drag_movementで使用）**

- [x] 4.3 cleanup_drag_state Systemでマーカー削除
  - cleanup_drag_state()関数を実装（DragEndEvent購読）
  - DragEndEventのtargetエンティティからDraggingMarkerを削除
  - エンティティ削除時の自動クリーンアップ確認
  - _Requirements: 4.6, 10.5_
  - **FILE: crates/wintf/src/ecs/drag/systems.rs (cleanup_drag_state)**
  - **STATUS: 完了**

- [x] 4.4 システムスケジュール統合
  - dispatch_drag_events → apply_window_drag_movement → cleanup_drag_state順序を確保
  - world.rsのScheduleに登録済み
  - ドラッグ処理とウィンドウ更新の一貫性を保証
  - _Requirements: 6.3, 13.5_
  - **FILE: crates/wintf/src/ecs/world.rs**
  - **STATUS: 完了**

### Phase 5: マルチモニター対応と高DPI

- [ ] 5. マルチモニター座標系とDPI対応
- [ ] 5.1 GlobalArrangement仮想スクリーン座標系確認
  - GlobalArrangement.boundsが既に仮想スクリーン座標系であることを確認
  - 負の座標値（プライマリモニタより左/上）が正しく処理されることを確認
  - WM_MOUSEMOVEのlParamがスクリーン座標であることを確認
  - SetWindowPosが仮想スクリーン座標系を受け付けることを確認
  - **現状**: 基本的な座標系は動作しているが、DPIスケール問題が存在
  - _Requirements: 8.1, 8.2, 8.4_
  - **STATUS: 要DPI問題解決（緊急課題参照）**

- [ ] 5.2* マルチモニター環境でのE2Eテスト（オプショナル）
  - マルチモニター環境でのウィンドウドラッグ動作確認
  - モニター境界をまたぐドラッグの正常動作確認
  - 高DPI環境での座標変換の正確性確認
  - 画面外配置時の可視領域補正動作確認（Requirement 8.3）
  - _Requirements: 8.1, 8.2, 8.3, 8.4, 8.5_
  - **STATUS: 保留（DPI問題解決後に実施）**

### Phase 6: taffy_flex_demo統合とサンプル実装

- [ ] 6. taffy_flex_demoへのドラッグ機能統合 **【🔴 BLOCKED: DPIスケール問題により未完了】**
- [x] 6.1 FlexDemoContainerにドラッグハンドラ登録
  - FlexDemoContainerエンティティにOnDragStart/OnDrag/OnDragEndを登録
  - 各ハンドラでイベント種別、sender/entityのName、座標、移動量をログ出力
  - "[Drag]" プレフィックスで既存Tunnel/Bubbleログと区別
  - DragConfigでドラッグ有効化（enabled: true, threshold: 5px）
  - _Requirements: 12.1, 12.2, 12.3, 12.4, 12.6_
  - **FILE: crates/wintf/examples/taffy_flex_demo.rs (on_container_drag_start/drag/drag_end)**
  - **STATUS: 実装完了（但し、1.25倍速バグあり）**

- [x] 6.2 taffy_flex_demoのコメントとドキュメント更新
  - サンプルコード冒頭にドラッグ可能であることをコメント記載
  - ドラッグハンドラの登録例を明確に記述
  - 既存の階層構造とレイアウトシステムとの統合を説明
  - _Requirements: 12.5_
  - **FILE: crates/wintf/examples/taffy_flex_demo.rs**
  - **STATUS: 実装完了（但し、1.25倍速バグあり）**

### Phase 7: 統合テストとドキュメント

- [ ] 7. 統合テストと最終検証 **【🔴 BLOCKED: DPIスケール問題により未完了】**
- [ ] 7.1 ドラッグ操作の基本動作確認 **【🔴 FAILED: 1.25倍速バグ】**
  - taffy_flex_demoで全ドラッグフローの動作確認（開始→移動→終了）
  - ESCキーキャンセルの動作確認
  - WM_CANCELMODEキャンセルの動作確認（Alt+Tab、モーダルダイアログ等）
  - ドラッグ閾値（5px）の動作確認
  - 単一クリック（閾値未到達）が正常動作することを確認
  - **🔴 検証結果FAILED**: 致命的な1.25倍速バグが発覚
    - 1.25倍スケール環境: マウス10px → ウィンドウ12.5px
    - 2.0倍スケール環境: マウス10px → ウィンドウ12.5px（同じ挙動）
    - DPIスケール値に依存しない謎の1.25倍係数
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 2.1, 5.1, 5.2, 5.3_
  - **STATUS: FAILED - バグ修正まで完了不可**

- [ ] 7.2* Phase::Tunnel/Bubbleイベント伝播の統合テスト（オプショナル）
  - 親エンティティでのイベント横取り動作確認
  - 子エンティティでのstopPropagation動作確認
  - Tunnelフェーズでの早期停止確認
  - Bubbleフェーズでの親への伝播停止確認
  - _Requirements: 7.2, 7.3, 7.4, 7.6_
  - **STATUS: 未実施**

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
