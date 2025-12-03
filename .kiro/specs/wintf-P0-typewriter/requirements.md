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
- 文字単位の表示制御（一文字ずつ追加表示）
- ウェイト制御（文字間の待機時間）
- さくらスクリプト互換のウェイトコマンド（\w, \_w）
- 表示完了コールバック

**含まれないもの**:
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

### Requirement 3: 中間表現（IR）入力

**Objective:** 描画エンジンとして、パース済みの構造化データを受け取りたい。それによりTypewriterは表示アニメーションに専念できる。

#### Acceptance Criteria

1. **The** Typewriter widget **shall** 中間表現（IR）形式でトークン列を受け取れる
2. **The** Typewriter widget **shall** テキストトークン（表示文字列）を処理できる
3. **The** Typewriter widget **shall** ウェイトトークン（待機時間）を処理できる
4. **When** ウェイトトークンを受け取った時, **the** Typewriter widget **shall** 指定時間だけ次のトークン処理を遅延する
5. **The** Typewriter widget **shall** IR型定義を `areka-P0-script-engine` と共有する
6. **The** Typewriter widget **shall** 将来の拡張トークン（速度変更、ポーズ等）に対応可能な設計とする

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

### Requirement 5: コールバック・イベント

**Objective:** 開発者として、表示の進行状況を監視したい。それにより表示完了後の処理やプログレス表示が可能になる。

#### Acceptance Criteria

1. **When** 文字が表示された時, **the** Typewriter widget **shall** 文字表示イベントを発火する
2. **When** 全文字の表示が完了した時, **the** Typewriter widget **shall** 完了イベントを発火する
3. **When** ウェイト中にスキップされた時, **the** Typewriter widget **shall** スキップイベントを発火する
4. **The** Typewriter widget **shall** 現在の表示進行度（0.0〜1.0）を取得できる
5. **The** Typewriter widget **shall** 残り表示時間（ウェイト含む）を推定できる

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
2. **The** Typewriter widget **shall** ECSシステムによってタイマー駆動される
3. **The** Typewriter widget **shall** ECSイベントを通じて進行状況を通知する
4. **When** エンティティが削除された時, **the** Typewriter widget **shall** 関連するタイマーをクリーンアップする
5. **The** Typewriter widget **shall** 他のコンポーネント（Visual、Layout等）と同様のライフサイクルを持つ

---

## Non-Functional Requirements

### NFR-1: アニメーション基盤

- **タイマー方式**: DirectComposition Animation Timer + Windows Animation API を使用
- **更新単位**: アニメーションフレーム単位（通常60fps = 約16.67ms間隔）
- **ウェイト計算**: 指定ミリ秒をフレーム数に変換（例: 50ms → 3フレーム）
- **描画同期**: DCompのコミットタイミングに同期

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
| タイプライター効果 | テキストを一文字ずつ表示する演出 |
| ウェイト | 文字表示間の待機時間 |
| 中間表現（IR） | Intermediate Representation。パース済みの構造化データ |
| TypewriterToken | IR内の個別要素（Text, Wait等） |
| グリフ | DirectWriteにおける描画単位。合字・結合文字を含む |

---

## Appendix

### A. 関連ドキュメント

- 親仕様: `.kiro/specs/ukagaka-desktop-mascot/requirements.md`
- Labelウィジェット実装: `crates/wintf/src/ecs/widget/label.rs`
- IR型定義共有先: `areka-P0-script-engine`

### B. 中間表現（IR）例

```rust
/// Typewriterが受け取るトークン
pub enum TypewriterToken {
    /// 表示するテキスト
    Text(String),
    /// ウェイト（待機時間）
    Wait(Duration),
    // 将来拡張:
    // Speed(f32),     // 表示速度変更
    // Pause,          // ユーザー入力待ち
}

/// 入力例: "こんにちは、今日もいい天気ですね。"（500ms + 200ms ウェイト付き）
let tokens = vec![
    TypewriterToken::Text("こんにちは".into()),
    TypewriterToken::Wait(Duration::from_millis(500)),
    TypewriterToken::Text("、今日も".into()),
    TypewriterToken::Wait(Duration::from_millis(200)),
    TypewriterToken::Text("いい天気ですね。".into()),
];
```

---

_Document generated by AI-DLC System on 2025-11-29_
