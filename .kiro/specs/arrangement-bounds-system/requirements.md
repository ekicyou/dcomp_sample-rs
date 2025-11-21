# Requirements Document

## Project Description (Input)
Arrangement拡張とバウンディングボックスシステム: ContentSizeコンポーネント、ArrangementへのSizeフィールド、GlobalArrangementへのBoundsフィールドによる軸平行バウンディングボックス管理。Surface生成最適化の前提条件を整備

## Introduction

本要件定義は、wintfフレームワークのレイアウトシステムにバウンディングボックス管理機能を追加するための要件を定義する。現在のArrangementコンポーネントは位置（Offset）とスケール（LayoutScale）のみを保持しており、サイズ情報が欠落している。これにより、Surface生成時のサイズ決定や子孫要素の描画領域集約が困難である。

本機能では、以下の2つの主要拡張を導入する：

1. **Arrangement.size**: レイアウト計算結果としての確定サイズ（taffyレイアウト出力を保持）
2. **GlobalArrangement.bounds**: ワールド座標系でのバウンディングボックス（描画領域）

これらにより、軸平行（Axis-Aligned）なバウンディングボックス管理システムを構築し、将来のSurface生成最適化の前提条件を整備する。

**重要な設計制約**: 回転やスキュー変換は今回のスコープ外とし、軸平行変換（平行移動とスケール）のみをサポートする。回転等の視覚効果は将来、DirectComposition Visual層で実装予定である。

**レイアウト統合**: `Arrangement.size`は将来的にtaffyレイアウトエンジンの出力として設定される。本仕様では、サイズが既に与えられている前提でバウンディングボックス変換計算の正確性を検証する。

---

## Requirements

### Requirement 1: Arrangementコンポーネントへのサイズフィールド追加

**Objective:** システム開発者として、レイアウト計算後の確定サイズをArrangementコンポーネントに保持したい。これにより、位置とサイズを一体として管理できる。

#### Acceptance Criteria

1. The wintfシステムは、`Arrangement`コンポーネントに`size: Size`フィールドを追加しなければならない（shall）。

2. The wintfシステムは、`crates/wintf/src/ecs/layout.rs`に`Size`構造体を定義しなければならない（shall）。この構造体は以下のフィールドを持つ：
   - `width: f32` - 幅（ピクセル単位）
   - `height: f32` - 高さ（ピクセル単位）

3. The `Size`構造体は、`Debug`、`Clone`、`Copy`、`PartialEq`、`Default`トレイトを実装しなければならない（shall）。

**Note:** `Size`は独自型として定義し、`Offset`、`LayoutScale`と同じ`ecs/layout.rs`に配置する。将来のtaffyレイアウトエンジン統合で、レイアウト計算結果を保持する用途に特化する。

4. The 既存の`Arrangement`コンポーネントは、以下のフィールド構成に変更されなければならない（shall）：
   ```rust
   pub struct Arrangement {
       pub offset: Offset,
       pub scale: LayoutScale,
       pub size: Size,  // 新規追加
   }
   ```

5. The `Arrangement::default()`は、`size: Size { width: 0.0, height: 0.0 }`を返さなければならない（shall）。

6. The wintfシステムは、`Arrangement`の`local_bounds() -> Rect`メソッドを提供しなければならない（shall）。このメソッドは、`offset`と`size`から軸平行バウンディングボックス（`Rect`）を返す。

7. When テストコードで`Arrangement`が構築される時、開発者は`size`フィールドを明示的に設定しなければならない（shall）。将来的にはtaffyレイアウトエンジンが`Arrangement.size`を設定する。

---

### Requirement 2: Rect型エイリアスと拡張トレイト

**Objective:** システム開発者として、軸平行バウンディングボックスを表現する型が必要である。Direct2D APIとの統合のため、`D2D_RECT_F`を直接使用し、拡張トレイトで必要な機能を提供する。

#### Acceptance Criteria

1. The wintfシステムは、`crates/wintf/src/com/d2d/mod.rs`に`Rect`型エイリアスを定義しなければならない（shall）：
   ```rust
   pub type Rect = D2D_RECT_F;
   ```
   この型は`windows::Win32::Graphics::Direct2D::Common::D2D_RECT_F`を参照する。

