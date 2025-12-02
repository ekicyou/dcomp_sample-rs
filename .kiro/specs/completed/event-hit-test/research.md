# Research & Design Decisions

---

| 項目 | 内容 |
|------|------|
| **Feature** | event-hit-test |
| **Discovery Scope** | Extension（既存ECSレイアウトシステムへの機能追加） |
| **Date** | 2025-12-02 |

---

## Summary

**Key Findings**:
1. `GlobalArrangement.bounds` は物理ピクセル座標で直接比較可能
2. `D2DRectExt::contains()` が再利用可能
3. `WindowPos` コンポーネントにウィンドウ位置情報が存在
4. 既存の ECS 階層パターン（`Children`, `ChildOf`）を活用

---

## Research Log

### 座標変換API（R1）

- **Context**: スクリーン座標からウィンドウクライアント座標への変換方法の調査
- **Sources Consulted**: 
  - `crates/wintf/src/ecs/window.rs` - WindowPos 定義
  - `crates/wintf/tests/client_area_positioning_test.rs` - テストパターン
  - windows crate ドキュメント
- **Findings**:
  - `WindowPos.position` に `POINT { x, y }` でウィンドウ位置を保持
  - Win32 `ScreenToClient` API は windows crate 経由で利用可能だが不要
  - `GlobalArrangement.bounds` が既に仮想デスクトップ座標（物理ピクセル）なので直接オフセット計算可能
- **Implications**:
  - `hit_test(world, root, screen_point)` でスクリーン座標を直接受け入れ
  - `hit_test_in_window` は `WindowPos.position` を使ってオフセット計算

### 逆イテレーション最適化（R2）

- **Context**: `Children` の逆順走査のパフォーマンス評価と実装方式
- **Sources Consulted**: 
  - bevy_ecs 0.17.2 ソースコード
  - Rust `DoubleEndedIterator` ドキュメント
  - docs.rs/bevy_ecs/0.17.2/bevy_ecs/hierarchy/struct.Children.html
- **Findings**:
  - `Children` は `SmallVec<[Entity; 8]>` ベース
  - `.iter().rev()` は `DoubleEndedIterator` を利用した O(1) 反転
  - 追加のメモリアロケーションなし
  - **bevy_ecs に逆順走査 API なし**: `iter_descendants_depth_first` は正順のみ
  - 後順走査（post-order）には独自実装が必要
- **Implications**:
  - **スタック + フラグ方式**: `(Entity, bool)` タプルで子展開済みかを管理
  - 汎用イテレータ `DepthFirstReversePostOrder` を `ecs::common` に配置
  - ヒットテスト以外（フォーカス管理等）でも再利用可能

### キャッシュ無効化（R3）

- **Context**: ヒットテスト結果キャッシュの無効化タイミング
- **Sources Consulted**: 
  - `crates/wintf/src/ecs/layout/arrangement.rs` - ArrangementTreeChanged 定義
- **Findings**:
  - `ArrangementTreeChanged` マーカーコンポーネントが存在
  - レイアウト変更時に伝播される仕組みが確立済
  - 数百エンティティ規模では毎回走査でも 1ms 以内
- **Implications**:
  - Phase 1 はキャッシュなしで実装（選択肢 A）
  - 将来的にクリッピング導入時に最適化検討

---

## Architecture Pattern Evaluation

| Option | Description | Strengths | Risks / Limitations | Notes |
|--------|-------------|-----------|---------------------|-------|
| **Option A: 既存モジュール拡張** | `arrangement.rs` に追加 | ファイル数最小 | 責務混在、肥大化 | 非推奨 |
| **Option B: 新規モジュール** | `hit_test.rs` を新規作成 | 単一責任、テスト容易 | ファイル数 +1 | ✅ 推奨 |
| **Option C: ハイブリッド** | 段階的に複数ファイル | 柔軟 | 今回の規模では過剰 | 不要 |

---

## Design Decisions

### Decision: モジュール配置

- **Context**: `HitTest` コンポーネントと API 関数の配置場所
- **Alternatives Considered**:
  1. `ecs/layout/arrangement.rs` に追加 — 既存ファイル拡張
  2. `ecs/layout/hit_test.rs` 新規作成 — 独立モジュール
- **Selected Approach**: Option B - `ecs/layout/hit_test.rs` 新規作成
- **Rationale**: 
  - 単一責任の原則遵守
  - `GlobalArrangement.bounds` 依存のため `ecs::layout` 名前空間が適切
  - テスト容易性（Visual なしでテスト可能）
- **Trade-offs**: ファイル数 +1 だが責務分離のメリットが上回る
- **Follow-up**: `ecs/layout/mod.rs` に `pub mod hit_test;` 追加

### Decision: API シグネチャ

- **Context**: ヒットテスト関数の引数設計
- **Alternatives Considered**:
  1. `hit_test(world, point)` — ルート固定
  2. `hit_test(world, root, point)` — ルート指定可能
- **Selected Approach**: `hit_test(world, root, screen_point)`
- **Rationale**: 
  - LayoutRoot, Window, 任意のサブツリーを検索スコープとして指定可能
  - テスト時に特定のサブツリーのみ検証可能
- **Trade-offs**: 引数が 1 つ増えるが柔軟性向上
- **Follow-up**: `hit_test_in_window` で Window エンティティを自動取得

