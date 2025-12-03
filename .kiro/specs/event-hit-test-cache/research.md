# Research: ヒットテストキャッシュ化

## 概要

本ドキュメントは `event-hit-test-cache` 仕様の背景調査と設計根拠を記録する。
`event-mouse-basic` 仕様レビュー中の議論から派生した関連仕様として立ち上げられた。

---

## 背景：event-mouse-basic からの情報提供

### WM_NCHITTEST の頻度問題

`event-mouse-basic` 仕様のレビュー中、以下の問題が特定された：

**問題**: Win32 は `WM_NCHITTEST` を非常に高頻度で送信する

| 状況 | 頻度 |
|------|------|
| マウス移動中 | 1ピクセル移動ごと |
| システムUI更新 | 定期的に送信 |
| マウス静止時でも | システム要因で送信されることがある |

**影響**:
- 単純なヒットテスト実装では毎回 ECS World を走査
- World 借用問題（後述）との複合で深刻なパフォーマンス影響
- 60fps 維持に必要な 16ms のバジェットを圧迫

### World 借用問題

WndProc から World を借用する際の課題：

```
WM_NCHITTEST 処理:
1. WndProc が呼ばれる
2. hit_test のために World を借用（RefCell::borrow）
3. ヒット結果を計算
4. World を返却
5. HTCLIENT / HTTRANSPARENT を返す
```

**問題点**:
- `Rc<RefCell<EcsWorld>>` を WndProc から借用
- 借用中は他の処理が World にアクセス不可
- WM_NCHITTEST の頻度が高いため借用時間が累積

### 設計判断：キャッシュは「必須」

当初 `event-mouse-basic` の Requirement 6 はオプション扱いだったが、
議論の結果、以下の理由から**必須**と判断された：

1. **WM_NCHITTEST の頻度**: オプションでは許容できないほど高頻度
2. **World 借用コスト**: 毎回の借用オーバーヘッドが無視できない
3. **キャッシュヒット率**: 同一座標での連続呼び出しが多く、効果が高い

---

## 技術調査

### キャッシュ戦略の選択肢

#### 選択肢 1: World 内 Resource

```rust
#[derive(Resource)]
pub struct HitTestCache {
    entries: HashMap<HWND, WindowHitTestCache>,
}

struct WindowHitTestCache {
    last_screen_point: PhysicalPoint,
    last_hit_result: Option<(Entity, PhysicalPoint)>,
    frame_count: u32,
}
```

**利点**:
- ECS パターンに統合
- フレームカウントで自動無効化

**欠点**:
- World 借用が必要（そもそもの問題を解決しない）
- WM_NCHITTEST ごとに借用コスト発生

#### 選択肢 2: World 外スレッドローカルキャッシュ（推奨）

```rust
thread_local! {
    static HIT_TEST_CACHE: RefCell<HashMap<HWND, WindowHitTestCache>> = RefCell::new(HashMap::new());
}

struct WindowHitTestCache {
    last_screen_point: PhysicalPoint,
    last_hit_result: Option<(Entity, PhysicalPoint)>,
    frame_count: u32,  // 無効化判定用
}
```

**利点**:
- World 借用不要でキャッシュ確認可能
- キャッシュヒット時は World に触れない
- メインスレッド専用なので thread_local で十分

**欠点**:
- ECS パターンから外れる
- フレームカウント取得のための設計が必要

### キャッシュ無効化戦略

| トリガー | 説明 |
|---------|------|
| 座標変化 | 異なる座標でのクエリ |
| フレーム変化 | 新しいフレーム開始時 |
| レイアウト変更 | ArrangementTreeChanged イベント |

### フレームカウントの取得

World 外キャッシュでフレームカウントを参照する方法：

```rust
// グローバルフレームカウンター（Atomic）
static GLOBAL_FRAME_COUNT: AtomicU32 = AtomicU32::new(0);

// tick 時にインクリメント
fn try_tick_world(&mut self) -> bool {
    GLOBAL_FRAME_COUNT.fetch_add(1, Ordering::Relaxed);
    // ...
}

// キャッシュ無効化判定
fn is_cache_valid(cached_frame: u32) -> bool {
    cached_frame == GLOBAL_FRAME_COUNT.load(Ordering::Relaxed)
}
```

---

## 期待効果

### パフォーマンス改善

| メトリクス | キャッシュなし | キャッシュあり |
|-----------|--------------|---------------|
| WM_NCHITTEST 処理時間 | 0.1-1.0ms | 0.001ms (キャッシュヒット時) |
| World 借用頻度 | 毎回 | キャッシュミス時のみ |
| 60fps バジェット消費 | 高 | 低 |

### 予想キャッシュヒット率

