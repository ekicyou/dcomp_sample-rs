# Research & Design Decisions: client-area-positioning

## Summary
- **Feature**: `client-area-positioning`
- **Discovery Scope**: Extension（既存システムへの統合）
- **Key Findings**:
  - DPI取得は`GetDpiForWindow` Win32 APIで実装可能（`DpiTransform`コンポーネントは未使用）
  - 既存の`effective_window_size`パターンを流用して座標変換を実装できる
  - エコーバックメカニズムは変換後の値を記録することで維持可能

## Research Log

### DPI値取得方法の調査
- **Context**: `AdjustWindowRectExForDpi` APIにDPI値を渡す必要があるが、`apply_window_pos_changes`システム内でDPI値へのアクセス方法が不明確
- **Sources Consulted**:
  - 既存コードベース（`win_state.rs`, `ecs/window.rs`）
  - Windows API仕様: `GetDpiForWindow`（user32.dll）
- **Findings**:
  - `DpiTransform`コンポーネントが定義されているが、ウィンドウEntityへの自動付与は未実装
  - `WinState::dpi()` traitは非ECSパターンで外部インターフェイス
  - `GetDpiForWindow(hwnd)` APIはHWNDから直接DPI値を取得可能
- **Implications**: `GetDpiForWindow`を直接呼び出すことで、`DpiTransform`コンポーネントへの依存なしに実装可能

### 既存の座標変換実装パターン
- **Context**: クライアント領域からウィンドウ全体への変換ロジックの実装方法
- **Sources Consulted**:
  - `win_state.rs`: `effective_window_size`メソッド（39-65行目）
  - Windows API: `AdjustWindowRectExForDpi`
- **Findings**:
  - `effective_window_size`はサイズ変換のみを行い、位置変換は含まれていない
  - `AdjustWindowRectExForDpi`は`RECT`構造体を変換し、`left`/`top`に負のオフセット値が設定される
  - 位置変換には`-rect.left`、`-rect.top`を元の位置に加算する必要がある
- **Implications**: 既存パターンを拡張して位置オフセット計算を追加することで実装可能

### エコーバックメカニズムの影響分析
- **Context**: 座標変換が既存のエコーバック検知に与える影響
- **Sources Consulted**:
  - `ecs/window.rs`: `WindowPos::is_echo`メソッド（432行目）
  - `ecs/graphics/systems.rs`: `apply_window_pos_changes`（520-560行目）
  - `ecs/window_proc.rs`: `WM_WINDOWPOSCHANGED`ハンドラー（78-113行目）
- **Findings**:
  - `WM_WINDOWPOSCHANGED`で受信する座標・サイズは、`SetWindowPos`に渡した**変換後の値**
  - `last_sent_position`/`last_sent_size`は変換後の値を記録する必要がある
  - 変換前の値を記録すると、エコーバック判定が常に失敗する
- **Implications**: 座標変換後の値を`last_sent_*`に記録することで、既存のエコーバックメカニズムを維持可能

### CW_USEDEFAULT特殊値の処理
- **Context**: ウィンドウ作成時の初期値における座標変換の扱い
- **Sources Consulted**:
  - `ecs/graphics/systems.rs`: `apply_window_pos_changes`（542-545行目）
  - Windows API仕様: `CW_USEDEFAULT`定数
- **Findings**:
  - 既存コードで`CW_USEDEFAULT`チェックが実装済み
  - `CW_USEDEFAULT`を含む座標・サイズは座標変換をスキップする必要がある
- **Implications**: 既存のチェックロジックを維持し、変換処理をその後に挿入すれば対応可能

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| Option A: Inline Extension | `apply_window_pos_changes`関数内に変換ロジックをインライン実装 | 最小限の変更、変更箇所が局所的、既存パターン踏襲 | 関数肥大化（38行→60-70行）、再利用性なし | ギャップ分析で評価済み |
| Option B: New Module | 専用モジュール`window_coords.rs`を作成 | 関心の分離、再利用可能、テスト容易 | ファイル数増加、インターフェイス設計コスト | ギャップ分析で評価済み |
| Option C: Hybrid | `WindowPos`にメソッド追加、`apply_window_pos_changes`で呼び出し | バランス良好、既存パターン整合、適度な結合度 | `window.rs`やや肥大化（435行→470行） | **推奨アプローチ** |

## Design Decisions

