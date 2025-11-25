# Requirements Document

## Project Description (Input)
apply_window_pos_changesがSetWindowPosを呼び出すとき、引数の
WindowPos::position,sizeについて、「そのウィンドウのクライアント領域」がその位置になるように、SetWindowPosのパラメーターを調整する。

## Introduction

現在の`apply_window_pos_changes`システムは、`WindowPos`コンポーネントの`position`および`size`をそのまま`SetWindowPos` Win32 APIに渡している。しかし、`SetWindowPos`はウィンドウ全体（タイトルバー、枠などを含む）の座標とサイズを指定するため、クライアント領域の位置・サイズを指定したい場合には、ウィンドウスタイルに応じた調整が必要となる。

本仕様では、`WindowPos::position`と`WindowPos::size`を「クライアント領域の座標・サイズ」として扱い、`SetWindowPos`呼び出し前にウィンドウ枠やタイトルバーのサイズ分を考慮した調整を行う機能を実装する。これにより、アプリケーション開発者がクライアント領域ベースでウィンドウ配置を指定できるようになる。

## Requirements

### Requirement 1: クライアント領域座標の調整機能
**Objective:** 開発者として、`WindowPos`のpositionとsizeをクライアント領域の座標・サイズとして扱いたい。そうすることで、ウィンドウ枠やタイトルバーのサイズを意識せずに配置を指定できる。

#### Acceptance Criteria
1. When `apply_window_pos_changes`システムが`WindowPos`コンポーネントの変更を検知した場合、wintfシステムは`SetWindowPos`呼び出し前にクライアント領域からウィンドウ全体への座標変換を実施すること
2. When `WindowPos::position`が設定されている場合、wintfシステムはそのpositionをクライアント領域の左上座標として扱い、ウィンドウスタイルに基づいて調整された座標で`SetWindowPos`を呼び出すこと
3. When `WindowPos::size`が設定されている場合、wintfシステムはそのsizeをクライアント領域のサイズとして扱い、ウィンドウ枠・タイトルバーのサイズを加算した値で`SetWindowPos`を呼び出すこと
4. If `WindowPos::position`または`WindowPos::size`が`None`である場合、wintfシステムは該当するパラメーターを0として扱い、対応するSWPフラグ（`SWP_NOMOVE`/`SWP_NOSIZE`）を使用すること

### Requirement 2: ウィンドウスタイル情報の取得
**Objective:** 開発者として、座標変換に必要なウィンドウスタイル情報を自動的に取得したい。そうすることで、手動でスタイル情報を管理する必要がなくなる。

#### Acceptance Criteria
1. When 座標変換が必要になった場合、wintfシステムは対象ウィンドウのHWNDから現在のウィンドウスタイル（`WINDOW_STYLE`）とExtended Style（`WINDOW_EX_STYLE`）を取得すること
2. When DPI対応が有効な場合、wintfシステムは現在のDPI値を考慮してウィンドウ枠サイズを計算すること
3. The wintfシステムは`AdjustWindowRectExForDpi` Win32 APIを使用してクライアント領域からウィンドウ全体の矩形を計算すること

### Requirement 3: エラーハンドリングと既存動作の保持
**Objective:** 開発者として、座標変換処理が失敗した場合でも安全にフォールバックしたい。そうすることで、システムの堅牢性が向上する。

#### Acceptance Criteria
1. If ウィンドウスタイル情報の取得に失敗した場合、wintfシステムは調整なしで元の座標・サイズを使用して`SetWindowPos`を呼び出すこと
2. If `AdjustWindowRectExForDpi`呼び出しが失敗した場合、wintfシステムは調整なしで元の座標・サイズを使用して`SetWindowPos`を呼び出すこと
3. The wintfシステムは座標変換の失敗をログ出力（`eprintln!`等）で記録し、デバッグを容易にすること
4. The wintfシステムは既存の`is_echo`メカニズムと`last_sent_position`/`last_sent_size`の動作を維持すること

### Requirement 4: CW_USEDEFAULTと特殊値の扱い
**Objective:** 開発者として、ウィンドウ作成時の初期値やシステムデフォルト値が適切に処理されることを期待する。そうすることで、初期化フローが正しく機能する。

#### Acceptance Criteria
1. If `WindowPos::position`または`WindowPos::size`が`CW_USEDEFAULT`を含む場合、wintfシステムは座標変換をスキップし、そのまま`SetWindowPos`に渡すこと
2. The wintfシステムは既存のエコーバックチェック（`is_echo`）を座標変換後も正しく機能させること

### Requirement 5: テストアプリケーション動作確認
**Objective:** 開発者として、`taffy_flex_demo`サンプルアプリケーションで本機能の動作を確認したい。そうすることで、実際のユースケースでクライアント領域座標調整が正しく機能することを検証できる。

#### Acceptance Criteria
1. When `taffy_flex_demo`アプリケーションを起動した場合、wintfシステムはウィンドウのクライアント領域が指定座標に配置されるように調整すること
2. The `taffy_flex_demo`は初期ウィンドウ位置を`POINT { x: 100, y: 100 }`に設定し、タイトルバーが画面外に出ないことを確認できること
3. The `taffy_flex_demo`は初期ウィンドウサイズを`SIZE { cx: 800, cy: 600 }`（クライアント領域）に設定し、意図したレイアウトが表示されること
