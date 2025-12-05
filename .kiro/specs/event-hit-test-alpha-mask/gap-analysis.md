# Gap Analysis: event-hit-test-alpha-mask

| 項目 | 内容 |
|------|------|
| **Document Title** | event-hit-test-alpha-mask ギャップ分析 |
| **Version** | 1.0 |
| **Date** | 2025-12-05 |
| **Parent Spec** | event-hit-test |
| **Requirements** | event-hit-test-alpha-mask/requirements.md v1.0 (Draft) |

---

## 1. 分析サマリー

- **スコープ**: αマスクによるピクセル単位ヒットテストを `event-hit-test` の拡張として実装
- **既存資産**: `hit_test.rs`、`bitmap_source/` モジュール、`WintfTaskPool` が再利用可能
- **主要ギャップ**: WICピクセルデータ取得API（`CopyPixels`）のラッパーが未実装
- **推奨アプローチ**: Option B（新規 `AlphaMask` 構造体）+ Option A（既存モジュール拡張）のハイブリッド
- **工数/リスク**: **M（3-7日）/ Low** - 既存パターン活用、WIC API追加のみ

---

## 2. 現状調査

### 2.1 ドメイン関連アセット

| ファイル/モジュール | 役割 | 再利用度 |
|-------------------|------|---------|
| `ecs/layout/hit_test.rs` | HitTest/HitTestMode、hit_test関数群 | 高（拡張対象） |
| `ecs/widget/bitmap_source/resource.rs` | BitmapSourceResource（WIC Source保持） | 高（AlphaMask追加先） |
| `ecs/widget/bitmap_source/wic_core.rs` | WicCore（WICファクトリ保持） | 高（ピクセル取得で使用） |
| `ecs/widget/bitmap_source/systems.rs` | 非同期画像読み込み、PBGRA32変換 | 高（パターン流用） |
| `ecs/widget/bitmap_source/task_pool.rs` | WintfTaskPool（非同期コマンド実行） | 高（αマスク生成で使用） |
| `com/wic.rs` | WICラッパー拡張trait | 中（CopyPixels追加必要） |

### 2.2 アーキテクチャパターン

**既存パターン**:
- **Component + Resource分離**: `BitmapSource`（マーカー）+ `BitmapSourceResource`（WIC）+ `BitmapSourceGraphics`（D2D）
- **非同期Command**: `WintfTaskPool.spawn()` → `CommandSender` → `drain_and_apply()`
- **Device Lost対応**: `invalidate()` + `is_valid()` パターン（GPUリソースのみ）

**適用方針**:
- `AlphaMask` はCPUリソースのためDevice Lost対応不要
- `BitmapSourceResource.alpha_mask: Option<AlphaMask>` として保持（議題4決定済み）
- 既存の `WintfTaskPool` パターンで非同期生成

### 2.3 統合サーフェス

| 統合ポイント | 既存インターフェース | 必要変更 |
|-------------|-------------------|---------|
| `HitTestMode` | `None`, `Bounds` | `AlphaMask` バリアント追加 |
| `HitTest` | `none()`, `bounds()` | `alpha_mask()` メソッド追加 |
| `hit_test_entity()` | `Bounds` 分岐のみ | `AlphaMask` 分岐追加 |
| `BitmapSourceResource` | `source: IWICBitmapSource` | `alpha_mask: Option<AlphaMask>` 追加 |

---

## 3. 要件実現可能性分析

### 3.1 Requirement-to-Asset マッピング

| Requirement | 必要資産 | ギャップ状態 |
|-------------|---------|-------------|
| R1: HitTestMode拡張 | `hit_test.rs` | ✅ 拡張容易 |
| R2: AlphaMaskデータ構造 | なし（新規） | 🆕 新規作成 |
| R3: BitmapSource統合 | `bitmap_source/`、`WintfTaskPool` | ⚠️ CopyPixels未実装 |
| R4: ピクセル単位判定 | `hit_test.rs` | ✅ 拡張容易 |
| R5: hit_test関数拡張 | `hit_test.rs` | ✅ 拡張容易 |
| R6: HitTest API | `hit_test.rs` | ✅ 拡張容易 |

### 3.2 技術的ギャップ

#### Gap 1: WIC CopyPixels API未実装

**状況**: `IWICBitmapSource::CopyPixels()` のラッパーが `com/wic.rs` に存在しない

**影響**: αマスク生成時にピクセルデータを取得できない

**対策案**:
- A) `WICBitmapSourceExt` トレイトを追加（`get_size()`, `copy_pixels()`）
- B) `bitmap_source/` 内にローカル関数として実装