2. The wintfシステムは、`crates/wintf/src/com/d2d/mod.rs`に`D2D_RECT_F`に対する拡張トレイト`D2DRectExt`を提供しなければならない（shall）。このトレイトは以下のメソッドを含む：
   - **構築**: `from_offset_size(offset: Offset, size: Size) -> Self` - offsetとsizeから矩形を構築
   - **取得**: 
     - `width(&self) -> f32` - 幅を返す（`right - left`）
     - `height(&self) -> f32` - 高さを返す（`bottom - top`）
     - `offset(&self) -> Vector2` - 左上座標を`Vector2 { X: left, Y: top }`として返す
     - `size(&self) -> Vector2` - サイズを`Vector2 { X: width, Y: height }`として返す
   - **設定**:
     - `set_offset(&mut self, offset: Vector2)` - 左上座標を設定（幅・高さは維持）
     - `set_size(&mut self, size: Vector2)` - サイズを設定（左上座標は維持）
     - `set_left(&mut self, left: f32)`, `set_top(&mut self, top: f32)`, `set_right(&mut self, right: f32)`, `set_bottom(&mut self, bottom: f32)` - 各座標を個別設定
   - **判定**: `contains(&self, x: f32, y: f32) -> bool` - 点が矩形内に含まれるか判定
   - **演算**: `union(&self, other: &Self) -> Self` - 2つの矩形を包含する最小外接矩形を返す

3. The `D2DRectExt::from_offset_size`メソッドは、以下の式で矩形を構築しなければならない（shall）：
   ```rust
   D2D_RECT_F {
       left: offset.x,
       top: offset.y,
       right: offset.x + size.width,
       bottom: offset.y + size.height,
   }
   ```

**Note:** 既存コードで`D2D_RECT_F`を直接使用している箇所との互換性を保つため、型エイリアスと拡張トレイトのパターンを採用。`D2DRectExt`は、`ecs/layout`モジュールの`Size`と`Offset`型を参照する（データ構造のみ、関数呼び出しなし）。

---

### Requirement 3: GlobalArrangementへのBoundsフィールド追加

**Objective:** システム開発者として、ワールド座標系での最終描画領域をGlobalArrangementに保持したい。これにより、Surface生成時のサイズ決定やヒットテストが容易になる。

#### Acceptance Criteria

1. The wintfシステムは、`GlobalArrangement`コンポーネントを以下の構造に変更しなければならない（shall）：
   ```rust
   pub struct GlobalArrangement {
       pub transform: Matrix3x2,  // 累積変換行列
       pub bounds: Rect,          // ワールド座標系でのバウンディングボックス
   }
   ```

2. The `GlobalArrangement::default()`は、`transform: Matrix3x2::identity()`と`bounds: Rect::default()`を返さなければならない（shall）。

3. When `GlobalArrangement`が親の`GlobalArrangement`と子の`Arrangement`から計算される時、wintfシステムは以下の手順で`bounds`を計算しなければならない（shall）：
   a. 子の`Arrangement.local_bounds()`から`local_bounds`（ローカル座標系での`D2D_RECT_F`）を取得
   b. `local_bounds`の左上と右下の2点を`transform`で変換
   c. 変換後の2点から新しい軸平行バウンディングボックス（`D2D_RECT_F`）を構築

4. When 回転やスキュー変換が`transform`に含まれる場合、wintfシステムは警告ログを出力してもよい（may）。本システムは軸平行変換のみをサポートする。

---

### Requirement 4: バウンディングボックス計算（trait実装拡張）

**Objective:** システム開発者として、既存のtrait実装を拡張してバウンディングボックスを自動的に計算したい。これにより、階層伝播システムの変更なしに機能を追加できる。

#### Acceptance Criteria

1. The `GlobalArrangement`の`Mul<Arrangement>`実装は、`transform`と`bounds`の両方を計算しなければならない（shall）。

2. When `parent * child`が計算される時、以下を実行しなければならない（shall）：
   a. `transform` = 親.transform × 子.Arrangement変換行列
   b. `bounds` = transform_rect_axis_aligned(子.local_bounds(), 結果のtransform)

3. The `From<Arrangement>`実装は、初期`GlobalArrangement`の`bounds`を`Arrangement.local_bounds()`から設定しなければならない（shall）。

4. The wintfシステムは、`transform_rect_axis_aligned(rect: &Rect, matrix: &Matrix3x2) -> Rect`ヘルパー関数を提供しなければならない（shall）。この関数は2点変換（左上と右下）で軸平行矩形を変換する。

