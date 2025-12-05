# Research & Design Decisions: event-hit-test-alpha-mask

---
**Feature**: event-hit-test-alpha-mask
**Discovery Scope**: Extension（親仕様 event-hit-test の拡張）
**Date**: 2025-12-05
---

## Summary

- **Feature**: αマスクによるピクセル単位ヒットテスト
- **Discovery Scope**: Extension（既存ヒットテストシステムの拡張）
- **Key Findings**:
  1. WIC `CopyPixels` APIラッパーが未実装（軽微な追加で対応可能）
  2. 既存パターン（`WintfTaskPool`、ECSクエリ）が完全に再利用可能
  3. `BitmapSourceResource` への `Option<AlphaMask>` 追加が最適な配置

## Research Log

### WIC CopyPixels API

- **Context**: αマスク生成にはピクセルデータ取得が必要
- **Sources Consulted**: 
  - Windows SDK Documentation (IWICBitmapSource::CopyPixels)
  - 既存コード `crates/wintf/src/com/wic.rs`
- **Findings**:
  - `IWICBitmapSource::CopyPixels(prc, stride, buffer)` で矩形領域のピクセルを取得
  - PBGRA32形式で4バイト/ピクセル、Aチャネルはオフセット+3
  - `prc = NULL` で全画像を取得
  - stride = width * 4（PBGRA32の場合）
- **Implications**: `WICBitmapSourceExt` トレイトに `copy_pixels()` と `get_size()` を追加

### ビットパック形式の設計

- **Context**: メモリ効率と判定速度のトレードオフ
- **Sources Consulted**: 一般的なビットマップ処理パターン
- **Findings**:
  - 1ビット/ピクセル形式が最もメモリ効率が良い
  - 行ごとに8ピクセル単位でアラインメント（行幅 = (width + 7) / 8）
  - MSBファースト（ビット7が最左ピクセル）
  - 判定は `(byte >> bit_index) & 1` で O(1)
- **Implications**: 1000x1000画像で約125KB、元の4MBから97%削減

### 非同期生成パターン

- **Context**: 画像読み込みと同様の非同期処理が必要
- **Sources Consulted**: `crates/wintf/src/ecs/widget/bitmap_source/systems.rs`
- **Findings**:
  - `WintfTaskPool.spawn()` でバックグラウンド実行
  - `CommandSender` でECSへコマンド送信
  - `drain_and_apply()` でInputスケジュールで適用
- **Implications**: 既存パターンをそのまま流用、新規設計不要

### αマスク生成トリガー検出

- **Context**: `HitTest::alpha_mask()` 設定時のみ生成したい
- **Sources Consulted**: bevy_ecs クエリパターン
- **Findings**:
  - `Added<BitmapSourceResource>` で新規追加を検出
  - `With<HitTest>` で対象を絞り込み
  - システム内で `HitTestMode::AlphaMask` をチェック
- **Implications**: ECS標準パターンで実現可能

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: 既存拡張のみ | 全て既存ファイルに追加 | ファイル数増加なし | resource.rs肥大化 | シンプルだが保守性低下 |
| B: 完全新規 | alpha_mask.rs + hit_test_alpha.rs | 責務分離明確 | ファイル数増加 | 過度な分離 |
| **C: ハイブリッド** | alpha_mask.rs新規 + 既存拡張 | バランス良好 | やや複雑 | **採用** |

## Design Decisions

### Decision: AlphaMask配置モジュール

- **Context**: AlphaMaskをどこに配置するか
- **Alternatives Considered**:
  1. `ecs::layout` に独立コンポーネントとして配置
  2. `ecs::widget::bitmap_source` に `BitmapSourceResource` のフィールドとして配置
- **Selected Approach**: Option 2 - `BitmapSourceResource.alpha_mask: Option<AlphaMask>`
- **Rationale**: 
  - αマスクはWICピクセルデータから生成されるためWICリソースと強く関連
  - ライフサイクルが`BitmapSourceResource`と一致（同時解放）
  - 独立コンポーネントだとエンティティ削除時の整合性管理が複雑
- **Trade-offs**: 
  - ✅ WICリソースと同じライフサイクル
  - ✅ 取得が容易（`BitmapSourceResource`経由）
  - ❌ `ecs::layout` からの参照が間接的
