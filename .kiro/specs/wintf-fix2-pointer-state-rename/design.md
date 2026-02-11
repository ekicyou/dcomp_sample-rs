# Design Document: wintf-fix2-pointer-state-rename

## Overview

`PointerState` コンポーネントの `screen_point` フィールドを `client_point` にリネームする。フィールド名が示唆する「スクリーン座標」と実際に保持する「クライアント座標（物理 px）」の不一致を解消し、コードの可読性と保守性を向上させる。

値の計算ロジック・データフロー・型は一切変更しない。純粋なリネームリファクタリング。

### Goals
- `PointerState.screen_point` → `client_point` へのリネーム完了
- 全参照箇所の一貫した更新
- コンパイル成功・テスト全パス

### Non-Goals
- 座標値の変換ロジック変更
- `nchittest_cache.rs` / `hit_test.rs` の `screen_point`（別概念）の修正
- 新しいフィールドやメソッドの追加

## Architecture

### Existing Architecture Analysis

本変更は既存アーキテクチャに構造的変更を加えない。影響範囲はフィールド名のテキスト置換に限定される。

- **現行パターン**: ECS コンポーネント `PointerState` は `pointer/mod.rs` で定義され、`handlers.rs` で初期化、各システム関数でアクセスされる
- **保持すべき統合ポイント**: `PointerState` の公開インターフェース（型・フィールド構成）は `client_point` へのリネーム以外に変更なし
- **技術的負債**: `screen_point` という誤解を招く命名を本仕様で解消する

### Architecture Pattern & Boundary Map

アーキテクチャパターンの変更なし。単一コンポーネントのフィールドリネームのため省略。

### Technology Stack

変更なし。既存の Rust / bevy_ecs / windows-rs スタックに準拠。

## Requirements Traceability

| Requirement | Summary | Components | Flows |
|-------------|---------|------------|-------|
| 1.1 | フィールド名 `screen_point` → `client_point` | PointerState 定義 | — |
| 1.2 | 型 `PhysicalPoint` 維持 | PointerState 定義 | — |
| 1.3 | doc コメント修正 | PointerState 定義 | — |
| 2.1 | handlers.rs 初期化箇所更新（4箇所） | PointerState 初期化 | — |
| 2.2 | Default 実装更新 | PointerState 定義 | — |
| 2.3 | pointer/mod.rs アクセス箇所更新（約12箇所） | PointerState アクセス | — |
| 2.4 | taffy_flex_demo.rs 更新（約10行） | サンプルコード | — |
| 2.5 | ユニットテスト更新（L878） | テストコード | — |
| 3.1 | nchittest_cache.rs 変更しない | — | — |
| 3.2 | hit_test.rs 変更しない | — | — |
| 3.3 | 同名ローカル変数・パラメータ変更しない | — | — |
| 4.1 | doc コメント「クライアント座標（物理ピクセル）」 | PointerState 定義 | — |
| 4.2 | PointerState フィールド参照コメント更新 | コメント | — |
| 4.3 | 非 PointerState コメントは変更しない | — | — |
| 5.1 | cargo build 成功 | 検証 | — |
| 5.2 | cargo test 全パス | 検証 | — |
| 5.3 | 値の不変性 | 検証 | — |

## Components and Interfaces

| Component | Domain/Layer | Intent | Req Coverage | Key Dependencies |
|-----------|-------------|--------|--------------|-----------------|
| PointerState 構造体 | ECS / pointer | フィールド定義・doc・Default | 1.1-1.3, 2.2, 4.1 | — |
| handlers.rs 初期化 | ECS / window_proc | 構造体リテラルのフィールド名 | 2.1 | PointerState (P0) |
| pointer/mod.rs システム関数 | ECS / pointer | フィールドアクセス・tracing ログ | 2.3, 4.2 | PointerState (P0) |
| taffy_flex_demo.rs | examples | サンプルコード内フィールドアクセス | 2.4 | PointerState (P0) |
| ユニットテスト | ECS / pointer | テスト内フィールドアクセス | 2.5 | PointerState (P0) |

### ECS / pointer

#### PointerState 構造体

