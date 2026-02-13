# 調査と設計決定: event-drag-system

---
**目的**: ディスカバリー調査結果、アーキテクチャ検討、設計根拠を記録する。

**用途**:
- ディスカバリーフェーズでの調査活動と結果を記録
- design.mdに記載するには詳細すぎる設計決定のトレードオフを文書化
- 将来の監査や再利用のための参照と根拠を提供

---

## サマリー
- **機能**: `event-drag-system`
- **ディスカバリー範囲**: Extension（既存ポインターイベントシステムの拡張）
- **重要な発見事項**:
  - Gap分析でOption B（新規モジュール作成）を推奨
  - 既存パターン（ButtonBuffer、SetWindowPosCommand、Phase<T>）の流用可能
  - Win32 APIのSetCapture/ReleaseCaptureは既存コードで使用実績あり

## 調査ログ

### 既存コードパターンの分析

- **コンテキスト**: ドラッグシステム実装のための既存パターン調査
- **参照資料**:
  - `crates/wintf/src/ecs/pointer/mod.rs` - ButtonBufferパターン
  - `crates/wintf/src/ecs/window.rs` - SetWindowPosCommandキューパターン
  - `crates/wintf/src/ecs/pointer/dispatch.rs` - Phase<T>配信パターン
- **発見事項**:
  - **ButtonBuffer**: thread_local! + RefCell<HashMap<(Entity, PointerButton), ButtonBuffer>>によるボタン押下記録
  - **SetWindowPosCommand**: World借用競合回避のためのキューパターン（enqueue → flush）
  - **Phase<T>**: Tunnel/Bubbleの2フェーズイベント伝播、stopPropagation対応
- **影響**: これらのパターンをDragStateとドラッグイベント配信に直接適用可能

### Win32 API マウスキャプチャ（Research Needed 1）

- **コンテキスト**: SetCapture/ReleaseCaptureの動作とウィンドウ外移動時の挙動
- **参照資料**:
  - Microsoft Docs: SetCapture function
  - Microsoft Docs: ReleaseCapture function
  - Microsoft Docs: WM_CANCELMODE message
  - 既存実装: `crates/wintf/src/ecs/window_proc/handlers.rs` (SetCapture使用例なし、調査必要)
- **発見事項**:
  - **SetCapture**: 指定ウィンドウにマウス入力をキャプチャ、ウィンドウ外でもマウスイベント受信
  - **ReleaseCapture**: キャプチャ解放、ユーザー操作で自動解放される場合あり（WM_CANCELMODE）
  - **WM_CANCELMODE**: システムがマウスキャプチャ等のモードをキャンセルする必要がある時に送信
    - 発生条件: モーダルダイアログ表示、EnableWindow()無効化、Alt+Tab等
    - DefWindowProcWの動作: スクロールバー/メニュー処理キャンセル、**マウスキャプチャ自動解放**
    - 推奨実装: アプリはドラッグ状態をクリーンアップし、DefWindowProcWに委譲（`None`を返す）
  - ウィンドウ外移動時もSetCaptureは有効、ReleaseCaptureはドラッグ終了時に明示的に呼び出す
- **影響**: 
  - Requirement 14.2（ウィンドウ外イベント受信）は実装可能
  - WM_CANCELMODE処理が必須、DefWindowProcW委譲で自動ReleaseCapture

### マルチモニター座標系（Research Needed 2）

- **コンテキスト**: 仮想スクリーン座標とGlobalArrangement.boundsの整合性
- **参照資料**:
  - Microsoft Docs: Multiple Display Monitors
  - Microsoft Docs: GetSystemMetrics(SM_XVIRTUALSCREEN/SM_YVIRTUALSCREEN)
  - 既存実装: `crates/wintf/src/ecs/layout/arrangement.rs` - GlobalArrangement
- **発見事項**:
  - 仮想スクリーン座標: プライマリモニタ左上が原点(0,0)ではなく、左側/上側のモニタで負の座標が発生
  - GlobalArrangement.bounds: 既に物理ピクセル座標系で管理、仮想スクリーン座標と互換性あり
  - SetWindowPosは仮想スクリーン座標を受け付ける
- **影響**: Requirement 8.2（仮想スクリーン座標系）は既存インフラで対応可能、追加実装不要

### DPI環境でのドラッグ閾値（Research Needed 3）

