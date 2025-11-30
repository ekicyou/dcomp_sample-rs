//! BitmapSource ウィジェットモジュール
//!
//! WIC画像読み込みとD2D描画を提供する。
//! 非同期読み込みにより、WorldをブロックせずUIの応答性を維持する。

mod bitmap_source;
mod resource;
pub mod systems;
mod task_pool;
mod wic_core;

pub use bitmap_source::BitmapSource;
pub use resource::{BitmapSourceGraphics, BitmapSourceResource};
pub use systems::draw_bitmap_sources;
pub use task_pool::{BoxedCommand, CommandSender, WintfTaskPool};
pub use wic_core::WicCore;

#[cfg(test)]
mod tests;
