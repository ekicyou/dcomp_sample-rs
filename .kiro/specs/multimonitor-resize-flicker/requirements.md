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

#### Acceptance Criteria

1. When `WM_DPICHANGED`メッセージを受信したとき, the フレームワーク shall `lparam`で提供される推奨RECTを使用してウィンドウサイズを設定する。
2. The フレームワーク shall DPI変更後も論理サイズ（DIP単位）を維持する（例: 800x600 DIPのウィンドウは新DPIでも800x600 DIPを維持）。
3. When 推奨RECTが適用されたとき, the フレームワーク shall その値を`WindowPos`コンポーネントに反映し、`BoxStyle`の論理サイズは変更しない。
4. The DPI変更処理 shall `WM_WINDOWPOSCHANGED`より前に完了し、後続のサイズ更新で論理サイズが縮小しない。

### Requirement 2: 座標変換丸め誤差の防止

**Objective:** As a フレームワーク開発者, I want 物理座標と論理座標の変換で丸め誤差が蓄積しないようにしたい, so that ウィンドウサイズが意図せず変動しない。

#### Acceptance Criteria

1. The `WM_WINDOWPOSCHANGED`処理 shall 物理座標を論理座標に変換する際、丸め方向を統一する（切り捨て、切り上げ、または四捨五入を一貫して使用）。
2. The `apply_window_pos_changes`システム shall 論理座標を物理座標に変換する際、同じ丸め方向を使用する。
3. When 物理→論理→物理の往復変換を行ったとき, the 結果 shall 元の物理座標と一致する（または許容誤差内に収まる）。
4. The フレームワーク shall 丸め誤差の許容範囲を定義し、その範囲内の差異はエコーバックとして扱う。

### Requirement 3: ドラッグ中のSetWindowPos抑制

**Objective:** As a フレームワーク開発者, I want ユーザーがウィンドウをドラッグしている間はSetWindowPosを呼び出さないようにしたい, so that フィードバックループを防止し、スムーズなドラッグ操作を実現する。

#### Acceptance Criteria

1. While ユーザーがウィンドウをドラッグしている間, the `apply_window_pos_changes`システム shall サイズ変更のみ許可し、位置変更のSetWindowPosを抑制する。
2. When `WM_ENTERSIZEMOVE`を受信したとき, the フレームワーク shall ドラッグ中フラグを設定する。
3. When `WM_EXITSIZEMOVE`を受信したとき, the フレームワーク shall ドラッグ中フラグをクリアする。
4. While ドラッグ中フラグが設定されているとき, the `WM_WINDOWPOSCHANGED`処理 shall `BoxStyle`の位置のみ更新し、サイズは変更しない（DPI変更時を除く）。

### Requirement 4: エコーバック判定の改善

**Objective:** As a フレームワーク開発者, I want エコーバック判定を丸め誤差を考慮したものにしたい, so that 不要なSetWindowPos呼び出しを防止する。

#### Acceptance Criteria

1. The `WindowPos::is_echo()`メソッド shall 完全一致ではなく、許容誤差（例: ±1ピクセル）を考慮した比較を行う。
2. When 座標の差が許容誤差内のとき, the `is_echo()` shall `true`を返す。
3. The 許容誤差 shall DPIスケールに応じて調整可能とする（高DPIでは大きめの許容誤差）。
4. When エコーバックと判定されたとき, the `apply_window_pos_changes` shall SetWindowPosを呼び出さない。

### Requirement 5: WndProc内tickとSetWindowPosの競合防止

**Objective:** As a フレームワーク開発者, I want WndProc内でworld tickが実行された場合にSetWindowPos呼び出しを抑制したい, so that 座標更新の競合を防止する。

#### Acceptance Criteria

1. While `WM_WINDOWPOSCHANGED`処理内でworld tickが実行されているとき, the `apply_window_pos_changes`システム shall SetWindowPosを呼び出さない。
2. The フレームワーク shall WndProc内tick実行中を示すフラグまたはコンテキストを提供する。
3. When WndProc内tick実行中フラグが設定されているとき, the `apply_window_pos_changes` shall 処理をスキップする。
4. The フラグ shall tick完了後に自動的にクリアされる。

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
2. When デバッグビルドのとき, the フレームワーク shall エコーバック判定の結果をログ出力する（判定理由、差分値）。
3. Where ドラッグ中抑制が発生したとき, the フレームワーク shall 抑制された操作をログ出力する。

