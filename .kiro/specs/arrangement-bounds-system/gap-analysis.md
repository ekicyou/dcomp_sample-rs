# Implementation Gap Analysis: arrangement-bounds-system

**Generated**: 2025-11-21  
**Status**: Gap analysis completed, ready for requirements revision

## Executive Summary

本機能の実装規模は**Small（1-2日、約10時間）**。既存の`propagate_parent_transforms`システムは**変更不要**で、trait実装（`Mul`, `From`）に数行追加するだけで実現可能。

### 実装の核心
```rust
// 既存のtrait実装を拡張するだけ
impl Mul<Arrangement> for GlobalArrangement {
    fn mul(self, rhs: Arrangement) -> Self::Output {
        let child_matrix: Matrix3x2 = rhs.into();
        let result_transform = self.0 * child_matrix;
        // ↓ 以下2-3行を追加
        let child_bounds = rhs.local_bounds();
        let result_bounds = transform_rect_axis_aligned(&child_bounds, &result_transform);
        GlobalArrangement { transform: result_transform, bounds: result_bounds }
    }
}
```

### 主な変更点
- **Size構造体追加** (`ecs/layout.rs`): レイアウトサイズ保持用の構造体定義
- **Rect型エイリアス + D2DRectExt** (`com/d2d/mod.rs`): D2D_RECT_Fの拡張トレイト
- **Arrangement/GlobalArrangement拡張**: `size`/`bounds`フィールド追加（破壊的変更）
- **trait実装拡張**: `Mul`と`From`にbounds計算を追加（各2-3行）

**破壊的変更**のため既存コード（examples, tests）の移行が必要だが、コンパイルエラーで漏れなく検出可能。

---

## 1. Current State Investigation

### 1.1 Key Files/Modules

#### `crates/wintf/src/ecs/layout.rs` (84行)
- **Arrangement**: `offset: Offset` + `scale: LayoutScale` のみ（**`size`フィールドなし**）
- **GlobalArrangement**: `Matrix3x2`のみ（**`bounds`フィールドなし**）
- **Offset**, **LayoutScale**: 既存のヘルパー構造体
- **ArrangementTreeChanged**: ダーティビットマーカー
- **From/Into変換**: `Arrangement → Matrix3x2` への変換実装済み

#### `crates/wintf/src/ecs/arrangement.rs` (58行)
- **sync_simple_arrangements**: 階層外エンティティのGlobalArrangement更新
- **mark_dirty_arrangement_trees**: ダーティビット伝播
- **propagate_global_arrangements**: 親→子へのGlobalArrangement伝播（`propagate_parent_transforms`を使用）

#### `crates/wintf/src/ecs/tree_system.rs` (371行)
- **propagate_parent_transforms**: 汎用階層伝播システム（並列処理対応）
  - ジェネリック型パラメータ: `L` (ローカル変換), `G` (グローバル変換), `M` (マーカー)
  - 制約: `G: Mul<L, Output = G>` (親グローバル × 子ローカル)
  - **現在の用途**: `Arrangement` → `GlobalArrangement` の変換行列伝播

#### `crates/wintf/src/com/d2d/mod.rs` (292行)
- **既存パターン**: `D2D1FactoryExt`, `D2D1DeviceExt`, `D2D1CommandListExt`, `D2D1DeviceContextExt`
- Direct2D APIの拡張トレイト集約場所
- **Color型エイリアス**: `pub type Color = D2D1_COLOR_F;` (既存パターン)

#### `crates/wintf/src/ecs/widget/shapes/rectangle.rs` (191行)
- **Rectangle**: `width: f32`, `height: f32`, `color: Color`
- **D2D_RECT_F使用例**: `D2D_RECT_F { left: 0.0, top: 0.0, right: width, bottom: height }` (line 141)

### 1.2 Architecture Patterns

#### レイヤードアーキテクチャ (依存方向: `ecs` → `com` → `windows`)
- **ecs層**: ビジネスロジック（ECSコンポーネント、システム）
- **com層**: Windows COM APIラッパー
- **例外ルール**: `com`から`ecs`のComponent型（データ構造のみ）を参照可能

