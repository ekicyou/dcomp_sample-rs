# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | event-dispatch 要件定義書 |
| **Version** | 0.2 (Draft) |
| **Date** | 2025-12-03 |
| **Parent Spec** | wintf-P0-event-system |
| **Author** | AI-DLC System |

---

## Introduction

本仕様書は wintf フレームワークにおけるイベント配信機構とECS統合の要件を定義する。親仕様「wintf-P0-event-system」の Requirement 7（イベント配信機構）および Requirement 8（ECS統合）を実装対象とする。

### 背景

wintf フレームワークでは、既存の `mouse.rs` モジュールおよび `handlers.rs` において:
- Win32 メッセージをECSコンポーネント（`MouseState`）に変換
- ヒットテストにより対象エンティティを特定し、`MouseState` を付与

ここまでは実装済みである。本仕様では、その先のイベント伝播機構（バブリング/キャプチャ）およびハンドラディスパッチを実現する。

### スコープ

**含まれるもの**:
- イベント伝播機構（バブリング: 子→親、キャプチャ: 親→子）
- イベントハンドラコンポーネントとディスパッチシステム
- イベント配信停止（ハンドラ戻り値による制御）
- 既存ECSシステム（`window.rs`, `layout/`）との統合
- イベント履歴保持（デバッグ用）
- エンティティ削除時のイベントリスナー解除

**含まれないもの**:
- ヒットテストロジック（`event-hit-test` 仕様で実装済み）
- Win32メッセージ→MouseState変換（`event-mouse-basic` 仕様で実装済み）
- キーボード/タッチイベント（将来の拡張）

### 親仕様からの要件マッピング

本仕様は以下の親要件に対応する：
- **Requirement 7**: イベント配信機構
- **Requirement 8**: ECS統合

### 設計方針（議論結果）

| 項目 | 決定内容 |
|------|---------|
| **ハンドラ型** | `fn(&mut World, Entity, &EventContext) -> bool` |
| **戻り値** | `true` = ハンドル済み（伝播停止）、`false` = 未処理（続行） |
| **状態管理** | ハンドラはステートレス、状態はコンポーネントに分離（ECS原則） |
| **伝播方式** | 2パス方式（ハンドラ収集→実行）でフレーム遅延なし |
| **システム種別** | 排他システム（`&mut World`）— イベント伝播の性質上必須 |
| **優先度** | バブリング（高）→ stopPropagation（中）→ キャプチャ（低） |

---

## Requirements

### Requirement 1: イベントバブリング（子→親伝播）

**Objective:** 開発者として、子エンティティで発生したイベントを親エンティティにも伝播させたい。それにより階層構造を持つUIでイベントハンドリングを一元化できる。

**Priority:** P0（MVP必須）

#### Acceptance Criteria

1. When ヒットテストにより子エンティティがイベント対象として特定された時, the Event System shall 子エンティティから親エンティティへ順にイベントを伝播する
2. The Event System shall ECS親子関係（`Parent`/`Children` コンポーネント）に基づいてバブリング経路を決定する
3. When バブリング中にルートエンティティに到達した時, the Event System shall 伝播を終了する
4. The Event System shall 各伝播段階でイベント対象エンティティを識別可能にする
5. While バブリング中, the Event System shall 元のヒット対象（`original_target`）情報を保持する

---

### Requirement 2: イベントキャプチャ（親→子伝播）

**Objective:** 開発者として、親エンティティでイベントを先に処理してから子エンティティに伝播させたい。それにより親でイベントを傍受・加工できる。

**Priority:** P2（将来実装）— デスクトップマスコット用途では即時必要なし

#### Acceptance Criteria

1. When キャプチャフェーズが有効な時, the Event System shall ルートエンティティからヒット対象エンティティへ順にイベントを伝播する
2. The Event System shall キャプチャフェーズとバブリングフェーズを区別できるフェーズ情報（`Phase::Capture`）を提供する
3. The Event System shall デフォルトではバブリングのみを有効とし、キャプチャはオプトイン方式とする
4. When キャプチャハンドラが登録されたエンティティに到達した時, the Event System shall キャプチャフェーズでそのハンドラを呼び出す
5. The Event System shall キャプチャフェーズ完了後にターゲットフェーズ、その後バブリングフェーズの順で処理する

---

### Requirement 3: イベントハンドラコンポーネント

**Objective:** 開発者として、エンティティにイベントハンドラを登録したい。それによりウィジェットごとのイベント処理を定義できる。

