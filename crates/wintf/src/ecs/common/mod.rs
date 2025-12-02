//! # Common Infrastructure - ECS階層システムの汎用的な伝播ロジック
//!
//! このモジュールは、ECS階層システム（`ChildOf`, `Children`）における変換伝播の
//! 汎用的なアルゴリズムを提供します。
//!
//! ## 主要機能
//!
//! ### ジェネリック階層伝播
//!
//! 3つのジェネリック型パラメータで、さまざまな変換タイプに対応：
//!
//! - **`L`**: ローカル変換コンポーネント (例: `Arrangement`, `Transform`)
//! - **`G`**: グローバル変換コンポーネント (例: `GlobalArrangement`, `GlobalTransform`)
//! - **`M`**: 変更マーカーコンポーネント (例: `ArrangementTreeChanged`, `TransformTreeChanged`)
//!
//! ### 3つの中核システム関数
//!
//! #### 1. `sync_simple_transforms<L, G, M>()`
//! 階層に属していないエンティティ（ルート）のグローバル変換を更新します。
//!
//! ```rust,ignore
//! // Layout Systemでの使用例
//! sync_simple_transforms::<Arrangement, GlobalArrangement, ArrangementTreeChanged>(
//!     query, orphaned
//! );
//! ```
//!
//! #### 2. `mark_dirty_trees<L, G, M>()`
//! 変更されたエンティティから祖先に向かってダーティビットを伝播します。
//! これにより、影響を受けるサブツリーのみを効率的に更新できます.
//!
//! ```rust,ignore
//! mark_dirty_trees::<Arrangement, GlobalArrangement, ArrangementTreeChanged>(
//!     changed, orphaned, transforms
//! );
//! ```
//!
//! #### 3. `propagate_parent_transforms<L, G, M>()`
//! 親から子へグローバル変換を伝播します。幅優先探索により階層を走査し、
//! 親のグローバル変換と子のローカル変換を積算します。
//!
//! ```rust,ignore
//! propagate_parent_transforms::<Arrangement, GlobalArrangement, ArrangementTreeChanged>(
//!     queue, roots, nodes
//! );
//! ```
//!
//! ## 階層走査イテレータ
//!
//! ### DepthFirstReversePostOrder
//! 深さ優先・逆順・後順走査イテレータ。ヒットテスト（最前面優先）や
//! フォーカス管理など、Z順序を考慮した処理に使用します。
//!
//! ```rust,ignore
//! use wintf::ecs::common::DepthFirstReversePostOrder;
//!
//! for entity in DepthFirstReversePostOrder::new(root, &children_query) {
//!     if hit_test_entity(world, entity, point) {
//!         return Some(entity);
//!     }
//! }
//! ```
//!
//! ## パフォーマンス最適化
//!
//! - **Changed検知**: bevy_ecsの`Changed<T>`フィルタで変更されたエンティティのみ処理
//! - **ダーティビット伝播**: 影響を受けるサブツリーのみを更新
//! - **幅優先探索**: `WorkQueue`を使用した効率的な階層走査
//! - **不要な伝播の回避**: `Ref<T>::is_changed()`で更新が必要なノードのみ処理
//!
//! ## Trait制約
//!
//! 型パラメータには以下の制約が必要です：
//!
//! - **`L`**: `Component + Copy + Into<G>`
//! - **`G`**: `Component<Mutability = Mutable> + Copy + PartialEq + Mul<L, Output = G>`
//! - **`M`**: `Component<Mutability = Mutable> + Copy`
//!
//! `Mul<L, Output = G>`により、親のグローバル変換と子のローカル変換を積算できます。

pub mod tree_iter;
pub mod tree_system;

pub use tree_iter::*;
pub use tree_system::*;
