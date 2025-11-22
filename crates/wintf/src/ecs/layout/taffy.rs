use bevy_ecs::prelude::*;
use taffy::prelude::*;

/// taffyのStyle
#[derive(Component, Debug, Clone, PartialEq, Default)]
pub struct BoxStyle(pub Style);

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
