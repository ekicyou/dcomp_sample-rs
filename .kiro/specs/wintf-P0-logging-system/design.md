# Technical Design: wintf-P0-logging-system

## Overview
wintfライブラリの全`eprintln!`呼び出しを`tracing`クレートによる構造化ログシステムに移行する。

- **Problem**: 現在の`eprintln!`は出力先が固定、フィルタリング不可、コンテキスト情報なし
- **Solution**: `tracing`マクロ（`info!`, `warn!`, `error!`, `debug!`, `trace!`）への置換とアプリケーション側のSubscriber設定
- **Scope**: wintf内の50+箇所のeprintln! + taffy_flex_demo参考実装

## Requirements Addressed

| Requirement | Design Reference |
|-------------|------------------|
| Req 1 (ログシステム選定) | Architecture § 依存関係 |
| Req 2 (ログレベル実装) | Detailed Design § ログレベルマッピング |
| Req 3 (構造化ログ出力) | Detailed Design § マクロ記法 |
| Req 4 (責務分離と参考実装) | Architecture § レイヤー構成, Components § taffy_flex_demo |
| Req 5 (パフォーマンス要件) | Architecture § tracing特性 |

## Architecture

### System Context

```
┌─────────────────────────────────────────────────────┐
│ Application (e.g., taffy_flex_demo)                 │
│ ┌─────────────────────────────────────────────────┐ │
│ │ main()                                          │ │
│ │   Subscriber初期化 (tracing_subscriber::init)  │ │
│ └─────────────────────────────────────────────────┘ │
│                          │                          │
│                          ▼                          │
│ ┌─────────────────────────────────────────────────┐ │
│ │ wintf (library)                                 │ │
│ │   - info!(), warn!(), error!(), debug!()        │ │
│ │   - ログ発行のみ、Subscriber初期化なし          │ │
│ └─────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────┘
                          │
                          ▼
                ┌───────────────────┐
                │ Output            │
                │ (stdout/file等)   │
                └───────────────────┘
```

### Layer Structure

| Layer | Responsibility | tracing Role |
|-------|----------------|--------------|
| Application | Subscriber初期化、フィルタ設定 | `tracing_subscriber::fmt::init()` |
| wintf Library | ログイベント発行 | `info!()`, `warn!()` 等マクロ呼び出し |
| tracing-subscriber | ログ出力整形・フィルタリング | `FmtSubscriber`, `EnvFilter` |

### 依存関係

**ワークスペース Cargo.toml (追加)**:
```toml
[workspace.dependencies]
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter"] }
```

**crates/wintf/Cargo.toml (追加)**:
```toml
[dependencies]
tracing.workspace = true
```

## Components

### Component: wintf ログ発行
- **Responsibility**: 構造化ログイベントを発行、Subscriber依存なし
- **Interface**: `tracing::info!`, `tracing::warn!`, `tracing::error!`, `tracing::debug!`, `tracing::trace!`
- **Behavior**: Subscriberが未設定の場合、イベントはゼロコストで無視される

### Component: taffy_flex_demo 参考実装
- **Responsibility**: Subscriber初期化の実例を提供
- **Interface**: `main()`先頭で`tracing_subscriber::fmt::init()`または`EnvFilter`付き初期化
- **Behavior**:
  - デフォルト: `info`レベル以上を表示
  - `RUST_LOG=debug`で`debug`レベルも表示
  - `RUST_LOG=wintf=trace`でwintf限定のtraceレベル表示

**参考実装コード**:
```rust
use tracing_subscriber::{fmt, EnvFilter};

fn main() {
    // RUST_LOG環境変数でフィルタリング、未設定時はinfoレベル
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    fmt()
        .with_env_filter(filter)
        .init();

    // ... 以下既存コード
}
```

## Detailed Design

### ログレベルマッピング規則

| ログレベル | 用途 | 例 |
|------------|------|-----|
| `error!` | 回復不能エラー、致命的問題 | DirectComposition初期化失敗 |
| `warn!` | 回復可能だが注意が必要 | タイムアウト、フォールバック処理 |
| `info!` | 重要なイベント、状態変化 | ウィンドウ作成/破棄 |
| `debug!` | 開発時有用な詳細情報 | 個々のメッセージ処理 |
| `trace!` | 非常に詳細なトレース | レイアウト計算ステップ |

### ファイル別マッピング表