**Priority:** P0（MVP必須）

#### Acceptance Criteria

1. The Event System shall `MouseEventHandler` コンポーネントを提供する
2. The Event System shall ハンドラとして `fn(&mut World, Entity, &EventContext) -> bool` 型の関数ポインタを受け付ける
3. When ハンドラが `true` を返した時, the Event System shall 以降のエンティティへの伝播を停止する
4. When ハンドラが `false` を返した時, the Event System shall 次のエンティティへ伝播を継続する
5. The Event System shall ハンドラ内で任意のコンポーネントの読み書きを可能にする（`&mut World` 経由）

---

### Requirement 4: イベントコンテキスト

**Objective:** 開発者として、ハンドラ内でイベントの詳細情報にアクセスしたい。それにより適切な処理判断ができる。

**Priority:** P0（MVP必須）

#### Acceptance Criteria

1. The Event System shall `EventContext` 構造体を提供する
2. The Event System shall `EventContext` に元のヒット対象（`original_target: Entity`）を含める
3. The Event System shall `EventContext` にマウス状態（`mouse_state: MouseState`）を含める
4. The Event System shall `EventContext` に現在のフェーズ（`phase: Phase`）を含める
5. The Event System shall `Phase` 列挙型として `Target`, `Bubble`, `Capture` を定義する

---

### Requirement 5: ディスパッチシステム

**Objective:** 開発者として、イベントディスパッチが自動的に実行されることを期待する。それにより手動でのイベント配信が不要になる。

**Priority:** P0（MVP必須）

#### Acceptance Criteria

1. The Event System shall `dispatch_mouse_events` システムを提供する
2. The Event System shall 同一フレーム内でパス収集とハンドラ実行を完了する（フレーム遅延なし）
3. The Event System shall 排他システム（`&mut World`）として実装する
4. The Event System shall Input スケジュール内で、`process_mouse_buffers` の後に実行する
5. When `MouseEventHandler` を持たないエンティティの場合, the Event System shall そのエンティティをスキップして次へ伝播する

---

### Requirement 6: ECSシステム統合

**Objective:** 開発者として、イベントシステムをbevy_ecsのシステムとして統合したい。それにより既存のwintfアーキテクチャと一貫性を保てる。

**Priority:** P0（MVP必須）

#### Acceptance Criteria

1. The Event System shall bevy_ecsシステムとして実装される
2. The Event System shall 既存のウィンドウシステム（`window.rs`, `window_system.rs`）と統合される
3. The Event System shall 既存のレイアウトシステム（`layout/`モジュール）のエンティティ階層を参照する
4. The Event System shall 適切なスケジュール（Input → Update → Render）で実行される
5. When 新しいウィンドウが作成された時, the Event System shall 自動的にイベント処理を開始する

---

### Requirement 7: イベントハンドラのメモリ戦略

**Objective:** 開発者として、イベントハンドラを持たないエンティティのメモリオーバーヘッドを最小化したい。それにより大量のエンティティがあってもパフォーマンスを維持できる。

**Priority:** P0（MVP必須）

#### Acceptance Criteria

1. The Event System shall イベントハンドラコンポーネントに SparseSet ストレージを使用する
2. The Event System shall ハンドラを持たないエンティティにメモリを割り当てない
3. The Event System shall ハンドラの追加・削除が O(1) で完了する
4. The Event System shall ハンドラコンポーネントのサイズを最小化する（fnポインタのみ）

---

### Requirement 8: 汎用イベントディスパッチ

**Objective:** 開発者として、マウス以外のイベントにも同じディスパッチロジックを適用したい。それにより一貫したイベント処理パターンを実現できる。

**Priority:** P1（体験向上）

#### Acceptance Criteria

1. The Event System shall イベント種別に依存しない汎用ディスパッチ関数を提供する
2. The Event System shall 新しいイベント種別の追加が容易な設計とする
3. The Event System shall イベントデータ型をジェネリックパラメータとして受け付ける
4. The Event System shall マウスイベントを汎用ディスパッチの具体例として実装する

---

### Requirement 9: イベント履歴保持

**Objective:** 開発者として、デバッグ目的でイベント履歴を参照したい。それにより問題発生時の原因調査が容易になる。

**Priority:** P2（高度機能）

#### Acceptance Criteria

