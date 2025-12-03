````markdown
# Requirements Document

| 項目 | 内容 |
|------|------|
| **Document Title** | event-hit-test-cache 要件定義書 |
| **Version** | 1.0 (Draft) |
| **Date** | 2025-12-03 |
| **Parent Spec** | event-mouse-basic |
| **Related Specs** | event-hit-test |
| **Author** | AI-DLC System |

---

## Introduction

本仕様書は wintf フレームワークにおけるヒットテストキャッシュ機構の要件を定義する。`event-mouse-basic` 仕様から派生した関連仕様として、WM_NCHITTEST の高頻度呼び出しに対するパフォーマンス最適化を提供する。

### 背景

Win32 は `WM_NCHITTEST` を非常に高頻度で送信する：

| 状況 | 頻度 |
|------|------|
| マウス移動中 | 1ピクセル移動ごと |
| システムUI更新 | 定期的に送信 |
| マウス静止時 | システム要因で送信されることがある |

毎回 ECS World を借用してヒットテストを実行すると、以下の問題が発生する：

- **World 借用コスト**: `Rc<RefCell<EcsWorld>>` の借用オーバーヘッドが累積
- **60fps バジェット圧迫**: 16ms のフレームバジェットを圧迫
- **パフォーマンス劣化**: 高頻度呼び出しによる処理負荷

### スコープ

**含まれるもの**:
- World 外スレッドローカルキャッシュの実装
- 座標ベースのキャッシュ判定
- フレームカウントによるキャッシュ無効化
- グローバルフレームカウンター
- レイアウト変更時のキャッシュ無効化

**含まれないもの**:
- ヒットテストロジック本体 → `event-hit-test` 仕様で実装済み
- マウスイベント処理 → `event-mouse-basic` 仕様で対応
- αマスクヒットテスト → `event-hit-test-alpha-mask` 仕様で対応

### 設計決定

**World 外キャッシュを採用する理由**:

World 内 Resource（`#[derive(Resource)]`）ではなく、スレッドローカルキャッシュを採用する：

| 観点 | World 内 Resource | World 外スレッドローカル |
|------|-------------------|--------------------------|
| キャッシュ確認時の World 借用 | 必要 | 不要 |
| ECS パターン統合 | ○ | △ |
| WM_NCHITTEST パフォーマンス | 改善なし | 大幅改善 |

**結論**: キャッシュヒット時に World 借用を回避することが主目的のため、World 外キャッシュを採用。

---

## Requirements

### Requirement 1: スレッドローカルキャッシュ

**Objective:** 開発者として、ヒットテスト結果をキャッシュしたい。それにより WM_NCHITTEST の高頻度呼び出しに対するパフォーマンスを向上できる。

#### Acceptance Criteria

1. The HitTest Cache System shall スレッドローカル変数でウィンドウごとのキャッシュを管理する
2. The HitTest Cache System shall キャッシュエントリにスクリーン座標（物理ピクセル）を保持する
3. The HitTest Cache System shall キャッシュエントリにヒット結果（Entity と ローカル座標）を保持する
4. The HitTest Cache System shall キャッシュエントリにフレームカウントを保持する
5. The HitTest Cache System shall ウィンドウ（HWND）ごとに独立したキャッシュエントリを管理する
6. When キャッシュ確認時, the HitTest Cache System shall World 借用なしでキャッシュの有効性を判定する

#### 構造定義

```rust
thread_local! {
    static HIT_TEST_CACHE: RefCell<HashMap<isize, WindowHitTestCache>> = RefCell::new(HashMap::new());
}

/// ウィンドウごとのヒットテストキャッシュ
struct WindowHitTestCache {
    /// 前回のスクリーン座標（物理ピクセル）
    last_screen_point: PhysicalPoint,
    /// 前回のヒット結果（ヒットなしの場合は None）
    last_hit_result: Option<(Entity, PhysicalPoint)>,
    /// キャッシュ時のフレームカウント
    frame_count: u32,
}
```

