# Requirements: 次に開発するべき開発要素をブレインストーミング

**Feature ID**: `brainstorming-next-features`  
**Phase**: Requirements Analysis  
**Updated**: 2025-11-14

---

## 📊 現状分析

### プロジェクトの達成状況

#### ✅ 完了済みフェーズ

**フェーズ1: ウィンドウの誕生 🐣** (100%完了)
- ✅ プロジェクトセットアップ
- ✅ ウィンドウクラス登録
- ✅ ウィンドウ生成
- ✅ メッセージループ実装
- ✅ ECS統合（Bevy ECS 0.17.2）
- ✅ `simple_window.rs`サンプルで動作確認済み（106.32 fps）

**最近のコミット履歴**:
- `c893a2d` - DirectCompositionをデフォルト有効化（仕様: `dcomp-default-window`）
- `dc68fe8` - `transform_system.rs` → `tree_system.rs` リネーム（仕様: `transform-to-tree-refactor`）
- `3aefa29` - transform_system インテグレーションテスト追加（仕様: `transform_system_test`）
- `015bce2` - transform_system ジェネリック化完了（仕様: `transform-system-generic`）

#### 🔄 進行中の仕様（既に完了）

`.kiro/specs/` ディレクトリに存在する仕様のうち、以下は完了済み:

1. **`dcomp-default-window`** ✅ 完了
   - ステータス: implementation_complete
   - 内容: `WindowStyle::default()`で`WS_EX_NOREDIRECTIONBITMAP`を有効化
   - 成果: DirectCompositionがデフォルトで利用可能に

2. **`ecs-window-display`** ✅ 完了
   - ステータス: Completed
   - 内容: ECSでのウィンドウ表示と終了処理の実装
   - 成果: `simple_window.rs`サンプルが動作

3. **`transform-system-generic`** ✅ 完了
   - ステータス: ✅ 完了
   - 内容: transform_systemの型パラメータ化（`<L, G, M>`）
   - 成果: 汎用的な階層変換システムを実装

4. **`transform-to-tree-refactor`** ✅ 完了
   - ステータス: ✅ COMPLETED
   - 内容: `transform_system.rs` → `tree_system.rs` リファクタリング
   - 成果: モジュール名が責務を正確に反映

5. **`transform_system_test`** ✅ 完了
   - ステータス: ✅ 実装完了
   - 内容: tree_systemの包括的インテグレーションテスト
   - 成果: 8シナリオ15テストすべてパス

**結論**: 既存の5つの仕様はすべて完了しており、進行中のものはありません。

#### 📅 未着手フェーズ（README.mdより）

**フェーズ2: はじめての描画 🎨** (0%完了)
- [ ] タスク2.1: 描画エンジンの初期化
  - DirectComposition、Direct2D、DirectWriteのファクトリオブジェクト作成
- [ ] タスク2.2: レンダリングターゲットの作成
  - Direct2Dレンダリングターゲット作成
- [ ] タスク2.3: 簡単な図形の描画
  - 四角形や円などの描画

**フェーズ3: 透過ウィンドウとヒットテスト** (0%完了)
- [ ] タスク3.1: 透過ウィンドウ対応
- [ ] タスク3.2: 論理ツリーとビジュアルツリーの構築
- [ ] タスク3.3: ビジュアルツリーに対するヒットテスト
- [ ] タスク3.4: ウィンドウレベルのヒットテスト

**フェーズ4: 文字との対話（横書き） ✍️** (0%完了)
- [ ] タスク4.1: テキストフォーマットの作成
- [ ] タスク4.2: テキストの描画
- [ ] タスク4.3: ブラシの作成と色の適用

**フェーズ5: 画像の表示と透過処理 🖼️** (0%完了)
- [ ] タスク5.1: 画像読み込みの準備（WIC）
- [ ] タスク5.2: 画像の読み込み
- [ ] タスク5.3: ビットマップの描画