1. The Event System shall 直近のイベント履歴をリングバッファで保持する
2. The Event System shall 最大1000件のイベント履歴を保持する
3. The Event System shall イベント履歴にタイムスタンプ、イベント種別、対象エンティティ、伝播経路を含める
4. The Event System shall イベント履歴へのアクセスAPIを提供する
5. Where デバッグビルドの場合, the Event System shall 詳細なイベントトレースログを出力できる

---

## Non-Functional Requirements

### NFR-1: パフォーマンス

- イベント伝播: 1イベントあたり1ms以内で完了（10階層まで）
- メモリ: イベント履歴は直近1000件まで、合計1MB以下
- バッファ処理: 1フレームあたり16ms以内で全バッファ処理完了
- **フレーム遅延なし**: 伝播は同一フレーム内で完結すること（必須）

### NFR-2: 信頼性

- イベントの取りこぼしなし
- 正確な伝播順序（ターゲット→バブリング、将来的にはキャプチャ→ターゲット→バブリング）

### NFR-3: 互換性

- 既存の `MouseState` コンポーネントとの後方互換性
- 既存のバッファ機構（`MOUSE_BUFFERS`, `BUTTON_BUFFERS` 等）との統合

---

## Glossary

| 用語 | 説明 |
|------|------|
| バブリング | イベントが子エンティティから親エンティティへ伝播する仕組み |
| キャプチャ | イベントが親エンティティから子エンティティへ伝播する仕組み |
| ターゲットフェーズ | 実際にヒットしたエンティティでイベントが処理されるフェーズ |
| ハンドラ | イベントを処理する関数ポインタ |
| 排他システム | `&mut World` を取るbevy_ecsシステム。他システムと並列実行不可 |
| SparseSet | bevy_ecsのストレージ戦略。少数のエンティティにのみ付くコンポーネント向け |

---

## Appendix

### A. 関連ドキュメント

- 親仕様: `.kiro/specs/wintf-P0-event-system/requirements.md`
- イベントシステム設計: `doc/spec/08-event-system.md`
- 既存マウス実装: `crates/wintf/src/ecs/mouse.rs`

### B. 仕様間の責務分担

| 仕様 | 責務 | 状態 |
|------|------|------|
| `event-mouse-basic` | Win32メッセージ → MouseState コンポーネント付与 | ✅ 実装済み |
| `event-hit-test` | 座標 → ヒット対象エンティティ特定 | ✅ 実装済み |
| `event-dispatch` | MouseState → 伝播・ハンドラ呼び出し | 📝 本仕様 |

### C. 型定義（設計案）

```rust
/// イベントハンドラコンポーネント（SparseSet: ほとんどのエンティティには付かない）
#[derive(Component, Clone, Copy)]
#[component(storage = "SparseSet")]
pub struct MouseEventHandler {
    /// ハンドラ関数
    /// 戻り値: true = ハンドル済み（伝播停止）、false = 未処理（続行）
    pub handler: fn(&mut World, Entity, &EventContext<MouseState>) -> bool,
}

/// 汎用イベントコンテキスト（イベントデータ型をジェネリックに）
#[derive(Clone)]
pub struct EventContext<E> {
    pub original_target: Entity,  // ヒット対象
    pub event_data: E,            // イベントデータ（MouseState等）
    pub phase: Phase,             // 現在のフェーズ
}

/// イベントフェーズ
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Capture,  // 親→子（将来実装）
    Target,   // ヒット対象
    Bubble,   // 子→親
}

/// 汎用ディスパッチ関数シグネチャ
fn dispatch_events<E, H>(
    world: &mut World,
    hit_entity: Entity,
    event_data: E,
) where
    E: Clone,
    H: Component + Copy,  // ハンドラコンポーネント
{ ... }
```

### D. イベント伝播フロー

```
Win32 Message (WM_*)
    ↓
handlers.rs（受信・バッファ蓄積・MouseState付与）[event-mouse-basic]
    ↓
hit_test（対象エンティティ特定）[event-hit-test]
    ↓
dispatch_mouse_events（伝播処理）[event-dispatch - 本仕様]
    │
    ├─ Pass 1: ハンドラ収集（fnポインタはCopy）
    │   path.iter().filter_map(|e| world.get::<MouseEventHandler>(e))
    │
    └─ Pass 2: ハンドラ実行
        for (entity, handler) in handlers {
            if handler(world, entity, &ctx) { break; }
        }
    ↓
Update Schedule（アプリケーションロジック）
    ↓
FrameFinalize Schedule - clear_transient_mouse_state（一時状態リセット）
```
