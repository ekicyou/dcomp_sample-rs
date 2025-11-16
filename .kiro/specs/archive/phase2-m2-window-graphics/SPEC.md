# Specification: Phase 2 Milestone 2 - WindowGraphics + Visual作成

**Feature ID**: `phase2-m2-window-graphics`  
**Created**: 2025-11-14  
**Status**: Phase 0 - Initialization

---

## 📋 Overview

ウィンドウ単位のグラフィックスリソース（WindowGraphics）とルートVisualを作成。CompositionTargetをhwndに紐付け、Visualをルートとして設定する。

**位置づけ**: Phase 2の2番目のマイルストーン（〇2）

---

## 🎯 Purpose

各ウィンドウに独立したグラフィックスリソースを提供し、DirectCompositionのVisualツリーのルートを確立する。

---

## 📊 Scope

### 含まれるもの
- `WindowGraphics`構造体の定義
  - `composition_target: IDCompositionTarget`
  - `device_context: ID2D1DeviceContext`
- `Visual`コンポーネントの定義（WindowエンティティにアタッチOK）
- `create_window_graphics`システム
- `create_window_visual`システム
- CompositionTargetのhwnd紐付け
- VisualをTargetのルートとして設定（`SetRoot`）

### 含まれないもの
- 描画処理（Milestone 3で実装）
- 子Visual管理（Milestone 4で実装）
- Surface作成（Milestone 3で実装）

---

## ✅ Success Criteria

- ✅ ウィンドウごとに`WindowGraphics`が存在
- ✅ ウィンドウに`Visual`コンポーネントが存在
- ✅ VisualがTargetに設定済み（`SetRoot`完了）
- ✅ `Query<(&WindowHandle, &WindowGraphics, &Visual)>`で取得可能

---

## 📝 Implementation Elements

- `WindowGraphics`構造体
- `Visual`構造体（コンポーネント）
- `create_window_graphics`システム
- `create_window_visual`システム
- COM APIラッパー:
  - `IDCompositionTarget::SetRoot`
  - `IDCompositionDevice::CreateTargetForHwnd`
  - `IDCompositionDevice::CreateVisual`

---

## 🔄 Dependencies

### 前提条件
- Phase 1完了（WindowHandleコンポーネント）

### 依存するマイルストーン
- ✅ **Milestone 1完了が必須**: `phase2-m1-graphics-core` (GraphicsCore初期化)

---

## ➡️ Next Milestone

このマイルストーン完了後:

```bash
/kiro-spec-requirements phase2-m3-first-rendering
```

**次**: `phase2-m3-first-rendering` - 初めての描画（●■▲）

---

## 📚 References

- `.kiro/specs/brainstorming-next-features/MILESTONES.md` - マイルストーン全体像
- `.kiro/specs/phase2-m1-graphics-core/` - 前提となるMilestone 1

---

## 🔄 Next Steps

```bash
/kiro-spec-requirements phase2-m2-window-graphics
```

---

_Phase 0 (Initialization) completed. Ready for requirements phase._
