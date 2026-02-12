# ギャップ分析: wintf-fix1-sync-window-pos-consolidation

## 1. 現状調査

### 1.1 重複システムの詳細比較

| 項目 | `sync_window_pos` (graphics/systems.rs) | `update_window_pos_system` (layout/systems.rs) |
|------|----------------------------------------|------------------------------------------------|
| **場所** | `crates/wintf/src/ecs/graphics/systems.rs` L699-784 | `crates/wintf/src/ecs/layout/systems.rs` L369-393 |
| **クエリフィルタ** | `With<Window>, Or<(Changed<GlobalArrangement>, Changed<Arrangement>)>` | `With<Window>, Changed<GlobalArrangement>` |
| **入力コンポーネント** | `Entity`, `GlobalArrangement`, `Arrangement`(未使用), `WindowPos`, `Option<Name>` | `GlobalArrangement`, `WindowPos` |
| **リソース依存** | `Res<FrameCount>` | なし |
| **無効 bounds ガード** | あり（width ≤ 0 or height ≤ 0 でスキップ） | **なし** |
| **差分検出** | あり（position/size の変更判定後に更新） | **なし**（常に上書き） |
| **サイズ計算** | `width.ceil() as i32`, `height.ceil() as i32` | `bounds.width() as i32`, `bounds.height() as i32`（切り捨て） |
| **tracing ログ** | 充実（debug/trace レベル、フレーム番号含む） | **なし** |
| **エクスポート** | `pub use graphics::*` → `wintf::ecs::sync_window_pos` | `pub use systems::*` → `wintf::ecs::update_window_pos_system` |

### 1.2 重要な差異: サイズ変換の丸めロジック

**`sync_window_pos`**: `width.ceil() as i32`（切り上げ）  
**`update_window_pos_system`**: `bounds.width() as i32`（切り捨て = `as i32` のデフォルト動作）

この差異はサブピクセル値（例: 799.5）で結果が異なる:
- `ceil`: 799.5 → 800
- `as i32`: 799.5 → 799

**影響**: 2つのシステムが連鎖実行（`sync_window_pos` → `update_window_pos_system`）されるため、現在は `update_window_pos_system` が `sync_window_pos` の結果を上書きしている。実効動作は truncation だが、これは意図しない挙動である。

**統合方針（決定済み）**: `sync_window_pos` の `ceil` を採用する。理由: 論理サイズいっぱいまで描画される可能性があり、物理ピクセルは整数であるため、論理サイズを完全に内包する物理サイズを保証するには切り上げが必要。旧実装の truncation 上書きは `update_window_pos_system` の重複による副作用であり、本統合でこの不整合を解消する。

### 1.3 スケジューリング構造

```
PostLayout schedule:
  sync_simple_arrangements
  → mark_dirty_arrangement_trees
  → propagate_global_arrangements
  → sync_window_pos              ← 廃止対象
  → update_window_pos_system     ← 廃止対象
  (.chain() で全体が順序保証)

UISetup schedule:
  create_windows
  apply_window_pos_changes       ← WindowPos → SetWindowPos（後段、変更なし）
```

統合後:
```
PostLayout schedule:
  sync_simple_arrangements
  → mark_dirty_arrangement_trees
  → propagate_global_arrangements
  → window_pos_sync_system       ← 新システム（1つに統合）
  (.chain() で全体が順序保証)
```

### 1.4 パブリック API エクスポート経路

```
graphics/systems.rs  → pub fn sync_window_pos
graphics/mod.rs      → pub use systems::*
ecs/mod.rs           → pub use graphics::*
                     → wintf::ecs::sync_window_pos（テストから参照）

layout/systems.rs    → pub fn update_window_pos_system
layout/mod.rs        → pub use systems::*
ecs/mod.rs           → pub use layout::*
                     → wintf::ecs::update_window_pos_system（直接テスト参照なし）
```

### 1.5 テスト影響範囲

**直接参照しているテスト**: `crates/wintf/tests/layout_graphics_sync_test.rs`

| テスト関数 | 参照箇所 | 影響 |
|-----------|---------|------|
| `test_sync_window_pos` | `wintf::ecs::sync_window_pos` をスケジュールに登録 | **要更新** |
| `test_skip_invalid_bounds` | `wintf::ecs::sync_window_pos` を `IntoSystem::into_system` で直接実行 | **要更新** |
| `test_echo_back_flow` | `wintf::ecs::sync_window_pos` をスケジュールに登録 | **要更新** |
| `test_echo_detection` | `WindowPos` のメソッドテスト、システム名参照なし | 影響なし |
| `test_reverse_flow_simulation` | システム参照なし | 影響なし |
| `test_visual_partial_eq_optimization` | システム参照なし | 影響なし |

3つのテストでシステム名の更新が必要。`FrameCount` リソースは引き続き必要。

## 2. 要件実現性分析

### 要件-アセットマッピング

