# ポインターイベントシステムのダブルクリック検出とシングルクリック抜けの修正 - タスク分解

## タスク概要

設計フェーズで定義された修正を3つのタスクに分割し、段階的に実装する。

## タスク一覧

### Task 1: `handle_double_click_message()`の修正
**ファイル**: `crates/wintf/src/ecs/window_proc/handlers.rs`  
**優先度**: 🔴 High  
**見積もり**: 30分  
**依存**: なし

#### 概要
`handle_double_click_message()`関数を`handle_button_message()`と同様のロジックに変更し、hit_testでターゲットエンティティを特定してPointerStateを付与する。

#### 実装内容

##### 1.1 `handle_double_click_message()`のシグネチャ変更

**before** (行1099):
```rust
unsafe fn handle_double_click_message(
    hwnd: HWND,
    double_click: crate::ecs::pointer::DoubleClick,
) -> HandlerResult {
```

**after**:
```rust
unsafe fn handle_double_click_message(
    hwnd: HWND,
    wparam: WPARAM,
    lparam: LPARAM,
    double_click: crate::ecs::pointer::DoubleClick,
) -> HandlerResult {
```

##### 1.2 `handle_double_click_message()`の実装変更

**変更場所**: 行1099-1131

**before**:
```rust
unsafe fn handle_double_click_message(
    hwnd: HWND,
    double_click: crate::ecs::pointer::DoubleClick,
) -> HandlerResult {
    let Some(entity) = super::get_entity_from_hwnd(hwnd) else {
        return None;
    };

    tracing::info!(
        entity = ?entity,
        double_click = ?double_click,
        "[handle_double_click_message] Double-click detected"
    );

    // ダブルクリック情報を設定
    crate::ecs::pointer::set_double_click(entity, double_click);
    
    // ダブルクリックも通常のボタン押下として記録
    // （WM_LBUTTONDBLCLKはWM_LBUTTONDOWNの代わりに来る）
    let button = match double_click {
        crate::ecs::pointer::DoubleClick::Left => crate::ecs::pointer::PointerButton::Left,
        crate::ecs::pointer::DoubleClick::Right => crate::ecs::pointer::PointerButton::Right,
        crate::ecs::pointer::DoubleClick::Middle => crate::ecs::pointer::PointerButton::Middle,
        crate::ecs::pointer::DoubleClick::XButton1 => crate::ecs::pointer::PointerButton::XButton1,
        crate::ecs::pointer::DoubleClick::XButton2 => crate::ecs::pointer::PointerButton::XButton2,
        crate::ecs::pointer::DoubleClick::None => return Some(LRESULT(0)),
    };
    crate::ecs::pointer::record_button_down(entity, button);
    
    Some(LRESULT(0))
}
```

