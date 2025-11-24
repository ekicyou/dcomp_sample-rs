# Gap Analysis: virtual-desktop-monitor-hierarchy

## 分析概要

本機能は、VirtualDesktop → Monitor → Window → Widget の4層階層構造を導入し、Taffyレイアウトエンジンによる統一的なレイアウト計算を実現します。既存のコードベースは強固なレイアウト基盤（Taffy統合、Arrangement伝播システム、Common Infrastructure）を持っており、**拡張アプローチ**が最適です。

### ⚠️ 重要な設計検討事項

#### 検討事項A: Monitorの階層位置
**問題提起**: MonitorはVirtualDesktopの子として階層に含めるべきか、それともWindowと同じ階層に配置すべきか？

**現在の想定**: `VirtualDesktop → Monitor → Window → Widget` (4層階層)

**代替案**: `VirtualDesktop → {Monitor, Window} → Widget` (MonitorとWindowが同階層)

**代替案の利点**:
- Monitorは全画面Windowと概念的に等価
- MonitorとWindowを対等に扱うことで、レイアウト計算がシンプルになる可能性
- Window移動時の親子関係変更が不要（Monitorは参照情報のみ）

**現在の想定の利点**:
- `MonitorFromWindow` APIの結果を直接親子関係にマッピング可能
- 物理的なモニター配置とWindow配置の対応が明確
- モニター削除時のWindow再配置ロジックが自然

**影響範囲**:
- Section 2.1 技術要件マッピング (Requirement 2, 4)
- Section 3 実装アプローチ選択 (Option C Phase 1-3)
- Section 4.3 モニター構成変更時の再配置戦略

**この検討事項は設計フェーズで詳細化が必要**

---

### 主要な発見
- ✅ **既存の強固な基盤**: `TaffyLayoutResource`, `Arrangement/GlobalArrangement`, `tree_system.rs`の汎用伝播システムが実装済み
- ✅ **モニタAPI統合**: `MonitorFromWindow`, `GetDpiForMonitor`が既に使用されている
- ⚠️ **名称変更の未完了**: `BoxStyle`/`BoxComputedLayout`の名称がまだ残存している（要件3の対象）
- ⚠️ **モニタ列挙の欠如**: `EnumDisplayMonitors`を使った全モニタ列挙システムが未実装
- ⚠️ **WM_DISPLAYCHANGE対応**: メッセージハンドラは存在するが、モニタ情報更新ロジックが未実装

### 推奨アプローチ
**Option C: Hybrid Approach** - 既存システムを拡張しつつ、新しいVirtualDesktop/Monitorコンポーネントを追加

---

## 1. Current State Investigation

### 1.1 既存アセットとアーキテクチャ

#### レイアウトシステム (`crates/wintf/src/ecs/layout/`)

**taffy.rs** - Taffy統合の中核
```rust
pub struct TaffyLayoutResource {
    tree: TaffyTree<()>,
    entity_to_node: HashMap<Entity, NodeId>,
    node_to_entity: HashMap<NodeId, Entity>,
    first_layout_done: bool,
}

#[derive(Component)]
pub struct TaffyStyle(pub(crate) Style);  // ❌ 要件3: BoxStyleから名称変更が必要

#[derive(Component)]
pub struct TaffyComputedLayout(pub(crate) Layout);  // ❌ 要件3: BoxComputedLayoutから名称変更が必要
```

**arrangement.rs** - 配置計算の基盤
```rust
#[derive(Component)]
#[component(on_add = on_arrangement_add)]
pub struct Arrangement {
    pub offset: Offset,
    pub scale: LayoutScale,
    pub size: Size,
}

#[derive(Component)]
pub struct GlobalArrangement {
    pub transform: Matrix3x2,
    pub bounds: D2DRect,  // Surface生成最適化に使用
}

#[derive(Component)]
pub struct ArrangementTreeChanged;  // ダーティマーカー
```

**systems.rs** - レイアウト伝播システム
```rust
// Common Infrastructure活用の配置伝播
pub fn sync_simple_arrangements(...)
pub fn mark_dirty_arrangement_trees(...)
pub fn propagate_global_arrangements(...)

// Taffyレイアウトシステム
pub fn build_taffy_styles_system(...)  // 高レベルコンポーネント→TaffyStyle変換
pub fn sync_taffy_tree_system(...)     // ECS階層→Taffyツリー同期
pub fn compute_taffy_layout_system(...)  // Taffyレイアウト計算実行
pub fn update_arrangements_system(...)   // TaffyComputedLayout→Arrangement変換
```

#### Common Infrastructure (`crates/wintf/src/ecs/common/tree_system.rs`)

