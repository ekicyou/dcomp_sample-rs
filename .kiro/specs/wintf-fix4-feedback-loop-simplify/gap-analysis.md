# ギャップ分析: wintf-fix4-feedback-loop-simplify

## 1. 現状調査

### 1.1 関連アセットマップ

| アセット | ファイル | 行 | 役割 |
|---------|----------|-----|------|
| `SetWindowPosCommand` 構造体 | `ecs/window.rs` | L92–L101 | Win32 `SetWindowPos` の遅延実行コマンド |
| `WINDOW_POS_COMMANDS` TLS | `ecs/window.rs` | L103–L105 | コマンドキュー（`RefCell<Vec<…>>`） |
| `enqueue()` / `flush()` | `ecs/window.rs` | L130–L196 | キュー操作メソッド |
| `WindowPosChanged` コンポーネント | `ecs/window.rs` | L202–L211 | `SparseSet` ECS コンポーネント（bool フラグ） |
| `WindowPos.last_sent_position/size` | `ecs/window.rs` | L687–L688 | エコーバック検知用フィールド |
| `WindowPos.is_echo()` | `ecs/window.rs` | L935–L939 | エコーバック判定メソッド |
| `DpiChangeContext` TLS | `ecs/window.rs` | L25–L68 | DPI 値の TLS 伝達 |
| `apply_window_pos_changes` | `ecs/graphics/systems.rs` | L700–L810 | ECS→Win32 同期システム（UISetup） |
| `sync_window_arrangement_from_window_pos` | `ecs/layout/systems.rs` | L460–L502 | Win32→ECS 逆同期（PostLayout） |
| `window_pos_sync_system` | `ecs/layout/systems.rs` | L370–L436 | Arrangement→WindowPos 同期（PostLayout） |
| `WM_WINDOWPOSCHANGED` ハンドラ | `ecs/window_proc/handlers.rs` | L111–L328 | 4ステッププロトコル |
| `WM_DPICHANGED` ハンドラ | `ecs/window_proc/handlers.rs` | L346–L425 | DPI 変更処理（直接 SetWindowPos） |
| `VsyncTick` トレイト実装 | `ecs/world.rs` | L38–L68 | `Rc<RefCell<EcsWorld>>` の tick + flush |
| `WM_VSYNC` ハンドラ | `win_thread_mgr.rs` | L239–L256 | メッセージループ内の tick + flush |
| スケジュール登録 | `ecs/world.rs` | L301, L357–L367 | UISetup / PostLayout システム登録 |
| テスト | `tests/feedback_loop_convergence_test.rs` | 全505行 | PostLayout パイプラインの収束テスト |

### 1.2 既存アーキテクチャパターン

#### 4ステッププロトコル（`WM_WINDOWPOSCHANGED` ハンドラ）

```
① World 第1借用 → DPI更新, WindowPosChanged=true, WindowPos更新, BoxStyle更新 → 借用解放
② try_tick_on_vsync() (内部で借用→解放→flush)
③ flush_window_pos_commands()
④ World 第2借用 → WindowPosChanged=false → 借用解放
```

#### flush 呼び出し箇所（3箇所）

| # | 場所 | ファイル | 行 | 呼び出しコンテキスト |
|---|------|----------|-----|---------------------|
| 1 | `VsyncTick` トレイト実装 | `world.rs` | L67 | `try_borrow_mut()` 成功・失敗にかかわらず実行 |
| 2 | `WM_VSYNC` メッセージループ | `win_thread_mgr.rs` | L248 | `borrow_mut()` → tick → 借用解放後 |
| 3 | `WM_WINDOWPOSCHANGED` ③ | `handlers.rs` | L308 | ② の後、④ の前 |

#### enqueue 呼び出し箇所（1箇所のみ）

| 場所 | ファイル | 行 |
|------|----------|-----|
| `apply_window_pos_changes` | `graphics/systems.rs` | L803 |

#### `WM_DPICHANGED` の特殊性

`WM_DPICHANGED` ハンドラは `SetWindowPosCommand::enqueue()` を**使用せず**、**直接** `SetWindowPos()` を呼ぶ（`handlers.rs` L408–L420）。これは `DefWindowProcW` の代わりに明示的に `suggested_rect` を適用するためである。この呼び出しにより `WM_WINDOWPOSCHANGED` が**同期的に**発火し、TLS 経由の `DpiChangeContext` を消費する。

