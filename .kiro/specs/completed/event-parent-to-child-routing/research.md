# 調査 & 設計判断記録

---
**目的**: ディスカバリーフェーズの調査結果、アーキテクチャ検証、設計根拠を記録する。

**利用方法**:
- ディスカバリーフェーズでの調査活動と成果を記録
- `design.md`に記載するには詳細すぎる設計判断のトレードオフを文書化
- 将来の監査や再利用のための参照情報と証跡を提供
---

## 概要
- **機能**: `event-parent-to-child-routing`
- **ディスカバリースコープ**: Extension（既存システム拡張）
- **主要調査結果**:
  - Tunnelフェーズは`dispatch.rs`で完全実装済み（`dispatch_event_for_handler`関数、127-156行）
  - `taffy_flex_demo`は全ハンドラで`if !ev.is_bubble() { return false; }`としてTunnelを無視
  - 統合作業の範囲: サンプルコードへのデモハンドラ追加のみ、コア実装変更不要

## 調査ログ

### 既存イベントシステムの実装状況
- **背景**: 要件定義時に「Tunnelフェーズは未実装」と誤認されていたが、実装済みであることが判明
- **調査対象**: `crates/wintf/src/ecs/pointer/dispatch.rs`
- **調査結果**:
  - **Phase<T> enum**: Tunnel/Bubbleの2フェーズを型安全に表現（13-47行）
    - `is_tunnel()`, `is_bubble()`, `value()`メソッド提供
  - **EventHandler<T> type**: 統一されたハンドラシグネチャ（52-70行）
    - 引数: `world`, `sender`, `entity`, `ev: &Phase<T>`
    - 戻り値: `bool`（`true`で伝播停止）
  - **dispatch_event_for_handler**: 2フェーズディスパッチ実装（120-156行）
    - Tunnel: `path.iter().rev()`でroot→sender順にイテレート（127-141行）
    - Bubble: `path.iter()`でsender→root順にイテレート（143-156行）
  - **ユニットテスト**: `test_dispatch_stop_propagation`でTunnel/Bubble両フェーズを検証済み
- **示唆**: コア機能は完全実装済み。デモ追加のみで要件を満たせる

### taffy_flex_demoの現在の実装パターン
- **背景**: 既存サンプルがどのようにイベントハンドラを実装しているかを確認
- **調査対象**: `crates/wintf/examples/taffy_flex_demo.rs`
- **調査結果**:
  - **全ハンドラのパターン**: 先頭で`if !ev.is_bubble() { return false; }`をチェック
    - `on_container_pressed` (636-672行)
    - `on_red_box_pressed` (678-724行)
    - `on_image_pressed` (731-753行)
    - `on_green_box_moved` (758-785行)
    - `on_blue_box_pressed` (790-828行)
  - **既存のエンティティ階層**:
    - Window (FlexDemoWindow)
    - ├─ Container (FlexDemoContainer)
    - │   ├─ RedBox (SeikatuImage子含む)
    - │   ├─ GreenBox
    - │   └─ BlueBox
  - **イベント登録パターン**: エンティティ生成時に`OnPointerPressed(handler_fn)`コンポーネントを追加
- **示唆**: 
  - Tunnelデモ追加には既存ハンドラの`if !ev.is_bubble()`部分を拡張
  - GreenBoxに子エンティティ追加が必要（Requirement 3対応）

### WinUI3/WPF/DOMイベントモデルとの対応
- **背景**: 設計ドキュメントで他UIフレームワークとの対応関係を明記する必要
- **参照**: 既存コメント（dispatch.rs）とWinUI3公式ドキュメント
- **対応表**:
  | wintf | WinUI3/WPF | DOM |
  |-------|------------|-----|
  | Phase::Tunnel | PreviewXxx イベント | Capture Phase |
  | Phase::Bubble | Xxx イベント | Bubble Phase |
  | sender (引数) | OriginalSource | event.target |
  | entity (引数) | sender | event.currentTarget |
- **示唆**: ドキュメントコメントでこの対応関係を明記することで、WinUI3/WPF開発者の理解を促進

## アーキテクチャパターン評価

本機能は既存システムの拡張であり、新規パターン導入は不要。既存の**ECSイベントディスパッチパターン**を維持。

| 選択肢 | 説明 | 長所 | リスク/制約 | 備考 |
|--------|------|------|------------|------|
| 既存パターン維持 | `dispatch_pointer_events`システムとHandlerコンポーネントパターンをそのまま使用 | 実装済み、テスト済み、変更不要 | なし | 推奨 |
| 新規Tunnel専用システム | Tunnel専用の`dispatch_tunnel_events`を追加 | 明示的分離 | コード重複、保守負荷増加 | 不採用 |

**決定**: 既存パターンを維持し、`taffy_flex_demo`のハンドラ実装を拡張する方向で進める。

