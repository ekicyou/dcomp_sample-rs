# Requirements Document

## Introduction

本ドキュメントは、`ecs_wndproc`関数のメッセージ処理をリファクタリングし、将来のメッセージ追加に備えた保守性の高い構造に変更するための要件を定義する。現在の実装では、すべてのメッセージ処理が単一の大きな`match`式に含まれているが、これを個別のハンドラ関数に分離することで、コードの可読性・保守性・拡張性を向上させる。

## Project Description (Input)

ecs_wndprocの全matchについて、すべて個別の関数をコールする形にリファクタリングしてほしい。将来的に処理するメッセージが多くなるため今のうちにリファクタリングが必要。呼び出す関数の名前は、あえてメッセージと同じ名前にして（WM_NCCREATEなら、関数名もWM_NCCREATE）。分離した関数はインライン展開するようにし、実質的な処理速度が変わらないようにせよ。各関数はOption<LRESULT>を返し、Noneが返った場合、DefWindowProcWを呼び出す実装とせよ。

## Requirements

### Requirement 1: メッセージハンドラ関数の分離

**Objective:** As a 開発者, I want 各Windowsメッセージの処理を独立した関数として分離できること, so that コードの保守性と可読性が向上し、新しいメッセージハンドラの追加が容易になる

#### Acceptance Criteria

1. When `ecs_wndproc`がWindowsメッセージを受信した場合, the window_procモジュール shall 対応するメッセージハンドラ関数を呼び出す
2. The window_procモジュール shall 以下のメッセージごとに個別のハンドラ関数を提供する:
   - `WM_NCCREATE` → `WM_NCCREATE`関数
   - `WM_NCDESTROY` → `WM_NCDESTROY`関数
   - `WM_ERASEBKGND` → `WM_ERASEBKGND`関数
   - `WM_PAINT` → `WM_PAINT`関数
   - `WM_CLOSE` → `WM_CLOSE`関数
   - `WM_WINDOWPOSCHANGED` → `WM_WINDOWPOSCHANGED`関数
   - `WM_DISPLAYCHANGE` → `WM_DISPLAYCHANGE`関数
   - `WM_DPICHANGED` → `WM_DPICHANGED`関数
3. The window_procモジュール shall 現時点で`DefWindowProcW`を呼び出すだけのメッセージ（`WM_NCHITTEST`等）はワイルドカードパターンに委譲する

### Requirement 2: 関数命名規則

**Objective:** As a 開発者, I want ハンドラ関数名がWindowsメッセージ定数と同じ名前であること, so that メッセージとハンドラの対応関係が一目で分かる

#### Acceptance Criteria

1. The window_procモジュール shall 各ハンドラ関数をWindowsメッセージ定数と完全に同じ名前で定義する（例: `WM_NCCREATE`, `WM_PAINT`）
2. The window_procモジュール shall Rustの命名規則（snake_case）よりもWindowsメッセージ定数との一貫性を優先する

### Requirement 3: 関数シグネチャの統一

**Objective:** As a 開発者, I want すべてのハンドラ関数が統一されたシグネチャを持つこと, so that 一貫性のあるAPIとなり、将来の拡張が容易になる

#### Acceptance Criteria

1. The window_procモジュール shall 各ハンドラ関数を`fn(hwnd: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT>`シグネチャで定義する
2. When ハンドラ関数が`Some(LRESULT)`を返した場合, the ecs_wndproc shall その値をそのまま返す
3. When ハンドラ関数が`None`を返した場合, the ecs_wndproc shall `DefWindowProcW(hwnd, message, wparam, lparam)`を呼び出してその結果を返す

### Requirement 4: インライン展開によるパフォーマンス維持

**Objective:** As a 開発者, I want リファクタリング後も実行時パフォーマンスが変わらないこと, so that 関数分離によるオーバーヘッドを回避できる

#### Acceptance Criteria

1. The window_procモジュール shall すべてのハンドラ関数に`#[inline]`属性を付与する
2. The window_procモジュール shall コンパイラによるインライン展開を許可し、関数呼び出しオーバーヘッドを最小化する