- **コンテキスト**: 物理ピクセル5pxが高DPI環境で小さすぎる可能性
- **参照資料**:
  - Requirement 2 AC 2.8: 将来的なDPI考慮の余地を記載済み
  - 既存実装: wndprocは物理ピクセル座標を使用（DPIスケーリング前）
- **発見事項**:
  - 物理ピクセル5pxはDPI 100%で5px、200%でも5px（論理的には2.5px相当）
  - 高DPI環境では体感的に閾値が小さくなる可能性あり
  - 現時点では物理ピクセル固定、将来的にDPI係数を乗算する拡張余地を残す
- **影響**: 初期実装は物理ピクセル5px固定、DragConfigでエンティティごとに調整可能

### ECSコンポーネント設計パターン

- **コンテキスト**: 命名規則とストレージ戦略の確認
- **参照資料**: `.kiro/steering/structure.md` - Component Naming Conventions
- **発見事項**:
  - GPU資源: `XxxGraphics`サフィックス
  - CPU資源: `XxxResource`サフィックス
  - マーカー/設定: サフィックスなし（例: `DragConfig`, `DragConstraint`）
  - 頻繁な挿入/削除: `#[component(storage = "SparseSet")]`
- **影響**: DragConfig、DragConstraint、OnDragStart/OnDrag/OnDragEnd、DraggingMarkerすべてSparseSet

## アーキテクチャパターン評価

| オプション | 説明 | 強み | リスク/制約 | 備考 |
|-----------|------|------|------------|------|
| Option A: 既存拡張 | handlers.rsにドラッグロジック直接追加 | 最小ファイル数、既存パターン踏襲 | handlers.rs肥大化、責務不明瞭 | Gap分析で非推奨 |
| Option B: 新規モジュール | ecs/drag/独立モジュール作成 | 責務分離、テスト容易、拡張性 | 新規ファイル4個、学習コスト | Gap分析推奨、採用 |
| Option C: ハイブリッド | Phase 1で既存拡張→Phase 2で分離 | 段階的、リスク分散 | 計画複雑、2度の実装 | 時間制約ある場合のフォールバック |

## 設計決定

### 決定: 新規ecs/dragモジュール作成（Option B採用）

- **コンテキスト**: Gap分析で3つのオプションを評価、実装アプローチ選択
- **検討した代替案**:
  1. Option A: 既存handlers.rs拡張 - 最小ファイル数だが肥大化リスク
  2. Option C: ハイブリッド - 段階的だが2度の実装工数
- **選択したアプローチ**: Option B（新規モジュール）
  - `crates/wintf/src/ecs/drag/mod.rs` - 公開API、型定義
  - `crates/wintf/src/ecs/drag/state.rs` - DragState管理（thread_local!）
  - `crates/wintf/src/ecs/drag/dispatch.rs` - Phase<T>イベント配信
  - `crates/wintf/src/ecs/drag/systems.rs` - ECS System群
- **根拠**:
  - **責務分離**: ドラッグはポインターイベントの上位抽象化、独立モジュールが適切
  - **保守性**: handlers.rsの肥大化防止、変更影響範囲の限定
  - **テスト容易性**: 単体テストが書きやすい
  - **拡張性**: 将来の非ウィンドウ移動型ドラッグ（ファイルD&D等）への基盤
- **トレードオフ**:
  - ✅ 明確な境界、独立性、長期保守性
  - ❌ 新規ファイル4個、初期学習コスト（ただし既存パターン踏襲で軽減）
- **フォローアップ**: Phase<T>のジェネリック化により既存dispatch_pointer_eventsと統一

### 決定: thread_local! + RefCellによるDragState管理

- **コンテキスト**: wndproc層でのドラッグ状態管理方法
- **検討した代替案**:
  1. ECS Resource - World借用競合のリスク
  2. グローバルstatic Mutex - パフォーマンス懸念
- **選択したアプローチ**: thread_local! + RefCell<DragState>（ButtonBufferパターン）
- **根拠**:
  - wndprocはメインスレッド固定、thread_local!で十分
  - 既存ButtonBufferと同じパターン、一貫性維持
  - RefCellによる実行時借用チェック、World借用と競合しない
- **トレードオフ**:
  - ✅ パフォーマンス、既存パターン一貫性
  - ❌ マルチスレッド非対応（ただしwndprocは単一スレッド）

### 決定: SetWindowPosCommandキューパターン流用

