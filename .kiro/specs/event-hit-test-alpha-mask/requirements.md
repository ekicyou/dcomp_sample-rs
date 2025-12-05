# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | event-hit-test-alpha-mask 要件定義書 |
| **Version** | 1.0 (Draft) |
| **Date** | 2025-12-05 |
| **Parent Spec** | event-hit-test |
| **Author** | AI-DLC System |

---

## Introduction

本仕様書は wintf フレームワークにおけるαマスクを使用したピクセル単位ヒットテストシステムの要件を定義する。親仕様「event-hit-test」を拡張し、画像の透明部分をヒット対象外とする機能を提供する。

### 背景

デスクトップマスコットアプリケーションでは、キャラクター画像の透明部分をクリックしてもヒットしないことが期待される。親仕様の矩形判定（`HitTestMode::Bounds`）では透明部分もヒット対象となってしまうため、αチャンネルを考慮したピクセル単位の判定が必要となる。

### スコープ

**含まれるもの**:
- `HitTestMode::AlphaMask` バリアントの追加
- 画像読み込み時の2値マスク（ヒットマスク）自動生成
- ピクセル座標からのヒット判定API
- αマスクコンポーネントとキャッシュ管理
- 閾値のカスタマイズ機能

**含まれないもの**:
- 多角形・カスタム形状によるヒットテスト（将来の仕様）
- 名前付きヒット領域（event-hit-test-named-regions）
- スケーリング時のマスク再計算（初期版はオリジナルサイズのみ）

### 前提条件

- 親仕様「event-hit-test」が実装済み（✅ 完了）
- `BitmapSource` ウィジェットが実装済み（✅ 完了）
- WIC画像読み込みシステムが稼働中（✅ 完了）

---

## Requirements

### Requirement 1: HitTestMode 拡張

**Objective:** 開発者として、画像のα値に基づくヒットテストモードを設定したい。それにより透明部分をクリック透過させられる。

#### Acceptance Criteria

1. The HitTest System shall `HitTestMode` enumに `AlphaMask` バリアントを追加する
2. When `HitTestMode::AlphaMask` が設定されている時, the HitTest System shall αマスクを使用してピクセル単位のヒット判定を行う
3. When `HitTestMode::AlphaMask` が設定されているが `AlphaMask` コンポーネントが存在しない時, the HitTest System shall `HitTestMode::Bounds`（矩形判定）にフォールバックする
4. The HitTest System shall 既存の `HitTestMode::None` および `HitTestMode::Bounds` の動作を維持する

---

### Requirement 2: AlphaMaskコンポーネント

**Objective:** 開発者として、画像のαマスク情報をエンティティに関連付けたい。それによりヒットテスト時に効率的な判定が可能になる。

#### Acceptance Criteria

1. The HitTest System shall `AlphaMask` コンポーネントを `ecs::layout` モジュールに提供する
2. The `AlphaMask` component shall 以下のフィールドを持つ：
   - `data: Vec<u8>` - 1ビット/ピクセルのマスクデータ（ビットパック）
   - `width: u32` - マスクの幅（ピクセル）
   - `height: u32` - マスクの高さ（ピクセル）
   - `threshold: u8` - ヒット判定閾値（デフォルト: 128 = 50%）
3. The `AlphaMask` component shall `is_hit(x: u32, y: u32) -> bool` メソッドを提供する
4. When 座標がマスク範囲外の時, the `is_hit` method shall `false` を返す
5. The `AlphaMask` component shall `with_threshold(threshold: u8)` ビルダーメソッドを提供する

#### 設計決定

**ビットパック方式**:
- 1ピクセル = 1ビットで保持（8ピクセル/バイト）
- メモリ効率: 1000x1000画像で約125KB（元の4MBから97%削減）
- バイト境界: 各行は8ピクセル単位でアラインメント

**閾値のデフォルト値**:
- `threshold = 128`（50%）: α ≧ 128 の領域がヒット対象
- 0-255の範囲で指定可能（0 = 完全透明以外すべてヒット、255 = 完全不透明のみヒット）

---

### Requirement 3: BitmapSourceとの統合

**Objective:** 開発者として、BitmapSource画像からαマスクを自動生成したい。それにより手動でのマスク設定が不要になる。

#### Acceptance Criteria

