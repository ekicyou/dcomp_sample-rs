# Next Milestones: 次期マイルストーン提案

**Proposal Date**: 2025-11-15  
**Based on**: PRIORITY_ANALYSIS.md優先順位Top 10

---

## 提案マイルストーン一覧

以下の3つのマイルストーンを優先順位順に提案します。

1. **Phase 3: 透過ウィンドウとヒットテスト** - 最優先
2. **Phase 4: 横書きテキスト** - 2番目
3. **Phase 5: 画像表示** - 3番目

---

## 提案 1: Phase 3 - 透過ウィンドウとヒットテスト

**Feature ID**: `phase3-transparent-window-hittest`  
**優先順位**: 1位（最優先）  
**推定工数**: 2-3週間

### 目的
透過ウィンドウの完成とヒットテストの実装により、「伺か」のようなデスクトップマスコットアプリケーションの基盤を完成させる。

### スコープ

**含まれるもの**:
- 完全な透過ウィンドウ実装（DirectComposition透過の完成）
- ヒットテスト実装（WM_NCHITTEST）
- マウスドラッグによるウィンドウ移動
- 透過領域でのクリックスルー
- 論理ツリーとビジュアルツリーの統合

**含まれないもの**:
- レイヤードウィンドウ方式（DirectCompositionに統一）
- 複雑なヒットテスト（矩形領域のみ）
- イベント処理システム（Phase 4以降）

### 成功基準
- ✅ デスクトップが完全に透けて見える
- ✅ 透過領域でクリックが背後に通る
- ✅ 不透明領域でクリックが反応する
- ✅ ウィンドウをドラッグして移動できる
- ✅ 120fps以上のパフォーマンスを維持

### 主要な実装要素
1. **完全透過ウィンドウ**
   - DirectComposition透過の最終調整
   - WS_EX_NOREDIRECTIONBITMAPフラグの確認
2. **ヒットテスト**
   - WM_NCHITTESTハンドラ実装
   - Rectangleコンポーネントとの統合
   - HTTRANSPARENTの返却
3. **マウスドラッグ**
   - WM_LBUTTONDOWN, WM_MOUSEMOVE, WM_LBUTTONUPハンドリング
   - ウィンドウ位置の動的更新

### 依存関係

**前提条件**:
- Phase 2完了（✅ 完了済み）
- Rectangleウィジット存在（✅ 完了済み）

**依存するマイルストーン**:
- なし（Phase 2のみ）

### 実装順序（推奨）
1. 完全透過ウィンドウ（1週間）
2. ヒットテスト基盤（1週間）
3. マウスドラッグ（1週間）

### Kiro Spec作成
```bash
/kiro-spec-init "phase3-transparent-window-hittest"
```

---

## 提案 2: Phase 4 - 横書きテキスト

**Feature ID**: `phase4-horizontal-text`  
**優先順位**: 2位  
**推定工数**: 3週間

### 目的
DirectWriteを統合し、横書きテキストレンダリングを実現。ラベルとボタンウィジットを実装し、実用的なUIの基盤を作る。

### スコープ

**含まれるもの**:
- DirectWrite統合（テキストフォーマット、レイアウト）
- 横書きテキストレンダリング
- Labelウィジット実装
- Buttonウィジット実装（テキスト + クリックイベント）
- 基本的なマウスクリックイベント

**含まれないもの**:
- 縦書きテキスト（Phase 6）
- テキスト編集（TextBox等）
- 高度なタイポグラフィ
- IME統合

### 成功基準
- ✅ 指定したフォント・サイズ・色でテキスト表示
- ✅ Labelウィジットで文字列表示
- ✅ Buttonウィジットでクリック反応
- ✅ 複数のテキストウィジットが同時表示
- ✅ 120fps以上のパフォーマンスを維持

### 主要な実装要素
1. **DirectWrite統合**
   - TextFormatコンポーネント
   - TextLayoutコンポーネント
   - COM APIラッパー拡張
2. **Labelウィジット**
   - Text + Position + Color
   - draw_labelsシステム
3. **Buttonウィジット**
   - Label + Rectangle + クリックイベント
   - draw_buttonsシステム
4. **マウスクリックイベント**
   - WM_LBUTTONDOWNハンドリング
   - ヒットテストとの統合

### 依存関係

**前提条件**:
- Phase 3完了（ヒットテスト）

**依存するマイルストーン**:
- Phase 3: ヒットテスト（クリックイベントの前提）

### 実装順序（推奨）
1. DirectWrite統合（1週間）
2. Labelウィジット（3日）
3. マウスクリックイベント（1週間）
4. Buttonウィジット（1週間）

### Kiro Spec作成
```bash
/kiro-spec-init "phase4-horizontal-text"
```

---

## 提案 3: Phase 5 - 画像表示

