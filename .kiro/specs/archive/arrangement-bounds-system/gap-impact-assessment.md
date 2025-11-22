# Gap Impact Assessment: Requirements Review

**Generated**: 2025-11-21  
**Purpose**: ギャップ分析結果を基に、要件定義への影響を評価し、必要な修正を特定する

---

## Executive Summary

ギャップ分析の結果、**要件定義は概ね実装可能だが、以下3点について修正・明確化が必要**と判断される：

1. **Requirement 5（子孫Bounds集約）のスコープ明確化**: Out of Scopeに移動すべき
2. **Requirement 4（計算システム）の実装戦略修正**: 新規システムではなく既存システム拡張
3. **依存関係例外の文書化強化**: `com → ecs`参照の正当性をより明確に説明

---

## 1. Critical Issues（要件修正必須）

### Issue 1.1: Requirement 5 - 子孫Bounds集約のスコープ逸脱

#### 問題
Requirement 5「子孫Boundsの集約」は、以下の理由で本仕様のスコープ外と判断される：

- **Out of Scopeに既に記載**: "Surface生成最適化本体: 本要件は前提条件の整備のみ。実際のSurface生成ロジックは`surface-allocation-optimization`仕様で実装"
- **逆行列計算が必要**: `windows_numerics::Matrix3x2`の逆行列メソッド調査が必要（ギャップ分析2.2節の🔍Research Needed）
- **Surface最適化の一部**: 子孫boundsの集約は、Surface生成サイズ決定のロジックであり、座標変換システムとは責務が異なる

#### 影響範囲
- Requirement 5全体（Acceptance Criteria 1-4）
- Requirement 6.3（`aggregate_child_bounds`最適化）
- Requirement 7.3（逆行列エラーハンドリング）

#### 推奨修正
**Requirement 5をOut of Scopeに移動し、以下の説明を追加**：

```markdown
## Out of Scope

### 子孫Boundsの集約（Requirement 5から移動）
- **理由**: Surface生成最適化の一部であり、座標変換システムとは責務が異なる
- **依存**: `GlobalArrangement.bounds`が実装されていること（本仕様で実装）
- **実装予定**: `surface-allocation-optimization`仕様で実装
- **技術要件**: 親の`GlobalArrangement.transform`逆行列計算、子孫bounds統合ロジック
```

**Requirement 4-8の番号を繰り上げ**（Requirement 5削除により）

---

### Issue 1.2: Requirement 4 - 計算システムの過大見積もり

#### 問題（重要な洞察）
Requirement 4では新規システム（`compute_local_bounds`, `propagate_global_bounds`）を要求しているが、**実際には既存のtrait実装を拡張するだけで実現可能**。

**既存の仕組み**：
- `propagate_parent_transforms<L, G, M>`は既に`G: Mul<L, Output = G>`で動作
- `impl Mul<Arrangement> for GlobalArrangement`が既に存在
- 変更検知（`ArrangementTreeChanged`）も既に機能

**必要な変更**：
1. `GlobalArrangement`に`bounds`フィールド追加
2. `impl Mul<Arrangement> for GlobalArrangement`で`bounds`計算追加
3. `impl From<Arrangement> for GlobalArrangement`で初期`bounds`設定

**不要な作業**：
- ❌ 新規システム実装（`compute_local_bounds`, `propagate_global_bounds`）
- ❌ `propagate_parent_transforms`の拡張
- ✅ 既存のtrait実装に1-2行の`bounds`計算を追加するだけ

#### 影響範囲
- Requirement 4全体（Acceptance Criteria 1-6）
- 実装工数: M（3-7日）→ **S（1-2日）に削減可能**

#### 推奨修正
**Requirement 4のAcceptance Criteriaを以下のように簡略化**：

```markdown
#### Acceptance Criteria

1. The `GlobalArrangement`の`Mul<Arrangement>`実装は、`transform`と`bounds`の両方を計算しなければならない（shall）。

2. When `parent * child`が計算される時、以下を実行しなければならない（shall）：
   a. `transform` = 親.transform × 子.Arrangement変換行列
   b. `bounds` = transform_rect_axis_aligned（子.local_bounds(), 結果のtransform）

3. The `From<Arrangement>`実装は、初期`GlobalArrangement`の`bounds`を`Arrangement.local_bounds()`から設定しなければならない（shall）。

4. The wintfシステムは、`transform_rect_axis_aligned(rect: &D2D_RECT_F, matrix: &Matrix3x2) -> D2D_RECT_F`ヘルパー関数を提供しなければならない（shall）。この関数は2点変換（左上と右下）で軸平行矩形を変換する。

**Note**: 既存の`propagate_parent_transforms`システムは変更不要。`Mul` trait実装だけで自動的にbounds伝播が動作する。
```

