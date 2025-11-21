# 将来要件調査報告書

**Feature ID**: `future-requirements-survey`  
**調査日**: 2025-11-21  
**Status**: Survey Completed

---

## 調査概要

完了した仕様（アーカイブ済み）および進行中の仕様から、「スコープ外」と判断された要件を洗い出し、未着手の機能要素を特定した。本ドキュメントは実装仕様ではなく、将来の仕様策定のための調査記録である。

---

## 1. Visual階層の同期・DirectComposition階層的合成

### 出典
- `visual-tree-implementation` (アーカイブ済)

### 内容
DirectCompositionのVisual階層を活用した高度な描画最適化と部分更新機能。

#### 詳細要件
1. **IDCompositionVisualの親子関係構築**
   - `AddVisual` / `RemoveVisual` APIの統合
   - ECS階層（ChildOf/Children）からDirectComposition Visual階層への自動同期

2. **子WidgetへのVisual+Surface個別作成**
   - Window以外のWidget EntityにもVisual+Surfaceを作成
   - 条件: アニメーション存在、スクロール存在、その他階層でSurfaceを生成する理由がある場合
   
3. **Visual階層に基づく部分更新**
   - 変更されたVisualのみを更新する効率的な描画
   - DirectCompositionの階層的合成機能を活用した最適化
   
4. **ネストされたSurfaceの独立管理**
   - 親Surfaceと子Surfaceの独立した描画サイクル
   - アニメーション/スクロール時の子Widget独自Surface最適化

### 現状
- ✅ ECS階層（ChildOf/Children）は実装済
- ✅ Window EntityのみVisual+Surfaceを持つ単一Surface描画は実装済
- ❌ DirectComposition Visual階層構築は未実装
- ❌ 子Widget個別Visual+Surfaceは未実装

### 優先度
**高**: DirectCompositionの本来の性能を引き出すために必須

---

## 2. taffyレイアウトエンジン統合

### 出典
- `visual-tree-implementation` (アーカイブ済)

### 内容
自動レイアウトシステムの統合により、手動座標指定からの脱却。

#### 詳細要件
1. **BoxComputedLayout → Arrangement変換**
   - taffyの計算結果をArrangementコンポーネントに変換
   - DPI対応とスケーリング
   
2. **Flexbox レイアウトサポート**
   - justify-content, align-items等の標準プロパティ
   - flex-direction, flex-wrap
   
3. **Grid レイアウトサポート**
   - grid-template-columns/rows
   - grid-gap, grid-auto-flow
   
4. **レイアウト再計算の最適化**
   - 変更検知による部分再計算
   - レイアウトキャッシング

### 現状
- ✅ Arrangementコンポーネントは実装済（手動設定のみ）
- ❌ taffyエンジン統合は未実装
- ❌ 自動レイアウト計算は未実装

### 優先度
**中**: 実用的なUIアプリケーション構築に必要

---

## 3. Surface生成の最適化

### 出典
- `surface-allocation-optimization` (アクティブ、要件定義待ち)

### 内容
Surface生成の動的判定により、不要なメモリ消費を削減。

#### 詳細要件
1. **GraphicsCommandListの集約分析**
   - 自分と子孫のGraphicsCommandList有無を走査
   - 描画コマンドが存在しない場合はSurface生成をスキップ
   
2. **要求サイズの動的決定**
   - 実際の描画領域に基づくSurfaceサイズ計算
   - 固定サイズからの脱却
   
3. **Surface生成要否の判定ロジック**
   - 描画コマンドなし → Surface不要
   - 透明度100% → Surface不要（将来拡張）
   - サイズ0 → Surface不要

### 現状
- ✅ Visual/Surface自動作成は実装済（一律作成）
- ❌ 生成要否の動的判定は未実装
- ❌ サイズの動的決定は未実装

### 優先度
**中**: メモリ効率とパフォーマンス最適化

---

## 4. Visual階層の同期（ECS → DirectComposition）

### 出典
- `visual-tree-synchronization` (アクティブ、要件定義待ち)

### 内容
ECS階層変更を検知してDirectCompositionのVisualツリーを自動同期。

#### 詳細要件
1. **ChildOf変更の検知**
   - `Changed<ChildOf>` による親変更検知
   - 新規追加、親変更、削除の3パターン対応
   
2. **AddVisual / RemoveVisual の自動呼び出し**
   - 親Visual取得とAddVisual実行
   - 旧親からのRemoveVisual実行
   
3. **Visualツリーの整合性保証**
   - ECSとDirectCompositionの状態一致
   - エラーハンドリングと回復処理

### 現状
- ✅ ECS階層管理（ChildOf/Children）は実装済
- ❌ DirectComposition Visualツリー構築は未実装
- ❌ 階層変更の自動同期は未実装