**フェーズ6: 縦書きの世界へ 📖** (0%完了)
- [ ] タスク6.1: 縦書き用テキストレイアウトの作成
- [ ] タスク6.2: 縦書きテキストの描画
- [ ] タスク6.3: 句読点などの調整

**フェーズ6（重複番号）: 高度なインタラクション 🖱️** (0%完了)
- [ ] タスク6.1: ヒットテストの実装
- [ ] タスク6.2: マウスクリックイベントの処理
- [ ] タスク6.3: IMEサポートの基礎

---

## 🧩 技術的依存関係の整理

### 既存の実装基盤

#### ✅ 現在利用可能な基盤
1. **ウィンドウシステム** (Phase 1完了)
   - `Window`コンポーネント: ウィンドウ作成パラメータ
   - `WindowHandle`コンポーネント: 作成後の情報（hwnd, instance, initial_dpi）
   - `create_windows`システム: 自動的にウィンドウを作成
   - `ecs_wndproc`: ECS専用ウィンドウプロシージャ
   - DirectComposition対応: デフォルトで有効

2. **階層変換システム** (完了)
   - `Transform`/`GlobalTransform`コンポーネント
   - `tree_system.rs`: ジェネリックな階層伝播システム
   - `ChildOf`/`Children`: bevy_ecs標準の親子関係
   - 包括的なテストカバレッジ（15テスト）

3. **COM APIラッパー** (部分的に実装)
   - `com/dcomp.rs`: DirectComposition API
   - `com/d3d11.rs`: Direct3D11 API
   - `com/d2d/`: Direct2D API（構造確認必要）
   - `com/dwrite.rs`: DirectWrite API
   - `com/wic.rs`: Windows Imaging Component

4. **ECSアーキテクチャ**
   - `bevy_ecs 0.17.2`
   - `WinThreadMgr`: メッセージループとECS統合
   - `World`/`Schedule`: シングルスレッド実行でメインスレッド保証

#### 🔍 現在不明な実装範囲

以下のCOM APIラッパーの実装状況を確認する必要があります:
- `com/d2d/`: Direct2D関連の実装内容
- `com/dcomp.rs`: DirectCompositionの実装範囲
- `com/d3d11.rs`: Direct3D11の実装範囲
- `com/dwrite.rs`: DirectWriteの実装範囲
- `com/wic.rs`: WICの実装範囲

### フェーズ間の依存関係

```
Phase 1 (完了) ← ベース
    ↓
Phase 2 (描画) ← 最優先
    ├→ Direct2D初期化
    ├→ DirectWrite初期化（テキスト用）
    └→ DirectComposition統合
    ↓
Phase 3 (透過とヒットテスト) ← Phase 2後
    ├→ ビジュアルツリー構築（Phase 2の描画が必要）
    └→ ヒットテスト（描画領域の認識が必要）
    ↓
Phase 4 (横書きテキスト) ← Phase 2後（並列可能）
    └→ DirectWrite（Phase 2で初期化）
    ↓
Phase 5 (画像表示) ← Phase 2後（並列可能）
    └→ WIC + Direct2D（Phase 2で初期化）
    ↓
Phase 6 (縦書き) ← Phase 4後
    └→ 横書きの知見を応用
    ↓
Phase 6' (インタラクション) ← Phase 3後
    └→ ヒットテスト基盤が必要
```

**結論**: **Phase 2（描画）が最も優先度が高く、他のすべてのフェーズのブロッカー**となっています。

---

## 🎯 優先度評価

### 評価基準

#### 1. ユーザー価値（「伺か」実現への寄与度）

**Phase 2（描画）**: ⭐⭐⭐⭐⭐ (5/5)
- 理由: 空のウィンドウから「見えるもの」への大きな一歩
- 影響: キャラクター表示の基盤、テキスト表示の基盤
- 「伺か」への寄与: 必須（描画なしでは何も表示できない）