- **コンテキスト**: ウィンドウ位置更新時のWorld借用競合回避
- **検討した代替案**:
  1. System内から直接SetWindowPos - World借用競合で不可
  2. 独自キュー実装 - 車輪の再発明
- **選択したアプローチ**: 既存SetWindowPosCommand::enqueue/flushを直接使用
- **根拠**:
  - 既にwindow.rsで実装済み、動作実績あり
  - World借用解放後のflush()呼び出しパターン確立
  - コード重複排除
- **トレードオフ**:
  - ✅ 既存パターン再利用、動作保証
  - ❌ window.rsへの依存（ただし同一クレート内で問題なし）

### 決定: Phase<T>ジェネリック関数による3種イベント配信（即座にジェネリック化）

- **コンテキスト**: DragStartEvent、DragEvent、DragEndEventの配信方法
- **検討した代替案**:
  1. dispatch_drag_events専用実装 - コード重複だが既存コード影響ゼロ
  2. 段階的実装（Phase 1独立実装→Phase 2共通化） - リスク分散だが2度の実装
  3. マクロ生成 - 複雑性増加
- **選択したアプローチ**: Option B（即座にジェネリック化）
  - **Phase 1（最優先タスク）**: pointer/dispatch.rsをジェネリック関数化
  - 既存PointerEventで動作確認、リグレッションテスト必須
  - すべてOKでコミット後、DragEvent実装へ
- **根拠**:
  - ジェネリック化は早い段階で判断すべき（設計レビュー合意事項）
  - リスクを最初のタスクで集中管理、後続タスクはリスクフリー
  - 既存dispatch_pointer_eventsのロジック流用でコード重複排除
  - 型安全性維持
- **トレードオフ**:
  - ✅ コード重複排除、一貫性、早期リスク解決
  - ❌ 最初のタスクでpointer/dispatch.rsのリファクタ必須、リグレッションテスト工数
- **互換性保証**: 既存PointerEvent配信の動作を完全保持、型推論エラーは型注釈で解決

## リスクと軽減策

- **リスク1: 状態遷移の複雑性** - Idle → Preparing → Dragging → Done/Cancelled
  - 軽減策: 状態図作成、既存ButtonBufferパターン応用
- **リスク2: 複数メッセージハンドラ統合** - WM_MOUSEMOVE、WM_*BUTTON*、WM_KEYDOWN、WM_CANCELMODE
  - 軽減策: 各ハンドラの責務最小化、drag::update_drag_state()への委譲
- **リスク3: マルチモニター座標系** - 仮想スクリーン座標の負の値
  - 軽減策: 調査完了、GlobalArrangementで既対応、追加実装不要

### 決定: DraggingMarker挿入ロジックとsenderフィールド

- **コンテキスト**: DraggingMarkerをどのエンティティに挿入するか、およびsender情報の保持
- **検討した代替案**:
  1. hit_testの結果エンティティに挿入 - 常にリーフウィジェットがターゲット、柔軟性なし
  2. ウィンドウエンティティに固定挿入 - タイトルバー限定ドラッグ不可
  3. イベント処理エンティティに挿入（sender記録なし） - 発動元追跡不可
- **選択したアプローチ**: イベント処理エンティティに挿入 + senderフィールド追加
  - `DraggingMarker { sender: Entity }`構造体に変更
  - Phase::Tunnel/Bubbleで**最初にOnDragStartハンドラを実行したエンティティ**に挿入
  - senderはそのエンティティ自身を記録（発動元追跡可能）
- **根拠**:
  - **3段階の挙動モデル**対応:
    1. taffy_flex_demoアプリ: タイトルバーウィジェットにハンドラ → タイトルバーにMarker → apply_window_drag_movementが親Window探索
    2. wintfフレームワーク: Phase::Tunnel/Bubble伝播で柔軟なハンドラ配置
    3. 将来拡張: 子ウィジェット独自ドラッグ（ウィンドウ移動なし）も可能
  - **スパースストレージ戦略**: 常時0〜1個のエンティティのみ保持、SparseSet最適
  - **sender情報**: どの子エンティティが発動したか追跡可能、デバッグ容易
  - **ウィンドウ移動との分離**: DraggingMarkerはイベント処理エンティティ示すのみ、実際のウィンドウ移動はapply_window_drag_movementが全DragEventを監視しWindow探索