→ このため `WM_DPICHANGED` 経路はコマンドキュー機構の対象外だが、ラッパー方式では同じ `guarded_set_window_pos()` を使用することで統一的に保護される。`DpiChangeContext` TLS は独立して維持される（R2）。

### 1.3 依存方向・制約

```
COM Layer → ECS Layer → Message Handling Layer
                ↕ (フィードバック)
         Win32 API (SetWindowPos / WM_*)
```

- `SetWindowPosCommand` は TLS（スレッド固有）— シングルスレッド UI アーキテクチャ前提
- `WindowPosChanged` は ECS SparseSet — bevy_ecs の変更検知と連動
- `RefCell<EcsWorld>` 再入保護は全 WndProc ハンドラで共通パターン
- `DpiChangeContext` は WndProc コールスタック内のみ有効（TLS set → 同期 take）

---

## 2. 要件実現性分析

### 2.1 要件-アセットマッピング

| 要件 | 関連アセット | ギャップ |
|------|-------------|--------|
| R1: SetWindowPos ラッパー | `SetWindowPosCommand`, `WindowPosChanged`, `is_echo()`, `last_sent_*`, `apply_window_pos_changes`, `WM_WINDOWPOSCHANGED` ハンドラ, `WM_DPICHANGED` ハンドラ | **設計必要**: ラッパー関数の API 設計、TLS フラグ構造 |
| R2: DpiChangeContext 維持 | `DpiChangeContext` | **ギャップなし**: 現行維持 |
| R3: 正確性保証 | 全メカニズム | **テスト不足**: UISetup パイプラインのテストが存在しない |
| R4: コード簡素化 | 4ステッププロトコル、3箇所 flush | **設計必要**: ステップ削減、削除対象の特定 |
| R5: 後方互換・テスト | `feedback_loop_convergence_test.rs` | **テスト拡張必要**: ラッパー TLS フラグの単体テスト |

### 2.2 技術的課題

#### 課題1: ラッパー方式の核心設計

`SetWindowPos` → `WM_WINDOWPOSCHANGED` は同期呼び出し（同一コールスタック内で発火）であるため、ラッパー関数のスコープで TLS フラグを管理するだけで、`WM_WINDOWPOSCHANGED` ハンドラ内で「自アプリ由来か外部由来か」を直接判定できる。

```rust
// 概念的なラッパー
fn guarded_set_window_pos(hwnd, ...) {
    IS_OUR_CALL.set(true);
    SetWindowPos(hwnd, ...);  // ← WM_WINDOWPOSCHANGED が同期発火
    IS_OUR_CALL.set(false);
}
```

これにより以下が不要になる：
- `WindowPosChanged` ECS コンポーネント（WndProc ハンドラでの設定/リセット）
- `last_sent_position` / `last_sent_size` フィールド（値比較による間接的エコーバック検知）
- `is_echo()` メソッド
- 4ステッププロトコルの④（World 第2借用での `WindowPosChanged=false` リセット）

ラッパー方式の適用範囲：

| 呼び出し元 | 現行 | ラッパー後 |
|------------|--------|----------|
| `flush()` 内の `SetWindowPos` | 直接呼び出し | ラッパー経由 |
| `WM_DPICHANGED` ハンドラの `SetWindowPos` | 直接呼び出し（キュー未使用） | ラッパー経由（統一） |

#### 課題2: `WM_WINDOWPOSCHANGED` ハンドラの簡素化

現行の4ステッププロトコル：
```
① World 第1借用 → DPI更新, WindowPosChanged=true, WindowPos更新, BoxStyle更新 → 借用解放
② try_tick_on_vsync() (内部で借用→解放→flush)
③ flush_window_pos_commands()
④ World 第2借用 → WindowPosChanged=false → 借用解放
```

ラッパー方式後：
```
① World 第1借用 → DPI更新, WindowPos更新, BoxStyle更新 → 借用解放
② try_tick_on_vsync() (内部で借用→解放→flush)
③ flush_window_pos_commands()
```