**汎用階層伝播システム** - 完全ジェネリック化
```rust
pub fn sync_simple_transforms<L, G, M>(...)  // ルートエンティティの更新
pub fn mark_dirty_trees<L, G, M>(...)        // ダーティビット伝播
pub fn propagate_parent_transforms<L, G, M>(...)  // 並列階層伝播

where
    L: Component + Copy + Into<G>,  // Arrangement
    G: Component + Copy + PartialEq + Mul<L, Output = G>,  // GlobalArrangement
    M: Component,  // ArrangementTreeChanged
```

✅ **利点**: Arrangementと同じパターンでVirtualDesktop/Monitorの伝播システムを構築可能

#### ウィンドウ管理 (`crates/wintf/src/ecs/window_system.rs`)

**既存のモニタAPI使用例**
```rust
pub fn create_windows(...) {
    // ✅ 既に実装済み
    let monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST) };
    let mut x_dpi = 0u32;
    let mut y_dpi = 0u32;
    let dpi_result = unsafe { 
        GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut x_dpi, &mut y_dpi) 
    };
    
    let initial_dpi = if dpi_result.is_ok() {
        x_dpi as f32
    } else {
        96.0  // デフォルト
    };
}
```

#### メッセージハンドリング (`crates/wintf/src/win_message_handler.rs`)

**WM_DISPLAYCHANGE ハンドラ**
```rust
fn WM_DISPLAYCHANGE(&mut self, wparam: WPARAM, lparam: LPARAM) -> Option<LRESULT> {
    // ❌ 現状: 空実装
    // ✅ 要件7: モニタ情報更新ロジックの追加が必要
    None
}
```

#### ECSスケジュール (`crates/wintf/src/ecs/world.rs`)

**既存のレイアウトシステムスケジュール**
```rust
// Layoutスケジュール
Schedule::default()
    .add_systems((
        crate::ecs::layout::build_taffy_styles_system,
        crate::ecs::layout::sync_taffy_tree_system
            .after(crate::ecs::layout::build_taffy_styles_system),
        crate::ecs::layout::compute_taffy_layout_system
            .after(crate::ecs::layout::sync_taffy_tree_system),
        crate::ecs::layout::update_arrangements_system
            .after(crate::ecs::layout::compute_taffy_layout_system),
    ))

// PostLayoutスケジュール
Schedule::default()
    .add_systems((
        crate::ecs::layout::sync_simple_arrangements,
        crate::ecs::layout::mark_dirty_arrangement_trees
            .after(crate::ecs::layout::sync_simple_arrangements),
        crate::ecs::layout::propagate_global_arrangements
            .after(crate::ecs::layout::mark_dirty_arrangement_trees),
        crate::ecs::window_system::update_window_pos_system
            .after(crate::ecs::layout::propagate_global_arrangements),
    ))
```

✅ **統合ポイント明確**: 新しいシステムを既存スケジュールに挿入可能

### 1.2 命名規則とコード規約

**コンポーネント命名**:
- ローカル: `Arrangement`, `TaffyStyle`
- グローバル/累積: `GlobalArrangement`, `TaffyComputedLayout`
- マーカー: `ArrangementTreeChanged`, `LayoutDirty`

**システム命名**:
- `sync_*`: ルートエンティティ更新
- `mark_dirty_*`: ダーティビット伝播
- `propagate_*`: 階層伝播
- `update_*`: コンポーネント変換
- `build_*`: 構築処理
- `compute_*`: 計算実行

**ファイル配置**:
- コンポーネント: `crates/wintf/src/ecs/layout/`
- システム: `crates/wintf/src/ecs/layout/systems.rs`
- テスト: `crates/wintf/tests/`

### 1.3 テストインフラ

**既存テスト** (`crates/wintf/tests/`):
- ✅ `taffy_layout_integration_test.rs` - Taffy統合テスト
- ✅ `arrangement_bounds_test.rs` - 配置計算テスト
- ✅ `hierarchical_bounds_test.rs` - 階層バウンディングボックステスト
- ✅ `surface_optimization_test.rs` - Surface最適化テスト

**テストパターン**:
```rust
#[test]
fn test_name() {
    let mut world = World::new();
    world.insert_resource(TaffyLayoutResource::default());
    
    // エンティティ生成
    let entity = world.spawn((
        TaffyStyle::default(),
        TaffyComputedLayout::default(),
    )).id();
    
    // システム実行
    // 検証
}
```

---

## 2. Requirements Feasibility Analysis

### 2.1 技術要件マッピング

