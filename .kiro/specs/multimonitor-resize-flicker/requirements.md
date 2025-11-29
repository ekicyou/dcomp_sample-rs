# Requirements Document

## Introduction

本仕様は、マルチモニター環境でウィンドウを異なるDPIのモニター間で移動する際に発生するちらつき（ウィンドウサイズの変動）問題を解決するための要件を定義する。

### 問題の背景

ログ調査により以下の問題が判明した：

#### 問題1: DPI変更時のサイズ縮小
- DPI=120（scale=1.25）からDPI=192（scale=2.00）への変更時
- 論理サイズが **800x600 → 496x363** に縮小
- 原因: `WM_DPICHANGED`で提供される推奨RECTを使用せず、既存の物理サイズを新DPIで割って論理サイズを計算している

#### 問題2: 移動中のサイズ漸増
- ドラッグ中に物理サイズが徐々に増加: 992→994→996→998→1000→1002→1004...
- 原因: 物理→論理→物理変換の丸め誤差が蓄積

#### 問題3: フィードバックループ
- `WM_WINDOWPOSCHANGED` → `BoxStyle更新` → `apply_window_pos_changes` → `SetWindowPos` → `WM_WINDOWPOSCHANGED`
- 丸め誤差によりエコーバック判定をすり抜け、無限ループに近い状態

### 目標

1. DPI変更時にWindowsが推奨するサイズを適切に処理する
2. 座標変換の丸め誤差による無限ループを防止する
3. ドラッグ中の不要なSetWindowPos呼び出しを抑制する

## Requirements

### Requirement 1: WM_DPICHANGED推奨サイズの適用

**Objective:** As a アプリケーションユーザー, I want DPI変更時にウィンドウサイズが適切にスケーリングして欲しい, so that 異なるDPIのモニター間を移動しても論理サイズが維持される。

#### Background

`WM_DPICHANGED`受信時点ではウィンドウサイズはまだ変更されていない。
`lparam`に含まれる`suggested_rect`はWindowsからの推奨サイズであり、`DefWindowProcW`を呼ぶことで内部的に`SetWindowPos`が実行され、推奨サイズが適用される。

#### Acceptance Criteria

1. When `WM_DPICHANGED`メッセージを受信したとき, the フレームワーク shall 新DPIと`suggested_rect`を`DpiChangeContext`に保存した後、`DefWindowProcW`を呼び出して推奨サイズを適用させる。
2. The フレームワーク shall DPI変更後も論理サイズ（DIP単位）を維持する（例: 800x600 DIPのウィンドウは新DPIでも800x600 DIPを維持）。
3. When `DefWindowProcW`内で`WM_WINDOWPOSCHANGED`が発火したとき, the フレームワーク shall `DpiChangeContext`から新DPIを取得してDPIコンポーネントを即時更新する。
4. The `WM_WINDOWPOSCHANGED`処理 shall 新DPIを使用して物理→論理座標変換を行い、正しい論理サイズを`BoxStyle`に反映する。

### ~~Requirement 2: 座標変換丸め誤差の防止~~ [REQ-009で解決]

**Status:** 廃止 - REQ-009「WM_WINDOWPOSCHANGED由来のSetWindowPos抑制」で解決

**理由:** `WindowPosChanged(bool)`フラグにより、フィードバックループは必ず1回で停止する。丸め誤差が蓄積する前に抑制されるため、エコーバック判定機構は不要。

**ガイドライン:** 丸め方向の統一（切り捨て推奨）は良い実践として推奨するが、要件としては不要。

### ~~Requirement 3: ドラッグ中のSetWindowPos抑制~~ [REQ-009に統合]

**Status:** 廃止 - REQ-009「物理座標ベースのエコーバック検知」に統合

**理由:** REQ-009 の`WindowPosChanged`コンポーネントによる物理座標比較により、ドラッグ中かどうかに関係なく`WM_WINDOWPOSCHANGED`由来の変更は自動的に抑制される。ドラッグ中フラグは不要。

### ~~Requirement 4: エコーバック判定の改善~~ [REQ-009に統合]

**Status:** 廃止 - REQ-009「WM_WINDOWPOSCHANGED由来のSetWindowPos抑制」に統合

**理由:** REQ-009 の`WindowPosChanged(bool)`フラグにより、`WM_WINDOWPOSCHANGED`由来の変更は完全に抑制される。許容誤差ベースのエコーバック判定は不要。

### ~~Requirement 5: WndProc内tickとSetWindowPosの競合防止~~ [REQ-011に統合]

**Status:** 廃止 - REQ-011「SetWindowPos遅延実行によるWorld借用競合の防止」に統合

**理由:** REQ-011 のキューベース遅延実行により、`apply_window_pos_changes` は常にキューに追加するだけとなり、フラグベースの抑制は不要になった。

### Requirement 6: 既存動作との互換性