**推奨**: Option A - `com/wic.rs` に追加（他機能でも再利用可能）

#### Gap 2: αマスク生成トリガー検出

**状況**: `BitmapSourceResource` 挿入時に `HitTest::alpha_mask()` を検出する仕組みがない

**影響**: αマスクを自動生成するタイミングが不明確

**対策案**:
- A) `Added<BitmapSourceResource>` + `With<HitTest>` クエリでシステム検出
- B) `on_add` フックで直接トリガー
- C) 専用 `NeedsAlphaMask` マーカーコンポーネント

**推奨**: Option A - ECS標準パターン（bevy_ecs `Added` + `Changed` クエリ）

---

## 4. 実装アプローチオプション

### Option A: 既存モジュール拡張

**対象ファイル**:
- `hit_test.rs`: HitTestMode/HitTest拡張、hit_test_entity分岐追加
- `bitmap_source/resource.rs`: AlphaMask構造体、BitmapSourceResource拡張
- `bitmap_source/systems.rs`: αマスク生成システム追加
- `com/wic.rs`: CopyPixels API追加

**トレードオフ**:
- ✅ ファイル数増加なし
- ✅ 既存パターン継承
- ❌ `resource.rs` が肥大化する可能性

### Option B: 新規モジュール作成

**新規ファイル**:
- `bitmap_source/alpha_mask.rs`: AlphaMask構造体、生成ロジック
- `layout/hit_test_alpha.rs`: αマスク判定専用ロジック

**トレードオフ**:
- ✅ 責務分離が明確
- ✅ テスト容易性向上
- ❌ ファイル数増加
- ❌ モジュール間依存の設計が必要

### Option C: ハイブリッド（推奨）

**方針**:
- `AlphaMask` 構造体: `bitmap_source/alpha_mask.rs` に新規作成
- `BitmapSourceResource` 拡張: `resource.rs` に `alpha_mask` フィールド追加
- `HitTestMode`/`HitTest` 拡張: `hit_test.rs` に追加
- `hit_test_entity()` 拡張: 既存関数に `AlphaMask` 分岐追加

**トレードオフ**:
- ✅ 責務分離と既存パターン維持のバランス
- ✅ `AlphaMask` ロジックがテストしやすい
- ✅ 既存APIシグネチャ維持
- ❌ やや複雑な構成

---

## 5. 工数・リスク評価

### 工数: M（3-7日）

**根拠**:
- 既存パターン（非同期Command、ECSクエリ）が確立
- WIC CopyPixels追加は軽微（1日）
- AlphaMask構造体・生成ロジック（1-2日）
- hit_test拡張（1日）
- 統合テスト（1-2日）

### リスク: Low

**根拠**:
- WIC CopyPixels APIは標準的なWIN32 API
- ビットパック処理は単純なビット演算
- 既存hit_testシステムへの影響は最小限
- 非同期生成中のフォールバック（Bounds）で安全性確保

---

## 6. 設計フェーズへの推奨事項

### 推奨アプローチ: Option C（ハイブリッド）

### 主要設計決定（要件定義で確定済み）

1. **αマスク配置**: `BitmapSourceResource.alpha_mask: Option<AlphaMask>`
2. **閾値**: 128固定（50%、セキュリティ要件）
3. **生成トリガー**: `HitTest::alpha_mask()` 設定時のみ
4. **フォールバック**: 生成中は `Bounds` として動作
5. **メモリ解放**: WICリソースと同時（自動）

### 設計フェーズでの検討事項

| 項目 | 検討内容 |
|------|---------|
| `alpha_mask.rs` の責務範囲 | 生成ロジックのみか、判定ロジックも含むか |
| WIC CopyPixels呼び出しタイミング | 画像読み込み完了時 or αマスク生成システム内 |
| テストアセット | 透明部分を含むPNG画像の作成 |
| ベンチマーク | 1000x1000画像での100ms以内達成確認 |

---

## 7. 次のステップ

1. **要件承認**: `/kiro-spec-approve event-hit-test-alpha-mask requirements`
2. **設計生成**: `/kiro-spec-design event-hit-test-alpha-mask`
3. **実装**: `/kiro-spec-impl event-hit-test-alpha-mask`

---

## Appendix: 参照資産

- 親仕様設計: `.kiro/specs/completed/event-hit-test/design.md`
- BitmapSourceウィジェット: `crates/wintf/src/ecs/widget/bitmap_source/`
- ヒットテストシステム: `crates/wintf/src/ecs/layout/hit_test.rs`
- WICラッパー: `crates/wintf/src/com/wic.rs`
