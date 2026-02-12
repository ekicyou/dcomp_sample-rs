# Gap Analysis: wintf-fix3-sync-arrangement-enable

## 1. 現状調査

### 1.1 対象システムの実装状態

| 資産 | ファイル | 状態 |
|------|----------|------|
| `sync_window_arrangement_from_window_pos` 本体 | `crates/wintf/src/ecs/layout/systems.rs` L451-503 | **実装済み・コメントアウトで無効** |
| スケジュール登録箇所 | `crates/wintf/src/ecs/world.rs` L360-363 | コメントアウト（3行コメント + 1行コード） |
| 順方向同期 `window_pos_sync_system` | `crates/wintf/src/ecs/layout/systems.rs` L370-439 | 有効・稼働中 |
| `apply_window_pos_changes` | `crates/wintf/src/ecs/graphics/systems.rs` L702-818 | 有効・稼働中 |
| `WM_WINDOWPOSCHANGED` ハンドラ | `crates/wintf/src/ecs/window_proc/handlers.rs` L118-302 | 有効・稼働中 |
| エコーバック検知 (`is_echo`) | `crates/wintf/src/ecs/window.rs` L937-939 | 有効・稼働中 |
| `WindowPosChanged` フラグ | `crates/wintf/src/ecs/window.rs` L211 | 有効・稼働中 |
| `pub use systems::*` (re-export) | `crates/wintf/src/ecs/layout/mod.rs` L78 | 関数は既にpub公開済み |

### 1.2 前提条件の充足状況

- **wintf-fix1 完了**: `.kiro/specs/completed/wintf-fix1-sync-window-pos-consolidation/` に存在 → **充足**
- **`update_window_pos_system` 削除済み**: grep結果でヒットなし → **充足**
- `window_pos_sync_system` が唯一の順方向同期システム → **確認済み**

### 1.3 既存の座標系設計

| コンポーネント | 座標系 | 単位 |
|---------------|--------|------|
| `WindowPos.position` | スクリーン座標 | 物理ピクセル (px) |
| `WindowPos.size` | — | 物理ピクセル (px) |
| `Arrangement.offset` | 親からの相対位置 | DIP（論理ピクセル） |
| `Arrangement.scale` | DPIスケール | 倍率 |
| `GlobalArrangement.bounds` | スクリーン座標 | 物理ピクセル (px) |

**座標変換フロー**:
- **逆方向**: `WindowPos.position` (物理px) → `÷ DPI.scale` → `Arrangement.offset` (DIP)
- **順方向**: `Arrangement.offset` (DIP) → `× scale` → `GlobalArrangement.bounds` (物理px) → `truncate` → `WindowPos.position` (物理px)

### 1.4 フィードバックループ防止機構（現行）

現在のコードベースには **3層のフィードバックループ防止機構** が存在する:

| 層 | 機構 | 場所 | 方向 |
|----|------|------|------|
| 1 | `WindowPosChanged` フラグ | `apply_window_pos_changes` L722-727 | ECS→Win32 抑制 |
| 2 | `is_echo()` 判定 | `apply_window_pos_changes` L733-743 | ECS→Win32 抑制 |
| 3 | `Changed<GlobalArrangement>` フィルタ | `window_pos_sync_system` L374 | 変更検知 |

**逆方向同期側の安全機構**（`sync_window_arrangement_from_window_pos` 内）:
- `arrangement.offset != new_offset` による等値チェック（L485）
- `WindowPos.position == None` スキップ（L460）
- `CW_USEDEFAULT` スキップ（L464）
- `scale <= 0.0` スキップ（L474）

## 2. 要件充足性分析

### 2.1 要件と既存資産のマッピング

| 要件 | 既存実装 | ギャップ |
|------|----------|----------|
| R1: 逆方向同期の有効化 | 関数本体は実装済み | コメントアウト解除 + スケジュール順序調整 |
| R2: 座標変換の正確性 | DPI変換ロジック実装済み (L469-481) | **ギャップなし** — 実装済み |
| R3: エッジケースの安全な処理 | None/CW_USEDEFAULT/等値チェック実装済み | **ギャップなし** — 実装済み |
| R4: フィードバックループの非発生 | 3層防止機構が存在 | **確認・検証が必要** (後述) |
| R5: 既存テストとの整合性 | テスト存在 | 有効化後の回帰テスト実行が必要 |
| R6: ログ出力 | `tracing::debug!` 実装済み (L487-496) | **ギャップなし** — 実装済み |

