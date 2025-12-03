# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | wintf-typewriter 要件定義書 |
| **Version** | 0.1 (Draft) |
| **Date** | 2025-11-29 |
| **Parent Spec** | ukagaka-desktop-mascot |
| **Author** | AI-DLC System |

---

## Introduction

本仕様書は wintf フレームワークにおけるタイプライター表示機能の要件を定義する。親仕様「伺的デスクトップマスコットアプリ」の実装前提条件（P0）として、文字単位の表示制御とウェイト制御機能を提供する。

### 背景

wintf フレームワークの既存 Label ウィジェットは静的テキスト表示のみをサポートしている。デスクトップマスコットアプリケーションでは、キャラクターの発言を一文字ずつ表示する「タイプライター効果」が伝統的かつ重要なUI要素である。これにより、キャラクターが「話している」という臨場感を演出できる。

### スコープ

**含まれるもの**:
- アニメーション基盤 `AnimationCore`（ECSリソース）
- 文字単位の表示制御（一文字ずつ追加表示）
- ウェイト制御（文字間の待機時間）
- さくらスクリプト互換のウェイトコマンド（\w, \_w）
- 表示完了コールバック

**含まれないもの**:
- 高度なアニメーション機能（→ `wintf-P0-animation-system` で拡張予定）
- テキストのフェードイン/アウト効果
- テキストアニメーション（揺れ、拡大縮小等）
- 音声同期

### 親仕様からの要件マッピング

本仕様は以下の親要件に対応する：
- **Requirement 3.5**: 一文字ずつタイプライター風に表示できる
- **Requirement 4.7**: ウェイト処理（タイミング制御）

---

## Requirements

### Requirement 1: 文字単位表示

**Objective:** 開発者として、テキストを一文字ずつ順番に表示したい。それによりキャラクターが話しているような演出が可能になる。

#### Acceptance Criteria

1. **The** Typewriter widget **shall** テキストを一文字ずつ順番に表示できる
2. **When** 新しい文字が追加された時, **the** Typewriter widget **shall** 既存のテキストに文字を追加して再描画する
3. **The** Typewriter widget **shall** DirectWriteのグリフ単位で文字を扱う（合字・結合文字・絵文字シーケンスは1つのアニメーション単位として表示）
4. **The** Typewriter widget **shall** 改行文字を正しく処理する
5. **When** 全文字の表示が完了した時, **the** Typewriter widget **shall** 完了イベントを発火する

---

### Requirement 2: ウェイト制御

**Objective:** 開発者として、文字間の待機時間を制御したい。それにより自然な会話リズムを表現できる。

#### Acceptance Criteria

1. **The** Typewriter widget **shall** デフォルトの文字間ウェイト時間を設定できる（例: 50ms）
2. **The** Typewriter widget **shall** 個別の文字に対してウェイト時間を指定できる
3. **When** ウェイト時間が経過した時, **the** Typewriter widget **shall** 次の文字を表示する
4. **The** Typewriter widget **shall** ウェイト時間0（瞬時表示）をサポートする
5. **The** Typewriter widget **shall** ウェイト中に表示をスキップできる

---

### Requirement 3: 2段階IR設計

**Objective:** 描画エンジンとして、パース済みの構造化データを受け取り、内部でグリフ単位に分解したい。それによりTypewriterは表示アニメーションに専念できる。

#### Acceptance Criteria

**Stage 1 IR（外部インターフェース）:**
1. **The** Typewriter widget **shall** Stage 1 IR形式でトークン列を受け取れる
2. **The** Typewriter widget **shall** テキストトークン（表示文字列）を処理できる
3. **The** Typewriter widget **shall** ウェイトトークン（f64秒単位、Windows Animation API互換）を処理できる
4. **The** Typewriter widget **shall** Stage 1 IR型定義を `areka-P0-script-engine` と共有する

**Stage 2 IR（内部タイムライン）:**
5. **The** Typewriter widget **shall** Stage 1 IRをDirectWriteでグリフ単位に分解しStage 2 IRを生成する
6. **The** Typewriter widget **shall** Stage 2 IRにグリフ情報（TextLayout内のクラスタ番号）を含める
7. **The** Typewriter widget **shall** Stage 2 IRでウェイトをf64秒単位で保持する
8. **The** Typewriter widget **shall** Stage 2 IRで縦書き/横書きの位置情報をTextLayoutから取得する