| 要件 | 既存アセット | ギャップ |
|------|------------|---------|
| Req 1: 統合 | `sync_window_pos` の完全な実装が基盤 | 新関数 `window_pos_sync_system` の作成（layout/systems.rs への移動） |
| Req 2: 変換正確性 | `sync_window_pos` に完備（ガード、ceil、差分検出） | `update_window_pos_system` のロジックは破棄（ceil 非使用、ガードなし） |
| Req 3: 旧システム除去 | 2関数の定義 + world.rs 登録 | 3ファイルの編集 |
| Req 4: スケジューリング | `.chain()` 制約で既に順序保証 | 登録行の置換のみ |
| Req 5: ログ | `sync_window_pos` に完備 | 関数名プレフィックスの変更 `[sync_window_pos]` → `[window_pos_sync_system]` |
| Req 6: テスト互換性 | 3テストで旧名参照あり | テスト内のシステム名更新 |

**ギャップ総評**: Missing な機能はない。既存の `sync_window_pos` がすべての要件機能をカバーしており、純粋なリファクタリング（移動+リネーム+旧コード削除）で実現可能。

## 3. 実装アプローチ選択肢

### Option A: 移動+リネーム（推奨）

`sync_window_pos` のロジックをそのまま `layout/systems.rs` に `window_pos_sync_system` として移動し、`graphics/systems.rs` の元関数と `layout/systems.rs` の `update_window_pos_system` を削除する。

**変更ファイル**:
1. `crates/wintf/src/ecs/layout/systems.rs` — `update_window_pos_system` を `window_pos_sync_system` に置換（`sync_window_pos` のロジックで上書き）
2. `crates/wintf/src/ecs/graphics/systems.rs` — `sync_window_pos` 関数を削除
3. `crates/wintf/src/ecs/world.rs` — システム登録を1行に統合
4. `crates/wintf/tests/layout_graphics_sync_test.rs` — 3テストのシステム名を更新

**依存移動**:
- `format_entity_name` は `graphics/systems.rs` に残る（他の graphics システムも使用）→ layout から `crate::ecs::graphics::format_entity_name` で参照
- `FrameCount` は `world.rs` で定義済み → `Res<crate::ecs::world::FrameCount>` で参照

**トレードオフ**:
- ✅ 最小限の変更（4ファイル）
- ✅ 既存の充実したロジック（ガード、差分検出、ログ）を活用
- ✅ `layout` モジュールに配置する設計意図と整合
- ❌ `layout/systems.rs` が `graphics::format_entity_name` に依存（逆方向依存）

### Option B: layout に独自ヘルパー追加

Option A と同様だが、`format_entity_name` を layout モジュールにもコピーまたは共通化する。

**追加変更**:
- `common/` モジュールまたは `layout/` モジュールに `format_entity_name` 相当のヘルパーを追加

**トレードオフ**:
- ✅ モジュール間依存を排除
- ❌ コード重複または追加リファクタリングが必要
- ❌ 本仕様のスコープ外の変更を誘発

### Option C: graphics に統合先を残す

`sync_window_pos` をリネームのみして `graphics/systems.rs` に残し、`update_window_pos_system` を削除する。

**トレードオフ**:
- ✅ 変更量が最小（リネーム + 1関数削除 + world.rs 更新）
- ❌ Requirement 1.3 に反する（layout モジュールに配置する要件）
- ❌ 設計意図（WindowPos は layout の責務）と不整合

## 4. 実装複雑度・リスク

**Effort: S（1日）**
- 既存パターンの移動+リネーム。新規ロジック不要。変更ファイル数 4。

**Risk: Low**
- 既知のコードベース内のリファクタリング
- テストが既に存在し、期待値の変更不要（`ceil` 採用のため）
- スケジューリング制約は `.chain()` で既に保証

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ: Option A（移動+リネーム）

**理由**:
1. 要件すべてを満たす最小変更
2. `format_entity_name` の逆方向依存は許容範囲（一時的、将来の共通化で解消可能）
3. `update_window_pos_system` の差異（ceil なし、ガードなし）は `sync_window_pos` のロジックで正しくカバー

### 設計フェーズでの確認事項

1. **`Arrangement` コンポーネント参照の要否（決定済み）**: クエリから除去する。`_arrangement` として未使用であり、`GlobalArrangement` が既に `Arrangement` の伝播結果を含む
2. **`Changed` フィルタの統合（決定済み）**: `Changed<GlobalArrangement>` のみに簡素化。`Changed<Arrangement>` は `Arrangement` 除去に伴い不要
3. **`format_entity_name` の依存方向（決定済み）**: `layout/systems.rs` は既に L14 で `use crate::ecs::graphics::format_entity_name` をインポートしている。既存パターンの踏襲であり追加コスト 0
4. **テスト名の変更**: テスト関数名は実装時の判断。設計上の必須要件ではない

### リサーチ不要項目

外部依存や未知技術は関与しない。すべて既存コードベース内の変更で完結する。