- **Follow-up**: `BitmapSourceResource.alpha_mask()` アクセサを提供

### Decision: 固定閾値128

- **Context**: α値の2値化閾値をどう設定するか
- **Alternatives Considered**:
  1. カスタマイズ可能な閾値（`AlphaMask::new(threshold)`）
  2. 固定値128（50%）
- **Selected Approach**: Option 2 - 固定値128
- **Rationale**: 
  - セキュリティ要件: 50%未満の透明領域でのクリック捕捉を防止
  - デスクトップマスコットで「ほぼ透明だがクリック可能」は意図しない動作
- **Trade-offs**:
  - ✅ セキュリティ確保
  - ✅ API簡素化
  - ❌ カスタマイズ不可
- **Follow-up**: 将来要件があれば別モード（`AlphaMaskCustom(u8)`）を検討

### Decision: 非同期生成中のフォールバック

- **Context**: αマスク生成完了前のヒット判定動作
- **Alternatives Considered**:
  1. `Bounds`（矩形判定）にフォールバック
  2. `None`（クリック透過）にフォールバック
- **Selected Approach**: Option 1 - `Bounds` フォールバック
- **Rationale**: 
  - UX優先: 起動直後からドラッグ操作可能
  - セキュリティより操作性を重視（生成は高速なため影響軽微）
- **Trade-offs**:
  - ✅ 即座にインタラクション可能
  - ❌ 一時的に透明部分がクリック可能
- **Follow-up**: 生成時間のベンチマーク（100ms以内目標）

### Decision: スケーリング対応方式

- **Context**: 表示サイズが元画像サイズと異なる場合の対応
- **Alternatives Considered**:
  1. αマスク再生成（表示サイズに合わせる）
  2. 座標変換（αマスクは元画像サイズのまま）
- **Selected Approach**: Option 2 - 座標変換
- **Rationale**: 
  - マスク再生成不要でメモリ・CPU効率が良い
  - 座標変換は単純な比率計算（O(1)）
- **Trade-offs**:
  - ✅ メモリ効率
  - ✅ 計算コスト最小
  - ❌ 拡大時に判定精度がやや低下（元画像解像度依存）
- **Follow-up**: 極端な拡大時の動作確認

### Decision: ビットパックのビットオーダー

- **Context**: αマスクのビットパック形式でMSBファースト or LSBファーストを選択
- **Alternatives Considered**:
  1. MSBファースト（ビット7が最左ピクセル）
  2. LSBファースト（ビット0が最左ピクセル）
- **Selected Approach**: Option 1 - MSBファースト
- **Rationale**: 
  - BMP/PNG等の画像フォーマットで一般的な慣例
  - ビジュアル的に左→右の順序が自然でデバッグしやすい
  - WIC 1bppへの変換は検討したが、COM呼び出しオーバーヘッドと閾値制御不可のため不採用
- **Trade-offs**:
  - ✅ 画像フォーマットの慣例に準拠
  - ✅ デバッグ時のバイト内容が直感的
  - ❌ 計算式に `7 -` が必要（軽微）
- **Follow-up**: なし

### Decision: WICBitmapSourceExt の配置

- **Context**: `IWICBitmapSource` の `GetSize` / `CopyPixels` ラッパーをどこに配置するか
- **Alternatives Considered**:
  1. `com/wic.rs` に追加（既存パターンに従う）
  2. 独立ファイル `com/wic_ext.rs` 
  3. `ecs::widget::bitmap_source` 内にプライベート実装
- **Selected Approach**: Option 1 - `com/wic.rs` に追加
- **Rationale**: 
  - `IWICBitmapSource` はWICのCOMインターフェース
  - `GetSize()` / `CopyPixels()` は unsafe COM呼び出し
  - 既存パターン（`WICImagingFactoryExt`, `WICBitmapDecoderExt`, `WICFormatConverterExt`）と同じ設計
  - `com/` 階層はCOMクラスのunsafe呼び出しをラップする場所
- **Trade-offs**:
  - ✅ 既存設計パターンとの一貫性
  - ✅ 他機能からも再利用可能
  - ✅ unsafe呼び出しが `com/` に集約