### Decision: 走査アルゴリズム

- **Context**: 深さ優先・逆順走査の実装方式
- **Alternatives Considered**:
  1. 再帰関数 — シンプルだが early return しにくい
  2. イテレータ + スタック — 汎用的、early return 容易
- **Selected Approach**: イテレータ + スタック + フラグ方式
- **Rationale**: 
  - `(Entity, bool)` タプルで子展開済みかを管理
  - 後順走査（子を全て返してから親を返す）を実現
  - for ループで使用でき、最初のヒットで即座に return 可能
  - `ecs::common::DepthFirstReversePostOrder` として汎用化
- **Trade-offs**: 再帰より若干複雑だが、再利用性と early return のメリットが上回る
- **Follow-up**: `ecs::common` モジュールを新規作成、フォーカス管理等で再利用検討

### Decision: HitTest コンポーネントのデフォルト動作

- **Context**: `HitTest` コンポーネントを持たないエンティティの扱い
- **Alternatives Considered**:
  1. スキップ（明示的）— `HitTest` 必須、毎回追加が必要
  2. Bounds として扱う（暗黙的）— 省略時は矩形判定
- **Selected Approach**: Option B - 暗黙的に `HitTestMode::Bounds` として扱う
- **Rationale**: 
  - 描画されているウィジェットは基本的にヒット対象とすべき
  - 記述量が減り、自然なデフォルト動作となる
  - 除外したい場合のみ `HitTest::none()` を明示的に追加
- **Trade-offs**: 
  - 暗黙的な動作のため、意図せずヒット対象になる可能性
  - 透明部分（α < 50%）もヒット対象となるセキュリティリスク（後述）
- **Security Note**: 
  - 矩形判定では透明部分もヒット対象となり、クリックジャッキング的リスクあり
  - 「疑わしきはヒットしない」が安全だが、本仕様では利便性を優先
  - セキュリティ要件が高いケースは `event-hit-test-alpha-mask` で対応
- **Follow-up**: 孫仕様 `event-hit-test-alpha-mask` でα判定を実装予定

### Decision: API シグネチャ（World 渡し）

- **Context**: ヒットテスト関数の引数設計
- **Alternatives Considered**:
  1. Query を引数に渡す — 呼び出し側で用意が必要
  2. `&World` を渡す — 内部で必要なコンポーネントを取得
  3. SystemParam 構造体 — 使いやすいがボイラープレート増
- **Selected Approach**: Option B - `&World` を渡す
- **Rationale**: 
  - 呼び出し側はシンプル（`&World` を渡すだけ）
  - 将来 `AlphaMask` 等を追加しても呼び出し側の変更不要
  - 内部で `world.get::<T>(entity)` を使用（O(1)、性能問題なし）
  - 排他システムで World を持っているため自然な設計
- **Trade-offs**: Query ほど効率的ではないが、ヒットテスト頻度では許容範囲
- **Follow-up**: 
  - `hit_test_entity(world, entity, point) -> bool` — 単一エンティティ判定
  - `hit_test(world, root, point) -> Option<Entity>` — ツリー走査

### Decision: 2層 API 構成

- **Context**: 単一エンティティ判定とツリー走査の分離
- **Selected Approach**: Layer 1（hit_test_entity）と Layer 2（hit_test）に分離
- **Rationale**: 
  - `hit_test_entity` は単一エンティティのみ判定（子孫は走査しない）
  - `hit_test` は `DepthFirstReversePostOrder` で走査し、各エンティティで `hit_test_entity` を呼び出し
  - 責務分離により将来の拡張（AlphaMask 等）が容易
- **Trade-offs**: 関数が増えるが、責務が明確になる
- **Follow-up**: テストでは `hit_test_entity` を直接呼び出し可能

### Decision: キャッシュ戦略

- **Context**: パフォーマンス最適化の必要性判断
- **Alternatives Considered**:
  1. キャッシュなし（毎回フル走査）
  2. 座標キャッシュのみ（前回座標と結果保持）
  3. 階層キャッシュ（ArrangementTreeChanged 連携）
- **Selected Approach**: Phase 1 はキャッシュなし
- **Rationale**: 
  - 数百エンティティで O(n) 走査は 1ms 以内
  - YAGNI: 問題発生時に最適化
  - API は同一でアルゴリズム差し替え可能
- **Trade-offs**: 将来のパフォーマンス問題リスクあるが、シンプルさ優先
- **Follow-up**: NFR-1 のパフォーマンス要件（1ms 以内）を実装後に検証

---

## Risks & Mitigations

| リスク | 緩和策 |
|--------|--------|
| パフォーマンス劣化（大量エンティティ） | trait ベース設計で将来的にアルゴリズム差し替え可能 |
| 座標変換の精度問題 | NFR-2 で 0.5 ピクセル以内を検証 |
| 階層走査のエッジケース | クリッピングなしを明示的にテスト |

---

## References

- bevy_ecs 0.17.2 - Children, ChildOf コンポーネント
- windows crate 0.62.1 - Win32 API バインディング
- `crates/wintf/src/ecs/layout/arrangement.rs` - GlobalArrangement 定義
- `crates/wintf/src/ecs/layout/rect.rs` - D2DRectExt::contains() 実装
