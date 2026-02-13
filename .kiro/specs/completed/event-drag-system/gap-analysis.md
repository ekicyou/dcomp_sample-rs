# ギャップ分析: event-drag-system

## 1. 現状調査

### 1.1 ドメイン関連資産

#### ディレクトリ構造
- `crates/wintf/src/ecs/pointer/` - ポインターイベントシステム（Phase<T>、イベントハンドラ）
- `crates/wintf/src/ecs/window_proc/` - Win32メッセージハンドリング（wndproc層）
- `crates/wintf/src/ecs/window.rs` - ウィンドウコンポーネント（Window、WindowPos、SetWindowPosCommand）
- `crates/wintf/src/ecs/layout/` - レイアウトシステム（GlobalArrangement、PhysicalPoint）
- `crates/wintf/examples/taffy_flex_demo.rs` - Tunnel/Bubbleフェーズデモ

#### 再利用可能なコンポーネント/サービス

**ポインターイベントシステム**
- `Phase<T>` enum（Tunnel/Bubble）: 2フェーズイベント伝播（`src/ecs/pointer/dispatch.rs:21`）
- `PointerState` component: マウス状態管理（物理ピクセル座標、ボタン状態、修飾キー）
- `ButtonBuffer`: ボタン押下/解放の一時記録バッファ（thread_local!管理）
- `OnPointerPressed`/`OnPointerReleased`/`OnPointerMoved` ハンドラコンポーネント（SparseSet）
- `dispatch_pointer_events()` システム: build_bubble_path + Tunnel/Bubbleフェーズ配信

**ウィンドウプロシージャ層**
- `WM_MOUSEMOVE` ハンドラ: 物理ピクセル座標抽出 + hit_test呼び出し（`src/ecs/window_proc/handlers.rs:452`）
- `WM_LBUTTONDOWN`/`WM_LBUTTONUP` ハンドラ: ButtonBufferへの記録（同上:758, 769）
- `SetWindowPosCommand` キュー: World借用競合回避（`src/ecs/window.rs:92`）
- `flush_window_pos_commands()`: キュー実行関数（同上）

**レイアウトシステム**
- `GlobalArrangement`: スクリーン物理座標系バウンディングボックス（`src/ecs/layout/arrangement.rs:77`）
- `PhysicalPoint`: 物理ピクセル座標型（`src/ecs/pointer/mod.rs:24`）
- `hit_test_in_window()`: ヒットテスト関数（物理ピクセル座標使用）

**ウィンドウ管理**
- `Window` component: HWNDラッパー
- `WindowPos` component: ウィンドウ位置（x, y, width, height）
- `SetWindowPos` API呼び出しパターン: DPI変更ハンドラ参照（`src/ecs/window_proc/handlers.rs:378`）

#### 支配的なアーキテクチャパターンと制約

**ECSアーキテクチャパターン**
- Component命名: `XxxGraphics`（GPU資源）、`XxxResource`（CPU資源）、マーカーはサフィックスなし
- Component storage: `#[component(storage = "SparseSet")]`（頻繁な挿入/削除）
- Hook pattern: `#[component(on_add = on_xxx_add)]`（自動コンポーネント挿入）
- System scheduling: `Input` schedule → ECSイベント配信 → World借用解放後にWin32 API実行

**Win32 API統合パターン**
- wndproc層でのメッセージ処理 → ECSコンポーネント/リソース更新 → Systemによる処理
- `thread_local!` + `RefCell` による状態管理（ButtonBuffer、SetWindowPosCommand）
- World借用競合回避: キューパターン（コマンド蓄積 → 借用解放後にフラッシュ）

**座標系の統一**
- 物理ピクセル座標（PhysicalPoint）: Win32メッセージ、ヒットテスト、GlobalArrangement全体で統一
- DPIスケーリング: GlobalArrangement.transformで管理（Win32メッセージは非スケーリング）

**イベント伝播制約**
- Phase<T>による2フェーズ必須: Tunnel（root→sender）→ Bubble（sender→root）
- ハンドラreturn trueでstopPropagation（WinUI3/WPF/DOM互換）
- sender（元ヒット対象）とentity（現在処理中）の引数分離

### 1.2 規約の抽出

#### 命名規則
- Files: `snake_case.rs`
- Modules: `snake_case`
- Types: `PascalCase`
- Component名: `DragConfig`（設定）、`DragState`（状態）、`OnDrag`（ハンドラ）
- Event名: `DragStartEvent`、`DragEvent`、`DragEndEvent`

