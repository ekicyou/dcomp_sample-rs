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

本仕様書は wintf フレームワークにおける WM_NCHITTEST 応答キャッシュの要件を定義する。`event-mouse-basic` 仕様から派生した関連仕様として、WM_NCHITTEST の高頻度呼び出しに対するパフォーマンス最適化を提供する。

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

**含まれるもの（Phase 1: 最小実装）**:
- WM_NCHITTEST 戻り値のキャッシュ
- 座標ベースのキャッシュ判定（座標一致ならキャッシュ返却）
- World tick 時のキャッシュクリア
- レイアウト更新時のキャッシュクリア

**含まれないもの**:
- Entity 情報のキャッシュ（本仕様では不要）
- hit_test API 本体の最適化（別仕様で対応）
- ヒットテストロジック本体 → `event-hit-test` 仕様で実装済み
- マウスイベント処理 → `event-mouse-basic` 仕様で対応

**将来拡張（Phase 2 以降）**:
- hit_test API 自体のキャッシュ（Entity 情報含む）
- 複数座標のキャッシュ保持
- LRU キャッシュ戦略

### 設計決定

**WM_NCHITTEST 戻り値のみをキャッシュする理由**:

| 観点 | Entity キャッシュ | LRESULT キャッシュ |
|------|-------------------|-------------------|
| 複雑さ | 高（ライフサイクル管理） | 低（値のみ） |
| 実装コスト | 中〜高 | 低 |
| 効果 | WM_NCHITTEST + hit_test | WM_NCHITTEST のみ |

**結論**: 最小実装として WM_NCHITTEST 戻り値（LRESULT）のみをキャッシュ。hit_test の最適化は別仕様で検討。

**World 外キャッシュを採用する理由**:

キャッシュヒット時に World 借用を回避することが主目的のため、スレッドローカルキャッシュを採用。

---

## Requirements

### Requirement 1: スレッドローカルキャッシュ

**Objective:** 開発者として、WM_NCHITTEST 戻り値をキャッシュしたい。それにより高頻度呼び出しに対するパフォーマンスを向上できる。

#### Acceptance Criteria

1. The NCHITTEST Cache System shall スレッドローカル変数でウィンドウごとのキャッシュを管理する
2. The NCHITTEST Cache System shall キャッシュエントリにスクリーン座標（物理ピクセル）を保持する
3. The NCHITTEST Cache System shall キャッシュエントリに WM_NCHITTEST 戻り値（LRESULT）を保持する
4. The NCHITTEST Cache System shall ウィンドウ（HWND）ごとに独立したキャッシュエントリを管理する
5. When キャッシュ確認時, the NCHITTEST Cache System shall World 借用なしでキャッシュの有効性を判定する

#### 実装ノート

- データ構造（HashMap / Vec / 単一エントリ）は設計フェーズで決定
- 1フレーム（約16ms）あたりの WM_NCHITTEST 頻度は最大20回程度（高速マウス移動時）
- ウィンドウ数は通常1-2個であり、線形探索でも十分な性能が期待される

---

### Requirement 2: キャッシュヒット判定

**Objective:** 開発者として、同一座標での WM_NCHITTEST をスキップしたい。それによりWorld借用なしで結果を返せる。

#### Acceptance Criteria

1. When 同一スクリーン座標で WM_NCHITTEST が要求された時, the NCHITTEST Cache System shall キャッシュから戻り値を返す
2. When 座標が異なる場合, the NCHITTEST Cache System shall 実際のヒットテストを実行しキャッシュを更新する
3. The NCHITTEST Cache System shall キャッシュヒット時に World を借用しない

#### キャッシュ判定ロジック

- HWND と座標の組み合わせが一致すればキャッシュヒット
- 座標は物理ピクセル単位で厳密比較

---

### Requirement 3: キャッシュクリア

**Objective:** 開発者として、適切なタイミングでキャッシュを無効化したい。それにより古い結果を返すことを防げる。

#### Acceptance Criteria

1. When World tick が実行された時, the NCHITTEST Cache System shall 全キャッシュをクリアする
2. When レイアウトが更新された時, the NCHITTEST Cache System shall 全キャッシュをクリアする
3. The NCHITTEST Cache System shall `clear_nchittest_cache()` 関数を提供する

#### クリアタイミング

| トリガー | アクション |
|---------|-----------|
| World tick | `clear_nchittest_cache()` 呼び出し |
| レイアウト更新完了 | `clear_nchittest_cache()` 呼び出し |