| 要件 | 必要な実装 | 既存アセット | ギャップ |
|------|-----------|-------------|---------|
| **Req 1: コンポーネント定義** | VirtualDesktop, Monitor, MonitorInfo | Arrangement, GlobalArrangement (パターン) | **Missing**: 新コンポーネント定義 |
| **Req 2: 階層構築** | Parent/Children, エンティティ生成システム | bevy_ecs::hierarchy, window_system.rs (パターン) | **Missing**: VirtualDesktop/Monitor生成ロジック |
| **Req 3: 名称変更** | BoxStyle→TaffyStyle, BoxComputedLayout→TaffyComputedLayout | taffy.rs, systems.rs | **Missing**: 完全な名称変更 (一部完了) |
| **Req 4: Taffyツリー構築** | VirtualDesktop/MonitorノードをTaffyツリーに追加 | TaffyLayoutResource, sync_taffy_tree_system | **Extend**: 既存ツリー構築ロジックを拡張 |
| **Req 5: レイアウト計算** | VirtualDesktopをルートとした計算 | compute_taffy_layout_system | **Extend**: ルートノード選択ロジックを変更 |
| **Req 6: 増分更新** | LayoutDirtyマーカー、変更検知 | ArrangementTreeChanged (パターン) | **Missing**: LayoutDirty定義と検知ロジック |
| **Req 7: モニタ情報更新** | EnumDisplayMonitors, WM_DISPLAYCHANGE対応 | MonitorFromWindow (パターン), WM_DISPLAYCHANGEハンドラ (空実装) | **Missing**: モニタ列挙/更新ロジック |
| **Req 8: スケジュール統合** | システム依存関係定義 | world.rs (既存スケジュール) | **Extend**: 新システムを既存スケジュールに追加 |
| **Req 9: 互換性維持** | 既存テストパス、段階的移行 | 全既存システム | **Constraint**: 破壊的変更の回避 |
| **Req 10: テスト** | 階層構築/レイアウト計算/増分更新テスト | taffy_layout_integration_test.rs (パターン) | **Missing**: 新機能用テストケース |

### 2.2 技術的課題と制約

#### 2.2.1 Taffyツリーのルート変更

**現状**: `sync_taffy_tree_system`は階層のルートを自動検出
```rust
// 既存ロジック: Parent<T>を持たないエンティティをルートとみなす
query: Query<Entity, (With<TaffyStyle>, Without<ChildOf>)>
```

**新要件**: VirtualDesktopを常にルートとする
```rust
// 必要な変更: VirtualDesktopエンティティを明示的にルート指定
query: Query<Entity, (With<VirtualDesktop>, With<TaffyStyle>)>
```

**制約**: 既存のWidget階層が引き続き機能する必要がある（要件9）

#### 2.2.2 モニタ物理情報のTaffyStyle変換

**必要な処理**: Monitor.bounds → TaffyStyle.size/inset
```rust
// Monitorコンポーネント
pub struct Monitor {
    pub hmonitor: HMONITOR,
    pub bounds: MonitorBounds,  // x, y, width, height
    pub dpi: u32,
}

// TaffyStyleへの変換 (要件4.3)
TaffyStyle::new(Style {
    position: Position::Absolute,
    size: Size {
        width: Dimension::Length(monitor.bounds.width as f32),
        height: Dimension::Length(monitor.bounds.height as f32),
    },
    inset: Rect {
        left: LengthPercentage::Length(monitor.bounds.x as f32),
        top: LengthPercentage::Length(monitor.bounds.y as f32),
        ..Default::default()
    },
    ..Default::default()
})
```

**既存パターン**: `build_taffy_styles_system`が高レベルコンポーネント→TaffyStyle変換を実装済み
✅ **拡張可能**: Monitorコンポーネント用の変換ロジックを追加

#### 2.2.3 階層整合性の保証

**課題**: ECS階層とTaffyツリーの同期
- VirtualDesktop → Monitor → Window → Widget
- Parent/Children関係の自動維持
- エンティティ削除時のクリーンアップ（要件2.5）

**既存ソリューション**: `sync_taffy_tree_system`が既にECS階層→Taffyツリー同期を実装
```rust
// 既存の同期ロジック (systems.rs)
pub fn sync_taffy_tree_system(
    mut taffy_res: ResMut<TaffyLayoutResource>,
    // 新規エンティティ検出
    new_entities: Query<Entity, (With<TaffyStyle>, Without<TaffyComputedLayout>)>,
    // 階層変更検出
    changed_hierarchy: Query<(Entity, &Children), Changed<Children>>,
    // 削除検出
    mut removed: RemovedComponents<TaffyStyle>,
) { /* ... */ }
```

