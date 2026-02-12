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

→ このため `WM_DPICHANGED` 経路は統合ゲートのキュー機構の対象外であり、`DpiChangeContext` TLS + `try_borrow_mut()` 再入保護が本質的に必要。

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
|------|-------------|---------|
| R1: 単一ゲート統合 | `SetWindowPosCommand`, `WindowPosChanged`, `apply_window_pos_changes`, `WM_WINDOWPOSCHANGED` ハンドラ | **設計必要**: 統合方法の詳細設計（TLS vs ECS Resource）|
| R2: DpiChangeContext 維持 | `DpiChangeContext` | **ギャップなし**: 現行維持 |
| R3: エコーバック統合 | `is_echo()`, `last_sent_*`, `bypass_change_detection()` | **軽微**: モジュール/ドキュメント整理のみ |
| R4: 正確性保証 | 全メカニズム | **テスト不足**: UISetup パイプライン（`apply_window_pos_changes`）のテストが存在しない |
| R5: コード簡素化 | 4ステッププロトコル、3箇所 flush | **設計必要**: ステップ削減方法 |
| R6: 後方互換・テスト | `feedback_loop_convergence_test.rs` | **テスト拡張必要**: 統合ゲートの状態遷移テスト |

### 2.2 技術的課題

#### 課題1: TLS キューと ECS コンポーネントの統合先

`SetWindowPosCommand` は TLS（WndProc コールスタックで安全にアクセス可能）、`WindowPosChanged` は ECS コンポーネント（スケジュール内で Query 可能）。統合先の選択が設計上の最大決定ポイント。

- **TLS に統合**: ゲート状態を TLS に置き、`apply_window_pos_changes` 内で TLS を参照してスキップ判定。`WindowPosChanged` ECS コンポーネントは削除可能。
- **ECS Resource に統合**: ゲート状態を ECS `Resource` に置く。ただしWndProc ハンドラからのアクセスには World 借用が必要。
- **ハイブリッド**: TLS にキュー + ゲートフラグを置き、ECS コンポーネントは削除。

#### 課題2: 4ステッププロトコルの簡素化限界

①②③④のうち、①と④は別々の World 借用が必要（②の tick 中は World が借用済み）。この制約は `RefCell` アーキテクチャに起因し、統合ゲートを導入しても借用境界自体は変わらない。

→ 簡素化の範囲は「④を不要にできるか」（＝フラグリセットのステップ削除）が鍵。TLS ゲートなら ③ の flush 時に自動リセット可能。

#### 課題3: flush 呼び出しの集約

3箇所の flush のうち:
- `VsyncTick` トレイト内（#1）と `WM_VSYNC` メッセージループ内（#2）は**冗長**。#1 が実行されれば #2 は空の flush になる。
- `WM_WINDOWPOSCHANGED` ③（#3）は②の `VsyncTick` 内 #1 で既に実行済みの場合が多い。

→ ただし各呼び出し元の安全保証（flush が確実に呼ばれること）のために冗等呼び出しが存在。削減は可能だが慎重な検討が必要。

#### 課題4: テストカバレッジの不整合

現在のテストは PostLayout パイプラインのみ検証。以下が未テスト:
- `apply_window_pos_changes` の `WindowPosChanged` ガード動作
- `apply_window_pos_changes` の `is_echo()` ガード動作
- 統合ゲートの状態遷移（設定→スキップ→リセット）
- `WM_DPICHANGED` → `WM_WINDOWPOSCHANGED` チェーンの end-to-end

---

## 3. 実装アプローチ選択肢

### Option A: TLS ゲート統合（WindowPosChanged 削除）

**概要**: `WINDOW_POS_COMMANDS` TLS を拡張し、キューだけでなくゲートフラグも管理する `WindowPosGate` TLS に統合。`WindowPosChanged` ECS コンポーネントは削除。