---

### Requirement 2: キャッシュヒット判定

**Objective:** 開発者として、同一座標・同一フレームでのヒットテストをスキップしたい。それによりWorld借用なしで結果を返せる。

#### Acceptance Criteria

1. When 同一スクリーン座標でヒットテストが要求された時, the HitTest Cache System shall キャッシュからヒット結果を返す
2. When 同一フレーム内でヒットテストが要求された時, the HitTest Cache System shall キャッシュを有効とみなす
3. When 座標が異なる場合, the HitTest Cache System shall キャッシュミスとして実際のヒットテストを実行する
4. When フレームカウントが異なる場合, the HitTest Cache System shall キャッシュを無効化し実際のヒットテストを実行する
5. The HitTest Cache System shall キャッシュヒット時に World を借用しない

#### キャッシュ判定ロジック

```rust
fn is_cache_valid(cache: &WindowHitTestCache, screen_point: PhysicalPoint, current_frame: u32) -> bool {
    cache.frame_count == current_frame && cache.last_screen_point == screen_point
}
```

---

### Requirement 3: グローバルフレームカウンター

**Objective:** 開発者として、World 外からフレームカウントを取得したい。それによりキャッシュ無効化判定を World 借用なしで行える。

#### Acceptance Criteria

1. The HitTest Cache System shall グローバルな `AtomicU32` フレームカウンターを提供する
2. When ECS tick が実行された時, the HitTest Cache System shall フレームカウンターをインクリメントする
3. The HitTest Cache System shall `get_current_frame_count()` 関数でフレームカウントを取得可能とする
4. The HitTest Cache System shall フレームカウンターを Relaxed メモリオーダリングでアクセスする

#### 実装例

```rust
use std::sync::atomic::{AtomicU32, Ordering};

static GLOBAL_FRAME_COUNT: AtomicU32 = AtomicU32::new(0);

/// フレームカウントをインクリメント（tick 時に呼び出し）
pub fn increment_frame_count() {
    GLOBAL_FRAME_COUNT.fetch_add(1, Ordering::Relaxed);
}

/// 現在のフレームカウントを取得
pub fn get_current_frame_count() -> u32 {
    GLOBAL_FRAME_COUNT.load(Ordering::Relaxed)
}
```

---

### Requirement 4: キャッシュ公開API

**Objective:** 開発者として、キャッシュ付きヒットテストAPIを使用したい。それにより透過的にキャッシュ最適化の恩恵を受けられる。

#### Acceptance Criteria

1. The HitTest Cache System shall `cached_hit_test(hwnd, screen_point)` 関数を提供する
2. When キャッシュヒット時, the `cached_hit_test` function shall World 借用なしで結果を返す
3. When キャッシュミス時, the `cached_hit_test` function shall 実際のヒットテストを実行しキャッシュを更新する
4. The `cached_hit_test` function shall `event-hit-test` 仕様の `hit_test_detailed` APIを内部で使用する
5. The HitTest Cache System shall 戻り値として `Option<(Entity, PhysicalPoint)>` を返す

#### API シグネチャ

```rust
/// キャッシュ付きヒットテスト
/// 
/// キャッシュヒット時は World 借用なしで結果を返す。
/// キャッシュミス時は hit_test_detailed を呼び出しキャッシュを更新。
/// 
/// # Arguments
/// * `hwnd` - ウィンドウハンドル
/// * `screen_point` - スクリーン座標（物理ピクセル）
/// * `world` - ECS World（キャッシュミス時のみ使用）
/// 
/// # Returns
/// * `Some((entity, local_point))` - ヒットしたエンティティとローカル座標
/// * `None` - ヒットなし
pub fn cached_hit_test(
    hwnd: HWND,
    screen_point: PhysicalPoint,
    world: &World,
) -> Option<(Entity, PhysicalPoint)>;
```

---

