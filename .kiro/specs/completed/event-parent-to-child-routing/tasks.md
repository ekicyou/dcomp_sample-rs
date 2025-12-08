# 実装タスク

## タスク概要

本実装は、`taffy_flex_demo`サンプルアプリケーションにTunnelフェーズ（親→子イベント伝播）のデモンストレーションを追加する。既存の`dispatch_pointer_events`システムは変更せず、サンプルコードのハンドラ実装とエンティティ構造のみを拡張する。

**スコープ**: 
- ファイル: `crates/wintf/examples/taffy_flex_demo.rs`のみ
- コアシステム（`dispatch.rs`）は変更不要
- 全5つの要件をカバーする8つのメジャータスク

---

## 実装タスクリスト

### 1. GreenBoxChildエンティティの追加

- [ ] 1.1 (P) GreenBoxの子エンティティとしてGreenBoxChildを作成
  - GreenBoxに子として小さな矩形エンティティ（50x50推奨）を追加
  - 黄色の初期色を設定（`D2D1_COLOR_F { r: 1.0, g: 1.0, b: 0.0, a: 1.0 }`）
  - `ChildOf(green_box)`コンポーネントで親子関係を確立
  - `GreenBoxChild`マーカーコンポーネントを追加
  - `Opacity(0.5)`で半透明表示
  - _Requirements: 3.1_
  - _Contracts: GreenBoxChild State Management (design.md)_

- [ ] 1.2 (P) GreenBoxのレイアウトをColumn方向に変更
  - GreenBoxの`BoxStyle`を`flex_direction: Column`に変更
  - 子エンティティが縦並びに配置されることを確認
  - 既存のFlexboxレイアウトが崩れないことを検証
  - _Requirements: 3.1_

### 2. GreenBoxChildのイベントハンドラ実装

- [ ] 2.1 (P) GreenBoxChild用のポインター押下ハンドラを実装
  - `on_green_child_pressed`関数を作成
  - Tunnelフェーズ: 「[Tunnel] GreenBoxChild: This should NOT be called if parent captured」ログを出力し、`false`を返す
  - Bubbleフェーズ: 右クリック時に色をオレンジ（`r: 1.0, g: 0.5, b: 0.0`）に変更し、ログ出力
  - `ev.value()`で`PointerState`にアクセスする例を実装
  - _Requirements: 3.6, 3.7, 4.3_
  - _Contracts: on_green_child_pressed Service Interface (design.md)_

- [ ] 2.2 (P) GreenBoxChildにOnPointerPressedハンドラを登録
  - GreenBoxChildエンティティ生成時に`OnPointerPressed(on_green_child_pressed)`コンポーネントを追加
  - ハンドラが正しく登録されることを確認
  - _Requirements: 3.2_

### 3. GreenBoxのTunnelキャプチャハンドラ実装

- [ ] 3.1 (P) GreenBox用のポインター押下ハンドラを実装
  - `on_green_box_pressed`関数を作成
  - Tunnelフェーズ: 左クリック時に`true`を返して伝播停止、色を黄緑（`r: 0.5, g: 1.0, b: 0.0`）に変更
  - Tunnelフェーズ: 「[Tunnel] GreenBox: Captured event, stopping propagation」ログを出力
  - Bubbleフェーズ: 右クリック時の処理を実装
  - `sender`と`entity`の違いをログで明示
  - _Requirements: 3.3, 3.4, 4.1, 4.2, 4.4, 4.5_
  - _Contracts: on_green_box_pressed Service Interface (design.md)_

- [ ] 3.2 (P) GreenBoxにOnPointerPressedハンドラを登録
  - GreenBoxエンティティ生成時に`OnPointerPressed(on_green_box_pressed)`コンポーネントを追加
  - 既存の`OnPointerMoved`ハンドラと共存することを確認
  - _Requirements: 3.2_

### 4. FlexDemoContainerのTunnelフェーズ拡張

- [ ] 4.1 FlexDemoContainerハンドラのTunnel対応
  - 既存の`on_container_pressed`関数を修正
  - 先頭の`if !ev.is_bubble() { return false; }`を削除
  - Tunnelフェーズ: Ctrl+左クリック時に`true`を返して伝播停止、色をピンクに変更
  - Tunnelフェーズ: 「[Tunnel] FlexContainer: Event stopped at Container」ログを出力
  - Bubbleフェーズ: 既存の右クリック処理を維持
  - `match ev { Phase::Tunnel(state) => {...}, Phase::Bubble(state) => {...} }`構文を使用
  - _Requirements: 1.1, 2.1, 2.2, 2.3, 2.4, 2.5, 4.1, 4.2, 4.3, 4.4, 4.5_
  - _Contracts: on_container_pressed Service Interface (design.md)_

### 5. 既存ハンドラのログ出力拡張

- [ ] 5.1 (P) RedBox/BlueBoxハンドラのログ拡張
  - `on_red_box_pressed`と`on_blue_box_pressed`にTunnel/Bubbleログを追加
  - 「[Tunnel]」または「[Bubble]」プレフィックスを付与
  - `sender`, `entity`, `screen_point`, `local_point`をログに含める
  - 既存のBubbleのみ処理ロジックは維持（`if !ev.is_bubble() { return false; }`をそのまま）
  - _Requirements: 1.2, 1.3, 1.4, 1.5_

