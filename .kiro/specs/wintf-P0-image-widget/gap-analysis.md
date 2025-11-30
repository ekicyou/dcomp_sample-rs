# Gap Analysis: wintf-P0-image-widget

| 項目 | 内容 |
|------|------|
| **Document Title** | wintf-P0-image-widget Gap分析レポート |
| **Date** | 2025-11-30 |
| **Language** | ja |

---

## 1. Analysis Summary

- **スコープ**: WIC画像読み込み、非同期ロード（WintfTaskPool）、D2D描画、ECS統合
- **既存資産**: WIC/D2Dラッパー（`com/wic.rs`, `com/d2d/`）、ウィジェットパターン（Label, Rectangle）、GraphicsCore
- **主要ギャップ**: WintfTaskPool（新規作成）、ImageResource/ImageGraphicsコンポーネント（新規）
- **推奨アプローチ**: **Option B（新規コンポーネント作成）** - 既存パターンに沿いつつ新規ファイルを追加
- **複雑度**: M（3-7日） / リスク: Low

---

## 2. Current State Investigation

### 2.1 既存資産マップ

| 資産 | 場所 | 状況 | 備考 |
|------|------|------|------|
| WIC Factory | `com/wic.rs` | ✅ 利用可能 | `wic_factory()`, デコーダー、フォーマットコンバーター |
| D2D Bitmap作成 | `com/d2d/mod.rs` | ✅ 利用可能 | `create_bitmap_from_wic_bitmap()` 実装済み |
| GraphicsCore | `ecs/graphics/core.rs` | ✅ 利用可能 | D2D DeviceContext、DWrite Factory等 |
| ウィジェットパターン | `ecs/widget/` | ✅ 参照可能 | Label, Rectangle の実装パターン |
| Visual/Surface管理 | `ecs/graphics/` | ✅ 利用可能 | VisualGraphics, SurfaceGraphics |
| bevy_tasks | `Cargo.toml` | ✅ 依存済み | TaskPool使用可能 |
| TaskPool使用例 | `ecs/common/tree_system.rs` | ✅ 参照可能 | `ComputeTaskPool::get_or_init()` |

### 2.2 既存ウィジェット実装パターン

**Rectangleウィジェット** (`ecs/widget/shapes/rectangle.rs`):
```
1. Component定義: #[derive(Component)]
2. on_add hook: Visual自動挿入
3. on_remove hook: GraphicsCommandListクリア
4. 描画システム: Changed<T>検知 → GraphicsCommandList生成
5. サイズ: Arrangementから取得
```

**Labelウィジェット** (`ecs/widget/text/label.rs`):
```
1. Component定義 + TextLayoutResource（キャッシュ）
2. on_add/on_remove hooks
3. 描画システム: draw_labels()
4. CPU/GPUリソース分離: TextLayoutResource（CPU）→ CommandList（描画）
```

### 2.3 命名規則（structure.mdより）

- **GPUリソース**: `XxxGraphics`（デバイスロスト対応、`invalidate()`, `generation`フィールド）
- **CPUリソース**: `XxxResource`（デバイス非依存、永続的）
- **論理コンポーネント**: サフィックスなし（`Label`, `Rectangle`, `Image`）

---

## 3. Requirements Feasibility Analysis

### 3.1 要件→資産マッピング

| 要件 | 必要な資産 | 既存 | ギャップ |
|------|-----------|------|---------|
| **Req 1**: 非同期読み込み | WintfTaskPool | ❌ Missing | 新規作成必要 |
| **Req 2**: WIC画像読み込み | wic.rs | ✅ | αチャンネル検証ロジック追加 |
| **Req 3**: 透過処理 | D2D premultiplied alpha | ✅ | なし |
| **Req 4**: D2D描画 | create_bitmap_from_wic_bitmap | ✅ | DrawBitmap統合 |
| **Req 5**: ECS統合 | ImageResource, ImageGraphics | ❌ Missing | 新規作成必要 |
| **Req 6**: 将来拡張性 | コンポーネント設計 | N/A | 設計考慮 |

### 3.2 ギャップ詳細

#### ギャップ1: WintfTaskPool（新規）

**必要性**: 要件1「EcsWorldの初期化時にResourceとして初期化」「Bevyの標準プールをブロックしない」

**実装案**:
```rust
#[derive(Resource)]
pub struct WintfTaskPool {
    pool: TaskPool,
}

impl WintfTaskPool {
    pub fn new() -> Self {
        Self {
            pool: TaskPoolBuilder::new()
                .thread_name("wintf-io")
                .num_threads(2)  // I/O向け少数スレッド
                .build(),
        }
    }
    
    pub fn spawn<T: Send + 'static>(&self, future: impl Future<Output = T> + Send + 'static) -> Task<T> {
        self.pool.spawn(future)
    }
}
```

**配置**: `ecs/task_pool.rs`（新規）

#### ギャップ2: ImageResource（新規CPUリソース）

**必要性**: 要件2, 5, 6

**設計**:
```rust
#[derive(Component)]
pub struct ImageResource {
    source: Option<IWICBitmapSource>,  // WICオブジェクト
    decoder: Option<IWICBitmapDecoder>, // 将来拡張用（アニメーション）
    // 将来: frame_count, frame_delays
}

unsafe impl Send for ImageResource {} // WICはスレッドフリー
unsafe impl Sync for ImageResource {}
```

**配置**: `ecs/widget/image/resource.rs`（新規）

#### ギャップ3: ImageGraphics（新規GPUリソース）

**必要性**: 要件4, 5