### Requirement 5: レイアウト変更時の無効化

**Objective:** 開発者として、レイアウト変更時にキャッシュを無効化したい。それにより古いヒット結果を返すことを防げる。

#### Acceptance Criteria

1. When `ArrangementTreeChanged` イベントが発生した時, the HitTest Cache System shall 該当ウィンドウのキャッシュを無効化する
2. When ウィンドウが破棄された時（WM_DESTROY）, the HitTest Cache System shall 該当ウィンドウのキャッシュエントリを削除する
3. The HitTest Cache System shall `invalidate_cache(hwnd)` 関数を提供する
4. The HitTest Cache System shall `clear_cache(hwnd)` 関数を提供する

#### 無効化タイミング

| トリガー | アクション |
|---------|-----------|
| 座標変化 | 自動無効化（キャッシュ判定で処理） |
| フレーム変化 | 自動無効化（フレームカウント比較） |
| `ArrangementTreeChanged` | `invalidate_cache(hwnd)` 呼び出し |
| WM_DESTROY | `clear_cache(hwnd)` 呼び出し |

---

## Non-Functional Requirements

### NFR-1: パフォーマンス目標

**Objective:** システムとして、WM_NCHITTEST 処理時間を大幅に削減したい。

#### Acceptance Criteria

1. When キャッシュヒット時, the HitTest Cache System shall 0.01ms 以下で結果を返す
2. The HitTest Cache System shall キャッシュ確認のために World を借用しない
3. The HitTest Cache System shall 60fps バジェット（16ms）への影響を最小化する

#### 期待効果

| メトリクス | キャッシュなし | キャッシュあり（ヒット時） |
|-----------|--------------|---------------------------|
| WM_NCHITTEST 処理時間 | 0.1-1.0ms | 0.001ms |
| World 借用 | 毎回 | キャッシュミス時のみ |

### NFR-2: スレッドセーフティ

**Objective:** システムとして、メインスレッドでの安全な動作を保証したい。

#### Acceptance Criteria

1. The HitTest Cache System shall メインスレッド専用として設計する
2. The HitTest Cache System shall `thread_local!` マクロを使用してスレッドローカルストレージを実装する
3. The HitTest Cache System shall グローバルフレームカウンターを `AtomicU32` で実装する

---

## Glossary

| 用語 | 定義 |
|------|------|
| WM_NCHITTEST | Win32 メッセージ。マウス位置がウィンドウのどの部分にあるかを問い合わせる |
| キャッシュヒット | 前回と同一条件でのヒットテスト要求。キャッシュから結果を返す |
| キャッシュミス | 条件が変化したヒットテスト要求。実際のヒットテストを実行 |
| フレームカウント | ECS tick の実行回数。キャッシュ有効期限の判定に使用 |
| スレッドローカル | スレッドごとに独立した変数。`thread_local!` マクロで実装 |

---

## Appendix A: 予想キャッシュヒット率

| 状況 | 予想ヒット率 |
|------|-------------|
| マウス静止時 | 90%以上 |
| マウス低速移動時 | 50-70% |
| マウス高速移動時 | 10-30% |

---

## Appendix B: キャッシュ動作シーケンス

```
WM_NCHITTEST 受信
    │
    ▼
cached_hit_test(hwnd, screen_point, world) 呼び出し
    │
    ▼
┌─────────────────────────────────────┐
│ キャッシュ有効性チェック              │
│ (World 借用なし)                     │
│                                     │
│ 1. フレームカウント一致?             │
│ 2. 座標一致?                        │
└─────────────────────────────────────┘
    │
    ├── Yes (キャッシュヒット)
    │       │
    │       ▼
    │   キャッシュから結果を返す
    │   (World 借用なし)
    │
    └── No (キャッシュミス)
            │
            ▼
        World を借用
            │
            ▼
        hit_test_detailed 実行
            │
            ▼
        キャッシュ更新
            │
            ▼
        結果を返す
```

````