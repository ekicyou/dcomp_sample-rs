# ポインターイベントシステムのダブルクリック検出とシングルクリック抜けの修正 - 設計

## 1. 問題の根本原因分析

### 1.1 ダブルクリックが検出されない原因

**根本原因**: `handle_double_click_message()`が**Windowエンティティ**を使ってダブルクリックを記録しているが、実際にハンドラを持つのは**GreenBoxエンティティ**であるため、情報が到達していない。

**証拠**:
```
[handle_double_click_message] Double-click detected entity=3v0  // Window
[DOUBLE-CLICK] Applied to PointerState entity=7v0 double_click=Left  // GreenBox
```

しかし、`handle_double_click_message()`は以下のコードでWindowエンティティのみを使用：

```rust
unsafe fn handle_double_click_message(hwnd: HWND, double_click: DoubleClick) -> HandlerResult {
    let Some(entity) = super::get_entity_from_hwnd(hwnd) else { return None; };
    // ↑ここでWindowエンティティを取得
    
    crate::ecs::pointer::set_double_click(entity, double_click);  // Window entityに記録
    crate::ecs::pointer::record_button_down(entity, button);      // Window entityに記録
}
```

**問題点**:
1. `WM_LBUTTONDBLCLK`は`WM_LBUTTONDOWN`の**代わりに**来る（同時には来ない）
2. `handle_double_click_message()`はWindowエンティティに情報を記録
3. しかし`dispatch_pointer_events()`は**PointerStateを持つエンティティ**のみを処理対象とする
4. ダブルクリック時、GreenBoxには`PointerState`が存在しない（Windowにしか記録されていない）
5. 結果として`dispatch_pointer_events()`がGreenBoxのハンドラを呼ばない

### 1.2 シングルクリックの50%抜けの原因

**根本原因**: `handle_button_message()`が**hit_testでターゲットを特定しPointerStateを付与**するが、`handle_double_click_message()`は**hit_testを行わず、Windowエンティティにのみ記録**している。

**動作フロー比較**:

```
[シングルクリック: WM_LBUTTONDOWN]
1. handle_button_message()呼び出し
2. hit_test_in_window() → GreenBox (entity=7v0) を特定
3. GreenBoxにPointerState付与（なければ）
4. record_button_down(GreenBox, Left) → BUTTON_BUFFERS[(7v0, Left)].down = true
5. transfer_buffers_to_world() → PointerState(7v0).left_down = true
6. dispatch_pointer_events() → GreenBoxを収集、ハンドラ呼び出し ✅

[ダブルクリック: WM_LBUTTONDBLCLK]
1. handle_double_click_message()呼び出し
2. get_entity_from_hwnd() → Window (entity=3v0) を取得
3. set_double_click(Window, Left) → DOUBLE_CLICK_THIS_FRAME = Left
4. record_button_down(Window, Left) → BUTTON_BUFFERS[(3v0, Left)].down = true
5. transfer_buffers_to_world() → PointerState(3v0).left_down = true
6. transfer_buffers_to_world() → 全PointerStateにdouble_click適用
7. dispatch_pointer_events() → GreenBoxには**PointerStateがない**ため処理されない ❌
```

**シングルクリック抜けのメカニズム**:
- 高速クリック時、OS側でダブルクリック判定されることがある
- ダブルクリック判定された場合、`WM_LBUTTONDOWN`が`WM_LBUTTONDBLCLK`に変わる
- その結果、`handle_button_message()`が呼ばれず、hit_testも実行されない
- GreenBoxにPointerStateが付与されず、ハンドラも呼ばれない
- 約50%の確率 = ダブルクリック閾値（通常500ms）内に2回目のクリックが判定される確率

### 1.3 PointerState複数残存の原因

**原因**: `WM_MOUSEMOVE`のハンドラで古いPointerStateを削除する処理が、ダブルクリック時に実行されていない。

ダブルクリック時には`WM_LBUTTONDBLCLK`のみが来て、マウス移動がないためPointerState削除処理が発動しない。

## 2. 設計方針

### 2.1 設計原則