**after**:
```rust
unsafe fn handle_double_click_message(
    hwnd: HWND,
    wparam: WPARAM,
    lparam: LPARAM,
    double_click: crate::ecs::pointer::DoubleClick,
) -> HandlerResult {
    use crate::ecs::layout::hit_test::{hit_test_in_window, PhysicalPoint as HitTestPoint};
    use crate::ecs::pointer::{PointerState, PhysicalPoint};

    let Some(window_entity) = super::get_entity_from_hwnd(hwnd) else {
        return None;
    };

    // クリック位置を取得
    let x = (lparam.0 & 0xFFFF) as i16 as i32;
    let y = ((lparam.0 >> 16) & 0xFFFF) as i16 as i32;
    
    // 修飾キー状態を抽出
    let wparam_val = wparam.0 as u32;
    let shift = (wparam_val & 0x04) != 0;
    let ctrl = (wparam_val & 0x08) != 0;

    // ダブルクリックに対応するボタンを取得
    let button = match double_click {
        crate::ecs::pointer::DoubleClick::Left => crate::ecs::pointer::PointerButton::Left,
        crate::ecs::pointer::DoubleClick::Right => crate::ecs::pointer::PointerButton::Right,
        crate::ecs::pointer::DoubleClick::Middle => crate::ecs::pointer::PointerButton::Middle,
        crate::ecs::pointer::DoubleClick::XButton1 => crate::ecs::pointer::PointerButton::XButton1,
        crate::ecs::pointer::DoubleClick::XButton2 => crate::ecs::pointer::PointerButton::XButton2,
        crate::ecs::pointer::DoubleClick::None => return Some(LRESULT(0)),
    };

    // hit_test でターゲットエンティティを特定し、PointerState を確保
    if let Some(world) = super::try_get_ecs_world() {
        if let Ok(mut world_borrow) = world.try_borrow_mut() {
            
            if let Some(target_entity) = hit_test_in_window(
                world_borrow.world(),
                window_entity,
                HitTestPoint::new(x as f32, y as f32),
            ) {
                tracing::info!(
                    window_entity = ?window_entity,
                    target_entity = ?target_entity,
                    double_click = ?double_click,
                    x, y,
                    "[handle_double_click_message] Double-click detected"
                );

                // PointerState がない場合は付与
                if world_borrow.world().get::<PointerState>(target_entity).is_none() {
                    world_borrow.world_mut().entity_mut(target_entity).insert(PointerState {
                        screen_point: PhysicalPoint::new(x, y),
                        local_point: PhysicalPoint::new(x, y),
                        left_down: button == crate::ecs::pointer::PointerButton::Left,
                        right_down: button == crate::ecs::pointer::PointerButton::Right,
                        middle_down: button == crate::ecs::pointer::PointerButton::Middle,
                        xbutton1_down: button == crate::ecs::pointer::PointerButton::XButton1,
                        xbutton2_down: button == crate::ecs::pointer::PointerButton::XButton2,
                        shift_down: shift,
                        ctrl_down: ctrl,
                        double_click,
                        ..Default::default()
                    });
                    debug!(
                        entity = ?target_entity,
                        button = ?button,
                        double_click = ?double_click,
                        "PointerState inserted on double-click event"
                    );
                } else {
                    // 既存の PointerState に double_click を設定
                    if let Some(mut ps) = world_borrow.world_mut().get_mut::<PointerState>(target_entity) {
                        ps.double_click = double_click;
                        ps.shift_down = shift;
                        ps.ctrl_down = ctrl;
                    }
                }

                // 修飾キー状態を記録
                crate::ecs::pointer::set_modifier_state(target_entity, shift, ctrl);

                // ボタン状態をバッファに記録
                crate::ecs::pointer::record_button_down(target_entity, button);
            }
        }
    }
    
    Some(LRESULT(0))
}
```

##### 1.3 `WM_LBUTTONDBLCLK`ハンドラの修正

**変更場所**: 行1133-1141

**before**:
```rust
pub(super) unsafe fn WM_LBUTTONDBLCLK(
    hwnd: HWND,
    _message: u32,
    _wparam: WPARAM,
    _lparam: LPARAM,
) -> HandlerResult {
    handle_double_click_message(hwnd, crate::ecs::pointer::DoubleClick::Left)
}
```

**after**:
```rust
pub(super) unsafe fn WM_LBUTTONDBLCLK(
    hwnd: HWND,
    _message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> HandlerResult {
    handle_double_click_message(hwnd, wparam, lparam, crate::ecs::pointer::DoubleClick::Left)
}
```

##### 1.4 `WM_RBUTTONDBLCLK`ハンドラの修正

**変更場所**: 行1143-1151

**before**:
```rust
pub(super) unsafe fn WM_RBUTTONDBLCLK(
    hwnd: HWND,
    _message: u32,
    _wparam: WPARAM,
    _lparam: LPARAM,
) -> HandlerResult {
    handle_double_click_message(hwnd, crate::ecs::pointer::DoubleClick::Right)
}
```

**after**:
```rust
pub(super) unsafe fn WM_RBUTTONDBLCLK(
    hwnd: HWND,
    _message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> HandlerResult {
    handle_double_click_message(hwnd, wparam, lparam, crate::ecs::pointer::DoubleClick::Right)
}
```

##### 1.5 `WM_MBUTTONDBLCLK`ハンドラの修正

