use crate::dpi::*;
use bevy_ecs::prelude::*;
use euclid::*;
use taffy::prelude::*;

/// DPI変換コンポーネント
/// スパースなので SparseSet ストレージを使用
#[derive(Component, Debug, Clone, Copy, PartialEq, Default)]
#[repr(transparent)]
#[component(storage = "SparseSet")]
pub struct DpiTransform(pub Transform2D<f32, Lx, Px>);

impl DpiTransform {
    /// 新規作成
    pub fn identity() -> Self {
        Self(Transform2D::identity())
    }
}

/// taffyのStyle
#[derive(Component, Debug, Clone, PartialEq, Default)]
pub struct BoxStyle(pub Style);

// SAFETY: Styleは実際には安全に送受信できる
// 内部のポインタは特定のケースでのみ使用される
unsafe impl Send for BoxStyle {}
unsafe impl Sync for BoxStyle {}

impl BoxStyle {
    pub fn new(style: Style) -> Self {
        Self(style)
    }
}

/// レイアウト計算結果
#[derive(Component, Debug, Clone, PartialEq, Default)]
pub struct BoxComputedLayout(pub Layout);