**Phase 4（横書きテキスト）**: ⭐⭐⭐⭐ (4/5)
- 理由: メッセージ表示が可能になる
- 影響: キャラクターとの対話の基盤
- 「伺か」への寄与: 高（セリフ表示は重要）

**Phase 5（画像表示）**: ⭐⭐⭐⭐⭐ (5/5)
- 理由: キャラクター立ち絵の表示
- 影響: 「伺か」の視覚的な核心
- 「伺か」への寄与: 必須（キャラクターが見えないと意味がない）

**Phase 3（透過とヒットテスト）**: ⭐⭐⭐⭐ (4/5)
- 理由: デスクトップマスコットとしての操作性
- 影響: クリック反応、ドラッグ移動
- 「伺か」への寄与: 高（インタラクションに必要）

**Phase 6（縦書き）**: ⭐⭐⭐ (3/5)
- 理由: プロジェクトの核心だが、横書きで代替可能
- 影響: 日本語表現の美しさ
- 「伺か」への寄与: 中（横書きでも動作可能）

**Phase 6'（インタラクション）**: ⭐⭐⭐⭐ (4/5)
- 理由: ユーザーとの対話
- 影響: IME、クリックイベント処理
- 「伺か」への寄与: 高（対話型アプリとして重要）

#### 2. 技術的リスクと実装コスト

**Phase 2（描画）**: リスク⚠️ 中、コスト💰 中
- リスク要因:
  - DirectComposition/Direct2D/Direct3D11の統合
  - レンダリングループの設計
  - リソース管理（COM オブジェクトのライフタイム）
  - **デバイスロスト対応**: 最初から実装しないと後でハマる（⚠️ 重要）
- 実装コスト: 2-3週間（COM APIラッパーの実装状況次第）
- 軽減策: 
  - 既存のCOM APIラッパーを活用
  - 段階的実装
  - **デバイスロスト対応を最初から組み込む**: グローバルリソースとウィンドウ単位リソースを明確に分離

**Phase 4（横書きテキスト）**: リスク⚠️ 低、コスト💰 低
- リスク要因: DirectWriteのAPI理解
- 実装コスト: 1週間
- 軽減策: Microsoftの公式ドキュメントが充実

**Phase 5（画像表示）**: リスク⚠️ 低、コスト💰 低
- リスク要因: WICのデコーディング、アルファブレンディング
- 実装コスト: 1週間
- 軽減策: WICは標準的なAPI

**Phase 3（透過とヒットテスト）**: リスク⚠️ 高、コスト💰 高
- リスク要因:
  - ビジュアルツリーの設計（論理ツリーとの分離）
  - ヒットテストアルゴリズムの複雑性
  - `WM_NCHITTEST`とのカスタム統合
- 実装コスト: 2-3週間
- 軽減策: Phase 2の描画基盤を先に固める

**Phase 6（縦書き）**: リスク⚠️ 中、コスト💰 中
- リスク要因: DirectWriteの縦書き設定、句読点処理
- 実装コスト: 1-2週間
- 軽減策: Phase 4の知見を活用

**Phase 6'（インタラクション）**: リスク⚠️ 高、コスト💰 高
- リスク要因: TSF (Text Services Framework)の複雑性
- 実装コスト: 3-4週間
- 軽減策: 基本的なマウス処理から段階的に実装

#### 3. 他フェーズへの影響範囲

**Phase 2（描画）**: 🌍 影響範囲: 極めて大
- ブロッカー: Phase 3, 4, 5, 6, 6'すべてがPhase 2に依存
- 基盤提供: レンダリングループ、リソース管理、描画API

**Phase 3（透過とヒットテスト）**: 🌍 影響範囲: 大
- ブロッカー: Phase 6'（インタラクション）
- 基盤提供: ビジュアルツリー、ヒットテスト機構

**Phase 4（横書きテキスト）**: 🌍 影響範囲: 中
- ブロッカー: Phase 6（縦書き）
- 基盤提供: テキストレイアウトの知見