**Feature ID**: `phase5-image-display`  
**優先順位**: 3位  
**推定工数**: 2週間

### 目的
WIC (Windows Imaging Component)を統合し、PNG等の透過画像を表示できるようにする。「伺か」のキャラクター立ち絵表示を実現。

### スコープ

**含まれるもの**:
- WIC統合（画像読み込み、デコード）
- 透過PNG読み込み
- Imageウィジット実装
- ID2D1Bitmapへの変換
- 画像描画（DrawBitmap）

**含まれないもの**:
- アニメーションGIF
- 動画再生
- 画像編集機能
- 高度な画像処理（リサイズ、回転等）

### 成功基準
- ✅ PNG画像（透過含む）が表示される
- ✅ 複数の画像が同時表示可能
- ✅ 背景が透けて見える
- ✅ Imageウィジットで位置・サイズ指定可能
- ✅ 120fps以上のパフォーマンスを維持

### 主要な実装要素
1. **WIC統合**
   - WICファクトリ
   - WICBitmapDecoder
   - WICFormatConverter
   - COM APIラッパー拡張
2. **Imageウィジット**
   - ImagePath + Position + Size
   - ID2D1Bitmapキャッシュ
   - draw_imagesシステム
3. **画像描画**
   - DrawBitmap統合
   - アルファブレンディング確認

### 依存関係

**前提条件**:
- Phase 2完了（描画基盤）
- Phase 3完了（透過ウィンドウ）

**依存するマイルストーン**:
- Phase 3: 透過ウィンドウ（背景透過の確認）

### 実装順序（推奨）
1. WIC統合（1週間）
2. Imageウィジット（3日）
3. サンプル作成・テスト（3日）

### Kiro Spec作成
```bash
/kiro-spec-init "phase5-image-display"
```

---

## マイルストーン間の依存関係

```
Phase 2 (完了)
    ↓
Phase 3: 透過ウィンドウとヒットテスト
    ↓
Phase 4: 横書きテキスト
    ↓
Phase 5: 画像表示
    ↓
Phase 6: レイアウトシステム (taffy統合)
    ↓
Phase 7: 縦書きテキスト (最終目標)
```

---

## 推奨実装順序

### 最速ルート（Phase 3 → 4 → 5）
1. **Phase 3** (2-3週間): 透過ウィンドウ完成
2. **Phase 4** (3週間): テキスト + 基本ウィジット
3. **Phase 5** (2週間): 画像表示

**合計**: 7-8週間（約2ヶ月）

この順序により、「伺か」のようなデスクトップマスコットの基本機能が実現します。

### Phase 3を最優先する理由
1. **README.mdロードマップとの整合**
2. **Phase 2の完成**: 透過ウィンドウが部分実装済み
3. **インタラクションの基盤**: すべてのイベント処理の前提
4. **比較的低難易度**: 2-3週間で完了可能
5. **視覚的インパクト**: デスクトップマスコットらしさの実現

### Phase 4を次にする理由
1. **実用性の向上**: テキストなしでは実用的でない
2. **Phase 6（縦書き）への布石**: 横書きで基盤を作る
3. **DirectWriteの活用**: 既に初期化済み

### Phase 5を3番目にする理由
1. **視覚的完成度**: キャラクター立ち絵の表示
2. **WIC統合の容易性**: 比較的簡単（2週間）
3. **プロダクトの華**: アプリケーションが一気に魅力的に

---

## その他の候補（Phase 6以降）

### Phase 6: レイアウトシステム
- **Feature ID**: `phase6-layout-system`
- **推定工数**: 5週間
- **理由**: 手動配置からの脱却、自動レイアウト

### Phase 7: 縦書きテキスト
- **Feature ID**: `phase7-vertical-text`
- **推定工数**: 4週間以上
- **理由**: プロダクト最終目標

### Phase 8: 高度なインタラクション
- **Feature ID**: `phase8-advanced-interaction`
- **推定工数**: 4週間
- **理由**: イベント処理の拡充

---

## まとめ

Phase 2完了により、描画基盤が確立しました。次の3つのマイルストーン（Phase 3-5）を実装することで、**「伺か」のようなデスクトップマスコットアプリケーションの基本機能が完成**します。

### 次のアクション

Phase 3から開始することを推奨します：

```bash
/kiro-spec-init "phase3-transparent-window-hittest"
```

---

## 関連ドキュメント

- [REVIEW.md](./REVIEW.md) - Phase 2レビュー結果
- [PRIORITY_ANALYSIS.md](./PRIORITY_ANALYSIS.md) - 優先順位分析
- [FEATURE_MATRIX.md](./FEATURE_MATRIX.md) - 未実装機能マトリクス

---

_Next Milestones completed on 2025-11-15_
