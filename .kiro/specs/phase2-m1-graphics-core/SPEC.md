# Specification: Phase 2 Milestone 1 - GraphicsCore初期化

**Feature ID**: `phase2-m1-graphics-core`  
**Created**: 2025-11-14  
**Status**: Phase 0 - Initialization

---

## 📋 Overview

グローバルグラフィックスリソース（GraphicsCore）の初期化を実装。D3D11, D2D, DWrite, DirectCompositionのファクトリをProcessSingletonとして管理する。

**位置づけ**: Phase 2の最初のマイルストーン（〇1）

---

## 🎯 Purpose

Phase 2「はじめての描画」の基盤となるグローバルリソースを初期化する。すべての描画処理はこのGraphicsCoreから派生する。

---

## 📊 Scope

### 含まれるもの
- `GraphicsCore`構造体の定義
- D3D11Deviceの作成
- D2DFactoryの作成
- D2DDeviceの作成（D3D11Deviceから）
- DWriteFactoryの作成
- DCompDeviceの作成
- `ProcessSingleton`としての管理
- `initialize_graphics_core()`システム

### 含まれないもの
- ウィンドウ単位のリソース（Milestone 2で実装）
- 描画処理（Milestone 3で実装）
- 子要素管理（Milestone 4で実装）

---

## ✅ Success Criteria

- ✅ エラーなく初期化完了
- ✅ ログで各ファクトリの作成を確認
- ✅ `GraphicsCore`がProcessSingletonとして取得可能
- ✅ アプリケーション起動が成功

---

## 📝 Implementation Elements

- `GraphicsCore`構造体
- `initialize_graphics_core()`システム
- COM APIラッパー拡張:
  - `com/d3d11.rs` - D3D11Device作成
  - `com/d2d/` - D2DFactory, D2DDevice作成
  - `com/dwrite.rs` - DWriteFactory作成
  - `com/dcomp.rs` - DCompDevice作成

---

## 🔄 Dependencies

- Phase 1完了（ウィンドウシステム）
- COM APIラッパーの基礎実装

---

## 📚 References

- `.kiro/specs/brainstorming-next-features/MILESTONES.md` - マイルストーン全体像
- `.kiro/steering/tech.md` - 技術スタック

---

## 🔄 Next Steps

```bash
/kiro-spec-requirements phase2-m1-graphics-core
```

---

_Phase 0 (Initialization) completed. Ready for requirements phase._