**Objective:** As a フレームワーク開発者, I want 修正後も既存のウィンドウ操作（リサイズ、最大化、最小化等）が正常に動作して欲しい, so that 回帰が発生しない。

#### Acceptance Criteria

1. The ウィンドウリサイズ操作（端のドラッグ） shall 修正前と同様に動作する。
2. The ウィンドウ最大化・最小化操作 shall 修正前と同様に動作する。
3. The プログラムによるウィンドウサイズ変更（`BoxStyle`更新） shall 修正前と同様に動作する。
4. The 単一モニター環境 shall 修正の影響を受けない。

### Requirement 7: デバッグ・診断サポート

**Objective:** As a フレームワーク開発者, I want DPI変更と座標変換の状況を監視できるようにしたい, so that 問題発生時に原因を特定できる。

#### Acceptance Criteria

1. When デバッグビルドのとき, the フレームワーク shall DPI変更イベントをログ出力する（変更前後のDPI、推奨RECT、実際に適用されたサイズ）。
2. When デバッグビルドのとき, the フレームワーク shall `WindowPosChanged`フラグによる抑制発生をログ出力する。
3. When デバッグビルドのとき, the フレームワーク shall `SetWindowPosCommand`のキュー追加・実行をログ出力する。

### Requirement 8: World外DPI変更コンテキスト管理

**Objective:** As a フレームワーク開発者, I want DPI変更情報をWorld借用に依存せずに管理したい, so that WndProc再入時でも正しいDPI値を参照できる。

#### Background

`WM_DPICHANGED`は`DefWindowProcW`内から`SetWindowPos`を呼び、その中で`WM_WINDOWPOSCHANGED`が同期的に発火する。
スレッドローカルコンテキストにより、World借用状態に関係なく新DPIを`WM_WINDOWPOSCHANGED`に渡すことができる。

```
WM_DPICHANGED (同期)
  ├─ ① DpiChangeContext をスレッドローカルに保存（new_dpi, suggested_rect）
  └─ ② DefWindowProcW を呼ぶ → 内部で SetWindowPos(suggested_rect)
       ↓
       WM_WINDOWPOSCHANGED (再入、同期)
         ├─ ③ DpiChangeContext を取得・消費
         ├─ ④ new_dpi で DPIコンポーネントを更新
         ├─ ⑤ new_dpi で物理→論理座標変換
         └─ ⑥ BoxStyle 更新（正しい論理サイズ）
       ↓
       DefWindowProcW から戻る
```

#### Acceptance Criteria

1. The フレームワーク shall スレッドローカルな`DpiChangeContext`構造体を提供する。
2. The `DpiChangeContext` shall 以下の情報を保持する：
   - `new_dpi`: 新しいDPI値（既存の`DPI`型を使用）
   - `suggested_rect`: Windowsが推奨するウィンドウRECT（物理座標）
3. When `WM_DPICHANGED`を受信したとき, the フレームワーク shall `DefWindowProcW`を呼ぶ前に`DpiChangeContext`をスレッドローカルストレージに保存する。
4. When `WM_WINDOWPOSCHANGED`を処理するとき, the フレームワーク shall まずスレッドローカルから`DpiChangeContext`を取得・消費する。
5. If `DpiChangeContext`が存在するとき, the `WM_WINDOWPOSCHANGED`処理 shall DPIコンポーネントを即時更新し、`new_dpi`を使用して論理座標を計算する。
6. If `DpiChangeContext`が存在しないとき, the `WM_WINDOWPOSCHANGED`処理 shall 従来通り現在のDPIコンポーネントを使用する。
7. The `DpiChangeContext` shall `WM_WINDOWPOSCHANGED`での消費後にクリアされる。

### Requirement 10: WM_DPICHANGED_DEFERRED の廃止

**Objective:** As a フレームワーク開発者, I want 不要になった非同期DPI更新機構を削除したい, so that コードの複雑さを軽減し保守性を向上させる。

#### Background

Requirement 8 により、DPIコンポーネントの更新は`WM_WINDOWPOSCHANGED`内で同期的に行われるようになる。
従来の`PostMessage`による`WM_DPICHANGED_DEFERRED`は不要となる。

#### Acceptance Criteria

1. The フレームワーク shall `WM_DPICHANGED_DEFERRED`カスタムメッセージを廃止する。
2. The フレームワーク shall `post_dpi_change()`関数を削除する。
3. The フレームワーク shall `process_deferred_dpi_change()`関数を削除する。
4. The `WM_DPICHANGED`ハンドラ shall `PostMessage`を呼び出さない。
5. The DPIコンポーネント更新 shall `WM_WINDOWPOSCHANGED`処理内で完結する。

### Requirement 9: WM_WINDOWPOSCHANGED由来のSetWindowPos抑制

**Objective:** As a フレームワーク開発者, I want `WM_WINDOWPOSCHANGED`由来の`BoxStyle`変更に対して`SetWindowPos`を発行しないようにしたい, so that 不要なフィードバックループを防止する。

