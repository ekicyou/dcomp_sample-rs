# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | wintf-image-widget 要件定義書 |
| **Version** | 0.1 (Draft) |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Author** | AI-DLC System |

---


## Introduction

本仕様書は wintf フレームワークにおける Image ウィジェット機能の要件を定義する。親仕様「伺的デスクトップマスコットアプリ」の実装前提条件（P0）として、WIC画像読み込み、D2D描画、透過PNG対応機能を提供する。

### 背景

wintf フレームワークは現在、Rectangle と Label ウィジェットを提供しているが、画像表示機能が未実装である。デスクトップマスコットアプリケーションでは、キャラクターのサーフェス（表情・ポーズ画像）表示が必須機能であり、Image ウィジェットの実装が最優先課題となる。

### スコープ

**含まれるもの**:
- WIC (Windows Imaging Component) による静止画像読み込み（Windows 11標準サポート形式）
- Direct2D による基本画像描画（1:1、変換なし）
- αチャンネル必須（透過処理が必須）
- 非同期画像読み込み（WintfTaskPool）

**含まれないもの（P1 wintf-P1-image-rendering で対応）**:
- ストレッチモード（None/Fill/Uniform/UniformToFill）
- ソース矩形（画像の一部切り抜き表示）
- 補間モード

**含まれないもの（将来対応）**:
- GIF/WebP アニメーション画像のフレーム抽出・再生
- 連番画像アニメーション
- 画像の動的生成・編集
- 画像フィルター・エフェクト
- 画像キャッシュ管理

### 設計原則

- **1ウィジェット = 1画像ファイル**: Imageウィジェットは単一の画像ファイルのみを担当する
- **組み合わせによる拡張**: 連番画像アニメーションやレイヤー合成は、複数のImageウィジェットを組み合わせて実現する
- **Single Responsibility**: 各ウィジェットは単一の責務のみを持つ

### 親仕様からの要件マッピング

本仕様は以下の親要件に対応する：
- **Requirement 1.1**: デスクトップ上に透過背景を持つキャラクター画像を表示する
- **Requirement 1.3**: 画像の透明部分を正しく透過処理して表示する
- **Requirement 2.4**: フレームアニメーション（連番画像）を再生できる **（将来対応）**

---

## Requirements

### Requirement 1: 非同期画像読み込み

**Objective:** 開発者として、画像読み込み中にWorldをブロックしたくない。それによりUIの応答性を維持できる。

#### Acceptance Criteria

1. **The** Image widget **shall** WintfTaskPool（専用TaskPool）を使用して画像を非同期に読み込む
2. **The** Image widget **shall** Bevyの標準タスクプール（ComputeTaskPool, IoTaskPool等）をブロックしない
3. **While** 画像読み込み中, **the** World **shall** 他の処理を継続できる
4. **When** 非同期読み込みが完了した時, **the** Image widget **shall** ImageResourceコンポーネントを更新する
5. **The** WintfTaskPool **shall** EcsWorldの初期化時にResourceとして初期化される

---

### Requirement 2: 静止画像読み込み

**Objective:** 開発者として、αチャンネルを持つ画像ファイルを読み込みたい。それによりキャラクター画像をウィジェットとして表示できる。

#### Acceptance Criteria

1. **The** Image widget **shall** 1つの画像ファイルのみを担当する（連番画像や組み合わせはウィジェットの組み合わせで実現）
2. **The** Image widget **shall** WICを使用してWindows 11標準でサポートされる画像形式を読み込める（PNG, TIFF, BMP, GIF, JPEG XR, ICO, WebP等）
3. **The** Image widget **shall** アルファチャンネルを持つ画像のみをサポートする
4. **The** Image widget **shall** WICオブジェクト（IWICBitmapSource）をImageResourceコンポーネントに保持する
5. **The** Image widget **shall** BGRAへの展開を行わず、WICからD2Dへの直接パスを使用する
6. **When** 画像にアルファチャンネルがない場合, **the** Image widget **shall** 明確なエラーを返す
7. **When** 画像ファイルが存在しない場合, **the** Image widget **shall** 明確なエラーを返す
8. **When** WICでデコードできない形式の場合, **the** Image widget **shall** 明確なエラーを返す
9. **When** エラー状態の場合, **the** Image widget **shall** 何も表示しない（プレースホルダー表示なし）

---

### Requirement 3: 透過処理

**Objective:** 開発者として、αチャンネル付き画像を正しく表示したい。それによりキャラクターの輪郭が自然に表示される。

#### Acceptance Criteria

1. **The** Image widget **shall** すべてのサポート形式でアルファチャンネルを正しく読み込める
2. **When** 画像に透明領域がある場合, **the** Image widget **shall** その領域を背景として透過表示する
3. **The** Image widget **shall** プリマルチプライドアルファを正しく処理する

---

### Requirement 4: Direct2D描画

**Objective:** 開発者として、読み込んだ画像をDirect2Dで高速に描画したい。それにより60fps以上の滑らかな表示が実現できる。

#### Acceptance Criteria