✅ **拡張可能**: VirtualDesktop/Monitorを同じロジックで処理可能

#### 2.2.4 WM_DISPLAYCHANGE処理のタイミング

**Windows APIの制約**:
- `WM_DISPLAYCHANGE`はウィンドウプロシージャで受信
- ECSシステムは別スレッドで実行

**必要な設計**:
1. メッセージハンドラでイベントをキューに追加
2. ECSシステムでキューを消費し、モニタ情報を更新

**Research Needed**: イベントキューの実装方法（既存パターンの調査）

### 2.3 外部依存とAPI

**Windows API** (既に使用中):
- ✅ `MonitorFromWindow` - ウィンドウ→モニタ取得
- ✅ `GetDpiForMonitor` - モニタDPI取得
- ✅ `GetMonitorInfoW` - モニタ情報取得
- ❌ `EnumDisplayMonitors` - 全モニタ列挙 (未使用)

**Taffy** (v0.9.1):
- ✅ `TaffyTree` - レイアウトツリー
- ✅ `Style` - レイアウトスタイル
- ✅ `Layout` - 計算結果
- ✅ `NodeId` - ノード識別子

**bevy_ecs** (v0.17.2):
- ✅ `Parent<T>`, `Children<T>` - 階層関係 (bevy_ecs 0.15+ 非推奨、`ChildOf`推奨)
- ⚠️ **注意**: 現在のコードは`ChildOf`を使用 (steering/structure.mdと一致)

---

## 3. Implementation Approach Options

### Option A: Extend Existing Components ❌ 不適切

**アプローチ**: WindowやArrangementに仮想デスクトップ/モニタ情報を追加

**理由**:
- VirtualDesktop/Monitorは独立した概念であり、Windowとは責務が異なる
- 階層レベルが異なる（VirtualDesktop > Monitor > Window）
- 既存コンポーネントが肥大化し、単一責任原則に違反

**評価**: ❌ **推奨しない**

---

### Option B: Create New Components ⚠️ 部分的に適切

**アプローチ**: VirtualDesktop, Monitor, MonitorInfo を完全に新規作成

**実装内容**:
1. 新規コンポーネント定義
```rust
// crates/wintf/src/ecs/layout/desktop.rs (新規ファイル)
#[derive(Component)]
pub struct VirtualDesktop {
    pub name: String,
    pub is_active: bool,
}

#[derive(Component)]
pub struct Monitor {
    pub hmonitor: HMONITOR,
    pub bounds: MonitorBounds,
    pub work_area: MonitorBounds,
    pub dpi: u32,
    pub is_primary: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct MonitorBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}
```

2. 新規システム
```rust
// crates/wintf/src/ecs/layout/desktop_system.rs (新規ファイル)
pub fn enumerate_monitors_system(...)
pub fn update_monitor_info_system(...)
pub fn handle_display_change_system(...)
```

**統合ポイント**:
- 既存の`TaffyLayoutResource`を使用
- 既存の`sync_taffy_tree_system`を拡張
- `world.rs`のスケジュールに新システムを追加

**Trade-offs**:
- ✅ 明確な責務分離
- ✅ 既存コードへの影響を最小化
- ✅ テストが容易
- ❌ ファイル数が増加（desktop.rs, desktop_system.rs）
- ⚠️ 既存のレイアウトシステムとの統合に注意が必要

**評価**: ⚠️ **部分的に推奨** - Hybrid Approachと組み合わせるべき

---

### Option C: Hybrid Approach ✅ 推奨

**アプローチ**: 新規コンポーネント作成 + 既存システムの段階的拡張

#### Phase 1: コンポーネント定義と基礎システム (要件1, 2, 3)

**新規作成**:
```rust
// crates/wintf/src/ecs/layout/desktop.rs
pub struct VirtualDesktop { ... }
pub struct Monitor { ... }
pub struct MonitorBounds { ... }

// crates/wintf/src/ecs/layout/desktop_system.rs
pub fn enumerate_monitors_system(...) { /* EnumDisplayMonitors使用 */ }
pub fn init_virtual_desktop_system(...) { /* VirtualDesktop生成 */ }
```

**既存ファイル拡張**:
```rust
// crates/wintf/src/ecs/layout/taffy.rs
// ✅ 名称はすでにTaffyStyle/TaffyComputedLayout
// ❌ BoxStyle/BoxComputedLayoutの名称が残っている箇所を全置換

// crates/wintf/src/ecs/layout/mod.rs
pub mod desktop;  // 追加
pub mod desktop_system;  // 追加
pub use desktop::*;
pub use desktop_system::*;
```

