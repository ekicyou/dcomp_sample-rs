# Research & Design Decisions: dpi-coordinate-transform-survey

---

## Summary
- **Feature**: `dpi-coordinate-transform-survey`
- **Discovery Scope**: Complex Integration（既存システムの深層分析 + 外部リファレンスモデル調査）
- **Key Findings**:
  1. WMメッセージの伝搬チェーンにおいて、`WM_DPICHANGED → SetWindowPos → WM_WINDOWPOSCHANGED` の同期カスケードが DpiChangeContext スレッドローカルで正しく制御されているが、フィードバックループ防止機構が3層（WindowPosChanged フラグ / エコーバック検知 / RefCell 再入保護）で冗長
  2. `BoxStyle.inset`（物理px）と `BoxStyle.size`（DIP）の単位混在が構造的設計課題の根本原因。Taffy レイアウトエンジンは単位非認識のため、入力座標系の統一がフレームワーク側の責務
  3. `sync_window_pos` と `update_window_pos_system` の重複が`Changed<WindowPos>` の不必要な再トリガーを引き起こす潜在リスク
  4. WPF の DIP 統一モデルが wintf の To-Be アーキテクチャの最有力候補。物理ピクセル変換は出力層（Win32 API / DirectComposition Visual）のみで実施する設計が妥当

---

## Research Log

### トピック 1: WM メッセージ伝搬マトリクスと循環リスク