#### レイヤリング
- COM wrapper (`com/`) ← ECS (`ecs/`) ← Message handling（ルート）
- wndproc層 → ECS System → Win32 API（双方向、World借用管理が必須）

#### 依存方向
- `unsafe`はCOMラッパー層に集約
- ECS Systemは`window_proc::handlers`モジュールの関数を直接呼ばない（逆はOK）

#### Import/Export パターン
```rust
// 標準ライブラリ
use std::sync::Arc;

// 外部クレート（アルファベット順）
use bevy_ecs::prelude::*;
use windows::Win32::Graphics::DirectComposition::*;

// 内部モジュール（相対パス）
use crate::com::dcomp::*;
use crate::ecs::window::*;
```

#### テスト配置とアプローチ
- `examples/` でインタラクティブデモ（taffy_flex_demo.rsパターン）
- `tests/` で自動テスト（ユニット/統合）

### 1.3 統合インターフェース

#### データモデル/スキーマ
- `PointerState`: マウス状態の正規化表現（物理座標、ボタン、修飾キー）
- `GlobalArrangement`: スクリーン物理座標系バウンディングボックス
- `WindowPos`: ウィンドウ位置（x, y, width, height）

#### APIクライアント
- Win32 API: `SetCapture`、`ReleaseCapture`、`SetWindowPos`、`GetCursorPos`
- ECS World: Component追加/削除/照会、Entity照会

#### 認証メカニズム
該当なし（デスクトップアプリケーション）

---

## 2. 要件実現可能性分析

### 2.1 技術ニーズリスト

#### データモデル
- **DragState**: ドラッグ状態管理（準備中/ドラッグ中、開始位置、現在位置、対象エンティティ、ボタン種別）
- **DragConfig**: ドラッグ設定（有効ボタン、閾値、制約）
- **DragConstraint**: ドラッグ範囲制約（制約エンティティ、軸制御）
- **DragStartEvent**、**DragEvent**、**DragEndEvent**: イベントデータ構造

#### API/サービス
- **ドラッグ閾値判定**: wndprocでのユークリッド距離計算（5物理ピクセルデフォルト）
- **マウスキャプチャ管理**: SetCapture/ReleaseCaptureのラッパー
- **ウィンドウ移動計算**: マウス移動量→WindowPos更新ロジック
- **制約適用**: GlobalArrangement.boundsによるクランプ
- **Phase<T>統合**: DragStart/Drag/DragEndイベントのTunnel/Bubble配信

#### UIコンポーネント
該当なし（システムレベル機能）

#### ビジネスルール/バリデーション
- ドラッグ閾値超過判定: `sqrt(dx^2 + dy^2) >= threshold`
- Escキー押下でキャンセル
- エンティティ削除でキャンセル
- WM_CANCELMODEでキャンセル

#### 非機能要件
- **パフォーマンス**: 16ms以内のイベント処理（60fps維持）
- **レスポンス**: リアルタイムウィンドウ追従
- **信頼性**: イベント取りこぼしなし、状態整合性保証

### 2.2 ギャップと制約の特定

#### 欠落している機能

**新規コンポーネント（7個）**
1. `DragState`: wndproc層でのドラッグ状態管理（thread_local!、ButtonBufferパターン）
2. `DragConfig`: エンティティごとのドラッグ設定（SparseSet）
3. `DragConstraint`: エンティティごとのドラッグ制約（SparseSet）
4. `OnDragStart`: DragStartイベントハンドラ（SparseSet）
5. `OnDrag`: Dragイベントハンドラ（SparseSet）
6. `OnDragEnd`: DragEndイベントハンドラ（SparseSet）
7. `DraggingMarker`: ドラッグ中マーカー（一時的、SparseSet）

**新規System（3個）**
1. `dispatch_drag_events()`: DragStart/Drag/DragEndのPhase<T>配信（dispatch_pointer_eventsパターン）
2. `apply_window_drag_movement()`: WindowPos更新 + SetWindowPosCommandキュー登録
3. `cleanup_drag_state()`: エンティティ削除時のクリーンアップ

