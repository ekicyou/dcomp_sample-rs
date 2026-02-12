# Research: wintf-fix3-sync-arrangement-enable

## Summary

本仕様の設計フェーズにおける技術調査。Extension 型（既存システムへの追加）のため Light Discovery プロセスを適用。主な調査対象は2点: (1) `set_if_neq` パターンの handlers.rs への適用方法、(2) DPI 125% (scale=1.25) での座標変換往復精度。

---

## Research Log

### R-1: DPI 125% 座標変換往復精度

**問題**: `WindowPos.position`（物理px, i32）→ `Arrangement.offset`（DIP, f32）→ `GlobalArrangement.bounds`（物理px, f32）→ `WindowPos.position`（物理px, i32）の往復変換で、非2冪 DPI スケールにおいて浮動小数点丸め誤差が発生するか。

**調査手法**: IEEE 754 単精度浮動小数点の演算精度分析

**DPI 別分析**:

| DPI | scale | 1/scale 表現 | 往復精度 |
|-----|-------|--------------|----------|
| 96 (100%) | 1.0 | 正確 | ✅ 完全一致 |
| 120 (125%) | 1.25 | 0.8 = 非正確 (循環小数) | ⚠️ 後述 |
| 144 (150%) | 1.5 | 正確 (0.666...は使わない) | ✅ ほぼ一致 |
| 168 (175%) | 1.75 | 非正確 | ⚠️ 125% と同種 |
| 192 (200%) | 2.0 | 0.5 = 正確 | ✅ 完全一致 |

**125% (scale=1.25) の詳細分析**:

```
position = 301 (物理px)
offset = 301.0f32 / 1.25f32
```

IEEE 754 除算は正しく丸められるが、`301 / 1.25 = 240.8` の `240.8` は f32 で正確に表現できない（`4/5` は二進循環小数）。最近接偶数丸めにより `240.8f32 ≈ 240.80000305` または `240.79998779` のいずれかになる。

復路: `240.8f32 * 1.25f32` の結果が `301.0f32` に正確に戻るかは IEEE 754 では **保証されない**。

**最悪ケース**: `truncate` (`as i32`) での丸め方向により 1 ピクセルのシフトが発生する可能性:
- `240.79998779 * 1.25 = 300.99998...` → `as i32` = **300** (1px シフト!)

**影響評価**:
- **振動ではなく一方向シフト**: 300 に安定した後、`300 / 1.25 * 1.25 = 300.0` で固定
- **自然収束**: 2フレーム以内に安定（1フレーム目でシフト、2フレーム目で不動点到達）
- **視覚的影響**: 1px は知覚不能
- **頻度**: ウィンドウドラッグ中の一部のピクセル座標でのみ発生

**結論**: Low risk。現行の等値チェック + `Changed` フィルタで振動は防止される。`truncate` → `round` への変更で改善可能だが、wintf-fix4 のスコープ。

---

### R-2: `set_if_neq` パターンの handlers.rs 適用方法

**問題**: `WM_WINDOWPOSCHANGED` ハンドラで `WindowPos` のフィールドを個別に更新しているが、同一値書き込みでも `DerefMut` が発火して `Changed` フラグが立つ。

**既存パターン調査**:

| 箇所 | パターン | コード |
|------|----------|--------|
| `update_arrangements` (systems.rs L349) | `Mut<T>::set_if_neq()` | `arr.set_if_neq(new_arrangement)` |
| `propagate_global_arrangements` (tree_system.rs L282) | `Mut<T>::set_if_neq()` | `global_transform.set_if_neq(a * b)` |
| `bitmap_source.rs` L145 | `Mut<T>::set_if_neq()` | `cmd_list.set_if_neq(...)` |

上記はすべて **構造体全体** を `set_if_neq` で比較・代入するパターン。

**handlers.rs の特殊事情**:

handlers.rs では `EntityWorldMut::get_mut()` 経由で `Mut<WindowPos>` を取得し、**個別フィールド** を更新している:

```rust
window_pos.position = Some(client_pos);      // DerefMut → Changed!
window_pos.size = Some(client_size);          // (既に DerefMut 済み)
window_pos.last_sent_position = Some(...);
window_pos.last_sent_size = Some(...);
```

