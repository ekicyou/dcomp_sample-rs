# Specification: Phase 2 Milestone 3 - 初めての描画（●■▲）

**Feature ID**: `phase2-m3-first-rendering`  
**Created**: 2025-11-14  
**Status**: Phase 0 - Initialization

---

## 📋 Overview

Windowエンティティに直接描画を実装。透明背景に赤い円●、緑の四角■、青い三角▲を描画し、DirectCompositionでCommitする。

**位置づけ**: Phase 2の3番目のマイルストーン（△） - **初めて視覚的な結果が見える！**

---

## 🎯 Purpose

DirectComposition + Direct2Dの描画パイプライン全体を動作させ、ウィンドウに図形が表示されることを確認する。Phase 2の最重要マイルストーン。

---

## 📊 Scope

### 含まれるもの
- `Surface`コンポーネントの定義
- `WindowPos`構造体（ウィンドウサイズ管理）
- `create_window_surface`システム
- `render_window`システム（描画処理）
- `commit_composition`システム（毎フレーム最後）
- 描画内容:
  1. `Clear(transparent)` - 透明背景
  2. 赤い円 ● (`FillEllipse`)
  3. 緑の四角 ■ (`FillRectangle`)
  4. 青い三角 ▲ (`FillGeometry` + PathGeometry)
- ブラシ作成（red, green, blue）
- PathGeometry作成（三角形用）

### 含まれないもの
- 子Visual管理（Milestone 4で実装）
- デバイスロスト対応（将来の拡張）
- テキスト描画（Phase 4）

---

## ✅ Success Criteria

- ✅ **ウィンドウに透過背景で●■▲が表示される** 🎉
- ✅ デスクトップが透けて見える（透過動作確認）
- ✅ エラーなし
- ✅ フレームレート安定（60fps程度）

---

## 📝 Implementation Elements

- `Surface`構造体（コンポーネント）
- `WindowPos`構造体
- `create_window_surface`システム
- `render_window`システム
- `commit_composition`システム
- COM APIラッパー:
  - `ID2D1DeviceContext::BeginDraw/EndDraw`
  - `ID2D1DeviceContext::Clear`
  - `ID2D1DeviceContext::FillEllipse`
  - `ID2D1DeviceContext::FillRectangle`
  - `ID2D1DeviceContext::FillGeometry`
  - `ID2D1DeviceContext::CreateSolidColorBrush`
  - `ID2D1Factory::CreatePathGeometry`
  - `IDCompositionDevice::Commit`

---

## 🔄 Dependencies

- Milestone 1完了（GraphicsCore初期化）
- Milestone 2完了（WindowGraphics + Visual作成）

---

## 📚 References

- `.kiro/specs/brainstorming-next-features/MILESTONES.md` - マイルストーン全体像
- `.kiro/specs/phase2-m1-graphics-core/` - GraphicsCore
- `.kiro/specs/phase2-m2-window-graphics/` - WindowGraphics

---

## 🔄 Next Steps

```bash
/kiro-spec-requirements phase2-m3-first-rendering
```

---

_Phase 0 (Initialization) completed. Ready for requirements phase._