## 設計判断

### 判断: GreenBoxの子エンティティ構造

- **背景**: Requirement 3で「親がTunnelでキャプチャ→子に到達しない」を実証する必要
- **検討した代替案**:
  1. **Option A**: GreenBox直下に小さな矩形（GreenBoxChild）を配置
  2. **Option B**: RedBoxと同様に画像エンティティを配置
  3. **Option C**: 既存BlueBoxを子として移動
- **選択したアプローチ**: Option A
  - GreenBoxに`ChildOf(green_box)`を持つ小さな矩形（例: 50x50の黄色矩形）を追加
  - GreenBoxChildにも`OnPointerPressed`ハンドラを登録
  - GreenBoxのTunnelハンドラで左クリックを`true`で停止→GreenBoxChildに到達しない
- **根拠**:
  - シンプルで理解しやすい階層（2階層のみ）
  - 既存レイアウト（Flexbox）に影響を与えない
  - デモの意図が明確（親キャプチャのみに焦点）
- **トレードオフ**:
  - 利点: 実装が最小限、デモが明確
  - 欠点: 複雑な階層でのTunnel動作は別途検証が必要（本機能のスコープ外）
- **フォローアップ**: 実装時にGreenBoxのFlexレイアウトパラメータ（`flex_direction: Column`等）を確認

### 判断: ログ出力フォーマット

- **背景**: Requirement 1, 3, 4でログ出力による動作確認が要求されている
- **検討した代替案**:
  1. **Option A**: `println!("[Tunnel] Entity: {:?}", entity)`形式
  2. **Option B**: `tracing::info!`マクロで構造化ログ
  3. **Option C**: カスタムログマクロ作成
- **選択したアプローチ**: Option B（既存の`tracing::info!`使用）
  - フォーマット: `info!("[Tunnel] GreenBox: ...", sender = ?sender, entity = ?entity, ...)`
  - `[Tunnel]`/`[Bubble]`プレフィックスでフェーズを明示
  - 構造化フィールド（sender, entity, 座標等）を追加
- **根拠**:
  - 既存の`taffy_flex_demo`が`tracing`を使用（一貫性）
  - 構造化ログにより後でフィルタリング可能
  - `RUST_LOG`環境変数で制御可能
- **トレードオフ**:
  - 利点: 既存パターンとの一貫性、フィルタリング可能
  - 欠点: `println!`より若干冗長
- **フォローアップ**: README更新で`RUST_LOG=info cargo run --example taffy_flex_demo`の実行例を追記

### 判断: Tunnel stopPropagation条件

- **背景**: Requirement 2, 3でTunnelフェーズでの伝播停止をデモする必要
- **検討した代替案**:
  1. **Option A**: Ctrl+クリックでContainer level停止
  2. **Option B**: 左クリックでGreenBox level停止
  3. **Option C**: 右クリックで条件分岐
- **選択したアプローチ**: Option A（Container）+ Option B（GreenBox）の組み合わせ
  - **FlexDemoContainer**: Tunnelで`state.ctrl_down && state.left_down`なら停止
  - **GreenBox**: Tunnelで`state.left_down`（条件なし）なら停止
  - 両方とも色変更で視覚フィードバック
- **根拠**:
  - 2つの異なるユースケースをデモ
  - ContainerはCtrl修飾キーの例（条件付きキャプチャ）
  - GreenBoxは無条件キャプチャの例（親優先処理）
- **トレードオフ**:
  - 利点: 多様なユースケースを実証
  - 欠点: 操作が若干複雑
- **フォローアップ**: 実装時に`println!`でユーザーガイドを出力

## リスクと軽減策

- **Risk 1**: GreenBoxに子を追加するとFlexレイアウトが崩れる可能性
  - **軽減策**: GreenBoxを`flex_direction: Column`に変更し、子を縦並びに配置
- **Risk 2**: ログが多すぎてコンソールが読みにくい
  - **軽減策**: `OnPointerMoved`のログは頻度を制限（30フレームに1回等）、または別スレッドに分離
- **Risk 3**: Tunnelフェーズの理解が難しい
  - **軽減策**: ファイル冒頭に詳細なコメント、WinUI3/WPF対応表、出力例を追加

## 参考文献

- [WinUI3 Routed Events Overview](https://learn.microsoft.com/en-us/windows/apps/design/controls/routing-events-overview) — WinUI3のイベントルーティングモデル
- [DOM Events - Event Flow](https://www.w3.org/TR/DOM-Level-3-Events/#event-flow) — W3C DOMイベントフロー仕様
- `crates/wintf/src/ecs/pointer/dispatch.rs` — 既存実装ソースコード
- `crates/wintf/examples/taffy_flex_demo.rs` — 既存サンプルコード
