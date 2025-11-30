# Gap Analysis: wintf-P0-logging-system

## 1. 現状調査

### 1.1 既存のログ出力箇所

| ファイル | 箇所数 | 用途 |
|----------|--------|------|
| `win_thread_mgr.rs` | 6 | ウィンドウ作成、メッセージループ、終了処理 |
| `winproc.rs` | 4 (2 commented) | WM_NCCREATE, WM_NCDESTROY, WM_LAST_WINDOW_DESTROYED |
| `process_singleton.rs` | 2 | ウィンドウクラス登録 |
| `ecs/world.rs` | 1 | ECS World操作 |
| `ecs/window_system.rs` | 5 | ウィンドウシステム初期化・操作 |
| `ecs/window_proc.rs` | 10+ | Windowsメッセージ処理（WM_WINDOWPOSCHANGED, WM_DPICHANGED等） |
| `ecs/window.rs` | 10+ | SetWindowPosCommand, ウィンドウ状態管理 |
| `ecs/widget/text/draw_labels.rs` | 10+ | テキスト描画エラー |
| `ecs/widget/shapes/rectangle.rs` | 3+ | 矩形描画エラー |
| **examples/dcomp_demo.rs** | 1 | WM_CREATE |
| **合計** | **50+箇所** | |

### 1.2 ログ内容の分類

| カテゴリ | 例 | 推奨ログレベル |
|----------|-----|----------------|
| 初期化完了 | "Window created", "window classes created" | `info!` |
| 処理開始/終了 | "Window creation...", "Processing N commands" | `debug!` |
| メッセージ処理 | "WM_NCCREATE", "WM_WINDOWPOSCHANGED" | `trace!` |
| エラー/失敗 | "SetWindowPos failed", "GraphicsCore not available" | `warn!` or `error!` |
| 状態変化 | DPI変更、サイズ変更の詳細 | `debug!` |

### 1.3 現在の依存関係

```toml
# crates/wintf/Cargo.toml - tracingは未追加
[dependencies]
bevy_ecs = { workspace = true }
bevy_app = { workspace = true }  # 存在するが未活用
# tracing = なし
# tracing-subscriber = なし（dev-dependenciesにも未追加）
```

### 1.4 taffy_flex_demo.rs の現状

```rust
fn main() -> Result<()> {
    human_panic::setup_panic!();
    let mgr = WinThreadMgr::new()?;
    // ... ログ初期化なし
}
```

---

## 2. 要件実現可能性分析

### Requirement 1: ログシステム選定 (`tracing`)

| AC | 状態 | 備考 |
|----|------|------|
| AC1: tracingクレート採用 | **Missing** | Cargo.toml に依存追加が必要 |
| AC2: tracingマクロ使用 | **Missing** | 50+箇所の`eprintln!`を置換 |
| AC3: Subscriber初期化は利用者 | **OK** | ライブラリに初期化コードを入れないだけ |
| AC4: tracing互換API | **OK** | 標準的なマクロのみ使用予定 |

### Requirement 2: ログレベル対応

| AC | 状態 | 備考 |
|----|------|------|
| AC1: 5段階ログレベル | **OK** | tracing標準機能 |
| AC2: RUST_LOG環境変数 | **OK** | tracing-subscriber標準機能 |
| AC3-4: ビルド別デフォルト | **Research Needed** | tracing-subscriber EnvFilter設定で実現 |

### Requirement 3: 構造化ログ出力

| AC | 状態 | 備考 |
|----|------|------|
| AC1: モジュール名自動付与 | **OK** | tracingが`target`として自動付与 |
| AC2: ファイル名・行番号 | **OK** | tracing-subscriber設定で有効化 |
| AC3: HWND/Entity IDコンテキスト | **Partial** | 手動でフィールド追加必要 |

### Requirement 4: 責務分離と参考実装

| AC | 状態 | 備考 |
|----|------|------|
| AC1: Subscriber初期化しない | **OK** | 設計方針通り |
| AC2: eprintln!置換 | **Missing** | 50+箇所の置換作業 |
| AC3: taffy_flex_demo初期化 | **Missing** | main関数に追加 |
| AC4: RUST_LOG対応 | **OK** | tracing-subscriber標準 |

### Requirement 5: パフォーマンス影響

| AC | 状態 | 備考 |
|----|------|------|
| AC1: 無効レベル最小化 | **OK** | tracingのゼロコスト抽象化 |
| AC2: 非同期出力 | **Research Needed** | tracing-appenderまたはデフォルトfmtで十分か |
| AC3: フレームレート影響なし | **Testing Required** | 実装後の検証 |

---

## 3. 実装アプローチ

### Option A: 最小限の依存追加

**概要**: `tracing`のみ追加、`tracing-subscriber`はdev-dependenciesとexamplesのみ

```toml
# crates/wintf/Cargo.toml
[dependencies]
tracing = "0.1"

[dev-dependencies]
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

**Trade-offs**:
- ✅ ライブラリのコア依存を最小化
- ✅ 利用者が任意のSubscriberを選択可能
- ❌ examples以外でデフォルト出力がない

### Option B: ワークスペース全体で依存管理

**概要**: workspace.dependenciesに`tracing`と`tracing-subscriber`を追加

```toml
# Cargo.toml (workspace)
[workspace.dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

**Trade-offs**:
- ✅ バージョン統一管理
- ✅ 将来のクレート追加時も一貫性
- ❌ わずかに設定が複雑

### 推奨: **Option B**

理由: 既存のworkspace.dependencies管理パターンに合致

---

## 4. eprintln! → tracing マッピング案

| 現在のパターン | 推奨マクロ | 理由 |
|----------------|------------|------|
| 処理開始通知 | `debug!` | 通常運用では不要 |
| 処理完了通知 | `info!` | 重要なマイルストーン |
| WMメッセージ詳細 | `trace!` | 非常に頻繁、デバッグ時のみ |
| リソース未取得 | `warn!` | 異常だが継続可能 |
| API呼び出し失敗 | `error!` | 要調査のエラー |
| コメントアウト済み | 削除または`trace!` | 必要なら復活 |

**例: win_thread_mgr.rs**
```rust
// Before
eprintln!("Window creation...");
// After
debug!("Window creation...");

// Before
eprintln!("Window created {:?}", rc);
// After
info!(hwnd = ?rc, "Window created");
```

---

## 5. 実装複雑度と リスク

| 項目 | 評価 | 根拠 |
|------|------|------|
| **Effort** | **S (1-3日)** | 依存追加＋機械的置換＋例1件 |
| **Risk** | **Low** | 確立された技術、bevy_log互換 |

---

## 6. 設計フェーズへの申し送り

### 確定事項
- `tracing` クレート採用
- Subscriber初期化はアプリケーション責任
- `taffy_flex_demo.rs`に参考実装追加

### 要調査事項
1. **ビルド別デフォルトレベル**: `#[cfg(debug_assertions)]`とEnvFilter組み合わせ
2. **非同期出力の必要性**: メッセージループ内でのブロッキング許容度
3. **HWNDコンテキスト付与**: spanベースかフィールドベースか

### 置換作業の範囲
- **ライブラリ**: 50+箇所（6ファイル）
- **examples**: 1箇所（dcomp_demo.rs）+ taffy_flex_demo.rs初期化追加