#### 実装ノート

- `clear_nchittest_cache()` は全ウィンドウのキャッシュをクリア
- 部分クリア（特定HWNDのみ）は Phase 2 で検討

---

### Requirement 4: キャッシュ公開API

**Objective:** 開発者として、キャッシュ付き WM_NCHITTEST 処理を使用したい。それにより透過的にキャッシュ最適化の恩恵を受けられる。

#### Acceptance Criteria

1. The NCHITTEST Cache System shall `cached_nchittest(hwnd, screen_point, world)` 関数を提供する
2. When キャッシュヒット時, the `cached_nchittest` function shall World 借用なしで LRESULT を返す
3. When キャッシュミス時, the `cached_nchittest` function shall 実際のヒットテストを実行しキャッシュを更新する
4. The NCHITTEST Cache System shall 戻り値として `LRESULT` を返す

#### API 概要

- `cached_nchittest(hwnd, screen_point, world) -> LRESULT`
  - キャッシュヒット時: World 借用なしで LRESULT を返す
  - キャッシュミス時: hit_test 実行 → LRESULT 変換 → キャッシュ更新
  - 戻り値: HTCLIENT（ヒットあり）または HTTRANSPARENT（ヒットなし）

---

## Non-Functional Requirements

### NFR-1: パフォーマンス目標

**Objective:** システムとして、WM_NCHITTEST 処理時間を大幅に削減したい。

#### Acceptance Criteria

1. When キャッシュヒット時, the NCHITTEST Cache System shall 0.01ms 以下で結果を返す
2. The NCHITTEST Cache System shall キャッシュ確認のために World を借用しない
3. The NCHITTEST Cache System shall 60fps バジェット（16ms）への影響を最小化する

#### 期待効果

| メトリクス | キャッシュなし | キャッシュあり（ヒット時） |
|-----------|--------------|---------------------------|
| WM_NCHITTEST 処理時間 | 0.1-1.0ms | 0.001ms |
| World 借用 | 毎回 | キャッシュミス時のみ |

### NFR-2: スレッドセーフティ

**Objective:** システムとして、メインスレッドでの安全な動作を保証したい。

#### Acceptance Criteria

1. The NCHITTEST Cache System shall メインスレッド専用として設計する
2. The NCHITTEST Cache System shall `thread_local!` マクロを使用してスレッドローカルストレージを実装する

---

## Glossary

| 用語 | 定義 |
|------|------|
| WM_NCHITTEST | Win32 メッセージ。マウス位置がウィンドウのどの部分にあるかを問い合わせる |
| HTCLIENT | WM_NCHITTEST 戻り値。クライアント領域内を示す |
| HTTRANSPARENT | WM_NCHITTEST 戻り値。透過領域（ヒットなし）を示す |
| LRESULT | Win32 メッセージ処理の戻り値型 |
| キャッシュヒット | 前回と同一座標での WM_NCHITTEST 要求。キャッシュから結果を返す |
| キャッシュミス | 座標が変化した WM_NCHITTEST 要求。実際のヒットテストを実行 |
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
cached_nchittest(hwnd, screen_point, world) 呼び出し
    │
    ▼
┌─────────────────────────────────────┐
│ キャッシュ有効性チェック              │
│ (World 借用なし)                     │
│                                     │
│ 座標一致?                           │
└─────────────────────────────────────┘
    │
    ├── Yes (キャッシュヒット)
    │       │
    │       ▼
    │   キャッシュから LRESULT を返す
    │   (World 借用なし)
    │
    └── No (キャッシュミス)
            │
            ▼
        World を借用
            │
            ▼
        hit_test 実行
            │
            ▼
        結果を LRESULT に変換
        (ヒットあり → HTCLIENT, なし → HTTRANSPARENT)
            │
            ▼
        キャッシュ更新
            │
            ▼
        LRESULT を返す

---

World tick / レイアウト更新
    │
    ▼
clear_nchittest_cache() 呼び出し
    │
    ▼
全キャッシュクリア
```

---

## Appendix C: 将来拡張

本仕様は WM_NCHITTEST 戻り値のみをキャッシュする最小実装（Phase 1）である。

| 将来仕様 | 説明 |
|---------|------|
| hit_test キャッシュ | Entity 情報を含むヒットテスト結果のキャッシュ（別仕様） |
| 複数座標キャッシュ | 直近N件の座標をキャッシュ保持 |
| LRU 戦略 | 最も古いエントリを自動削除 |

````