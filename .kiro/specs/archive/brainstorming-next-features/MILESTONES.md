# Phase 2マイルストーン: 初めての描画への道

**Updated**: 2025-11-14

---

## 設計判断の確定

### Q1: ルートVisualはどこに？
**決定**: **WindowエンティティにVisualコンポーネントを作る**

**理由**:
- Windowエンティティをルートとしたウィジットツリーが作成される
- エンティティは複数の子を持つことができる
- 複数の子を持つためにはVisualが1つ必要 → Windowエンティティ自体にVisualが必要
- `WindowGraphics`に含めるか？ → **いいえ**
  - Visualという機能はWindowを含むあらゆるウィジットが持つ可能性
  - 分離したコンポーネントとして扱う

**構成**:
```rust
Window Entity
├─ WindowHandle
├─ WindowGraphics (CompositionTarget + DeviceContext)
└─ Visual (IDCompositionVisual) ← ルートビジュアル
```

---

### Q2: Window自体に描画要素を持たせるか？
**決定**: **可能、かつ初めての描画では推奨**

**理由**:
- Visualを持つ以上、コンテンツを持つことは可能
- ECS設計なので、コンポーネントさえ持たせればWindowエンティティ自体に描画の責務を載せられる
- **初めての描画の最短ルート** → Window自体で描画する方が楽
- 子ビジュアルを作る必要がない（後で追加可能）

**構成**:
```rust
Window Entity (最初の実装)
├─ WindowHandle
├─ WindowGraphics
├─ Visual (ルートビジュアル)
└─ Surface (描画先) ← Window自体が描画可能
```

---

### Q3: Surfaceのサイズは？
**決定**: **とりあえずはWindowPosに従う**

**理由**:
- 本来はレイアウトを実行した後にBOXサイズが決定するのが原則
- ただし、レイアウト部分の実装に入ると進捗が出なくなりモチベーションが大変
- Windowを作るときに`WindowPos`コンポーネントを作るのでそこからサイズを取ってくる
- レイアウト実装時に個々の部分は改造が必要（将来の開発要素）

**実装**:
```rust
fn create_surface(
    windows: Query<(&WindowPos, &WindowGraphics)>,
) {
    for (pos, graphics) in windows.iter() {
        let size = (pos.width, pos.height);
        // Surfaceをウィンドウサイズで作成
    }
}
```

---

### Q4: Commit()のタイミング
**決定**: **とりあえずは毎フレームの最後**

**検討点**:
- DCompはマルチスレッド対応、Commitの挙動は？
- Visual更新、Surface更新の小さなタイミングでCommitすると性能影響？
- スケジューラーがあるので`Composition`スケジューラーの後ろに`PostComposition`とか作ればよいか？

**最初の実装**:
- 毎フレーム、描画処理の最後に`Commit()`
- 最適化は後で（変更検出など）

---

### Q5: 最初に何を描画するか
**決定**: **透明で塗りつぶして、●■▲**

**理由**:
- ちょっとは頑張りたい
- DCompらしく、背景透過で描画したい
- 3つの図形（円・四角・三角）で視覚的に面白い

**実装内容**:
1. `Clear(transparent)` - 透明背景
2. 赤い円 ●
3. 緑の四角 ■
4. 青い三角 ▲

---

### Q6: 描画の前に子ビジュアルを作るか
**決定**: **NO**

**理由**:
- 前述の理由により、とりあえずWindowエンティティに閉じた実装を行う方針
- 子ビジュアルは将来の拡張として後で追加

---

## 確定したマイルストーン

### 〇1: GraphicsCore初期化
**スコープ**:
- D3D11Device, D2DFactory, D2DDevice, DWriteFactory, DCompDevice
- `ProcessSingleton`として管理

**成功基準**:
- ✅ エラーなく初期化完了
- ✅ ログで各ファクトリの作成を確認

**実装要素**:
- `GraphicsCore`構造体
- `initialize_graphics_core()`システム
- COM APIラッパー拡張

**Kiro仕様**: `phase2-m1-graphics-core`

---

### 〇2: WindowGraphics + Visual作成
**スコープ**:
- `WindowGraphics`作成（CompositionTarget + DeviceContext）
- CompositionTargetをhwndに紐付け
- **`Visual`コンポーネント作成**（Windowエンティティにアタッチ）
- VisualをTargetのルートとして設定

**成功基準**:
- ✅ ウィンドウごとに`WindowGraphics`が存在
- ✅ ウィンドウに`Visual`コンポーネントが存在
- ✅ VisualがTargetに設定済み

**実装要素**:
- `WindowGraphics`構造体
- `Visual`構造体（コンポーネント）
- `create_window_graphics`システム
- `create_window_visual`システム
- `IDCompositionTarget::SetRoot(visual)`

**Kiro仕様**: `phase2-m2-window-graphics`

---

