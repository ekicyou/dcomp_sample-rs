//! # Layout System - taffyレイアウトエンジン統合と配置計算
//!
//! このモジュールは、taffyレイアウトエンジンとの統合、および階層的な配置計算を提供します。
//!
//! ## サブモジュール構成
//!
//! - **`taffy`**: taffyエンジン連携 (`TaffyStyle`, `TaffyComputedLayout`)
//! - **`metrics`**: レイアウトメトリクス (`Size`, `Offset`, `LayoutScale`, `TextLayoutMetrics`)
//! - **`arrangement`**: 配置情報コンポーネント (`Arrangement`, `GlobalArrangement`, `ArrangementTreeChanged`)
//! - **`rect`**: 矩形操作ユーティリティ (`Rect`, `D2DRectExt`, `transform_rect_axis_aligned`)
//! - **`systems`**: 配置伝播システム関数 (`sync_simple_arrangements`, `propagate_global_arrangements`)
//!
//! ## 主要コンポーネント
//!
//! ### Arrangement - ローカル配置
//! 親エンティティからの相対的な位置・スケール・サイズを保持します。
//!
//! ```rust,ignore
//! use wintf::ecs::*;
//!
//! commands.spawn((
//!     Arrangement {
//!         offset: Offset { x: 100.0, y: 50.0 },
//!         scale: LayoutScale { x: 1.5, y: 1.5 },
//!         size: Size { width: 200.0, height: 100.0 },
//!     },
//!     // GlobalArrangementは自動的に追加されます（on_add hook）
//! ));
//! ```
//!
//! ### GlobalArrangement - グローバル累積変換
//! 親からの累積変換行列とワールド座標系でのバウンディングボックスを保持します。
//! `propagate_global_arrangements`システムにより自動的に伝播されます。
//!
//! ## taffyレイアウトエンジン統合
//!
//! `TaffyStyle`でflexboxレイアウトを宣言的に記述できます：
//!
//! ```rust,ignore
//! use wintf::ecs::*;
//! use taffy::prelude::*;
//!
//! commands.spawn((
//!     TaffyStyle::new(Style {
//!         size: Size { width: length(200.0), height: length(100.0) },
//!         padding: Rect { left: length(10.0), right: length(10.0), top: length(5.0), bottom: length(5.0) },
//!         ..Default::default()
//!     }),
//!     TaffyComputedLayout::default(), // taffyが自動計算
//! ));
//! ```
//!
//! The Layout System supports only translation and scaling, optimized for axis-aligned transformations.
//! The transform_rect_axis_aligned function transforms only the top-left and bottom-right points
//! instead of all four points, achieving O(2) fast rectangle transformation.
//!
//! Constraint: Rotation and skew transformations are not supported. They will be handled by the
//! DirectComposition Visual layer in the future.
//!
//! The arrangement propagation systems in the systems module leverage generic functions from
//! ecs::common::tree_system: sync_simple_transforms, mark_dirty_trees, and propagate_parent_transforms.

// Layout System サブモジュール
pub mod arrangement;
pub mod high_level;
pub mod metrics;
pub mod rect;
pub mod systems;
pub mod taffy;

// 公開API
pub use arrangement::*;
pub use high_level::*;
pub use metrics::*;
pub use rect::*; // D2DRect, D2DRectExt, transform_rect_axis_aligned
pub use systems::*;
pub use taffy::*;

use bevy_ecs::prelude::*;

/// レイアウト計算のルートを示すマーカーコンポーネント
///
/// このコンポーネントが付与されたエンティティは、Taffyレイアウト計算のルートとして扱われます。
/// 通常、仮想デスクトップ（VirtualDesktop）またはテスト用のルートエンティティに付与されます。
///
/// # 設計意図
///
/// - `Window`コンポーネントはレイアウトの対象であり、ルートではありません
/// - 真のルート階層: `VirtualDesktop → Monitor → Window → Widget`
/// - 現在の実装では仮想デスクトップ未実装のため、暫定的にルートマーカーを使用
///
/// # ストレージ戦略
///
/// `SparseSet`を使用します。このコンポーネントは通常1つのエンティティにのみ付与されるため、
/// メモリ効率の良い疎行列ストレージが適しています。
///
/// # 使用例
///
/// ```rust,ignore
/// use wintf::ecs::layout::LayoutRoot;
///
/// // テスト用のルートエンティティ
/// commands.spawn((
///     LayoutRoot,
///     BoxSize { width: Some(Dimension::Px(800.0)), height: Some(Dimension::Px(600.0)) },
///     FlexContainer::default(),
/// ));
/// ```
/// LayoutRootコンポーネント
///
/// # ライフタイムイベント
/// - `on_add`: `Arrangement::default()`を自動挿入
///   - これにより`Arrangement`の`on_add`が連鎖的に`GlobalArrangement`と`ArrangementTreeChanged`を挿入
#[derive(Component)]
#[component(storage = "SparseSet", on_add = on_layout_root_add)]
pub struct LayoutRoot;

/// LayoutRootコンポーネントが追加されたときに呼ばれるフック
/// Arrangementを自動挿入する（既に存在する場合はスキップ）
fn on_layout_root_add(mut world: bevy_ecs::world::DeferredWorld, context: bevy_ecs::lifecycle::HookContext) {
    let entity = context.entity;
    // Arrangementがまだ存在しない場合のみ挿入
    if world.get::<Arrangement>(entity).is_none() {
        world
            .commands()
            .entity(entity)
            .insert(Arrangement::default());
    }
}