#### win_thread_mgr.rs (6箇所)
| 行 | 現在 | 変換後 | 理由 |
|----|------|--------|------|
| 67 | `eprintln!("JoinHandle returned None")` | `warn!("JoinHandle returned None")` | 異常だが致命的ではない |
| 90 | `eprintln!("Window already exists")` | `debug!(hwnd = ?hwnd, "Window already exists")` | デバッグ情報 |
| (他4箇所) | 内容に基づき分類 | | |

#### winproc.rs (4箇所)
| 行 | 現在 | 変換後 | 理由 |
|----|------|--------|------|
| (調査後詳細化) | `eprintln!(...)` | `debug!(...)`/`warn!(...)` | |

#### ecs/window_proc.rs (10+箇所)
| パターン | 変換後 | 理由 |
|----------|--------|------|
| メッセージ受信ログ | `trace!(msg = %msg, hwnd = ?hwnd, ...)` | 大量発生、デフォルト非表示 |
| エラーハンドリング | `error!(error = ?e, ...)` | 問題特定に重要 |

#### ecs/window.rs (10+箇所)
| パターン | 変換後 | 理由 |
|----------|--------|------|
| ウィンドウ作成/破棄 | `info!(hwnd = ?hwnd, "Window created/destroyed")` | 重要なライフサイクルイベント |
| プロパティ変更 | `debug!(...)` | デバッグ時のみ必要 |

#### ecs/widget/text/draw_labels.rs (10+箇所)
| パターン | 変換後 | 理由 |
|----------|--------|------|
| レイアウト計算 | `trace!(...)` | 頻繁に発生 |
| フォント関連エラー | `warn!(...)` | 表示に影響するが致命的でない |

#### ecs/widget/shapes/rectangle.rs (3+箇所)
| パターン | 変換後 | 理由 |
|----------|--------|------|
| 描画エラー | `warn!(...)` | |

### マクロ記法標準

```rust
// 基本形式
info!("Window created");

// フィールド付き（Debug実装使用）
info!(hwnd = ?hwnd, entity = ?entity_id, "Window created");

// フィールド付き（Display実装使用）
warn!(message = %msg_name, "Unknown message received");

// エラー付き
error!(error = ?e, "Failed to create window");
```

## Data Design
N/A（ログシステムは状態を持たない）

## Error Handling

| Error Condition | Handling Strategy | Log Level |
|-----------------|-------------------|-----------|
| Subscriber未初期化 | ログ発行はゼロコストで無視される | N/A |
| EnvFilter解析失敗 | デフォルトフィルタ("info")にフォールバック | warn! |

## Testing Strategy

| Test Type | Approach | Coverage |
|-----------|----------|----------|
| Unit Test | 不要（tracingは十分テスト済み） | — |
| Integration Test | taffy_flex_demo手動実行 | ログ出力確認 |
| Manual Test | `RUST_LOG=debug cargo run --example taffy_flex_demo` | フィルタ動作確認 |

### 確認手順
1. `cargo run --example taffy_flex_demo` — デフォルト（infoレベル）
2. `RUST_LOG=debug cargo run --example taffy_flex_demo` — debugレベル
3. `RUST_LOG=wintf=trace cargo run --example taffy_flex_demo` — wintf限定trace

## Performance Considerations

| Concern | Mitigation | Verification |
|---------|------------|--------------|
| メッセージループ負荷 | Subscriber無効時ゼロコスト | tracing公式保証 |
| 文字列フォーマット | フィルタで除外時は実行されない | tracing実装仕様 |
| リリースビルド | EnvFilterでinfo以上のみ出力 | RUST_LOG未設定時デフォルト |

## Security Considerations
N/A（ログシステムはセキュリティ境界を越えない）

## Dependencies

| Dependency | Version | Purpose | Risk |
|------------|---------|---------|------|
| tracing | 0.1 | 構造化ログマクロ | Low（tokio-rs維持、広く採用） |
| tracing-subscriber | 0.3 | Subscriber実装 | Low（同上） |

### bevy_log互換性
将来的にbevy統合が進んだ場合、`bevy_log`は`tracing`を再エクスポートするため、
wintf側のコード変更なしで移行可能。

## Open Questions

| Question | Impact | Resolution Target |
|----------|--------|-------------------|
| 各ファイルの詳細なログレベル分類 | Medium | タスク実装時 |

## Design Review Checklist
- [x] 全要件がDesign Referenceで対応付けられている
- [x] 責務分離が明確（wintf=発行、app=Subscriber）
- [x] パフォーマンス影響が許容範囲
- [x] 依存クレートのリスク評価完了
- [x] テスト戦略が定義されている