#### Background

`WM_WINDOWPOSCHANGED`由来の`BoxStyle`更新は、Windowsが既にウィンドウを正しい位置/サイズに設定済みであるため、`SetWindowPosCommand`を生成すべきではない。シンプルなフラグで変更の発生源を区別する。

```
WM_WINDOWPOSCHANGED
  └─ WindowPosChanged.0 = true
  └─ BoxStyle更新

apply_window_pos_changes (同tick)
  ├─ Changed<BoxStyle>検知
  ├─ if WindowPosChanged.0 == true → 抑制、WindowPosChanged.0 = false
  └─ else → SetWindowPosCommand生成
```

#### Acceptance Criteria

1. The フレームワーク shall `WM_WINDOWPOSCHANGED`の発生を記録する`WindowPosChanged(bool)`コンポーネントを提供する。
2. The `WindowPosChanged`コンポーネント shall `#[component(storage = "SparseSet")]`属性を持ち、Windowエンティティのみに効率的に割り当てられる。
3. When `WM_WINDOWPOSCHANGED`を処理するとき, the フレームワーク shall `WindowPosChanged.0 = true`に設定する。
4. The `apply_window_pos_changes`システム shall `WindowPosChanged.0 == true`の場合、`SetWindowPosCommand`を生成せずにフラグを`false`にリセットする。
5. When `WindowPosChanged.0 == false`のとき, the `apply_window_pos_changes` shall 通常通り`SetWindowPosCommand`を生成する。
6. The `apply_window_pos_changes` shall 処理後に必ず`WindowPosChanged.0 = false`にリセットする。

#### Implementation Notes

```rust
#[derive(Component, Default)]
#[component(storage = "SparseSet")]
pub struct WindowPosChanged(pub bool);
```

#### Known Limitations

- 同一tick内で`WM_WINDOWPOSCHANGED`→アプリによる`BoxStyle`変更が発生した場合、アプリの変更が1フレーム遅延する可能性がある
- これは稀なケースであり、次tickで正しく適用されるため許容する

### Requirement 11: SetWindowPos遅延実行によるWorld借用競合の防止

**Objective:** As a フレームワーク開発者, I want `apply_window_pos_changes`から直接`SetWindowPos`を呼び出さないようにしたい, so that World借用中の再入による二重借用エラーを防止する。

#### Background

現在の問題:
```
apply_window_pos_changes (World借用中)
  └─ SetWindowPos()
       └─ WM_WINDOWPOSCHANGED (同期)
            └─ try_tick_on_vsync() → World借用試行 → 二重借用エラー！
```

解決策: SetWindowPosをtick外に追い出す
```
apply_window_pos_changes (World借用中)
  └─ WINDOW_POS_COMMANDS キューに追加（SetWindowPos呼ばない）

try_tick_on_vsync() 終了直後（World借用解放後）
  └─ flush_window_pos_commands() → SetWindowPos実行
       └─ WM_WINDOWPOSCHANGED → World借用可能（安全）
```

#### Acceptance Criteria

1. The フレームワーク shall スレッドローカルな`WINDOW_POS_COMMANDS`キューを提供する。
2. The `SetWindowPosCommand`構造体 shall 以下の情報を保持する：
   - `hwnd`: 対象ウィンドウハンドル
   - `x`, `y`: 位置（物理座標）
   - `width`, `height`: サイズ（物理座標）
   - `flags`: SetWindowPosフラグ（SWP_*定数）
3. When `apply_window_pos_changes`でSetWindowPosが必要なとき, the システム shall `SetWindowPosCommand`をキューに追加し、即座にSetWindowPosを呼び出さない。
4. The `VsyncTick::try_tick_on_vsync()`実装 shall EcsWorld借用を解放した直後に`flush_window_pos_commands()`を呼び出す。
5. The `flush_window_pos_commands()`関数 shall キュー内のすべてのコマンドを順次実行し、SetWindowPosを呼び出す。
6. When `flush_window_pos_commands()`内でSetWindowPosを呼び出すとき, the WM_WINDOWPOSCHANGED shall 安全にWorldを借用できる（二重借用なし）。
7. The キュー shall 各flush後にクリアされる。

#### Implementation Notes

```rust
thread_local! {
    static WINDOW_POS_COMMANDS: RefCell<Vec<SetWindowPosCommand>> = RefCell::new(Vec::new());
}

pub struct SetWindowPosCommand {
    hwnd: HWND,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    flags: u32,
}

impl VsyncTick for Rc<RefCell<EcsWorld>> {
    fn try_tick_on_vsync(&self) -> bool {
        let result = match self.try_borrow_mut() {
            Ok(mut world) => world.try_tick_on_vsync(),
            Err(_) => return false,
        };
        // EcsWorld借用解放後にSetWindowPosを実行
        flush_window_pos_commands();
        result
    }
}
```