### 6. ドキュメントコメントの追加

- [ ] 6.1 (P) ファイル冒頭にTunnel/Bubbleフェーズの説明を追加
  - `taffy_flex_demo.rs`の先頭にTunnel/Bubbleフェーズの概念を説明するコメントを追加
  - WinUI3/WPF/DOMイベントモデルとの対応表をコメントで記述
  - 実行時の出力例を`println!`またはコメントで提示
  - _Requirements: 5.1, 5.2, 5.5_
  - _Reference: design.md「WinUI3/WPF/DOMイベントモデル対応表（詳細版）」_

- [ ] 6.2 (P) 各ハンドラ関数にdocコメントを追加
  - `on_green_box_pressed`, `on_green_child_pressed`, `on_container_pressed`にRustdocコメントを追加
  - Tunnel/Bubbleでの処理意図を説明
  - stopPropagationの使用例とユースケースを記述
  - `sender`と`entity`の違いを説明
  - _Requirements: 5.3, 5.4_

### 7. 動作確認とログ検証

- [ ] 7.1 Tunnelキャプチャの動作確認
  - GreenBoxChild（黄色矩形）を左クリック → GreenBoxのTunnelログのみ出力、GreenBoxChildのログは出ないことを確認
  - GreenBoxChild（黄色矩形）を右クリック → Tunnel/Bubble両フェーズで両エンティティのログが出力されることを確認
  - GreenBoxが適切に色変更されることを確認
  - _Requirements: 3.3, 3.4, 3.5, 3.6, 3.7_

- [ ] 7.2 Container Ctrl+クリックの動作確認
  - Ctrlキーを押しながらRedBoxを左クリック → Containerで伝播停止、RedBoxのログが出ないことを確認
  - 通常の左クリック → Tunnel/Bubble両フェーズのログが正常に出力されることを確認
  - Bubbleフェーズが実行されないことをログで確認
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5_

- [ ] 7.3 PointerState情報のログ検証
  - 各ハンドラのログに`sender`, `entity`, `screen_point`, `local_point`, ボタン状態, 修飾キー状態が含まれることを確認
  - Tunnel/Bubbleフェーズで同じPointerState値が参照されることを確認
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5_

### 8. 最終統合テスト

- [ ] 8.1 実行順序の全体確認
  - BlueBoxを左クリック → 「[Tunnel] Container → [Tunnel] BlueBox → [Bubble] BlueBox → [Bubble] Container」の順序でログ出力されることを確認
  - 実行順序がコンソールログから理解できることを確認
  - _Requirements: 1.5_

- [ ] 8.2 OnPointerMoved共存確認
  - GreenBoxChild上でマウス移動 → 「[Bubble] GreenBox: Pointer moved」ログが正常に出力されることを確認
  - OnPointerMovedとOnPointerPressedが共存しても問題ないことを確認
  - _Reference: design.md「on_green_box_moved」_

- [ ] 8.3 パフォーマンス検証
  - サンプル実行時に60 FPS維持されることを確認（既存の`taffy_flex_demo`と同等）
  - クリックから視覚フィードバックまで16ms以内を確認
  - _NFR: NFR-1 パフォーマンス_

---

## 要件カバレッジマトリクス

| 要件 | タスク |
|------|--------|
| 1.1 | 4.1 |
| 1.2 | 5.1 |
| 1.3 | 5.1 |
| 1.4 | 5.1 |
| 1.5 | 5.1, 8.1 |
| 2.1 | 4.1, 7.2 |
| 2.2 | 4.1, 7.2 |
| 2.3 | 4.1, 7.2 |
| 2.4 | 4.1, 7.2 |
| 2.5 | 4.1, 7.2 |
| 3.1 | 1.1, 1.2 |
| 3.2 | 2.2, 3.2 |
| 3.3 | 3.1, 7.1 |
| 3.4 | 3.1, 7.1 |
| 3.5 | 7.1 |
| 3.6 | 2.1, 7.1 |
| 3.7 | 2.1, 7.1 |
| 4.1 | 3.1, 4.1, 7.3 |
| 4.2 | 3.1, 4.1, 7.3 |
| 4.3 | 2.1, 4.1, 7.3 |
| 4.4 | 3.1, 4.1, 7.3 |
| 4.5 | 3.1, 4.1, 7.3 |
| 5.1 | 6.1 |
| 5.2 | 6.1 |
| 5.3 | 6.2 |
| 5.4 | 6.2 |
| 5.5 | 6.1 |

---

## 並列実行ガイド

以下のタスクは並列実行可能（`(P)`マーク付き）:
- **Group A**: 1.1, 1.2（GreenBoxChild作成とレイアウト変更）
- **Group B**: 2.1, 2.2（GreenBoxChildハンドラ実装と登録）
- **Group C**: 3.1, 3.2（GreenBoxハンドラ実装と登録）
- **Group D**: 5.1, 6.1, 6.2（既存ハンドラログ拡張とドキュメント追加）

**依存関係**:
- タスク4.1は既存ハンドラの修正のため、他タスクと競合しない
- タスク7.x, 8.xは全実装完了後に実行（検証フェーズ）

---

_Generated by AI-DLC System on 2025-12-08_
