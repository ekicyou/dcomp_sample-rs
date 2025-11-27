# マーカーコンポーネントからChanged検出への移行

## Introduction

本仕様は、bevy_ecsにおけるマーカーコンポーネントの`With<Marker>` + `remove()` パターンを、`Changed<T>` パターンに移行し、アーキタイプ変更のオーバーヘッドを排除するためのリファクタリングを定義する。

### 背景

現在のマーカーコンポーネントパターンでは：
- `commands.entity(entity).insert(MarkerComponent)` → アーキタイプ変更（高コスト）
- `commands.entity(entity).remove::<MarkerComponent>()` → アーキタイプ変更（高コスト）

これが毎フレーム複数エンティティで発生すると、パフォーマンス上の問題となる。

### Changedパターンの利点

1. **アーキタイプ変更の排除**: コンポーネントの値変更のみでフラグが立ち、`insert`/`remove`によるアーキタイプ変更が発生しない
2. **同一スケジュール内での即時伝搬（最重要）**: `insert()`はCommandsキューに積まれるため、同じスケジュール内の後続システムに変更が伝搬しない可能性が高い。一方、`Changed<T>`はシステム実行中に直ちにフラグが立つため、同一スケジュール内の後続システムで即座に検出可能
3. **デバッグ容易性**: フレーム番号や世代番号による状態追跡が可能

### Changedパターンのデメリットと許容性

**デメリット1**: フレームを跨いだ変更通知ができない

- `Changed`フラグは全スケジュールの最後に自動でOFFになる
- マーカー方式では`remove()`を明示的に呼ぶまでマーカーが残るため、次フレーム以降でも検出可能だった

**デメリット2**: コンポーネントの事前登録が必要

- `Changed<T>`はコンポーネントの値変更を検出するため、コンポーネントが事前に存在している必要がある
- マーカー方式では`insert()`で同時に追加・検出できたが、Changedパターンでは不可
- エンティティ生成時にライフサイクルフック等で確実に事前登録する設計が必要

**許容性**: これらのデメリットは実質的に影響しない

- 「フレームを跨いだ変更通知」を必要とするシステムは現状存在しない
- そのような要件は設計上存在すべきではない（各フレームで状態を完結させるのがECSの原則）
- 仮に必要な場合は、世代番号や処理済みフラグで明示的に管理すべき

### 対象マーカーコンポーネント

| マーカー | 用途 | 新コンポーネント | 対象リソース |
|---------|------|------------------|-------------|
| `SurfaceUpdateRequested` | Surface描画更新のリクエスト | `SurfaceGraphicsDirty` | `SurfaceGraphics` |
| `GraphicsNeedsInit` | グラフィックス初期化/再初期化 | `WindowGraphicsDirty` | `WindowGraphics`, `VisualGraphics` |

**命名規則**: `[対象リソース]Dirty` - 「何がダーティか」を明確にする

---

## Requirements

### Requirement 1: SurfaceGraphicsDirty コンポーネント定義

**Objective:** As a システム開発者, I want `SurfaceUpdateRequested`マーカーを`SurfaceGraphicsDirty`コンポーネントに置き換える, so that Surface描画リクエストにおけるアーキタイプ変更を排除できる

#### Acceptance Criteria

1. The wintf shall `SurfaceGraphicsDirty`コンポーネントを`ecs/graphics/components.rs`に定義する
2. The wintf shall `SurfaceGraphicsDirty`にリクエストフレーム番号を保持する`requested_frame: u64`フィールドを持たせる
3. The wintf shall `SurfaceGraphicsDirty`に`Default`トレイトを実装し、初期値として`requested_frame: 0`を設定する
4. When Surface描画がリクエストされた時, the wintf shall `requested_frame`フィールドを現在のフレーム番号で更新する
5. The wintf shall `SurfaceUpdateRequested`マーカーコンポーネントの定義を削除する

---

### Requirement 2: SurfaceGraphicsDirty を使用したシステム変更

**Objective:** As a システム開発者, I want 既存のマーカー検出システムを`Changed<SurfaceGraphicsDirty>`パターンに変更する, so that 描画リクエストの検出がアーキタイプ変更なしで行える

#### Acceptance Criteria

