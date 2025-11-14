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

【将来の子要素】
Child Entity (Phase 2以降)
├─ Visual
└─ Surface
```

---

## 今後の拡張（Phase 2完了後）

- 子ビジュアルの追加
- レイアウトシステムの実装
- デバイスロスト対応の完全実装
- テキスト描画（Phase 4）
- 画像表示（Phase 5）

---

_Phase 2の明確な道筋が確定しました！_
