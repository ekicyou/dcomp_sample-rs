# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | event-dispatch 要件定義書 |
| **Version** | 0.1 (Draft) |
| **Date** | 2025-12-03 |
| **Parent Spec** | wintf-P0-event-system |
| **Author** | AI-DLC System |

---

## Introduction

本仕様書は wintf フレームワークにおけるイベント配信機構とECS統合の要件を定義する。親仕様「wintf-P0-event-system」の Requirement 7（イベント配信機構）および Requirement 8（ECS統合）を実装対象とする。

### 背景

wintf フレームワークでは、既存の `mouse.rs` モジュールにおいて Win32 メッセージをECSコンポーネント（`MouseState`）に変換する基盤が実装されている。しかし、イベントの伝播（バブリング/キャプチャ）機構、および配信停止（stopPropagation相当）の仕組みが未実装である。デスクトップマスコットアプリケーションでは、親子関係を持つウィジェット間でのイベント伝播が必須であり、本仕様でこれを実現する。

### スコープ

**含まれるもの**:
- イベント伝播機構（バブリング: 子→親、キャプチャ: 親→子）
- イベント配信停止（stopPropagation相当）
- Win32メッセージからECSイベントへの変換拡張
- 既存ECSシステム（`window.rs`, `layout/`）との統合
- イベント履歴保持（デバッグ用）
- エンティティ削除時のイベントリスナー解除

**含まれないもの**:
- ヒットテストロジック（`event-hit-test` 仕様で実装）
- 個別マウスイベント生成（`event-mouse-basic` 仕様で実装）
- キーボード/タッチイベント（将来の拡張）

### 親仕様からの要件マッピング

本仕様は以下の親要件に対応する：
- **Requirement 7**: イベント配信機構
- **Requirement 8**: ECS統合

---

## Requirements

### Requirement 1: イベントバブリング（子→親伝播）

**Objective:** 開発者として、子エンティティで発生したイベントを親エンティティにも伝播させたい。それにより階層構造を持つUIでイベントハンドリングを一元化できる。

#### Acceptance Criteria

1. When ヒットテストにより子エンティティがイベント対象として特定された時, the Event System shall 子エンティティから親エンティティへ順にイベントを伝播する
2. The Event System shall ECS親子関係（`Parent`/`Children` コンポーネント）に基づいてバブリング経路を決定する
3. When バブリング中にルートエンティティに到達した時, the Event System shall 伝播を終了する
4. The Event System shall 各伝播段階でイベント対象エンティティを識別可能にする
5. While バブリング中, the Event System shall 元のヒット対象（`originalTarget`）情報を保持する

---

### Requirement 2: イベントキャプチャ（親→子伝播）

**Objective:** 開発者として、親エンティティでイベントを先に処理してから子エンティティに伝播させたい。それにより親でイベントを傍受・加工できる。

#### Acceptance Criteria

1. When キャプチャフェーズが有効な時, the Event System shall ルートエンティティからヒット対象エンティティへ順にイベントを伝播する
2. The Event System shall キャプチャフェーズとバブリングフェーズを区別できるフェーズ情報を提供する
3. The Event System shall デフォルトではバブリングのみを有効とし、キャプチャはオプトイン方式とする
4. When キャプチャリスナーが登録されたエンティティに到達した時, the Event System shall キャプチャフェーズでそのリスナーを呼び出す
5. The Event System shall キャプチャフェーズ完了後にターゲットフェーズ、その後バブリングフェーズの順で処理する

---

### Requirement 3: イベント伝播停止（stopPropagation）

**Objective:** 開発者として、イベント伝播を任意の時点で停止させたい。それにより不要な親エンティティへの伝播を防げる。

#### Acceptance Criteria

1. The Event System shall イベントに伝播停止フラグを提供する
2. When 伝播停止フラグがセットされた時, the Event System shall 以降のエンティティへの伝播を停止する
3. The Event System shall 伝播停止とデフォルト動作防止を区別できる（stopPropagation vs preventDefault相当）
4. While 伝播停止状態, the Event System shall 現在のエンティティのハンドラ処理は完了させる
5. The Event System shall 即時伝播停止（stopImmediatePropagation相当）もサポートし、同一エンティティの残りハンドラもスキップできる

---

### Requirement 4: Win32メッセージ変換

**Objective:** 開発者として、Win32マウスメッセージを統一されたECSイベント形式に変換したい。それによりプラットフォーム固有の処理を隠蔽できる。

#### Acceptance Criteria

