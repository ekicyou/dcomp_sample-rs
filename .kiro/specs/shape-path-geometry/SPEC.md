# Specification: shape-path-geometry

**Feature ID**: `shape-path-geometry`  
**Route**: AI粛々ルート（ルートB）  
**Created**: 2025-11-15  
**Status**: Phase 0 - Initialization

---

## 概要

WPF/WinUI3互換のPath Geometry構文（ミニ言語）を実装し、PathGeometryによる任意の図形描画を実現する。

### 目的
- SVG/XAML互換のPath Data構文パーサー実装
- ID2D1PathGeometry統合
- Pathウィジット実装

### スコープ

**含まれるもの**:
- Path Dataパーサー（M, L, C, Q, A, H, V, Z等）
- ID2D1PathGeometryラッパー
- Pathウィジット（PathData + Fill + Stroke）
- draw_pathsシステム

**含まれないもの**:
- Brush拡張（shape-brush-systemで実装）
- Stroke詳細（shape-stroke-widgetsで実装）
- アニメーション

### 技術スタック
- **パーサー**: nom（パーサーコンビネータ）
- **COM API**: ID2D1PathGeometry, ID2D1GeometrySink
- **参考実装**: WPF PathGeometry, SVG Path

---

## Path Data構文仕様

### サポートするコマンド

#### 基本コマンド
- `M x,y` / `m dx,dy`: MoveTo（絶対/相対）
- `L x,y` / `l dx,dy`: LineTo（絶対/相対）
- `H x` / `h dx`: Horizontal LineTo
- `V y` / `v dy`: Vertical LineTo
- `Z` / `z`: ClosePath

#### 曲線コマンド
- `C x1,y1 x2,y2 x,y`: Cubic Bezier（絶対）
- `c dx1,dy1 dx2,dy2 dx,dy`: Cubic Bezier（相対）
- `Q x1,y1 x,y`: Quadratic Bezier（絶対）
- `q dx1,dy1 dx,dy`: Quadratic Bezier（相対）
- `S x2,y2 x,y`: Smooth Cubic Bezier
- `T x,y`: Smooth Quadratic Bezier

#### 円弧コマンド
- `A rx,ry rotation large-arc sweep x,y`: Arc（絶対）
- `a rx,ry rotation large-arc sweep dx,dy`: Arc（相対）

### 構文例

```
// 三角形
"M 10,10 L 100,10 L 55,90 Z"

// 曲線
"M 10,10 C 20,20 40,20 50,10"

// 複雑な図形
"M 100,200 C 100,100 250,100 250,200 S 400,300 400,200"

// 円弧
"M 50,50 A 25,25 0 1,0 50,100"
```

---

## 次のステップ

Phase 0完了後、要件定義フェーズに進みます：

```bash
/kiro-spec-requirements shape-path-geometry
```

---

_Specification initialized on 2025-11-15_