**wndproc層拡張（4箇所）**
1. `WM_MOUSEMOVE`: ドラッグ閾値判定 + DragStart/Dragイベント生成
2. `WM_LBUTTONDOWN`/`WM_RBUTTONDOWN`/`WM_MBUTTONDOWN`: ドラッグ準備状態記録
3. `WM_LBUTTONUP`/`WM_RBUTTONUP`/`WM_MBUTTONUP`: DragEndイベント生成 + 状態クリア
4. `WM_KEYDOWN` (Esc): ドラッグキャンセル
5. `WM_CANCELMODE`: 強制キャンセル

**taffy_flex_demo拡張**
- ウィンドウドラッグ可能なウィジェット追加（例: トップレベルコンテナ）
- OnDragStartハンドラ実装例
- ログ出力による動作確認

#### 不明点（研究必要項目）

**Research Needed 1: マウスキャプチャとウィンドウ外移動**
- 課題: SetCapture中にウィンドウを画面外に移動した場合、ReleaseCaptureのタイミング
- 影響範囲: Requirement 14.2（ウィンドウ外イベント受信）
- 調査内容: SetCapture APIのドキュメント確認、実験コード作成

**Research Needed 2: マルチモニター環境での仮想スクリーン座標**
- 課題: 負の座標を含む仮想スクリーン座標系とGlobalArrangement.boundsの整合性
- 影響範囲: Requirement 8.2（仮想スクリーン座標系）
- 調査内容: GetSystemMetrics(SM_XVIRTUALSCREEN/SM_YVIRTUALSCREEN)の動作確認

**Research Needed 3: 高DPI環境でのドラッグ閾値**
- 課題: 物理ピクセル5pxが高DPI環境で体感的に小さすぎる可能性
- 影響範囲: Requirement 1.2（ドラッグ閾値）
- 調査内容: DPI別の閾値調整必要性の検証

#### 既存アーキテクチャからの制約

**制約1: World借用競合**
- 内容: SetWindowPosをSystem内から直接呼ぶとWorld借用と競合
- 影響: Requirement 6.3（SetWindowPos使用）
- 対応: SetWindowPosCommandキューパターン必須（既存パターン流用可能）

**制約2: Phase<T>での型統一**
- 内容: Phase<DragStartEvent>、Phase<DragEvent>、Phase<DragEndEvent>の3種類が必要
- 影響: Requirement 7（イベント配信）
- 対応: dispatch_pointer_eventsをテンプレート化して3種類インスタンス化

**制約3: wndproc層の単一スレッド性**
- 内容: wndprocはメインスレッド固定、Worldアクセスは慎重に
- 影響: Requirement 13.1（wndproc処理）
- 対応: thread_local! + RefCellパターン（ButtonBufferと同じ）

### 2.3 複雑性シグナル

- **CRUD**: 該当なし（システム機能）
- **アルゴリズムロジック**: 中程度（ユークリッド距離、制約クランプ）
- **ワークフロー**: 複雑（準備→閾値判定→ドラッグ中→終了/キャンセルの状態遷移）
- **外部統合**: 中程度（Win32 API、Phase<T>イベントシステム）

---

## 3. 実装アプローチオプション

### Option A: 既存コンポーネント拡張

**拡張対象ファイル/モジュール**
1. `crates/wintf/src/ecs/window_proc/handlers.rs`:
   - `WM_MOUSEMOVE`: ドラッグ閾値判定ロジック追加（60行程度）
   - `WM_LBUTTONDOWN`等: ドラッグ準備状態記録（各10行程度）
   - `WM_KEYDOWN`: Escキーハンドリング追加（30行程度）
2. `crates/wintf/src/ecs/pointer/dispatch.rs`:
   - `dispatch_pointer_events`のジェネリック化（50行程度リファクタ）
   - 3種類のドラッグイベント向けインスタンス化関数追加
3. `crates/wintf/examples/taffy_flex_demo.rs`:
   - ウィンドウドラッグ実装例追加（100行程度）

**互換性評価**
- ✅ 既存のPhase<T>インターフェース尊重
- ✅ ButtonBufferパターンと同じthread_local!設計
- ✅ 後方互換性維持（既存ポインターイベントに影響なし）

**複雑性と保守性**
- `handlers.rs`が肥大化（現在800行 → 1000行程度）
- ドラッグ状態管理がwndproc層に密結合
- 単一ファイルでの変更追跡が困難

**トレードオフ**
- ✅ 最小限の新規ファイル（学習コスト低）
- ✅ 既存パターンとの一貫性
- ❌ handlers.rsの責務肥大化
- ❌ ドラッグ機能のモジュール境界不明瞭

