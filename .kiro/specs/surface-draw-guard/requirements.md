# Requirements Document

## Introduction

DirectComposition Surfaceの描画操作（`begin_draw`/`end_draw`）をRAIIパターンでカプセル化するガード構造体を提供する。現在の`DCompositionSurfaceExt::begin_draw()`は`Result<(ID2D1DeviceContext3, POINT)>`を返し、呼び出し側でオフセット適用と`end_draw()`呼び出しを手動で行う必要がある。この新しい構造体により、リソース管理の安全性と利便性を向上させる。

## Project Description (Input)
DCompositionSurfaceExt::begin_draw()が返す型を、Result<(ID2D1DeviceContext3, POINT)>から、ID2D1DeviceContext3にderefする新たな構造体に変えて、構造体の開始時にoffsetを自動適用し、drop時にend_drawするようにしたい。

## Requirements

### Requirement 1: SurfaceDrawGuard構造体の提供
**Objective:** As a ライブラリ利用者, I want DirectComposition Surfaceの描画セッションを管理するガード構造体を取得したい, so that 描画リソースのライフサイクルを安全かつ簡潔に管理できる

#### Acceptance Criteria
1. When `begin_draw()`が呼び出された時, the SurfaceDrawGuard shall `IDCompositionSurface::BeginDraw()`を呼び出し、成功した場合にガード構造体を返す
2. The SurfaceDrawGuard shall `ID2D1DeviceContext3`への参照を内部に保持する
3. The SurfaceDrawGuard shall `IDCompositionSurface`への参照を内部に保持する（`end_draw`呼び出し用）
4. When `begin_draw()`が失敗した時, the SurfaceDrawGuard shall `windows::core::Error`を返す

### Requirement 2: オフセットの自動適用
**Objective:** As a ライブラリ利用者, I want 描画オフセットが自動的に適用されるようにしたい, so that 手動でのオフセット計算ミスを防止できる

#### Acceptance Criteria
1. When SurfaceDrawGuardが生成された時, the SurfaceDrawGuard shall `BeginDraw`から返された`updateOffset`を`ID2D1DeviceContext`の変換行列に自動適用する
2. The SurfaceDrawGuard shall `SetTransform`を使用してオフセットを平行移動として適用する
3. If 描画領域が指定された場合, the SurfaceDrawGuard shall 指定された`RECT`を`BeginDraw`に渡す

### Requirement 3: Deref による DeviceContext アクセス
**Objective:** As a ライブラリ利用者, I want ガード構造体を通じて`ID2D1DeviceContext3`に透過的にアクセスしたい, so that 既存の描画コードを最小限の変更で使用できる

#### Acceptance Criteria
1. The SurfaceDrawGuard shall `Deref<Target = ID2D1DeviceContext3>`を実装する
2. When ガード構造体が参照解決された時, the SurfaceDrawGuard shall 内部の`ID2D1DeviceContext3`への参照を返す
3. The SurfaceDrawGuard shall `DerefMut`を実装し、可変参照によるアクセスも提供する

### Requirement 4: Drop時の自動クリーンアップ
**Objective:** As a ライブラリ利用者, I want ガード構造体のスコープ終了時に自動的に`end_draw()`が呼ばれるようにしたい, so that リソースリークやAPI呼び出し忘れを防止できる

#### Acceptance Criteria
1. When SurfaceDrawGuardがドロップされた時, the SurfaceDrawGuard shall `IDCompositionSurface::EndDraw()`を自動的に呼び出す
2. If `EndDraw()`が失敗した場合, the SurfaceDrawGuard shall エラーをログ出力する（panicしない）
3. The SurfaceDrawGuard shall 描画セッション中に適用したオフセット変換を元に戻す（Identity行列に復元）

### Requirement 5: 既存APIとの互換性
**Objective:** As a ライブラリ利用者, I want 既存の`DCompositionSurfaceExt`トレイトとの互換性を維持したい, so that 段階的な移行が可能になる

#### Acceptance Criteria
1. The DCompositionSurfaceExt shall 新しい`begin_draw_guard()`メソッドを提供する
2. The DCompositionSurfaceExt shall 既存の`begin_draw()`メソッドを非推奨（deprecated）としてマークするが、削除しない
3. The SurfaceDrawGuard shall `crates/wintf/src/com/dcomp.rs`モジュール内に配置する