---

### Requirement 4: 表示制御

**Objective:** 開発者として、タイプライター表示の開始・停止・リセットを制御したい。それにより会話の流れを適切に管理できる。

#### Acceptance Criteria

1. **The** Typewriter widget **shall** 表示開始（start）操作を提供する
2. **The** Typewriter widget **shall** 表示停止（pause）操作を提供する
3. **The** Typewriter widget **shall** 表示再開（resume）操作を提供する
4. **The** Typewriter widget **shall** 全文即時表示（skip）操作を提供する
5. **The** Typewriter widget **shall** テキストクリア（clear）操作を提供する
6. **When** 新しいテキストがセットされた時, **the** Typewriter widget **shall** 表示位置をリセットする

---

### Requirement 5: IR駆動イベント

**Objective:** 開発者として、テキスト表示の進行に応じてイベントを発火したい。それにより表示完了後の処理や状態変更が可能になる。

#### Acceptance Criteria

1. **The** Typewriter widget **shall** IRトークンにイベント発火コマンドを含められる
2. **When** FireEventトークンを処理した時, **the** Typewriter widget **shall** 指定されたエンティティにコンポーネントを投入する
3. **The** Typewriter widget **shall** 表示完了時のイベント発火をIRで記述できる
4. **The** Typewriter widget **shall** 任意のタイミング（特定文字後等）でのイベント発火をIRで記述できる
5. **The** Typewriter widget **shall** 投入されたコンポーネントの処理は別Systemに委譲する
6. **The** Typewriter widget **shall** 現在の表示進行度（0.0〜1.0）をコンポーネントとして公開する

---

### Requirement 6: Label互換性

**Objective:** 開発者として、既存のLabelウィジェットと互換性のあるAPIを使用したい。それにより学習コストを抑えられる。

#### Acceptance Criteria

1. **The** Typewriter widget **shall** Labelと同様のテキスト設定APIを提供する
2. **The** Typewriter widget **shall** Labelと同様のスタイル設定（フォント、色、サイズ）をサポートする
3. **The** Typewriter widget **shall** 縦書き/横書きの両方をサポートする
4. **The** Typewriter widget **shall** Labelと同様にレイアウトシステムと統合される
5. **When** タイプライター効果が不要な場合, **the** Typewriter widget **shall** 即時全文表示モードで動作する

---

### Requirement 7: ECS統合

**Objective:** 開発者として、Typewriterウィジェットをbevy_ecsと統合したい。それにより既存のwintfアーキテクチャと一貫性を保てる。

#### Acceptance Criteria

1. **The** Typewriter widget **shall** ECSコンポーネントとして実装される
2. **The** Typewriter widget **shall** Windows Animation API を時間管理の正として使用する
3. **The** Typewriter widget **shall** ECSシステムでAnimation APIの状態を参照し表示を更新する
4. **When** エンティティが削除された時, **the** Typewriter widget **shall** 関連するアニメーションリソースをクリーンアップする
5. **The** Typewriter widget **shall** 他のコンポーネント（Visual、Layout等）と同様のライフサイクルを持つ

---

## Non-Functional Requirements

### NFR-1: アニメーション基盤

### NFR-1: アニメーション基盤 (AnimationCore)

- **リソース構成**: `AnimationCore` ECSリソースとして実装
  - `IUIAnimationTimer`: システム時刻取得
  - `IUIAnimationManager2`: アニメーション状態管理
  - `IUIAnimationTransitionLibrary2`: トランジション生成
- **初期化タイミング**: `EcsWorld::new()` で即座に初期化（CPUリソースのみのため）
  - `GraphicsCore`（GPUリソース）とは異なり、HWND不要・Device Lost無関係
  - `WicCore` と同様のパターン
- **タイマー方式**: Windows Animation API を時間管理の正として使用
- **更新タイミング**: `animation_tick_system` を Input スケジュール先頭で実行
- **時刻精度**: f64秒単位 (`UI_ANIMATION_SECONDS`)
- **拡張性**: 将来 `wintf-P0-animation-system` で高度な機能を追加予定

