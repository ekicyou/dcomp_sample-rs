# Dual Route Strategy: モチベーションGO! vs AI粛々

**Strategy Date**: 2025-11-15  
**Purpose**: 縦書き最速到達 + Shape基盤充実

---

## 戦略概要

Phase 2完了後、2つのルートを並行して進める：

### 🔥 ルートA: モチベーションGO!ルート
**担当**: 人間主導・AI支援  
**目標**: 縦書きテキスト最速到達  
**期間**: 2-3週間

### 🤖 ルートB: AI粛々ルート
**担当**: AI主導  
**目標**: Shape/Path基盤の充実  
**期間**: 2-4週間（3つ並行）

---

## ルートA: モチベーションGO!

### Milestone A1: 横書きテキスト最小実装
**Feature ID**: `phase4-mini-horizontal-text`  
**期間**: 1-2週間  
**担当**: 人間主導・AI支援

#### スコープ
- DirectWrite統合（最小限）
- TextFormat, TextLayout
- DrawTextLayout
- Labelウィジット（基本のみ）
- **除外**: Button, 複雑なレイアウト、イベント処理

#### 成功基準
- ✅ "Hello, World!"が表示される
- ✅ フォント・サイズ・色が指定できる
- ✅ 複数のLabelが同時表示可能

---

### Milestone A2: 縦書きテキスト実装
**Feature ID**: `phase7-vertical-text`  
**期間**: 1-2週間  
**担当**: 人間主導

#### スコープ
- SetReadingDirection(TOP_TO_BOTTOM)
- SetFlowDirection(RIGHT_TO_LEFT)
- 句読点・括弧の回転処理
- VerticalLabelウィジット
- 縦書きサンプルアプリ

#### 成功基準
- ✅ 縦書きで日本語が表示される
- ✅ 句読点が正しい位置に表示される
- ✅ 右から左への行送りが機能する

---

## ルートB: AI粛々ルート

### Milestone B1: PathGeometry + パーサー
**Feature ID**: `shape-path-geometry`  
**期間**: 2週間  
**担当**: AI主導

#### スコープ
- PathGeometry構文パーサー
  - M (MoveTo), L (LineTo), C (CurveTo), Z (Close)
  - H, V (Horizontal/Vertical)
  - A (Arc)
- ID2D1PathGeometry統合
- Pathウィジット基本実装

#### 成功基準
- ✅ `"M 10,10 L 100,100 Z"`が描画される
- ✅ 曲線（Bezier）が描画される
- ✅ WPF/WinUI3互換の構文

---

### Milestone B2: Brush拡張
**Feature ID**: `shape-brush-system`  
**期間**: 2週間  
**担当**: AI主導

#### スコープ
- LinearGradientBrush
- RadialGradientBrush
- GradientStopコレクション
- Brushコンポーネント設計

#### 成功基準
- ✅ 線形グラデーションが表示される
- ✅ 放射グラデーションが表示される
- ✅ 複数のGradientStopが機能する

---

### Milestone B3: Stroke + Shapeウィジット群
**Feature ID**: `shape-stroke-widgets`  
**期間**: 2週間  
**担当**: AI主導

#### スコープ
- StrokeWidth, StrokeDashArray
- StrokeDashCap, StrokeLineJoin
- Ellipseウィジット
- Polygonウィジット
- Polylineウィジット

#### 成功基準
- ✅ 点線・破線が描画される
- ✅ 楕円・円が描画される
- ✅ 多角形が描画される

---

## 実装スケジュール

### Week 1-2
```
[ルートA] Milestone A1: 横書きテキスト最小実装
[ルートB] Milestone B1: PathGeometry並行実施
[ルートB] Milestone B2: Brush並行実施（後半）
```

### Week 3-4
```
[ルートA] Milestone A2: 縦書きテキスト実装 🎉
[ルートB] Milestone B3: Stroke + Widgets並行実施
```

---

## Kiro Spec作成コマンド

### ルートA（順次実行）
```bash
# Step 1
/kiro-spec-init "phase4-mini-horizontal-text"

# Step 2（Step 1完了後）
/kiro-spec-init "phase7-vertical-text"
```

### ルートB（並行実行可能）
```bash
# 並行1
/kiro-spec-init "shape-path-geometry"

# 並行2
/kiro-spec-init "shape-brush-system"

# 並行3（並行1-2完了後）
/kiro-spec-init "shape-stroke-widgets"
```

---

## Phase 3（透過ウィンドウ）の扱い

縦書き達成後、以下のオプション：

### Option 1: AI並行タスクとして追加
```bash
/kiro-spec-init "phase3-transparent-window-hittest"
```
- 縦書き実装中にAIが並行処理
- 縦書き完成時に統合

### Option 2: 縦書き完成後に実施
- モチベーション維持のため後回し
- Shape充実後に実装

---

## まとめ

- **ルートA**: 2-3週間で縦書き到達 🎉
- **ルートB**: AI並行でShape基盤充実
- **並行処理**: 2-3個同時進行
- **人間の役割**: ルートAの動作確認・調整
- **AIの役割**: ルートBの粛々実装

---

_Dual Route Strategy defined on 2025-11-15_
