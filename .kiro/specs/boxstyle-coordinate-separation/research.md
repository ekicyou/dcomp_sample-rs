# Research & Design Decisions

## Summary
- **Feature**: `boxstyle-coordinate-separation`
- **Discovery Scope**: Extension（既存ドラッグ/レイアウト/ウィンドウハンドラシステムの改修）
- **Key Findings**:
  1. `WM_WINDOWPOSCHANGED` で `get_mut::<BoxStyle>()` が1回でinsetとsizeの両方を書く → サイズ不変時に `get_mut` を呼ばない分岐が必要
  2. `DragState` enum にはHWNDフィールドがなく、`DragConfig.move_window` も未キャッシュ → enum拡張が必要
  3. `WindowPos.position` を spawn 前に設定すれば `CreateWindowExW` に直接渡されるため、BoxStyle.inset を初期位置に使う必要がない

## Research Log

### EntityWorldMut の借用制約
- **Context**: `WM_WINDOWPOSCHANGED` ハンドラで BoxStyle のサイズ変更を事前判定する方法
- **Findings**:
  - 現在の `entity_ref` は `EntityWorldMut` 型。`get::<BoxStyle>()` で読み取り後に `get_mut::<BoxStyle>()` を呼ぶと借用衝突
  - 代替: `entity_ref.get::<BoxStyle>().map(|bs| bs.size.clone())` で値をコピーし、`drop` 後に `get_mut` を呼ぶパターン
  - bevy_ecs 0.18.0 では `EntityWorldMut::get()` は `&self` で `get_mut()` は `&mut self` なので直接連続呼び出しは不可
- **Implications**: サイズ比較ロジックは `get_mut` 前に値コピー方式で実装。もしくは `get_mut` 後に値比較し、変更なければ `bypass_change_detection` で巻き戻す方式

### DragState enum 拡張設計
- **Context**: WndProcレベルドラッグで HWND と move_window フラグが必要
- **Findings**:
  - 現在の `Dragging` バリアント: `{ entity, start_pos, current_pos, prev_pos, start_time }`
  - 追加フィールド: `hwnd: HWND`, `initial_window_pos: POINT`, `move_window: bool`
  - `DragConstraint` 情報もキャッシュ候補だが、Option<DragConstraint> がコピー可能なのでそのまま格納可能
  - HWND は `WindowHandle` コンポーネントから取得（ECS → thread_local へのコピー）
- **Implications**: `start_dragging()` の引数拡張。ECS側の `dispatch_drag_events` で Window 探索時にキャッシュ情報を DragState にセットする

### ドラッグ終了時の ECS 同期パス
- **Context**: WndProc レベルドラッグ後に ECS の一貫性を回復する方法
- **Findings**:
  - `DragState::JustEnded` が設定されると、次のECSフレームの `dispatch_drag_events` が `flush()` で検知
  - `DragTransition::Ended` 処理で `DraggingState` を remove
  - **重要**: ドラッグ中は `guarded_set_window_pos()` で移動 → `WM_WINDOWPOSCHANGED` (echo) → `WindowPos` が `bypass_change_detection` で更新 → `Changed<WindowPos>` 不発火
  - ドラッグ終了時に最終位置を `WindowPos` に `DerefMut` で書き込めば `Changed<WindowPos>` が発火 → `sync_window_arrangement_from_window_pos` → Arrangement 回復
- **Implications**: ドラッグ終了は `dispatch_drag_events` 内で WindowPos を DerefMut 更新するのが安全。または専用の drag_end_sync システムを追加

