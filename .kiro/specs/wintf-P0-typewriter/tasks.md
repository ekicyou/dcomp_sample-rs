````markdown
# Implementation Plan

## Task Format

| Major | Sub | Description | Status |
|-------|-----|-------------|--------|
| 1-3 | - | Track A/B 基盤層（並行可能） | ⏳ |
| 4-6 | - | Track C Typewriter本体 | ⏳ |

---

## Tasks

### Track A/B: 基盤層（並行実装可能）

- [ ] 1. (P) AnimationCore リソース実装
- [ ] 1.1 (P) Windows Animation API COM オブジェクト初期化
  - IUIAnimationTimer インスタンス作成
  - IUIAnimationManager2 インスタンス作成
  - IUIAnimationTransitionLibrary2 インスタンス作成
  - スレッドセーフ（Send + Sync）のため thread-free marshaling 確認
  - _Requirements: 7.1, 7.2_

- [ ] 1.2 (P) AnimationCore 時刻取得・更新 API 実装
  - `get_time()` で現在時刻（f64秒）取得
  - `tick()` でタイマー更新とマネージャー状態更新
  - マネージャー・トランジションライブラリへの参照取得
  - エラー時は警告ログ出力、graceful degradation
  - _Requirements: 7.2, 7.3_

- [ ] 1.3 (P) EcsWorld への AnimationCore 統合
  - `EcsWorld::new()` で AnimationCore 初期化・登録
  - WicCore パターンに準拠（CPU リソース即時初期化）
  - COM 初期化失敗時のフォールバック（ログ出力、リソース未登録）
  - _Requirements: 7.1, 7.5_

- [ ] 2. (P) DirectWrite クラスタ API 拡張
- [ ] 2.1 (P) DWriteTextLayoutExt トレイト定義と実装
  - `get_cluster_metrics()` でクラスタメトリクス取得
  - `get_cluster_count()` でクラスタ数取得
  - `hit_test_text_position()` でテキスト位置から描画座標取得
  - 縦書き/横書き両対応
  - _Requirements: 1.3, 3.5, 3.6, 3.8_

- [ ] 3. (P) animation_tick_system 実装
- [ ] 3.1 (P) システム関数の定義とスケジュール登録
  - Input スケジュール先頭に登録（他システムより先に時刻確定）
  - AnimationCore.tick() を毎フレーム呼び出し
  - Option<Res<AnimationCore>> で未初期化時のスキップ対応
  - 他 Input システムは `.after(animation_tick_system)` で順序保証
  - _Requirements: 7.2, 7.3_

- [ ] 4. (P) Stage 1 IR 型定義（TypewriterToken）
- [ ] 4.1 (P) TypewriterToken enum 定義
  - Text(String) - 表示テキスト
  - Wait(f64) - ウェイト（秒単位）
  - FireEvent { target, event } - イベント発火
  - areka-P0-script-engine との共有を考慮したモジュール配置
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [ ] 4.2 (P) TypewriterEvent enum Component 定義
  - None（デフォルト）/ Complete / Paused / Resumed
  - SparseSet ストレージ戦略
  - Default derive で None をデフォルト値に
  - Changed<TypewriterEvent> で検出、処理後に None へ戻す set パターン
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

- [ ] 5. (P) Stage 2 IR 型定義（TimelineItem）
- [ ] 5.1 (P) TimelineItem enum と TypewriterTimeline 構造体定義
  - Glyph { cluster_index, show_at } - グリフ表示
  - Wait { duration, start_at } - ウェイト
  - FireEvent { target, event, fire_at } - イベント発火
  - TypewriterTimeline で全文テキスト・タイムライン項目・総再生時間を保持
  - _Requirements: 3.5, 3.6, 3.7, 3.8_

### Track C: Typewriter 本体（Track A/B 完了後）

- [ ] 6. Typewriter コンポーネント（永続ウィジェット論理）
- [ ] 6.1 Typewriter 構造体定義
  - スタイル設定（font_family, font_size, color, direction）で Label 互換
  - デフォルト文字間ウェイト設定（default_char_wait）
  - SparseSet ストレージ戦略、on_add/on_remove フック
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

