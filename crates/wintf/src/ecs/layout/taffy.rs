use bevy_ecs::prelude::*;
use taffy::prelude::*;

/// taffyのStyle
#[derive(Component, Debug, Clone, PartialEq, Default)]
#[repr(transparent)]
pub struct TaffyStyle(pub(crate) Style);

unsafe impl Send for TaffyStyle {}
unsafe impl Sync for TaffyStyle {}

impl TaffyStyle {
    pub fn new(style: Style) -> Self {
        Self(style)
    }
}

/// レイアウト計算結果
#[derive(Component, Debug, Clone, PartialEq, Copy, Default)]
#[repr(transparent)]
pub struct TaffyComputedLayout(pub(crate) Layout);