#### Phase 2: Taffyツリー統合 (要件4, 5)

**既存システム拡張**:
```rust
// crates/wintf/src/ecs/layout/systems.rs (拡張)

// 新規: VirtualDesktopスタイル更新
pub fn update_virtual_desktop_style_system(
    mut query: Query<(&VirtualDesktop, &mut TaffyStyle), Changed<VirtualDesktop>>,
) {
    for (desktop, mut style) in query.iter_mut() {
        style.0 = Style {
            display: Display::Flex,
            flex_direction: FlexDirection::Row,
            ..Default::default()
        };
    }
}

// 新規: Monitorスタイル更新
pub fn update_monitor_style_system(
    mut query: Query<(&Monitor, &mut TaffyStyle), Changed<Monitor>>,
) {
    for (monitor, mut style) in query.iter_mut() {
        // 要件4.3: 物理サイズと座標をTaffyStyleに反映
        style.0 = Style {
            position: Position::Absolute,
            size: Size {
                width: Dimension::Length(monitor.bounds.width as f32),
                height: Dimension::Length(monitor.bounds.height as f32),
            },
            inset: Rect {
                left: LengthPercentage::Length(monitor.bounds.x as f32),
                top: LengthPercentage::Length(monitor.bounds.y as f32),
                ..Default::default()
            },
            ..Default::default()
        };
    }
}

// 拡張: ルートノード選択ロジック変更
pub fn compute_taffy_layout_system(
    mut taffy_res: ResMut<TaffyLayoutResource>,
    // 変更: VirtualDesktopをルートとして明示
    virtual_desktops: Query<Entity, With<VirtualDesktop>>,
) {
    for desktop_entity in virtual_desktops.iter() {
        if let Some(node_id) = taffy_res.get_node(desktop_entity) {
            let available_space = Size {
                width: AvailableSpace::MaxContent,
                height: AvailableSpace::MaxContent,
            };
            let _ = taffy_res.taffy_mut().compute_layout(node_id, available_space);
        }
    }
}
```

**既存システム互換性** (要件9):
```rust
// sync_taffy_tree_systemは変更不要
// - VirtualDesktop/MonitorもTaffyStyleを持つため、自動的に処理される
// - 既存のWindow/Widget階層も引き続き機能
```

#### Phase 3: 増分更新とモニタ変更対応 (要件6, 7)

**新規コンポーネント**:
```rust
// crates/wintf/src/ecs/layout/desktop.rs (追加)
#[derive(Component)]
pub struct LayoutDirty {
    pub subtree_dirty: bool,
}
```

**新規システム**:
```rust
// crates/wintf/src/ecs/layout/desktop_system.rs (追加)

pub fn mark_layout_dirty_system(
    mut changed_monitors: Query<&mut LayoutDirty, Changed<Monitor>>,
    mut changed_styles: Query<&mut LayoutDirty, Changed<TaffyStyle>>,
) {
    // 要件6.2, 6.3: 変更検知とマーカー付与
    for mut dirty in changed_monitors.iter_mut() {
        dirty.subtree_dirty = true;
    }
    for mut dirty in changed_styles.iter_mut() {
        dirty.subtree_dirty = false;
    }
}

pub fn handle_display_change_system(
    mut app: ResMut<App>,
    mut commands: Commands,
    monitors: Query<(Entity, &Monitor)>,
) {
    // Phase 3実装: App resourceフラグベースのイベント検知
    if !app.display_configuration_changed {
        return;
    }
    
    // 要件7.3: 全モニター情報を再取得 (EnumDisplayMonitors)
    // 要件7.4, 7.5: モニター追加/削除の検出とエンティティ更新
    // 要件7.6: LayoutDirtyマーカーを付与
    
    // 要件7.7: フラグをリセット
    app.display_configuration_changed = false;
}
```

**既存ファイル拡張**:
```rust
// crates/wintf/src/win_message_handler.rs (拡張)
fn WM_DISPLAYCHANGE(&mut self, _wparam: WPARAM, _lparam: LPARAM) -> Option<LRESULT> {
    // Phase 3実装: App resourceフラグを設定
    if let Ok(mut app_guard) = self.app.lock() {
        app_guard.set_display_configuration_changed();
    }
    None
}
```

#### Phase 4: スケジュール統合とテスト (要件8, 10)

