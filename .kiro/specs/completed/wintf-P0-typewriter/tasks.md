````markdown
# Implementation Plan

## Task Format

| Major | Sub | Description | Status |
|-------|-----|-------------|--------|
| 1-3 | - | Track A/B 基盤層（並行可能） | ✅ |
| 4-6 | - | Track C Typewriter本体 | ✅ |

---

## Implementation Notes

### 設計変更: AnimationCore → FrameTime

当初設計では Windows Animation API (IUIAnimationTimer/Manager) を使用予定だったが、
STA (Single-Threaded Apartment) 要件により ECS マルチスレッドスケジューラと競合するため廃止。

**採用アプローチ**:
- `GetSystemTimePreciseAsFileTime` (Windows 8以降、100ns精度) を使用
- `FrameTime` リソースで経過時間を管理 (f64秒)
- Typewriter の再生制御は FrameTime ベースで実装

### 設計変更: TypewriterTalk 分離

**当初設計**: TypewriterTalk が論理情報と COM リソース（TextLayout）を保持

**採用設計**:
- `TypewriterTalk`: 論理情報のみ（トークン列、再生状態、進行度）
- `TypewriterLayoutCache`: COM リソース（TextLayout、Stage 2 IR タイムライン）
- 描画システムが TypewriterTalk 追加時に LayoutCache を自動生成

**メリット**:
- 関心の分離（論理 vs リソース）
- デモコードの簡素化（トークン列を渡すだけでOK）
- Arrangement 変更時にLayoutCache再生成でレイアウト追従

### 設計変更: BoxStyle に min_size/max_size 追加

レイアウト追従テストのため、BoxStyle に min_size/max_size フィールドを追加。
Taffy の min_size/max_size プロパティに変換される。

---

## Tasks

### Track A/B: 基盤層（並行実装可能）

- [x] 1. ~~(P) AnimationCore リソース実装~~ → FrameTime ベースに変更
- [x] 1.1 ~~(P) Windows Animation API COM オブジェクト初期化~~ → 不要（STA制約により廃止）
- [x] 1.2 (P) 時刻取得 API 実装
  - `GetSystemTimePreciseAsFileTime` で高精度時刻取得
  - `FrameTime` リソースで経過時間管理（f64秒）
  - _Requirements: 7.2, 7.3_

- [x] 1.3 ~~(P) EcsWorld への AnimationCore 統合~~ → FrameTime として統合済み
  - _Requirements: 7.1, 7.5_

- [x] 2. (P) DirectWrite クラスタ API 拡張
- [x] 2.1 (P) DWriteTextLayoutExt トレイト定義と実装
  - `get_cluster_metrics()` でクラスタメトリクス取得
  - 縦書き/横書き両対応
  - _Requirements: 1.3, 3.5, 3.6, 3.8_

- [x] 3. ~~(P) animation_tick_system 実装~~ → FrameTime 更新は EcsWorld で実施
- [x] 3.1 時刻更新の実装
  - FrameTime リソースの更新
  - _Requirements: 7.2, 7.3_

- [x] 4. (P) Stage 1 IR 型定義（TypewriterToken）
- [x] 4.1 (P) TypewriterToken enum 定義
  - Text(String) - 表示テキスト
  - Wait(f64) - ウェイト（秒単位）
  - FireEvent { target, event } - イベント発火
  - _Requirements: 3.1, 3.2, 3.3, 3.4_

- [x] 4.2 (P) TypewriterEvent enum Component 定義
  - None（デフォルト）/ Complete / Paused / Resumed
  - SparseSet ストレージ戦略
  - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

- [x] 5. (P) Stage 2 IR 型定義（TimelineItem）
- [x] 5.1 (P) TimelineItem enum と TypewriterTimeline 構造体定義
  - Glyph { cluster_index, show_at } - グリフ表示
  - Wait { duration, start_at } - ウェイト
  - FireEvent { target, event, fire_at } - イベント発火
  - _Requirements: 3.5, 3.6, 3.7, 3.8_

### Track C: Typewriter 本体（Track A/B 完了後）

- [x] 6. Typewriter コンポーネント（永続ウィジェット論理）
- [x] 6.1 Typewriter 構造体定義
  - スタイル設定（font_family, font_size, color, direction）
  - デフォルト文字間ウェイト設定（default_char_wait）
  - SparseSet ストレージ戦略、on_add/on_remove フック
  - _Requirements: 6.1, 6.2, 6.3, 6.4, 6.5_

