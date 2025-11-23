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
//! ```
//!
//! ## 軸平行変換最適化
//!
//! Layout Systemは平行移動とスケールのみをサポートし、軸平行変換に最適化されています。
//! `transform_rect_axis_aligned`関数は、4点すべてを変換する代わりに左上と右下の2点のみを変換することで、
//! O(2)の高速な矩形変換を実現します。
//!
//! **制約**: 回転・スキュー変換には対応していません。将来的にDirectComposition Visual層で対応予定です。
//!
//! ## Common Infrastructureとの連携
//!
//! `systems`モジュールの配置伝播システムは、`ecs::common::tree_system`の汎用関数を活用しています：
//!
//! - `sync_simple_transforms<Arrangement, GlobalArrangement, ArrangementTreeChanged>()`
//! - `mark_dirty_trees<Arrangement, GlobalArrangement, ArrangementTreeChanged>()`
//! - `propagate_parent_transforms<Arrangement, GlobalArrangement, ArrangementTreeChanged>()`

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