- マウス静止時: 90%以上
- マウス低速移動時: 50-70%
- マウス高速移動時: 10-30%

---

## 関連仕様との関係

### event-mouse-basic との関係

- `event-mouse-basic` の Requirement 6 を独立仕様として分離
- `event-mouse-basic` はキャッシュ機構を前提として設計を進める
- 本仕様が完了するまで `event-mouse-basic` の WM_NCHITTEST 処理は仮実装

### event-hit-test との関係

- `event-hit-test` で実装された `hit_test` API のラッパーとして機能
- キャッシュミス時に `hit_test` を呼び出す

---

## 次のステップ

1. `/kiro-spec-requirements event-hit-test-cache` で要件定義を生成
2. World 外キャッシュ設計の詳細化
3. フレームカウント共有機構の設計
4. キャッシュ無効化条件の網羅

---

_Created: 2025-12-02_
_Updated: 2025-12-03 (Design Phase)_
_Source: event-mouse-basic 仕様レビュー議論_

---

## Design Phase 決定事項

### Discovery Scope
**Extension**（既存システムへの拡張）

### Key Findings
- `thread_local!` パターンは `window.rs` に2つの実装例（DpiChangeContext, SetWindowPosCommand）があり、参照可能
- WM_NCHITTEST ハンドラは `handlers.rs:399-460` に実装済み、キャッシュ統合ポイントが明確
- try_tick_world() は Layout スケジュールを含むため、1箇所のクリアで両トリガー（tick/layout）をカバー可能

---

## Design Decisions

### Decision 1: HashMap を採用

| 項目 | 内容 |
|------|------|
| Context | 座標→LRESULT のマッピングに使用するデータ構造の選択 |
| Alternatives | 1. HashMap<HWND, CacheEntry> — O(1)<br>2. Vec<(HWND, CacheEntry)> — O(n)<br>3. 単一エントリ — 最もシンプル |
| Selected | `HashMap<HWND, CacheEntry>` |
| Rationale | ウィンドウ数が1-2でもオーバーヘッドは無視可能。将来的な複数ウィンドウ対応の拡張性 |
| Trade-offs | 極小 N に対してオーバーヘッドがあるが実測可能な差にならない |

### Decision 2: ecs/ 直下に新規モジュール配置

| 項目 | 内容 |
|------|------|
| Context | キャッシュモジュールの配置場所 |
| Alternatives | 1. window.rs 拡張<br>2. nchittest_cache.rs 新規（ecs/ 直下）<br>3. window_proc/ 配下 |
| Selected | `ecs/nchittest_cache.rs` を新規作成 |
| Rationale | 責務分離、handlers.rs と world.rs 両方からアクセス容易、テスト容易性 |
| Trade-offs | モジュール数が1つ増加するが保守性向上のメリットが上回る |

### Decision 3: try_tick_world() 終了時に1回クリア

| 項目 | 内容 |
|------|------|
| Context | キャッシュクリアの呼び出しポイント |
| Alternatives | 1. try_tick_world() 終了時のみ<br>2. tick と layout 更新を別々に検知 |
| Selected | try_tick_world() 終了直前に1回クリア |
| Rationale | try_tick_world() は Layout スケジュールを含むため、1箇所で両トリガーをカバー |
| Trade-offs | tick 外で layout のみ更新されるケースは現在存在しない |

### Decision 4: Phase 1 は LRESULT のみキャッシュ

| 項目 | 内容 |
|------|------|
| Context | キャッシュする情報の範囲 |
| Alternatives | 1. LRESULT のみ<br>2. Entity + LRESULT 両方 |
| Selected | LRESULT（HTCLIENT/HTTRANSPARENT）のみ |
| Rationale | WM_NCHITTEST ハンドラは Entity 情報を使用せず LRESULT のみ返す。Phase 1 最小実装として十分 |
| Trade-offs | 将来 hit_test API 自体のキャッシュが必要になれば Phase 2 で対応 |

---

## Risks & Mitigations

| リスク | 影響度 | 軽減策 |
|-------|-------|-------|
| パフォーマンス退行 | 低 | キャッシュオーバーヘッドは極小。問題発生時は単一エントリへ簡略化 |
| メモリリーク | 低 | try_tick_world() で確実にクリア |
| 借用競合 | 低 | thread_local! + RefCell パターンは既存コードで実績あり |

---

## References

- [Rust std::collections::HashMap](https://doc.rust-lang.org/std/collections/struct.HashMap.html)
- [Rust std::cell::RefCell](https://doc.rust-lang.org/std/cell/struct.RefCell.html)
- [thread_local! マクロ](https://doc.rust-lang.org/std/macro.thread_local.html)