**Requirement 4の旧Acceptance Criteria 5-6を削除**（既存システムが既に実装済み）

6. The `transform_rect_axis_aligned`関数は、4点変換ではなく2点変換（左上と右下のみ）を使用しなければならない（shall）。軸平行変換では2点で十分である。
```

**Requirement 4の旧Acceptance Criteria 1-2を削除**（`compute_local_bounds`システム、ローカルbounds計算式は`Arrangement.local_bounds()`メソッドに統合）

---

## 2. Moderate Issues（要件明確化推奨）

### Issue 2.1: 依存関係例外の正当性説明不足

#### 問題
Requirement 2のNoteで`com → ecs`依存例外を説明しているが、ギャップ分析で明らかになった**実装上の凝集性とトレードオフ**が要件に反映されていない。

現在の説明：
> **依存ルール例外**: `com`モジュールから`ecs`モジュールのComponent型（データ構造）の参照のみ許可。関数・システムの呼び出しは禁止。

#### 影響範囲
- Requirement 2（D2DRectExt実装）
- アーキテクチャ理解の明確性

#### 推奨修正
**Requirement 2のNoteに以下の説明を追加**：

```markdown
**Note:** 
- 既存コードで`D2D_RECT_F`を直接使用している箇所（`Rectangle`の描画等）との互換性を保つため、型エイリアスと拡張トレイトのパターンを採用。これは既存の`Color`型（`D2D1_COLOR_F`の型エイリアス）と同じアプローチである。
- `D2DRectExt::from_offset_size()`は、`ecs/layout`モジュールの`Size`と`Offset`型を参照する。

**依存ルール例外の正当性**:
- **例外**: `com`モジュールから`ecs`モジュールのComponent型（データ構造のみ）の参照を許可
- **制約**: 関数・システムの呼び出しは禁止（依存方向の逆転防止）
- **理由**: `D2D_RECT_F`はDirect2D APIの基盤型であり、`Size`/`Offset`は純粋なデータ構造（f32のペア）。実装の凝集性（D2D関連APIを`com/d2d/mod.rs`に集約）と実用性のトレードオフとして、データ型参照のみを許可する。
- **影響範囲**: `D2DRectExt`実装のみ。他の`com`モジュールには影響しない。
```

---

### Issue 2.2: パフォーマンス検証方法の未定義

#### 問題
Requirement 6.4で"10階層、100エンティティのWidgetツリーで16.67ms以内"を要求しているが、検証方法が定義されていない（ギャップ分析2.2節の🔍Research Needed）。

#### 影響範囲
- Requirement 8（テストとドキュメント）- パフォーマンステストが含まれていない

#### 推奨修正
**Requirement 8に以下のAcceptance Criteriaを追加**：

```markdown
6. The wintfシステムは、バウンディングボックス計算のパフォーマンステストを提供しなければならない（shall）。10階層、100エンティティのWidgetツリーを構築し、1フレーム分の計算時間を測定する。