### 優先度
**高**: 項目1の実装と密接に関連

---

## 5. Render Dirty Tracking の高度化

### 出典
- `render-dirty-tracking` (アクティブ、requirements.md空)
- `surface-render-optimization` (アーカイブ済、実装完了)

### 内容
変更検知による描画判定の基本実装は完了。さらなる最適化の可能性。

#### 詳細要件（検討事項）
1. **細粒度の変更検知**
   - コンポーネント単位での変更追跡
   - 影響範囲の最小化
   
2. **描画スキップの統計情報**
   - スキップ率の可視化
   - パフォーマンスメトリクス収集
   
3. **手動Dirty制御API**
   - アプリケーション開発者による明示的な再描画要求

### 現状
- ✅ 基本的な変更検知と描画スキップは実装済
- ❓ さらなる最適化の必要性は要検討

### 優先度
**低**: 基本機能は実装済、最適化は必要に応じて

---

## 6. Shape関連機能（AI粛々ルート）

### 出典
- `shape-brush-system` (SPEC.mdのみ)
- `shape-path-geometry` (SPEC.mdのみ)
- `shape-stroke-widgets` (SPEC.mdのみ)

### 内容
Shape描画システムの充実化。DUAL_ROUTE_STRATEGYのルートBとして計画済。

#### 6.1 Brush System
**Spec ID**: `shape-brush-system`

- LinearGradientBrush（開始点、終了点、GradientStop）
- RadialGradientBrush（中心、半径、GradientStop）
- GradientStopコレクション（位置・色）
- Brushコンポーネント（enum: Solid/LinearGradient/RadialGradient）
- ID2D1GradientBrush統合

#### 6.2 Path Geometry
**Spec ID**: `shape-path-geometry`

- Path Data構文パーサー（nom使用）
  - M (MoveTo), L (LineTo), H/V (Horizontal/Vertical)
  - C (Cubic Bezier), Q (Quadratic Bezier), S/T (Smooth)
  - A (Arc), Z (Close)
- ID2D1PathGeometry統合
- Pathウィジット実装
- WPF/WinUI3/SVG互換構文

#### 6.3 Stroke & Shape Widgets
**Spec ID**: `shape-stroke-widgets`

- Stroke詳細設定
  - StrokeWidth, StrokeDashArray
  - StrokeDashCap, StrokeLineJoin
- 基本Shapeウィジット群
  - Ellipse（楕円・円）
  - Polygon（多角形）
  - Polyline（連続線）

### 現状
- ✅ Rectangle実装済
- ✅ 基本的なSolid Brush実装済
- ❌ Gradient Brushは未実装
- ❌ PathGeometryは未実装
- ❌ Ellipse/Polygon/Polylineは未実装

### 優先度
**中**: DUAL_ROUTE_STRATEGY（ルートB）で並行実装予定

---

## 7. 透過ウィンドウ・ヒットテスト

### 出典
- `DUAL_ROUTE_STRATEGY.md` (phase3として言及)

### 内容
透過ウィンドウの実装とヒットテスト処理。

#### 詳細要件（推測）
1. **透過ウィンドウの作成**
   - WS_EX_LAYERED または DWM拡張による透過
   - アルファチャンネル対応
   
2. **ヒットテスト処理**
   - 透過領域のクリック貫通
   - WM_NCHITTESTハンドリング
   - 形状に基づくヒットテスト（将来）

### 現状
- ❌ 仕様ファイル自体が存在しない
- ❌ 要件定義未実施

### 優先度
**低～中**: 縦書き実装後の検討事項

---

## 8. Transform階層伝播の廃止（リファクタリング）

### 出典
- `visual-tree-implementation` (アーカイブ済、実装時のコメント)

### 内容
誤った設計として認識されているTransform階層伝播ロジックの削除。

#### 詳細要件
1. **transform.rsのGlobalTransform削除**
   - GlobalTransformコンポーネントの削除
   - TransformTreeChangedの削除
   
2. **Transformの役割明確化**
   - Transformは視覚効果のみ（回転、傾斜、スケール）
   - 累積伝播は行わない
   - レイアウトはArrangementが担当
   
3. **最終変換の計算式確立**
   - `final_transform = GlobalArrangement * Transform`
   - Arrangementが階層的座標、Transformが視覚効果

### 現状
- ✅ Arrangementシステムは実装済
- ❌ GlobalTransform等の誤った実装が残存
- ❌ 削除作業は未実施

### 優先度
**中**: 設計の整合性を保つために必要

---

## 9. Container Widget

### 出典
- `visual-tree-implementation` (アーカイブ済、「スコープ外」として明記)

### 内容
専用のContainer Widget実装。

#### 詳細要件
1. **Containerコンポーネント**
   - 背景色、境界線等のスタイリング
   - パディング、マージン設定
   