**変更場所**: 行1153-1161

**before**:
```rust
pub(super) unsafe fn WM_MBUTTONDBLCLK(
    hwnd: HWND,
    _message: u32,
    _wparam: WPARAM,
    _lparam: LPARAM,
) -> HandlerResult {
    handle_double_click_message(hwnd, crate::ecs::pointer::DoubleClick::Middle)
}
```

**after**:
```rust
pub(super) unsafe fn WM_MBUTTONDBLCLK(
    hwnd: HWND,
    _message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> HandlerResult {
    handle_double_click_message(hwnd, wparam, lparam, crate::ecs::pointer::DoubleClick::Middle)
}
```

##### 1.6 `WM_XBUTTONDBLCLK`ハンドラの修正

**変更場所**: 行1163-1179

**before**:
```rust
pub(super) unsafe fn WM_XBUTTONDBLCLK(
    hwnd: HWND,
    _message: u32,
    wparam: WPARAM,
    _lparam: LPARAM,
) -> HandlerResult {
    let xbutton = ((wparam.0 >> 16) & 0xFFFF) as u16;
    let double_click = if xbutton == 1 {
        crate::ecs::pointer::DoubleClick::XButton1
    } else {
        crate::ecs::pointer::DoubleClick::XButton2
    };
    handle_double_click_message(hwnd, double_click)
}
```

**after**:
```rust
pub(super) unsafe fn WM_XBUTTONDBLCLK(
    hwnd: HWND,
    _message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> HandlerResult {
    let xbutton = ((wparam.0 >> 16) & 0xFFFF) as u16;
    let double_click = if xbutton == 1 {
        crate::ecs::pointer::DoubleClick::XButton1
    } else {
        crate::ecs::pointer::DoubleClick::XButton2
    };
    handle_double_click_message(hwnd, wparam, lparam, double_click)
}
```

#### 検証ポイント
- [ ] コンパイルエラーなし
- [ ] `handle_double_click_message()`がhit_testを実行している
- [ ] ターゲットエンティティにPointerStateが付与される
- [ ] `double_click`フィールドが正しく設定される

---

### Task 2: グローバルダブルクリック情報の削除
**ファイル**: `crates/wintf/src/ecs/pointer/mod.rs`  
**優先度**: 🔴 High  
**見積もり**: 20分  
**依存**: Task 1完了後

#### 概要
グローバルなダブルクリック状態管理を削除し、コードの複雑性を低減する。

#### 実装内容

##### 2.1 `DOUBLE_CLICK_THIS_FRAME`変数の削除

**変更場所**: 行327-331（thread_local!ブロック内）

**before**:
```rust
thread_local! {
    // ... 他の変数
    
    /// グローバルなダブルクリック情報（エンティティに紐付けない）
    /// このフレームでダブルクリックが発生したかを記録し、全PointerStateに適用する
    pub(crate) static DOUBLE_CLICK_THIS_FRAME: RefCell<DoubleClick> = RefCell::new(DoubleClick::None);
}
```

**after**:
```rust
thread_local! {
    // ... 他の変数
    
    // DOUBLE_CLICK_THIS_FRAME を削除（不要になった）
}
```

**注意**: `DOUBLE_CLICK_BUFFERS`は削除しない（将来のマルチタッチ対応用に残す）。

##### 2.2 `set_double_click()`関数の削除

**変更場所**: 行769-778

**before**:
```rust
/// DoubleClickを設定（グローバル）
/// エンティティには紐付けず、このフレームでダブルクリックが発生したことを記録
#[inline]
pub(crate) fn set_double_click(_entity: Entity, double_click: DoubleClick) {
    DOUBLE_CLICK_THIS_FRAME.with(|dc| {
        let mut dc = dc.borrow_mut();
        // 既にダブルクリックが記録されていない場合のみ設定（最初のみ）
        if *dc == DoubleClick::None {
            *dc = double_click;
        }
    });
}
```

**after**:
```rust
// set_double_click()関数を削除（不要になった）
// ダブルクリック情報は handle_double_click_message() 内で直接 PointerState に設定される
```

