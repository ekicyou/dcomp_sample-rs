# ポインターイベントシステムのダブルクリック検出とシングルクリック抜けの修正 - 要件定義

## 1. 背景・目的

### 1.1 背景

wintfフレームワークのポインターイベントシステムにおいて、以下の2つの重大な問題が発生している：

1. **ダブルクリック無反応**: ダブルクリックしても`DoubleClick::Left`が検出されず、ハンドラで処理できない
2. **シングルクリック抜け**: クリックの約50%が反応せず、UIの操作性が著しく低下している

これらの問題により、`taffy_flex_demo`のGreenBox（緑の矩形）で：
- 左クリック → 色変更が約50%の確率でしか動作しない
- ダブルクリック → サイズ変更が全く動作しない

### 1.2 目的

- ダブルクリックを100%検出し、ハンドラで`state.double_click == DoubleClick::Left`が正常に判定できるようにする
- シングルクリックを100%検出し、すべてのクリックに反応するようにする

## 2. 現状分析

### 2.1 確認済みの動作状況

#### ✅ 正常に動作している部分

1. **Win32メッセージ受信**
   - `WM_LBUTTONDBLCLK`は正常に受信されている
   - ログ証拠: `handle_double_click_message] Double-click detected entity=3v0`

2. **ダブルクリック情報の記録と転送**
   - `DOUBLE_CLICK_THIS_FRAME`への記録は成功
   - `transfer_buffers_to_world()`での`PointerState.double_click`への適用も成功
   - ログ証拠: `[DOUBLE-CLICK] Applied to PointerState entity=7v0 double_click=Left`

3. **ボタン押下の記録**
   - `record_button_down()`は呼ばれている
   - ログ証拠: `[ButtonBuffer] record_button_down entity=7v0 button=Left`

#### ❌ 動作していない部分

1. **ハンドラでのdouble_click検出**
   - `on_green_box_pressed()`ハンドラ内で`state.double_click`が常に`None`
   - 期待値: `state.double_click == DoubleClick::Left`
   - 実際: ハンドラの`[Tunnel] GreenBox: Button pressed`ログすら出ない
   - **推測**: ハンドラ自体が呼ばれていない可能性が高い

2. **シングルクリックの安定性**
   - 約50%のクリックでハンドラが呼ばれない
   - ログパターン: `record_button_down entity=7v0`と`entity=3v0`が交互に出現

### 2.2 アーキテクチャの流れ

```
[Win32 Thread: WndProc]
  ↓ WM_LBUTTONDOWN / WM_LBUTTONDBLCLK
  ↓ handle_button_message() / handle_double_click_message()
  ↓ 
  ├─ record_button_down(entity, button) → BUTTON_BUFFERS[(entity, button)]
  └─ set_double_click(_entity, double_click) → DOUBLE_CLICK_THIS_FRAME = double_click

[ECS Thread: try_tick_world()]
  ↓ transfer_buffers_to_world(world)
  │   ├─ BUTTON_BUFFERS → PointerState.{left|right|...}_down = true
  │   ├─ DOUBLE_CLICK_THIS_FRAME → 全PointerState.double_click = double_click
  │   └─ リセット: BUTTON_BUFFERS.reset(), DOUBLE_CLICK_THIS_FRAME = None
  │
  ↓ dispatch_pointer_events(world)
      ├─ PointerStateを持つエンティティを収集
      ├─ Phase::Tunnel → Phase::Bubble でイベントディスパッチ
      └─ PointerState.{button}_down = false, .double_click = None (リセット)
```

### 2.3 関連ファイル

- **`crates/wintf/src/ecs/pointer/mod.rs`**
  - `transfer_buffers_to_world()` - thread_local → World への転送
  - `DOUBLE_CLICK_THIS_FRAME` - グローバルなダブルクリック状態
  - `set_double_click()` - ダブルクリック記録

- **`crates/wintf/src/ecs/pointer/dispatch.rs`**
  - `dispatch_pointer_events()` - イベントディスパッチとリセット

- **`crates/wintf/src/ecs/window_proc/handlers.rs`**
  - `handle_double_click_message()` - WM_LBUTTONDBLCLK ハンドラ
  - `handle_button_message()` - WM_LBUTTONDOWN/UP ハンドラ

- **`crates/wintf/examples/taffy_flex_demo.rs`**
  - `on_green_box_pressed()` - テスト用ハンドラ

### 2.4 実装済み修正（効果なし）

以下の修正を実施したが、問題は解決していない：

1. ✅ `transfer_buffers_to_world`でのコピー戦略実装（c1a9b0b）
2. ✅ ボタンチャタリング修正（d404b83）
3. ✅ ダブルクリック時の`record_button_down`追加（227a2d7）
4. ✅ ダブルクリック情報のグローバル化（57aee4d）

## 3. 疑われる原因

### 仮説1: ハンドラが呼ばれていない（最有力）

**症状**:
- `on_green_box_pressed()`の`[Tunnel] GreenBox: Button pressed`ログが出ない
- クリックの約50%で反応がない