1. When `render_surface`システムがSurface描画を行う時, the wintf shall `With<SurfaceUpdateRequested>`フィルターの代わりに`Changed<SurfaceGraphicsDirty>`フィルターを使用する
2. When Surface描画が完了した時, the wintf shall `remove::<SurfaceUpdateRequested>()`呼び出しを削除する（Changedは自動リセットのため不要）
3. When `mark_dirty_surfaces`システムがSurfaceを汚染マークする時, the wintf shall `insert(SurfaceUpdateRequested)`の代わりに`dirty.requested_frame = current_frame`を実行する
4. When `deferred_surface_creation_system`がSurfaceを作成した時, the wintf shall 描画トリガーとして`SurfaceGraphicsDirty`のフレーム更新を実行する
5. The wintf shall `on_surface_graphics_changed`フックの`SafeInsertSurfaceUpdateRequested`コマンドを`SurfaceGraphicsDirty`の更新に置き換える
6. The wintf shall `SafeInsertSurfaceUpdateRequested`カスタムコマンドを削除する

---

### Requirement 3: WindowGraphicsDirty コンポーネント定義

**Objective:** As a システム開発者, I want `GraphicsNeedsInit`マーカーを`WindowGraphicsDirty`コンポーネントに置き換える, so that グラフィックス初期化リクエストにおけるアーキタイプ変更を排除できる

#### Acceptance Criteria

1. The wintf shall `WindowGraphicsDirty`コンポーネントを`ecs/graphics/components.rs`に定義する
2. The wintf shall `WindowGraphicsDirty`に以下のフィールドを持たせる:
   - `needs_init_generation: u32` - 初期化が必要な世代番号
   - `processed_generation: u32` - 処理済みの世代番号
3. The wintf shall `WindowGraphicsDirty`に`Default`トレイトを実装し、両フィールドを`0`に初期化する
4. The wintf shall `WindowGraphicsDirty`に`request_init()`メソッドを実装し、`needs_init_generation`をインクリメントする
5. The wintf shall `WindowGraphicsDirty`に`needs_init() -> bool`メソッドを実装し、`needs_init_generation != processed_generation`を返す
6. The wintf shall `WindowGraphicsDirty`に`mark_initialized()`メソッドを実装し、`processed_generation = needs_init_generation`を設定する
7. The wintf shall `GraphicsNeedsInit`マーカーコンポーネントの定義を削除する

---

### Requirement 4: WindowGraphicsDirty を使用したシステム変更

**Objective:** As a システム開発者, I want 既存の初期化マーカー検出システムを`Changed<WindowGraphicsDirty>`パターンに変更する, so that グラフィックス初期化リクエストの検出がアーキタイプ変更なしで行える

#### Acceptance Criteria

1. When `init_graphics_core`システムが再初期化をトリガーする時, the wintf shall `insert(GraphicsNeedsInit)`の代わりに`dirty.request_init()`を呼び出す
2. When `init_window_graphics`システムが初期化対象を検索する時, the wintf shall `With<GraphicsNeedsInit>`の代わりに`Changed<WindowGraphicsDirty>`と`dirty.needs_init()`条件を使用する
3. When `init_window_visual`システムが初期化対象を検索する時, the wintf shall `With<GraphicsNeedsInit>`の代わりに`Changed<WindowGraphicsDirty>`と`dirty.needs_init()`条件を使用する
4. When `cleanup_graphics_needs_init`システムが初期化完了を処理する時, the wintf shall `remove::<GraphicsNeedsInit>()`の代わりに`dirty.mark_initialized()`を呼び出す
5. When `cleanup_command_list_on_reinit`システムが再初期化対象を検索する時, the wintf shall `With<GraphicsNeedsInit>`の代わりに適切な条件を使用する
6. When `create_visuals_for_init_marked`システムがVisual作成対象を検索する時, the wintf shall `With<GraphicsNeedsInit>`の代わりに`Changed<WindowGraphicsDirty>`と`dirty.needs_init()`条件を使用する

---

### Requirement 5: コンポーネント初期化の統合

**Objective:** As a システム開発者, I want 新コンポーネントがエンティティ生成時に適切に初期化される, so that `Changed<T>`パターンが正常に機能し既存のエンティティ生成フローが動作する