- [ ] 7. TypewriterTalk コンポーネント（1回のトーク）
- [ ] 7.1 Stage 1 → Stage 2 IR 変換ロジック
  - TextToken を DirectWrite でグリフ単位に分解
  - デフォルトウェイトを適用し累積時刻を計算
  - Wait/FireEvent トークンをタイムラインに追加
  - DWriteTextLayoutExt を使用してクラスタ情報取得
  - _Requirements: 3.5, 3.6, 3.7, 3.8, 1.3_

- [ ] 7.2 TypewriterTalk 構造体と再生状態管理
  - TextLayout と TypewriterTimeline を保持
  - Playing/Paused/Completed 状態遷移
  - 再生開始時刻、一時停止時経過時間の管理
  - visible_cluster_count と progress の計算
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 5.6_

- [ ] 7.3 TypewriterTalk 操作 API 実装
  - `new()` - Stage 1 IR から TypewriterTalk 生成
  - `pause()` / `resume()` - 一時停止・再開
  - `skip()` - 全文即時表示
  - トーク完了・クリア時は remove で解放
  - on_remove フックでリソース解放ログ
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_

- [ ] 8. Typewriter 更新・描画システム
- [ ] 8.1 Typewriter 更新システム（Update スケジュール）
  - AnimationCore から現在時刻取得
  - タイムラインを走査し visible_cluster_count 更新
  - FireEvent トークンを処理（対象エンティティの TypewriterEvent を設定）
  - 全クラスタ表示完了で状態を Completed に遷移
  - _Requirements: 1.1, 1.2, 2.3, 5.2, 5.3, 5.4_

- [ ] 8.2 draw_typewriters システム（Draw スケジュール）
  - Changed<TypewriterTalk> でクエリフィルタリング
  - TextLayout から visible_cluster_count までのグリフを描画
  - GraphicsCommandList に描画コマンド記録
  - Typewriter スタイル（color 等）を適用
  - _Requirements: 1.1, 1.2, 6.3_

- [ ] 9. 統合と動作確認
- [ ] 9.1 Typewriter ウィジェットシステム統合
  - widget/text モジュールに Typewriter 関連を配置
  - EcsWorld にシステム登録（Update: 更新、Draw: 描画）
  - Label との共存確認
  - _Requirements: 7.1, 7.4, 7.5_

- [ ] 9.2 サンプルアプリケーションで動作確認
  - Stage 1 IR を使用したタイプライター表示
  - ウェイト制御の視覚確認
  - pause/resume/skip 操作の動作確認
  - FireEvent による完了イベント受信確認
  - _Requirements: 1.1, 1.2, 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 2.5, 4.1, 4.2, 4.3, 4.4, 4.5, 5.1, 5.2, 5.3, 5.4, 5.5_

- [ ]* 9.3 ユニットテスト整備
  - AnimationCore 初期化・時刻取得テスト
  - DWriteTextLayoutExt クラスタメトリクス取得テスト
  - Stage 1 → Stage 2 IR 変換テスト
  - TypewriterTalk 状態遷移テスト
  - _Requirements: 1.1, 1.3, 2.1, 2.3, 3.1, 3.5, 4.1, 4.2, 4.3, 4.4, 4.5, 7.1, 7.2, 7.3_

---

## Requirements Coverage Matrix

| Requirement | Tasks |
|-------------|-------|
| 1.1-1.5 | 7.1, 8.1, 8.2, 9.2 |
| 2.1-2.5 | 7.2, 9.2 |
| 3.1-3.4 | 4.1, 4.2 |
| 3.5-3.8 | 2.1, 5.1, 7.1 |
| 4.1-4.6 | 7.2, 7.3, 9.2 |
| 5.1-5.6 | 4.2, 7.2, 8.1, 9.2 |
| 6.1-6.5 | 6.1, 8.2 |
| 7.1-7.5 | 1.1, 1.2, 1.3, 3.1, 9.1, 9.3 |

---

## Parallel Execution Summary

**Track A/B（並行可能）**:
- Task 1 (AnimationCore) — 独立実装
- Task 2 (DirectWrite拡張) — 独立実装
- Task 3 (animation_tick_system) — Task 1 に軽微依存だが型定義後即実装可
- Task 4 (Stage 1 IR) — 独立実装
- Task 5 (Stage 2 IR) — 独立実装

**Track C（シーケンシャル）**:
- Task 6 → Task 7 → Task 8 → Task 9

---

_Document generated by AI-DLC System on 2025-12-03_

````