#### ECSアーキテクチャ
- **bevy_ecs 0.17.2**: Entity-Component-System
- **階層管理**: `ChildOf` (親参照), `Children` (子リスト)
- **変更検知**: `Changed<T>`, `Added<T>`, `RemovedComponents<T>`
- **並列処理**: `par_iter_mut` + `ComputeTaskPool`

#### 汎用階層伝播システム
- **propagate_parent_transforms**: ジェネリック型で再利用可能
- **既存適用例**: `Transform`/`GlobalTransform` (回転・スキュー対応), `Arrangement`/`GlobalArrangement` (軸平行のみ)
- **最適化**: ダーティビット (`ArrangementTreeChanged`) による変更検知

#### 命名規則
- **Component型**: `PascalCase` + サフィックス規則
  - GPUリソース: `XxxGraphics` (例: `WindowGraphics`)
  - CPUリソース: `XxxResource` (例: `TextLayoutResource`)
  - 論理コンポーネント: サフィックスなし (例: `Rectangle`, `Label`)
- **型エイリアス**: 既存Windows型のラッパー (例: `type Color = D2D1_COLOR_F`)
- **拡張トレイト**: `XxxExt` (例: `D2D1DeviceContextExt`)

### 1.3 Integration Surfaces

#### 既存のArrangement使用箇所
- **examples/areka.rs**, **examples/dcomp_demo.rs**: サンプルアプリケーション
- **Rectangle**: `width`, `height`フィールドを持つが、Arrangementとは独立
- **Label**: `TextLayoutMetrics` (width, height) を持つが、Arrangementとは独立

#### Direct2D統合
- **D2D_RECT_F**: 既存の描画コードで使用 (`Rectangle::draw`)
- **Matrix3x2**: `GlobalArrangement`で使用、`windows_numerics`クレート提供

---

## 2. Requirements Feasibility Analysis

### 2.1 Technical Needs (from Requirements)

#### Requirement 1: Size構造体 (`ecs/layout.rs`)
- **データモデル**: `Size { width: f32, height: f32 }`
- **トレイト**: `Debug`, `Clone`, `Copy`, `PartialEq`, `Default`
- **統合**: `Arrangement.size`フィールド追加

#### Requirement 2: Rect型エイリアス + D2DRectExt (`com/d2d/mod.rs`)
- **型エイリアス**: `pub type Rect = D2D_RECT_F;`
- **拡張トレイト**: `D2DRectExt` (12メソッド)
  - 構築: `from_offset_size(Offset, Size) -> Rect`
  - 取得: `width()`, `height()`, `offset()`, `size()`
  - 設定: `set_offset()`, `set_size()`, `set_left()`, `set_top()`, `set_right()`, `set_bottom()`
  - 判定: `contains(x, y)`
  - 演算: `union(&Rect)`

#### Requirement 3: GlobalArrangement.bounds追加
- **データモデル**: `GlobalArrangement { transform: Matrix3x2, bounds: Rect }`
- **計算**: `Mul` trait実装内でbounds計算（2-3行追加）
- **最適化**: 2点変換のみ（軸平行専用）

#### Requirement 4-6: バウンディングボックス計算
- **実装方法**: 既存trait実装（`Mul<Arrangement>`, `From<Arrangement>`）拡張
- **変更箇所**: `impl Mul`と`impl From`に各2-3行追加
- **新規関数**: `transform_rect_axis_aligned(rect: &Rect, matrix: &Matrix3x2) -> Rect`（2点変換ヘルパー）
- **子孫集約** (Requirement 5): 別仕様で実装（本仕様のスコープ外）

### 2.2 Gaps & Constraints

#### ✅ 既存機能で完全対応可能
- **階層伝播システム**: `propagate_parent_transforms`は**変更不要**（`G: Mul<L, Output = G>`で既に動作）
- **trait制約**: `impl Mul<Arrangement> for GlobalArrangement`が既に存在、拡張するだけ
- **並列処理**: bevy_ecsの並列クエリ実行が既に動作
- **変更検知**: `ArrangementTreeChanged`マーカーが既に機能
- **Direct2D統合**: `D2D_RECT_F`は既存のRectangle描画で使用済み