**原因推測**:
- `dispatch_pointer_events()`が`PointerState`を正しく収集できていない
- または、イベントディスパッチのロジックに問題がある
- `left_down`フラグが`transfer_buffers_to_world()`と`dispatch_pointer_events()`の間でリセットされている

### 仮説2: PointerState複数残存問題

**症状**:
- ログに`entity=3v0`（Window）と`entity=7v0`（GreenBox）が混在
- マウスの場合、同時に1つのPointerStateのはずが複数存在

**原因推測**:
- `WM_MOUSEMOVE`でのPointerState削除が正常に動作していない
- 古いエンティティからの削除ログ（`PointerState moved, Leave marker inserted`）が出ていない可能性

### 仮説3: タイミング・順序問題

**症状**:
- `DOUBLE_CLICK_THIS_FRAME`は`entity=7v0`に適用されている（ログ確認済み）
- しかしハンドラでは`None`になっている

**原因推測**:
- `dispatch_pointer_events()`内でのリセットタイミングが早すぎる
- または、複数回`transfer_buffers_to_world()`が呼ばれて上書きされている

## 4. 要件

### 4.1 機能要件

#### FR1: ダブルクリック検出の完全動作
- **要件**: ダブルクリックを100%検出し、ハンドラで`state.double_click == DoubleClick::Left`が判定できる
- **成功基準**: 
  - GreenBoxをダブルクリックすると、必ずサイズが100x100⇔150x150にトグルする
  - ハンドラログ`[Tunnel] GreenBox: DOUBLE-CLICK detected`が出力される

#### FR2: シングルクリック検出の完全動作
- **要件**: すべてのシングルクリックが検出され、ハンドラが呼ばれる
- **成功基準**:
  - GreenBoxをクリックすると、必ず色が緑⇔黄緑にトグルする
  - クリック10回で10回とも反応する（100%成功率）

#### FR3: PointerStateの一意性保証
- **要件**: マウス使用時、同時に存在するPointerStateは最大1つ
- **成功基準**:
  - `debug`ログレベルで`PointerState moved, Leave marker inserted`が適切に出力される
  - 複数のエンティティに同時にPointerStateが存在しない

### 4.2 非機能要件

#### NFR1: ログの可読性
- **要件**: デバッグログが過剰にならず、重要な情報だけが`info`レベルで出力される
- **成功基準**: 1秒間のログ行数が20行以下（マウス移動除く）

#### NFR2: 既存機能の非破壊
- **要件**: ドラッグ移動、ホイール、修飾キーなど既存機能が影響を受けない
- **成功基準**: `taffy_flex_demo`のすべての既存機能が正常動作する

## 5. 制約条件

### 5.1 技術制約
- Rust + bevy_ecs (0.15) + windows-rs
- WndProcスレッドとECSスレッドは異なる（thread_local経由でデータ受け渡し）
- PointerStateはSparseSetストレージ（頻繁な挿入/削除に最適化）

### 5.2 設計制約
- WinUI3/WPFスタイルのTunnel/Bubbleイベントモデルを維持
- 将来のタッチパネル対応を考慮（複数PointerState対応）

## 6. スコープ外

以下は本仕様のスコープ外とする：

- タッチパネル/ペン入力の実装
- パフォーマンス最適化（機能修正を優先）
- ホイールイベント、ドラッグイベントの改善
- 他のサンプルアプリケーションへの影響調査

## 7. 検証方法

### 7.1 手動テスト

**テスト環境**: `taffy_flex_demo`

**テストケース1: ダブルクリック検出**
1. アプリを起動
2. GreenBoxを5回ダブルクリック
3. 期待結果: 毎回サイズが変わる（5/5成功）

**テストケース2: シングルクリック検出**
1. アプリを起動
2. GreenBoxを10回シングルクリック
3. 期待結果: 毎回色が変わる（10/10成功）

**テストケース3: クリック混在**
1. アプリを起動
2. GreenBoxをシングルクリック3回、ダブルクリック2回、シングルクリック3回
3. 期待結果: すべてに反応する

### 7.2 ログ検証

**検証ポイント**:
1. `handle_double_click_message` → `[DOUBLE-CLICK] Applied` → `[Tunnel] GreenBox: DOUBLE-CLICK detected` の順で出力
2. `record_button_down entity=7v0` → `[Tunnel] GreenBox: Button pressed` の順で出力
3. `entity=3v0`と`entity=7v0`が不自然に混在しない

## 8. 関連資料

- event-drag-system仕様（`C:\home\maz\git\dcomp_sample-rs\.kiro\specs\event-drag-system\`）
- 既存コミット履歴（c1a9b0b〜b0597d8）

## 9. 承認基準

- [ ] すべての機能要件（FR1-FR3）を満たす
- [ ] すべての非機能要件（NFR1-NFR2）を満たす
- [ ] 手動テストがすべて成功する
- [ ] ログ検証ポイントがすべてクリアする