### Option B: 新規コンポーネント作成

**新規作成の根拠**
- ドラッグシステムは独立した責務（ポインターイベントの上位機能）
- 状態遷移ロジックが複雑（準備→閾値判定→ドラッグ→終了/キャンセル）
- 将来の拡張性（非ウィンドウ移動型ドラッグの追加可能性）

**統合ポイント**
1. **新規モジュール**: `crates/wintf/src/ecs/drag/`
   - `mod.rs`: 公開API（DragConfig、DragState、イベント型）
   - `state.rs`: DragState管理（thread_local!）
   - `dispatch.rs`: イベント配信（dispatch_pointer_eventsパターン）
   - `systems.rs`: apply_window_drag_movement、cleanup_drag_state
2. **wndproc統合**: `crates/wintf/src/ecs/window_proc/handlers.rs`
   - 各メッセージハンドラから`drag::update_drag_state()`呼び出し（各5-10行）
3. **ECS World統合**: `crates/wintf/src/ecs/world.rs`
   - `Input` scheduleに`dispatch_drag_events`登録
   - `PostUpdate` scheduleに`apply_window_drag_movement`登録

**責務境界**
- `drag/` モジュール: ドラッグロジック専任
- `window_proc/handlers.rs`: Win32メッセージ受信のみ（ドラッグ状態更新を委譲）
- `pointer/dispatch.rs`: 既存ポインターイベントのみ（ドラッグは別系統）

**トレードオフ**
- ✅ 明確な責務分離（SRP準拠）
- ✅ 独立したテスト容易性
- ✅ 既存コードの複雑性増加なし
- ❌ 新規ファイル追加（4ファイル）
- ❌ 初期学習コスト増

### Option C: ハイブリッドアプローチ

**組み合わせ戦略**
- **Phase 1（最小実装）**: Option A（既存拡張）で基本機能実装
  - handlers.rsにドラッグロジック統合
  - DragStart/DragEnd/Dragイベント配信
  - taffy_flex_demoでの動作確認
- **Phase 2（リファクタリング）**: Option B（新規モジュール）への分離
  - ドラッグロジックを`ecs/drag/`に抽出
  - handlers.rsの責務整理
  - 単体テスト追加

**段階的実装**
1. Phase 1: 1週間（基本機能実装 + デモ統合）
2. Phase 2: 3日（モジュール分離 + テスト追加）
3. 合計: 約10日

**リスク軽減**
- ✅ 早期の動作確認（Phase 1）
- ✅ 段階的なアーキテクチャ改善
- ✅ ロールバック可能（Phase 1で十分なら停止）

**トレードオフ**
- ✅ 柔軟な実装戦略
- ✅ リスク分散
- ❌ 計画の複雑性増加
- ❌ リファクタリング工数

---

## 4. 実装複雑性とリスク

### 4.1 工数見積もり: **M (5日)**

**内訳**
- 新規モジュール作成: 2日（drag/mod.rs、state.rs、dispatch.rs、systems.rs）
- wndproc統合: 1日（handlers.rs修正、各メッセージハンドラ）
- ECS World統合: 0.5日（world.rs、schedule登録）
- taffy_flex_demo拡張: 1日（ウィンドウドラッグ実装例）
- テスト実装: 0.5日（単体テスト、統合テスト）

**根拠**
- S判定要因: 既存パターン（ButtonBuffer、SetWindowPosCommand、Phase<T>）の流用可能
- M判定要因: 新規モジュール作成、状態遷移ロジック、複数メッセージハンドラ統合
- L判定除外: 未知の技術なし、外部依存なし、アーキテクチャ変更なし

### 4.2 リスク評価: **Medium**

**リスク要因**
1. **新規パターン（状態遷移）**: ドラッグ準備→閾値判定→ドラッグ中→終了の遷移ロジック
   - 軽減策: 既存ButtonBufferパターンの応用、状態図作成
2. **複数メッセージハンドラ統合**: WM_MOUSEMOVE、WM_*BUTTON*、WM_KEYDOWN、WM_CANCELMODE
   - 軽減策: 各ハンドラでの責務最小化（drag::update_drag_state()に委譲）
3. **Phase<T>ジェネリック化**: 既存dispatch_pointer_eventsのテンプレート化
   - 軽減策: 段階的リファクタリング（元コードを残したまま新関数追加）