### Requirement 5: デフォルト処理の集約

**Objective:** As a 開発者, I want 未処理メッセージのデフォルト処理が一箇所で管理されること, so that `DefWindowProcW`呼び出しの一貫性が保証される

#### Acceptance Criteria

1. The ecs_wndproc shall `match`式の`_`（ワイルドカード）パターンで`None`を返す
2. When ハンドラ関数（ワイルドカード含む）が`None`を返した場合, the ecs_wndproc shall 後続処理で`DefWindowProcW`を呼び出す
3. The window_procモジュール shall `DefWindowProcW`の呼び出しを`ecs_wndproc`関数内の1箇所のみに集約する
4. The 各ハンドラ関数 shall `DefWindowProcW`を直接呼び出さず、`None`を返すことで委譲を表現する

### Requirement 6: 既存機能の維持

**Objective:** As a 開発者, I want リファクタリング後も既存のすべての機能が正常に動作すること, so that 回帰バグを防ぐことができる

#### Acceptance Criteria

1. The WM_NCCREATE関数 shall GWLP_USERDATAにEntity IDを保存し、`None`を返す
2. The WM_NCDESTROY関数 shall エンティティを削除し、GWLP_USERDATAをクリアし、`None`を返す
3. The WM_ERASEBKGND関数 shall `Some(LRESULT(1))`を返す（背景消去をスキップ）
4. The WM_PAINT関数 shall ValidateRectを呼び出し、`Some(LRESULT(0))`を返す
5. The WM_CLOSE関数 shall DestroyWindowを呼び出し、`Some(LRESULT(0))`を返す
6. The WM_WINDOWPOSCHANGED関数 shall DPI更新、WindowPos更新、BoxStyle更新、Vsync Tick、flush_window_pos_commandsを実行し、`None`を返す
7. The WM_DISPLAYCHANGE関数 shall Appリソースのmark_display_changeを呼び出し、`None`を返す
8. The WM_DPICHANGED関数 shall DpiChangeContextを設定し、SetWindowPosを呼び出し、`Some(LRESULT(0))`を返す

### Requirement 7: unsafe境界の適切な管理

**Objective:** As a 開発者, I want unsafe操作が最小限の範囲に限定されること, so that コードの安全性を維持できる

#### Acceptance Criteria

1. The 各ハンドラ関数 shall `unsafe fn`として定義する（Windows API呼び出しが必要なため）
2. The window_procモジュール shall unsafeブロックを必要最小限の範囲に留める

### Requirement 8: モジュール構造の分離

**Objective:** As a 開発者, I want ハンドラ関数が独立したサブモジュールに配置されること, so that 将来的に多数のメッセージハンドラが追加されてもファイルが巨大化せず、保守性を維持できる

#### Acceptance Criteria

1. The window_procモジュール shall ディレクトリベースのモジュール構造（`window_proc/mod.rs`）に変換する
2. The window_procモジュール shall メッセージハンドラ関数を`window_proc/handlers.rs`サブモジュールに配置する
3. The window_proc/mod.rs shall `ecs_wndproc`関数、`get_entity_from_hwnd`関数、`set_ecs_world`関数を保持する
4. The window_proc/handlers.rs shall すべてのメッセージハンドラ関数（`WM_NCCREATE`, `WM_PAINT`等）を保持する
5. The window_procモジュール shall ハンドラ関数に`pub(super)`可視性を使用する

### Requirement 9: 公開APIの最小化

**Objective:** As a 開発者, I want 外部公開APIを最小限に抑えること, so that 内部実装の変更が外部に影響を与えにくくなる

#### Acceptance Criteria

1. The window_procモジュール shall `set_ecs_world`関数を`pub(crate)`可視性で定義する（テストからの呼び出しなし、crate内部のみで使用）
2. The window_procモジュール shall `get_entity_from_hwnd`関数を`pub(crate)`可視性で定義し、`#[inline]`属性を付与する
3. The window_procモジュール shall `ecs_wndproc`関数を`pub(crate)`可視性で定義する（Windows APIコールバックとして使用、`extern "system"` ABIのためインライン不可）