**問題点**: `position` と `size` が同一値でも `DerefMut` が発火し、`Changed<WindowPos>` が立つ。これにより `sync_window_arrangement_from_window_pos`（`Changed<WindowPos>` フィルタ付き）が不必要に実行される。

**解決策の比較**:

| 方式 | メリット | デメリット |
|------|----------|-----------|
| A: 全体 `set_if_neq` | シンプル | `last_sent` 更新が漏れる: `last_sent` 変更だけでも `Changed` が立つ |
| B: 条件分岐 + `bypass_change_detection()` | 正確な制御 | ボイラープレート増 |
| C: `last_sent` を別コンポーネントに分離 | 根本解決 | スコープ外 |

**推奨**: **方式 B（条件分岐 + `bypass_change_detection()`）**

```rust
let pos_changed = window_pos.position != Some(client_pos);
let size_changed = window_pos.size != Some(client_size);

if pos_changed || size_changed {
    // 実際の変更あり → 通常の DerefMut で Changed 発火
    window_pos.position = Some(client_pos);
    window_pos.size = Some(client_size);
    window_pos.last_sent_position = Some((client_pos.x, client_pos.y));
    window_pos.last_sent_size = Some((client_size.cx, client_size.cy));
} else {
    // 同一値（エコーバック等）→ bypass で Changed を抑制
    let wp = window_pos.bypass_change_detection();
    wp.last_sent_position = Some((client_pos.x, client_pos.y));
    wp.last_sent_size = Some((client_size.cx, client_size.cy));
}
```

**根拠**:
- `WindowPos` は `PartialEq` を derive 済みであり、`position` / `size` のフィールド単位比較が可能
- `bypass_change_detection()` は bevy_ecs 0.17 の公式 API（`Mut<T>` メソッド）
- `last_sent` フィールドはエコーバック検知用であり、変更検知を発火させる必要がない

---

## Architecture Pattern Evaluation

### 分類: Extension（既存システムへの統合）

**根拠**:
- 新規コンポーネント・リソースの追加なし
- 既存の commented-out コードのアンコメント + フィルタ追加が中心
- スケジュール登録パターンは既存の `.chain()` に倣う
- handlers.rs の `set_if_neq` パターンも既存プロジェクトで実績あり

**Full Discovery への昇格判断**: 不要。アーキテクチャ変更なし、外部依存追加なし、セキュリティ影響なし。

---

## Design Decisions

| ID | 決定事項 | 根拠 | 代替案 |
|----|----------|------|--------|
| D-1 | Option B' 採用（Changed + set_if_neq） | 効率的かつ既存パターン準拠 | Option A（最小変更）、Option B（Changed のみ） |
| D-2 | 条件分岐 + `bypass_change_detection()` | `last_sent` 更新で Changed を立てない | 全体 `set_if_neq`、`last_sent` 別コンポーネント化 |
| D-3 | `.chain()` による順序保証 | 既存 PostLayout パターン踏襲、`.after()` より簡潔 | 個別 `.after()` 指定 |
| D-4 | R-1 丸め誤差は対処不要 | 1px シフト・自然収束・知覚不能 | `truncate` → `round` 変更（fix4 スコープ） |

---

## Risks & Mitigations

| リスク | 確率 | 影響 | 緩和策 |
|--------|------|------|--------|
| DPI 125% で 1px シフト | Low | Very Low | 等値チェックで振動防止、fix4 で `round` 化検討 |
| `bypass_change_detection()` の誤用 | Very Low | Medium | `last_sent` のみに限定、位置・サイズは通常 DerefMut |
| 既存テスト regression | Low | Medium | `cargo test` + `taffy_flex_demo` で検証 |

---

## References

- bevy_ecs `Mut::set_if_neq` ドキュメント: bevy_ecs 0.17 API
- bevy_ecs `Mut::bypass_change_detection` ドキュメント: bevy_ecs 0.17 API
- IEEE 754-2019: 単精度浮動小数点演算の丸め規則
- 親仕様 report.md §3.4: フィードバックループ防止 L1/L2/L3 層
- gap-analysis.md: Option A/B/B'/C の評価と G1-G3 ギャップ
