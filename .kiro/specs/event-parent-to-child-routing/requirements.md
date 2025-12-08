# Requirements Document

## Project Description (Input)
イベントルーティングにおいて、親から子方向のイベント通知を実現して欲しい。具体的には、現在子⇒親への通知が実現しているので、あらかじめvecで子から親までのエンティティをリストアップし、現在の「子⇒親」のイベント通知処理に先立って、リストアップしたエンティティを利用した親⇒子通知を実現できると考える。

## Introduction

本仕様書は、wintfフレームワークのイベントシステムにおける親→子方向のイベント伝播（Tunnelフェーズ）のデモンストレーション実装の要件を定義する。`dispatch_pointer_events`システムはTunnel/Bubble両フェーズを完全実装済みだが、`taffy_flex_demo`サンプルはBubbleフェーズのみをデモしている。本仕様ではTunnelフェーズの動作を視覚的に確認できるデモを追加する。

### 背景

**実装状況**:
- ✅ `dispatch_pointer_events`はTunnel（親→子）→Bubble（子→親）の2フェーズディスパッチを完全実装
- ✅ `Phase::Tunnel`/`Phase::Bubble`列挙型とハンドラシグネチャは定義済み
- ✅ ユニットテスト（`test_dispatch_stop_propagation`等）でTunnel/Bubble動作を検証済み
- ❌ `taffy_flex_demo`は全ハンドラで`if !ev.is_bubble() { return false; }`としてTunnelを無視

**問題点**:
開発者がTunnelフェーズの動作を視覚的に理解できるデモが不足している。

### スコープ

**含まれるもの**:
- `taffy_flex_demo`にTunnelフェーズのデモハンドラを追加
- Tunnel/Bubbleの実行順序を視覚的に確認できるログ出力
- 親エンティティでTunnelフェーズを処理するサンプル実装
- Tunnelフェーズでの伝播停止（stopPropagation）のデモ

**含まれないもの**:
- `dispatch_pointer_events`の実装変更（既に完全実装済み）
- 新しいイベント種別の追加
- ハンドラシグネチャの変更
- 既存のBubbleフェーズデモの削除

### 関連仕様

- **親仕様**: `wintf-P0-event-system` (Requirement 7: イベント配信機構)
- **実装ファイル**: `crates/wintf/src/ecs/pointer/dispatch.rs`

---

## Requirements

### Requirement 1: Tunnelフェーズのデモハンドラ追加

**Objective:** 開発者として、Tunnelフェーズがどのように動作するかを視覚的に理解したい。それにより親エンティティでイベントを事前処理するパターンを学べる。

#### Acceptance Criteria

1. **The** taffy_flex_demo **shall** 少なくとも1つのハンドラでTunnelフェーズを処理する（`if ev.is_tunnel()`分岐を実装）
2. **When** Tunnelフェーズが実行された時, **the** taffy_flex_demo **shall** コンソールに「[Tunnel]」プレフィックス付きログを出力する
3. **When** Bubbleフェーズが実行された時, **the** taffy_flex_demo **shall** コンソールに「[Bubble]」プレフィックス付きログを出力する
4. **The** taffy_flex_demo **shall** 親エンティティ（FlexDemoContainer）と子エンティティ（RedBox等）の両方でハンドラを登録し、実行順序を明示する
5. **The** taffy_flex_demo **shall** Tunnel/Bubbleの実行順序がコンソールログから理解できるようにする（例: Window→Container→RedBox→Container→Window）

---

### Requirement 2: Tunnelフェーズでのイベント前処理デモ

**Objective:** 開発者として、親エンティティでイベントを事前処理するユースケースを理解したい。それにより子エンティティに到達する前に条件判定や状態チェックを行う方法を学べる。

#### Acceptance Criteria

1. **The** taffy_flex_demo **shall** FlexDemoContainerのTunnelフェーズハンドラで、特定条件下でイベントを子に伝播させない例を実装する
2. **When** Tunnelフェーズで伝播が停止された時, **the** taffy_flex_demo **shall** 「[Tunnel] Event stopped at Container」のようなログを出力する
3. **The** taffy_flex_demo **shall** Ctrlキー押下時などの条件でTunnel stopPropagationをデモする
4. **When** Tunnelで伝播が停止された時, **the** taffy_flex_demo **shall** Bubbleフェーズが実行されないことをログで確認できるようにする
5. **The** taffy_flex_demo **shall** 通常ケース（伝播継続）とstopPropagationケースの両方をデモする

---

### Requirement 3: 階層的イベント処理のデモ

**Objective:** 開発者として、エンティティ階層でのイベント処理順序を理解したい。それにより複雑なUI構造でのイベントフローを設計できる。

#### Acceptance Criteria

1. **The** taffy_flex_demo **shall** Window → Container → ChildWidget の3階層でTunnel/Bubbleを実行する
2. **The** taffy_flex_demo **shall** コンソールログにエンティティ名とフェーズを明示する（例: "[Tunnel] Window", "[Tunnel] Container", "[Tunnel] RedBox", "[Bubble] RedBox", "[Bubble] Container", "[Bubble] Window"）
3. **The** taffy_flex_demo **shall** 各階層のハンドラで`sender`と`entity`の違いを説明するコメントを含める
4. **The** taffy_flex_demo **shall** クリック時にどのエンティティがイベント発生元（sender）かをログ出力する
5. **The** taffy_flex_demo **shall** 実行順序がWinUI3/WPF/DOMイベントモデルと一致することをコメントで明記する