**変更箇所**:
| ファイル | 変更内容 |
|---------|---------|
| `ecs/window.rs` | `WindowPosGate` TLS 構造体を新設（キュー + `suppressing: bool`）。`WindowPosChanged` 削除 |
| `ecs/graphics/systems.rs` | `apply_window_pos_changes` の Query から `WindowPosChanged` を除去、TLS `is_suppressing()` に変更 |
| `ecs/window_proc/handlers.rs` | ①④ の `WindowPosChanged` 操作を TLS `WindowPosGate::begin_external_sync()` / `end_external_sync()` に置換 |
| `ecs/world.rs` | スケジュール変更なし。`VsyncTick` 内の flush 呼び出しは維持 |

**4ステッププロトコルの変化**:
```
① World 第1借用 → DPI更新, [TLS] gate.begin_suppression(), WindowPos更新, BoxStyle更新 → 借用解放
② try_tick_on_vsync() → apply_window_pos_changes が TLS gate.is_suppressing() で skip
③ flush_window_pos_commands() + gate.end_suppression()  ← ③と④を統合
④ 削除
```

**トレードオフ**:
- ✅ World 第2借用（④）が不要になり、ステップが3に削減
- ✅ ECS コンポーネント1つ削除（SparseSet ストレージ解放）
- ✅ TLS に情報が集約され、1箇所で全状態を確認可能
- ❌ ECS システム内から TLS を参照するのは bevy_ecs のイディオムから外れる
- ❌ テストで TLS 状態のモック/制御が ECS Query より難しい

### Option B: ECS Resource ゲート統合（TLS キューは維持、WindowPosChanged を Resource 化）

**概要**: `WindowPosChanged` をエンティティコンポーネントから `Resource` に変更し、`HashMap<Entity, bool>` で全ウィンドウの抑制状態を管理。TLS キューは物理的制約（World 借用外 flush）のため維持。

**変更箇所**:
| ファイル | 変更内容 |
|---------|---------|
| `ecs/window.rs` | `WindowPosSuppression` Resource（`HashMap<Entity, bool>`）新設。`WindowPosChanged` コンポーネント削除 |
| `ecs/graphics/systems.rs` | `apply_window_pos_changes` の Query 変更、`Res<WindowPosSuppression>` 参照 |
| `ecs/window_proc/handlers.rs` | ① で Resource の entity キーを設定、④ でリセット |

**トレードオフ**:
- ✅ bevy_ecs の Resource パターンに準拠
- ✅ テスト容易性が高い（World に Resource を挿入してテスト可能）
- ❌ 4ステッププロトコルは維持（④が必要）
- ❌ HashMap 管理のオーバーヘッド（軽微だがエンティティ削除時のクリーンアップ必要）
- ❌ TLS キューとの二重管理は残る

### Option C: TLS ゲート統合 + flush 集約（推奨ハイブリッド）

**概要**: Option A の TLS ゲート統合に加え、flush 呼び出しポイントを整理。`flush()` 自体にゲートリセットを組み込み、③と④を完全統合。

**変更箇所**:
| ファイル | 変更内容 |
|---------|---------|
| `ecs/window.rs` | `WindowPosGate` TLS 構造体（キュー + 抑制フラグ + エコーバック記録）。`WindowPosChanged` 削除 |
| `ecs/graphics/systems.rs` | `WindowPosChanged` Query 除去、TLS ゲートチェックに変更。エコーバック記録も TLS 経由 |
| `ecs/window_proc/handlers.rs` | ① で `WindowPosGate::begin_wm_sync(entity)` 呼び出し、③④ を `WindowPosGate::flush_and_reset()` に統合 |
| `ecs/world.rs` | flush 呼び出しは主要2箇所に集約（VsyncTick + WM_VSYNC） |
| `win_thread_mgr.rs` | flush 呼び出し維持（安全保証） |

**4ステッププロトコルの変化**:
```
① World 第1借用 → DPI更新, [TLS] gate.begin_wm_sync(entity), WindowPos更新, BoxStyle更新 → 借用解放
② try_tick_on_vsync() → apply_window_pos_changes が TLS gate.is_suppressed(entity) で skip
③ flush_and_reset() ← flush + 抑制解除を一体化。④は不要
```