**既存ファイル拡張**:
```rust
// crates/wintf/src/ecs/world.rs (拡張)

// Layoutスケジュール
Schedule::default()
    .add_systems((
        // 新規追加 (要件8.1)
        crate::ecs::layout::update_virtual_desktop_style_system,
        crate::ecs::layout::update_monitor_style_system
            .after(crate::ecs::layout::update_virtual_desktop_style_system),
        
        // 既存システム
        crate::ecs::layout::build_taffy_styles_system
            .after(crate::ecs::layout::update_monitor_style_system),
        crate::ecs::layout::sync_taffy_tree_system
            .after(crate::ecs::layout::build_taffy_styles_system),
        crate::ecs::layout::compute_taffy_layout_system  // 拡張済み
            .after(crate::ecs::layout::sync_taffy_tree_system),
        crate::ecs::layout::update_arrangements_system
            .after(crate::ecs::layout::compute_taffy_layout_system),
    ))

// Initスケジュール (新規追加)
Schedule::default()
    .add_systems((
        crate::ecs::layout::init_virtual_desktop_system,
        crate::ecs::layout::enumerate_monitors_system
            .after(crate::ecs::layout::init_virtual_desktop_system),
    ))
```

**新規テストファイル**:
```rust
// crates/wintf/tests/virtual_desktop_hierarchy_test.rs (新規)

#[test]
fn test_virtual_desktop_monitor_window_hierarchy() {
    // 要件10.1: 階層構築の検証
}

#[test]
fn test_monitor_style_conversion() {
    // 要件10.2: Monitor→TaffyStyle変換の検証
}

#[test]
fn test_layout_dirty_incremental_update() {
    // 要件10.4: 増分更新の検証
}
```

#### Hybrid Approachの統合戦略

**Phase 1-2**: 破壊的変更なし
- VirtualDesktop/Monitorが存在しない場合、既存動作を維持（要件9.2）
- 名称変更（BoxStyle→TaffyStyle）は全ファイルで一括実行

**Phase 3-4**: 段階的有効化
- `VirtualDesktop`エンティティが生成された場合のみ、新しい階層を使用
- 既存テストは引き続きパス（要件9.3）

**Trade-offs**:
- ✅ 段階的移行が可能
- ✅ 既存システムとの互換性を保持
- ✅ 各フェーズでテスト・検証可能
- ✅ 既存パターン（Arrangement, Common Infrastructure）を最大限活用
- ⚠️ Phase 3のイベントキュー実装に調査が必要
- ⚠️ 複数フェーズにわたる実装（ただし、各フェーズは独立して機能）

**評価**: ✅ **強く推奨** - リスクを最小化し、既存基盤を最大限活用

---

## 4. Research Items for Design Phase

### 4.1 イベントキュー実装パターン

**結論**: `App` resourceパターンを活用し、`display_configuration_changed: bool` フラグで実装

**選択理由**:
- ✅ 既存パターンとの一貫性: `app.rs`で`window_count`トラッキング + `PostMessageW(WM_LAST_WINDOW_DESTROYED)`パターンが確立済み
- ✅ シンプルで明確: フラグベースの状態管理は軽量かつデバッグが容易
- ✅ bevy_ecsイベント不要: `bevy_ecs::event::Events`を導入せずに済む（依存関係を最小化）

**実装詳細**:
1. `App` resourceに`display_configuration_changed: bool`フィールドを追加 (`app.rs`)
2. `WM_DISPLAYCHANGE`ハンドラーが`App::set_display_configuration_changed()`を呼び出し (`win_message_handler.rs`)
3. `detect_display_change_system`が毎フレーム`App`リソースのフラグを監視 (`monitor_system.rs`)
4. フラグが`true`の場合、`EnumDisplayMonitors`を再実行し、モニター情報を更新
5. 処理完了後にフラグを`false`にリセット

**参考パターン**:
- `crates/wintf/src/ecs/app.rs:65-90` - `on_window_destroyed` + `PostMessageW` + `WM_LAST_WINDOW_DESTROYED`
- `crates/wintf/src/ecs/window.rs:88` - `DpiTransform::from_WM_DPICHANGED` (wparam抽出パターン)

**Trade-offs**:
- ✅ 既存コードとの整合性が高い
- ✅ スレッドセーフ（`Arc<Mutex<App>>`経由でアクセス）
- ❌ 毎フレームフラグチェックのオーバーヘッド（ただしboolチェックは軽量）

### 4.2 bevy_ecsの階層API最新動向

**目的**: `Parent<T>`/`Children<T>` vs `ChildOf`の選択

**現状**:
- steeringドキュメントでは`ChildOf`を推奨
- 既存コードは主に`ChildOf`を使用

**調査項目**:
- bevy_ecs 0.17.2での推奨API
- `ChildOf`の型パラメータ設計（`ChildOf<Monitor>` vs `ChildOf`）

