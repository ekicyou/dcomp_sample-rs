# Gap Analysis: wintf-P0-typewriter

| 項目 | 内容 |
|------|------|
| **Document Title** | wintf-P0-typewriter ギャップ分析 |
| **Version** | 1.0 |
| **Date** | 2025-11-29 |
| **Author** | AI-DLC System |

---

## Executive Summary

本ドキュメントは `wintf-P0-typewriter` 要件を既存コードベースと照合し、実装に必要な差分（ギャップ）を特定する。

**調査結果サマリ:**
- 既存アセット: Label/TextLayout基盤、Windows Animation APIラッパー、ECSスケジュール構造
- 主要ギャップ: DirectWrite クラスタ取得API未実装、IUIAnimationTimer 未統合、IR型定義なし
- 全体工数見積: **L** (1-2週間) ※設計+実装

---

## Requirement-to-Asset Map

### Requirement 1: 文字単位表示

| AC | 既存アセット | ギャップ | 工数 | リスク |
|----|-------------|---------|------|--------|
| 1.1 一文字ずつ表示 | `draw_labels.rs` (TextLayout描画) | Typewriter新コンポーネント必要 | M | Low |
| 1.2 文字追加再描画 | `GraphicsCommandList` (差分描画対応) | VisibleCharCount状態管理必要 | S | Low |
| 1.3 DirectWriteグリフ単位 | `dwrite.rs` (CreateTextLayout) | **GetClusterMetrics API未ラップ** | M | Medium |
| 1.4 改行処理 | `TextLayoutResource` | 改行込みクラスタ番号計算 | S | Low |
| 1.5 完了イベント | なし | **IR駆動イベント機構新規** | M | Medium |

**ギャップ詳細:**
- `IDWriteTextLayout::GetClusterMetrics()` が `dwrite.rs` に未実装
- `IDWriteTextLayout::HitTestTextPosition()` が `dwrite.rs` に未実装
- グリフ位置情報取得には両APIが必要

### Requirement 2: ウェイト制御

| AC | 既存アセット | ギャップ | 工数 | リスク |
|----|-------------|---------|------|--------|
| 2.1 デフォルトウェイト | なし | Typewriterコンポーネントに追加 | S | Low |
| 2.2 個別ウェイト | なし | Stage 1 IR Waitトークン | S | Low |
| 2.3 時間経過で次文字 | `animation.rs` (AnimationManager) | **IUIAnimationTimer未統合** | M | Medium |
| 2.4 瞬時表示 | なし | ウェイト0ハンドリング | S | Low |
| 2.5 スキップ機能 | なし | skip()メソッド追加 | S | Low |

**ギャップ詳細:**
- `IUIAnimationTimer` が `animation.rs` に未実装
- `IUIAnimationManager2::Update()` は存在するが、Timer連携未整備
- タイマーコールバック機構が必要

### Requirement 3: 2段階IR設計

| AC | 既存アセット | ギャップ | 工数 | リスク |
|----|-------------|---------|------|--------|
| 3.1-3.4 Stage 1 IR | なし | **TypewriterToken型定義新規** | S | Low |
| 3.5-3.8 Stage 2 IR | `TextLayoutResource` (IDWriteTextLayout保持) | **TimelineItem型定義新規** | M | Medium |

**ギャップ詳細:**
- 共有IR型は `wintf` 側に定義し、`areka-P0-script-engine` から参照
- Stage 2 IRはCOM依存（IDWriteTextLayout保持）で設計済み

### Requirement 4: 表示制御

| AC | 既存アセット | ギャップ | 工数 | リスク |
|----|-------------|---------|------|--------|
| 4.1-4.6 操作API | なし | TypewriterState列挙型 + メソッド | S | Low |

**ギャップ詳細:**
- 単純なステートマシン実装で対応可能
- `start()`, `pause()`, `resume()`, `skip()`, `clear()` メソッド

### Requirement 5: IR駆動イベント