#### ❌ 新規実装が必要
- **Size構造体**: 独自型定義（`ecs/layout.rs`に約10行）
- **Rect型エイリアス + D2DRectExt**: 拡張トレイト定義（`com/d2d/mod.rs`に約50行）
- **Arrangement.size**: フィールド追加（破壊的変更）
- **GlobalArrangement.bounds**: フィールド追加（破壊的変更）
- **trait実装拡張**: `Mul`と`From`に各2-3行追加
- **transform_rect_axis_aligned**: 2点変換ヘルパー関数（約15行）

#### ⚠️ 制約と考慮事項
- **破壊的変更**: `Arrangement`と`GlobalArrangement`の構造変更
  - 影響: 全examples/testsで`Arrangement`初期化時に`size`フィールド追加が必要
  - 検出: コンパイルエラーで漏れなく検出可能
- **軸平行変換のみ**: 回転・スキュー変換は非対応（将来のDirectComposition Visual層で実装予定）
- **依存関係例外**: `com/d2d/mod.rs`から`ecs/layout.rs`のComponent型参照
  - 範囲: `Size`, `Offset`型のみ（データ構造のみ、関数呼び出しなし）

#### 🔍 調査不要（既に確認済み）
- **propagate_parent_transformsの再利用性**: ✅ ジェネリック型制約で完全対応
- **Mul trait活用**: ✅ `G: Mul<L, Output = G>`で既に動作中
- **Matrix3x2の点変換**: ✅ `windows_numerics`で提供済み
- **D2D_RECT_F統合**: ✅ Rectangle描画で使用実績あり

### 2.3 Complexity Assessment

- **Simple**: Size構造体、Rect型エイリアス定義（データ構造のみ）
- **Simple**: trait実装拡張（`Mul`, `From`に各2-3行追加）
- **Simple**: `transform_rect_axis_aligned`（2点変換 + min/max、約15行）
- **Simple**: `D2DRectExt`実装（12メソッド、約50行、定型的なgetter/setter）
- **Simple**: 既存コード移行（コンパイルエラー修正、機械的作業）

**総合評価**: Simple - 新規アルゴリズムなし、既存パターン踏襲、変更箇所明確

---

## 3. Implementation Approach

### 推奨アプローチ: Extend Existing Components (唯一の現実的選択肢)

#### 変更対象ファイルと作業量

##### `crates/wintf/src/ecs/layout.rs` (+約30行)
```rust
// 追加: Size構造体定義 (約10行)
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

// 変更: Arrangementにsizeフィールド追加
pub struct Arrangement {
    pub offset: Offset,
    pub scale: LayoutScale,
    pub size: Size,  // 新規
}

// 変更: GlobalArrangementに構造体化
pub struct GlobalArrangement {
    pub transform: Matrix3x2,
    pub bounds: Rect,  // 新規
}

// 追加: Arrangement::local_bounds()メソッド (約5行)
impl Arrangement {
    pub fn local_bounds(&self) -> Rect {
        Rect::from_offset_size(self.offset, self.size)
    }
}

// 変更: impl Mul - boundsフィールド計算追加 (2-3行追加)
impl Mul<Arrangement> for GlobalArrangement {
    fn mul(self, rhs: Arrangement) -> Self::Output {
        let child_matrix: Matrix3x2 = rhs.into();
        let result_transform = self.transform * child_matrix;
        let child_bounds = rhs.local_bounds();
        let result_bounds = transform_rect_axis_aligned(&child_bounds, &result_transform);
        GlobalArrangement { transform: result_transform, bounds: result_bounds }
    }
}

// 変更: impl From - boundsフィールド初期化追加 (1-2行追加)
impl From<Arrangement> for GlobalArrangement {
    fn from(arrangement: Arrangement) -> Self {
        Self { 
            transform: arrangement.into(),
            bounds: arrangement.local_bounds(),
        }
    }
}

// 追加: transform_rect_axis_aligned関数 (約15行)
```