- **トレードオフ**:
  - ✅ アプリ側で柔軟にドラッグ挙動制御、将来の非ウィンドウドラッグ対応、sender追跡可能
  - ❌ DraggingMarkerが構造体に（8バイト増加、ただしSparseSetで影響最小）
- **実装詳細**:
  - dispatch_drag_events()でdispatch_event()の戻り値（最初のハンドラ実行エンティティ）を取得
  - そのエンティティに`DraggingMarker { sender }`挿入
  - Query<(Entity, &DraggingMarker)>でターゲットとsender両方取得可能

- **リスク4: DPI環境での閾値調整** - 物理ピクセル5pxの体感差
  - 軽減策: DragConfigでエンティティごとに調整可能、将来的にDPI係数対応可能

## 実装時の問題と調査

### 問題: thread_local変数のスレッド間非共有（2025-12-09）

- **発見日時**: 2025-12-09T11:33:00Z
- **コンテキスト**: PointerBufferの位置更新が250ms（4Hz）でしか反映されない問題を調査中
- **根本原因**: 
  - `POINTER_BUFFERS`はthread_local!で定義されている
  - **WndProcスレッド**で`push_pointer_sample()`が呼ばれている（WM_MOUSEMOVEハンドラから）
  - **ECSスレッド**で`process_pointer_buffers()`が実行されている（Inputスケジュール）
  - thread_local変数は**スレッド固有**のため、WndProcスレッドとECSスレッドで**別の実体**を参照している
  - 結果: WndProcで蓄積したサンプルがECSから見えない → `No buffer found`
- **影響範囲**:
  - PointerBuffer（位置サンプル）
  - ButtonBuffer（ボタン押下状態）
  - WheelBuffer（ホイール回転）
  - ModifierBuffer（修飾キー）
  - すべてのthread_local!バッファがこの問題の影響を受ける可能性
- **症状**:
  - `push_pointer_sample(entity=7v0)`が60Hzで呼ばれている（WndProcスレッド）
  - `process_pointer_buffers(entity=7v0)`も60Hzで呼ばれている（ECSスレッド）
  - しかし`No buffer found`となる
  - PointerStateの更新が250msごとにしか起きない（別の更新経路が存在？）
- **調査メモ**:
  - WM_MOUSEMOVEは60Hzで来ている（正常）
  - `process_pointer_buffers`も60Hzで呼ばれている（正常）
  - entity IDの不一致も修正済み（Windowエンティティ対応追加）
  - WM_NCHITTESTがHTTRANSPARENTを返していた問題も修正済み
  - しかし依然として`POINTER_BUFFERS.with()`が異なるインスタンスを参照
- **仮説**:
  - wintfのアーキテクチャがWndProcとECSを異なるスレッドで動かしている
  - または、wndproc処理が別スレッドのメッセージループで動いている
  - ECSスケジュールはメインスレッドで実行されている
- **次のステップ**:
  1. wintf初期化コードでスレッドアーキテクチャを確認
  2. WndProcとECSのスレッドIDをログ出力して検証
  3. thread_local!をArc<Mutex<>>またはECS Resourceに置き換える設計変更を検討
  4. または、WndProcからECSへのクロススレッド通信機構（チャネル等）の導入
- **設計への影響**:
  - 当初設計でthread_local!を選択した根拠（ButtonBufferパターン踏襲）が前提条件違反
  - 「wndprocはメインスレッド固定」の前提が誤りだった可能性
  - drag/state.rsのDragStateも同じ問題を抱える可能性（要再検証）
- **ブロック状況**: 
  - PointerState更新が正常に動作しないため、ドラッグ実装が進められない
  - 優先度: **Critical** - 全体アーキテクチャに関わる問題

## 参考資料

- [SetCapture function (Microsoft Docs)](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setcapture) - マウスキャプチャAPI
- [Multiple Display Monitors (Microsoft Docs)](https://learn.microsoft.com/en-us/windows/win32/gdi/multiple-display-monitors) - マルチモニター座標系
- [bevy_ecs Documentation](https://docs.rs/bevy_ecs/latest/bevy_ecs/) - ECS Component/System設計
- 親仕様: `.kiro/specs/wintf-P0-event-system/requirements.md`
- Gap分析: `.kiro/specs/event-drag-system/gap-analysis.md`
- プロジェクト構造: `.kiro/steering/structure.md`
- 技術スタック: `.kiro/steering/tech.md`