| AC | 既存アセット | ギャップ | 工数 | リスク |
|----|-------------|---------|------|--------|
| 5.1-5.4 FireEventトークン | なし | **ECS Commands注入機構** | M | Medium |
| 5.5 別System委譲 | ECSシステム構造 | Observer/on_addパターン流用 | S | Low |
| 5.6 進行度公開 | なし | TypewriterProgress コンポーネント | S | Low |

**ギャップ詳細:**
- `FireEvent` トークン処理時に `Commands::entity(target).insert(Component)` を呼ぶ
- 既存 `on_add` フックパターンで後続処理を実装

### Requirement 6: Label互換性

| AC | 既存アセット | ギャップ | 工数 | リスク |
|----|-------------|---------|------|--------|
| 6.1 テキスト設定API | `Label` コンポーネント | 継承/共通trait抽出 | S | Low |
| 6.2 スタイル設定 | `Label` (font_family, font_size, color) | 同上 | S | Low |
| 6.3 縦書き/横書き | `TextDirection`, `draw_labels.rs` | 流用可能 | S | Low |
| 6.4 レイアウト統合 | Taffy統合、`Arrangement` | 流用可能 | S | Low |
| 6.5 即時表示モード | なし | skip()呼び出しで対応 | S | Low |

**ギャップ詳細:**
- 既存 `Label` の実装を参照し、共通部分をtraitに抽出推奨

### Requirement 7: ECS統合

| AC | 既存アセット | ギャップ | 工数 | リスク |
|----|-------------|---------|------|--------|
| 7.1 ECSコンポーネント | ECS基盤完備 | Typewriterコンポーネント定義 | S | Low |
| 7.2 Windows Animation | `animation.rs` (Manager, Storyboard) | **Timer統合** | M | Medium |
| 7.3 Animation状態参照 | `UIAnimationVariableExt::get_value()` | 流用可能 | S | Low |
| 7.4 クリーンアップ | `on_remove` フックパターン | 流用可能 | S | Low |
| 7.5 ライフサイクル | `Visual`, `VisualGraphics` パターン | 流用可能 | S | Low |

---

## Gap Summary

### 必須実装項目

| ID | 項目 | 説明 | 工数 | リスク |
|----|------|------|------|--------|
| G1 | DirectWrite Cluster API | `GetClusterMetrics()`, `HitTestTextPosition()` ラッパー追加 | M | Medium |
| G2 | IUIAnimationTimer | Timer作成・コールバック・DComp連携 | M | Medium |
| G3 | Stage 1 IR型定義 | `TypewriterToken` enum (Text, Wait, FireEvent) | S | Low |
| G4 | Stage 2 IR型定義 | `TimelineItem` struct (グリフ情報含む) | M | Low |
| G5 | Typewriterコンポーネント | 状態管理、描画制御 | M | Low |
| G6 | FireEventシステム | コンポーネント注入機構 | M | Medium |
| G7 | Typewriter描画システム | draw_labels類似、進行状態反映 | M | Low |

### 流用可能アセット

| アセット | 場所 | 流用方法 |
|----------|------|----------|
| TextLayoutResource | `label.rs` | Stage 2 IRで保持 |
| TextDirection | `label.rs` | そのまま流用 |
| GraphicsCommandList | `command_list.rs` | 描画システムで使用 |
| VisualGraphics | `components.rs` | Visual統合パターン |
| Animation Wrappers | `animation.rs` | Manager/Variable/Storyboard流用 |
| ECSスケジュール | `world.rs` | Draw/Update等に登録 |

---

## Implementation Options

### Option A: 最小実装（Timer使用しない）

**概要:** ECS Updateスケジュールでフレームカウントベースの疑似タイマーを使用

**Pros:**
- IUIAnimationTimer実装不要
- 既存フレームワーク内で完結

**Cons:**
- フレームレート依存（60fps固定前提）
- Windows Animation APIとの一貫性なし
- NFR-3精度要件に影響