**Phase 5（画像表示）**: 🌍 影響範囲: 小
- 他フェーズへの直接的な影響は少ない
- ただし「伺か」実現には必須

**Phase 6（縦書き）**: 🌍 影響範囲: 小
- 他フェーズへの依存なし（Phase 4の後）

**Phase 6'（インタラクション）**: 🌍 影響範囲: 小
- 他フェーズへの依存なし（Phase 3の後）

---

## 📋 推奨開発ロードマップ

### 短期計画（1-2週間）: Phase 2 - はじめての描画 🎨

**優先度**: 🔥 最優先

**理由**:
1. すべての後続フェーズのブロッカー
2. 視覚的フィードバックが得られる（開発者モチベーション向上）
3. DirectCompositionがデフォルトで有効化済み（Phase 1の成果を活用）

**実装範囲**:
- タスク2.1: グローバルリソースの初期化
  - **`GraphicsCore`の作成**: D3D11, D2D, DWrite, DCompファクトリの統合シングルトン
  - `ProcessSingleton`として管理（プロセス全体で共有）
- タスク2.2: ウィンドウ単位のリソース作成（ECSコンポーネント）
  - **`WindowGraphics`コンポーネント**: DirectCompositionターゲット + D2Dデバイスコンテキスト（統合）
    - `composition_target: IDCompositionTarget` (ルートターゲット)
    - `device_context: ID2D1DeviceContext` (リソース作成専用、描画には使わない)
  - `WindowHandle`を持つエンティティに自動的にアタッチ
- タスク2.3: デバイスロスト対応
  - **マーカー不要**: `WindowGraphics`を削除するだけ
  - `EndDraw`で`D2DERR_RECREATE_TARGET`を検出 → `WindowGraphics`を削除
  - 次フレームで`Query<Entity, (With<WindowHandle>, Without<WindowGraphics>)>`が自動再作成
  - `D3D11_CREATE_DEVICE_BGRA_SUPPORT`でデバイス作成（Direct2D互換性）
- タスク2.4: 簡単な図形描画（子要素エンティティで実装）
  - 子要素エンティティに`Visual`, `Surface`コンポーネントを追加
  - 四角形（`DrawRectangle`, `FillRectangle`）
  - 円（`DrawEllipse`, `FillEllipse`）
  - ソリッドカラーブラシ作成（`WindowGraphics`のデバイスコンテキストで作成）
  - `BeginDraw`/`EndDraw`のラッピング

**成功基準**:
- ✅ ウィンドウに色付きの四角形が表示される
- ✅ ウィンドウに円が描画される
- ✅ フレームレート 60fps以上を維持
- ✅ COM オブジェクトのリソースリークなし
- ✅ デバイスロスト時に自動でリソース再作成される
- ✅ 複数ウィンドウで独立したレンダリングターゲットを持つ

**技術的課題**:
- DirectCompositionとDirect2Dの統合パターン
- ECSコンポーネント設計（`GraphicsCore`, `WindowGraphics`, `Visual`, `Surface`）
- レンダリングループのタイミング（`WM_TIMER`との統合）
- **デバイスロストの検出と再作成のタイミング**: `EndDraw`で`D2DERR_RECREATE_TARGET`を検出 → `WindowGraphics`削除
- **グローバルリソースとウィンドウ/子要素リソースの分離**: 
  - グローバル: `GraphicsCore` (ファクトリ群)
  - ウィンドウ: `WindowGraphics` (ターゲット + デバイスコンテキスト)
  - 子要素: `Visual`, `Surface`

