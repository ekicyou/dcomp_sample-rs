# Research & Design Decisions

## Summary
- **Feature**: `layout-to-graphics-sync`
- **Discovery Scope**: Extension（既存PostLayoutスケジュール・ECSコンポーネント・グラフィックスシステムへの統合拡張）
- **Key Findings**:
  - PostLayoutスケジュールの既存システム順序は確立されており、4つの新規システムを挿入可能
  - Bevy ECS 0.14+の`PartialEq` derive変更検知最適化が利用可能
  - WinUI3のエコーバック検知パターンが適用可能（last_sent値キャッシュ）
  - init_window_surfaceは完全冗長で削除可能（visual_resource_management_systemが担当）

## Research Log

### 既存スケジュール順序とシステム実行チェーン
- **Context**: 新規システムをどこに挿入すべきか判断するため、PostLayoutスケジュールの現在の実行順序を調査
- **Sources Consulted**:
  - `crates/wintf/src/ecs/world.rs` (line 175-214): PostLayoutスケジュール登録
  - `crates/wintf/src/ecs/layout/systems.rs`: propagate_global_arrangementsシステム
  - `crates/wintf/src/ecs/graphics/systems.rs`: init_window_surface等
- **Findings**:
  - 既存順序: `propagate_global_arrangements` → `update_window_pos_system`
  - 新規システム挿入ポイント: `propagate_global_arrangements`の直後
  - `.after()`による依存関係チェーンが確立されている
  - init_window_surfaceは7番目に実行されるが実質的に何もしていない
- **Implications**:
  - 4つの新規システムを`propagate_global_arrangements`と`update_window_pos_system`の間に挿入
  - 既存のチェーン構造を維持しながら拡張可能

### Visual/VisualGraphics/SurfaceGraphicsコンポーネント構造
- **Context**: Surface再作成のために必要なコンポーネント関係を理解
- **Sources Consulted**:
  - `crates/wintf/src/ecs/graphics/components.rs` (line 158-175): Visual定義
  - `crates/wintf/src/ecs/graphics/components.rs` (line 68-98): VisualGraphics定義
  - `crates/wintf/src/ecs/graphics/components.rs` (line 99-140): SurfaceGraphics定義
  - `crates/wintf/src/ecs/graphics/visual_manager.rs` (line 1-80): リソース管理
- **Findings**:
  - `Visual`: 論理コンポーネント（size: Vector2, opacity, is_visible等）
  - `VisualGraphics`: GPU資源（IDCompositionVisual3ラッパー）
  - `SurfaceGraphics`: GPU資源（IDCompositionSurfaceラッパー + sizeフィールド）
  - `SurfaceGraphics`はon_replace hookで`SurfaceUpdateRequested`を自動追加
  - `visual_resource_management_system`が`Added<Visual>`で両方のGPU資源を作成
- **Implications**:
  - `resize_surface_from_visual`は`Changed<Visual>`をトリガーにできる
  - Surface再作成には`VisualGraphics::visual()`と`GraphicsCore`が必要
  - 既存の`create_surface_for_window`ヘルパー関数が再利用可能

### WindowPosコンポーネント構造
- **Context**: エコーバック検知のためのフィールド追加可能性を確認
- **Sources Consulted**:
  - `crates/wintf/src/ecs/window.rs` (line 174-262): WindowPos定義
- **Findings**:
  - すでに`PartialEq` deriveが実装済み
  - `position: Option<POINT>`, `size: Option<SIZE>`フィールド存在
  - `zorder`, `no_redraw`等のフラグが多数定義済み
  - エコーバック用の`last_sent_*`フィールドは未定義
- **Implications**:
  - `last_sent_position: Option<(i32, i32)>`と`last_sent_size: Option<(i32, i32)>`を追加可能
  - 既存のフィールド構造と整合性が取れる

### WM_WINDOWPOSCHANGEDハンドラの現状
- **Context**: メッセージハンドリング実装の既存パターンを確認
- **Sources Consulted**:
  - `crates/wintf/src/win_message_handler.rs` (line 1052): WM_WINDOWPOSCHANGED実装
- **Findings**:
  - 関数定義は存在するが、実装は空（`None`を返すのみ）
  - WM_MOVE (line 616)とWM_SIZE (line 932)が現在使用中
  - `WinState` traitを通じてECS Worldにアクセス可能
- **Implications**:
  - WM_WINDOWPOSCHANGEDの実装は完全に新規
  - WM_MOVE/WM_SIZEは削除対象（Requirement 7）
  - WINDOWPOS構造体からの位置・サイズ抽出が必要

### Bevy ECS 0.14+の変更検知最適化
- **Context**: PartialEq deriveによる自動最適化の動作を確認
- **Sources Consulted**:
  - Bevy ECS 0.14 changelog（既知の機能）
  - `WindowPos`の既存PartialEq実装
- **Findings**:
  - `#[derive(PartialEq)]`でDrop時に自動比較
  - 値が同じ場合は`Changed<T>`フラグが立たない
  - `Mut::bypass_change_detection()`で手動制御可能
- **Implications**:
  - `Visual`に`#[derive(PartialEq)]`を追加（Requirement 8）
  - エコーバック記録時に`bypass_change_detection()`を使用