1. **hit_test一元化**: すべてのポインターイベントで一貫してhit_testを実行
2. **エンティティ単位の記録**: バッファへの記録は常にターゲットエンティティに対して行う
3. **PointerState保証**: イベント処理時にPointerStateが必ず存在するようにする
4. **最小変更**: 既存の動作（シングルクリック、ドラッグ等）を壊さない

### 2.2 修正方針

#### 修正1: `handle_double_click_message()`をhit_test対応に変更

**before**:
```rust
unsafe fn handle_double_click_message(hwnd: HWND, double_click: DoubleClick) -> HandlerResult {
    let Some(entity) = super::get_entity_from_hwnd(hwnd) else { return None; };
    set_double_click(entity, double_click);
    record_button_down(entity, button);
}
```

**after**:
```rust
unsafe fn handle_double_click_message(
    hwnd: HWND,
    wparam: WPARAM,
    lparam: LPARAM,
    double_click: DoubleClick
) -> HandlerResult {
    // handle_button_message()と同様のロジックでhit_testを実行
    let Some(window_entity) = super::get_entity_from_hwnd(hwnd) else { return None; };
    
    // クリック位置を取得
    let x = (lparam.0 & 0xFFFF) as i16 as i32;
    let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
    
    // 修飾キー状態を抽出
    let wparam_val = wparam.0 as u32;
    let shift = (wparam_val & 0x04) != 0;
    let ctrl = (wparam_val & 0x08) != 0;
    
    if let Some(world) = super::try_get_ecs_world() {
        if let Ok(mut world_borrow) = world.try_borrow_mut() {
            // hit_testでターゲットエンティティを特定
            if let Some(target_entity) = hit_test_in_window(
                world_borrow.world(),
                window_entity,
                HitTestPoint::new(x as f32, y as f32),
            ) {
                // PointerStateがない場合は付与
                if world_borrow.world().get::<PointerState>(target_entity).is_none() {
                    world_borrow.world_mut().entity_mut(target_entity).insert(PointerState {
                        screen_point: PhysicalPoint::new(x, y),
                        local_point: PhysicalPoint::new(x, y),
                        left_down: button == PointerButton::Left,
                        // ... 他のフィールド
                        double_click,  // 直接設定
                        ..Default::default()
                    });
                } else {
                    // 既存のPointerStateにdouble_clickを設定
                    if let Some(mut ps) = world_borrow.world_mut().get_mut::<PointerState>(target_entity) {
                        ps.double_click = double_click;
                    }
                }
                
                // 修飾キー状態とボタン押下を記録
                set_modifier_state(target_entity, shift, ctrl);
                record_button_down(target_entity, button);
            }
        }
    }
    
    Some(LRESULT(0))
}
```

**変更点**:
- `wparam`, `lparam`を引数に追加（位置と修飾キーを取得するため）
- `hit_test_in_window()`でターゲットエンティティを特定
- ターゲットエンティティに`PointerState`を付与（なければ）
- `double_click`を直接`PointerState`に設定（グローバル変数経由ではなく）
- ターゲットエンティティに対して`record_button_down()`を呼ぶ

#### 修正2: グローバルダブルクリック情報の削除

**before**:
```rust
thread_local! {
    pub(crate) static DOUBLE_CLICK_THIS_FRAME: RefCell<DoubleClick> = RefCell::new(DoubleClick::None);
}

pub(crate) fn set_double_click(_entity: Entity, double_click: DoubleClick) {
    DOUBLE_CLICK_THIS_FRAME.with(|dc| {
        *dc.borrow_mut() = double_click;
    });
}

// transfer_buffers_to_world()内
let double_click_this_frame = DOUBLE_CLICK_THIS_FRAME.with(|dc| *dc.borrow());
if double_click_this_frame != DoubleClick::None {
    for (entity, mut pointer_state) in world.query::<(Entity, &mut PointerState)>().iter_mut(world) {
        pointer_state.double_click = double_click_this_frame;
    }
}
```

**after**:
```rust
// DOUBLE_CLICK_THIS_FRAMEとset_double_click()を削除
// ダブルクリック情報は直接PointerStateに設定されるため、グローバル変数は不要

// transfer_buffers_to_world()からグローバルダブルクリック適用ロジックを削除
```