- **Follow-up**: なし

### Decision: generate_alpha_mask_system のスケジュールと検知

- **Context**: αマスク生成システムの実行タイミングと変更検知方法
- **Alternatives Considered**:
  1. `Added<BitmapSourceResource>` のみで検知
  2. `Or<(Changed<BitmapSourceResource>, Changed<HitTest>)>` で検知
- **Selected Approach**: Option 2 - `Changed` ベースの検知
- **Rationale**: 
  - `BitmapSourceResource` 追加時は常に `alpha_mask: None`（`new()` で初期化）
  - `Changed<HitTest>` で動的なモード変更にも対応
  - 生成済みチェック（`alpha_mask.is_some()`）で無駄な再生成を防止
- **Implementation Details**:
  - スケジュール: `WidgetGraphics`（既存BitmapSourceシステムと同じ）
  - 順序: `draw_bitmap_sources` の直後（`.after(draw_bitmap_sources)`）
  - スキップ条件: `HitTestMode != AlphaMask` または `alpha_mask.is_some()`
- **Trade-offs**:
  - ✅ `draw_bitmap_sources` 直後で依存関係が明確
  - ✅ 動的なHitTest変更にも対応
  - ✅ 既存パターン（Changed検知 + スキップ条件）に準拠
- **Follow-up**: なし

### Decision: 座標変換の丸め処理

- **Context**: スクリーン座標→マスク座標変換時の丸め方法
- **Alternatives Considered**:
  1. 切り捨て（`as u32`）
  2. 四捨五入（`(x + 0.5) as u32` または `.round() as u32`）
  3. floor明示（`.floor() as u32`）
- **Selected Approach**: Option 2 - 四捨五入
- **Rationale**: 
  - 拡大時の境界ピクセルで正確な判定が可能
  - 気にするユーザーへの配慮
  - 計算コストの増加は軽微（`+ 0.5` のみ）
- **Implementation**:
  ```rust
  let mask_x = (rel_x * mask.width() as f32 + 0.5) as u32;
  let mask_y = (rel_y * mask.height() as f32 + 0.5) as u32;
  ```
- **Trade-offs**:
  - ✅ より正確な座標変換
  - ❌ わずかな計算コスト増（無視可能）
- **Follow-up**: なし

### Decision: エラー時のログレベル

- **Context**: WIC CopyPixels 失敗等のエラー時のログレベルとシステム動作
- **Alternatives Considered**:
  1. `warn` レベル
  2. `error` レベル
  3. `trace` / `debug` レベル
- **Selected Approach**: シナリオ別にログレベルを設定
- **Rationale**: 
  - WIC CopyPixels失敗は通常起こらない異常事態 → `error`
  - ただしECSシステムは停止できないためフォールバック継続
  - エンティティ削除済みは正常なレースコンディション → `debug`
  - 範囲外座標は正常動作 → ログなし
- **Implementation**:
  | シナリオ | ログレベル | 動作 |
  |----------|-----------|------|
  | WIC CopyPixels失敗 | `error` | Boundsフォールバック |
  | エンティティ削除済み | `debug` | 処理スキップ |
  | 範囲外座標 | なし | `is_hit()` が `false` |
- **Trade-offs**:
  - ✅ 異常事態は `error` で可視化
  - ✅ システムは継続動作（フォールバック）
  - ✅ 正常なレースコンディションはノイズにならない
- **Follow-up**: なし

## Risks & Mitigations

| Risk | Mitigation |
|------|------------|
| WIC CopyPixels失敗 | 失敗時はαマスク生成をスキップし、Boundsフォールバック |
| 大画像での生成遅延 | 非同期実行 + Boundsフォールバックで応答性確保 |
| メモリ圧迫（大量画像） | ビットパックで97%削減、必要な画像のみ生成 |

## References

- [IWICBitmapSource::CopyPixels](https://docs.microsoft.com/en-us/windows/win32/api/wincodec/nf-wincodec-iwicbitmapsource-copypixels) - Windows SDK
- 親仕様設計: `.kiro/specs/completed/event-hit-test/design.md`
- ギャップ分析: `.kiro/specs/event-hit-test-alpha-mask/gap-analysis.md`