**重要な設計決定**:
1. **`GraphicsCore`でファクトリを統合**: D3D11, D2D, DWrite, DCompを1つのシングルトンで管理
2. **`WindowGraphics`でターゲットとデバイスコンテキストを統合**: ライフタイムが同じなので分離不要
3. **`WindowGraphics`は`WindowHandle`と同じエンティティ**: 1ウィンドウ = 1グラフィックスリソース
4. **`Visual`, `Surface`は子要素用**: ウィンドウエンティティには直接紐づかない、階層構造で使用
5. **デバイスロスト時はマーカー不要**: `WindowGraphics`削除 → 次フレームで自動再作成（ECSらしいシンプルな設計）
6. **デバイスロストは最初から考慮**: 後回しにするとリファクタリングが困難

**事前調査が必要**:
- `com/dcomp.rs`の実装状況確認
- `com/d2d/`の実装状況確認
- `com/d3d11.rs`の実装状況確認

### 中期計画（1ヶ月）: Phase 4 + Phase 5

**Phase 4: 文字との対話（横書き）** ✍️
- 実装期間: 1週間
- 依存: Phase 2完了後
- 内容:
  - DirectWriteテキストフォーマット作成
  - テキスト描画（`DrawText`, `DrawTextLayout`）
  - ブラシ作成と色適用

**Phase 5: 画像の表示と透過処理** 🖼️
- 実装期間: 1週間
- 依存: Phase 2完了後（Phase 4と並列可能）
- 内容:
  - WICファクトリ作成
  - 画像デコード（PNG、JPEG等）
  - Direct2Dビットマップ変換
  - アルファブレンディング描画

**マイルストーン**:
- ✅ "Hello, World!" がウィンドウに表示される
- ✅ キャラクター立ち絵が透過付きで表示される
- ✅ 「伺か」の最小MVPが視覚的に確認できる

### 長期計画（3ヶ月）: Phase 3 + Phase 6 + Phase 6'

**Phase 3: 透過ウィンドウとヒットテスト** (2-3週間)
- 論理ツリーとビジュアルツリーの設計
- ヒットテストアルゴリズム実装
- `WM_NCHITTEST`カスタマイズ

**Phase 6: 縦書きの世界へ** 📖 (1-2週間)
- DirectWriteの縦書き設定（`SetReadingDirection`, `SetFlowDirection`）
- 句読点の回転処理
- `IDWriteTypography`の活用

**Phase 6': 高度なインタラクション** 🖱️ (3-4週間)
- マウスクリックイベント処理
- ドラッグ移動
- TSF (Text Services Framework) 基礎実装

**最終マイルストーン**:
- ✅ 透過ウィンドウでキャラクターをクリック可能
- ✅ 縦書きテキストが美しく表示される
- ✅ 日本語IMEでテキスト入力可能

---

## 🔍 既存実装の調査項目

次のステップに進む前に、以下のCOM APIラッパーの実装状況を確認する必要があります:

### 調査対象ファイル
1. `crates/wintf/src/com/dcomp.rs` - DirectComposition実装範囲
2. `crates/wintf/src/com/d2d/` - Direct2D実装範囲
3. `crates/wintf/src/com/d3d11.rs` - Direct3D11実装範囲
4. `crates/wintf/src/com/dwrite.rs` - DirectWrite実装範囲
5. `crates/wintf/src/com/wic.rs` - WIC実装範囲

### 調査内容
- ✅ 実装済みのAPI
- ⚠️ 未実装のAPI
- 📝 使用例（既存サンプル）

---

## 📝 要件サマリー

### 最優先: Phase 2 - はじめての描画 🎨

**目標**: 空のウィンドウから「見える」ウィンドウへ

**要件**:
1. **グローバルリソース初期化**: `GraphicsCore` (D3D11, D2D, DWrite, DCompファクトリの統合)
2. **ウィンドウ単位コンポーネント**: `WindowGraphics` (CompositionTarget + DeviceContext の統合)
3. **子要素/ウィジット用コンポーネント**: `Visual`, `Surface` (階層構造で使用)
4. **デバイスロスト対応**: マーカー不要、`WindowGraphics`削除 → 自動再作成
5. **図形描画**: 四角形と円の描画（子要素エンティティで実装）