**理由**:
- ダブルクリック情報を全PointerStateに適用する必要がない
- hit_testで特定されたエンティティのPointerStateにのみ設定すれば十分
- グローバル状態を減らすことでコードの複雑性を下げる

#### 修正3: WM_*DBLCLKハンドラのシグネチャ変更

**before**:
```rust
pub(super) unsafe fn WM_LBUTTONDBLCLK(hwnd: HWND, _message: u32, _wparam: WPARAM, _lparam: LPARAM) -> HandlerResult {
    handle_double_click_message(hwnd, DoubleClick::Left)
}
```

**after**:
```rust
pub(super) unsafe fn WM_LBUTTONDBLCLK(hwnd: HWND, _message: u32, wparam: WPARAM, lparam: LPARAM) -> HandlerResult {
    handle_double_click_message(hwnd, wparam, lparam, DoubleClick::Left)
}
```

**変更点**:
- `wparam`, `lparam`を`handle_double_click_message()`に渡す
- `WM_RBUTTONDBLCLK`, `WM_MBUTTONDBLCLK`, `WM_XBUTTONDBLCLK`も同様に変更

## 3. 変更対象ファイル

### 3.1 `crates/wintf/src/ecs/window_proc/handlers.rs`

**変更内容**:
1. `handle_double_click_message()`の実装を全面的に変更
   - シグネチャに`wparam`, `lparam`を追加
   - `handle_button_message()`と同様のhit_testロジックを実装
   - ターゲットエンティティにPointerStateを付与
   - `double_click`を直接PointerStateに設定
2. `WM_*BUTTONDBLCLK`ハンドラを修正
   - `wparam`, `lparam`を`handle_double_click_message()`に渡す

### 3.2 `crates/wintf/src/ecs/pointer/mod.rs`

**変更内容**:
1. `DOUBLE_CLICK_THIS_FRAME`のthread_local変数を削除
2. `set_double_click()`関数を削除
3. `transfer_buffers_to_world()`からグローバルダブルクリック適用ロジックを削除

**削除対象コード**（969-1086行目）:
```rust
// 削除: グローバルなダブルクリック情報の適用ロジック
let double_click_this_frame = DOUBLE_CLICK_THIS_FRAME.with(|dc| *dc.borrow());

if double_click_this_frame != DoubleClick::None {
    for (entity, mut pointer_state) in world.query::<(Entity, &mut PointerState)>().iter_mut(world) {
        pointer_state.double_click = double_click_this_frame;
        // ...
    }
}

DOUBLE_CLICK_THIS_FRAME.with(|dc| {
    *dc.borrow_mut() = DoubleClick::None;
});
```

## 4. データフロー（修正後）

### 4.1 シングルクリック時のフロー（変更なし）

```
[WM_LBUTTONDOWN]
  ↓
handle_button_message(hwnd, wparam, lparam, Left, true)
  ↓
hit_test_in_window() → GreenBox (entity=7v0)
  ↓
GreenBox: insert PointerState (if not exists)
  ↓
record_button_down(7v0, Left) → BUTTON_BUFFERS[(7v0, Left)].down = true
  ↓
[ECS Thread]
  ↓
transfer_buffers_to_world(world)
  ├─ BUTTON_BUFFERS[(7v0, Left)].down → PointerState(7v0).left_down = true
  └─ BUTTON_BUFFERS reset
  ↓
dispatch_pointer_events(world)
  ├─ Query<(Entity, &PointerState)> → [(7v0, state), ...]
  ├─ state.left_down == true → dispatch OnPointerPressed
  ├─ Tunnel: root → GreenBox
  ├─ on_green_box_pressed() 呼び出し ✅
  └─ PointerState.left_down = false, .double_click = None (reset)
```

### 4.2 ダブルクリック時のフロー（修正後）