## Architecture Pattern Evaluation

本機能は既存アーキテクチャへの統合拡張であり、新規パターン選択は不要。既存のECSスケジュールベース・システムチェーンパターンを踏襲。

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| ECSスケジュール拡張（選択） | PostLayoutスケジュールに4つの新規システムを`.after()`チェーンで追加 | 既存パターンと整合、変更検知の自動化、テスト容易性 | スケジュール順序の複雑化 | Steering準拠、bevy_ecs標準パターン |
| 手動同期ループ | 単一システム内でループ処理 | シンプル | 変更検知が使えない、保守性低下 | 不採用 |

## Design Decisions

### Decision: PostLayoutスケジュールへの4システム挿入
- **Context**: GlobalArrangement更新後のフローが未実装
- **Alternatives Considered**:
  1. 単一の大きなシステムで全処理を実行 → 変更検知が活用できない
  2. 別スケジュール（例: PostGraphics）を新設 → 既存構造を複雑化
- **Selected Approach**:
  - `propagate_global_arrangements`の直後に4つのシステムを順次追加
  - 各システムは単一責任で`Changed<T>`クエリを使用
- **Rationale**:
  - 既存のPostLayoutスケジュール構造を維持
  - Bevy ECSの変更検知機能を最大限活用
  - 各システムが独立してテスト可能
- **Trade-offs**:
  - 利点: 保守性、テスト容易性、最適化（Changed検知）
  - 欠点: システム数が増加（4つ）、実行順序の把握が必要
- **Follow-up**: スケジュール順序図をドキュメント化

### Decision: init_window_surfaceの削除
- **Context**: visual_resource_management_systemと役割が完全重複
- **Alternatives Considered**:
  1. init_window_surfaceを残してresize_surface_from_visualと統合 → 責任が曖昧
  2. init_window_surfaceをリファクタリング → 本質的に不要
- **Selected Approach**: init_window_surfaceを完全削除
- **Rationale**:
  - visual_resource_management_systemが初回Surface作成を担当
  - resize_surface_from_visualが`Changed<Visual>`でリサイズを担当
  - init_window_surfaceは実質的に何も実行していない
- **Trade-offs**:
  - 利点: 冗長性排除、実行コスト削減、保守性向上
  - 欠点: なし
- **Follow-up**: cleanup_graphics_needs_initの動作確認

### Decision: エコーバック検知にWinUI3パターンを採用
- **Context**: SetWindowPos呼び出しによるWM_WINDOWPOSCHANGEDエコーバックで無限ループリスク
- **Alternatives Considered**:
  1. フラグベース（is_echoback: bool）→ タイミング制御が複雑
  2. タイムスタンプベース → 精度問題
  3. 送信値キャッシュ（WinUI3パターン）→ 確実
- **Selected Approach**: `last_sent_position`/`last_sent_size`フィールドをWindowPosに追加
- **Rationale**: Microsoft WinUI3の実装パターンと同じで信頼性が高い
- **Trade-offs**:
  - 利点: 確実な検知、シンプルな実装
  - 欠点: WindowPosのメモリ増加（16バイト）
- **Follow-up**: bypass_change_detection()の使用確認

### Decision: Visualに PartialEq追加
- **Context**: 不要な変更検知を防ぐ最適化
- **Alternatives Considered**:
  1. 手動比較ロジック → 保守コスト高
  2. PartialEq derive（Bevy ECS 0.14+の自動最適化）→ 自動化
- **Selected Approach**: `#[derive(PartialEq)]`を追加
- **Rationale**: Bevy ECS 0.14+が自動でDrop時に値比較し、同じ値なら変更フラグを立てない
- **Trade-offs**:
  - 利点: 自動最適化、コード簡潔
  - 欠点: すべてのフィールドが比較対象（is_visible, opacity等も含む）
- **Follow-up**: パフォーマンステスト

## Risks & Mitigations

- **Risk**: init_window_surface削除後にSurface作成が失敗する
  - **Mitigation**: visual_resource_management_systemの動作を事前確認、テストで検証

- **Risk**: PostLayoutスケジュール順序の複雑化でデバッグが困難
  - **Mitigation**: スケジュール順序図をドキュメント化、ログ出力を充実

- **Risk**: エコーバック検知の精度不足（浮動小数点誤差等）
  - **Mitigation**: i32での厳密比較、誤検知時の影響は軽微（1フレームの無駄なレイアウト計算）

- **Risk**: PartialEq追加でVisualの全フィールド比較によるオーバーヘッド
  - **Mitigation**: ベンチマークテストで確認、必要なら手動実装に切り替え

## References
- [Bevy ECS Change Detection](https://docs.rs/bevy_ecs/latest/bevy_ecs/change_detection/) — PartialEq最適化の動作
- [WinUI3 DesktopWindowImpl.cpp](https://github.com/microsoft/microsoft-ui-xaml) — エコーバック検知パターンの参考実装
- [Windows API: WM_WINDOWPOSCHANGED](https://learn.microsoft.com/en-us/windows/win32/winmsg/wm-windowposchanged) — メッセージ仕様
- [Taffy Layout Engine](https://github.com/DioxusLabs/taffy) — レイアウトエンジン統合状況