2. **レイアウトコンテナ**
   - StackPanel（縦/横積み）
   - Grid（グリッドレイアウト）
   - Canvas（絶対座標配置）
   
3. **taffyとの統合**
   - Flexboxコンテナ
   - Gridコンテナ

### 現状
- ✅ Rectangle/Labelでの階層実装は完了
- ❌ 専用Container Widgetは未実装

### 優先度
**低**: Rectangle/Labelで代替可能、taffy統合後に検討

---

## 10. その他の将来拡張（長期）

### 10.1 デバイスロスト対応
- 出典: `phase2-m3-first-rendering`
- GPU/ドライバーエラーからの自動復旧

### 10.2 アニメーションシステム
- 出典: 複数の仕様で「スコープ外」として言及
- プロパティアニメーション
- イージング関数
- タイムライン管理

### 10.3 イベント処理システム
- 出典: `phase4-mini-horizontal-text` (Out of Scope)
- クリック、ホバー等の入力イベント
- イベントバブリング/キャプチャリング

### 10.4 ImageBrush / 画像表示
- 出典: `shape-brush-system` (Out of Scope)
- WIC統合による画像読み込み
- ImageBrushによる画像Fill

### 10.5 テキスト編集機能
- 出典: `phase4-mini-horizontal-text` (Out of Scope)
- TextBox実装
- IME統合
- キャレット表示

### 10.6 リッチテキスト
- 出典: `phase4-mini-horizontal-text` (Out of Scope)
- 部分的な装飾（太字、斜体、色変更等）
- インラインオブジェクト埋め込み

---

## 優先度マトリクス

### 高優先度（重要かつ基盤的）
1. **Visual階層の同期・DirectComposition階層的合成** (項目1)
2. **Visual階層の同期（ECS → DirectComposition）** (項目4)

### 中優先度（実用性向上）
3. **taffyレイアウトエンジン統合** (項目2)
4. **Surface生成の最適化** (項目3)
5. **Shape関連機能** (項目6) - AI粛々ルートで並行実施
6. **Transform階層伝播の廃止** (項目8) - リファクタリング
7. **透過ウィンドウ・ヒットテスト** (項目7)

### 低優先度（最適化・拡張）
8. **Render Dirty Tracking の高度化** (項目5)
9. **Container Widget** (項目9)
10. **その他の将来拡張** (項目10)

---

## 実装ルート提案

### ルートA: モチベーションGO!（継続中）
- ✅ Phase 4: 横書きテキスト（完了）
- ✅ Phase 7: 縦書きテキスト（完了）
- 🔜 **次**: Visual階層統合 (項目1, 4)

### ルートB: AI粛々ルート（並行実施）
- 🔄 Shape関連機能 (項目6)
  - shape-brush-system
  - shape-path-geometry
  - shape-stroke-widgets

### ルートC: 基盤最適化（適宜）
- Transform階層廃止 (項目8)
- Surface生成最適化 (項目3)

---

## 次のアクション

各項目について仕様を作成する場合のコマンド:

```bash
# 高優先度
/kiro-spec-init "Visual階層の統合: DirectCompositionのVisual親子関係（AddVisual/RemoveVisual）構築と、子WidgetへのVisual+Surface個別作成による階層的合成最適化"

/kiro-spec-init "ECS階層変更の自動同期: ChildOf変更を検知してDirectCompositionのVisualツリー（AddVisual/RemoveVisual）を自動的に同期させる仕組み"

# 中優先度
/kiro-spec-init "taffyレイアウトエンジン統合: FlexboxとGridによる自動レイアウト計算とBoxComputedLayout→Arrangement変換"

/kiro-spec-init "Surface生成の最適化: GraphicsCommandListの有無や要求サイズを集約し、SurfaceGraphicsの生成要否やサイズを動的に決定する仕組み"

/kiro-spec-init "Transform階層伝播の廃止: GlobalTransformとTransformTreeChangedを削除し、Transformを視覚効果のみに限定するリファクタリング"

/kiro-spec-init "透過ウィンドウとヒットテスト: WS_EX_LAYEREDまたはDWM拡張による透過ウィンドウ実装と、WM_NCHITTESTによるヒットテスト処理"

# Shape関連（既に初期化済、要件定義待ち）
# - shape-brush-system
# - shape-path-geometry  
# - shape-stroke-widgets
```

---

## まとめ

- **完了済仕様**: 9件（アーカイブ済）
- **進行中仕様**: 6件（3件は要件定義待ち、3件はSPEC.mdのみ）
- **未着手の重要要件**: 10カテゴリ、40+個の詳細要件
- **最優先事項**: Visual階層統合（DirectComposition本来の性能活用）

この調査により、今後の開発ロードマップが明確化された。

---

_Survey completed on 2025-11-21_