##### `crates/wintf/src/com/d2d/mod.rs` (+約60行)
```rust
// 追加: Rect型エイリアス (1行)
pub type Rect = D2D_RECT_F;

// 追加: D2DRectExt trait定義 + 実装 (約50行)
pub trait D2DRectExt {
    fn from_offset_size(offset: Offset, size: Size) -> Self;
    fn width(&self) -> f32;
    fn height(&self) -> f32;
    // ... 残り9メソッド
}

impl D2DRectExt for D2D_RECT_F {
    // 12メソッドの実装
}

// 追加: use文 (1行)
use crate::ecs::layout::{Size, Offset};
```

##### `crates/wintf/examples/*.rs`, `crates/wintf/tests/*.rs` (既存コード修正)
- **変更箇所**: `Arrangement { offset, scale }`を`Arrangement { offset, scale, size }`に修正
- **推定**: 約10-20箇所
- **作業時間**: 2-3時間（コンパイルエラー追跡と修正）

#### 実装手順（推奨）
1. `Size`構造体定義 → 0.5時間
2. `Arrangement`に`size`追加 → 0.5時間
3. 既存コード修正（コンパイルエラー解消） → 2-3時間
4. `Rect`型エイリアス + `D2DRectExt` → 2時間
5. `GlobalArrangement`構造体化 → 0.5時間
6. `transform_rect_axis_aligned` → 1時間
7. trait実装拡張（`Mul`, `From`） → 1時間
8. ユニットテスト → 2時間

**合計**: 約10時間（1-2日）

#### なぜOption B/Cは不適切か
- **Option B（独立コンポーネント）**: `Arrangement`は本来「位置+サイズ」を表現する概念。分離は設計理念に反する
- **Option C（段階実装）**: 規模が小さいため、段階化のオーバーヘッドの方が大きい

---

## 4. Implementation Complexity & Risk

### Effort: **S (1-2 days, ~10 hours)**

#### 作業内訳
| タスク | 時間 | 難易度 | 備考 |
|--------|------|--------|------|
| Size構造体定義 | 0.5h | Simple | データ構造定義のみ |
| Rect型エイリアス + D2DRectExt | 2h | Simple | 定型的なgetter/setter |
| Arrangement.size追加 | 0.5h | Simple | フィールド追加 |
| GlobalArrangement構造体化 | 0.5h | Simple | 1フィールド→2フィールド |
| trait実装拡張（Mul, From） | 1h | Simple | 各2-3行追加 |
| transform_rect_axis_aligned | 1h | Simple | 2点変換 + min/max |
| 既存コード移行 | 2-3h | Tedious | コンパイルエラー修正 |
| ユニットテスト | 2h | Simple | 既存パターン踏襲 |
| **合計** | **~10h** | - | - |

**重要**: `propagate_parent_transforms`システムは**変更不要**。trait実装だけで自動的にbounds伝播が動作する。

### Risk: **Low**

#### リスク評価
| リスク | レベル | 軽減策 | 評価 |
|--------|--------|--------|------|
| 破壊的変更の影響範囲 | Medium | コンパイルエラーで全箇所検出 | ✅ 管理可能 |
| trait実装のバグ | Low | ユニットテストで検証 | ✅ 低リスク |
| パフォーマンス劣化 | Low | 2点変換のみ、最適化済み | ✅ 問題なし |
| 依存関係例外 | Low | データ型のみ、関数呼び出しなし | ✅ 影響限定的 |

**総合評価**: Low Risk
- 既存システム変更なし（trait実装拡張のみ）
- 新規アルゴリズムなし（2点変換は自明）
- テスト範囲明確（trait実装とヘルパー関数のみ）

---

## 5. Recommendations for Design Phase

### 設計方針: trait実装拡張による最小侵襲アプローチ

#### 核心的な決定事項

##### 1. propagate_parent_transformsは変更しない
- **理由**: `G: Mul<L, Output = G>`の制約で既に汎用化済み
- **実装**: `impl Mul<Arrangement> for GlobalArrangement`拡張のみでbounds伝播が自動的に動作
- **影響**: 既存システムへの影響ゼロ、テスト範囲が限定的