> Note: 旧 R6「スケジュール順序の保証」は R1 に統合済み。旧 R7「ログ出力」は R6 に繰り上げ。

### 2.2 重要な技術的ギャップ

#### ギャップ G1: スケジュール順序の調整

**現在の PostLayout 順序**:
```
sync_simple_arrangements
  → mark_dirty_arrangement_trees
    → propagate_global_arrangements
      → window_pos_sync_system
```

**必要な PostLayout 順序**:
```
sync_window_arrangement_from_window_pos  ← 追加
  → sync_simple_arrangements
    → mark_dirty_arrangement_trees
      → propagate_global_arrangements
        → window_pos_sync_system
```

`sync_window_arrangement_from_window_pos` は Window エンティティの `Arrangement.offset` を変更するため、`sync_simple_arrangements` より **前に** 実行する必要がある。これにより:
1. Window の `Arrangement.offset` が更新される
2. `sync_simple_arrangements` ではルートエンティティ（`Without<ChildOf>`）の `GlobalArrangement` が更新される
3. `mark_dirty_arrangement_trees` が `Changed<Arrangement>` を検知
4. `propagate_global_arrangements` が子孫に伝播
5. `window_pos_sync_system` が最終的な `WindowPos` を設定

**影響度**: 低（コメントアウト解除 + `.before()` 指定のみ）

#### ギャップ G2: フィードバックループの数値的収束性

**シナリオ分析**: ユーザーがウィンドウを位置 (300, 200) にドラッグ（DPI=96, scale=1.0）

| ステップ | 処理 | 値 |
|----------|------|-----|
| WM_WINDOWPOSCHANGED | `WindowPos.position = (300, 200)` | 物理px |
| sync_window_arrangement | `Arrangement.offset = (300/1.0, 200/1.0) = (300.0, 200.0)` | DIP |
| propagate_global | `GlobalArrangement.bounds.left/top = (300.0, 200.0)` | 物理px |
| window_pos_sync | `WindowPos.position = (300, 200)` — **変更なし** | 物理px |

**DPI=192 (scale=2.0) の場合**:

| ステップ | 処理 | 値 |
|----------|------|-----|
| WM_WINDOWPOSCHANGED | `WindowPos.position = (300, 200)` | 物理px |
| sync_window_arrangement | `Arrangement.offset = (300/2.0, 200/2.0) = (150.0, 100.0)` | DIP |
| propagate_global | `GlobalArrangement.bounds.left/top = (150.0 × 2.0, 100.0 × 2.0) = (300.0, 200.0)` | 物理px |
| window_pos_sync | `WindowPos.position = (300, 200)` — **変更なし** | 物理px |

**結論**: 数値的に 1 フレームで収束する。`truncate` (f32 → i32) による丸め誤差も、元が整数値のため発生しない。

**リスク**: Low — 浮動小数点の精度問題は `position.x as f32` → `/ scale` → `× scale` → `as i32` のパスで発生しうるが、`window_pos_sync_system` は `Changed<GlobalArrangement>` フィルタを使用しており、同一値の再代入では bevy_ecs の変更検知が反応しないため、安全。ただし `sync_window_arrangement_from_window_pos` は `&mut Arrangement` を直接変更するため、bevy の変更追跡が発火する。等値チェック（`arrangement.offset != new_offset`）があるため、2フレーム目には `Arrangement` は更新されず、連鎖は停止する。

#### ギャップ G3: `Changed` フィルタ未使用

現在の `sync_window_arrangement_from_window_pos` は `Changed<WindowPos>` フィルタを使用していない:

```rust
Query<(Entity, &WindowPos, &DPI, &mut Arrangement, Option<&Name>), With<Window>>
```

**影響**: 毎フレーム全 Window エンティティを走査する。ただし等値チェックにより実際の変更は発生しないため、`Arrangement` の変更検知は発火しない。パフォーマンスへの影響は Window 数に依存するが、通常のユースケース（1-5ウィンドウ）では問題にならない。

**検討**: `Changed<WindowPos>` フィルタの追加は最適化として有効だが、本仕様のスコープ外（wintf-fix4 で検討可能）。

## 3. 実装アプローチの評価

### Option A: コメントアウト解除のみ（最小変更）

**変更内容**:
1. `world.rs` L360-363 のコメントアウト解除（4行）
2. `.before(sync_simple_arrangements)` の順序制約追加

**変更行数**: 約5行

