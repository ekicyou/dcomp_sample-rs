# Specification: Phase 2 Milestone 4 - 初めてのウィジット

**Feature ID**: `phase2-m4-first-widget`  
**Created**: 2025-11-14  
**Status**: Phase 0 - Initialization

---

## 📋 Overview

子エンティティの作成とVisualツリー管理を実装。親子関係（ChildOf）を使い、子VisualをWindowの親Visualに追加し、階層的な描画を実現する。

**位置づけ**: Phase 2の4番目のマイルストーン（◇） - **Visualツリーの基盤完成**

---

## 🎯 Purpose

Windowエンティティだけでなく、任意の子エンティティでもVisual + Surfaceによる描画が可能になることを実証する。将来の複雑なウィジット階層の基盤を構築する。

---

## 📊 Scope

### 含まれるもの
- 子エンティティの作成とVisualツリー管理
- `ChildOf`リレーションを使った親子関係
- 子エンティティに`Visual`コンポーネント追加
- 親Visual（Window）に子Visualを追加（`AddVisual`）
- 子Visualにも`Surface`を作成して描画
- Visualツリーの階層的な更新・削除
- `tree_system`との統合（Transform伝播）
- 描画内容:
  - Window: 薄い灰色背景
  - 子ウィジット1: 小さな赤い円（中央）
  - 子ウィジット2: 小さな青い四角（右下）

### 含まれないもの
- レイアウトシステム（将来の拡張）
- 複雑なウィジット（ボタン、テキストボックス等）
- イベント処理（Phase 6'で実装）

---

## ✅ Success Criteria

- ✅ **子エンティティの`Visual`が親の`Visual`に追加される**
- ✅ **子ウィジットがWindow上に表示される**
- ✅ 親子関係が正しく管理される（`ChildOf`, `Children`）
- ✅ 子エンティティを削除すると表示も消える
- ✅ 階層変換システム（`tree_system`）との統合
- ✅ Transformによる位置制御が動作

---

## 📝 Implementation Elements

- `create_child_visual`システム（子Visualの作成・親への追加）
- `sync_visual_hierarchy`システム（VisualツリーとECS階層の同期）
- `remove_child_visual`システム（削除時のクリーンアップ）
- 子エンティティの`Surface`作成・描画
- Visualの位置・サイズ管理（`Transform`との連携）
- COM APIラッパー:
  - `IDCompositionVisual::AddVisual`
  - `IDCompositionVisual::RemoveVisual`
  - `IDCompositionVisual::SetOffsetX/Y`

---

## 🔄 Dependencies

- Milestone 1完了（GraphicsCore初期化）
- Milestone 2完了（WindowGraphics + Visual作成）
- Milestone 3完了（初めての描画）
- `tree_system`（Transform伝播）

---

## 📚 References

- `.kiro/specs/brainstorming-next-features/MILESTONES.md` - マイルストーン全体像
- `.kiro/specs/phase2-m3-first-rendering/` - 前提となるMilestone 3
- `crates/wintf/src/ecs/tree_system.rs` - 階層変換システム

---

## 🔄 Next Steps

```bash
/kiro-spec-requirements phase2-m4-first-widget
```

---

_Phase 0 (Initialization) completed. Ready for requirements phase._