##### 2.3 `transfer_buffers_to_world()`のグローバルダブルクリック適用ロジック削除

**変更場所**: 行1054-1071（`transfer_buffers_to_world()`関数内）

**before**:
```rust
    // グローバルなダブルクリック情報を、PointerStateを持つ全エンティティに適用
    let double_click_this_frame = DOUBLE_CLICK_THIS_FRAME.with(|dc| *dc.borrow());
    
    if double_click_this_frame != DoubleClick::None {
        for (entity, mut pointer_state) in world.query::<(Entity, &mut PointerState)>().iter_mut(world) {
            pointer_state.double_click = double_click_this_frame;
            
            tracing::info!(
                entity = ?entity,
                double_click = ?double_click_this_frame,
                "[DOUBLE-CLICK] Applied to PointerState"
            );
        }
    }
    
    // DOUBLE_CLICK_THIS_FRAMEをリセット
    DOUBLE_CLICK_THIS_FRAME.with(|dc| {
        *dc.borrow_mut() = DoubleClick::None;
    });
```

**after**:
```rust
    // グローバルダブルクリック適用ロジックを削除
    // ダブルクリック情報は handle_double_click_message() で直接設定されるため不要
```

#### 検証ポイント
- [ ] コンパイルエラーなし
- [ ] `DOUBLE_CLICK_THIS_FRAME`への参照がすべて削除されている
- [ ] `set_double_click()`の呼び出しがない（handlers.rsから削除済み）
- [ ] `transfer_buffers_to_world()`が正常に動作する

---

### Task 3: 動作確認とログ調整
**ファイル**: 複数  
**優先度**: 🟡 Medium  
**見積もり**: 30分  
**依存**: Task 1, Task 2完了後

#### 概要
修正後の動作をテストし、過剰なログ出力を調整する。

#### 実装内容

##### 3.1 ビルドとテスト実行

**コマンド**:
```powershell
# ビルド
cargo build --example taffy_flex_demo

# 実行
cargo run --example taffy_flex_demo
```

##### 3.2 手動テスト実行

**テストケース1: ダブルクリック検出**
1. GreenBoxを5回ダブルクリック
2. 期待: 毎回サイズが変わる（5/5成功）
3. ログ確認: `[Tunnel] GreenBox: DOUBLE-CLICK detected` が5回出力

**テストケース2: シングルクリック安定性**
1. GreenBoxを10回シングルクリック（ゆっくり）
2. 期待: 毎回色が変わる（10/10成功）

**テストケース3: 高速クリック**
1. GreenBoxを高速で10回クリック
2. 期待: 毎回色が変わる or サイズが変わる（反応率100%）

##### 3.3 ログレベル調整

必要に応じて以下のログレベルを調整：

**`handlers.rs` (handle_double_click_message内)**:
```rust
// 過剰な場合は info → debug に変更
tracing::debug!(  // info から変更
    window_entity = ?window_entity,
    target_entity = ?target_entity,
    double_click = ?double_click,
    x, y,
    "[handle_double_click_message] Double-click detected"
);
```

**`mod.rs` (record_button_down内)**:
```rust
// 過剰な場合は info → debug に変更
tracing::debug!(  // info から変更
    entity = ?entity,
    button = ?button,
    "[ButtonBuffer] record_button_down"
);
```

##### 3.4 ドキュメントコメント更新

**`handlers.rs` (handle_double_click_message)**:
```rust
/// WM_*BUTTONDBLCLK ハンドラ共通処理
///
/// ダブルクリックイベントを処理し、hit_testでターゲットエンティティを特定して
/// PointerStateを付与する。WM_LBUTTONDOWNの代わりにWM_LBUTTONDBLCLKが来るため、
/// ボタン押下記録も同時に行う。
///
/// # Arguments
/// - `hwnd`: ウィンドウハンドル
/// - `wparam`: 修飾キー状態とXBUTTON情報
/// - `lparam`: クリック座標（クライアント座標）
/// - `double_click`: ダブルクリック種別
unsafe fn handle_double_click_message(
    hwnd: HWND,
    wparam: WPARAM,
    lparam: LPARAM,
    double_click: crate::ecs::pointer::DoubleClick,
) -> HandlerResult {
```