##### 2. データ構造の配置
| 型 | 配置場所 | 理由 |
|----|----------|------|
| `Size` | `ecs/layout.rs` | `Offset`, `LayoutScale`と同じレイアウト関連型 |
| `Rect` | `com/d2d/mod.rs` | 既存の`Color`型エイリアスと同じパターン |
| `D2DRectExt` | `com/d2d/mod.rs` | 既存のDirect2D拡張トレイト群と集約 |

##### 3. 依存関係例外の正当性
- **例外内容**: `com/d2d/mod.rs`から`ecs/layout::{Size, Offset}`を参照
- **制約**: データ型のみ参照、関数・システム呼び出しは禁止
- **正当性**: 
  - `D2D_RECT_F`はDirect2D APIの基盤型
  - `Size`/`Offset`はf32のペア（純粋なデータ構造）
  - 実装の凝集性（D2D関連APIを`com/d2d/mod.rs`に集約）とのトレードオフ

##### 4. bounds計算の実装箇所
```rust
// impl Mul内で計算（2-3行追加）
impl Mul<Arrangement> for GlobalArrangement {
    fn mul(self, rhs: Arrangement) -> Self::Output {
        let child_matrix: Matrix3x2 = rhs.into();
        let result_transform = self.transform * child_matrix;
        // ↓ 新規
        let child_bounds = rhs.local_bounds();
        let result_bounds = transform_rect_axis_aligned(&child_bounds, &result_transform);
        GlobalArrangement { transform: result_transform, bounds: result_bounds }
    }
}
```

##### 5. 軸平行変換の最適化
- **2点変換のみ**: 左上と右下の2点を変換、min/maxで新しいbounds構築
- **理由**: 軸平行変換では4点変換は冗長（2点で十分）
- **実装**: `transform_rect_axis_aligned`ヘルパー関数（約15行）

#### 設計フェーズで決定すべき詳細

##### 必須事項
1. `D2DRectExt`の各メソッドのシグネチャ確認
2. `transform_rect_axis_aligned`の詳細実装（点変換メソッド確認）
3. 既存コード移行の優先順位（examples → tests）

##### 任意事項（本仕様のスコープ外）
1. Matrix3x2逆行列メソッド調査（子孫bounds集約で必要、別仕様）
2. パフォーマンステスト実装方法（bevy_ecsベンチマークパターン）

---

## 6. Next Steps & Implementation Roadmap

### Step 1: Requirements Revision (必須)
Gap Impact Assessmentで指摘された以下の修正を要件定義に反映：
1. **Requirement 5削除**: 子孫bounds集約をOut of Scopeに移動
2. **Requirement 4簡略化**: 新規システム要求を削除、trait実装拡張に変更
3. **工数見積もり修正**: M（3-7日）→ S（1-2日）

### Step 2: Requirements Approval
- 修正後の要件定義を確認・承認
- `/kiro-spec-design arrangement-bounds-system`で設計フェーズへ進む

### Step 3: Design Phase (0.5日)
- `D2DRectExt`の12メソッドの詳細仕様
- `transform_rect_axis_aligned`の実装詳細
- 既存コード移行の優先順位とチェックリスト

### Step 4: Implementation (1-2日)
推奨実装順序：
1. **データ構造** (1時間): `Size`, `Rect`, `GlobalArrangement`構造体化
2. **D2DRectExt** (2時間): 12メソッド実装
3. **Arrangement.size追加** (0.5時間): フィールド追加
4. **既存コード修正** (2-3時間): コンパイルエラー解消
5. **trait実装拡張** (1時間): `Mul`, `From`にbounds計算追加
6. **ヘルパー関数** (1時間): `transform_rect_axis_aligned`
7. **ユニットテスト** (2時間): trait実装とヘルパー関数のテスト

### Step 5: Testing & Verification (含まれる)
- ユニットテスト: trait実装、`transform_rect_axis_aligned`
- 統合テスト: 階層的bounds伝播の検証
- 既存テスト: 全テストがパスすることを確認

---

_Gap analysis completed. Implementation is straightforward with ~10 hours of work. Ready for requirements revision._