**Note**: 既存の`propagate_parent_transforms`システムは**変更不要**。`Mul` trait実装の拡張だけで、階層全体にbounds伝播が自動的に動作する。

---

### Requirement 5: エラーハンドリングとバリデーション

**Objective:** システム開発者として、不正なサイズや変換を検出したい。これにより、デバッグが容易になる。

#### Acceptance Criteria

1. If `Arrangement.size.width`または`Arrangement.size.height`が負の値である時、wintfシステムは警告ログを出力しなければならない（shall）。

2. If `Arrangement.scale.x`または`Arrangement.scale.y`が0.0である時、wintfシステムは警告ログを出力しなければならない（shall）。

3. The wintfシステムは、デバッグビルド時に`D2D_RECT_F`の一貫性を検証しなければならない（shall）：`left <= right`かつ`top <= bottom`。デバッグアサーション用の`D2DRectExt::validate()`メソッドを提供する。

---

### Requirement 6: テストとドキュメント

**Objective:** システム開発者として、バウンディングボックスシステムの動作を検証したい。これにより、将来の変更で機能が壊れないことを保証できる。

#### Acceptance Criteria

1. The wintfシステムは、`Arrangement.local_bounds()`のユニットテストを提供しなければならない（shall）。

2. The wintfシステムは、`transform_rect_axis_aligned`関数のユニットテストを提供しなければならない（shall）。以下のケースをカバーする：
   - 恒等変換
   - 平行移動のみ
   - スケールのみ
   - 平行移動とスケールの組み合わせ

3. The wintfシステムは、`D2DRectExt::union`メソッドのユニットテストを提供しなければならない（shall）。

4. The wintfシステムは、階層的バウンディングボックス計算の統合テストを提供しなければならない（shall）。親子3階層のWidgetツリーで、最終的な`GlobalArrangement.bounds`が正しいことを検証する。

5. The wintfシステムは、`crates/wintf/src/ecs/layout.rs`に以下のドキュメントコメントを追加しなければならない（shall）：
   - `Arrangement.size`とtaffyレイアウト計算の関係
   - `GlobalArrangement.bounds`の座標系とSurface生成との関連
   - 軸平行変換のみサポートすることの説明

---

## Out of Scope

以下の機能は本要件の対象外：

- **ContentSizeコンポーネント**: レイアウト入力情報。taffyレイアウトエンジン統合仕様で実装
- **Arrangement.sizeの自動設定**: taffyレイアウト出力から設定するロジックは別仕様で実装
- **回転・スキュー変換**: DirectComposition Visual層で将来実装
- **Transform層の実装**: 視覚効果用のTransformコンポーネントは別仕様で実装
- **Surface生成最適化本体**: 本要件は前提条件の整備のみ。実際のSurface生成ロジックは`surface-allocation-optimization`仕様で実装
- **taffyレイアウトエンジン統合**: `Arrangement.size`への出力設定は別仕様
- **ヒットテストシステム**: `GlobalArrangement.bounds`はヒットテストで利用されるが、ヒットテストシステム自体は別仕様

### 子孫Boundsの集約（旧Requirement 5）

**スコープ外の理由**: Surface生成最適化の一部であり、座標変換システムとは責務が異なる

**依存関係**: `GlobalArrangement.bounds`が実装されていること（本仕様で実装）

**実装予定**: `surface-allocation-optimization`仕様で実装

**技術要件**:
- 親の`GlobalArrangement.transform`逆行列計算
- 各子の`GlobalArrangement.bounds`を親のローカル座標系に変換
- すべての子孫boundsをunionして親のローカルboundsに統合
- `SurfaceGraphics`を持つサブツリーのスキップ処理

---

## Final Success Criteria

1. ✅ Arrangement.sizeフィールド（独自Size型）が追加され、Rect型エイリアス（D2D_RECT_F）とD2DRectExt拡張トレイトが定義される
2. ✅ GlobalArrangement.boundsフィールド（D2D_RECT_F）が追加され、trait実装（Mul, From）で計算される
3. ✅ 既存の`propagate_parent_transforms`システムが変更なしでbounds伝播を実行する
4. ✅ 軸平行変換専用の最適化された矩形変換関数（2点変換）が実装される
5. ✅ テストコードで`Arrangement.size`を明示的に設定してバウンディングボックス計算の正確性が検証される
6. ✅ ユニットテスト、統合テスト、パフォーマンステストが実装される

## Success Criteria Summary
