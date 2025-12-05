# Research & Design Decisions: event-dispatch

---

## Summary
- **Feature**: `event-dispatch`
- **Discovery Scope**: Extension（既存システムへの拡張）
- **Key Findings**:
  - 既存の `ChildOf.parent()` パターンで親チェーン取得が O(depth) で実現可能
  - bevy_ecs 0.17.2 の排他システム（`&mut World`）パターンが確立済み
  - SparseSet ストレージが `MouseState` で実績あり、同一パターンを適用可能

---

## Research Log

### Topic 1: bevy_ecs 排他システムパターン

- **Context**: ディスパッチシステムでは `&mut World` が必要（ハンドラ内でコンポーネント読み書き）
- **Sources Consulted**: 
  - 既存コード: `ecs/world.rs`, `ecs/graphics/systems.rs`
  - bevy_ecs 0.17.2 ドキュメント
- **Findings**:
  - 排他システムは `fn(world: &mut World)` シグネチャで定義
  - 他システムと並列実行されないため、借用競合なし
  - 既存の `commit_composition` が同様のパターンを使用
- **Implications**: 新規の `dispatch_mouse_events` も同一パターンで実装可能

### Topic 2: 親チェーン取得アルゴリズム

- **Context**: バブリングのため、ヒットエンティティから親チェーンを収集する必要あり
- **Sources Consulted**:
  - `ecs/graphics/systems.rs:1017-1020` の depth 計算パターン
  - `ecs/common/tree_iter.rs` の走査アルゴリズム
- **Findings**:
  ```rust
  let mut path = vec![hit_entity];
  let mut current = hit_entity;
  while let Ok(child_of) = child_of_query.get(current) {
      current = child_of.parent();
      path.push(current);
  }
  ```
  - 計算量: O(depth)、典型的なUI階層で depth < 10
  - `ChildOf.parent()` メソッドで親エンティティを取得
- **Implications**: バブリングパス構築は単純なループで実現

### Topic 3: 2パス方式の必要性

- **Context**: ハンドラ実行中に `World` を借用するため、同時にクエリ実行不可
- **Sources Consulted**:
  - Rust 借用規則
  - bevy_ecs のクエリパターン
- **Findings**:
  - Pass 1: ハンドラ（fnポインタ）を収集して `Vec<(Entity, fn(...) -> bool)>` に格納
  - Pass 2: 収集したハンドラを順次実行、`true` 返却で停止
  - fnポインタは `Copy` のためムーブなしで収集可能
- **Implications**: フレーム遅延なしで同一フレーム内に伝播完了

### Topic 4: SparseSet ストレージ選択

- **Context**: ハンドラを持つエンティティは全体の少数
- **Sources Consulted**:
  - `ecs/mouse.rs:98-99` の `MouseState` 実装
  - bevy_ecs ストレージドキュメント
- **Findings**:
  - Table ストレージ: アーキタイプ変更が頻繁な場合に非効率
  - SparseSet: 挿入/削除 O(1)、少数エンティティに最適
  - `MouseState` が SparseSet で実績あり
- **Implications**: `MouseEventHandler` も SparseSet を採用

---

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| A: mouse.rs 拡張 | 既存モジュールにディスパッチ追加 | 統合が自然 | 800行超の肥大化、単一責任違反 | 非推奨 |
| B: dispatch.rs 新規 | 独立モジュールとして作成 | 責務分離明確、拡張容易 | 新規ファイル追加 | **推奨** |
| C: layout/event.rs | レイアウトモジュール配下に配置 | hit_test と近い | 責務不一致 | 非推奨 |

**選択**: Option B（新規 `dispatch.rs` モジュール作成）

---

## Design Decisions

### Decision 1: ハンドラ型としての関数ポインタ

- **Context**: ハンドラが状態を持つかどうかの設計選択
- **Alternatives Considered**:
  1. `Box<dyn Fn(...) -> bool>` — クロージャ、状態キャプチャ可能
  2. `fn(&mut World, Entity, &EventContext) -> bool` — 関数ポインタ、ステートレス
- **Selected Approach**: 関数ポインタ
- **Rationale**: 
  - ECS原則: 状態はコンポーネントに分離
  - `Copy` トレイト対応でハンドラ収集が容易
  - コンポーネントサイズ最小化（8バイト）
- **Trade-offs**: 
  - ✅ メモリ効率、Copy可能
  - ❌ ハンドラ内で外部状態アクセスには `World.get()` が必要
- **Follow-up**: ハンドラ登録APIの使いやすさを実装時に検証

### Decision 2: 2パス伝播方式

- **Context**: ハンドラ実行中の World 借用競合を回避
- **Alternatives Considered**:
  1. 遅延実行（Commands）— フレーム遅延発生
  2. 2パス（収集→実行）— 同一フレーム完結
- **Selected Approach**: 2パス方式
- **Rationale**: フレーム遅延は UI レスポンス低下の原因となる
- **Trade-offs**:
  - ✅ 同一フレーム内で伝播完結
  - ❌ 一時的な Vec 割り当て（通常10エンティティ未満）
- **Follow-up**: パフォーマンス計測でボトルネックにならないことを確認

### Decision 3: EventContext のジェネリック化

- **Context**: 将来のキーボード/タイマーイベント対応
- **Alternatives Considered**:
  1. `EventContext` 固定 — マウス専用
  2. `EventContext<E>` — イベントデータ型をパラメータ化
- **Selected Approach**: `EventContext<E>` ジェネリック
- **Rationale**: 汎用ディスパッチ（Req 8）への拡張を見据えた設計
- **Trade-offs**:
  - ✅ 将来拡張が容易
  - ❌ 型定義がやや複雑
- **Follow-up**: `KeyboardEventHandler` 追加時にパターン検証

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| ハンドラ内でのパニック | 伝播中断、アプリケーション不安定 | `catch_unwind` の検討（設計外） |
| 深い階層でのパフォーマンス | 1ms 超過 | 階層深度警告ログ、ベンチマーク追加 |
| 循環参照（不正な親子関係） | 無限ループ | `ChildOf` 設定時の検証は別仕様 |

---

## References

- [bevy_ecs 0.17.2 hierarchy](https://docs.rs/bevy_ecs/0.17.2/bevy_ecs/hierarchy/index.html) — ChildOf/Children API
- [bevy_ecs storage](https://docs.rs/bevy_ecs/0.17.2/bevy_ecs/component/index.html) — SparseSet vs Table
- 既存実装: `crates/wintf/src/ecs/mouse.rs`, `graphics/systems.rs`, `common/tree_iter.rs`