**設計**:
```rust
#[derive(Component)]
pub struct ImageGraphics {
    bitmap: Option<ID2D1Bitmap1>,
    generation: u32,  // デバイスロスト対応
}

impl ImageGraphics {
    pub fn invalidate(&mut self) {
        self.bitmap = None;
    }
}
```

**配置**: `ecs/widget/image/graphics.rs`（新規）

#### ギャップ4: Imageコンポーネント（新規）

**必要性**: 要件全体

**設計**:
```rust
#[derive(Component)]
#[component(on_add = on_image_add, on_remove = on_image_remove)]
pub struct Image {
    pub path: PathBuf,  // または String
}
```

**配置**: `ecs/widget/image/mod.rs`（新規）

### 3.3 研究が必要な項目

| 項目 | 不明点 | 優先度 |
|------|--------|--------|
| WIC αチャンネル検証 | `IWICBitmapSource::GetPixelFormat()` でPBGRA32を検証 | 高 |
| 非同期→ECS結果受け渡し | `Task<T>::poll()` またはChannel経由 | 高 |
| エラー状態のコンポーネント表現 | `ImageResource::error: Option<ImageError>` | 中 |

---

## 4. Implementation Approach Options

### Option A: 既存ファイル拡張

**アプローチ**: `ecs/graphics/components.rs` にImageResource/ImageGraphicsを追加

**Trade-offs**:
- ✅ ファイル数増加なし
- ❌ graphics.rsが肥大化（既に複雑）
- ❌ Image固有ロジック（WIC読み込み）がgraphicsに混入

**推奨度**: ❌ 非推奨

### Option B: 新規コンポーネント作成（推奨）

**アプローチ**: `ecs/widget/image/` ディレクトリを新規作成

```
ecs/widget/image/
├── mod.rs          # Image コンポーネント、on_add/on_remove フック
├── resource.rs     # ImageResource（WICオブジェクト保持）
├── graphics.rs     # ImageGraphics（ID2D1Bitmap保持）
├── loader.rs       # 非同期読み込みロジック
└── systems.rs      # draw_images、update_image_graphics等
```

**Trade-offs**:
- ✅ Label/Rectangleと同じ構造で一貫性
- ✅ Image固有ロジックが独立
- ✅ テストしやすい
- ❌ 新規ファイル追加

**推奨度**: ✅ **推奨**

### Option C: ハイブリッド

**アプローチ**: ImageResource/ImageGraphicsはgraphicsに、Imageとシステムはwidgetに

**Trade-offs**:
- ❌ コンポーネントと使用箇所が分離
- ❌ 依存関係が複雑化

**推奨度**: ❌ 非推奨

---

## 5. WintfTaskPool配置

**Option B-1**: `ecs/task_pool.rs`（新規ファイル）
- ✅ 独立したモジュール
- ✅ 将来的に他のI/O処理でも使用可能

**Option B-2**: `ecs/world.rs` に追加
- ❌ world.rsが既に大きい（500行超）
- ❌ 責務混在

**推奨**: **Option B-1**

---

## 6. Implementation Complexity & Risk

### 努力量: **M（3-7日）**

**内訳**:
- WintfTaskPool: 0.5日
- ImageResource/ImageGraphics: 1日
- Imageコンポーネント + hooks: 0.5日
- 非同期読み込みロジック: 1日
- 描画システム: 1日
- テスト: 1-2日

### リスク: **Low**

**理由**:
- 既存パターン（Label/Rectangle）を踏襲
- WIC/D2D APIラッパーは既存
- bevy_tasksは依存済み
- 新規技術要素なし

---

## 7. Recommendations for Design Phase

### 7.1 推奨アプローチ

**Option B（新規コンポーネント作成）** を採用:
- `ecs/widget/image/` ディレクトリ構成
- `ecs/task_pool.rs` にWintfTaskPool

### 7.2 設計フェーズでの検討事項

1. **非同期結果のECS統合パターン**
   - `Task<T>::poll()` をUpdateスケジュールで実行？
   - または `async-channel` でメインスレッドに送信？

2. **エラー状態の表現**
   - `ImageResource` にエラーフィールドを持たせるか
   - 別の `ImageLoadError` コンポーネントを使うか

3. **αチャンネル検証タイミング**
   - WIC読み込み時に検証してエラー
   - D2D Bitmap作成時に検証

4. **描画システムの登録**
   - `Draw` スケジュールに `draw_images` を追加
   - `Changed<ImageResource>` → `ImageGraphics` 更新

### 7.3 次のステップ

1. `/kiro-spec-design wintf-P0-image-widget` を実行して設計フェーズへ進む
2. 上記検討事項を設計ドキュメントで詳細化

---

## Appendix: 既存コード参照

### WIC読み込み例（dcomp_demo.rs）

```rust
fn create_image() -> Result<IWICFormatConverter> {
    let factory = wic_factory()?;
    let decoder = factory.create_decoder_from_filename(
        path, None, GENERIC_READ, WICDecodeMetadataCacheOnDemand,
    )?;
    let source = decoder.frame(0)?;
    let image = factory.create_format_converter()?;
    image.init(
        &source,
        &GUID_WICPixelFormat32bppBGR,  // ← P0ではPBGRA必須
        WICBitmapDitherTypeNone, None, 0.0, WICBitmapPaletteTypeMedianCut,
    )?;
    Ok(image)
}
```

### D2D Bitmap作成例

```rust
let bitmap = dc.create_bitmap_from_wic_bitmap(&self.image)?;
```

---

_Document generated by AI-DLC System on 2025-11-30_