### 4.3 モニタ構成変更時のウィンドウ再配置戦略

**目的**: モニタ削除時のWindow移動ロジック

**調査項目**:
- モニタが削除された場合、そのモニタ上のWindowをどこに移動させるか
- プライマリモニタへの自動移動
- ユーザー設定の保存/復元

**参考**: Windows OS標準の動作を調査

---

## 5. Implementation Complexity & Risk Assessment

### 5.1 Effort Estimation

| フェーズ | 作業内容 | 推定工数 | 理由 |
|---------|---------|---------|------|
| **Phase 1** | コンポーネント定義、名称変更、基礎システム | **M (3-7 days)** | 既存パターン踏襲、名称変更は機械的作業 |
| **Phase 2** | Taffyツリー統合、レイアウト計算 | **M (3-7 days)** | 既存システム拡張が主、新規ロジックは限定的 |
| **Phase 3** | 増分更新、モニタ変更対応 | **L (1-2 weeks)** | イベントキュー実装、Windows API統合、複雑な状態管理 |
| **Phase 4** | スケジュール統合、テスト | **M (3-7 days)** | 既存テストパターン活用、統合テストのみ新規 |
| **合計** | | **L-XL (2-3 weeks)** | 段階的実装により、各フェーズは独立して完了可能 |

### 5.2 Risk Assessment

| リスク要因 | リスクレベル | 理由 | 軽減策 |
|-----------|-------------|------|-------|
| **WM_DISPLAYCHANGE統合** | **Low** | App resourceフラグパターンで実装可能（既存パターン踏襲） | Phase 3を独立して実装、app.rs + win_message_handler.rsを参考 |
| **Taffyルート変更の副作用** | **Low** | 既存の明確なAPI、拡張ポイントが存在 | Phase 2で既存テストを全実行、互換性検証 |
| **名称変更の漏れ** | **Low** | 機械的な検索・置換作業 | Grep全検索、コンパイラエラーで検出 |
| **階層整合性のバグ** | **Medium** | Parent/Children関係の複雑な管理 | Common Infrastructureの汎用システム活用、段階的テスト |
| **パフォーマンス劣化** | **Low** | 増分更新で不要な計算を回避 | LayoutDirtyによる最適化、既存のArrangementTreeChangedパターン踏襲 |
| **既存システムの破壊** | **Low** | Hybrid Approachで段階的移行 | 要件9の徹底、各フェーズで既存テスト実行 |

### 5.3 総合評価

**Effort**: **L (1-2 weeks)** - 各フェーズは明確、既存基盤が強固
**Risk**: **Low-Medium** - イベント伝播パターン確立済み、モニター変更対応に設計検討が必要

**根拠**:
- ✅ 既存のTaffy統合、Arrangement伝播システムが成熟
- ✅ Common Infrastructureの汎用パターンが利用可能
- ✅ 明確な統合ポイント（sync_taffy_tree_system, world.rs）
- ⚠️ WM_DISPLAYCHANGE統合とイベントキューは新規実装領域
- ⚠️ モニタ構成変更時の状態管理は複雑

---

## 6. Recommendations for Design Phase

### 6.1 推奨アプローチ

**Option C: Hybrid Approach** を推奨

**理由**:
1. **既存基盤の最大活用**: TaffyLayoutResource, Arrangement伝播システム、Common Infrastructureを拡張
2. **段階的移行**: 各フェーズが独立して機能、破壊的変更を回避
3. **明確な統合ポイント**: 既存のシステムスケジュール、ファイル構造に自然に統合
4. **テスト容易性**: 各フェーズで既存テスト実行、互換性を保証

### 6.2 設計フェーズでの重点事項

#### 6.2.1 Phase 1: 基礎構築
- **優先**: 名称変更（BoxStyle→TaffyStyle）を完全実施
- **設計**: VirtualDesktop/Monitorコンポーネントのフィールド詳細化
- **決定**: `ChildOf` vs `Parent<T>`の選択

#### 6.2.2 Phase 2: Taffyツリー統合
- **設計**: `update_monitor_style_system`の詳細実装
- **設計**: `compute_taffy_layout_system`のルート選択ロジック
- **検証**: 既存テストがすべてパスすることを確認

#### 6.2.3 Phase 3: イベント統合
- **調査完了**: WM_DISPLAYCHANGEイベントキュー実装パターン
- **設計**: `handle_display_change_system`の状態管理
- **設計**: モニタ削除時のWindow再配置戦略

#### 6.2.4 Phase 4: 統合とテスト
- **設計**: システムスケジュールの依存関係グラフ
- **設計**: テストケース詳細（要件10の各項目）