4. **マルチモニター座標系**: 仮想スクリーン座標とGlobalArrangementの整合性
   - 軽減策: Research Needed 2（設計フェーズで調査）

**High判定除外要因**
- ✅ Win32 API知識: 既存（SetCapture/ReleaseCapture使用例あり）
- ✅ ECS統合: 既存パターン流用可能
- ✅ アーキテクチャ: 既存のレイヤード構造維持
- ✅ パフォーマンス: 既存ポインターイベントシステムで実証済み

**Low判定除外要因**
- ❌ 新規モジュール作成必要
- ❌ 状態遷移ロジック（既存に類例なし）
- ❌ 複数メッセージハンドラの協調動作

---

## 5. 推奨事項

### 5.1 要件-資産マッピング表

| 要件 | 既存資産 | ギャップ | タグ |
|------|---------|---------|------|
| Req 1: ドラッグ開始検出 | ButtonBuffer（押下記録）<br>WM_MOUSEMOVE（座標取得） | ・閾値判定ロジック<br>・DragState管理 | Missing |
| Req 2: ドラッグ中イベント | WM_MOUSEMOVE<br>PointerState | ・Dragイベント構造<br>・Phase<T>配信 | Missing |
| Req 3: ドラッグ終了検出 | WM_*BUTTONUP<br>ButtonBuffer | ・DragEndイベント構造<br>・ReleaseCapture呼び出し | Missing |
| Req 4: DragEnd詳細情報 | PointerState（座標）<br>Instant（時刻） | ・総移動量計算<br>・キャンセルフラグ | Missing |
| Req 5: ドラッグキャンセル | - | ・Escキーハンドラ<br>・WM_CANCELMODE処理<br>・エンティティ削除検知 | Missing |
| Req 6: ウィンドウドラッグ移動 | SetWindowPosCommand<br>WindowPos | ・DragConfig（有効/無効）<br>・WindowPos更新System | Missing |
| Req 7: イベント配信 | Phase<T><br>dispatch_pointer_events | ・Phase<DragEvent>型<br>・dispatch_drag_events関数 | Missing |
| Req 8: マルチモニター対応 | GlobalArrangement（物理座標） | ・仮想スクリーン座標変換 | Unknown |
| Req 9: ドラッグ制約 | GlobalArrangement.bounds | ・DragConstraint component<br>・クランプ処理 | Missing |
| Req 10: ECS統合 | bevy_ecs<br>Component/System | ・7個のComponent定義<br>・3個のSystem定義 | Missing |
| Req 11: ドラッグ位置通知 | - | ・DragEndイベント拡張<br>・モニター情報取得 | Missing |
| Req 12: taffy_flex_demo統合 | taffy_flex_demo.rs<br>OnPointerPressed例 | ・OnDragStartハンドラ例<br>・ログ出力 | Missing |
| Req 13: 処理場所と座標系 | wndproc層<br>PhysicalPoint | ・DragState in thread_local! | Missing |
| Req 14: マウスキャプチャ | - | ・SetCapture/ReleaseCapture呼び出し<br>・WM_CANCELMODE処理 | Missing/Unknown |

### 5.2 推奨アプローチ

**Option B: 新規コンポーネント作成**を推奨

**選定理由**
1. **責務分離**: ドラッグシステムは独立した機能であり、既存ポインターイベントシステムとは別の抽象化レベル
2. **保守性**: handlers.rsの肥大化を防ぎ、ドラッグ機能の変更影響範囲を限定
3. **テスト容易性**: 独立モジュールにより単体テストが容易
4. **将来拡張性**: 非ウィンドウ移動型ドラッグ（例: ファイルドラッグ&ドロップ）追加時の基盤

**重要な決定事項**
1. **モジュール構造**: `crates/wintf/src/ecs/drag/` を新規作成
   - `mod.rs`: 公開API
   - `state.rs`: DragState管理（thread_local!）
   - `dispatch.rs`: イベント配信
   - `systems.rs`: ECS System群
2. **wndproc統合**: handlers.rsから`drag::update_drag_state()`を呼び出す最小結合
3. **Phase<T>ジェネリック化**: `dispatch_pointer_events`をテンプレート関数化してドラッグイベントでも再利用
4. **SetWindowPosCommand流用**: 既存のキューパターンをそのまま使用