---

### Requirement 4: PointerStateアクセスのデモ

**Objective:** 開発者として、Tunnel/Bubbleフェーズで同じPointerStateにアクセスできることを理解したい。それにより各フェーズで座標やボタン状態を参照する方法を学べる。

#### Acceptance Criteria

1. **The** taffy_flex_demo **shall** TunnelフェーズでPointerStateの情報（座標、ボタン状態等）をログ出力する
2. **The** taffy_flex_demo **shall** BubbleフェーズでPointerStateの情報をログ出力し、Tunnelと同じ値であることを示す
3. **The** taffy_flex_demo **shall** `ev.value()`メソッドを使用してPhaseに関係なくPointerStateにアクセスする例を実装する
4. **The** taffy_flex_demo **shall** 左クリック、右クリック、Ctrl/Shift修飾キーの状態をログ出力する
5. **The** taffy_flex_demo **shall** screen_pointとlocal_pointの両方をログ出力し、座標変換を示す

---

### Requirement 5: ドキュメントとコメントの追加

**Objective:** 開発者として、Tunnelフェーズの実装を理解しやすいドキュメントが欲しい。それによりサンプルコードを参照して自分のアプリケーションに適用できる。

#### Acceptance Criteria

1. **The** taffy_flex_demo **shall** ファイル冒頭にTunnel/Bubbleフェーズの説明コメントを追加する
2. **The** taffy_flex_demo **shall** WinUI3/WPF/DOMイベントモデルとの対応関係をコメントで説明する
3. **The** taffy_flex_demo **shall** 各ハンドラ関数にTunnel/Bubbleでの処理意図をコメントで記述する
4. **The** taffy_flex_demo **shall** Tunnel stopPropagationの使用例とユースケースをコメントで説明する
5. **The** taffy_flex_demo **shall** 実行時の出力例をコメントまたはprintln!で示す（例: 「Expected output: [Tunnel] Window → [Tunnel] Container → ...」）

---

## Non-Functional Requirements

### NFR-1: パフォーマンス

- エンティティパス構築: イベントディスパッチあたり1回のみ
- Tunnel/Bubbleフェーズ合計: 既存Bubbleのみの場合と比較して2倍以内のオーバーヘッド
- メモリ: パス保持用の`Vec<Entity>`のアロケーションのみ（追加のヒープ確保なし）

### NFR-2: 互換性

- 既存のイベントハンドラコードが無修正で動作
- `Phase<T>`列挙型の`is_tunnel()`/`is_bubble()`メソッドが正しく機能
- 段階的な移行が可能（Tunnelフェーズを使わないハンドラも共存可能）

### NFR-3: コード品質

- 既存の`dispatch_event_for_handler`関数の構造を維持
- Tunnel/Bubbleのループロジックが対称的で理解しやすい
- ドキュメントコメントでWinUI3/WPFとの対応関係を明記

---

## Glossary

| 用語 | 説明 |
|------|------|
| Tunnel / Capture | 親→子方向のイベント伝播（WinUI3 PreviewXxx, DOM Capture相当） |
| Bubble | 子→親方向のイベント伝播（通常のイベント、WinUI3 Xxx相当） |
| Phase | イベント伝播の段階（Tunnel または Bubble） |
| Sender | イベントの発生元エンティティ（ヒットテスト結果、WinUI3 OriginalSource相当） |
| Entity | 現在処理中のエンティティ（バブリング中に変化、WinUI3 sender相当） |
| Path | senderからrootまでの親チェーン（`Vec<Entity>`） |

---

## Appendix

### A. 関連ドキュメント

- 親仕様: `.kiro/specs/wintf-P0-event-system/requirements.md` (Requirement 7)
- 実装ファイル: `crates/wintf/src/ecs/pointer/dispatch.rs`
- WinUI3 Routed Events: https://learn.microsoft.com/en-us/windows/apps/design/controls/routing-events-overview

### B. WinUI3/WPF対応表

| wintf | WinUI3/WPF | DOM |
|-------|------------|-----|
| Phase::Tunnel | PreviewXxx イベント | Capture Phase |
| Phase::Bubble | Xxx イベント | Bubble Phase |
| sender (引数) | OriginalSource | event.target |
| entity (引数) | sender | event.currentTarget |

### C. 実装例: 既存コードとの対比

**現在の実装（Bubbleのみ）**:
```rust
// Bubble フェーズのみ: sender → root
for &entity in path.iter() {
    if let Some(handler_comp) = world.get::<H>(entity).copied() {
        let handler = get_handler(&handler_comp);
        if handler(world, sender, entity, &Phase::Bubble(state.clone())) {
            return; // handled
        }
    }
}
```

**改修後の実装（Tunnel + Bubble）**:
```rust
// Tunnel フェーズ: root → sender
for &entity in path.iter().rev() {
    if let Some(handler_comp) = world.get::<H>(entity).copied() {
        let handler = get_handler(&handler_comp);
        if handler(world, sender, entity, &Phase::Tunnel(state.clone())) {
            return; // handled
        }
    }
}

// Bubble フェーズ: sender → root
for &entity in path.iter() {
    if let Some(handler_comp) = world.get::<H>(entity).copied() {
        let handler = get_handler(&handler_comp);
        if handler(world, sender, entity, &Phase::Bubble(state.clone())) {
            return; // handled
        }
    }
}
```

---

_Document generated by AI-DLC System on 2025-12-08_
