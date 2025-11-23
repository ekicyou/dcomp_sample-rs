//! **⚠️ 非推奨: このモジュールは実験的機能です**
//!
//! # 概要
//! `Transform`コンポーネント群は、WinUI3/WPF/XAMLの`RenderTransform`プロパティを模倣した
//! 実験的な変換システムです。現在は**非推奨**となっており、将来的に削除される可能性があります。
//!
//! # 非推奨理由
//! - **WinUI3模倣の限界**: WinUI3のRenderTransformは回転・スキュー変換をサポートしますが、
//!   Direct2D/DirectCompositionレイヤーでの実装が複雑化します。
//! - **レイアウトエンジンとの統合不足**: taffyレイアウトエンジンと`Transform`の統合が不十分で、
//!   レイアウト計算後の手動変換が必要です。
//! - **パフォーマンス**: 回転・スキュー変換を含む場合、軸平行矩形最適化が適用できません。
//!
//! # 推奨される代替手段
//! **`Arrangement`ベースのレイアウトシステム**を使用してください。
//!
//! ## Arrangementの利点
//! - **taffyレイアウトエンジン統合**: `TaffyStyle`で宣言的レイアウトを記述し、自動計算されます。
//! - **軸平行変換最適化**: 平行移動とスケールのみをサポートし、O(2)の高速変換を実現します。
//! - **階層伝播システム**: `GlobalArrangement`で親からの累積変換が自動的に伝播されます。
//! - **Surface生成最適化**: バウンディングボックス集約により最小限のSurfaceサイズを計算します。
//!
//! ## Migration Guide
//! ### 変換前 (Transform)
//! ```rust,ignore
//! commands.spawn((
//!     Transform {
//!         translate: Some(Translate { x: 100.0, y: 50.0 }),
//!         scale: Some(Scale { x: 1.5, y: 1.5 }),
//!         ..Default::default()
//!     },
//!     GlobalTransform::default(),
//! ));
//! ```
//!
//! ### 変換後 (Arrangement)
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
//! ### taffyレイアウトエンジンとの統合
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
//! # 将来の対応
//! - **Direct2D Visual層での回転・スキュー**: DirectComposition Visualの変換行列プロパティを使用予定
//! - **現在のTransform**: 将来的に削除または大幅に変更される可能性があります
//!
//! # 使用上の注意
//! このモジュールを使用する場合、以下の制約に注意してください：
//! - 回転・スキュー変換は`transform_rect_axis_aligned`で正しく処理されません
//! - taffyレイアウトエンジンとの統合が必要な場合は`Arrangement`を使用してください
//! - 新規開発では`Arrangement`ベースのレイアウトシステムを推奨します

pub mod components;

pub use components::*;