**トレードオフ**:
- ✅ 最大の簡素化（4ステップ→3ステップ、World 第2借用削除）
- ✅ ゲート状態 + キュー + エコーバック記録が単一 TLS に集約
- ✅ `flush_and_reset()` により状態リセット忘れのリスク排除
- ❌ TLS 構造体がやや複雑化（3つの責務を持つ）
- ❌ ECS システム内の TLS アクセスは bevy_ecs イディオム外
- ❌ 単体テストで TLS 状態の制御が必要

---

## 4. 複雑度とリスク

### 工数見積もり: **M（3–5日）**

- TLS ゲート構造体の設計・実装: 1日
- `apply_window_pos_changes` のガード変更: 0.5日
- `WM_WINDOWPOSCHANGED` ハンドラの 4→3 ステップ化: 1日
- エコーバック検知の再配置: 0.5日
- テスト拡張（統合ゲートのユニットテスト + 既存テスト適合）: 1日
- 手動検証（taffy_flex_demo 等でのスムーズ動作確認）: 1日

### リスク: **Medium**

| リスク | 影響度 | 緩和策 |
|--------|--------|--------|
| TLS ゲートフラグのリセット漏れ | 高（フィードバックループ永続化） | `flush_and_reset()` で flush と同時にリセット。Drop guard 検討 |
| `WM_DPICHANGED` 経路との干渉 | 高（DPI 変更時の不具合） | `DpiChangeContext` は独立維持。`WM_DPICHANGED` は直接 SetWindowPos なのでゲート対象外 |
| bevy_ecs イディオム逸脱 | 低（保守性） | doc comment で TLS 使用理由を明記 |
| 複数ウィンドウでのゲート競合 | 中（マルチウィンドウ不具合） | TLS ゲートをウィンドウ(hwnd/entity)単位で管理、または全ウィンドウ共通フラグで十分かを設計時に評価 |

---

## 5. 推奨アプローチと設計フェーズへの引継ぎ

### 推奨: Option C（TLS ゲート統合 + flush 集約）

**理由**:
- 最大のコード簡素化効果（要件R5に最も合致）
- World 第2借用（④）の削除は実質的な複雑度低減
- TLS はシングルスレッド UI アーキテクチャで既に `DpiChangeContext` で使用済みの確立パターン
- エコーバック記録を TLS に移すことで `WindowPos` 構造体の責務が純粋化

### 設計フェーズで決定すべき事項

| # | 決定事項 | 選択肢 | 判断材料 |
|---|---------|--------|----------|
| D1 | TLS ゲートの粒度 | (a) 全ウィンドウ共通フラグ / (b) Per-entity フラグ | 複数ウィンドウの同時 `WM_WINDOWPOSCHANGED` 処理が現実的に発生するか |
| D2 | `last_sent_*` の配置 | (a) TLS ゲート内に移動 / (b) `WindowPos` に残留 | テスト容易性 vs 一貫性 |
| D3 | flush 呼び出しポイント | (a) 2箇所に削減 / (b) 3箇所維持（冪等保証） | 安全性 vs 簡潔性 |
| D4 | `WindowPosChanged` の完全削除 vs マーカー残留 | (a) 完全削除 / (b) デバッグ用に残す | 外部から観測可能なフラグが必要か |

### Research Items（設計フェーズ向け）

| # | 調査項目 | 目的 |
|---|---------|------|
| RI-1 | bevy_ecs での TLS アクセスのベストプラクティス | ECS システム内の TLS 参照が他プロジェクトでどう扱われているか |
| RI-2 | `WM_WINDOWPOSCHANGED` の同期発火タイミング詳細 | `SetWindowPos` 呼び出し→ `WM_WINDOWPOSCHANGED` が**必ず**同期的か、非同期になるケースはあるか |
| RI-3 | マルチウィンドウでの `WM_WINDOWPOSCHANGED` 発火順序 | 1つの `SetWindowPos` で複数ウィンドウに連鎖的に通知が来るケースの調査 |