1. **The** Image widget **shall** CreateBitmapFromWicBitmapを使用してWICからID2D1Bitmapを直接作成する
2. **The** Image widget **shall** ImageGraphicsコンポーネントにID2D1Bitmapを保持する
3. **When** ImageResourceが変更された時, **the** システム **shall** ImageGraphicsを自動的に更新する（Changed検知）
4. **The** Image widget **shall** BoxStyleからのレイアウト計算結果に従って描画する
5. **The** Image widget **shall** 画像をOFFSET(0,0)からレイアウト指定サイズでクリッピングして描画する
6. **The** Image widget **shall** 描画開始OFFSETや描画域RECTのパラメータを持たない（P1で対応）
7. **The** Image widget **shall** デバイスロスト時にImageResourceからImageGraphicsを再作成できる
8. **While** 描画中, **the** Image widget **shall** 他のウィジェットと同様にレイアウトシステムと統合される

> **Note**: ストレッチモード、ソース矩形、補間モード、描画開始OFFSETは P1（wintf-P1-image-rendering）で対応予定

---

### Requirement 5: ECS統合

**Objective:** 開発者として、Image ウィジェットをECSコンポーネントとして使用したい。それにより既存のwintfアーキテクチャと統合できる。

#### Acceptance Criteria

1. **The** Image widget **shall** ImageResource（CPUリソース）とImageGraphics（GPUリソース）の2コンポーネント構成で実装される
2. **The** ImageResource **shall** WICオブジェクトを保持し、Send/Syncを実装する（WICはスレッドフリー）
3. **The** ImageGraphics **shall** ID2D1Bitmapとgenerationフィールドでデバイスロストに対応する
4. **The** Image widget **shall** 画像サイズに関わらずレイアウトサイズを持つ（BoxStyleによる指定）
5. **The** Image widget **shall** 既存のレイアウトシステム（taffy）と統合される
6. **The** Image widget **shall** Visual/Surface階層に正しく統合される
6. **When** エンティティが削除された時, **the** Image widget **shall** 関連リソース（WICオブジェクト、ID2D1Bitmap等）を正しく解放する
7. **The** Image widget **shall** 他のウィジェット（Label、Rectangle）と同様のAPI設計を持つ

---

### Requirement 6: 将来拡張性（アニメーション対応）

**Objective:** 開発者として、将来のアニメーション対応が容易な設計を確保したい。それにより「アニメーションできない設計」を回避できる。

#### Acceptance Criteria

1. **The** ImageResource **shall** 将来的にフレームカウントとフレーム遅延情報を追加できる設計とする
2. **The** ImageResource **shall** WICデコーダー（IWICBitmapDecoder）を保持できる構造とする（アニメーションフレーム切り替え用）
3. **The** ImageGraphics **shall** 現在フレームを動的に切り替えできる設計とする
4. **The** Image widget **shall** 画像の動的差し替え（ImageResourceの変更）に対応できる設計とする

---

## Non-Functional Requirements

### NFR-1: パフォーマンス

- 画像読み込み: 1MB未満の画像を100ms以内に読み込み可能
- 描画: 60fps以上を維持（複数Image表示時も）
- メモリ: 画像サイズに比例した適切なメモリ使用量

### NFR-2: 互換性

- Windows 11以降をサポート
- Direct2D 1.1 以降を使用
- WIC標準コーデック（Windows 11でサポートされるすべてのαチャンネル対応形式）をサポート

### NFR-3: エラーハンドリング

- ファイル読み込みエラー時に明確なエラーメッセージを提供
- デバイスロスト時の自動復旧をサポート

---

## Glossary

| 用語 | 説明 |
|------|------|
| WIC | Windows Imaging Component - Windowsの画像処理API |
| D2D | Direct2D - ハードウェアアクセラレーション2D描画API |
| プリマルチプライドアルファ | RGBにアルファ値が乗算済みの形式 |
| サーフェス | キャラクターの表情・ポーズを表す画像 |
| WintfTaskPool | wintf専用のバックグラウンドタスクプール（Bevyの標準プールとは独立） |
| ImageResource | WICオブジェクトを保持するCPUリソースコンポーネント |
| ImageGraphics | ID2D1Bitmapを保持するGPUリソースコンポーネント |
| スレッドフリー | COMのスレッドモデルで、どのスレッドからでもアクセス可能 |
| wintf-P1-image-rendering | 本仕様の次フェーズ。描画オプション（ストレッチモード等）を追加 |

---

## Appendix

### A. 関連ドキュメント

- 親仕様: `.kiro/specs/ukagaka-desktop-mascot/requirements.md`
- wintf設計: `doc/spec/06-visual-directcomp.md`
- ECSコンポーネント設計: `doc/spec/01-ecs-components.md`

### B. 技術参考

- [WIC Overview (MSDN)](https://docs.microsoft.com/en-us/windows/win32/wic/)
- [Direct2D Bitmap (MSDN)](https://docs.microsoft.com/en-us/windows/win32/direct2d/id2d1bitmap)

---

_Document generated by AI-DLC System on 2025-11-29_