### 6.3 キーデシジョン

| 決定事項 | 推奨 | 代替案 | 影響 |
|---------|------|-------|------|
| **階層API** | `ChildOf` | `Parent<T>` | steeringと一貫性、既存コードとの統合 |
| **ファイル配置** | `crates/wintf/src/ecs/layout/desktop.rs` | 独立モジュール | レイアウトシステムの一部として配置 |
| **イベントキュー** | bevy_ecs::event::Events | カスタム実装 | 調査結果に基づき設計フェーズで決定 |
| **モニタ削除時の動作** | プライマリモニタへ移動 | エンティティ削除 | Windows OS標準動作に準拠 |
| **VirtualDesktop数** | 単一（初期実装）| 複数対応 | 段階的拡張、初期は1つのVirtualDesktop |

### 6.4 次のステップ

1. **設計フェーズ開始**: 上記の推奨事項を反映した詳細設計を作成
2. **Research Items完了**: イベントキュー実装パターンの調査
3. **タスク分解**: Phase 1-4を独立したタスクに分解、依存関係を明確化

---

## 7. Appendix: 要件マッピング詳細

### 要件1: コンポーネント定義とECS統合

**Gap**:
- ❌ `VirtualDesktop`コンポーネント未定義
- ❌ `Monitor`コンポーネント未定義
- ❌ `MonitorInfo`構造体未定義
- ❌ `EnumDisplayMonitors`統合未実装

**Approach**: Option C Phase 1 - 新規コンポーネント作成

### 要件2: エンティティ階層の構築

**Gap**:
- ✅ `ChildOf`/`Children`は既存
- ❌ VirtualDesktop→Monitor→Window階層の構築ロジック未実装
- ❌ `MonitorFromWindow`を使ったWindow→Monitor紐付け未実装

**Approach**: Option C Phase 1 - desktop_system.rs で実装

### 要件3: Taffy スタイルコンポーネントの名称変更

**Gap**:
- ⚠️ `TaffyStyle`/`TaffyComputedLayout`の名称は既に使用されているが、一部に`BoxStyle`の記述が残存している可能性
- ✅ コンポーネント定義自体は完了

**Approach**: Option C Phase 1 - 全ファイルでGrep検索・置換

### 要件4: Taffy ツリーの構築と管理

**Gap**:
- ✅ `TaffyLayoutResource`は実装済み
- ✅ `sync_taffy_tree_system`は実装済み
- ❌ VirtualDesktop/MonitorノードをTaffyツリーに追加するロジック未実装

**Approach**: Option C Phase 2 - 既存システムを拡張

### 要件5: レイアウト計算の実行

**Gap**:
- ✅ `compute_taffy_layout_system`は実装済み
- ❌ VirtualDesktopをルートとするロジック未実装

**Approach**: Option C Phase 2 - ルート選択ロジックを変更

### 要件6: 増分更新とパフォーマンス最適化

**Gap**:
- ✅ `ArrangementTreeChanged`マーカーパターンが存在
- ❌ `LayoutDirty`コンポーネント未定義
- ❌ 変更検知システム未実装

**Approach**: Option C Phase 3 - 新規コンポーネントとシステム

### 要件7: モニタ情報の動的更新

**Gap**:
- ⚠️ `WM_DISPLAYCHANGE`ハンドラは存在するが空実装
- ❌ モニタ列挙/更新ロジック未実装
- ❌ モニタ追加/削除検出未実装

**Approach**: Option C Phase 3 - イベントキュー統合

### 要件8: システムスケジュールの統合

**Gap**:
- ✅ `world.rs`の既存スケジュールは明確
- ❌ 新システムの依存関係定義未実装

**Approach**: Option C Phase 4 - スケジュールに新システムを追加

### 要件9: 既存システムとの互換性維持

**Gap**:
- ✅ 既存システムは安定
- ⚠️ 段階的移行戦略の設計が必要

**Approach**: Option C 全Phase - Hybrid Approachにより保証

### 要件10: テストとバリデーション

**Gap**:
- ✅ 既存テストインフラが充実
- ❌ 新機能用テストケース未作成

**Approach**: Option C Phase 4 - 新規テストファイル作成

---

## 分析完了

本Gap Analysisは、要件定義と既存コードベースの詳細な調査に基づき、**Option C: Hybrid Approach**を強く推奨します。既存の強固なレイアウト基盤を最大限活用し、段階的な移行により互換性を保持しながら、新機能を追加できます。

設計フェーズでは、特に**Phase 3のイベントキュー統合**と**モニタ削除時の再配置戦略**に焦点を当てた詳細設計を行うことを推奨します。