1. When `BitmapSourceResource` が挿入され、かつ `HitTest { mode: AlphaMask }` が設定されている時, the HitTest System shall 非同期でαマスクを生成する
2. When `HitTest { mode: Bounds }` または `HitTest` コンポーネントがない時, the HitTest System shall αマスク生成をスキップする
3. The αマスク生成 shall WICのピクセルデータからα値を抽出してビットマスクに変換する
4. The HitTest System shall αマスク生成完了時に `AlphaMask` コンポーネントをエンティティに挿入する
5. When αマスク生成中の時, the HitTest System shall `HitTestMode::Bounds` として動作する（フォールバック）
6. The HitTest System shall αマスク生成を `WintfTaskPool` を使用して非同期実行する

#### 設計決定: ハイブリッド生成トリガー

**方針**: `HitTest::alpha_mask()` が設定されている場合のみ、`BitmapSourceResource` 挿入時にαマスクを自動生成する。

**理由**:
- 不要な画像（背景、装飾等）でのマスク生成を回避し、メモリ・CPU使用量を削減
- 開発者が明示的にαマスク判定を指定することで意図が明確になる
- `HitTest::bounds()` の画像は矩形判定のまま動作（既存動作との互換性維持）

**使用例**:
```rust
// αマスク生成される（HitTest::alpha_mask() 指定）
commands.spawn((
    BitmapSource::new("character.png"),
    HitTest::alpha_mask(),
));

// αマスク生成されない（HitTest::bounds() または未指定）
commands.spawn((
    BitmapSource::new("background.png"),
    HitTest::bounds(),  // 矩形判定のまま
));
```

#### 生成アルゴリズム

```rust
// PBGRA32形式: B, G, R, A の順（4バイト/ピクセル）
for y in 0..height {
    for x in 0..width {
        let offset = (y * stride + x * 4) as usize;
        let alpha = pixels[offset + 3];  // Aチャネル
        if alpha >= threshold {
            set_bit(mask_data, x, y, width);
        }
    }
}
```

---

### Requirement 4: ピクセル単位ヒット判定

**Objective:** 開発者として、座標がαマスク内のヒット領域にあるか判定したい。それにより透明部分を正確に判定できる。

#### Acceptance Criteria

1. The HitTest System shall `hit_test_alpha_mask(world: &World, entity: Entity, local_point: PhysicalPoint) -> bool` 関数を提供する
2. When 座標が `GlobalArrangement.bounds` 外の時, the function shall `false` を返す（早期リターン）
3. When 座標が `GlobalArrangement.bounds` 内の時, the function shall 座標をマスク座標に変換してヒット判定を行う
4. The HitTest System shall `GlobalArrangement.bounds` とマスクサイズの比率に基づいて座標変換を行う
5. When `AlphaMask` コンポーネントが存在しない時, the function shall `true` を返す（矩形判定と同等）

#### 座標変換

```rust
// スクリーン座標 → マスク座標への変換
let bounds = global_arrangement.bounds;
let mask_x = ((screen_x - bounds.left) / (bounds.right - bounds.left) * mask.width as f32) as u32;
let mask_y = ((screen_y - bounds.top) / (bounds.bottom - bounds.top) * mask.height as f32) as u32;
```

---

### Requirement 5: hit_test関数の拡張

**Objective:** 開発者として、既存のhit_test APIでαマスク判定を利用したい。それにより呼び出し側コードの変更が不要になる。

#### Acceptance Criteria

1. The `hit_test_entity` function shall `HitTestMode::AlphaMask` を処理する分岐を追加する
2. When `HitTestMode::AlphaMask` の時, the `hit_test_entity` function shall まず矩形判定を行い、通過した場合にαマスク判定を行う
3. The HitTest System shall 矩形判定とαマスク判定の二段階判定で効率的な処理を行う
4. The HitTest System shall 既存の `hit_test`, `hit_test_in_window` APIとの互換性を維持する

---

### Requirement 6: HitTest コンポーネントAPI

**Objective:** 開発者として、αマスクモードを簡単に設定したい。それによりコードの可読性が向上する。

#### Acceptance Criteria

1. The `HitTest` component shall `alpha_mask()` コンストラクタメソッドを提供する
2. The `HitTest::alpha_mask()` shall `HitTest { mode: HitTestMode::AlphaMask }` を返す
3. The HitTest System shall 以下の使用例をサポートする：
   ```rust
   // αマスクモードで画像を表示
   commands.spawn((
       BitmapSource::new("character.png"),
       HitTest::alpha_mask(),
   ));
   ```

---

## Non-Functional Requirements

### NFR-1: パフォーマンス

- αマスク生成: 1000x1000画像で100ms以内（非同期処理）
- ヒット判定: 1μs以内（ビットアクセスのみ）
- メモリ使用量: 1000x1000画像で約125KB（ビットパック）

### NFR-2: メモリ効率

- ビットパック形式により元画像の約3%のメモリで保持
- 大きな画像（4000x4000以上）でも1MB以下

