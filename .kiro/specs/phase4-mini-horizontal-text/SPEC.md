# Specification: phase4-mini-horizontal-text

**Feature ID**: `phase4-mini-horizontal-text`  
**Route**: モチベーションGO!ルート（ルートA）  
**Created**: 2025-11-15  
**Status**: Phase 0 - Initialization

---

## 概要

DirectWriteを統合し、横書きテキストレンダリングの最小実装を行う。縦書き実装への第一歩。

### 目的
- DirectWrite統合（最小限）
- 横書きテキスト表示
- Labelウィジット基本実装

### スコープ

**含まれるもの**:
- DirectWrite基本統合
  - IDWriteTextFormat作成
  - IDWriteTextLayout作成
- 横書きテキストレンダリング
  - DrawTextLayout
- Labelウィジット（Text + Position + Color）
- draw_labelsシステム

**含まれないもの**:
- Button（Phase 7後に実装）
- 複雑なレイアウト（折り返し等は最小限）
- イベント処理
- 縦書き（Phase 7で実装）

### 技術スタック
- **COM API**: IDWriteFactory7, IDWriteTextFormat, IDWriteTextLayout
- **参考実装**: Phase 2のDirectWrite初期化（既存）

---

## アーキテクチャ

### モジュール構成

\\\
crates/wintf/src/
├── com/
│   └── dwrite.rs          # DirectWrite拡張
│
└── ecs/
    └── widget/
        └── text/
            ├── mod.rs
            ├── label.rs       # Labelウィジット
            └── draw_labels.rs # draw_labelsシステム
\\\

### コンポーネント設計

#### Labelウィジット
\\\
ust
#[derive(Component)]
pub struct Label {
    pub text: String,
    pub font_family: String,    // "メイリオ"
    pub font_size: f32,         // 16.0
    pub color: D2D1_COLOR_F,
    pub x: f32,
    pub y: f32,
}
\\\

#### TextLayout（キャッシュ）
\\\
ust
#[derive(Component)]
pub struct TextLayout {
    layout: IDWriteTextLayout,
}
\\\

---

## 実装フェーズ

### Phase 1: DirectWrite COM API拡張
1. IDWriteTextFormat作成
2. IDWriteTextLayout作成
3. テスト

### Phase 2: Labelウィジット実装
1. Labelコンポーネント定義
2. TextLayoutコンポーネント定義
3. draw_labelsシステム実装
   - Changed<Label>検知
   - TextLayout生成・キャッシュ
   - DrawTextLayout
4. サンプル作成

---

## 成功基準

- ✅ \"Hello, World!\"が表示される
- ✅ 日本語（\"こんにちは\"）が表示される
- ✅ フォント・サイズ・色が指定できる
- ✅ 複数のLabelが同時表示可能
- ✅ 60fps以上のパフォーマンスを維持（Vsync同期環境）

---

## 次のステップ

\\\ash
/kiro-spec-requirements phase4-mini-horizontal-text
\\\

---

_Specification initialized on 2025-11-15_
