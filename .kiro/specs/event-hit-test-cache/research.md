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
_Source: event-mouse-basic 仕様レビュー議論_