#### 背景

`Changed<T>`パターンでは、コンポーネントが**事前に存在している**必要がある。後から`insert()`した場合、そのフレームでは`Changed`として検出されるが、意図しないタイミングでの検出やレースコンディションの原因となりうる。関連コンポーネントのライフサイクルフックで確実に事前登録する設計が望ましい。

#### Acceptance Criteria

1. The wintf shall グラフィックス関連の状態追跡コンポーネントを一括で挿入する`GraphicsDirtyBundle`を定義する
2. The wintf shall `GraphicsDirtyBundle`に`SurfaceGraphicsDirty`と`WindowGraphicsDirty`を含める
3. When `HasGraphicsResources`コンポーネントがエンティティに追加される時, the wintf shall `on_add`フックで`GraphicsDirtyBundle`の各コンポーネントを挿入する
4. When `Visual`コンポーネントがエンティティに追加される時, the wintf shall `on_add`フックで`SurfaceGraphicsDirty`を挿入する（Surfaceを持つ可能性のあるエンティティ）
5. While エンティティが`SurfaceGraphicsDirty`を持つ場合, the wintf shall `Changed`検出が初回フレームでトリガーされることを保証する
6. While エンティティが`WindowGraphicsDirty`を持つ場合, the wintf shall 初期状態では`needs_init()`が`false`を返すことを保証する
7. The wintf shall 既存の手動`spawn()`呼び出しで状態追跡コンポーネントの明示的追加を不要にする

---

### Requirement 8: 状態追跡コンポーネントの集約設計

**Objective:** As a システム開発者, I want 複数の状態追跡コンポーネントが一箇所で管理される, so that コンポーネント追加漏れを防ぎメンテナンス性を向上させる

#### 背景

現在のマーカーコンポーネント:
- `HasGraphicsResources` - 静的マーカー（グラフィックスリソースを使用するエンティティ、本仕様のスコープ外）
- `GraphicsNeedsInit` → `WindowGraphicsDirty`（動的Dirty）
- `SurfaceUpdateRequested` → `SurfaceGraphicsDirty`（動的Dirty）

将来追加される可能性のあるDirtyコンポーネントも考慮し、集約管理する設計とする。

#### Acceptance Criteria

1. The wintf shall `ecs/graphics/state_tracking.rs`モジュールを新設し、状態追跡コンポーネントを集約する
2. The wintf shall `ensure_graphics_state_components()`関数を提供し、必要なコンポーネントが存在しない場合のみ挿入する
3. The wintf shall コンポーネント追加フックから`ensure_graphics_state_components()`を呼び出す
4. If 将来新しい状態追跡コンポーネントが追加される場合, then the wintf shall `state_tracking.rs`への追加のみで対応可能とする
5. The wintf shall 状態追跡コンポーネントの一覧と用途をドキュメントコメントで記載する

---

### Requirement 6: テストコードの更新

**Objective:** As a テスト作成者, I want 既存のテストが新しいパターンに対応する, so that テストカバレッジが維持される

#### Acceptance Criteria

1. The wintf shall `surface_optimization_test.rs`の`test_surface_update_requested_component_exists`を`SurfaceGraphicsDirty`用に更新する
2. The wintf shall `surface_optimization_test.rs`の`test_mark_dirty_surfaces_propagation`を新しいパターンに更新する
3. The wintf shall `surface_optimization_test.rs`の`test_surface_update_requested_on_add_hook`を新しいパターンに更新またはフック不要の場合は削除する
4. The wintf shall `WindowGraphicsDirty`の`needs_init()`、`request_init()`、`mark_initialized()`メソッドのユニットテストを追加する
5. If 既存テストがマーカーコンポーネントの存在を検証している場合, then the wintf shall 新コンポーネントの状態検証に置き換える
6. The wintf shall `cargo test --all-targets`で全テストが成功することを保証する

---

### Requirement 7: 公開APIの互換性

**Objective:** As a ライブラリ利用者, I want API変更が明確に文書化される, so that マイグレーションが容易にできる

#### Acceptance Criteria