① で `WindowPosChanged=true` の設定が不要、④ が丸ごと削除。ハンドラが TLS フラグをチェックするだけで「自アプリ由来か」を判定。

#### 課題3: `apply_window_pos_changes` のガード変更

現行では `apply_window_pos_changes` 内に2つのガードがある：
1. `WindowPosChanged` コンポーネントチェック（ECS Query）
2. `is_echo()` 値比較

ラッパー方式ではこれらが不要になるが、ガードの完全削除が可能かどうかは設計時に検証が必要。ラッパーで TLS フラグが ON の間に発火した `WM_WINDOWPOSCHANGED` は `Changed<WindowPos>` を発火させないため、`apply_window_pos_changes` のトリガー自体が発火しないはず。

#### 課題4: flush 呼び出しの集約

3箇所の flush のうち:
- `VsyncTick` トレイト内（#1）と `WM_VSYNC` メッセージループ内（#2）は**冗長**。#1 が実行されれば #2 は空の flush になる。
- `WM_WINDOWPOSCHANGED` ③（#3）は②の `VsyncTick` 内 #1 で既に実行済みの場合が多い。

→ ただし各呼び出し元の安全保証（flush が確実に呼ばれること）のために冗等呼び出しが存在。削減は可能だが慎重な検討が必要。

#### 課題5: テストカバレッジの不整合

現在のテストは PostLayout パイプラインのみ検証。以下が未テスト:
- ラッパー関数の TLS フラグの ON/OFF 動作
- `WM_WINDOWPOSCHANGED` ハンドラでのフラグ参照動作
- `WM_DPICHANGED` 経路のラッパー統一

---

## 3. 実装アプローチ: SetWindowPos ラッパー方式

### 核心アイデア

`SetWindowPos` → `WM_WINDOWPOSCHANGED` は同期呼び出し（同一コールスタック内で発火）であるため、`SetWindowPos` をラッパー関数で囲み TLS フラグを管理するだけで、「自アプリ由来の `WM_WINDOWPOSCHANGED` か」を直接判定できる。

これにより `WindowPosChanged` ECS コンポーネント（ハンドラでの設定/リセット）と `is_echo()` 値比較検知の**両方を代替**できる。

### 変更箇所

| ファイル | 変更内容 |
|---------|----------|
| `ecs/window.rs` | ラッパー関数 `guarded_set_window_pos()` 新設、TLS フラグ `IS_SELF_INITIATED` 新設。`WindowPosChanged` 削除。`WindowPos` から `last_sent_position` / `last_sent_size` / `is_echo()` 削除 |
| `ecs/graphics/systems.rs` | `apply_window_pos_changes` から `WindowPosChanged` Query 除去、`is_echo()` ガード除去、`last_sent_*` 記録除去。`enqueue()` は維持 |
| `ecs/window_proc/handlers.rs` | `WM_WINDOWPOSCHANGED` ハンドラ: ①から `WindowPosChanged=true` 削除、TLS フラグ参照で echo 判定追加、④（World 第2借用）削除。`WM_DPICHANGED` ハンドラ: 直接 `SetWindowPos` → ラッパー経由に変更 |
| `ecs/window.rs` (flush) | `flush()` 内の `SetWindowPos` 呼び出しをラッパー経由に変更 |
| テスト | ラッパー TLS フラグの単体テスト追加。既存 `feedback_loop_convergence_test.rs` は `is_echo`/`last_sent` 使用箇所のみ修正 |

### 削除対象一覧

| 削除対象 | ファイル | 理由 |
|----------|----------|------|
| `WindowPosChanged` コンポーネント | `ecs/window.rs` | TLS ラッパーフラグで代替 |
| `WindowPos.last_sent_position` | `ecs/window.rs` | ラッパー方式で値比較不要 |
| `WindowPos.last_sent_size` | `ecs/window.rs` | 同上 |
| `WindowPos.is_echo()` | `ecs/window.rs` | 同上 |
| `apply_window_pos_changes` 内 `WindowPosChanged` ガード | `ecs/graphics/systems.rs` | TLS フラグ or ガード自体不要 |
| `apply_window_pos_changes` 内 `is_echo()` ガード | `ecs/graphics/systems.rs` | 同上 |
| `apply_window_pos_changes` 内 `last_sent_*` 記録 | `ecs/graphics/systems.rs` | 同上 |
| `WM_WINDOWPOSCHANGED` ④（World 第2借用） | `handlers.rs` | フラグリセット不要 |