### NFR-3: 互換性

- 既存の `HitTestMode::None`, `HitTestMode::Bounds` の動作に影響しない
- 既存の `hit_test`, `hit_test_in_window` APIシグネチャを維持
- `AlphaMask` なしの `HitTestMode::AlphaMask` は矩形判定にフォールバック

### NFR-4: テスト容易性

- 単体テストでαマスク生成とヒット判定を検証可能
- テスト用画像（透明部分あり）を使用した統合テスト
- デモでの動作確認（taffy_flex_demo等）

---

## Glossary

| 用語 | 説明 |
|------|------|
| αマスク | 画像のα値を2値化したビットマップ |
| ヒットマスク | αマスクの別名、ヒットテスト用マスク |
| ビットパック | 1ビット/ピクセルでデータを格納する方式 |
| PBGRA32 | Pre-multiplied Blue-Green-Red-Alpha 32bit形式 |
| 閾値 | ヒット判定の境界となるα値（デフォルト128） |

---

## Appendix

### A. 関連ドキュメント

- 親仕様: `.kiro/specs/completed/event-hit-test/requirements.md`
- BitmapSourceウィジェット: `crates/wintf/src/ecs/widget/bitmap_source/`
- ヒットテストシステム: `crates/wintf/src/ecs/layout/hit_test.rs`

### B. AlphaMaskコンポーネント設計

```rust
/// αマスクコンポーネント
/// 
/// 画像のα値を2値化したビットマスクを保持する。
/// ヒットテスト時にピクセル単位の判定に使用。
#[derive(Component, Debug, Clone)]
pub struct AlphaMask {
    /// 1ビット/ピクセルのマスクデータ（ビットパック）
    /// 行ごとに8ピクセル単位でアラインメント
    data: Vec<u8>,
    /// マスクの幅（ピクセル）
    width: u32,
    /// マスクの高さ（ピクセル）
    height: u32,
    /// ヒット判定閾値（α ≧ threshold でヒット）
    threshold: u8,
}

impl AlphaMask {
    /// PBGRA32ピクセルデータからαマスクを生成
    pub fn from_pbgra32(
        pixels: &[u8],
        width: u32,
        height: u32,
        stride: u32,
        threshold: u8,
    ) -> Self {
        // 行あたりのバイト数（8ピクセル単位に切り上げ）
        let row_bytes = (width + 7) / 8;
        let mut data = vec![0u8; (row_bytes * height) as usize];
        
        for y in 0..height {
            for x in 0..width {
                let pixel_offset = (y * stride + x * 4) as usize;
                let alpha = pixels.get(pixel_offset + 3).copied().unwrap_or(0);
                if alpha >= threshold {
                    let bit_offset = (y * row_bytes * 8 + x) as usize;
                    let byte_index = bit_offset / 8;
                    let bit_index = 7 - (bit_offset % 8);  // MSBファースト
                    data[byte_index] |= 1 << bit_index;
                }
            }
        }
        
        Self { data, width, height, threshold }
    }
    
    /// 指定座標がヒット対象かを判定
    pub fn is_hit(&self, x: u32, y: u32) -> bool {
        if x >= self.width || y >= self.height {
            return false;
        }
        let row_bytes = (self.width + 7) / 8;
        let bit_offset = (y * row_bytes * 8 + x) as usize;
        let byte_index = bit_offset / 8;
        let bit_index = 7 - (bit_offset % 8);
        (self.data[byte_index] >> bit_index) & 1 == 1
    }
    
    /// 閾値を指定してビルド
    pub fn with_threshold(mut self, threshold: u8) -> Self {
        self.threshold = threshold;
        self
    }
}
```

### C. HitTestMode拡張

```rust
/// ヒットテストの動作モード（拡張版）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HitTestMode {
    /// ヒットテスト対象外（マウスイベントを透過）
    None,
    /// バウンディングボックス（GlobalArrangement.bounds）でヒットテスト
    #[default]
    Bounds,
    /// αマスクによるピクセル単位ヒットテスト
    AlphaMask,
}
```

### D. 使用例

```rust
use wintf::ecs::{BitmapSource, HitTest};

// 透明部分をクリック透過させる画像
commands.spawn((
    BitmapSource::new("assets/character.png"),
    HitTest::alpha_mask(),
    BoxSize::fixed(256.0, 256.0),
));

// カスタム閾値（25% = 64）で判定
commands.spawn((
    BitmapSource::new("assets/semi_transparent.png"),
    HitTest::alpha_mask(),
    AlphaMaskConfig { threshold: 64 },  // 将来の拡張
));
```