- [x] 7. TypewriterTalk / TypewriterLayoutCache コンポーネント
- [x] 7.1 TypewriterTalk（論理情報）
  - Stage 1 IR トークン列を保持
  - 再生状態（Playing/Paused/Completed）管理
  - visible_cluster_count, progress 計算
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6, 5.6_

- [x] 7.2 TypewriterLayoutCache（COM リソース）
  - TextLayout と Stage 2 IR タイムライン保持
  - init_typewriter_layout システムで自動生成
  - Arrangement 変更時に無効化・再生成
  - _Requirements: 3.5, 3.6, 3.7, 3.8, 1.3_

- [x] 7.3 TypewriterTalk 操作 API 実装
  - `new()` - トークン列と開始時刻から生成
  - `pause()` / `resume()` - 一時停止・再開
  - `skip()` - 全文即時表示
  - on_remove フックでログ出力
  - _Requirements: 4.1, 4.2, 4.3, 4.4, 4.5, 4.6_

- [x] 8. Typewriter システム群
- [x] 8.1 init_typewriter_layout システム（Draw スケジュール）
  - TypewriterTalk 追加時に LayoutCache 自動生成
  - Typewriter.direction に応じて縦書き/横書き設定
  - Arrangement.size から TextLayout サイズ取得
  - _Requirements: 1.1, 1.2, 6.3_

- [x] 8.2 invalidate_typewriter_layout_on_arrangement_change システム
  - Arrangement 変更検知で LayoutCache 削除
  - レイアウトボックス変動への追従
  - _Requirements: 6.3_

- [x] 8.3 update_typewriters システム（Update スケジュール）
  - FrameTime から現在時刻取得
  - タイムラインを走査し visible_cluster_count 更新
  - FireEvent トークンを処理
  - 全クラスタ表示完了で Completed に遷移
  - _Requirements: 1.1, 1.2, 2.3, 5.2, 5.3, 5.4_

- [x] 8.4 draw_typewriters システム（Draw スケジュール）
  - TypewriterLayoutCache から TextLayout 取得
  - visible_cluster_count までのグリフを描画
  - 非表示部分は透明ブラシで SetDrawingEffect
  - _Requirements: 1.1, 1.2, 6.3_

- [x] 9. 統合と動作確認
- [x] 9.1 Typewriter ウィジェットシステム統合
  - widget/text モジュールに配置
  - EcsWorld にシステム登録
  - Label との共存確認
  - _Requirements: 7.1, 7.4, 7.5_

- [x] 9.2 サンプルアプリケーション
  - `examples/typewriter_demo.rs` で動作確認
  - 横書き・縦書き両方のデモ
  - pause/resume/skip 操作の動作確認
  - FireEvent による完了イベント受信確認
  - _Requirements: 1.1, 1.2, 1.4, 1.5, 2.1, 2.2, 2.3, 2.4, 2.5, 4.1, 4.2, 4.3, 4.4, 4.5, 5.1, 5.2, 5.3, 5.4, 5.5_

- [x]* 9.3 ユニットテスト整備
  - Stage 1/2 IR 型テスト (typewriter_ir::tests)
  - TypewriterState テスト (typewriter::tests)
  - COM依存テストは統合テストで実施
  - _Requirements: 3.1, 3.5, 4.1, 4.2, 4.3, 4.4, 4.5, 7.1, 7.2, 7.3_

---

## 🚨 ~~未解決の問題~~ → ✅ 解決済み (2025-12-04)

### 問題の症状
- ~~横書き・縦書きともにテキストが表示されない~~
- ~~灰色の背景ボックスは正常に表示される~~
- ~~ウィンドウリサイズによるレイアウト追従は動作している~~

**解決**: Visual ライフサイクル統合、縦書きorigin計算修正、flex_grow によるレイアウト修正で解決。
詳細は `validation-impl.md` を参照。

---

## Requirements Coverage Matrix

| Requirement | Tasks |
|-------------|-------|
| 1.1-1.5 | 7.1, 8.3, 8.4, 9.2 |
| 2.1-2.5 | 7.1, 9.2 |
| 3.1-3.4 | 4.1, 4.2 |
| 3.5-3.8 | 2.1, 5.1, 7.2 |
| 4.1-4.6 | 7.1, 7.3, 9.2 |
| 5.1-5.6 | 4.2, 7.1, 8.3, 9.2 |
| 6.1-6.5 | 6.1, 8.1, 8.2, 8.4 |
| 7.1-7.5 | 1.2, 3.1, 9.1, 9.3 |

---

_Document updated on 2025-12-03 (MVP Complete)_

````