**工数:** S  
**リスク:** Medium（精度・一貫性問題）

### Option B: IUIAnimationTimer統合（推奨）

**概要:** Windows Animation APIのTimerを使用し、DCompコミットと同期

**Pros:**
- 要件通りの高精度タイミング
- DirectComposition Animationとの自然な統合
- 既存animation.rsの拡張として実装

**Cons:**
- Timer COM統合の実装コスト
- タイマーコールバック機構の設計必要

**工数:** M  
**リスク:** Low（既存パターンに沿った拡張）

### Option C: カスタムタイマースレッド

**概要:** 専用スレッドでタイマー管理、ECS Worldへイベント送信

**Pros:**
- 完全な制御可能

**Cons:**
- 複雑度増加
- スレッド間同期オーバーヘッド
- 既存アーキテクチャから逸脱

**工数:** L  
**リスク:** High

### 推奨: Option B

理由:
- 要件定義で「Windows Animation API を時間管理の正として使用」と明記
- 既存 `animation.rs` の自然な拡張
- DirectComposition統合を維持

---

## Risk Assessment

| リスク | 影響 | 発生確率 | 緩和策 |
|--------|------|----------|--------|
| GetClusterMetrics APIの複雑さ | 縦書き時のクラスタ位置計算 | Medium | HitTestTextPosition併用、テストケース充実 |
| Timer統合のデバッグ難度 | コールバックタイミング問題 | Low | ログ出力、単体テスト |
| IR型定義の変更 | areka-P0-script-engineとの互換性 | Low | crate分離、バージョニング |
| パフォーマンス | 長文テキストでの再描画コスト | Low | 差分描画、クリッピング |

---

## Effort Estimation

| フェーズ | 項目 | 工数 |
|----------|------|------|
| 設計 | design.md作成 | S |
| 実装 | G1: DirectWrite API | M |
| 実装 | G2: Timer統合 | M |
| 実装 | G3-G4: IR型定義 | S |
| 実装 | G5-G7: Typewriterコンポーネント・システム | M |
| テスト | 単体テスト・統合テスト | M |
| **合計** | | **L** (1-2週間) |

---

## Appendix: コードベース調査結果

### 調査対象ファイル

1. `crates/wintf/src/ecs/widget/text/label.rs`
   - `Label` コンポーネント（font_family, font_size, color, direction）
   - `TextLayoutResource`（IDWriteTextLayout保持）
   - `TextDirection` enum（Horizontal, Vertical）

2. `crates/wintf/src/ecs/widget/text/draw_labels.rs`
   - `draw_labels()` システム
   - TextFormat/TextLayout作成フロー
   - GraphicsCommandList使用パターン

3. `crates/wintf/src/com/animation.rs`
   - `UIAnimationManagerExt` (create_animation_variable, update, create_storyboard)
   - `UIAnimationStoryboardExt` (schedule, add_transition)
   - `UIAnimationVariableExt` (get_value, get_curve)
   - **IUIAnimationTimer未実装**

4. `crates/wintf/src/com/dwrite.rs`
   - `DWriteFactoryExt` (create_text_format, create_text_layout)
   - `DWriteTextFormatExt` (set_text_alignment, set_paragraph_alignment)
   - **GetClusterMetrics, HitTestTextPosition未実装**

5. `crates/wintf/src/ecs/world.rs`
   - ECSスケジュール: Input → Update → PreLayout → Layout → PostLayout → UISetup → GraphicsSetup → Draw → PreRenderSurface → RenderSurface → Composition → CommitComposition → FrameFinalize
   - Typewriterは `Update` (時間計算) + `Draw` (描画) に登録予定

6. `crates/wintf/src/ecs/graphics/components.rs`
   - `VisualGraphics` (IDCompositionVisual3保持、on_removeフック)
   - `HasGraphicsResources` マーカー
   - 流用可能なパターン多数