#### 検証ポイント
- [ ] すべてのテストケースが成功する
- [ ] ログ出力が適切なレベルである
- [ ] ドキュメントコメントが正確である
- [ ] 既存機能（ドラッグ移動等）が正常に動作する

---

## タスク実行順序

```
Task 1: handle_double_click_message()の修正
  ├─ 1.1 シグネチャ変更
  ├─ 1.2 実装変更（hit_testロジック追加）
  ├─ 1.3 WM_LBUTTONDBLCLK修正
  ├─ 1.4 WM_RBUTTONDBLCLK修正
  ├─ 1.5 WM_MBUTTONDBLCLK修正
  └─ 1.6 WM_XBUTTONDBLCLK修正
  ↓
Task 2: グローバルダブルクリック情報の削除
  ├─ 2.1 DOUBLE_CLICK_THIS_FRAME削除
  ├─ 2.2 set_double_click()削除
  └─ 2.3 transfer_buffers_to_world()修正
  ↓
Task 3: 動作確認とログ調整
  ├─ 3.1 ビルドとテスト実行
  ├─ 3.2 手動テスト実行
  ├─ 3.3 ログレベル調整
  └─ 3.4 ドキュメントコメント更新
```

## 完了基準

### Task 1完了基準
- [x] `handle_double_click_message()`がコンパイル可能
- [x] 5つのWM_*BUTTONDBLCLKハンドラがコンパイル可能
- [x] hit_testロジックが実装されている

### Task 2完了基準
- [x] `DOUBLE_CLICK_THIS_FRAME`が削除されている
- [x] `set_double_click()`が削除されている
- [x] `transfer_buffers_to_world()`からグローバル適用ロジックが削除されている
- [x] コンパイルエラーなし

### Task 3完了基準
- [x] GreenBoxダブルクリックで100%サイズ変更する
- [x] GreenBoxシングルクリックで100%色変更する
- [x] 高速クリックで100%反応する
- [x] ログ出力が適切である
- [x] 既存機能が正常動作する

### 全体完了基準
- [x] すべてのタスクが完了している
- [x] すべての検証ポイントがチェックされている
- [x] requirements.mdのすべての要件を満たしている
- [x] design.mdの期待効果が達成されている

## トラブルシューティング

### 問題1: コンパイルエラー「cannot find function `set_double_click`」

**原因**: Task 1でまだ`set_double_click()`の呼び出しが残っている  
**解決**: `handle_double_click_message()`から`set_double_click()`の呼び出しを削除

### 問題2: ダブルクリックがまだ検出されない

**原因**: hit_test結果が`None`を返している  
**解決**: 
- `GlobalArrangement`コンポーネントが正しく設定されているか確認
- `hit_test_in_window()`のログを有効化して調査

### 問題3: シングルクリックが動作しなくなった

**原因**: `handle_button_message()`を誤って変更した  
**解決**: `handle_button_message()`は変更しないこと（Task 1は`handle_double_click_message()`のみ）

## 見積もりサマリー

| タスク | 見積もり | 優先度 |
|--------|----------|--------|
| Task 1 | 30分 | 🔴 High |
| Task 2 | 20分 | 🔴 High |
| Task 3 | 30分 | 🟡 Medium |
| **合計** | **80分** | - |

## リスクとコンティンジェンシー

### リスク1: hit_testロジックの実装ミス
**確率**: 低  
**影響**: 中  
**対策**: `handle_button_message()`のロジックをコピー＆ペーストして最小限の変更にする

### リスク2: グローバル状態削除後のバグ
**確率**: 低  
**影響**: 低  
**対策**: Task 2後に即座にビルドして検証する

### リスク3: 既存機能のリグレッション
**確率**: 極低  
**影響**: 中  
**対策**: Task 3でドラッグ移動、ホイール、修飾キーの動作確認を行う