1. If `SurfaceUpdateRequested`が公開APIとして使用されている場合, then the wintf shall `SurfaceGraphicsDirty`への移行ガイドを提供する
2. The wintf shall 新コンポーネントを`pub`として公開し、`wintf::ecs`モジュールからアクセス可能にする
3. The wintf shall 削除されるコンポーネント名と新しいコンポーネント名のマッピングをドキュメント化する

---

## 参考情報

### 現在の使用箇所（SurfaceUpdateRequested）

| ファイル | 行 | 用途 | パターン |
|---------|-----|------|---------|
| `systems.rs` | 164 | `render_surface` クエリフィルター | `With<SurfaceUpdateRequested>` |
| `systems.rs` | 278 | `render_surface` 処理後削除 | `commands.entity(entity).remove::<SurfaceUpdateRequested>()` |
| `systems.rs` | 842 | `mark_dirty_surfaces` マーカー挿入 | `commands.entity(entity).insert(SurfaceUpdateRequested)` |
| `systems.rs` | 1129 | `deferred_surface_creation_system` 描画トリガー | `commands.entity(entity).insert(SurfaceUpdateRequested)` |
| `components.rs` | 189-196 | `on_surface_graphics_changed` フック | `SafeInsertSurfaceUpdateRequested` Command |

### 現在の使用箇所（GraphicsNeedsInit）

| ファイル | 行 | 用途 | パターン |
|---------|-----|------|---------|
| `systems.rs` | 365 | `init_graphics_core` 再初期化時マーカー挿入 | `commands.entity(entity).insert(GraphicsNeedsInit)` |
| `systems.rs` | 392 | `init_graphics_core` 初期化時マーカー挿入 | `commands.entity(entity).insert(GraphicsNeedsInit)` |
| `systems.rs` | 416 | `init_window_graphics` クエリフィルター | `With<GraphicsNeedsInit>` |
| `systems.rs` | 483 | `init_window_visual` クエリフィルター | `With<GraphicsNeedsInit>` |
| `systems.rs` | 752 | `cleanup_graphics_needs_init` クエリフィルター | `With<GraphicsNeedsInit>` |
| `systems.rs` | 762 | `cleanup_graphics_needs_init` マーカー削除 | `commands.entity(entity).remove::<GraphicsNeedsInit>()` |
| `systems.rs` | 772 | `cleanup_command_list_on_reinit` クエリフィルター | `With<GraphicsNeedsInit>` |
| `visual_manager.rs` | 113 | `create_visuals_for_init_marked` クエリフィルター | `With<GraphicsNeedsInit>` |

### 新コンポーネント設計

#### SurfaceGraphicsDirty
```rust
/// SurfaceGraphicsがダーティ（再描画が必要）
#[derive(Component, Default)]
pub struct SurfaceGraphicsDirty {
    /// 最後に描画をリクエストしたフレーム番号
    pub requested_frame: u64,
}
```

#### WindowGraphicsDirty
```rust
/// WindowGraphics/VisualGraphicsがダーティ（初期化/再初期化が必要）
#[derive(Component, Default)]
pub struct WindowGraphicsDirty {
    /// 初期化が必要な世代番号（0=初期化不要）
    pub needs_init_generation: u32,
    /// 処理済みの世代番号
    pub processed_generation: u32,
}

impl WindowGraphicsDirty {
    /// 初期化をリクエスト（ダーティにする）
    pub fn request_init(&mut self) {
        self.needs_init_generation = self.processed_generation.wrapping_add(1);
    }
    
    /// 初期化が必要か判定
    pub fn needs_init(&self) -> bool {
        self.needs_init_generation != self.processed_generation
    }
    
    /// 初期化完了をマーク（クリーンにする）
    pub fn mark_initialized(&mut self) {
        self.processed_generation = self.needs_init_generation;
    }
}
```

### 期待される効果

1. **同一スケジュール内での即時伝搬（最重要）**: `insert()`はCommandsキューに積まれ適用が遅延するが、`Changed<T>`はシステム実行中に直ちにフラグが立ち、同一スケジュール内の後続システムで即座に検出可能
2. **パフォーマンス向上**: アーキタイプ変更の排除
3. **コード簡素化**: `insert`/`remove`の冗長なコードが削減
4. **デバッグ容易性**: フレーム番号や世代番号による追跡が可能
5. **一貫性**: 全マーカーコンポーネントが同じパターンに統一