- **Context**: ユーザー追加指示により、DPI/座標に影響するウィンドウメッセージの伝搬（入出力・API呼び出しによるWM発動）を網羅的に調査
- **Sources Consulted**: 
  - `crates/wintf/src/ecs/window_proc/mod.rs` — ディスパッチテーブル
  - `crates/wintf/src/ecs/window_proc/handlers.rs` — 各WMハンドラ実装
  - `crates/wintf/src/ecs/window.rs` — DpiChangeContext, SetWindowPosCommand
  - Microsoft Learn: [High DPI Desktop Application Development](https://learn.microsoft.com/en-us/windows/win32/hidpi/high-dpi-desktop-application-development-on-windows)

- **Findings**:

  **ハンドルされるDPI/座標関連メッセージ（全16種）**:

  | メッセージ | 重要度 | 入力座標系 | 出力先 | トリガーするWM |
  |-----------|--------|-----------|--------|---------------|
  | `WM_DPICHANGED` | ★★★ | wparam: DPI値, lparam: suggested_rect（物理px/スクリーン） | DpiChangeContext(TLS) | → `SetWindowPos` → `WM_WINDOWPOSCHANGED` |
  | `WM_WINDOWPOSCHANGED` | ★★★ | lparam: WINDOWPOS（物理px/ウィンドウ全体） | DPI, WindowPos, BoxStyle | → `try_tick_on_vsync` → `flush_window_pos_commands` → `SetWindowPos` |
  | `WM_DISPLAYCHANGE` | ★★ | wparam: bit depth, lparam: 解像度 | App.display_change | → tick内で Monitor 再列挙 |
  | `WM_NCHITTEST` | ★★ | lparam: スクリーン座標（物理px） | LRESULT(HTCLIENT) | なし |
  | `WM_MOUSEMOVE` | ★★ | lparam: クライアント座標（物理px） | PointerState, DragState | なし |
  | `WM_L/R/M/XBUTTONDOWN/UP` | ★ | lparam: クライアント座標（物理px） | PointerState, DragState | なし |
  | `WM_MOUSEWHEEL` | ○ | wparam: デルタ | WheelState | なし |
  | `WM_NCCREATE` | ○ | — | Entity ID保存 | なし |

  **ハンドルされていない注目メッセージ（DefWindowProcW委譲）**:

  | メッセージ | 影響 | 備考 |
  |-----------|------|------|
  | `WM_SIZE` / `WM_SIZING` | 低 | `WM_WINDOWPOSCHANGED` で代替 |
  | `WM_MOVE` / `WM_MOVING` | 低 | `WM_WINDOWPOSCHANGED` で代替 |
  | `WM_NCCALCSIZE` | 中 | カスタムフレーム実装時に必要 |
  | `WM_GETMINMAXINFO` | 中 | サイズ制約実装時に必要 |
  | `WM_WINDOWPOSCHANGING` | 低 | 事前介入不要の現設計 |
  | `WM_GETDPISCALEDSIZE` | 中 | Microsoft推奨: DPI変更時のカスタムサイズ計算に使用可能 |

- **Implications**:
  - 現在のメッセージハンドリングは最小限で健全。`WM_WINDOWPOSCHANGED` に集約する設計は Win32 の推奨パターンに合致
  - ただし `WM_GETDPISCALEDSIZE` 未対応は、DPI変更時のウィンドウサイズ精密制御の障壁
  - `WM_NCCALCSIZE` 未対応は、カスタムタイトルバー等の将来要件でボトルネック化

### トピック 2: メッセージカスケードチェーンと循環分析

- **Context**: フィードバックループの有無と防止機構の完全性を検証
- **Sources Consulted**:
  - `handlers.rs` L118-310（WM_WINDOWPOSCHANGED）
  - `window.rs` L85-177（SetWindowPosCommand）
  - `graphics/systems.rs` L699-909（sync_window_pos, apply_window_pos_changes）
  - `world.rs` L614-630（スケジュール実行順序）

- **Findings**:

  **カスケードチェーン識別（全5チェーン）**:

  ```
  Chain 1 (DPI変更): WM_DPICHANGED → DpiChangeContext::set → SetWindowPos
    → [同期] WM_WINDOWPOSCHANGED → DpiChangeContext::take → DPI/WindowPos/BoxStyle更新
    → try_tick_on_vsync → Layout/Composition → flush_window_pos_commands
    → [可能性] SetWindowPos → WM_WINDOWPOSCHANGED（再入）

  Chain 2 (ウィンドウ作成): create_windows → CreateWindowExW
    → WM_NCCREATE → WM_WINDOWPOSCHANGED

  Chain 3 (ユーザー操作): ドラッグ/リサイズ → WM_WINDOWPOSCHANGED
    → WindowPos/BoxStyle更新 → try_tick_on_vsync

  Chain 4 (ECS→Win32): WindowPos変更 → apply_window_pos_changes
    → SetWindowPosCommand::enqueue → flush → SetWindowPos
    → WM_WINDOWPOSCHANGED → WindowPosChanged=true で抑制

  Chain 5 (ディスプレイ変更): WM_DISPLAYCHANGE → App::mark_display_change
    → tick内 detect_display_change_system → Monitor再列挙
  ```

  **フィードバックループ防止機構（3層防御）**:

  | 層 | メカニズム | 有効範囲 | 潜在リスク |
  |----|----------|---------|-----------|
  | L1: `WindowPosChanged` フラグ | WM_WINDOWPOSCHANGED処理中 `true` → `apply_window_pos_changes` スキップ | WM_WINDOWPOSCHANGED内のtick | ⚠ tick外の`sync_window_pos`には無効 |
  | L2: エコーバック検知 | `last_sent_position/size` と受信値比較 → 一致時スキップ | 値一致時のみ | ⚠ f32→i32丸めで微差が出た場合は検知失敗 |
  | L3: RefCell再入保護 | `try_borrow_mut()` 失敗時スキップ | 同一WM内の再入 | ✓ 堅牢 |

  **発見された潜在的循環パス**:

  ```
  sync_window_pos (PostLayout) → WindowPos書き込み(Changed!)
    → [次フレーム] apply_window_pos_changes (UISetup) → SetWindowPos
    → WM_WINDOWPOSCHANGED → WindowPos再更新 → BoxStyle更新
    → Layout再計算 → sync_window_pos → ...
  ```
  
  ただし、L2エコーバック検知が値一致で切断するため、定常状態では発散しない。問題は初回収束までに1-2フレームのジッター可能性がある点。

- **Implications**:
  - 3層防御は概ね堅牢だが、`sync_window_pos` と `update_window_pos_system` の重複が不必要な `Changed<WindowPos>` トリガーを生む
  - To-Be設計では、WindowPos → ECS の逆流パスを明確に制御する単一のゲートシステムが望ましい

### トピック 3: 座標変換関連の Win32 API 挙動（Per-Monitor v2 下）

- **Context**: Req 1.4 — Win32 API の入出力座標系と PMv2 下での挙動文書化
- **Sources Consulted**:
  - Microsoft Learn: [High DPI Desktop Application Development](https://learn.microsoft.com/en-us/windows/win32/hidpi/high-dpi-desktop-application-development-on-windows)
  - Microsoft Learn: [DPI_AWARENESS_CONTEXT](https://learn.microsoft.com/en-us/windows/win32/hidpi/dpi-awareness-context)
  - `process_singleton.rs` L64: `SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2)`

- **Findings**:

  **Per-Monitor v2 下での API 挙動マトリクス**:

  | API | 入力座標系 | 出力座標系 | DPI引数 | PMv2 での挙動 |
  |-----|----------|----------|--------|-------------|
  | `SetWindowPos` | 物理px/スクリーン | — | なし | 物理ピクセルをそのまま使用。仮想化なし |
  | `GetWindowRect` | — | 物理px/スクリーン RECT | なし | 実際の物理ピクセルを返す |
  | `GetClientRect` | — | クライアント相対 RECT (0,0起点) | なし | 実際の物理ピクセルサイズ |
  | `AdjustWindowRectExForDpi` | クライアントRECT → ウィンドウRECT | 物理px | 明示的DPI引数 | 指定DPIの非クライアント領域で拡張 |
  | `GetDpiForWindow` | — | DPI値 (u32) | なし | ウィンドウが存在するモニターのDPIを返す |
  | `GetDpiForMonitor` | — | DPI値 | MDT_EFFECTIVE_DPI | モニター固有のDPIを返す |
  | `ScreenToClient` | スクリーン座標 → クライアント座標 | 物理px | なし | 物理ピクセルで変換 |
  | `ClientToScreen` | クライアント座標 → スクリーン座標 | 物理px | なし | 物理ピクセルで変換 |
  | `GetCursorPos` | — | スクリーン座標/物理px | なし | PMv2では仮想化なし |
  | `GetSystemMetrics` | — | システムDPI基準 | なし | ⚠ PMv2でも**システムDPI**基準。`GetSystemMetricsForDpi` を使うべき |
  | `CreateWindowExW` | 物理px/スクリーン（Xが`CW_USEDEFAULT`の場合OS任せ） | — | なし | 作成スレッドのDPIコンテキストに基づく |
  | `WM_MOUSEMOVE` lparam | — | クライアント座標/物理px | — | 物理ピクセル値 |
  | `WM_NCHITTEST` lparam | — | スクリーン座標/物理px | — | 物理ピクセル値 |

  **PMv2 の重要な特性**:
  1. 非クライアント領域（タイトルバー、スクロールバー）の自動DPIスケーリング
  2. 共通コントロールのテーマ描画ビットマップの自動スケーリング
  3. `CreateDialog` によるダイアログの自動スケーリング
  4. 子HWNDにもDPI変更通知が配信される
  5. **仮想化なし**: すべてのAPIが実際の物理ピクセル値を返す

- **Implications**:
  - PMv2 環境では全座標がraw物理ピクセル → フレームワーク内部で DIP ↔ 物理px 変換を明示的に管理する責務がある
  - `GetSystemMetrics` は非DPI対応。既存コードで `GetSystemMetrics(SM_XVIRTUALSCREEN)` 等を使用中 → `GetSystemMetricsForDpi` への移行を検討すべき（ただし仮想デスクトップ座標は物理pxが正しいため影響軽微）

### トピック 4: WPF / WinUI3 の座標系モデル比較

- **Context**: Req 4 — あるべき座標変換アーキテクチャの参照モデル調査
- **Sources Consulted**:
  - Microsoft Learn: [WPF Graphics Rendering Overview](https://learn.microsoft.com/en-us/dotnet/desktop/wpf/graphics-multimedia/wpf-graphics-rendering-overview)
  - Microsoft Learn: [About Resolution and Device-Independent Graphics](https://learn.microsoft.com/en-us/dotnet/desktop/wpf/graphics-multimedia/wpf-graphics-rendering-overview#about-resolution-and-device-independent-graphics)

- **Findings**:

  **WPF のDIPモデル**:
  - 主要単位: DIP (Device Independent Pixel, 1 DIP = 1/96 inch)
  - **全ての座標・サイズ・マージン・パディングが DIP 単位**
  - ラスタライズ段階でのみ物理ピクセル変換が発生
  - `Visual.Transform` → レンダリング変換（DIP空間内での変換）
  - `Visual.Offset` → DIP単位の親からの相対位置
  - DPIスケーリングはルートVisualのレンダリングトランスフォームとして適用
  - ヒットテスト: DIP空間で実行。物理座標は入力時にDIPに変換

  **WinUI3 のモデル**:
  - UIElement.Scale / UIElement.Translation → DIP空間での変換
  - EffectiveViewportChanged → スクロール・輩DPI変更の通知
  - Compositor → DirectComposition ベースの合成（wintfと同様のアプローチ）
  - DPIスケーリングはComposition層で自動適用

  **wintf への適用パターン（提案）**:

  | 層 | WPF の設計 | wintf の現状 | wintf To-Be |
  |----|----------|------------|-----------|
  | 入力(ポインタ) | 物理px → DIP変換（フレームワーク内） | 物理pxのまま使用 | 物理px → DIP変換を入力層で実施 |
  | レイアウト | DIP統一 | BoxStyle.inset=物理px, size=DIP **混在** | **DIP統一** |
  | レイアウトエンジン | DIP入力 | Taffy: 単位非認識（混在入力） | Taffy: DIP入力に統一 |
  | 配置計算 | DIP空間 | Arrangement.offset=混在（物理px/DIP） | **DIP のみ** |
  | 描画 | DIP → 物理px変換（ラスタライズ時） | DIP × scale → 物理px | 維持 |
  | Win32出力 | DIP → 物理px変換（API呼び出し時） | bounds(物理px) → SetWindowPos | **DIP → 物理px変換を出力層で実施** |

- **Implications**:
  - WPF モデルの核心: 「全内部座標を DIP で統一し、物理ピクセル変換はレンダリング/出力層のみ」
  - wintf では `BoxStyle.inset` の物理px → DIP 統一が最重要の設計変更
  - LayoutRoot の `BoxStyle.inset`（仮想デスクトップ座標）も DIP 化が必要
  - Monitor エンティティの `BoxStyle.inset` も DIP 化（物理px ÷ monitor.dpi.scale）

### トピック 5: `sync_window_pos` と `update_window_pos_system` の重複問題

- **Context**: コード解析中に発見された設計上の懸念
- **Sources Consulted**:
  - `graphics/systems.rs` L699-789 (`sync_window_pos`)
  - `layout/systems.rs` L385-400 (`update_window_pos_system`)
  - `world.rs` L384-386（PostLayout スケジュール内の実行順）

- **Findings**:
  - 両システムが `GlobalArrangement.bounds → WindowPos` の同一変換を実行
  - PostLayout 内で `sync_window_pos` が先に実行、`update_window_pos_system` が後
  - 2回目の書き込みは `set_if_neq` で値が同一なら `Changed` トリガーを回避するが、`set_if_neq` が使われているかは要確認
  - 重複の起源: おそらく graphics と layout の責務分離時に両方に残った

- **Implications**:
  - To-Be 設計では単一のシステムに統合すべき
  - `WindowPos` 更新は layout (PostLayout) の責務とし、graphics 側は参照のみにする

### トピック 6: PointerState.screen_point の命名不整合

- **Context**: handlers.rs L688 で発見
- **Findings**:
  - `screen_point: PhysicalPoint::new(x, y)` — `x, y` は WM_MOUSEMOVE の lparam から取得した**クライアント座標**
  - 正しいスクリーン座標 `screen_x = x + WindowPos.position.x` は L543-L547 で計算済みだが、PointerState 挿入時には使用されていない
  - ドラッグ処理では `screen_x/y` を正しく使用（L604）

- **Implications**:
  - To-Be ではフィールド名を `client_point` に修正、または実際のスクリーン座標を格納すべき
  - ドラッグ座標チェーンには影響なし（ドラッグは別経路で正しいスクリーン座標を使用）

### トピック 7: Matrix3x2 乗算順序と bounds 計算の乖離

- **Context**: gap-analysis.md で指摘された `translation × scale` 順序の問題
- **Sources Consulted**: `arrangement.rs` L178-236

- **Findings**:
  - `From<Arrangement> for Matrix3x2`: `translation(offset) * scale(scale)` → M31 = offset.x（スケールなし）
  - ただし `Mul<Arrangement> for GlobalArrangement` の bounds 計算では `offset × parent_scale` を手動計算
  - 実際にはtransform行列のM31値は直接使用されず、bounds.left/top がWindowPos等に使用される
  - transform.M31/M32 を直接参照する箇所: `offset_x()`/`offset_y()` アクセサが存在するが、**bounds のかわりに使用している箇所は検出されなかった**
  - 描画時: `render_surface` は `scale_x()` (M11) と `scale_y()` (M22) のみ使用

- **Implications**:
  - 現時点では `transform.M31/M32` の値が不正でも実害なし（bounds が権威的な値）
  - ただし将来、任意の変換（回転・スキュー）をサポートする場合、transform の乗算順序を `scale × translation` に修正する必要がある
  - To-Be 設計方針: 軸平行変換に限定する現設計を維持し、bounds を権威的な値として保持

---

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A. DIP統一モデル（WPF型） | 全内部座標をDIPに統一。物理px変換は出力層のみ | 単位混在バグの根絶。ドラッグ等の座標チェーンが単純化 | BreakingChange: BoxStyle.inset の全使用箇所に影響。LayoutRoot/Monitor の座標もDIP化が必要 | **推奨**。WPF の実績で妥当性が証明済み |
| B. 物理px統一モデル | 全内部座標を物理pxに統一。DPIスケーリングは描画時のみ | 現状からの差分が最小 | BoxStyle.size（現DIP）の変更が必要。レイアウト計算結果がDPI依存になり再利用性低下 | 不採用。レイアウトエンジンの意味が薄れる |
| C. 型安全座標系（Phantom Type） | `LogicalPx<T>` / `PhysicalPx<T>` の型レベル区別 | コンパイル時に単位混在を検出 | 実装コスト高。Taffy との統合が困難 | 将来的な拡張として検討。現段階では過剰 |
| D. ハイブリッド（現状維持+明文化） | 現在の混在を許容し、各コンポーネントの座標系をドキュメント化 | 変更コスト最小 | バグの温床が残る。新規開発者の学習コスト高 | 不採用。構造的課題が温存される |

---

## Design Decisions

### Decision: 内部座標系の統一方針

- **Context**: BoxStyle.inset（物理px）と BoxStyle.size（DIP）の混在が、ドラッグ座標チェーンの複雑化やフレームワーク利用者の座標系意識責務の原因
- **Alternatives Considered**:
  1. Option A: DIP統一モデル — 全内部座標をDIPに統一
  2. Option B: 物理px統一モデル — 全内部座標を物理pxに統一
  3. Option D: 現状維持 — ドキュメント化のみ
- **Selected Approach**: Option A（DIP統一モデル）
- **Rationale**: WPF/WinUI3 の実績で実証済み。Taffy レイアウトエンジンは DIP 値での計算に自然にフィットする。座標系の一貫性により、ドラッグ等のインタラクション実装が大幅に単純化される
- **Trade-offs**: BoxStyle.inset の全使用箇所に影響。LayoutRoot/Monitor/Window の座標をDIP化する必要がある。テストの期待値も修正が必要
- **Follow-up**: `wintf-P1-dpi-scaling` 仕様にて段階的に実装

### Decision: レポート構造とセクション設計

- **Context**: Req 6 で定義された report.md の構造設計
- **Selected Approach**: 7セクション構成（エグゼクティブサマリー / 座標系インベントリ / DPIデータフロー / ドラッグ評価 / To-Beアーキテクチャ / ギャップマトリクス / ロードマップ）
- **Rationale**: requirements.md の Req 6 AC 2 に定義された構成に準拠
- **Follow-up**: 各セクションの詳細構造は design.md の Components で定義

### Decision: WMメッセージ伝搬マトリクスの調査スコープ

- **Context**: ユーザー追加指示による WM メッセージ網羅調査
- **Selected Approach**: 調査スコープを「DPI/座標に影響するメッセージ」に限定し、マトリクスに入出力座標系・トリガーされるメッセージ・循環リスクを記載
- **Rationale**: 本仕様は調査仕様であり、発見事項は report.md に集約。コード修正は後続仕様に委譲
- **Follow-up**: report.md の「DPIデータフロー図」セクションにマトリクスとカスケードチェーン図を含める

---

## Risks & Mitigations

- **R1: DIP統一への移行で既存テストが大量破損** — 段階的移行（Phase 1: Window のみ → Phase 2: Widget → Phase 3: LayoutRoot/Monitor）で影響を局所化
- **R2: sync_window_pos / update_window_pos_system 重複の放置** — report.md で明示的にギャップとして記載し、P1 仕様の前提条件として統合を提言
- **R3: 1.25倍速ドラッグバグの根本原因が未特定のまま設計が進む** — 本仕様のスコープはバグ特定ではなく設計指針策定。To-Be 設計が正しければ、移行により自然解消する見通し
- **R4: LayoutRoot の仮想デスクトップ座標 DIP 化でマルチモニター互換性問題** — 各モニターのDPIが異なる場合、仮想デスクトップのDIP化基準を「プライマリモニターDPI」として統一する案を提示

---

## References

- [High DPI Desktop Application Development on Windows](https://learn.microsoft.com/en-us/windows/win32/hidpi/high-dpi-desktop-application-development-on-windows) — PMv2 の挙動詳細、API置換マトリクス
- [WPF Graphics Rendering Overview](https://learn.microsoft.com/en-us/dotnet/desktop/wpf/graphics-multimedia/wpf-graphics-rendering-overview) — DIP統一モデルの参照実装
- [DPI_AWARENESS_CONTEXT](https://learn.microsoft.com/en-us/windows/win32/hidpi/dpi-awareness-context) — PMv2 (Context -4) の定義と制約
- [WM_DPICHANGED](https://learn.microsoft.com/en-us/windows/win32/hidpi/wm-dpichanged) — suggested_rect の使用義務、循環回避