### Decision: DPI取得方法
- **Context**: `AdjustWindowRectExForDpi` APIにDPI値を渡す必要があるが、現状`DpiTransform`コンポーネントは自動付与されていない
- **Alternatives Considered**:
  1. `GetDpiForWindow(hwnd)` Win32 APIを直接呼び出し
  2. `DpiTransform`コンポーネントをウィンドウ作成時に自動付与し、クエリに追加
  3. `WinState` traitを実装したリソースからDPI値を取得
- **Selected Approach**: `GetDpiForWindow(hwnd)`を直接呼び出し
- **Rationale**:
  - 最小限の変更で実装可能
  - `DpiTransform`コンポーネントの自動付与は別仕様として実装すべき
  - 既存コードベースへの影響が最も少ない
- **Trade-offs**:
  - ✅ シンプル、既存システムへの影響なし
  - ⚠️ 将来的に`DpiTransform`が完成したら切り替えが必要
- **Follow-up**: `GetDpiForWindow`がウィンドウ作成直後に正しく機能するか単体テストで検証

### Decision: 実装アプローチ (Option C採用)
- **Context**: 座標変換ロジックをどこに配置するか
- **Alternatives Considered**:
  1. `apply_window_pos_changes`関数内にインライン実装
  2. 専用モジュール`window_coords.rs`を作成
  3. `WindowPos`コンポーネントにメソッド追加
- **Selected Approach**: Option C - `WindowPos::to_window_coords`メソッドを追加
- **Rationale**:
  - `WindowPos::set_window_pos`と同様のパターン（HWNDを引数に取るメソッド）
  - 座標変換は`WindowPos`の責務として論理的に妥当
  - ファイル数増加なし、適度な結合度
- **Trade-offs**:
  - ✅ 保守性とシンプルさのバランス
  - ✅ 既存パターンとの整合性
  - ⚠️ `window.rs`がやや肥大化（+35行程度）
- **Follow-up**: メソッド単体でテスト可能な設計を確保

### Decision: エコーバック値の記録タイミング
- **Context**: 座標変換後の`last_sent_position`/`last_sent_size`記録方法
- **Alternatives Considered**:
  1. 変換前の値を記録（クライアント領域座標）
  2. 変換後の値を記録（ウィンドウ全体座標）
- **Selected Approach**: 変換後の値を記録
- **Rationale**:
  - `WM_WINDOWPOSCHANGED`で受信する値は変換後の値
  - 変換前の値を記録するとエコーバック判定が常に失敗する
- **Trade-offs**:
  - ✅ 既存のエコーバックメカニズムを破壊しない
  - ✅ 追加のロジック不要
- **Follow-up**: エコーバック判定が正しく機能することを統合テストで確認

### Decision: エラーハンドリング戦略
- **Context**: 座標変換失敗時の動作
- **Alternatives Considered**:
  1. エラーを返してSetWindowPos呼び出しをスキップ
  2. フォールバック（元の座標・サイズでSetWindowPos呼び出し）
- **Selected Approach**: フォールバック + エラーログ出力
- **Rationale**:
  - Requirement 3の要件「調整なしで元の座標・サイズを使用」を満たす
  - 部分的な機能degradationで完全な失敗を避ける
- **Trade-offs**:
  - ✅ 堅牢性向上、ユーザー体験の維持
  - ⚠️ 座標変換失敗が静かに無視される可能性（ログで検知可能）
- **Follow-up**: エラーログフォーマットとデバッグ情報の充実度を確認

## Risks & Mitigations

- **Risk 1: GetDpiForWindow APIがウィンドウ作成直後に不正な値を返す可能性**
  - Mitigation: 単体テストでウィンドウ作成直後のDPI値取得を検証、fallback to `GetDpiForSystem()`を検討
- **Risk 2: タイトルバーなしウィンドウ（WS_POPUP等）での座標変換の挙動**
  - Mitigation: `taffy_flex_demo`に加えて、WS_POPUPスタイルのテストケースを追加
- **Risk 3: マルチモニター環境でのDPI変化時の座標ずれ**
  - Mitigation: 将来的な`WM_DPICHANGED`イベント対応時に再検証、現時点では既知の制約として文書化

## References
- [AdjustWindowRectExForDpi - Win32 API](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-adjustwindowrectexfordpi) — クライアント領域からウィンドウ全体への矩形変換
- [GetDpiForWindow - Win32 API](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getdpiforwindow) — ウィンドウのDPI値取得
- [SetWindowPos - Win32 API](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos) — ウィンドウ位置・サイズ設定
- Gap Analysis: `.kiro/specs/client-area-positioning/gap-analysis.md` — 実装アプローチの詳細比較