1. When WM_MOUSEMOVE を受信した時, the Event System shall ECSマウス移動イベントに変換する
2. When WM_LBUTTONDOWN/WM_LBUTTONUP を受信した時, the Event System shall ECSマウスボタンイベントに変換する
3. When WM_RBUTTONDOWN/WM_RBUTTONUP を受信した時, the Event System shall ECSマウスボタンイベントに変換する
4. When WM_MBUTTONDOWN/WM_MBUTTONUP を受信した時, the Event System shall ECSマウスボタンイベントに変換する
5. When WM_MOUSEWHEEL/WM_MOUSEHWHEEL を受信した時, the Event System shall ECSホイールイベントに変換する
6. When WM_LBUTTONDBLCLK/WM_RBUTTONDBLCLK/WM_MBUTTONDBLCLK を受信した時, the Event System shall ECSダブルクリックイベントに変換する
7. The Event System shall 既存の `MouseState` コンポーネントおよびバッファ機構（`MouseBuffer`, `ButtonBuffer` 等）と統合する

---

### Requirement 5: ECSシステム統合

**Objective:** 開発者として、イベントシステムをbevy_ecsのシステムとして統合したい。それにより既存のwintfアーキテクチャと一貫性を保てる。

#### Acceptance Criteria

1. The Event System shall bevy_ecsシステムとして実装される
2. The Event System shall 既存のウィンドウシステム（`window.rs`, `window_system.rs`）と統合される
3. The Event System shall 既存のレイアウトシステム（`layout/`モジュール）と統合される
4. The Event System shall 適切なスケジュール（Input → Update → Render）で実行される
5. When 新しいウィンドウが作成された時, the Event System shall 自動的にイベント処理を開始する

---

### Requirement 6: エンティティ削除時の解除

**Objective:** 開発者として、エンティティ削除時に関連するイベント状態が自動的にクリーンアップされることを保証したい。それによりメモリリークやダングリング参照を防げる。

#### Acceptance Criteria

1. When エンティティが削除された時, the Event System shall 関連するイベントバッファ（`MouseBuffer`, `ButtonBuffer`, `WheelBuffer`）をクリアする
2. When エンティティが削除された時, the Event System shall 進行中の伝播経路からそのエンティティを除外する
3. The Event System shall エンティティ削除を検出するためにbevy_ecsの `RemovedComponents` を使用する
4. When 親エンティティが削除された時, the Event System shall 子エンティティへの影響を適切に処理する
5. The Event System shall 削除されたエンティティへのイベント配信を試みない

---

### Requirement 7: イベント履歴保持

**Objective:** 開発者として、デバッグ目的でイベント履歴を参照したい。それにより問題発生時の原因調査が容易になる。

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

### NFR-2: 信頼性

- イベントの取りこぼしなし
- 正確な伝播順序（キャプチャ→ターゲット→バブリング）
- エンティティ削除時のリソースリーク防止

### NFR-3: 互換性

- 既存の `MouseState` コンポーネントとの後方互換性
- 既存のバッファ機構（`MOUSE_BUFFERS`, `BUTTON_BUFFERS` 等）との統合

---

## Glossary

| 用語 | 説明 |
|------|------|
| バブリング | イベントが子エンティティから親エンティティへ伝播する仕組み |
| キャプチャ | イベントが親エンティティから子エンティティへ伝播する仕組み |
| stopPropagation | イベント伝播を停止する機能 |
| preventDefault | イベントのデフォルト動作を防止する機能 |
| ターゲットフェーズ | 実際にヒットしたエンティティでイベントが処理されるフェーズ |

---

## Appendix

### A. 関連ドキュメント

- 親仕様: `.kiro/specs/wintf-P0-event-system/requirements.md`
- イベントシステム設計: `doc/spec/08-event-system.md`
- 既存マウス実装: `crates/wintf/src/ecs/mouse.rs`

### B. 既存実装との統合ポイント

| モジュール | 統合内容 |
|-----------|---------|
| `mouse.rs` | `MouseState`コンポーネント、バッファ機構の拡張 |
| `window.rs` | ウィンドウエンティティとの関連付け |
| `window_proc/handlers.rs` | Win32メッセージ受信と変換処理 |
| `layout/` | エンティティ階層（`Parent`/`Children`）の参照 |
| `world.rs` | スケジュール登録（Input, Update, FrameFinalize）|

### C. イベント伝播フロー

```
Win32 Message (WM_*)
    ↓
window_proc/handlers.rs（受信・バッファ蓄積）
    ↓
Input Schedule - process_mouse_buffers（バッファ→コンポーネント反映）
    ↓
Input Schedule - hit_test（対象エンティティ特定）
    ↓
Input Schedule - dispatch_events（伝播処理）
    ├─ Capture Phase（親→子）
    ├─ Target Phase（ヒット対象）
    └─ Bubble Phase（子→親）
    ↓
Update Schedule（アプリケーションロジック）
    ↓
FrameFinalize Schedule - clear_transient_mouse_state（一時状態リセット）
```