**トレードオフ**:
- ✅ 最小リスク・最小変更量
- ✅ 既存実装をそのまま活用
- ✅ レガシーコメント削除で可読性向上
- ❌ `Changed<WindowPos>` フィルタ未使用のまま（軽微）

### Option B: コメントアウト解除 + `Changed` フィルタ追加

**変更内容**:
1. Option A の内容すべて
2. `sync_window_arrangement_from_window_pos` のクエリに `Changed<WindowPos>` 追加

**変更行数**: 約8行

**トレードオフ**:
- ✅ Option A のメリットに加え、不要な走査を削減
- ✅ bevy_ecs の変更検知と整合的なパターン
- ✅ `apply_window_pos_changes` で既に `Changed<WindowPos>` の使用実績あり
- ❌ handlers.rs が普通の代入を使用しているため、エコーバック時に同一値でもフラグが立ち無駄反応する
- ❌ 効率化が不完全

### Option B': コメントアウト解除 + `Changed` フィルタ + `set_if_neq` パターン

**変更内容**:
1. Option B の内容すべて
2. `WM_WINDOWPOSCHANGED` ハンドラで `WindowPos` 更新時に `set_if_neq` パターンを使用
3. `WindowPos` に `set_if_neq` メソッドを実装（または `bypass_change_detection()` + 条件分岐）

**変更行数**: 約15-20行

**トレードオフ**:
- ✅ 最も効率的 — エコーバック時の無駄反応を完全に抑制
- ✅ `update_arrangements` (L349) で既に `set_if_neq` パターンの使用実績あり
- ✅ bevy_ecs の推奨パターンに準拠
- ❌ handlers.rs の変更が必要（fix3 のスコープ拡大）
- ❌ `WindowPos` への `set_if_neq` メソッド実装またはボイラープレート増

### Option C: コメントアウト解除 + エコーバック耐性強化

**変更内容**:
1. Option A の内容すべて
2. `sync_window_arrangement_from_window_pos` 内で `last_sent_position` チェックを追加し、ECS 発の `SetWindowPos` エコーバックを明示的にスキップ

**変更行数**: 約15行

**トレードオフ**:
- ✅ フィードバックループ耐性が明示的に強化
- ~~✅ R4-AC4（エコーバック検知との協調）を明示的に満たす~~ → AC4 は AC1 に吸収・削除済み
- ❌ 等値チェックで既に安全のため、冗長な可能性
- ❌ 後続仕様 wintf-fix4 のスコープと重複する可能性

## 4. 複雑性とリスク評価

### 工数見積もり: **M（2日程度）**

**根拠**: Option B' 採用により handlers.rs の変更が追加。`set_if_neq` パターンの実装方法（メソッド追加 or 条件分岐）の検討と、エコーバック時の動作確認テストが必要。

### リスク: **Low**

**根拠**:
- 既存実装が座標変換・エッジケース・ログ出力を網羅
- 3層のフィードバックループ防止機構が存在
- 数値的収束が証明可能
- wintf-fix1 完了により同期方向の競合が解消済み
- `Changed<GlobalArrangement>` フィルタと等値チェックの組み合わせで安全

## 5. 設計フェーズへの推奨事項

### 推奨アプローチ: **Option B'（Changed フィルタ + set_if_neq パターン）**

**理由**:
1. 既存実装が要件 R2-R3-R6 を既に満たしている
2. フィードバックループは数値的に安全であることが証明済み
3. `Changed<WindowPos>` フィルタは `apply_window_pos_changes` で使用実績あり
4. `set_if_neq` パターンは `update_arrangements` (L349) で使用実績あり
5. エコーバック時の無駄な処理を完全に抑制でき、効率的
6. handlers.rs の変更は限定的（`WindowPos` 更新箇所のみ）

### 設計フェーズでの確認事項

1. **スケジュール順序**: `.before()` と `.chain()` の組み合わせによる実装パターンの確定
2. **回帰テスト戦略**: 有効化後にどのテスト・サンプルで検証するかの明確化
3. **f32丸め誤差の境界条件**: 奇数DPIスケール（例: 125%, scale=1.25）で丸め誤差による振動が発生しないことの確認

### Research Needed（設計フェーズで調査）

- **R-1**: DPI 125%（scale=1.25）での座標変換往復精度テスト — `position / 1.25 * 1.25` で元の整数値に戻るか
- ~~**R-2**: `set_if_neq` vs 直接代入のトレードオフ — bevy_ecs の推奨パターン確認~~ → Option B' 採用により `set_if_neq` パターン使用に決定