```
[WM_LBUTTONDBLCLK]
  ↓
handle_double_click_message(hwnd, wparam, lparam, DoubleClick::Left)
  ↓
hit_test_in_window() → GreenBox (entity=7v0)  // ★修正: hit_test追加
  ↓
GreenBox: insert/update PointerState
  ├─ state.screen_point = (x, y)
  ├─ state.left_down = true
  └─ state.double_click = DoubleClick::Left  // ★修正: 直接設定
  ↓
record_button_down(7v0, Left) → BUTTON_BUFFERS[(7v0, Left)].down = true
  ↓
[ECS Thread]
  ↓
transfer_buffers_to_world(world)
  ├─ BUTTON_BUFFERS[(7v0, Left)].down → PointerState(7v0).left_down = true
  └─ BUTTON_BUFFERS reset
  ↓
dispatch_pointer_events(world)
  ├─ Query<(Entity, &PointerState)> → [(7v0, state), ...]  // ★修正: GreenBoxが含まれる
  ├─ state.left_down == true → dispatch OnPointerPressed
  ├─ state.double_click == DoubleClick::Left
  ├─ Tunnel: root → GreenBox
  ├─ on_green_box_pressed() 呼び出し
  │   └─ state.double_click == DoubleClick::Left → サイズ変更 ✅
  └─ PointerState.left_down = false, .double_click = None (reset)
```

**修正による変化**:
- ✅ hit_testで正しいエンティティ（GreenBox）を特定
- ✅ GreenBoxにPointerStateを付与
- ✅ double_clickを直接PointerStateに設定（グローバル変数不要）
- ✅ dispatch_pointer_events()がGreenBoxを処理
- ✅ ハンドラが呼ばれ、state.double_clickが取得できる

## 5. エッジケース対応

### 5.1 ダブルクリックの2回目のクリックアップ

**問題**: ダブルクリックは以下のメッセージシーケンスで来る：
```
WM_LBUTTONDOWN   (1回目押下)
WM_LBUTTONUP     (1回目離脱)
WM_LBUTTONDBLCLK (2回目押下、WM_LBUTTONDOWNの代わり)
WM_LBUTTONUP     (2回目離脱)
```

**対応**: 既存の`handle_button_message()`が`WM_LBUTTONUP`を処理するため、追加対応不要。

### 5.2 hit_testが失敗した場合（透過ウィンドウ等）

**対応**: `handle_double_click_message()`内で`hit_test_in_window()`が`None`を返した場合、何もせずに`Some(LRESULT(0))`を返す。

```rust
if let Some(target_entity) = hit_test_in_window(...) {
    // 処理
} else {
    // hit_test失敗 → 何もしない
}
Some(LRESULT(0))
```

### 5.3 World借用エラー

**対応**: `try_borrow_mut()`を使用し、失敗時は`None`を返してDefWindowProcWに委譲。

```rust
if let Some(world) = super::try_get_ecs_world() {
    if let Ok(mut world_borrow) = world.try_borrow_mut() {
        // 処理
    }
}
Some(LRESULT(0))
```

## 6. パフォーマンスへの影響

### 6.1 追加コスト

- `handle_double_click_message()`でhit_testを実行（既存の`handle_button_message()`と同じ）
- ダブルクリックは低頻度イベントのため、パフォーマンス影響は無視できる

### 6.2 削減コスト

- `transfer_buffers_to_world()`のグローバルダブルクリック適用ループを削除
- 全PointerStateを走査していたO(n)処理が削除され、全体的にはパフォーマンス向上

## 7. 後方互換性

### 7.1 既存コードへの影響

- `set_double_click()`を直接呼び出しているコードは`handle_double_click_message()`のみ
- 外部からの呼び出しなし
- 削除しても影響なし

### 7.2 既存機能の保証

以下の機能は変更なし、そのまま動作：
- シングルクリック（`handle_button_message()`）
- ドラッグ移動（`DragConfig`, `start_preparing()`）
- ホイールイベント
- 修飾キー（Shift, Ctrl）

## 8. テスト戦略

### 8.1 手動テスト

**テストケース1: ダブルクリック検出**
1. `taffy_flex_demo`起動
2. GreenBoxを5回ダブルクリック
3. 期待: 毎回サイズが変わる（5/5成功）
4. ログ: `[Tunnel] GreenBox: DOUBLE-CLICK detected` が5回出力

