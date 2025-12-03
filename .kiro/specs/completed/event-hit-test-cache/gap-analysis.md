# Gap Analysis Report: event-hit-test-cache

## 分析日時
2025年1月

## 分析対象
**Feature**: WM_NCHITTEST キャッシュ Phase 1 最小実装
**Requirements Version**: 1.0

---

## 1. 既存アセット分析

### 1.1 再利用可能なコンポーネント

| コンポーネント | 場所 | 再利用方法 |
|-------------|------|----------|
| thread_local! パターン | `ecs/window.rs:32-77` (DpiChangeContext) | キャッシュストレージの実装パターンとして参照 |
| thread_local! パターン | `ecs/window.rs:102-150` (SetWindowPosCommand) | 同上 |
| thread_local! パターン | `ecs/event/mouse.rs` (MouseBuffer) | 同上 |
| hit_test_in_window() | `ecs/layout/hit_test.rs:211-233` | 既存関数。キャッシュミス時に呼び出す |
| try_tick_world() | `ecs/world.rs:508-540` | キャッシュクリアの呼び出しポイント |
| WM_NCHITTEST ハンドラ | `ecs/window_proc/handlers.rs:399-460` | cached_nchittest() への置き換え対象 |

### 1.2 パターン分析: DpiChangeContext

```rust
// window.rs:32-77 - 参照実装パターン
pub struct DpiChangeContext {
    inner: Option<DpiChange>,
}

thread_local! {
    static DPI_CHANGE: RefCell<DpiChangeContext> = RefCell::new(DpiChangeContext::new());
}

impl DpiChangeContext {
    fn new() -> Self { ... }
    pub fn set(dpi_change: DpiChange) { ... }
    pub fn take() -> Option<DpiChange> { ... }
    pub fn is_some() -> bool { ... }
}
```

**適用可能性**: キャッシュ構造体と get/set/clear API に直接適用可能

---

## 2. 既存実装詳細

### 2.1 現在の WM_NCHITTEST ハンドラ

**場所**: `handlers.rs:399-460`

```rust
pub fn handle_nchittest(world: &World, hwnd: HWND, lparam: LPARAM) -> Option<LRESULT> {
    let window = get_window_entity(world, hwnd)?;
    let point = lparam_to_physical_point(lparam);
    let window_pos = world.get::<WindowPos>(window)?;
    let client_point = /* スクリーン座標→クライアント座標変換 */;
    
    match hit_test_in_window(world, window, client_point) {
        Some(_entity) => Some(LRESULT(HTCLIENT as _)),
        None => Some(LRESULT(HTTRANSPARENT as _)),
    }
}
```

**観察**:
- 現在は毎回 hit_test_in_window() を呼び出し
- 戻り値は HTCLIENT または HTTRANSPARENT のみ
- Entity 情報は戻り値には使用されていない（設計Phase 1の方針と一致）

### 2.2 try_tick_world() 実装

**場所**: `world.rs:508-540`

```rust
pub fn try_tick_world() {
    // ... 
    // 各種スケジュール実行（Layout含む）
    // この関数終了時にキャッシュクリアを挿入可能
}
```

**統合ポイント**: 関数終了直前に `clear_nchittest_cache()` を追加

---

## 3. ギャップ一覧

### 3.1 新規実装が必要

| 項目 | 説明 | 要件マッピング |
|-----|------|--------------|
| キャッシュストレージ | thread_local! で座標→LRESULT のマッピング | Req-1 |
| cached_nchittest() | キャッシュ判定API | Req-4 |
| clear_nchittest_cache() | キャッシュクリアAPI | Req-3, Req-4 |
| ハンドラ統合 | handlers.rs の WM_NCHITTEST を cached_nchittest() 呼び出しに変更 | Req-2 |
| tick統合 | try_tick_world() に clear_nchittest_cache() 呼び出し追加 | Req-3 |

### 3.2 変更が必要な既存コード

| ファイル | 変更内容 | 影響範囲 |
|---------|---------|---------|
| `handlers.rs:399-460` | cached_nchittest() への委譲 | 小（関数呼び出し変更のみ） |
| `world.rs:508-540` | clear_nchittest_cache() 呼び出し追加 | 小（1行追加） |

---

## 4. 統合ポイント

### 4.1 コード変更箇所

```
handlers.rs
├── handle_nchittest()
│   └── 変更: hit_test_in_window() → cached_nchittest() 呼び出し

world.rs
├── try_tick_world()
│   └── 追加: clear_nchittest_cache() 呼び出し（関数終了直前）

新規モジュール（設計Phaseで決定）
├── NchittestCache 構造体
├── thread_local! ストレージ
├── cached_nchittest() 関数
└── clear_nchittest_cache() 関数
```

### 4.2 依存関係

```
cached_nchittest()
├── 依存: hit_test_in_window() (キャッシュミス時)
├── 依存: NchittestCache (ストレージ)
└── 呼び出し元: handle_nchittest()

clear_nchittest_cache()
├── 依存: NchittestCache (ストレージ)
└── 呼び出し元: try_tick_world()
```

---

## 5. 要件カバレッジ

| 要件ID | 概要 | カバー状況 | ギャップ |
|-------|------|----------|---------|
| Req-1 | thread_local! キャッシュストレージ | ❌ 未実装 | 新規作成必要 |
| Req-2 | キャッシュヒット判定 | ❌ 未実装 | cached_nchittest()に含める |
| Req-3 | tick/レイアウトでキャッシュクリア | ❌ 未実装 | try_tick_world()への統合必要 |
| Req-4 | API（cached_nchittest, clear） | ❌ 未実装 | 新規作成必要 |

---

## 6. リスク評価

### 6.1 技術的リスク

| リスク | 影響度 | 発生確率 | 軽減策 |
|-------|-------|---------|-------|
| パフォーマンス退行 | 低 | 低 | キャッシュはオーバーヘッド最小の設計 |
| メモリリーク | 低 | 低 | try_tick_world()で確実にクリア |
| マルチスレッド問題 | 低 | 低 | thread_local!で分離 |

### 6.2 設計上の決定事項（Design Phaseで解決）

- キャッシュのデータ構造（HashMap vs Vec vs 単一エントリ）
- 新規モジュールの配置場所
- 座標のキー形式（PhysicalPoint vs (i32, i32)）

---

## 7. 実装推定

### 7.1 工数見積もり

| 項目 | 見積もり |
|-----|---------|
| 新規モジュール作成 | 小（50-100行） |
| 既存コード変更 | 極小（5-10行） |
| テスト | 小（基本ケースのみ） |

### 7.2 実装順序案

1. キャッシュモジュール作成（ストレージ + API）
2. handlers.rs 統合
3. world.rs 統合
4. テスト追加

---

## 8. 結論

**実装可能性**: ✅ 高

**主な所見**:
- thread_local! パターンは既存コードに3つの実装例があり、参照可能
- 既存の hit_test_in_window() をそのまま利用可能
- 統合ポイントが明確（handlers.rs, world.rs）
- 新規実装は小規模（50-100行程度）

**次のステップ**: Design Phase で以下を決定
1. キャッシュのデータ構造選択
2. 新規モジュールの配置場所
3. 具体的なAPI設計