### ウィンドウ初期位置の代替パス
- **Context**: Examples が `BoxStyle.inset` で初期位置を指定しているが、Req 1 AC5 で Window の inset を常時 0/None にする
- **Findings**:
  - `create_windows` システムは `WindowPos.position` を直接読む（[window_system.rs L79-L99](crates/wintf/src/ecs/window_system.rs#L79-L99)）
  - `CW_USEDEFAULT` でない値が入っていれば、その座標で CreateWindowExW が呼ばれる
  - Examples を `WindowPos { position: Some(POINT { x: 100, y: 100 }), size: Some(SIZE { cx: 800, cy: 600 }) }` に変更するだけでよい
- **Implications**: 破壊的変更なし。Examples の修正のみ

### update_arrangements での Window offset スキップ方式
- **Context**: Req 3 AC5 — taffy 結果で Window の Arrangement.offset を上書きしない
- **Findings**:
  - `update_arrangements_system` のクエリに `Without<Window>` フィルタを追加すれば、Window エンティティを完全スキップ可能
  - ただし Window の Arrangement.size（ウィンドウサイズ）も taffy から更新されなくなる
  - 代替: offset のみスキップし、size と scale は更新する → クエリフィルタではなく本体内で `Window` の有無をチェックし offset の書き込みだけスキップ
- **Implications**: クエリ内分岐方式を採用。`Option<&Window>` をクエリに追加し、`Window` がある場合は offset のみ据え置き

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| Option C | WndProcレベル直接SetWindowPos | ECSフレーム待ち排除、最短パス | WndProcロジック増加、DragConstraint移植 | ギャップ分析で推奨済み |

## Design Decisions

### Decision: WM_WINDOWPOSCHANGED での サイズ変更判定方式
- **Context**: `get_mut::<BoxStyle>()` を呼ぶと無条件で `Changed` が発火する
- **Alternatives Considered**:
  1. **事前コピー比較**: `get::<BoxStyle>()` で読んでからサイズ比較 → 変更ありなら `get_mut`
  2. **事後 bypass**: `get_mut` 後に値比較 → 変更なしなら `bypass_change_detection` でロールバック
- **Selected Approach**: 案2（事後 bypass）
- **Rationale**: `EntityWorldMut` の借用制約で案1は複雑になる。案2は bevy_ecs の `Mut::bypass_change_detection` が標準APIとして存在し、パターンとして確立されている
- **Trade-offs**: 毎回 `get_mut` を呼ぶオーバーヘッド（軽微）。コードの意図が明示的

### Decision: DragState への HWND キャッシュタイミング
- **Context**: WndProcレベルでSetWindowPos を呼ぶにはHWNDが必要
- **Selected Approach**: `dispatch_drag_events` の `DragTransition::Started` 処理で、親Window 探索時に `WindowHandle.hwnd` を取得し `DragState::Dragging` にセット
- **Rationale**: 既存の親Window探索ロジックを流用可能。ECS→thread_local の転送は1回のみ（ドラッグ開始時）

### Decision: update_arrangements_system での Window offset スキップ方式
- **Context**: taffy が Window の location=(0,0) を計算して Arrangement.offset を上書きするリスク
- **Selected Approach**: クエリに `Option<&Window>` を追加し、Window エンティティの場合は offset の書き込みをスキップ（size/scale は更新）
- **Rationale**: `Without<Window>` フィルタだと size/scale も更新されなくなり不適切。本体内分岐が最も柔軟

### Decision: ドラッグ終了時の ECS 同期方式
- **Context**: WndProcレベルドラッグ終了後にECSの WindowPos/Arrangement 整合性を回復する必要がある
- **Selected Approach**: `dispatch_drag_events` の `DragTransition::Ended` 処理内で、DragState の最終位置を `WindowPos.position` に DerefMut で書き込み → `Changed<WindowPos>` 発火 → PostLayout で `sync_window_arrangement_from_window_pos` が Arrangement を更新
- **Rationale**: 既存パイプラインを最大限活用。追加システム不要

## Risks & Mitigations
- **Risk**: ドラッグ中の WM_WINDOWPOSCHANGED echo で BoxStyle.size が変更される可能性（DPI変更中のドラッグ等）→ サイズ変更時のみ `Changed<BoxStyle>` を発火する設計で対処
- **Risk**: DragConstraint の WndProc レベル適用漏れ → thread_local にキャッシュ、設計で詳細化
- **Risk**: ドラッグ終了→WindowPos書き込み→window_pos_sync→SetWindowPos echo のフィードバックループ → `guarded_set_window_pos` + `is_self_initiated` による既存エコーバック防止で収束

## References
- bevy_ecs 0.18.0 Change Detection: `Mut::bypass_change_detection()` — `Changed<T>` フラグを発火せずにコンポーネントを更新する標準API
- Win32 `SetWindowPos` / `WM_WINDOWPOSCHANGED`: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setwindowpos
