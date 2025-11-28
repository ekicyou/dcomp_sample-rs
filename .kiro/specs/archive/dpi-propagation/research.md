# Research & Design Decisions: dpi-propagation

---

## Summary
- **Feature**: `dpi-propagation`
- **Discovery Scope**: Extension（既存システムの拡張）
- **Key Findings**:
  1. `DpiTransform`は定義のみで未使用 — 安全に削除可能、実装コードは`DPI`に流用
  2. `on_window_handle_add`フックが存在 — DPI自動付与の挿入ポイントとして最適
  3. `Or<(Changed<A>, Changed<B>)>`パターンが既に複数箇所で使用 — 実装の前例あり

---

## Research Log

### bevy_ecs Or条件クエリパターン
- **Context**: `update_arrangements_system`に`Changed<DPI>`を追加する方法
- **Sources Consulted**: 
  - 既存コード `layout/systems.rs:23` (`Or<(Changed<Arrangement>, Added<GlobalArrangement>)>`)
  - 既存コード `widget/text/draw_labels.rs:20` (`Or<(Changed<Label>, Without<GraphicsCommandList>)>`)
- **Findings**:
  - `Or`フィルターは複数の変更条件をORで結合可能
  - 変更元の区別は不可能（どのコンポーネントが変更されたかは判定できない）
  - 既存パターンでは「マッチしたら全フィールドを再計算」アプローチを採用
- **Implications**: 要件定義で決定済みの「全フィールド再計算」戦略に合致

### WindowHandleフックシステム
- **Context**: DPIコンポーネントの自動付与方法
- **Sources Consulted**: 
  - `window.rs:212-227` (`on_window_handle_add`関数)
- **Findings**:
  - `#[component(on_add = ...)]`属性でフック登録
  - フック内で`DeferredWorld`と`HookContext`を受け取る
  - `world.commands().entity(...).insert(...)`でコンポーネント追加可能
  - 現在はApp通知のみ実装
- **Implications**: DPI挿入ロジックを`on_window_handle_add`に追加するのが最適

### GetDpiForWindow API
- **Context**: ウィンドウのDPI値取得方法
- **Sources Consulted**: 
  - `window.rs:49` (`WindowHandle::get_dpi()`)
  - Microsoft Docs: GetDpiForWindow
- **Findings**:
  - `WindowHandle`に`get_dpi()`メソッドが既存
  - 戻り値: `u32`（単一のDPI値、通常X/Y同一）
  - 失敗時は0を返す
- **Implications**: 既存メソッドを活用、Y方向は同一値を使用

### WM_DPICHANGED wparam解析
- **Context**: DPI変更メッセージのパラメータ形式
- **Sources Consulted**: 
  - `window.rs:267-269` (`DpiTransform::from_WM_DPICHANGED`)
  - `window_proc.rs:182-184` (既存ログ実装)
- **Findings**:
  - LOWORD(wparam) = X方向DPI
  - HIWORD(wparam) = Y方向DPI
  - 既存の解析ロジック: `(wparam.0 & 0xFFFF) as u16`, `((wparam.0 >> 16) & 0xFFFF) as u16`
- **Implications**: `DpiTransform::from_WM_DPICHANGED`のロジックをそのまま流用

### DpiTransform使用状況
- **Context**: 既存コンポーネントの削除可否判断
- **Sources Consulted**: 
  - `grep_search`で全`.rs`ファイルを検索
- **Findings**:
  - `DpiTransform`は`window.rs`での定義のみ
  - コードベース内で実際に使用・インスタンス化されている箇所なし
  - `pub use`でエクスポートされていない（内部定義のみ）
- **Implications**: 安全に削除可能、実装コード（`from_WM_DPICHANGED`、`from_dpi`）は`DPI`に移行

---

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: window.rs拡張 | DPIをwindow.rsに定義 | Windowエンティティ専用、既存パターン踏襲 | window.rsがやや肥大化 | **採用** |
| B: dpi.rs新設 | 独立モジュール化 | 責務分離が明確 | 単一コンポーネントにはオーバーヘッド | 却下 |

---

## Design Decisions

### Decision: DPIコンポーネント配置場所
- **Context**: 新しい`DPI`コンポーネントをどこに定義するか
- **Alternatives Considered**:
  1. `window.rs`に追加（他のWindow関連コンポーネントと同列）
  2. `ecs/dpi.rs`を新設（独立モジュール化）
- **Selected Approach**: `window.rs`に追加
- **Rationale**: 
  - DPIはWindowエンティティ専用コンポーネント
  - 既存の`WindowHandle`, `WindowStyle`等と同じ配置が自然
  - 削除対象の`DpiTransform`と同じファイルで置換できる
- **Trade-offs**: window.rsの行数増加（約40行追加、約35行削除 → 純増約5行）
- **Follow-up**: `pub use`への追加を忘れずに実施

### Decision: フック内でのDPI取得方法
- **Context**: `on_window_handle_add`でDPIをどう取得するか
- **Alternatives Considered**:
  1. `WindowHandle::get_dpi()`を呼び出し
  2. `GetDpiForWindow` APIを直接呼び出し
- **Selected Approach**: `WindowHandle::get_dpi()`を使用
- **Rationale**: 既存のラッパーメソッドを活用、HWNDアクセスをカプセル化
- **Trade-offs**: なし（既存メソッドの活用）
- **Follow-up**: 失敗時のデフォルト値(96)処理を実装

### Decision: update_arrangements_systemのクエリ変更方式
- **Context**: DPI変更検知をどう実装するか
- **Alternatives Considered**:
  1. 既存クエリに`Or`条件を追加
  2. DPI専用の別システムを作成
- **Selected Approach**: 既存クエリに`Or<(Changed<TaffyComputedLayout>, Changed<DPI>)>`を追加
- **Rationale**: 
  - 要件定義のDesign Decisionに準拠
  - Arrangement変更の責務を単一システムに集約
  - 既存パターンと整合性あり
- **Trade-offs**: クエリがやや複雑化するが、責務の分散を防ぐ
- **Follow-up**: `Option<&DPI>`パラメータを追加

---

## Risks & Mitigations

- **Risk 1: フック内でのEcsWorld借用競合** — `DeferredWorld`のコマンドキューを使用して遅延挿入（既存パターン）
- **Risk 2: GetDpiForWindow失敗** — デフォルト値96を使用（要件R2.3）
- **Risk 3: WM_DPICHANGEDでのEntity取得失敗** — 既存の`get_entity_from_hwnd`を使用、Noneの場合はスキップ

---

## References

- [GetDpiForWindow - Win32 API](https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-getdpiforwindow)
- [WM_DPICHANGED - Win32 API](https://learn.microsoft.com/en-us/windows/win32/hidpi/wm-dpichanged)
- bevy_ecs 0.17 Query filters (`Or`, `Changed`)