### △: Windowに描画（透明背景 + ●■▲）
**スコープ**:
- `Surface`コンポーネント作成（Windowエンティティにアタッチ）
- Surfaceサイズは`WindowPos`から取得
- VisualのコンテンツにSurfaceを設定
- 描画処理:
  1. `Clear(transparent)` - 透明背景
  2. 赤い円 ● (`FillEllipse`)
  3. 緑の四角 ■ (`FillRectangle`)
  4. 青い三角 ▲ (`FillGeometry`)
- `Commit()`で画面反映

**成功基準**:
- ✅ **ウィンドウに透過背景で●■▲が表示される** 🎉
- ✅ デスクトップが透けて見える（透過動作確認）
- ✅ エラーなし

**実装要素**:
- `Surface`構造体（コンポーネント）
- `WindowPos`構造体（ウィンドウサイズ管理）
- `create_window_surface`システム
- `render_window`システム（描画処理）
- `commit_composition`システム（毎フレーム最後）
- ブラシ作成（red, green, blue）
- PathGeometry作成（三角形用）

**Kiro仕様**: `phase2-m3-first-rendering`

---

### ◇: 初めてのウィジット（子要素Visual管理）
**スコープ**:
- 子エンティティの作成とVisualツリー管理
- `ChildOf`リレーションを使った親子関係
- 子エンティティに`Visual`コンポーネント追加
- 親Visual（Window）に子Visualを追加（`AddVisual`）
- 子Visualにも`Surface`を作成して描画
- Visualツリーの階層的な更新・削除

**描画内容**:
- Windowには背景として薄い灰色の矩形
- 子ウィジット1: 小さな赤い円（中央）
- 子ウィジット2: 小さな青い四角（右下）
- 子ウィジットは独立したSurfaceを持つ

**成功基準**:
- ✅ **子エンティティの`Visual`が親の`Visual`に追加される**
- ✅ **子ウィジットがWindow上に表示される**
- ✅ 親子関係が正しく管理される（`ChildOf`, `Children`）
- ✅ 子エンティティを削除すると表示も消える
- ✅ 階層変換システム（`tree_system`）との統合

**実装要素**:
- `create_child_visual`システム（子Visualの作成・親への追加）
- `sync_visual_hierarchy`システム（VisualツリーとECS階層の同期）
- `remove_child_visual`システム（削除時のクリーンアップ）
- 子エンティティの`Surface`作成・描画
- Visualの位置・サイズ管理（`Transform`との連携）

**Kiro仕様**: `phase2-m4-first-widget`

---

## 実装順序

1. **Phase 2 Milestone 1**: GraphicsCore初期化
   - 仕様: `phase2-m1-graphics-core`
   - `/kiro-spec-init "phase2-m1-graphics-core"`

2. **Phase 2 Milestone 2**: WindowGraphics + Visual作成
   - 仕様: `phase2-m2-window-graphics`
   - `/kiro-spec-init "phase2-m2-window-graphics"`

3. **Phase 2 Milestone 3**: 初めての描画（●■▲）
   - 仕様: `phase2-m3-first-rendering`
   - `/kiro-spec-init "phase2-m3-first-rendering"`

4. **Phase 2 Milestone 4**: 初めてのウィジット（子要素Visual管理）
   - 仕様: `phase2-m4-first-widget`
   - `/kiro-spec-init "phase2-m4-first-widget"`

---

## アーキテクチャ図（最終版）

```
【グローバル】
GraphicsCore (Singleton)
├─ d3d_device: ID3D11Device
├─ d2d_factory: ID2D1Factory
├─ d2d_device: ID2D1Device
├─ dwrite_factory: IDWriteFactory
└─ dcomp_device: IDCompositionDevice

【Windowエンティティ】
Window Entity
├─ WindowHandle (hwnd, instance, dpi)
├─ WindowPos (x, y, width, height) ← サイズ管理
├─ WindowGraphics {
│   composition_target: IDCompositionTarget,  // hwnd紐付け
│   device_context: ID2D1DeviceContext,       // リソース作成
│  }
├─ Visual (IDCompositionVisual) ← ルートビジュアル
└─ Surface (ID2D1Bitmap) ← Windowに直接描画

【将来の子要素】→【Milestone 4で実装】
Child Entity
├─ Visual (IDCompositionVisual) ← 親Visualに追加
└─ Surface (ID2D1Bitmap) ← 子ウィジット独自の描画

親子関係の管理:
- ChildOf リレーション（bevy_ecs標準）
- Children コンポーネント（自動管理）
- tree_system との統合（Transform伝播）
```

---

## Phase 2完了後の拡張

### Milestone 5以降（Phase 2完全完了へ）

**Milestone 5: デバイスロスト対応**
- `EndDraw`でのエラー検出
- `WindowGraphics`削除
- 自動再作成の確認
- 子Visualも含めた再構築

**Milestone 6: 複数図形の高度な描画**
- グラデーションブラシ
- 複雑なパス（ベジェ曲線等）
- クリッピング
- 不透明度の制御

---

## 今後の拡張（Phase 3以降）

- 子ビジュアルの追加
- レイアウトシステムの実装
- デバイスロスト対応の完全実装
- テキスト描画（Phase 4）
- 画像表示（Phase 5）

---

_Phase 2の明確な道筋が確定しました！_