7. The パフォーマンステストは、`tests/arrangement_performance_test.rs`に配置し、`#[ignore]`属性を付けて通常のテスト実行から除外しなければならない（shall）。ベンチマーク実行時のみ実行する。
```

---

## 3. Minor Issues（文書品質向上）

### Issue 3.1: Requirement 1.6 - local_bounds()の実装詳細

#### 問題
`Arrangement::local_bounds()`の実装詳細が要件に記載されているが、これは設計レベルの情報である。要件では"何を提供するか"のみを記述し、"どう実装するか"は設計フェーズで決定すべき。

現在の記述：
> このメソッドは、`D2DRectExt::from_offset_size()`を使用して`offset`と`size`から軸平行バウンディングボックス（`D2D_RECT_F`）を計算する。

#### 推奨修正
**Requirement 1.6を以下に修正**：

```markdown
6. The wintfシステムは、`Arrangement`の`local_bounds() -> D2D_RECT_F`メソッドを提供しなければならない（shall）。このメソッドは、`offset`と`size`から軸平行バウンディングボックス（`D2D_RECT_F`）を返す。
```

**Note**: 実装詳細（`D2DRectExt::from_offset_size()`使用）は設計フェーズで決定。

---

### Issue 3.2: Requirement 4.2 - ローカルbounds計算式の重複

#### 問題
Requirement 4.2でローカルbounds計算式を定義しているが、これは`Arrangement::local_bounds()`メソッドの実装詳細であり、Requirement 1と重複している。

#### 推奨修正
**Issue 1.2の修正（Requirement 4再構成）で解決**。`Arrangement.local_bounds()`メソッド呼び出しに統合。

---

## 4. Implementation Complexity Re-evaluation

### 当初の見積もり（ギャップ分析）
- **工数**: M（3-7日）
- **主な作業**: 既存システム拡張 + bounds計算ロジック統合 + 既存コード移行

### 修正後の見積もり
- **工数**: **S（1-2日）**
- **主な作業**:
  1. `Size`構造体定義（0.5時間）
  2. `Rect`型エイリアス + `D2DRectExt`実装（2時間）
  3. `GlobalArrangement`に`bounds`フィールド追加（0.5時間）
  4. `Mul<Arrangement>`実装に`bounds`計算追加（1時間）
  5. `From<Arrangement>`実装に`bounds`設定追加（0.5時間）
  6. `transform_rect_axis_aligned`ヘルパー関数（1時間）
  7. 既存コード移行（`Arrangement`に`size`追加）（2-3時間）
  8. ユニットテスト（2時間）

**合計**: 約10時間（1-2日）

### 実装の核心
既存の`propagate_parent_transforms`は変更不要。trait実装（`Mul`, `From`）に数行追加するだけ：

```rust
// 既存: impl Mul<Arrangement> for GlobalArrangement
fn mul(self, rhs: Arrangement) -> Self::Output {
    let child_matrix: Matrix3x2 = rhs.into();
    let child_bounds = rhs.local_bounds();
    let result_transform = self.0 * child_matrix;
    let result_bounds = transform_rect_axis_aligned(&child_bounds, &result_transform);
    GlobalArrangement { 
        transform: result_transform, 
        bounds: result_bounds 
    }
}
```

### Research Items Confirmation

### ✅ 調査完了
- **D2D_RECT_F型の存在**: 確認済み（`Rectangle::draw`で使用）
- **Matrix3x2型の存在**: 確認済み（`windows_numerics`クレート）
- **propagate_parent_transformsの再利用性**: 確認済み（**完全に再利用可能、変更不要**）
- **Mul trait活用**: 確認済み（bounds計算を追加するだけ）

### 🔍 調査中（設計フェーズで実施）
- **Matrix3x2の逆行列メソッド**: `windows_numerics::Matrix3x2`のドキュメント調査
  - **影響**: Requirement 5（Out of Scopeに移動予定）のため、本仕様では調査不要
- **パフォーマンス検証方法**: bevy_ecsベンチマークツール調査
  - **影響**: Requirement 8にテスト要件追加（Issue 2.2）

---

## 5. Requirements Revision Summary

### 必須修正（Critical）
1. **Requirement 5削除**: Out of Scopeに移動、理由を明記
2. **Requirement 4簡略化**: trait実装拡張のみ（新規システム不要）
3. **Requirement番号繰り上げ**: 旧Requirement 6-8 → 新Requirement 5-7
4. **工数見積もり修正**: M（3-7日）→ **S（1-2日）**

### 推奨修正（Moderate）
5. **Requirement 2 Note拡充**: 依存例外の正当性説明追加
6. **Requirement 8拡充**: パフォーマンステスト要件追加（Acceptance Criteria 6-7）

### 任意修正（Minor）
7. **Requirement 1.6簡略化**: 実装詳細を設計フェーズに委譲
8. **Requirement 4.2削除**: Arrangement.local_bounds()に統合（Issue 1.2で解決）

---

## 6. Revised Requirements Structure (Proposal)

修正後の要件構造提案：

```
Requirement 1: Arrangementコンポーネントへのサイズフィールド追加
  - AC 1-7（変更なし、AC 6を簡略化）