### トレードオフ

- ✅ 根本的な簡素化（値比較検知 + ECS コンポーネントフラグ → コールスタックスコープの TLS フラグ）
- ✅ 4ステップ → 3ステップ（World 第2借用削除）
- ✅ `WM_DPICHANGED` 経路も統一的にラッパーで保護
- ✅ TLS は既に `DpiChangeContext` で使用済みの確立パターン
- ❌ ECS システム内からの TLS アクセスは bevy_ecs イディオム外（ガードが不要になるためこのデメリットは実質的に解消）
- ❌ 単体テストで TLS 状態の制御が必要

---

## 4. 複雑度とリスク

### 工数見積もり: **S（1–3日）**

ラッパー方式は旧 Option A/B/C より大幅にシンプル：
- ラッパー関数 + TLS フラグ実装: 0.5日
- `apply_window_pos_changes` のガード削除 + `WindowPos` フィールド削除: 0.5日
- `WM_WINDOWPOSCHANGED` ハンドラ 4→3ステップ化 + `WM_DPICHANGED` ラッパー統一: 0.5日
- テスト拡張 + 手動検証: 1日

### リスク: **Low**

| リスク | 影響度 | 緩和策 |
|--------|--------|--------|
| `SetWindowPos` が非同期に `WM_WINDOWPOSCHANGED` を発火するケース | 高 | MSDN では同期と明記。RI-2 で裏取り。万が一の保険として `RefCell` 再入保護（L3）が存在 |
| `WM_DPICHANGED` 経路でのラッパー適用忘れ | 中 | ラッパーを唯一の `SetWindowPos` 呼び出し口とし、`unsafe SetWindowPos` の直接呼び出しを禁止する doc comment で保護 |
| TLS フラグのリセット漏れ | 低 | ラッパー関数のスコープで自動管理されるためリセット漏れは構造的に不可能 |

---

## 5. 推奨アプローチと設計フェーズへの引継ぎ

### 推奨: SetWindowPos ラッパー方式

**理由**:
- 「`SetWindowPos` → `WM_WINDOWPOSCHANGED` は同期」という Win32 の性質を直接活用する最も自然な設計
- `WindowPosChanged` + `is_echo()` + `last_sent_*` の3つを一挙に削除できる
- 4ステップ → 3ステップ（World 第2借用削除）
- `WM_DPICHANGED` 経路も同じラッパーで統一
- 工数が旧アプローチ（M: 3-5日）より削減（S: 1-3日）

### 設計フェーズで決定すべき事項

| # | 決定事項 | 選択肢 | 判断材料 |
|---|---------|--------|----------|
| D1 | ラッパー関数の API 設計 | (a) 単純 TLS bool (b) RAII Drop guard | 安全性 vs シンプルさ。同期呼び出し保証があるので bool で十分か |
| D2 | flush 呼び出しポイント | (a) 2箇所に削減 / (b) 3箇所維持（冪等保証） | 安全性 vs 簡潔性 |
| D3 | `WM_WINDOWPOSCHANGED` ハンドラでの echo 判定時の動作 | (a) 値更新を完全スキップ / (b) `bypass_change_detection()` で更新しつつ Changed 抑制 | データ整合性 vs パフォーマンス |

### Research Items（設計フェーズ向け）

| # | 調査項目 | 目的 |
|---|---------|------|
| RI-1 | `SetWindowPos` 呼び出し → `WM_WINDOWPOSCHANGED` の同期発火タイミング詳細 | 同期保証の MSDN 裏取り。非同期になるケースがあるか |
| RI-2 | ~~マルチウィンドウでの `WM_WINDOWPOSCHANGED` 発火順序~~ | ラッパー方式では TLS フラグがウィンドウ非依存で全 `SetWindowPos` 呼び出しを保護するため、マルチウィンドウ連鎖は原理的に発生しない。**調査不要** |