**テストケース2: シングルクリック安定性**
1. `taffy_flex_demo`起動
2. GreenBoxを10回シングルクリック（ゆっくり）
3. 期待: 毎回色が変わる（10/10成功）

**テストケース3: 高速クリック**
1. `taffy_flex_demo`起動
2. GreenBoxを高速で10回クリック（ダブルクリック閾値ギリギリ）
3. 期待: 毎回色が変わる or サイズが変わる（反応率100%）

### 8.2 ログ検証

**成功パターン**:
```
[handle_double_click_message] Double-click detected entity=7v0 target_entity=7v0
[ButtonBuffer] record_button_down entity=7v0 button=Left
[transfer_buffers_to_world] Button pressed entity=7v0 button=Left
[dispatch_pointer_events] Dispatching to entity=7v0 left_down=true double_click=Left
[Tunnel] GreenBox: DOUBLE-CLICK detected
```

**失敗パターン（修正前）**:
```
[handle_double_click_message] Double-click detected entity=3v0  ← Window
[ButtonBuffer] record_button_down entity=3v0 button=Left       ← Window
[dispatch_pointer_events] Dispatching to entity=3v0            ← Windowのみ
(GreenBoxのログなし)                                            ← ハンドラ未実行
```

## 9. ロールバック計画

### 9.1 修正のロールバック手順

修正によって予期せぬ問題が発生した場合：

1. `handlers.rs`の`handle_double_click_message()`を元に戻す
2. `mod.rs`の`DOUBLE_CLICK_THIS_FRAME`, `set_double_click()`, `transfer_buffers_to_world()`を復元
3. gitでrevertを実行

### 9.2 リスク評価

**低リスク**:
- 変更範囲が明確（2ファイル、3関数）
- 既存機能への影響なし（シングルクリック、ドラッグは変更なし）
- ダブルクリック機能は現在動作していないため、改悪のリスクなし

## 10. 実装タスク

実装は以下の3つのタスクに分割：

### Task 1: `handle_double_click_message()`の修正（handlers.rs）
- `handle_double_click_message()`にhit_testロジックを追加
- PointerState付与ロジックを実装
- `WM_*BUTTONDBLCLK`ハンドラのシグネチャ変更

### Task 2: グローバルダブルクリック情報の削除（mod.rs）
- `DOUBLE_CLICK_THIS_FRAME`変数を削除
- `set_double_click()`関数を削除
- `transfer_buffers_to_world()`のグローバル適用ロジックを削除

### Task 3: 動作確認とログ調整
- `taffy_flex_demo`でテスト実行
- 過剰なログ出力を調整（info → debug）
- ドキュメントコメントの更新

## 11. 今後の展望

### 11.1 将来の拡張

本修正により、以下の拡張が容易になる：

- **タッチパネル対応**: 複数PointerStateの同時処理が可能
- **ペン入力対応**: 筆圧、傾きなどの情報をPointerStateに追加
- **ジェスチャー認識**: ダブルタップ、長押し、スワイプなどの高レベルジェスチャー

### 11.2 最適化の余地

本修正では機能修正を優先し、最適化は行わない。将来的には：

- hit_test結果のキャッシュ（同一フレーム内での重複実行回避）
- PointerStateのプーリング（頻繁な挿入/削除の最適化）
- イベントバッチ処理（複数イベントの一括ディスパッチ）

## 12. まとめ

本設計は、ダブルクリック検出とシングルクリック抜けの問題を、**hit_testの一元化**と**エンティティ単位の記録**により解決する。グローバルな状態を削除し、コードの複雑性を下げながら、既存機能への影響を最小限に抑える。

**主要な変更点**:
1. ✅ `handle_double_click_message()`にhit_testロジックを追加
2. ✅ グローバル`DOUBLE_CLICK_THIS_FRAME`を削除
3. ✅ ダブルクリック情報を直接PointerStateに設定

**期待される効果**:
- ✅ ダブルクリック検出率: 0% → 100%
- ✅ シングルクリック検出率: 50% → 100%
- ✅ コードの複雑性: 低減（グローバル状態削除）
- ✅ 既存機能: 影響なし
