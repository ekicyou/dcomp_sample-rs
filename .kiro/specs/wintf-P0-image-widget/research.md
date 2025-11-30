# Research Document: wintf-P0-image-widget

## 1. Summary

### 1.1 Feature Overview
wintf ECSフレームワークに静止画像表示ウィジェットを追加する機能。非同期ファイル読み込み、WIC/D2D統合、透過（αチャネル）サポートを含む。

### 1.2 Discovery Type
**Extension** - 既存のウィジェットパターン（Rectangle、Label）を拡張し、新しい画像ウィジェットを追加する。

### 1.3 Key Findings Summary
- **既存パターン**: Rectangle/Label が on_add/on_remove フック、Visual自動挿入、GraphicsCommandList描画パターンを確立済み
- **非同期統合**: `bevy_tasks::TaskPool` + `std::sync::mpsc` チャネルによるCommand発行パターンを新規採用
- **WIC/D2D連携**: 既存の `com/wic.rs` と `com/d2d/mod.rs` に必要なAPIが揃っている

---

## 2. Research Log

### 2.1 Discovery Process

| Phase | 対象 | 結果 |
|-------|------|------|
| 既存パターン分析 | `rectangle.rs`, `label.rs` | on_add/on_remove、GraphicsCommandList、Arrangement統合パターンを確認 |
| WIC API確認 | `com/wic.rs` | `WICImagingFactoryExt`, `WICBitmapDecoderExt`, `WICFormatConverterExt` 確認済み |
| D2D API確認 | `com/d2d/mod.rs` | `create_bitmap_from_wic_bitmap`, `draw_bitmap` 確認済み |
| 依存関係確認 | `Cargo.toml` | `bevy_tasks` は既存依存、追加不要 |
| ディレクトリ構造 | `ecs/widget/` | `shapes/`, `text/` パターン存在、`image/` 新規追加推奨 |

### 2.2 Technology Alignment

| 技術 | 状態 | 備考 |
|------|------|------|
| `bevy_tasks::TaskPool` | ✅ 利用可能 | Cargo.tomlに依存済み |
| `std::sync::mpsc` | ✅ 標準ライブラリ | 追加依存不要 |
| WIC (IWICBitmapSource) | ✅ 既存ラッパー | `com/wic.rs` |
| D2D (ID2D1Bitmap1) | ✅ 既存ラッパー | `com/d2d/mod.rs` |
| GUID_WICPixelFormat32bppPBGRA | ✅ WIC標準 | αチャネル必須フォーマット |

---

## 3. Architecture Pattern Evaluation

### 3.1 Option A: 既存モジュール内追加
- **概要**: `ecs/widget/shapes/` または `ecs/widget/text/` に image.rs を追加
- **長所**: ファイル数最小
- **短所**: 責務混在、将来拡張困難
- **評価**: ❌ 不採用

### 3.2 Option B: 新規 `image/` モジュール（推奨）
- **概要**: `ecs/widget/image/` ディレクトリを新設
- **長所**: 明確な責務分離、Rectangle/Labelと一貫したパターン
- **短所**: ファイル数増加（許容範囲）
- **評価**: ✅ 採用

### 3.3 非同期パターン選択

| パターン | 特徴 | 評価 |
|----------|------|------|
| async/await + Reactor | bevy_tasks標準 | ❌ Input scheduleでのpoller必要 |
| TaskPool + mpsc | 明示的チャネル | ✅ 採用（要件1.6, 1.7準拠） |
| oneshot channel | 単一結果 | ❌ 複数画像対応困難 |

---

## 4. Design Decisions

### 4.1 Component Architecture

```
Image (Component)
  └─ path: String
  └─ on_add → spawn async task
  └─ on_remove → cleanup

ImageResource (Component) - CPU側
  └─ source: IWICBitmapSource
  └─ Send + Sync (WIC thread-free marshaling)

ImageGraphics (Component) - GPU側
  └─ bitmap: ID2D1Bitmap1
  └─ generation: u64 (device lost対応)
```

**Decision**: CPU/GPUリソース分離パターンを採用（Label/TextLayoutResourceパターンと一致）

### 4.2 Async Integration

```
WintfTaskPool (Resource)
  └─ pool: TaskPool
  └─ sender: mpsc::Sender<BoxedCommand>
  └─ receiver: mpsc::Receiver<BoxedCommand>

BoxedCommand = Box<dyn Command + Send>
CommandSender = mpsc::Sender<BoxedCommand>
```

**Decision**: 
- `Box<dyn Command + Send>` で汎用化（ImageCommand enum廃止）
- `spawn(|tx| async move { ... })` 形式でCommandSenderを自動渡し
- 将来の他の非同期処理（TextLayout等）も同じパターンで対応可能

### 4.3 Error Handling

| エラー種別 | 対応 |
|------------|------|
| ファイル不存在 | ImageResource未生成 + eprintln |
| フォーマット非対応 | ImageResource未生成 + eprintln |
| αチャネル欠落 | WIC読み込み時に拒否 + eprintln |
| Device Lost | ImageGraphics再生成（generation比較） |

**Decision**: エラー時は「無表示 + ログ出力」方式

---

## 5. Risk Assessment

### 5.1 Technical Risks

| リスク | 影響度 | 軽減策 |
|--------|--------|--------|
| WIC Send/Sync | 中 | Thread-free marshaling確認済み |
| Device Lost競合 | 低 | generation比較パターン |
| 非同期タイミング | 低 | Input schedule drain |

### 5.2 Integration Risks

| リスク | 影響度 | 軽減策 |
|--------|--------|--------|
| 既存ウィジェット影響 | 低 | 独立モジュール構成 |
| GraphicsCore依存 | 低 | 既存パターン踏襲 |

---

## 6. Boundary Analysis

### 6.1 Module Boundaries

```
crates/wintf/src/
├── ecs/
│   └── widget/
│       └── image/          # 新規モジュール
│           ├── mod.rs
│           ├── image.rs    # Image component
│           ├── resource.rs # ImageResource, ImageGraphics
│           └── systems.rs  # load_images, draw_images
├── com/
│   ├── wic.rs             # 既存利用
│   └── d2d/mod.rs         # 既存利用
```

### 6.2 Public API Surface

- `Image` component (public)
- `ImageResource` component (pub(crate))
- `ImageGraphics` component (pub(crate))
- `WintfTaskPool` resource (pub(crate))

---

## 7. References

- [WIC Pixel Formats](https://learn.microsoft.com/windows/win32/wic/-wic-codec-native-pixel-formats)
- [D2D1DeviceContext::CreateBitmapFromWicBitmap](https://learn.microsoft.com/windows/win32/api/d2d1_1/nf-d2d1_1-id2d1devicecontext-createbitmapfromwicbitmap)
- [bevy_tasks documentation](https://docs.rs/bevy_tasks/)
- 既存実装: `ecs/widget/shapes/rectangle.rs`, `ecs/widget/text/label.rs`
