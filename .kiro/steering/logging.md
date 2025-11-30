# Logging Guidelines (tracing)

このプロジェクトでは `tracing` クレートを使用した構造化ロギングを採用しています。

## 依存関係

```toml
# ライブラリ (wintf)
tracing = { workspace = true }

# アプリケーション (examples, bins)
tracing-subscriber = { workspace = true }  # env-filter feature有効
```

## ログレベル選択基準

| レベル | 用途 | 例 |
|--------|------|-----|
| `error!` | 致命的エラー、回復不能な失敗 | COM API失敗、リソース作成失敗 |
| `warn!` | 回復可能なエラー、警告、非推奨の使用 | 無効なパラメータ、フォールバック発生 |
| `info!` | ライフサイクルイベント | 初期化完了、終了、ディスプレイ構成変更 |
| `debug!` | 開発者向け詳細情報 | エンティティ作成、コマンド実行、状態変更 |
| `trace!` | 高頻度イベント、詳細トレース | WMメッセージ、フレームごとの処理、描画詳細 |

## 構造化フィールドの規約

よく使用するフィールド名を統一：

```rust
// Entity識別子
debug!(entity = %entity_name, "message");
debug!(entity = ?entity, "message");  // Entity IDのDebug出力

// ウィンドウハンドル（16進数）
trace!(hwnd = format!("0x{:X}", hwnd.0), "message");

// フレーム番号
trace!(frame = frame_count.0, "message");

// エラー詳細
error!(error = ?e, "operation failed");
error!(error = %e, hresult = format!("0x{:08X}", e.code().0), "COM error");

// サイズ・座標
debug!(width = width, height = height, "size");
debug!(x = pos.x, y = pos.y, "position");
```

## 書式パターン

### 関数名プレフィックス

ログメッセージには関数名をプレフィックスとして含める：

```rust
info!("[GraphicsCore] Initialization completed");
debug!(entity = %name, "[init_window_graphics] WindowGraphics created");
trace!(frame = frame_count.0, "[commit_composition] DComp device not available");
```

### 構造化フィールド優先

文字列補間より構造化フィールドを優先：

```rust
// Good: 構造化フィールド
debug!(
    entity = %entity_name,
    width = width,
    height = height,
    "[deferred_surface_creation] Creating Surface"
);

// Avoid: 文字列補間
debug!("[deferred_surface_creation] Creating Surface for Entity={}, size={}x{}", entity_name, width, height);
```

## Subscriber初期化（アプリケーション側）

```rust
use tracing_subscriber::EnvFilter;

fn main() {
    // RUST_LOG環境変数対応、未設定時はinfoレベル
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info"))
        )
        .init();
}
```

## 環境変数によるフィルタリング

```powershell
# infoレベル以上
$env:RUST_LOG="info"; cargo run --example taffy_flex_demo

# debugレベルも表示
$env:RUST_LOG="debug"; cargo run --example taffy_flex_demo

# wintfクレートのみtrace
$env:RUST_LOG="wintf=trace"; cargo run --example taffy_flex_demo

# 特定モジュールのみ
$env:RUST_LOG="wintf::ecs::graphics=debug"; cargo run --example taffy_flex_demo
```

## ライブラリ vs アプリケーション

- **ライブラリ (wintf)**: `tracing`マクロでログを発行するのみ。Subscriber初期化は行わない。
- **アプリケーション**: `tracing-subscriber`を使用してSubscriberを初期化。フィルタリングや出力形式を制御。

これにより、ライブラリ使用時にSubscriber未設定であればログ出力はゼロコストとなる。