**受け入れ基準**:
- ✅ ウィンドウに色付き図形が表示される
- ✅ 60fps以上のフレームレート維持
- ✅ リソースリークなし
- ✅ デバイスロスト時に自動復旧する
- ✅ 複数ウィンドウで独立して描画可能
- ✅ 既存の`simple_window.rs`との互換性維持

**除外事項**:
- テキスト描画（Phase 4で実装）
- 画像表示（Phase 5で実装）
- 透過処理の高度な制御（Phase 3で実装）

---

## 🔄 次のアクション

### Phase 1: 設計フェーズ（このドキュメント完了後）

```bash
/kiro-spec-design brainstorming-next-features
```

設計フェーズでは以下を定義:
1. Phase 2の詳細設計
2. **ECSコンポーネント設計**:
   - グローバルリソース: `GraphicsCore` (D3D11, D2D, DWrite, DCompファクトリの統合シングルトン)
   - ウィンドウ単位コンポーネント: `WindowGraphics` (CompositionTarget + DeviceContext の統合)
   - 子要素/ウィジット用: `Visual`, `Surface` (階層構造で使用、ウィンドウには直接紐づかない)
3. **システム設計**:
   - `initialize_graphics_core`: グローバルリソース初期化（起動時1回）
   - `create_window_graphics`: ウィンドウにグラフィックスリソースをアタッチ
   - `render_system`: 描画処理（子要素の`Surface`に描画）
4. **デバイスロスト対応の詳細設計**:
   - `EndDraw`で`D2DERR_RECREATE_TARGET`を検出
   - `WindowGraphics`コンポーネントを削除
   - 次フレームで`create_window_graphics`が自動的に再作成
   - マーカー不要（ECSのコンポーネント有無で状態を表現）
5. COM APIラッパーの拡張計画

### Phase 2: タスク分解

```bash
/kiro-spec-tasks brainstorming-next-features
```

### Phase 3: 実装（タスクベースで進行）

**注意**: このブレインストーミング仕様は実装を含みません。Phase 2（描画）の実装は別の仕様として作成します。

---

## 🎯 推奨される次の仕様

**仕様名**: `phase2-basic-rendering`

**スコープ**:
- グローバルリソース初期化: `GraphicsCore` (D3D11, D2D, DWrite, DCompファクトリの統合)
- ウィンドウ単位リソース作成: `WindowGraphics` (CompositionTarget + DeviceContext の統合)
- **デバイスロスト対応**（最初から実装、マーカー不要）
- 四角形と円の描画（子要素エンティティで実装）
- ECSコンポーネント設計:
  - グローバル: `GraphicsCore`
  - ウィンドウ単位: `WindowGraphics`
  - 子要素/ウィジット用: `Visual`, `Surface`
- システム設計: `initialize_graphics_core`, `create_window_graphics`, `render_system`
- サンプル実装（`rendering_demo.rs`）

**依存**:
- Phase 1完了（✅）
- COM APIラッパーの実装状況調査（特にデバイスロスト関連API）

**重要な技術的決定**:
- **`GraphicsCore`でファクトリを統合**: バラバラに管理せず、1つのシングルトンで統合
- **`WindowGraphics`でターゲットとデバイスコンテキストを統合**: ライフタイムが同じなので分離不要
- **`WindowGraphics`は`WindowHandle`と同じエンティティに配置**: 1ウィンドウ = 1グラフィックスリソース
- **`Visual`, `Surface`はウィンドウには直接紐づかない**: 子要素/ウィジット用コンポーネント、階層構造で使用
- **デバイスロスト時はマーカー不要**: `WindowGraphics`削除 → 次フレームで自動再作成
- **デバイスロストは後回しにせず、最初から実装**（後からやるとハマる）

---

_このドキュメントは要件分析フェーズの成果物です。次のステップは設計フェーズに進みます。_