Requirement 2: Rect型エイリアスと拡張トレイト
  - AC 1-3（変更なし）
  - Note拡充（依存例外の正当性追加）

Requirement 3: GlobalArrangementへのBoundsフィールド追加
  - AC 1-5（変更なし）

Requirement 4: バウンディングボックス計算システム（旧Requirement 4, 6から統合）
  - AC 1-6（新規構成: 既存システム拡張 + 最適化要件統合）

Requirement 5: エラーハンドリングとバリデーション（旧Requirement 7）
  - AC 1-4（変更なし）
  - AC 3削除（逆行列エラーは別仕様）

Requirement 6: テストとドキュメント（旧Requirement 8）
  - AC 1-5（変更なし）
  - AC 6-7追加（パフォーマンステスト）

Out of Scope追加:
  - 子孫Boundsの集約（旧Requirement 5）
```

---

## 7. Impact on Design Phase

修正後の要件定義が設計フェーズに与える影響：

### 設計大幅簡略化（重要）
- **新規システム不要**: `compute_local_bounds`, `propagate_global_bounds`システムの設計が不要
- **既存システム変更不要**: `propagate_parent_transforms`は完全に再利用可能
- **trait実装のみ**: `Mul<Arrangement>`と`From<Arrangement>`にbounds計算を追加（各1-2行）

### 設計作業の焦点
- **データ構造**: `Size`, `GlobalArrangement { transform, bounds }`定義
- **ヘルパー関数**: `transform_rect_axis_aligned`（2点変換ロジック）
- **拡張トレイト**: `D2DRectExt`（12メソッド）

### 設計フェーズでの調査項目（最小限）
- `windows_numerics::Matrix3x2`のAPIドキュメント確認（点変換メソッド）
- bevy_ecsパフォーマンステストパターン調査（Requirement 6.6-7実装）

### 実装リスクの低減
- ✅ 既存システムの動作保証（変更なしのため）
- ✅ テスト範囲の縮小（新規システムのテスト不要）
- ✅ バグ混入リスクの低減（変更箇所が明確かつ小規模）

---

## 8. Recommendation

### 要件承認前の推奨アクション

#### 優先度: High（承認前に必須）
1. **Requirement 5削除**: Out of Scopeに移動、surface-allocation-optimization仕様で実装予定と明記
2. **Requirement 4再構成**: 既存システム拡張アプローチに変更、新規システム削除

#### 優先度: Medium（設計前に推奨）
3. **Requirement 2 Note拡充**: 依存例外の正当性を明確化
4. **Requirement 8拡充**: パフォーマンステスト要件追加

#### 優先度: Low（文書品質向上）
5. **Requirement 1.6簡略化**: 実装詳細を設計に委譲

### 修正後の承認フロー
1. **要件修正**: 上記Critical修正を反映（Requirement 4, 5）
2. **レビュー**: 修正内容の妥当性確認
3. **承認**: `/kiro-spec-design arrangement-bounds-system`で設計フェーズへ

---

## Conclusion

ギャップ分析により、**要件定義は実装可能だが、実装規模を過大評価していた**と判断される。主な修正点：

- ✅ **スコープ明確化**: Requirement 5（子孫bounds集約）をOut of Scopeに移動
- ✅ **実装戦略の現実化**: Requirement 4を既存trait実装拡張に簡略化（新規システム不要）
- ✅ **工数見積もり修正**: M（3-7日）→ **S（1-2日）**
- ✅ **文書品質**: 依存例外の正当性、パフォーマンステスト要件を明確化

### 修正後の評価
- **規模**: **Small（1-2日、約10時間）**
- **リスク**: **Low**（既存システム変更なし、trait実装に数行追加のみ）
- **複雑性**: **低**（データ構造追加 + trait実装拡張 + ヘルパー関数）

### 実装の本質
既存の`propagate_parent_transforms`は完全に再利用可能。`impl Mul<Arrangement> for GlobalArrangement`と`impl From<Arrangement> for GlobalArrangement`に`bounds`計算を追加するだけで、階層伝播システム全体が自動的に`bounds`を計算する。

---

_Gap impact assessment completed. Implementation is simpler than initially estimated. Ready for requirements revision._