| Field | Detail |
|-------|--------|
| Intent | フィールド名・doc コメント・Default 実装のリネーム |
| Requirements | 1.1, 1.2, 1.3, 2.2, 4.1 |

**変更内容**

1. フィールド定義:
   - `pub screen_point: PhysicalPoint` → `pub client_point: PhysicalPoint`
   - 型 `PhysicalPoint` は変更しない
2. doc コメント:
   - `/// スクリーン座標（物理ピクセル）` → `/// クライアント座標（物理ピクセル）`
3. Default 実装:
   - `screen_point: PhysicalPoint::default()` → `client_point: PhysicalPoint::default()`

#### pointer/mod.rs システム関数内アクセス

| Field | Detail |
|-------|--------|
| Intent | フィールドアクセス・tracing ログ・コメントの更新 |
| Requirements | 2.3, 4.2 |

**変更内容**

| 行番号 | 変更前 | 変更後 |
|--------|--------|--------|
| L506 | `pointer.screen_point.x` | `pointer.client_point.x` |
| L507 | `pointer.screen_point.y` | `pointer.client_point.y` |
| L508 | `pointer.screen_point = ...` | `pointer.client_point = ...` |
| L509 | `Phase 1ではscreen_pointと同じ` | `Phase 1ではclient_pointと同じ` |
| L510 | `pointer.local_point = pointer.screen_point` | `pointer.local_point = pointer.client_point` |
| L515 | `new_x = pointer.screen_point.x` | `new_x = pointer.client_point.x` |
| L516 | `new_y = pointer.screen_point.y` | `new_y = pointer.client_point.y` |
| L618 | `screen_x = pointer.screen_point.x` | `client_x = pointer.client_point.x` |
| L619 | `screen_y = pointer.screen_point.y` | `client_y = pointer.client_point.y` |
| L878 | `state.screen_point` | `state.client_point` |
| L969 | `pointer_state.screen_point = ...` | `pointer_state.client_point = ...` |
| L971 | `pointer_state.local_point = pointer_state.screen_point` | `pointer_state.local_point = pointer_state.client_point` |

**設計判断**: tracing ログのフィールド名 `screen_x`/`screen_y` → `client_x`/`client_y` に更新する。根拠と代替案は `research.md` に記録済み。

### ECS / window_proc

#### handlers.rs 初期化

| Field | Detail |
|-------|--------|
| Intent | PointerState 構造体リテラルのフィールド名更新 |
| Requirements | 2.1 |

**変更内容**（4箇所）

| 行番号 | 変更 |
|--------|------|
| L673 | `screen_point: crate::ecs::pointer::PhysicalPoint::new(x, y)` → `client_point: ...` |
| L736 | 同上 |
| L918 | `screen_point: PhysicalPoint::new(x, y)` → `client_point: ...` |
| L1258 | 同上 |

### examples

#### taffy_flex_demo.rs

| Field | Detail |
|-------|--------|
| Intent | サンプルコード内 `state.screen_point` アクセスの更新 |
| Requirements | 2.4 |

**変更内容**（10行）: 全て `state.screen_point` → `state.client_point` のテキスト置換

| 行番号 |
|--------|
| L779, L808, L999, L1141, L1188, L1231, L1243, L1288, L1289, L1322 |

## 変更対象外（Requirement 3）

以下のファイルは同名の `screen_point` を含むが、`PointerState` フィールドとは別概念であり変更しない:

| ファイル | 理由 |
|----------|------|
| `nchittest_cache.rs` | WM_NCHITTEST 用の実際のスクリーン座標。型は `(i32, i32)` で `PointerState` と無関係 |
| `hit_test.rs` | ヒットテスト関数のパラメータ名。`PointerState` のフィールドアクセスではない |

## Testing Strategy

### Unit Tests
- `cargo test` で `pointer/mod.rs` 内インラインテスト（L878 含む）の通過を確認
- テスト内のフィールドアクセスが `client_point` に更新されていることを確認

### Build Verification
- `cargo build` が警告なしで成功
- `cargo build --examples` でサンプルコードのコンパイル成功

### Regression
- 値の変更がないことを保証（コードレビューで確認）
- `cargo test` がリネーム前と同じテスト結果であること