### NFR-2: パフォーマンス

- 描画更新: 文字追加ごとに1フレーム以内で再描画
- メモリ: 表示テキスト長に比例した適切なメモリ使用量
- CPU負荷: アイドル時はタイマーイベントを発生させない

### NFR-3: 精度

- フレーム精度: ±1フレーム以内の誤差
- Unicode対応: DirectWriteグリフ単位での正しい処理

### NFR-4: 互換性

- さくらスクリプトの基本ウェイトコマンド（\w, \_w）との互換性
- 既存Labelウィジェットとの共存

---

## Glossary

| 用語 | 説明 |
|------|------|
| AnimationCore | Windows Animation API を統合したECSリソース。Timer/Manager/TransitionLibraryを保持 |
| タイプライター効果 | テキストを一文字ずつ表示する演出 |
| ウェイト | 文字表示間の待機時間（f64秒単位） |
| Stage 1 IR | 外部インターフェース用の中間表現。Text, Wait, FireEvent等 |
| Stage 2 IR | 内部タイムライン用の中間表現。グリフ単位に分解済み |
| TypewriterToken | Stage 1 IR内の個別要素 |
| TimelineItem | Stage 2 IR内の個別要素 |
| グリフ | DirectWriteにおける描画単位。合字・結合文字を含む |
| FireEvent | IRトークンの一種。指定エンティティにコンポーネントを投入する |
| 論理エンティティ | Visualツリーに参加しない、処理用のエンティティ |

---

## Appendix

### A. 関連ドキュメント

- 親仕様: `.kiro/specs/ukagaka-desktop-mascot/requirements.md`
- Labelウィジェット実装: `crates/wintf/src/ecs/widget/label.rs`
- IR型定義共有先: `areka-P0-script-engine`
- Windows Animation API: `crates/wintf/src/com/animation.rs`

### B. 2段階IR設計例

```rust
// ============================================
// Stage 1 IR (外部インターフェース)
// Script Engine から受け取る形式
// ============================================
pub enum TypewriterToken {
    /// 表示するテキスト
    Text(String),
    /// ウェイト（f64秒単位、Windows Animation API互換）
    Wait(f64),
    /// イベント発火（対象エンティティにコンポーネント投入）
    FireEvent {
        target: Entity,
        event_type: String,
        payload: Option<Value>,
    },
}

/// Stage 1 入力例: "こんにちは、今日もいい天気ですね。"
let stage1_tokens = vec![
    TypewriterToken::Text("こんにちは".into()),
    TypewriterToken::Wait(0.5),  // 500ms = 0.5秒
    TypewriterToken::Text("、今日も".into()),
    TypewriterToken::Wait(0.2),  // 200ms = 0.2秒
    TypewriterToken::Text("いい天気ですね。".into()),
    TypewriterToken::FireEvent {
        target: callback_entity,
        event_type: "talk_complete".into(),
        payload: None,
    },
];

// ============================================
// Stage 2 IR (内部タイムライン)
// DirectWriteでグリフ単位に分解後の形式
// ============================================
pub enum TimelineItem {
    /// グリフ表示（TextLayout内のクラスタ番号）
    Glyph { cluster_index: u32 },
    /// ウェイト（f64秒単位）
    Wait(f64),
    /// イベント発火
    FireEvent {
        target: Entity,
        event_type: String,
        payload: Option<Value>,
    },
}

pub struct TypewriterTimeline {
    /// 全文のTextLayout（位置情報源、縦書き/横書き対応）
    pub text_layout: IDWriteTextLayout,
    /// タイムライン
    pub items: Vec<TimelineItem>,
}

/// Stage 2 変換後:
/// "こんにちは" → [Glyph(0), Glyph(1), Glyph(2), Glyph(3), Glyph(4)]
/// Wait(0.5)   → Wait(0.5)
/// "、今日も"   → [Glyph(5), Glyph(6), Glyph(7), Glyph(8)]
/// ...
```

---

_Document generated by AI-DLC System on 2025-11-29_