### 5.3 設計フェーズへの引き継ぎ

#### Research Needed項目
1. **マウスキャプチャ挙動**: SetCapture中のウィンドウ外移動、ReleaseCapture後のイベント受信停止タイミング
2. **仮想スクリーン座標**: マルチモニター環境での負座標、GetSystemMetrics API動作確認
3. **高DPI閾値**: DPI別のドラッグ閾値調整必要性、ユーザビリティ検証

#### 技術的考慮事項
1. **DragState設計**: 状態遷移図（Idle → Preparing → Dragging → Done/Cancelled）
2. **イベントデータ構造**: DragStartEvent、DragEvent、DragEndEventのフィールド定義
3. **制約適用タイミング**: WindowPos更新時 vs Dragイベント生成時
4. **エラーハンドリング**: SetCapture/ReleaseCapture失敗時の動作、WM_CANCELMODE処理

#### 実装優先順位
1. **Phase 1（基本機能）**: DragStart/Drag/DragEndイベント配信、ウィンドウ移動
2. **Phase 2（拡張機能）**: ドラッグ制約、キャンセル処理
3. **Phase 3（統合）**: taffy_flex_demo拡張、テスト追加

---

## 6. 付録

### 6.1 既存コードパターン参照

**ButtonBufferパターン（DragState実装の参考）**
```rust
// crates/wintf/src/ecs/pointer/mod.rs:299
pub struct ButtonBuffer {
    down_entity: Option<Entity>,
    down_timestamp: Instant,
    down_position: PhysicalPoint,
}

thread_local! {
    pub(crate) static BUTTON_BUFFERS: RefCell<HashMap<(Entity, PointerButton), ButtonBuffer>> = 
        RefCell::new(HashMap::new());
}
```

**SetWindowPosCommandパターン（ウィンドウ移動の参考）**
```rust
// crates/wintf/src/ecs/window.rs:92
pub struct SetWindowPosCommand {
    pub hwnd: HWND,
    pub x: i32, pub y: i32,
    pub width: i32, pub height: i32,
    pub flags: SET_WINDOW_POS_FLAGS,
}

thread_local! {
    static WINDOW_POS_COMMANDS: RefCell<Vec<SetWindowPosCommand>> = 
        const { RefCell::new(Vec::new()) };
}

impl SetWindowPosCommand {
    pub fn enqueue(cmd: SetWindowPosCommand) { /* ... */ }
    pub fn flush() { /* World借用解放後に実行 */ }
}
```

**Phase<T>配信パターン（ドラッグイベント配信の参考）**
```rust
// crates/wintf/src/ecs/pointer/dispatch.rs:21
pub enum Phase<T> {
    Tunnel(T),  // 親→子
    Bubble(T),  // 子→親
}

fn build_bubble_path(world: &World, start: Entity) -> Vec<Entity> {
    // sender → root の順で格納
}

// Tunnel: root → sender
for &entity in path.iter().rev() {
    if handler(world, sender, entity, &Phase::Tunnel(state.clone())) {
        return; // stopPropagation
    }
}

// Bubble: sender → root
for &entity in path.iter() {
    if handler(world, sender, entity, &Phase::Bubble(state.clone())) {
        return;
    }
}
```

### 6.2 参考ドキュメント

- 親仕様: `.kiro/specs/wintf-P0-event-system/requirements.md`
- イベントシステム設計: `doc/spec/08-event-system.md`
- ヒットテスト設計: `doc/spec/09-hit-test.md`
- プロジェクト構造: `.kiro/steering/structure.md`
- 技術スタック: `.kiro/steering/tech.md`

### 6.3 想定される課題

**課題1: ドラッグ中のエンティティ削除**
- 現象: ドラッグ対象エンティティが削除されたときの状態不整合
- 対応: cleanup_drag_stateシステムで自動キャンセル（Requirement 5.5）

**課題2: 複数ボタン同時ドラッグ**
- 現象: 左ボタンドラッグ中に右ボタン押下
- 対応: DragState内でボタン種別を記録、混在を禁止

**課題3: ドラッグ閾値の体感差**
- 現象: 高DPI環境で5pxが小さすぎる可能性
- 対応: Research Needed 3で調査、必要に応じてDPI依存閾値実装

---

_このギャップ分析は設計フェーズでの意思決定を支援するための情報提供を目的としています。最終的な実装方針は設計フェーズで決定されます。_
